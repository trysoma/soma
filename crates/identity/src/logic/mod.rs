use jsonwebtoken::{Algorithm, Validation, decode, decode_header};
use serde_json::{Map, Value};
use shared::error::CommonError;

use crate::logic::api_key::EncryptedApiKeyConfig;
use crate::logic::sts::config::StsTokenConfig;
use crate::logic::sts::external_jwk_cache::ExternalJwksCache;
use crate::logic::token_mapping::template::JwtTokenTemplateValidationConfig;
use crate::logic::user_auth_flow::config::EncryptedUserAuthFlowConfig;

pub mod api_key;
pub mod auth_client;
pub mod internal_token_issuance;
pub mod jwk;
pub mod sts;
pub mod token_mapping;
pub mod user;
pub mod user_auth_flow;

/// Default DEK alias for client secret encryption
pub const DEFAULT_DEK_ALIAS: &str = "default";

/// Decode a JWT token and return its claims as a serde_json Map.
/// Validates the signature using the provided JWKS cache.
pub async fn decode_jwt_to_claims(
    token: &str,
    jwks_uri: &str,
    external_jwks_cache: &ExternalJwksCache,
    validation_config: &JwtTokenTemplateValidationConfig,
) -> Result<Map<String, Value>, CommonError> {
    let header = decode_header(token)
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to decode JWT header: {e}")))?;

    let kid = header.kid.ok_or_else(|| {
        CommonError::Unknown(anyhow::anyhow!("JWT token missing 'kid' in header"))
    })?;

    // Get or fetch the external JWKS
    if external_jwks_cache.get_key(jwks_uri, &kid).is_none() {
        external_jwks_cache.fetch_jwks(jwks_uri).await?;
    }

    let decoding_key = external_jwks_cache.get_key(jwks_uri, &kid).ok_or_else(|| {
        CommonError::Unknown(anyhow::anyhow!(
            "Key '{kid}' not found in JWKS from {jwks_uri}"
        ))
    })?;

    // Build validation
    let mut validation = Validation::new(Algorithm::RS256);

    if let Some(issuer) = &validation_config.issuer {
        validation.set_issuer(&[issuer]);
    }

    if let Some(audiences) = &validation_config.valid_audiences {
        validation.set_audience(audiences);
    }

    let token_data = decode::<Value>(token, &decoding_key, &validation)
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("JWT validation failed: {e}")))?;

    match token_data.claims {
        Value::Object(obj) => Ok(obj),
        _ => Err(CommonError::Unknown(anyhow::anyhow!(
            "Invalid token claims"
        ))),
    }
}

/// Decode a JWT token and return its claims as a serde_json Map.
/// Validates the signature using the provided JWKS cache.
/// This function is unsafe because it does not validate the signature.
/// It is used to decode tokens from external identity providers that are trusted
/// as we've just completed the handshake with the external identity provider.
/// This is only used for OIDC and OAuth callbacks.
pub async fn decode_jwt_to_claims_unsafe(
    token: &str,
    jwks_uri: &str,
    external_jwks_cache: &ExternalJwksCache,
) -> Result<Map<String, Value>, CommonError> {
    let header = decode_header(token)
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to decode JWT header: {e}")))?;

    let kid = header.kid.ok_or_else(|| {
        CommonError::Unknown(anyhow::anyhow!("JWT token missing 'kid' in header"))
    })?;

    // Get or fetch the external JWKS
    if external_jwks_cache.get_key(jwks_uri, &kid).is_none() {
        external_jwks_cache.fetch_jwks(jwks_uri).await?;
    }

    let decoding_key = external_jwks_cache.get_key(jwks_uri, &kid).ok_or_else(|| {
        CommonError::Unknown(anyhow::anyhow!(
            "Key '{kid}' not found in JWKS from {jwks_uri}"
        ))
    })?;

    // Build validation - disable all validation since we just want to extract claims
    // The token comes from a trusted source (the IdP's token endpoint over HTTPS)
    let mut validation = Validation::new(Algorithm::RS256);
    validation.insecure_disable_signature_validation();
    validation.validate_aud = false; // Don't validate audience
    validation.validate_exp = false; // Don't validate expiration (we'll use our own token expiry)

    let token_data = decode::<Value>(token, &decoding_key, &validation)
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("JWT validation failed: {e}")))?;

    match token_data.claims {
        Value::Object(obj) => Ok(obj),
        _ => Err(CommonError::Unknown(anyhow::anyhow!(
            "Invalid token claims"
        ))),
    }
}

/// Default timeout for HTTP requests (30 seconds)
const HTTP_TIMEOUT_SECS: u64 = 30;

