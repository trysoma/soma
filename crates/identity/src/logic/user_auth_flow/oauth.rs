//! OAuth2 authorization flow logic.
//!
//! This module handles the OAuth2 authorization code flow (with optional PKCE).
//!
//! Flow:
//! 1. Authorization: Generate state/PKCE, redirect to IdP
//! 2. Callback: Exchange code for tokens, fetch userinfo, map claims, issue internal tokens

use chrono::{Duration, Utc};
use encryption::logic::CryptoCache;
use oauth2::{AuthUrl, ClientId, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope, TokenUrl};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use shared::error::CommonError;
use shared::primitives::WrappedChronoDateTime;
use utoipa::ToSchema;

use crate::logic::internal_token_issuance::{NormalizedTokenInputFields, NormalizedTokenIssuanceResult, issue_tokens_for_normalized_user};
use crate::logic::token_mapping::template::{apply_mapping_template, DecodedTokenSources};
use crate::logic::token_mapping::TokenMapping;
use crate::logic::user_auth_flow::config::{OauthConfig, UserAuthFlowConfig};
use crate::repository::{CreateOAuthState, UserRepositoryLike};

// ============================================
// Authorization Flow Types
// ============================================

// OAuth state types (for CSRF protection and PKCE)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct OAuthState {
    pub state: String,
    pub config_id: String,
    pub code_verifier: Option<String>,
    pub nonce: Option<String>,
    pub redirect_uri: Option<String>,
    pub created_at: WrappedChronoDateTime,
    pub expires_at: WrappedChronoDateTime,
}

/// Parameters for starting the OAuth authorization flow
#[derive(Debug)]
pub struct StartAuthorizationParams {
    /// IdP configuration ID
    pub config_id: String,
    /// Optional override for post-login redirect
    pub redirect_after_login: Option<String>,
}

/// Result of starting the authorization flow
#[derive(Debug, Serialize, ToSchema)]
pub struct StartAuthorizationResult {
    /// The URL to redirect the user to
    pub login_redirect_url: String,
}

/// Parameters for handling the OAuth callback
#[derive(Debug, Deserialize)]
pub struct OAuthCallbackParams {
    /// Authorization code from the IdP
    pub code: String,
    /// State parameter (for CSRF validation)
    pub state: String,
    /// Error from the IdP (if any)
    pub error: Option<String>,
    /// Error description from the IdP
    pub error_description: Option<String>,
}

/// Result of handling the OAuth callback
#[derive(Debug, Serialize, ToSchema)]
pub struct OAuthCallbackResult {
    /// Access token
    pub access_token: String,
    /// Refresh token
    pub refresh_token: String,
    /// Token expiration in seconds
    pub expires_in: i64,
    /// Optional redirect URI after login
    pub redirect_uri: Option<String>,
}

// ============================================
// Base Authorization Flow Parameters
// ============================================

/// Base parameters for building an authorization URL.
/// These are the common primitives needed for any OAuth2/OIDC flow.
pub struct BaseAuthorizationParams<'a> {
    pub authorization_endpoint: &'a str,
    pub token_endpoint: &'a str,
    pub redirect_uri: &'a str,
    pub client_id: &'a str,
    pub scopes: &'a [String],
    pub pkce_challenge: Option<&'a PkceCodeChallenge>,
    pub csrf_state: &'a CsrfToken,
    /// Additional query parameters to add to the authorization URL (e.g., nonce for OIDC)
    pub extra_params: Vec<(&'a str, String)>,
}

/// Base parameters for token exchange
pub struct BaseTokenExchangeParams<'a> {
    pub token_endpoint: &'a str,
    pub client_id: &'a str,
    pub client_secret: &'a str,
    pub redirect_uri: &'a str,
    pub code: &'a str,
    pub code_verifier: Option<&'a str>,
}

// ============================================
// HTTP Client Helper
// ============================================

