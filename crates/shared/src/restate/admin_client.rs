// TAKEN FROM https://github.com/restatedev/restate/blob/main/cli/src/clients/admin_client.rs

use http::StatusCode;
use restate_admin_rest_model::version::{AdminApiVersion, VersionInformation};
// use restate_cli_util::{CliContext, c_warn};
use restate_types::SemanticRestateVersion;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, trace};
use url::Url;

use crate::error::CommonError;

// use crate::build_info;
// use crate::cli_env::CliEnv;
use super::admin_interface::AdminClientInterface;

use super::errors::{ApiError, ApiErrorBody};

/// Min/max supported admin API versions
pub const MIN_ADMIN_API_VERSION: AdminApiVersion = AdminApiVersion::V2;
pub const MAX_ADMIN_API_VERSION: AdminApiVersion = AdminApiVersion::V3;

#[derive(Error, Debug)]
#[error(transparent)]
pub enum Error {
    // Error is boxed because ApiError can get quite large if the message body is large.
    Api(#[from] Box<ApiError>),
    #[error("(Protocol error) {0}")]
    Serialization(#[from] serde_json::Error),
    Network(#[from] reqwest::Error),
}

/// A lazy wrapper around a reqwest response that deserializes the body on
/// demand and decodes our custom error body on non-2xx responses.
pub struct Envelope<T> {
    inner: reqwest::Response,

    _phantom: std::marker::PhantomData<T>,
}

impl<T> Envelope<T>
where
    T: DeserializeOwned,
{
    pub fn status_code(&self) -> StatusCode {
        self.inner.status()
    }

    pub fn url(&self) -> &Url {
        self.inner.url()
    }

    pub async fn into_body(self) -> Result<T, Error> {
        let http_status_code = self.inner.status();
        let url = self.inner.url().clone();
        if !self.status_code().is_success() {
            let body = self.inner.text().await?;
            debug!(url = %url, status = %http_status_code, body = %body, "API error response");
            // Wrap the error into ApiError
            return Err(Error::Api(Box::new(ApiError {
                http_status_code,
                url,
                body: serde_json::from_str(&body)?,
            })));
        }

        let body = self.inner.text().await?;
        trace!(url = %url, status = %http_status_code, body = %body, "API response");
        Ok(serde_json::from_str(&body)?)
    }

    pub async fn into_api_error(self) -> Result<ApiError, Error> {
        let http_status_code = self.inner.status();
        let url = self.inner.url().clone();

        let body = self.inner.text().await?;
        trace!(url = %url, status = %http_status_code, body = %body, "API error response parsed");
        Ok(ApiError {
            http_status_code,
            url,
            body: serde_json::from_str(&body)?,
        })
    }

    pub async fn into_text(self) -> Result<String, Error> {
        Ok(self.inner.text().await?)
    }
    pub fn success_or_error(self) -> Result<StatusCode, Error> {
        let http_status_code = self.inner.status();
        let url = self.inner.url().clone();
        trace!(url = %url, status = %http_status_code, "Response received");
        match self.inner.error_for_status() {
            Ok(_) => Ok(http_status_code),
            Err(e) => Err(Error::Network(e)),
        }
    }
}

impl<T> From<reqwest::Response> for Envelope<T> {
    fn from(value: reqwest::Response) -> Self {
        Self {
            inner: value,
            _phantom: Default::default(),
        }
    }
}

/// A handy client for the admin HTTP service.
#[derive(Clone)]
pub struct AdminClient {
    pub(crate) inner: reqwest::Client,
    pub(crate) base_url: Url,
    pub(crate) bearer_token: Option<String>,
    pub(crate) request_timeout: Duration,
    pub(crate) admin_api_version: AdminApiVersion,
    pub(crate) restate_server_version: SemanticRestateVersion,
    pub(crate) advertised_ingress_address: Option<String>,
}

impl AdminClient {
    // pub async fn new(env: &CliEnv) -> anyhow::Result<Self> {
    pub async fn new(base_url: Url, bearer_token: Option<String>) -> Result<Self, CommonError> {
        let raw_client = reqwest::Client::builder()
            .user_agent(format!(
                "{}/{} {}-{}",
                env!("CARGO_PKG_NAME"),
                // build_info::RESTATE_CLI_VERSION,
                "0.0.1",
                std::env::consts::OS,
                std::env::consts::ARCH,
            ))
            // .connect_timeout(CliContext::get().connect_timeout())
            // .danger_accept_invalid_certs(CliContext::get().insecure_skip_tls_verify())
            .build()?;

        // let base_url = env.admin_base_url()?.clone();
        // let bearer_token = env.bearer_token()?.map(str::to_string);

        let client = Self {
            inner: raw_client,
            base_url,
            bearer_token,
            // request_timeout: CliContext::get().request_timeout(),
            request_timeout: Duration::from_secs(10),
            admin_api_version: AdminApiVersion::Unknown,
            restate_server_version: SemanticRestateVersion::unknown(),
            advertised_ingress_address: None,
        };

        if let Ok(envelope) = client.version().await {
            match envelope.into_body().await {
                Ok(version_information) => {
                    return Self::choose_api_version(client, version_information);
                }
                Err(err) => debug!("Failed parsing the version information: {err}"),
            }
        }

        Ok(client)
    }

    pub fn versioned_url(&self, path: impl IntoIterator<Item = impl AsRef<str>>) -> Url {
        let mut url = self.base_url.clone();

        {
            let mut segments = url.path_segments_mut().expect("Bad url!");
            segments.pop_if_empty();

            match self.admin_api_version {
                AdminApiVersion::Unknown => segments.extend(path),
                // v1 clusters didn't support versioned urls
                AdminApiVersion::V1 => segments.extend(path),
                AdminApiVersion::V2 => segments.push("v2").extend(path),
                AdminApiVersion::V3 => segments.push("v3").extend(path),
            };
        }

        url
    }

    fn choose_api_version(
        mut client: AdminClient,
        version_information: VersionInformation,
    ) -> Result<AdminClient, CommonError> {
        if let Some(admin_api_version) = AdminApiVersion::choose_max_supported_version(
            MIN_ADMIN_API_VERSION..=MAX_ADMIN_API_VERSION,
            version_information.min_admin_api_version..=version_information.max_admin_api_version,
        ) {
            client.restate_server_version =
                match SemanticRestateVersion::parse(&version_information.version) {
                    Ok(version) => version,
                    Err(err) => {
                        debug!(
                            "Failed to parse Restate server version {}: {err}",
                            version_information.version
                        );
                        SemanticRestateVersion::unknown()
                    }
                };
            client.admin_api_version = admin_api_version;
            client.advertised_ingress_address =
                version_information.ingress_endpoint.map(|u| u.to_string());
            Ok(client)
        } else {
            Err(CommonError::Unknown(anyhow::anyhow!(
                "The CLI is not compatible with the Restate server '{}'. Please update the CLI to match the Restate server version '{}'.",
                client.base_url,
                version_information.version
            )))
        }
    }

    /// Ensures the Restate server is reachable and healthy.
    /// This should be called when you need to verify connectivity before using the client.
    pub async fn ensure_healthy(&self) -> Result<(), CommonError> {
        // Try to get version information first
        if let Ok(envelope) = self.version().await {
            if envelope.into_body().await.is_ok() {
                // Version endpoint works, server is healthy
                return Ok(());
            }
        }

        // If version endpoint doesn't work, check health endpoint
        // This could mean that the server is not running or runs an old version
        // which does not support version information.
        self.health()
            .await
            .map_err(Into::into)
            .and_then(|r| r.success_or_error())
            .map_err(|_| {
                CommonError::Unknown(anyhow::anyhow!(
                    "Unable to connect to the Restate server '{}'. Please make sure that it is running and reachable.",
                    self.base_url
                ))
            })?;

        Ok(())
    }

    /// Prepare a request builder for the given method and path.
    pub(crate) fn prepare(&self, method: reqwest::Method, path: Url) -> reqwest::RequestBuilder {
        let request_builder = self
            .inner
            .request(method, path)
            .timeout(self.request_timeout);

        match self.bearer_token.as_deref() {
            Some(token) => request_builder.bearer_auth(token),
            None => request_builder,
        }
    }

    /// Prepare a request builder that encodes the body as JSON.
    fn prepare_with_body<B>(
        &self,
        method: reqwest::Method,
        path: Url,
        body: B,
    ) -> reqwest::RequestBuilder
    where
        B: Serialize,
    {
        self.prepare(method, path)
            .header("Accept", "application/json")
            .json(&body)
    }

    /// Execute a request and return the response as a lazy Envelope.
    pub(crate) async fn run<T>(
        &self,
        method: reqwest::Method,
        path: Url,
    ) -> reqwest::Result<Envelope<T>>
    where
        T: DeserializeOwned + Send,
    {
        trace!(method = %method, url = %path, "Sending request");
        let request = self.prepare(method, path.clone());
        let resp = request.send().await?;
        trace!(url = %path, status = %resp.status(), "Request completed");
        Ok(resp.into())
    }

    pub(crate) async fn run_with_body<T, B>(
        &self,
        method: reqwest::Method,
        path: Url,
        body: B,
    ) -> reqwest::Result<Envelope<T>>
    where
        T: DeserializeOwned + Send,
        B: Serialize + std::fmt::Debug + Send,
    {
        trace!(method = %method, url = %path, body = ?body, "Sending request with body");
        let request = self.prepare_with_body(method, path.clone(), body);
        let resp = request.send().await?;
        trace!(url = %path, status = %resp.status(), "Request completed");
        Ok(resp.into())
    }

    /// Get state from Restate using SQL API
    pub async fn get_state(
        &self,
        service: &str,
        key: &str,
    ) -> Result<HashMap<String, String>, Error> {
        // Use Restate SQL API to query state
        let query = format!(
            "SELECT key, value_utf8, value FROM state WHERE service_name = '{}' AND service_key = '{}'",
            service.replace("'", "''"), // Escape single quotes
            key.replace("'", "''")
        );

        // Use versioned URL for the query endpoint
        let query_url = self.versioned_url(["query"]);

        trace!(url = %query_url, query = %query, "Executing SQL query");

        #[derive(Serialize, Debug)]
        struct SqlQueryRequest {
            query: String,
        }

        let envelope: Envelope<SqlQueryResponse> = self
            .run_with_body(
                reqwest::Method::POST,
                query_url.clone(),
                SqlQueryRequest { query },
            )
            .await?;

        // Check status code first before consuming the envelope
        let status = envelope.status_code();
        let url = envelope.url().clone();

        // Get the raw response text
        let raw_body = envelope.into_text().await?;

        // Handle non-success status codes
        if !status.is_success() {
            trace!(url = %url, status = %status, body = %raw_body, "SQL query error response");
            // Try to parse as ApiError body
            let error_body =
                serde_json::from_str::<ApiErrorBody>(&raw_body).unwrap_or_else(|_| ApiErrorBody {
                    restate_code: None,
                    message: raw_body.clone(),
                });
            return Err(Error::Api(Box::new(ApiError {
                http_status_code: status,
                url,
                body: error_body,
            })));
        }

        // Handle empty responses (when query returns no rows)
        if raw_body.trim().is_empty() {
            trace!("SQL query returned no rows");
            return Ok(HashMap::new());
        }

        // Parse the JSON response
        let response: SqlQueryResponse = match serde_json::from_str(&raw_body) {
            Ok(r) => r,
            Err(e) => {
                debug!(error = %e, body = %raw_body, "Failed to parse SQL query response");
                return Err(Error::Serialization(e));
            }
        };

        // Convert rows to HashMap, parsing JSON strings from value_utf8
        let state_map: HashMap<String, String> = response
            .rows
            .into_iter()
            .map(|row| {
                // Parse the JSON-encoded string to get the actual value
                // value_utf8 contains a JSON string like "\"actual_value\"", so we need to deserialize it
                let parsed_value: String = serde_json::from_str(&row.value_utf8)
                    .unwrap_or_else(|_| row.value_utf8.clone()); // Fallback to original if parsing fails
                (row.key, parsed_value)
            })
            .collect();

        // Return the value for the requested state_key
        Ok(state_map)
    }
}

#[derive(Deserialize)]
struct SqlQueryRow {
    key: String,
    value_utf8: String,
    #[allow(dead_code)]
    value: String,
}

#[derive(Deserialize)]
struct SqlQueryResponse {
    rows: Vec<SqlQueryRow>,
}

// Ensure that AdminClient is Send + Sync. Compiler will fail if it's not.
const _: () = {
    const fn assert_send<T: Send + Sync>() {}
    assert_send::<AdminClient>();
};
