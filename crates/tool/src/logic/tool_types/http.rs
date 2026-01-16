use reqwest::Client;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use shared::error::CommonError;
use shared::primitives::{PaginatedResponse, PaginationRequest, WrappedJsonValue};
use utoipa::IntoParams;

use crate::logic::{DecryptionService, InvokeResult};

use super::{HttpEndpointConfiguration, ToolSerialized};

/// Invoke an HTTP tool with the given parameters and credentials
///
/// This function:
/// 1. Decrypts the endpoint configuration to get the URL and invocation key
/// 2. Makes a POST request to the tool's HTTP endpoint
/// 3. Includes decrypted static credentials, resource server credentials, and user credentials in the request body
/// 4. Returns the result or error
pub async fn invoke_http_tool(
    decryption_service: &DecryptionService,
    tool: &ToolSerialized,
    static_credentials: Option<WrappedJsonValue>,
    resource_server_credential: Option<WrappedJsonValue>,
    user_credential: Option<WrappedJsonValue>,
    params: WrappedJsonValue,
) -> Result<InvokeResult, CommonError> {
    // Decrypt the endpoint configuration
    let config = decrypt_http_endpoint_config(decryption_service, &tool.endpoint_configuration).await?;

    // Build the request payload with credentials and parameters
    let request_body = json!({
        "params": params.get_inner(),
        "static_credentials": static_credentials.map(|c| c.into_inner()),
        "resource_server_credential": resource_server_credential.map(|c| c.into_inner()),
        "user_credential": user_credential.map(|c| c.into_inner()),
    });

    // Create HTTP client
    let client = Client::new();

    // Make the HTTP request
    let response = client
        .post(&config.url)
        .header("Authorization", format!("Bearer {}", config.invocation_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("HTTP request failed: {}", e)))?;

    // Get status before consuming response
    let status = response.status();

    // Check response status
    if status.is_success() {
        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to parse response: {}", e)))?;

        Ok(InvokeResult::Success(WrappedJsonValue::new(result)))
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());

        Ok(InvokeResult::Error(crate::logic::InvokeError {
            message: format!("HTTP request failed with status {}: {}", status, error_text),
        }))
    }
}

/// Encrypt HTTP endpoint configuration for storage
///
/// This function encrypts the invocation_key field and stores the configuration
/// as a JSON object with the url and encrypted invocation_key
pub async fn encrypt_http_endpoint_config(
    crypto_cache: &encryption::logic::CryptoCache,
    dek_alias: &str,
    config: &HttpEndpointConfiguration,
) -> Result<WrappedJsonValue, CommonError> {
    // Get encryption service for the specified DEK alias
    let encryption_service = crypto_cache.get_encryption_service(dek_alias).await?;

    // Encrypt the invocation key
    let encrypted_key = encryption_service
        .encrypt_data(config.invocation_key.clone())
        .await?;

    // Create the encrypted configuration JSON
    let encrypted_config = json!({
        "url": config.url,
        "invocation_key": encrypted_key.0,  // Extract the String from EncryptedString
    });

    Ok(WrappedJsonValue::new(encrypted_config))
}

/// Decrypt HTTP endpoint configuration from storage
///
/// This function decrypts the invocation_key field from the stored JSON configuration
pub async fn decrypt_http_endpoint_config(
    decryption_service: &DecryptionService,
    encrypted_config: &WrappedJsonValue,
) -> Result<HttpEndpointConfiguration, CommonError> {
    let config_obj = encrypted_config.get_inner().as_object().ok_or_else(|| {
        CommonError::Unknown(anyhow::anyhow!("Endpoint configuration is not a JSON object"))
    })?;

    // Extract URL
    let url = config_obj
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CommonError::Unknown(anyhow::anyhow!("Missing 'url' in endpoint configuration")))?
        .to_string();

    // Extract the encrypted invocation key
    let encrypted_key = config_obj
        .get("invocation_key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CommonError::Unknown(anyhow::anyhow!("Missing 'invocation_key' in endpoint configuration")))?
        .to_string();

    // Decrypt the invocation key
    let invocation_key = decryption_service
        .decrypt_data(encryption::logic::EncryptedString(encrypted_key))
        .await?;

    Ok(HttpEndpointConfiguration {
        url,
        invocation_key,
    })
}

