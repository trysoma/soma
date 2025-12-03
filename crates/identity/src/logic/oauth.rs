//! OAuth/OIDC authorization flow logic.
//!
//! This module handles the OAuth2/OIDC authorization code flow using the
//! `openidconnect` and `oauth2` crates for standards-compliant implementation.
//!
//! Flow:
//! 1. Authorization: Generate state/PKCE, redirect to IdP
//! 2. Callback: Exchange code for tokens, map claims, issue internal tokens

use chrono::{Duration, Utc};
use encryption::logic::CryptoCache;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, Scope, TokenUrl,
};
use openidconnect::core::{
    CoreClient, CoreIdTokenClaims, CoreIdTokenVerifier, CoreProviderMetadata,
};
use openidconnect::{
    AccessTokenHash, IssuerUrl, Nonce, OAuth2TokenResponse, TokenResponse as OidcTokenResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shared::error::CommonError;
use shared::primitives::WrappedChronoDateTime;
use utoipa::ToSchema;

use crate::logic::auth_client::Role;
use crate::logic::auth_config::{standardize_group_name, NormalizedStsFields};
use crate::logic::idp_config::{
    load_idp_config, IdpConfig, OauthConfig, OidcConfig, OidcMappingConfig, TokenSource,
};
use crate::logic::sts_exchange::issue_tokens_for_normalized_user;
use crate::repository::{CreateOAuthState, UserRepositoryLike};

// ============================================
// Authorization Flow Types
// ============================================

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
    pub authorization_url: String,
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
    pub refresh_token: Option<String>,
    /// Token expiration in seconds
    pub expires_in: i64,
    /// Where to redirect the user after login
    pub redirect_uri: String,
}

// ============================================
// HTTP Client Helper
// ============================================

/// Create an HTTP client for OAuth/OIDC requests
fn create_http_client() -> Result<reqwest::Client, CommonError> {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to create HTTP client: {e}")))
}

// ============================================
// Authorization Flow Logic
// ============================================

/// Start the OAuth/OIDC authorization flow.
///
/// This function:
/// 1. Loads the IdP configuration
/// 2. Builds the appropriate client (OIDC or OAuth2)
/// 3. Generates state, PKCE, and nonce
/// 4. Stores state in database
/// 5. Returns the authorization URL to redirect to
pub async fn start_authorization<R: UserRepositoryLike>(
    repository: &R,
    crypto_cache: &CryptoCache,
    params: StartAuthorizationParams,
) -> Result<StartAuthorizationResult, CommonError> {
    // 1. Load IdP configuration
    let (_db_config, idp_config) = load_idp_config(repository, crypto_cache, &params.config_id).await?;

    // 2. Generate PKCE if enabled
    let (pkce_challenge, pkce_verifier) = if idp_config.uses_pkce() {
        let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
        (Some(challenge), Some(verifier))
    } else {
        (None, None)
    };

    // 3. Generate CSRF state
    let csrf_state = CsrfToken::new_random();

    // 4. Generate nonce for OIDC
    let nonce = if idp_config.is_oidc() {
        Some(Nonce::new_random())
    } else {
        None
    };

    // Get OAuth config for common fields
    let oauth_config = idp_config.oauth_config();

    // 5. Build authorization URL based on flow type
    let authorization_url = match &idp_config {
        IdpConfig::OidcAuthorizationCodeFlow(oidc) | IdpConfig::OidcAuthorizationCodePkceFlow(oidc) => {
            build_oidc_authorization_url(oidc, &csrf_state, nonce.as_ref(), pkce_challenge.as_ref()).await?
        }
        IdpConfig::OauthAuthorizationCodeFlow(oauth) | IdpConfig::OauthAuthorizationCodePkceFlow(oauth) => {
            build_oauth2_authorization_url(oauth, &csrf_state, pkce_challenge.as_ref())?
        }
    };

    // 6. Calculate expiration
    let now = Utc::now();
    let expires_at = now + Duration::seconds(oauth_config.state_ttl_seconds as i64);

    // 7. Store state in database
    let redirect_uri = params
        .redirect_after_login
        .unwrap_or(oauth_config.post_login_redirect_uri.clone());

    let create_state = CreateOAuthState {
        state: csrf_state.secret().to_string(),
        config_id: params.config_id,
        code_verifier: pkce_verifier.map(|v| v.secret().to_string()),
        nonce: nonce.map(|n| n.secret().to_string()),
        redirect_uri: Some(redirect_uri),
        created_at: WrappedChronoDateTime::now(),
        expires_at: WrappedChronoDateTime::from(expires_at),
    };
    repository.create_oauth_state(&create_state).await?;

    Ok(StartAuthorizationResult { authorization_url })
}

