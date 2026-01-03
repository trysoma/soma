use crate::error::CommonError;

// BASED ON https://github.com/restatedev/cdk/blob/main/lib/restate-constructs/register-service-handler/index.mts
// AND https://github.com/restatedev/cdk/blob/main/lib/restate-constructs/register-service-handler/entrypoint.mts
use super::admin_client::AdminClient;
use super::admin_interface::AdminClientInterface;
use anyhow::anyhow;
use http::{HeaderName, HeaderValue, Uri};
use restate_admin_rest_model::deployments::RegisterDeploymentRequest;
use restate_admin_rest_model::services::ModifyServiceRequest;
use restate_serde_util::SerdeableHeaderHashMap;
use restate_types::identifiers::LambdaARN;
use restate_types::schema::service::ServiceMetadata;
use std::fmt::Formatter;
use std::str::FromStr;
use std::time::Duration;
use std::{collections::HashMap, fmt::Display};
use tokio::time::sleep;
use tracing::{debug, trace, warn};
use url::Url;

/// Deployment type configuration
#[derive(Debug, Clone)]
pub enum DeploymentType {
    /// Lambda deployment
    Lambda {
        /// The Lambda ARN for the service
        arn: String,
        /// Optional assume role ARN for invoking the Lambda
        assume_role_arn: Option<String>,
    },
    /// HTTP deployment
    Http {
        /// The URI of the HTTP service endpoint
        uri: String,
        /// Additional HTTP headers to include in requests
        additional_headers: HashMap<String, String>,
    },
}

impl Display for DeploymentType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DeploymentType::Lambda { arn, .. } => write!(f, "Lambda ARN: {arn}"),
            DeploymentType::Http { uri, .. } => write!(f, "HTTP URI: {uri}"),
        }
    }
}

/// Configuration for registering a Restate deployment
#[derive(Debug, Clone)]
pub struct DeploymentRegistrationConfig {
    /// The admin URL of the Restate server
    pub admin_url: String,
    /// The service path to register
    pub service_path: String,
    /// The deployment type (Lambda or HTTP)
    pub deployment_type: DeploymentType,
    /// Optional bearer token for authentication
    pub bearer_token: Option<String>,
    /// Whether the service should be private
    pub private: bool,
    /// Whether to skip TLS verification (insecure)
    pub insecure: bool,
    /// Whether to force registration even if it already exists
    pub force: bool,
}

/// Registers a deployment with Restate with retry logic
pub async fn register_deployment(
    config: DeploymentRegistrationConfig,
) -> Result<ServiceMetadata, CommonError> {
    // Wait for Restate admin to be healthy
    wait_for_healthy_restate_admin(&config.admin_url).await?;

    // For HTTP deployments, also check if the service endpoint is healthy
    // This prevents Restate from trying to discover endpoints before the service is ready
    if let DeploymentType::Http { uri, .. } = &config.deployment_type {
        wait_for_healthy_http_service(uri).await?;
    }

    // Register the deployment with retry
    let service_metadata = register_deployment_with_retry(&config).await?;

    debug!(
        service = %service_metadata.name,
        deployment = %config.deployment_type,
        "Registered Restate deployment"
    );

    Ok(service_metadata)
}