/// Create an HTTP client for OAuth/OIDC requests
pub(crate) fn create_http_client() -> Result<reqwest::Client, CommonError> {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to create HTTP client: {e}")))
}

// ============================================
// Base Authorization URL Builder
// ============================================

/// Build an authorization URL from base parameters.
/// This is the core function that both OAuth2 and OIDC flows use.
pub fn build_authorization_url(params: BaseAuthorizationParams<'_>) -> Result<String, CommonError> {
    let auth_url = AuthUrl::new(params.authorization_endpoint.to_string()).map_err(|e| {
        CommonError::InvalidRequest {
            msg: format!("Invalid authorization endpoint: {e}"),
            source: None,
        }
    })?;

    let token_url =
        TokenUrl::new(params.token_endpoint.to_string()).map_err(|e| CommonError::InvalidRequest {
            msg: format!("Invalid token endpoint: {e}"),
            source: None,
        })?;

    let redirect_url =
        RedirectUrl::new(params.redirect_uri.to_string()).map_err(|e| CommonError::InvalidRequest {
            msg: format!("Invalid redirect URI: {e}"),
            source: None,
        })?;

    let client = oauth2::basic::BasicClient::new(ClientId::new(params.client_id.to_string()))
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(redirect_url);

    let csrf_for_closure = params.csrf_state.clone();
    let mut auth_request = client.authorize_url(move || csrf_for_closure.clone());

    // Add scopes
    for scope in params.scopes {
        auth_request = auth_request.add_scope(Scope::new(scope.clone()));
    }

    // Add PKCE challenge if provided
    if let Some(challenge) = params.pkce_challenge {
        auth_request = auth_request.set_pkce_challenge(challenge.clone());
    }

    let (mut url, _) = auth_request.url();

    // Add extra parameters
    for (key, value) in params.extra_params {
        url.query_pairs_mut().append_pair(key, &value);
    }

    Ok(url.to_string())
}

// ============================================
// Base Token Exchange
// ============================================

/// Exchange authorization code for tokens.
/// This is the core token exchange function used by both OAuth2 and OIDC flows.
pub async fn exchange_code_for_tokens(
    params: BaseTokenExchangeParams<'_>,
) -> Result<Map<String, Value>, CommonError> {
    let client = create_http_client()?;

    let mut form_params = vec![
        ("grant_type", "authorization_code".to_string()),
        ("code", params.code.to_string()),
        ("redirect_uri", params.redirect_uri.to_string()),
        ("client_id", params.client_id.to_string()),
        ("client_secret", params.client_secret.to_string()),
    ];

    if let Some(verifier) = params.code_verifier {
        form_params.push(("code_verifier", verifier.to_string()));
    }

    let response = client
        .post(params.token_endpoint)
        .form(&form_params)
        .send()
        .await
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Token exchange request failed: {e}")))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CommonError::Unknown(anyhow::anyhow!(
            "Token exchange failed: HTTP {} - {}",
            status,
            body
        )));
    }

    let token_response: Value = response
        .json()
        .await
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to parse token response: {e}")))?;

    match token_response {
        Value::Object(obj) => Ok(obj),
        _ => Err(CommonError::Unknown(anyhow::anyhow!(
            "Token response is not a JSON object"
        ))),
    }
}

// ============================================
// Userinfo Fetching
// ============================================

/// Fetch userinfo from endpoint
pub async fn fetch_userinfo(
    userinfo_url: &str,
    access_token: &str,
) -> Result<Map<String, Value>, CommonError> {
    let client = create_http_client()?;

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

    let value: Value = response
        .json()
        .await
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to parse userinfo: {e}")))?;

    match value {
        Value::Object(obj) => Ok(obj),
        _ => Err(CommonError::Unknown(anyhow::anyhow!(
            "Userinfo response is not a JSON object"
        ))),
    }
}

// ============================================
// JWT Decoding
// ============================================