/// Register a new HTTP-based tool
///
/// This function:
/// 1. Encrypts the tool's endpoint configuration using the "local" DEK alias
/// 2. Creates a ToolSerialized object with all metadata
/// 3. Stores the tool in the repository
///
/// # Parameters
/// - `repo`: Repository for persisting the tool
/// - `crypto_cache`: Encryption service for encrypting the invocation key
/// - `request`: Tool registration request containing name, documentation, endpoint config, etc.
///
/// # Returns
/// The registered tool with its encrypted endpoint configuration
///
/// # Errors
/// - Returns error if encryption fails
/// - Returns error if database save fails
#[shared_macros::authz_role(Admin, permission = "tool:write")]
#[shared_macros::authn]
pub async fn register_tool(
    repo: &impl crate::repository::ProviderRepositoryLike,
    crypto_cache: &encryption::logic::CryptoCache,
    request: super::RegisterToolRequest,
) -> Result<super::RegisterToolResponse, CommonError> {
    use shared::primitives::WrappedChronoDateTime;
    use tracing::trace;

    trace!(
        type_id = %request.type_id,
        deployment_id = %request.deployment_id,
        "Registering tool with encrypted endpoint configuration"
    );

    let now = WrappedChronoDateTime::now();

    // Encrypt the endpoint configuration using the "local" DEK alias
    let encrypted_config =
        encrypt_http_endpoint_config(crypto_cache, "local", &request.endpoint_configuration).await?;

    let tool = ToolSerialized {
        type_id: request.type_id.clone(),
        deployment_id: request.deployment_id.clone(),
        name: request.name,
        documentation: request.documentation,
        categories: request.categories,
        endpoint_type: super::EndpointType::Http,
        endpoint_configuration: encrypted_config,
        metadata: request
            .metadata
            .unwrap_or_else(crate::logic::Metadata::new),
        created_at: now.clone(),
        updated_at: now,
    };

    // Store in repository
    let create_params = crate::repository::CreateTool::from(tool.clone());
    repo.create_tool(&create_params).await?;

    trace!(
        type_id = %tool.type_id,
        deployment_id = %tool.deployment_id,
        "Tool registered successfully"
    );

    Ok(super::RegisterToolResponse { tool })
}

// ============================================================================
// Tool Query Request/Response Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize, IntoParams, JsonSchema)]
#[into_params(style = Form, parameter_in = Query)]
pub struct ListToolsParams {
    pub page_size: i64,
    pub next_page_token: Option<String>,
    pub endpoint_type: Option<String>,
    pub category: Option<String>,
}

impl ListToolsParams {
    pub fn pagination(&self) -> PaginationRequest {
        PaginationRequest {
            page_size: self.page_size,
            next_page_token: self.next_page_token.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListToolsResponse {
    #[serde(flatten)]
    pub tools: PaginatedResponse<ToolSerialized>,
}

// ============================================================================
// Tool Query Logic Functions
// ============================================================================

/// List registered tools
///
/// Returns a paginated list of registered tools, optionally filtered by endpoint type and category
#[shared_macros::authz_role(Admin, Maintainer, permission = "tool:read")]
#[shared_macros::authn]
pub async fn list_tools(
    repo: &impl crate::repository::ProviderRepositoryLike,
    params: ListToolsParams,
) -> Result<ListToolsResponse, CommonError> {
    use tracing::trace;

    trace!(
        endpoint_type = ?params.endpoint_type,
        category = ?params.category,
        "Listing tools"
    );

    let pagination = params.pagination();
    let tools = if let Some(category) = params.category {
        repo.list_tools_by_category(&category, &pagination, params.endpoint_type.as_deref())
            .await?
    } else {
        repo.list_tools(&pagination, params.endpoint_type.as_deref())
            .await?
    };

    trace!(count = tools.items.len(), "Tools listed successfully");

    Ok(ListToolsResponse { tools })
}

/// Get a specific tool by ID
///
/// Returns tool details including encrypted endpoint configuration
#[shared_macros::authz_role(Admin, Maintainer, permission = "tool:read")]
#[shared_macros::authn]
pub async fn get_tool_by_id(
    repo: &impl crate::repository::ProviderRepositoryLike,
    type_id: String,
    deployment_id: String,
) -> Result<ToolSerialized, CommonError> {
    use tracing::trace;

    trace!(
        type_id = %type_id,
        deployment_id = %deployment_id,
        "Getting tool by ID"
    );

    let tool = repo
        .get_tool_by_id(&type_id, &deployment_id)
        .await?
        .ok_or_else(|| CommonError::NotFound {
            msg: format!("Tool not found: {}/{}", type_id, deployment_id),
            lookup_id: format!("{}/{}", type_id, deployment_id),
            source: None,
        })?;

    trace!(
        type_id = %tool.type_id,
        deployment_id = %tool.deployment_id,
        "Tool retrieved successfully"
    );

    Ok(tool)
}

/// Deregister a tool
///
/// Removes a tool registration and all its aliases
#[shared_macros::authz_role(Admin, permission = "tool:write")]
#[shared_macros::authn]
pub async fn delete_tool(
    repo: &impl crate::repository::ProviderRepositoryLike,
    type_id: String,
    deployment_id: String,
) -> Result<(), CommonError> {
    use tracing::trace;

    trace!(
        type_id = %type_id,
        deployment_id = %deployment_id,
        "Deleting tool"
    );

    repo.delete_tool(&type_id, &deployment_id).await?;

    trace!(
        type_id = %type_id,
        deployment_id = %deployment_id,
        "Tool deleted successfully"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod unit {
        use super::super::*;

        #[test]
        fn test_http_endpoint_config_structure() {
            let config = HttpEndpointConfiguration {
                url: "https://example.com/api/tool".to_string(),
                invocation_key: "test_key_123".to_string(),
            };
            assert_eq!(config.url, "https://example.com/api/tool");
            assert_eq!(config.invocation_key, "test_key_123");
        }
    }

    // Integration tests would require setting up encryption service and mock HTTP server
    // These are skipped in CI but can be run locally with external services
    #[cfg(test)]
    mod integration {
        // use super::super::*;
        // use shared_macros::integration_test;

        // Would implement tests with:
        // - Mock HTTP server (using wiremock or similar)
        // - Test encryption service
        // - End-to-end tool invocation test
    }
}