/// Wait for an HTTP service endpoint to be healthy with exponential backoff
/// This is a lightweight check - we just verify the service is listening and can accept connections.
/// Since the SDK server uses HTTP/2, we use a TCP connection check which works regardless of HTTP version.
/// Restate will handle the actual endpoint discovery after registration.
async fn wait_for_healthy_http_service(uri: &str) -> Result<(), CommonError> {
    const MAX_HEALTH_CHECK_ATTEMPTS: u32 = 5;
    const INITIAL_BACKOFF_MS: u64 = 500;
    const CONNECT_TIMEOUT_SECS: u64 = 2;

    trace!(uri = %uri, "Checking HTTP service connectivity");

    // Parse the URI to extract host and port
    let url = Url::parse(uri)
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Invalid URI '{uri}': {e}")))?;

    let host = url
        .host_str()
        .ok_or_else(|| CommonError::Unknown(anyhow::anyhow!("URI '{uri}' has no host")))?;
    let port = url
        .port()
        .or_else(|| {
            // Default ports based on scheme
            match url.scheme() {
                "http" => Some(80),
                "https" => Some(443),
                _ => None,
            }
        })
        .ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!("Could not determine port for URI '{uri}'"))
        })?;

    for attempt in 0..MAX_HEALTH_CHECK_ATTEMPTS {
        // Use a TCP connection check - this works for HTTP/1.1, HTTP/2, or any protocol
        match tokio::time::timeout(
            Duration::from_secs(CONNECT_TIMEOUT_SECS),
            tokio::net::TcpStream::connect((host, port)),
        )
        .await
        {
            Ok(Ok(_stream)) => {
                trace!(uri = %uri, "HTTP service accepting connections");
                return Ok(());
            }
            Ok(Err(e)) => {
                if attempt < MAX_HEALTH_CHECK_ATTEMPTS - 1 {
                    let backoff_ms = INITIAL_BACKOFF_MS * 2u64.pow(attempt.min(3));
                    trace!(uri = %uri, attempt = attempt + 1, error = %e, backoff_ms, "HTTP service not ready, retrying");
                    sleep(Duration::from_millis(backoff_ms)).await;
                } else {
                    debug!(uri = %uri, "HTTP service not ready after retries, proceeding anyway");
                    return Ok(());
                }
            }
            Err(_) => {
                if attempt < MAX_HEALTH_CHECK_ATTEMPTS - 1 {
                    let backoff_ms = INITIAL_BACKOFF_MS * 2u64.pow(attempt.min(3));
                    trace!(uri = %uri, attempt = attempt + 1, backoff_ms, "HTTP service connection timeout, retrying");
                    sleep(Duration::from_millis(backoff_ms)).await;
                } else {
                    debug!(uri = %uri, "HTTP service connection timeout after retries, proceeding anyway");
                    return Ok(());
                }
            }
        }
    }

    debug!(uri = %uri, "HTTP service health check inconclusive, proceeding");
    Ok(())
}

/// Wait for the Restate admin endpoint to be healthy with exponential backoff
pub async fn wait_for_healthy_restate_admin(admin_url: &str) -> Result<(), CommonError> {
    const MAX_HEALTH_CHECK_ATTEMPTS: u32 = 10;
    const INITIAL_BACKOFF_MS: u64 = 1000;

    debug!(admin_url = %admin_url, "Checking Restate admin health");

    for attempt in 0..MAX_HEALTH_CHECK_ATTEMPTS {
        let base_url = Url::parse(admin_url).map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!("Invalid admin URL '{admin_url}': {e}"))
        })?;

        match AdminClient::new(base_url, None).await {
            Ok(client) => match client.ensure_healthy().await {
                Ok(_) => {
                    trace!(admin_url = %admin_url, "Restate admin healthy");
                    return Ok(());
                }
                Err(e) => {
                    trace!(attempt = attempt + 1, error = ?e, "Restate admin health check failed");
                }
            },
            Err(e) => {
                trace!(attempt = attempt + 1, error = ?e, "Failed to create Restate admin client");
            }
        }

        if attempt < MAX_HEALTH_CHECK_ATTEMPTS - 1 {
            let backoff_ms = INITIAL_BACKOFF_MS * 2u64.pow(attempt);
            trace!(backoff_ms, "Waiting before next health check");
            sleep(Duration::from_millis(backoff_ms)).await;
        }
    }

    Err(CommonError::Unknown(anyhow::anyhow!(
        "Restate admin at {admin_url} did not become healthy after {MAX_HEALTH_CHECK_ATTEMPTS} attempts"
    )))
}

/// Register the deployment with retry logic
async fn register_deployment_with_retry(
    config: &DeploymentRegistrationConfig,
) -> Result<ServiceMetadata, CommonError> {
    const MAX_REGISTRATION_ATTEMPTS: u32 = 3;
    const REGISTRATION_BACKOFF_MS: u64 = 2000;

    let base_url = Url::parse(&config.admin_url).map_err(|e| {
        CommonError::Unknown(anyhow::anyhow!(
            "Invalid admin URL '{}': {}",
            config.admin_url,
            e
        ))
    })?;

    let client = AdminClient::new(base_url, config.bearer_token.clone()).await?;
    client.ensure_healthy().await?;

    for attempt in 0..MAX_REGISTRATION_ATTEMPTS {
        debug!(
            deployment = %config.deployment_type,
            attempt = attempt + 1,
            "Registering deployment"
        );

        match try_register_deployment(&client, config).await {
            Ok(service_metadata) => {
                return Ok(service_metadata);
            }
            Err(e) => {
                if attempt < MAX_REGISTRATION_ATTEMPTS - 1 {
                    warn!(
                        attempt = attempt + 1,
                        error = ?e,
                        "Registration failed, retrying"
                    );
                    sleep(Duration::from_millis(REGISTRATION_BACKOFF_MS)).await;
                } else {
                    return Err(CommonError::Unknown(anyhow::anyhow!(
                        "Failed to register deployment after {MAX_REGISTRATION_ATTEMPTS} attempts: {e:?}"
                    )));
                }
            }
        }
    }

    Err(CommonError::Unknown(anyhow::anyhow!(
        "Failed to register deployment after {MAX_REGISTRATION_ATTEMPTS} attempts"
    )))
}