/// Decode JWT claims without signature verification (unsafe - only use when you trust the source)
pub fn decode_jwt_claims_unsafe(token: &str) -> Result<Map<String, Value>, CommonError> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(CommonError::Unknown(anyhow::anyhow!("Invalid JWT format")));
    }

    // Decode the payload (second part)
    use base64::Engine;
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(parts[1])
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to decode JWT payload: {e}")))?;

    let claims: Value = serde_json::from_slice(&payload)
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to parse JWT claims: {e}")))?;

    match claims {
        Value::Object(obj) => Ok(obj),
        _ => Err(CommonError::Unknown(anyhow::anyhow!(
            "JWT claims is not a JSON object"
        ))),
    }
}

// ============================================
// Token Mapping
// ============================================

/// Apply token mapping to extract normalized fields
pub fn apply_token_mapping(
    mapping: &TokenMapping,
    sources: &DecodedTokenSources,
) -> Result<NormalizedTokenInputFields, CommonError> {
    match mapping {
        TokenMapping::JwtTemplate(config) => {
            let result = apply_mapping_template(sources, config)?;
            Ok(NormalizedTokenInputFields {
                subject: result.subject,
                email: result.email,
                groups: result.groups,
                role: result.role,
            })
        }
    }
}

// ============================================
// OAuth2 Authorization Flow
// ============================================

/// Start the OAuth2 authorization flow.
///
/// This function:
/// 1. Generates PKCE challenge (for PKCE flows)
/// 2. Generates CSRF state
/// 3. Builds the authorization URL
/// 4. Stores state in database
/// 5. Returns the authorization URL to redirect to
pub async fn start_authorization_handshake<R: UserRepositoryLike>(
    repository: &R,
    crypto_cache: &CryptoCache,
    params: StartAuthorizationParams,
) -> Result<StartAuthorizationResult, CommonError> {
    let config_db = repository
        .get_user_auth_flow_config_by_id(&params.config_id)
        .await?
        .ok_or_else(|| CommonError::InvalidRequest {
            msg: "Invalid or expired config_id".to_string(),
            source: None,
        })?;

    let config = config_db.config.decrypt(crypto_cache).await?;

    // Only handle OAuth flows in this module
    let (oauth_config, uses_pkce) = match &config {
        UserAuthFlowConfig::OauthAuthorizationCodeFlow(oauth) => (oauth, false),
        UserAuthFlowConfig::OauthAuthorizationCodePkceFlow(oauth) => (oauth, true),
        _ => {
            return Err(CommonError::InvalidRequest {
                msg: "Configuration is not an OAuth2 flow. Use OIDC module for OIDC flows."
                    .to_string(),
                source: None,
            });
        }
    };

    // Generate PKCE if needed
    let (pkce_challenge, pkce_verifier) = if uses_pkce {
        let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
        (Some(challenge), Some(verifier))
    } else {
        (None, None)
    };

    // Generate CSRF state
    let csrf_state = CsrfToken::new_random();

    // Build authorization URL
    let base_params = BaseAuthorizationParams {
        authorization_endpoint: &oauth_config.authorization_endpoint,
        token_endpoint: &oauth_config.token_endpoint,
        redirect_uri: &oauth_config.authorization_endpoint, // This should be the app's redirect URI
        client_id: &oauth_config.client_id,
        scopes: &oauth_config.scopes,
        pkce_challenge: pkce_challenge.as_ref(),
        csrf_state: &csrf_state,
        extra_params: vec![],
    };

    let login_redirect_url = build_authorization_url(base_params)?;

    // Calculate expiration
    let state_ttl = 600; // 10 minutes
    let now = Utc::now();
    let expires_at = now + Duration::seconds(state_ttl);

    // Store state in database
    let create_state = CreateOAuthState {
        state: csrf_state.secret().to_string(),
        config_id: params.config_id,
        code_verifier: pkce_verifier.map(|v| v.secret().to_string()),
        nonce: None, // OAuth2 doesn't use nonce
        redirect_uri: params.redirect_after_login,
        created_at: WrappedChronoDateTime::now(),
        expires_at: WrappedChronoDateTime::from(expires_at),
    };
    repository.create_oauth_state(&create_state).await?;

    Ok(StartAuthorizationResult { login_redirect_url })
}