/// Build OIDC authorization URL using discovery
async fn build_oidc_authorization_url(
    config: &OidcConfig,
    csrf_state: &CsrfToken,
    nonce: Option<&Nonce>,
    pkce_challenge: Option<&PkceCodeChallenge>,
) -> Result<String, CommonError> {
    let discovery_endpoint = config.discovery_endpoint.as_ref().ok_or_else(|| {
        CommonError::InvalidRequest {
            msg: "discovery_endpoint is required for OIDC flows".to_string(),
            source: None,
        }
    })?;

    // Extract issuer from discovery endpoint (remove /.well-known/openid-configuration)
    let issuer_url = discovery_endpoint
        .strip_suffix("/.well-known/openid-configuration")
        .unwrap_or(discovery_endpoint);

    let issuer = IssuerUrl::new(issuer_url.to_string()).map_err(|e| {
        CommonError::InvalidRequest {
            msg: format!("Invalid issuer URL: {e}"),
            source: None,
        }
    })?;

    let http_client = create_http_client()?;

    // Discover provider metadata
    let provider_metadata =
        CoreProviderMetadata::discover_async(issuer, &http_client)
            .await
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!(
                    "Failed to discover OIDC provider metadata: {e}"
                ))
            })?;

    let redirect_url = RedirectUrl::new(config.base_config.redirect_uri.clone()).map_err(|e| {
        CommonError::InvalidRequest {
            msg: format!("Invalid redirect URI: {e}"),
            source: None,
        }
    })?;

    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(config.base_config.client_id.clone()),
        None, // No client secret needed for authorization URL
    )
    .set_redirect_uri(redirect_url);

    let nonce_for_closure = nonce.cloned();
    let csrf_for_closure = csrf_state.clone();

    let mut auth_request = client.authorize_url(
        openidconnect::core::CoreAuthenticationFlow::AuthorizationCode,
        move || csrf_for_closure.clone(),
        move || nonce_for_closure.clone().unwrap_or_else(Nonce::new_random),
    );

    // Add scopes
    for scope in &config.base_config.scopes {
        auth_request = auth_request.add_scope(Scope::new(scope.clone()));
    }

    // Add PKCE challenge
    if let Some(challenge) = pkce_challenge {
        auth_request = auth_request.set_pkce_challenge(challenge.clone());
    }

    let (url, _, _) = auth_request.url();
    Ok(url.to_string())
}

/// Build OAuth2 authorization URL manually
fn build_oauth2_authorization_url(
    config: &OauthConfig,
    csrf_state: &CsrfToken,
    pkce_challenge: Option<&PkceCodeChallenge>,
) -> Result<String, CommonError> {
    let auth_url = AuthUrl::new(config.authorization_endpoint.clone())
        .map_err(|e| CommonError::InvalidRequest {
            msg: format!("Invalid authorization endpoint: {e}"),
            source: None,
        })?;

    let token_url = TokenUrl::new(config.token_endpoint.clone())
        .map_err(|e| CommonError::InvalidRequest {
            msg: format!("Invalid token endpoint: {e}"),
            source: None,
        })?;

    let redirect_url = RedirectUrl::new(config.redirect_uri.clone()).map_err(|e| {
        CommonError::InvalidRequest {
            msg: format!("Invalid redirect URI: {e}"),
            source: None,
        }
    })?;

    let client = oauth2::basic::BasicClient::new(ClientId::new(config.client_id.clone()))
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(redirect_url);

    let csrf_for_closure = csrf_state.clone();
    let mut auth_request = client.authorize_url(move || csrf_for_closure.clone());

    // Add scopes
    for scope in &config.scopes {
        auth_request = auth_request.add_scope(Scope::new(scope.clone()));
    }

    // Add PKCE challenge
    if let Some(challenge) = pkce_challenge {
        auth_request = auth_request.set_pkce_challenge(challenge.clone());
    }

    let (url, _) = auth_request.url();
    Ok(url.to_string())
}