/// Try to register the deployment once
async fn try_register_deployment(
    client: &AdminClient,
    config: &DeploymentRegistrationConfig,
) -> Result<ServiceMetadata, CommonError> {
    // Create the registration request based on deployment type
    let register_request = match &config.deployment_type {
        DeploymentType::Lambda {
            arn,
            assume_role_arn,
        } => {
            // Parse and validate the Lambda ARN
            let lambda_arn = LambdaARN::from_str(arn)
                .map_err(|e| anyhow!("Invalid Lambda ARN '{arn}': {e:?}"))?;

            trace!(arn = %arn, "Preparing Lambda deployment request");

            RegisterDeploymentRequest::Lambda {
                arn: lambda_arn.to_string(),
                assume_role_arn: assume_role_arn.clone(),
                additional_headers: Default::default(),
                force: Some(config.force),
                dry_run: false,
                metadata: Default::default(),
                breaking: false,
            }
        }
        DeploymentType::Http {
            uri,
            additional_headers,
        } => {
            // Parse and validate the HTTP URI
            let parsed_uri = uri
                .parse::<Uri>()
                .map_err(|e| anyhow!("Invalid HTTP URI '{uri}': {e}"))?;

            trace!(uri = %uri, "Preparing HTTP deployment request");

            // Convert HashMap<String, String> to HashMap<HeaderName, HeaderValue>
            let headers = if additional_headers.is_empty() {
                None
            } else {
                let mut header_map: HashMap<HeaderName, HeaderValue> = HashMap::new();
                for (key, value) in additional_headers {
                    let header_name = key
                        .parse::<HeaderName>()
                        .map_err(|e| anyhow!("Invalid header name '{key}': {e}"))?;
                    let header_value = HeaderValue::from_str(value)
                        .map_err(|e| anyhow!("Invalid header value for '{key}': {e}"))?;
                    header_map.insert(header_name, header_value);
                }
                Some(SerdeableHeaderHashMap::from(header_map))
            };

            RegisterDeploymentRequest::Http {
                uri: parsed_uri,
                additional_headers: headers,
                force: Some(config.force),
                dry_run: false,
                use_http_11: false,
                metadata: Default::default(),
                breaking: false,
            }
        }
    };

    trace!(request = ?register_request, "Sending registration request");

    // Discover/register the deployment
    let register_response = client
        .discover_deployment(register_request)
        .await
        .map_err(|e| anyhow!("Failed to discover deployment: {e:?}"))?;

    let deployment_response = register_response
        .into_body()
        .await
        .map_err(|e| anyhow!("Failed to parse deployment response: {e:?}"))?;

    trace!(
        service_count = deployment_response.services.len(),
        "Deployment registered"
    );

    // Find the service we're looking for
    let service_name = deployment_response
        .services
        .iter()
        .find(|s| s.name.as_str() == config.service_path)
        .map(|s| s.name.as_str())
        .ok_or_else(|| {
            anyhow!(
                "Service '{}' not found in deployment. Available services: {:?}",
                config.service_path,
                deployment_response
                    .services
                    .iter()
                    .map(|s| s.name.as_str())
                    .collect::<Vec<_>>()
            )
        })?;

    // Update the service visibility if needed
    if config.private {
        trace!(service = %service_name, "Marking service as private");
        let modify_request = ModifyServiceRequest {
            public: Some(false),
            idempotency_retention: None,
            workflow_completion_retention: None,
            inactivity_timeout: None,
            abort_timeout: None,
            journal_retention: None,
        };

        let service_response = client
            .patch_service(service_name, modify_request)
            .await
            .map_err(|e| anyhow!("Failed to mark service as private: {e:?}"))?;

        let service_metadata = service_response
            .into_body()
            .await
            .map_err(|e| anyhow!("Failed to parse service response: {e:?}"))?;

        Ok(service_metadata)
    } else {
        trace!(service = %service_name, "Service remains public");

        // Fetch the service metadata to return
        let service_response = client
            .get_service(service_name)
            .await
            .map_err(|e| anyhow!("Failed to get service metadata: {e:?}"))?;

        let service_metadata = service_response
            .into_body()
            .await
            .map_err(|e| anyhow!("Failed to parse service metadata: {e:?}"))?;

        Ok(service_metadata)
    }
}