/// Handle the OAuth2 callback.
///
/// This function:
/// 1. Validates state parameter
/// 2. Exchanges authorization code for tokens
/// 3. Fetches userinfo
/// 4. Applies the mapping template to extract normalized fields
/// 5. Issues internal access/refresh tokens
pub async fn handle_authorization_handshake_callback<R: UserRepositoryLike>(
    repository: &R,
    crypto_cache: &CryptoCache,
    params: OAuthCallbackParams,
) -> Result<NormalizedTokenIssuanceResult, CommonError> {
    // Check for error response from IdP
    if let Some(error) = &params.error {
        return Err(CommonError::InvalidRequest {
            msg: format!(
                "OAuth error from IdP: {} - {}",
                error,
                params.error_description.as_deref().unwrap_or("No description")
            ),
            source: None,
        });
    }

    // Validate state and get stored data
    let oauth_state = repository
        .get_oauth_state_by_state(&params.state)
        .await?
        .ok_or_else(|| CommonError::InvalidRequest {
            msg: "Invalid or expired state parameter".to_string(),
            source: None,
        })?;

    // Check if state has expired
    if oauth_state.expires_at.get_inner() < &Utc::now() {
        repository.delete_oauth_state(&params.state).await?;
        return Err(CommonError::InvalidRequest {
            msg: "State parameter has expired".to_string(),
            source: None,
        });
    }

    // Delete state (one-time use)
    repository.delete_oauth_state(&params.state).await?;

    // Load and decrypt config
    let config_db = repository
        .get_user_auth_flow_config_by_id(&oauth_state.config_id)
        .await?
        .ok_or_else(|| CommonError::InvalidRequest {
            msg: "Configuration not found".to_string(),
            source: None,
        })?;

    let config = config_db.config.decrypt(crypto_cache).await?;

    // Only handle OAuth flows
    let oauth_config = match &config {
        UserAuthFlowConfig::OauthAuthorizationCodeFlow(oauth)
        | UserAuthFlowConfig::OauthAuthorizationCodePkceFlow(oauth) => oauth,
        _ => {
            return Err(CommonError::InvalidRequest {
                msg: "Configuration is not an OAuth2 flow".to_string(),
                source: None,
            });
        }
    };

    // Exchange code for tokens
    let token_exchange_params = BaseTokenExchangeParams {
        token_endpoint: &oauth_config.token_endpoint,
        client_id: &oauth_config.client_id,
        client_secret: &oauth_config.client_secret,
        redirect_uri: &oauth_config.authorization_endpoint, // Should be app's redirect URI
        code: &params.code,
        code_verifier: oauth_state.code_verifier.as_deref(),
    };

    let token_response = exchange_code_for_tokens(token_exchange_params).await?;

    // Get access token
    let access_token = token_response
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CommonError::Unknown(anyhow::anyhow!("No access token in response")))?;

    // For OAuth2, we need to fetch userinfo
    let userinfo_endpoint = oauth_config.userinfo_endpoint.as_ref().ok_or_else(|| {
        CommonError::InvalidRequest {
            msg: "userinfo_endpoint is required for OAuth2 flows".to_string(),
            source: None,
        }
    })?;

    let userinfo_claims = fetch_userinfo(userinfo_endpoint, access_token).await?;

    // Build decoded token sources
    let sources = DecodedTokenSources::new().with_userinfo(userinfo_claims);

    // Apply mapping template
    let normalized = apply_token_mapping(&oauth_config.mapping, &sources)?;

    // Issue internal tokens
    let token_result =
        issue_tokens_for_normalized_user(repository, crypto_cache, normalized).await?;

    Ok(token_result)

    
}