/// Handle the OAuth/OIDC callback.
///
/// This function:
/// 1. Validates state parameter
/// 2. Exchanges authorization code for tokens
/// 3. For OIDC: validates and extracts claims from ID token
/// 4. For OAuth2: calls userinfo endpoint
/// 5. Maps claims to normalized fields
/// 6. Issues internal access/refresh tokens
pub async fn handle_callback<R: UserRepositoryLike>(
    repository: &R,
    crypto_cache: &CryptoCache,
    params: OAuthCallbackParams,
) -> Result<OAuthCallbackResult, CommonError> {
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

    // 1. Validate state and get stored data
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

    // 2. Load IdP configuration (already decrypted)
    let (_db_config, idp_config) = load_idp_config(repository, crypto_cache, &oauth_state.config_id).await?;

    // Get OAuth config for common fields
    let oauth_config = idp_config.oauth_config();

    // 3. Exchange code and extract claims based on flow type
    let normalized = match &idp_config {
        IdpConfig::OidcAuthorizationCodeFlow(oidc) | IdpConfig::OidcAuthorizationCodePkceFlow(oidc) => {
            exchange_oidc_code(
                oidc,
                &params.code,
                oauth_state.code_verifier.as_deref(),
                oauth_state.nonce.as_deref(),
            )
            .await?
        }
        IdpConfig::OauthAuthorizationCodeFlow(oauth) | IdpConfig::OauthAuthorizationCodePkceFlow(oauth) => {
            exchange_oauth2_code(
                oauth,
                &params.code,
                oauth_state.code_verifier.as_deref(),
            )
            .await?
        }
    };

    // 4. Validate email domain if configured
    if !oauth_config.allowed_domains.is_empty() {
        if let Some(email) = &normalized.email {
            let domain = email.split('@').last().unwrap_or("");
            if !oauth_config.allowed_domains.contains(&domain.to_string()) {
                return Err(CommonError::InvalidRequest {
                    msg: format!("Email domain '{}' is not allowed", domain),
                    source: None,
                });
            }
        }
    }

    // 5. Issue internal tokens
    let token_result =
        issue_tokens_for_normalized_user(repository, crypto_cache, normalized).await?;

    Ok(OAuthCallbackResult {
        access_token: token_result.access_token,
        refresh_token: token_result.refresh_token,
        expires_in: token_result.expires_in,
        redirect_uri: oauth_state
            .redirect_uri
            .unwrap_or(oauth_config.post_login_redirect_uri.clone()),
    })
}

// ============================================
// OIDC Token Exchange
// ============================================