#[cfg(test)]
mod tests {
    mod unit {
        use super::super::*;

        #[test]
        fn test_lambda_deployment_config_creation() {
            let config = DeploymentRegistrationConfig {
                admin_url: "http://localhost:8080".to_string(),
                service_path: "my-service".to_string(),
                deployment_type: DeploymentType::Lambda {
                    arn: "arn:aws:lambda:us-east-1:123456789012:function:my-function:$LATEST"
                        .to_string(),
                    assume_role_arn: Some("arn:aws:iam::123456789012:role/my-role".to_string()),
                },
                bearer_token: Some("my-token".to_string()),
                private: false,
                insecure: false,
                force: true,
            };

            assert_eq!(config.admin_url, "http://localhost:8080");
            assert_eq!(config.service_path, "my-service");
            assert!(!config.private);
            assert!(config.force);
        }

        #[test]
        fn test_http_deployment_config_creation() {
            let mut headers = HashMap::new();
            headers.insert("x-custom-header".to_string(), "value".to_string());

            let config = DeploymentRegistrationConfig {
                admin_url: "http://localhost:8080".to_string(),
                service_path: "my-http-service".to_string(),
                deployment_type: DeploymentType::Http {
                    uri: "http://localhost:9080".to_string(),
                    additional_headers: headers.clone(),
                },
                bearer_token: None,
                private: true,
                insecure: false,
                force: false,
            };

            assert_eq!(config.admin_url, "http://localhost:8080");
            assert_eq!(config.service_path, "my-http-service");
            assert!(config.private);
            assert!(!config.force);

            if let DeploymentType::Http {
                uri,
                additional_headers,
            } = &config.deployment_type
            {
                assert_eq!(uri, "http://localhost:9080");
                assert_eq!(additional_headers.len(), 1);
                assert_eq!(
                    additional_headers.get("x-custom-header"),
                    Some(&"value".to_string())
                );
            } else {
                panic!("Expected HTTP deployment type");
            }
        }

        #[test]
        fn test_lambda_arn_parsing() {
            // LambdaARN requires a version or alias suffix
            // Testing with a versioned ARN
            let valid_arn = "arn:aws:lambda:us-east-1:123456789012:function:my-function:$LATEST";
            let result = LambdaARN::from_str(valid_arn);
            assert!(
                result.is_ok(),
                "Lambda ARN parsing failed: {:?}",
                result.err()
            );
        }

        #[test]
        fn test_http_uri_parsing() {
            let valid_uri = "http://localhost:9080";
            let result = Url::parse(valid_uri);
            assert!(
                result.is_ok(),
                "HTTP URI parsing failed: {:?}",
                result.err()
            );

            let valid_https_uri = "https://my-service.example.com:8080/path";
            let result = Url::parse(valid_https_uri);
            assert!(
                result.is_ok(),
                "HTTPS URI parsing failed: {:?}",
                result.err()
            );
        }

        #[test]
        fn test_deployment_type_variants() {
            // Test Lambda variant
            let lambda = DeploymentType::Lambda {
                arn: "arn:aws:lambda:us-east-1:123456789012:function:test:$LATEST".to_string(),
                assume_role_arn: None,
            };

            match lambda {
                DeploymentType::Lambda { arn, .. } => {
                    assert!(arn.contains("test"));
                }
                _ => panic!("Expected Lambda variant"),
            }

            // Test HTTP variant
            let http = DeploymentType::Http {
                uri: "http://localhost:8080".to_string(),
                additional_headers: HashMap::new(),
            };

            match http {
                DeploymentType::Http { uri, .. } => {
                    assert_eq!(uri, "http://localhost:8080");
                }
                _ => panic!("Expected HTTP variant"),
            }
        }
    }
}
