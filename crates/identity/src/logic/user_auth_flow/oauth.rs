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
use serde_json::{Map, Value};
use shared::error::CommonError;
use shared::primitives::WrappedChronoDateTime;
use utoipa::ToSchema;

use crate::logic::internal_token_issuance::{
    NormalizedTokenInputFields, issue_tokens_for_normalized_user,
};
use crate::logic::sts::external_jwk_cache::ExternalJwksCache;
use crate::logic::token_mapping::TokenMapping;
use crate::logic::token_mapping::template::{DecodedTokenSources, apply_mapping_template};
use crate::logic::user_auth_flow::config::UserAuthFlowConfig;
use crate::logic::user_auth_flow::{
    OAuthCallbackParams, OAuthCallbackResult, OauthConfig, StartAuthorizationParams,
    StartAuthorizationResult,
};
use crate::logic::{decode_jwt_to_claims_unsafe, introspect_token};
use crate::repository::UserRepositoryLike;
use crate::router::{API_VERSION_1, PATH_PREFIX, SERVICE_ROUTE_KEY};

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

    let token_url = TokenUrl::new(params.token_endpoint.to_string()).map_err(|e| {
        CommonError::InvalidRequest {
            msg: format!("Invalid token endpoint: {e}"),
            source: None,
        }
    })?;

    let redirect_url = RedirectUrl::new(params.redirect_uri.to_string()).map_err(|e| {
        CommonError::InvalidRequest {
            msg: format!("Invalid redirect URI: {e}"),
            source: None,
        }
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
            "Token exchange failed: HTTP {status} - {body}"
        )));
    }

    let token_response: Value = response.json().await.map_err(|e| {
        CommonError::Unknown(anyhow::anyhow!("Failed to parse token response: {e}"))
    })?;

    match token_response {
        Value::Object(obj) => Ok(obj),
        _ => Err(CommonError::Unknown(anyhow::anyhow!(
            "Token response is not a JSON object"
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
    base_redirect_uri: &str,
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
        redirect_uri: &format!(
            "{base_redirect_uri}{PATH_PREFIX}/{SERVICE_ROUTE_KEY}/{API_VERSION_1}/auth/callback"
        ),
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
    let create_state = OAuthState {
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
    external_jwks_cache: &ExternalJwksCache,
    base_redirect_uri: &str,
    params: OAuthCallbackParams,
    config: &OauthConfig,
    oauth_state: &OAuthState,
) -> Result<OAuthCallbackResult, CommonError> {
    // Exchange code for tokens
    let token_exchange_params = BaseTokenExchangeParams {
        token_endpoint: &config.token_endpoint,
        client_id: &config.client_id,
        client_secret: &config.client_secret,
        redirect_uri: &format!(
            "{base_redirect_uri}{PATH_PREFIX}/{SERVICE_ROUTE_KEY}/{API_VERSION_1}/auth/callback"
        ),
        code: &params.code,
        code_verifier: oauth_state.code_verifier.as_deref(),
    };

    let token_response = exchange_code_for_tokens(token_exchange_params).await?;

    // Get access token
    let access_token = token_response
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CommonError::Unknown(anyhow::anyhow!("No access token in response")))?;

    let access_token_claims = if let Some(introspect_url) = &config.introspect_url {
        // If introspect_url is set, use token introspection (RFC 7662)
        // This treats the access token as opaque and validates it via the introspection endpoint
        tracing::debug!("Using token introspection for access token");
        introspect_token(
            introspect_url,
            access_token,
            &config.client_id,
            &config.client_secret,
        )
        .await?
    } else {
        // Try to decode access token as JWT
        match decode_jwt_to_claims_unsafe(access_token, &config.jwks_endpoint, external_jwks_cache)
            .await
        {
            Ok(claims) => claims,
            Err(e) => {
                // Access token is not a JWT and no introspection endpoint configured
                // This is an error - we need to be able to get claims from the access token
                tracing::error!(
                    "Access token is not a JWT and no introspect_url configured: {:?}",
                    e
                );
                return Err(e);
            }
        }
    };

    // Build decoded token sources
    let sources = DecodedTokenSources::new().with_access_token(access_token_claims);

    // Apply mapping template
    let normalized = apply_token_mapping(&config.mapping, &sources)?;

    // Issue internal tokens
    let token_result =
        issue_tokens_for_normalized_user(repository, crypto_cache, normalized).await?;

    Ok(OAuthCallbackResult {
        issued_tokens: token_result,
        redirect_uri: oauth_state.redirect_uri.clone(),
    })
}

#[cfg(all(test, feature = "integration_test"))]
mod integration_test {
    use super::*;
    use crate::logic::token_mapping::TokenMapping;
    use crate::logic::token_mapping::template::{JwtTokenMappingConfig, MappingSource};
    use crate::test::dex::{
        DEX_AUTH_ENDPOINT, DEX_CLIENT_ID, DEX_CLIENT_SECRET, DEX_JWKS_ENDPOINT, DEX_OAUTH_SCOPES,
        DEX_REDIRECT_URI, DEX_TOKEN_ENDPOINT,
    };

    /// Create a test OAuth config using Dex endpoints (no OIDC/userinfo).
    fn create_test_oauth_config() -> OauthConfig {
        OauthConfig {
            id: "test-oauth".to_string(),
            authorization_endpoint: DEX_AUTH_ENDPOINT.to_string(),
            token_endpoint: DEX_TOKEN_ENDPOINT.to_string(),
            client_id: DEX_CLIENT_ID.to_string(),
            client_secret: DEX_CLIENT_SECRET.to_string(),
            scopes: DEX_OAUTH_SCOPES.iter().map(|s| s.to_string()).collect(),
            jwks_endpoint: DEX_JWKS_ENDPOINT.to_string(),
            introspect_url: None,
            mapping: create_test_mapping(),
        }
    }

    /// Create a minimal token mapping config for OAuth tests.
    /// For OAuth (no OIDC), claims come from the access token.
    fn create_test_mapping() -> TokenMapping {
        TokenMapping::JwtTemplate(JwtTokenMappingConfig {
            issuer_field: MappingSource::AccessToken("iss".to_string()),
            audience_field: MappingSource::AccessToken("aud".to_string()),
            scopes_field: None,
            sub_field: MappingSource::AccessToken("sub".to_string()),
            email_field: Some(MappingSource::AccessToken("email".to_string())),
            groups_field: None,
            group_to_role_mappings: vec![],
            scope_to_role_mappings: vec![],
            scope_to_group_mappings: vec![],
        })
    }

    #[tokio::test]
    async fn test_build_authorization_url() {
        let csrf_state = CsrfToken::new_random();

        let params = BaseAuthorizationParams {
            authorization_endpoint: DEX_AUTH_ENDPOINT,
            token_endpoint: DEX_TOKEN_ENDPOINT,
            redirect_uri: DEX_REDIRECT_URI,
            client_id: DEX_CLIENT_ID,
            scopes: &DEX_OAUTH_SCOPES
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
            pkce_challenge: None,
            csrf_state: &csrf_state,
            extra_params: vec![],
        };

        let url = build_authorization_url(params).expect("Failed to build authorization URL");

        // Verify URL structure
        assert!(url.starts_with(DEX_AUTH_ENDPOINT));
        assert!(url.contains(&format!("client_id={DEX_CLIENT_ID}")));
        assert!(url.contains(&format!(
            "redirect_uri={}",
            urlencoding::encode(DEX_REDIRECT_URI)
        )));
        assert!(url.contains("response_type=code"));
        assert!(url.contains(&format!("state={}", csrf_state.secret())));
    }

    #[tokio::test]
    async fn test_build_authorization_url_with_pkce() {
        let csrf_state = CsrfToken::new_random();
        let (pkce_challenge, _verifier) = oauth2::PkceCodeChallenge::new_random_sha256();

        let params = BaseAuthorizationParams {
            authorization_endpoint: DEX_AUTH_ENDPOINT,
            token_endpoint: DEX_TOKEN_ENDPOINT,
            redirect_uri: DEX_REDIRECT_URI,
            client_id: DEX_CLIENT_ID,
            scopes: &DEX_OAUTH_SCOPES
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
            pkce_challenge: Some(&pkce_challenge),
            csrf_state: &csrf_state,
            extra_params: vec![],
        };

        let url = build_authorization_url(params).expect("Failed to build authorization URL");

        // Verify PKCE parameters are included
        assert!(url.contains("code_challenge="));
        assert!(url.contains("code_challenge_method=S256"));
    }

    #[tokio::test]
    async fn test_build_authorization_url_with_extra_params() {
        let csrf_state = CsrfToken::new_random();

        let params = BaseAuthorizationParams {
            authorization_endpoint: DEX_AUTH_ENDPOINT,
            token_endpoint: DEX_TOKEN_ENDPOINT,
            redirect_uri: DEX_REDIRECT_URI,
            client_id: DEX_CLIENT_ID,
            scopes: &DEX_OAUTH_SCOPES
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
            pkce_challenge: None,
            csrf_state: &csrf_state,
            extra_params: vec![("nonce", "test-nonce-123".to_string())],
        };

        let url = build_authorization_url(params).expect("Failed to build authorization URL");

        // Verify extra parameters are included
        assert!(url.contains("nonce=test-nonce-123"));
    }

    #[tokio::test]
    async fn test_oauth_token_exchange_invalid_code() {
        let params = BaseTokenExchangeParams {
            token_endpoint: DEX_TOKEN_ENDPOINT,
            client_id: DEX_CLIENT_ID,
            client_secret: DEX_CLIENT_SECRET,
            redirect_uri: DEX_REDIRECT_URI,
            code: "invalid_oauth_code",
            code_verifier: None,
        };

        let result = exchange_code_for_tokens(params).await;

        assert!(
            result.is_err(),
            "Should fail with invalid authorization code"
        );
    }

    #[tokio::test]
    async fn test_oauth_token_exchange_wrong_redirect_uri() {
        let params = BaseTokenExchangeParams {
            token_endpoint: DEX_TOKEN_ENDPOINT,
            client_id: DEX_CLIENT_ID,
            client_secret: DEX_CLIENT_SECRET,
            redirect_uri: "http://wrong.example.com/callback",
            code: "some_code",
            code_verifier: None,
        };

        let result = exchange_code_for_tokens(params).await;

        assert!(result.is_err(), "Should fail with mismatched redirect URI");
    }

    #[tokio::test]
    async fn test_oauth_config_validation() {
        let config = create_test_oauth_config();

        // Verify OAuth config (no OIDC)
        assert_eq!(config.client_id, DEX_CLIENT_ID);
        assert_eq!(config.token_endpoint, DEX_TOKEN_ENDPOINT);
        assert_eq!(config.jwks_endpoint, DEX_JWKS_ENDPOINT);

        // OAuth config should NOT have openid scope
        assert!(
            !config.scopes.contains(&"openid".to_string()),
            "OAuth config should not include openid scope"
        );
    }

    #[tokio::test]
    async fn test_create_http_client() {
        let client = create_http_client();
        assert!(client.is_ok(), "Should create HTTP client successfully");
    }
}