/// Exchange authorization code for tokens using OIDC flow
async fn exchange_oidc_code(
    config: &OidcConfig,
    code: &str,
    code_verifier: Option<&str>,
    nonce: Option<&str>,
) -> Result<NormalizedStsFields, CommonError> {
    let discovery_endpoint = config.discovery_endpoint.as_ref().ok_or_else(|| {
        CommonError::InvalidRequest {
            msg: "discovery_endpoint is required for OIDC flows".to_string(),
            source: None,
        }
    })?;

    // Extract issuer from discovery endpoint (remove /.well-known/openid-configuration)
    let issuer_url = discovery_endpoint
        .strip_suffix("/.well-known/openid-configuration")
        .unwrap_or(discovery_endpoint);

    let issuer = IssuerUrl::new(issuer_url.to_string()).map_err(|e| {
        CommonError::InvalidRequest {
            msg: format!("Invalid issuer URL: {e}"),
            source: None,
        }
    })?;

    let http_client = create_http_client()?;

    // Discover provider metadata
    let provider_metadata =
        CoreProviderMetadata::discover_async(issuer, &http_client)
            .await
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!(
                    "Failed to discover OIDC provider metadata: {e}"
                ))
            })?;

    let redirect_url = RedirectUrl::new(config.base_config.redirect_uri.clone()).map_err(|e| {
        CommonError::InvalidRequest {
            msg: format!("Invalid redirect URI: {e}"),
            source: None,
        }
    })?;

    let client = CoreClient::from_provider_metadata(
        provider_metadata.clone(),
        ClientId::new(config.base_config.client_id.clone()),
        Some(ClientSecret::new(config.base_config.client_secret.clone())),
    )
    .set_redirect_uri(redirect_url);

    // Build token request
    let mut token_request = client
        .exchange_code(AuthorizationCode::new(code.to_string()))
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to build token request: {e}")))?;

    // Add PKCE verifier if present
    if let Some(verifier) = code_verifier {
        token_request = token_request.set_pkce_verifier(PkceCodeVerifier::new(verifier.to_string()));
    }

    // Exchange code for tokens
    let token_response = token_request
        .request_async(&http_client)
        .await
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Token exchange failed: {e}")))?;

    // Get ID token
    let id_token = token_response.id_token().ok_or_else(|| {
        CommonError::Unknown(anyhow::anyhow!("No ID token in OIDC token response"))
    })?;

    // Verify ID token
    let id_token_verifier: CoreIdTokenVerifier<'_> = client.id_token_verifier();
    let nonce_verifier = nonce.map(|n| Nonce::new(n.to_string()));

    let claims: &CoreIdTokenClaims = id_token
        .claims(&id_token_verifier, |_: Option<&Nonce>| {
            // Manual nonce verification
            Ok(())
        })
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("ID token verification failed: {e}")))?;

    // Verify nonce manually if provided
    if let Some(expected_nonce) = &nonce_verifier {
        match claims.nonce() {
            Some(claim_nonce) if claim_nonce.secret() == expected_nonce.secret() => {
                // Nonce matches
            }
            _ => {
                return Err(CommonError::InvalidRequest {
                    msg: "Nonce mismatch in ID token".to_string(),
                    source: None,
                });
            }
        }
    }

    // Verify access token hash if present
    if let Some(expected_access_token_hash) = claims.access_token_hash() {
        let actual_access_token_hash = AccessTokenHash::from_token(
            token_response.access_token(),
            id_token.signing_alg().map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to get signing algorithm: {e}"))
            })?,
            id_token.signing_key(&id_token_verifier).map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to get signing key: {e}"))
            })?,
        )
        .map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!("Failed to compute access token hash: {e}"))
        })?;

        if actual_access_token_hash != *expected_access_token_hash {
            return Err(CommonError::InvalidRequest {
                msg: "Access token hash mismatch".to_string(),
                source: None,
            });
        }
    }

    // Extract claims based on OIDC mapping config
    let (subject, email, mut groups) = extract_oidc_claims(
        claims,
        &config.oidc_mapping_config,
        config.base_config.userinfo_endpoint.as_deref(),
        token_response.access_token(),
    )
    .await?;

    // Map scopes to groups
    let scope_groups = config.base_config.map_scopes_to_groups(&groups);
    for g in scope_groups {
        if !groups.contains(&g) {
            groups.push(g);
        }
    }

    // Determine role from groups or use default
    let role = config
        .base_config
        .determine_role_from_groups(&groups)
        .unwrap_or_else(|| Role::from_str(&config.base_config.default_role).unwrap_or(Role::User));

    Ok(NormalizedStsFields {
        subject,
        email,
        groups,
        role,
    })
}