/// Fetch userinfo from the userinfo endpoint using the access token
pub async fn fetch_userinfo(
    userinfo_url: &str,
    access_token: &str,
) -> Result<Map<String, Value>, CommonError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(HTTP_TIMEOUT_SECS))
        .build()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to create HTTP client: {e}")))?;
    let response = client
        .get(userinfo_url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to fetch userinfo: {e}")))?;

    if !response.status().is_success() {
        return Err(CommonError::Unknown(anyhow::anyhow!(
            "Userinfo endpoint returned status: {}",
            response.status()
        )));
    }

    let value: Value = response.json().await.map_err(|e| {
        CommonError::Unknown(anyhow::anyhow!("Failed to parse userinfo response: {e}"))
    })?;

    match value {
        Value::Object(obj) => Ok(obj),
        _ => Err(CommonError::Unknown(anyhow::anyhow!(
            "Userinfo response is not a JSON object"
        ))),
    }
}

/// Introspect an opaque access token using RFC 7662 Token Introspection.
///
/// This function:
/// 1. Makes a POST request to the introspection endpoint with the token
/// 2. Checks if the token is active
/// 3. Returns the introspection response claims if active
///
/// See: https://www.oauth.com/oauth2-servers/token-introspection-endpoint/
pub async fn introspect_token(
    introspect_url: &str,
    access_token: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<Map<String, Value>, CommonError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(HTTP_TIMEOUT_SECS))
        .build()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to create HTTP client: {e}")))?;

    // RFC 7662: Token introspection request
    // POST with token in form body, authenticated with client credentials
    let response = client
        .post(introspect_url)
        .basic_auth(client_id, Some(client_secret))
        .form(&[("token", access_token)])
        .send()
        .await
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to introspect token: {e}")))?;

    if !response.status().is_success() {
        return Err(CommonError::Unknown(anyhow::anyhow!(
            "Token introspection endpoint returned status: {}",
            response.status()
        )));
    }

    let value: Value = response.json().await.map_err(|e| {
        CommonError::Unknown(anyhow::anyhow!(
            "Failed to parse introspection response: {e}"
        ))
    })?;

    let obj = match value {
        Value::Object(obj) => obj,
        _ => {
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "Token introspection response is not a JSON object"
            )));
        }
    };

    // RFC 7662: Check if token is active
    // The "active" claim is REQUIRED in the response
    let is_active = obj.get("active").and_then(|v| v.as_bool()).unwrap_or(false);

    if !is_active {
        return Err(CommonError::Authentication {
            msg: "Token is not active (revoked or expired)".to_string(),
            source: None,
        });
    }

    tracing::debug!("Token introspection successful, token is active");
    Ok(obj)
}

/// Validate that an ID is a valid identifier.
///
/// Valid IDs must:
/// - Not be empty
/// - Only contain lowercase letters, numbers, and hyphens
/// - Start with a letter
/// - Not end with a hyphen
/// - Not contain consecutive hyphens
pub fn validate_id(id: &str, resource_type: &str) -> Result<(), CommonError> {
    if id.is_empty() {
        return Err(CommonError::InvalidRequest {
            msg: format!("{resource_type} ID cannot be empty"),
            source: None,
        });
    }

    // Check that it starts with a letter
    if !id.chars().next().unwrap().is_ascii_lowercase() {
        return Err(CommonError::InvalidRequest {
            msg: format!("{resource_type} ID must start with a lowercase letter, got: '{id}'"),
            source: None,
        });
    }

    // Check for valid characters
    for (i, c) in id.chars().enumerate() {
        if !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '-' {
            return Err(CommonError::InvalidRequest {
                msg: format!(
                    "{resource_type} ID can only contain lowercase letters, numbers, and hyphens. Invalid character '{c}' at position {i}"
                ),
                source: None,
            });
        }
    }

    // Check that it doesn't end with a hyphen
    if id.ends_with('-') {
        return Err(CommonError::InvalidRequest {
            msg: format!("{resource_type} ID cannot end with a hyphen: '{id}'"),
            source: None,
        });
    }

    // Check for consecutive hyphens
    if id.contains("--") {
        return Err(CommonError::InvalidRequest {
            msg: format!("{resource_type} ID cannot contain consecutive hyphens: '{id}'"),
            source: None,
        });
    }

    Ok(())
}

/// Events fired when identity configuration changes
#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum OnConfigChangeEvt {
    /// An API key was created
    ApiKeyCreated(EncryptedApiKeyConfig),
    /// An API key was deleted (contains id)
    ApiKeyDeleted(String),
    /// An STS configuration was created
    StsConfigCreated(StsTokenConfig),
    /// An STS configuration was deleted (contains id)
    StsConfigDeleted(String),
    /// A user auth flow configuration was created
    UserAuthFlowConfigCreated(EncryptedUserAuthFlowConfig),
    /// A user auth flow configuration was deleted (contains id)
    UserAuthFlowConfigDeleted(String),
}

/// Sender for config change events
pub type OnConfigChangeTx = tokio::sync::broadcast::Sender<OnConfigChangeEvt>;
/// Receiver for config change events
pub type OnConfigChangeRx = tokio::sync::broadcast::Receiver<OnConfigChangeEvt>;