/// Extract claims from OIDC ID token and optionally userinfo
async fn extract_oidc_claims(
    claims: &CoreIdTokenClaims,
    mapping: &OidcMappingConfig,
    userinfo_endpoint: Option<&str>,
    access_token: &oauth2::AccessToken,
) -> Result<(String, Option<String>, Vec<String>), CommonError> {
    // Extract subject based on mapping
    let subject = match &mapping.sub_field {
        TokenSource::IdToken(_) | TokenSource::AccessToken(_) => claims.subject().to_string(),
        TokenSource::Userinfo(field) => {
            if let Some(endpoint) = userinfo_endpoint {
                let userinfo = fetch_userinfo_as_json(endpoint, access_token).await?;
                userinfo
                    .get(field)
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .ok_or_else(|| {
                        CommonError::Unknown(anyhow::anyhow!("Missing subject in userinfo"))
                    })?
            } else {
                return Err(CommonError::InvalidRequest {
                    msg: "userinfo_endpoint required for userinfo subject".to_string(),
                    source: None,
                });
            }
        }
    };

    // Extract email based on mapping
    let email = if let Some(email_source) = &mapping.email_field {
        match email_source {
            TokenSource::IdToken(_) | TokenSource::AccessToken(_) => claims.email().map(|e| e.to_string()),
            TokenSource::Userinfo(field) => {
                if let Some(endpoint) = userinfo_endpoint {
                    if let Ok(userinfo) = fetch_userinfo_as_json(endpoint, access_token).await {
                        userinfo
                            .get(field)
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    } else {
        None
    };

    // Extract groups based on mapping
    let groups = if let Some(groups_source) = &mapping.groups_field {
        match groups_source {
            TokenSource::IdToken(_) | TokenSource::AccessToken(_) => {
                // Groups typically come from userinfo, not ID/access token
                Vec::new()
            }
            TokenSource::Userinfo(field) => {
                if let Some(endpoint) = userinfo_endpoint {
                    if let Ok(userinfo) = fetch_userinfo_as_json(endpoint, access_token).await {
                        if let Some(value) = userinfo.get(field) {
                            extract_groups_from_json_value(value)
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            }
        }
    } else {
        Vec::new()
    };

    Ok((subject, email, groups))
}

// ============================================
// OAuth2 Token Exchange
// ============================================

/// Exchange authorization code for tokens using plain OAuth2 flow
async fn exchange_oauth2_code(
    config: &OauthConfig,
    code: &str,
    code_verifier: Option<&str>,
) -> Result<NormalizedStsFields, CommonError> {
    let auth_url = AuthUrl::new(config.authorization_endpoint.clone())
        .map_err(|e| CommonError::InvalidRequest {
            msg: format!("Invalid authorization endpoint: {e}"),
            source: None,
        })?;

    let token_url = TokenUrl::new(config.token_endpoint.clone())
        .map_err(|e| CommonError::InvalidRequest {
            msg: format!("Invalid token endpoint: {e}"),
            source: None,
        })?;

    let redirect_url = RedirectUrl::new(config.redirect_uri.clone()).map_err(|e| {
        CommonError::InvalidRequest {
            msg: format!("Invalid redirect URI: {e}"),
            source: None,
        }
    })?;

    let client = oauth2::basic::BasicClient::new(ClientId::new(config.client_id.clone()))
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(redirect_url)
        .set_client_secret(ClientSecret::new(config.client_secret.clone()));

    let http_client = create_http_client()?;

    // Build token request
    let mut token_request = client.exchange_code(AuthorizationCode::new(code.to_string()));

    // Add PKCE verifier if present
    if let Some(verifier) = code_verifier {
        token_request = token_request.set_pkce_verifier(PkceCodeVerifier::new(verifier.to_string()));
    }

    // Exchange code for tokens
    let token_response = token_request
        .request_async(&http_client)
        .await
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Token exchange failed: {e}")))?;

    // For OAuth2, we need to call userinfo endpoint
    let userinfo_endpoint = config.userinfo_endpoint.as_ref().ok_or_else(|| {
        CommonError::InvalidRequest {
            msg: "userinfo_endpoint is required for OAuth2 flows".to_string(),
            source: None,
        }
    })?;

    let userinfo = fetch_userinfo_as_json(userinfo_endpoint, token_response.access_token()).await?;

    // Extract subject using mapping config
    let subject = userinfo
        .get(&config.mapping_config.sub_field)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| CommonError::Unknown(anyhow::anyhow!("Missing subject claim")))?;

    // Extract email using mapping config
    let email = config
        .mapping_config
        .email_field
        .as_ref()
        .and_then(|field| userinfo.get(field))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Extract groups using mapping config
    let mut groups = if let Some(groups_field) = &config.mapping_config.groups_field {
        userinfo
            .get(groups_field)
            .map(|v| extract_groups_from_json_value(v))
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    // Map scopes to groups
    let scope_groups = config.map_scopes_to_groups(&groups);
    for g in scope_groups {
        if !groups.contains(&g) {
            groups.push(g);
        }
    }

    // Determine role from groups or use default
    let role = config
        .determine_role_from_groups(&groups)
        .unwrap_or_else(|| Role::from_str(&config.default_role).unwrap_or(Role::User));

    Ok(NormalizedStsFields {
        subject,
        email,
        groups,
        role,
    })
}

/// Fetch userinfo from endpoint as JSON
async fn fetch_userinfo_as_json(
    endpoint: &str,
    access_token: &oauth2::AccessToken,
) -> Result<Value, CommonError> {
    let client = reqwest::Client::new();
    let response = client
        .get(endpoint)
        .bearer_auth(access_token.secret())
        .send()
        .await
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Userinfo request failed: {e}")))?;

    if !response.status().is_success() {
        return Err(CommonError::Unknown(anyhow::anyhow!(
            "Userinfo request failed: HTTP {}",
            response.status()
        )));
    }

    response
        .json()
        .await
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to parse userinfo: {e}")))
}

// ============================================
// Claim Extraction Helpers
// ============================================

/// Extract groups from a JSON value (simple version without scope mapping)
fn extract_groups_from_json_value(value: &Value) -> Vec<String> {
    let mut groups = Vec::new();

    if let Some(arr) = value.as_array() {
        for item in arr {
            if let Some(s) = item.as_str() {
                groups.push(standardize_group_name(s));
            }
        }
    } else if let Some(s) = value.as_str() {
        // Space or comma separated
        for g in s.split(|c| c == ' ' || c == ',') {
            let g = g.trim();
            if !g.is_empty() {
                groups.push(standardize_group_name(g));
            }
        }
    }

    groups
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_groups_from_array() {
        let value = serde_json::json!(["Admin", "Users", "Developers"]);
        let groups = extract_groups_from_json_value(&value);

        assert_eq!(groups.len(), 3);
        assert!(groups.contains(&"admin".to_string()));
        assert!(groups.contains(&"users".to_string()));
        assert!(groups.contains(&"developers".to_string()));
    }

    #[test]
    fn test_extract_groups_from_string() {
        let value = serde_json::json!("admin,users, developers");
        let groups = extract_groups_from_json_value(&value);

        assert_eq!(groups.len(), 3);
        assert!(groups.contains(&"admin".to_string()));
        assert!(groups.contains(&"users".to_string()));
        assert!(groups.contains(&"developers".to_string()));
    }

    #[test]
    fn test_extract_groups_with_spaces() {
        let value = serde_json::json!("admin users developers");
        let groups = extract_groups_from_json_value(&value);

        assert_eq!(groups.len(), 3);
        assert!(groups.contains(&"admin".to_string()));
        assert!(groups.contains(&"users".to_string()));
        assert!(groups.contains(&"developers".to_string()));
    }

    #[test]
    fn test_extract_groups_empty() {
        let value = serde_json::json!([]);
        let groups = extract_groups_from_json_value(&value);
        assert!(groups.is_empty());

        let value = serde_json::json!("");
        let groups = extract_groups_from_json_value(&value);
        assert!(groups.is_empty());
    }
}
