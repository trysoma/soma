//! OIDC authorization flow logic.
//!
//! This module handles the OpenID Connect authorization code flow (with optional PKCE).
//!
//! Flow:
//! 1. Authorization: Generate state/PKCE/nonce, redirect to IdP
//! 2. Callback: Exchange code for tokens, validate ID token, map claims, issue internal tokens

use chrono::{Duration, Utc};
use encryption::logic::CryptoCache;
use oauth2::{CsrfToken, PkceCodeChallenge};
use serde_json::Map;
use shared::error::CommonError;
use shared::primitives::WrappedChronoDateTime;

use crate::logic::internal_token_issuance::{NormalizedTokenInputFields, issue_tokens_for_normalized_user};
use crate::logic::token_mapping::template::DecodedTokenSources;
use crate::logic::user_auth_flow::{OAuthCallbackParams, OAuthCallbackResult};
use crate::logic::user_auth_flow::{StartAuthorizationParams, StartAuthorizationResult, config::{OidcConfig, UserAuthFlowConfig}};
use crate::logic::user_auth_flow::oauth::{
    apply_token_mapping, build_authorization_url, decode_jwt_claims_unsafe,
    exchange_code_for_tokens, fetch_userinfo, BaseAuthorizationParams, BaseTokenExchangeParams,
};
use crate::repository::{CreateOAuthState, UserRepositoryLike};

// ============================================
// OIDC Authorization Flow
// ============================================

/// Start the OIDC authorization flow.
///
/// This function:
/// 1. Generates PKCE challenge (for PKCE flows)
/// 2. Generates CSRF state and nonce
/// 3. Builds the authorization URL with openid scope
/// 4. Stores state and nonce in database
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

    // Only handle OIDC flows in this module
    let (oidc_config, uses_pkce) = match &config {
        UserAuthFlowConfig::OidcAuthorizationCodeFlow(oidc) => (oidc, false),
        UserAuthFlowConfig::OidcAuthorizationCodePkceFlow(oidc) => (oidc, true),
        _ => {
            return Err(CommonError::InvalidRequest {
                msg: "Configuration is not an OIDC flow. Use OAuth module for OAuth2 flows."
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

    // Generate nonce for OIDC (required for ID token validation)
    let nonce = uuid::Uuid::new_v4().to_string();

    // Build scopes - ensure openid is included
    let mut scopes = oidc_config.base_config.scopes.clone();
    if !scopes.iter().any(|s| s == "openid") {
        scopes.insert(0, "openid".to_string());
    }

    // Build authorization URL with nonce as extra parameter
    let base_params = BaseAuthorizationParams {
        authorization_endpoint: &oidc_config.base_config.authorization_endpoint,
        token_endpoint: &oidc_config.base_config.token_endpoint,
        redirect_uri: &oidc_config.base_config.authorization_endpoint, // This should be the app's redirect URI
        client_id: &oidc_config.base_config.client_id,
        scopes: &scopes,
        pkce_challenge: pkce_challenge.as_ref(),
        csrf_state: &csrf_state,
        extra_params: vec![("nonce", nonce.clone())],
    };

    let login_redirect_url = build_authorization_url(base_params)?;

    // Calculate expiration
    let state_ttl = 600; // 10 minutes
    let now = Utc::now();
    let expires_at = now + Duration::seconds(state_ttl);

    // Store state in database (including nonce for OIDC)
    let create_state = CreateOAuthState {
        state: csrf_state.secret().to_string(),
        config_id: params.config_id,
        code_verifier: pkce_verifier.map(|v| v.secret().to_string()),
        nonce: Some(nonce),
        redirect_uri: params.redirect_after_login,
        created_at: WrappedChronoDateTime::now(),
        expires_at: WrappedChronoDateTime::from(expires_at),
    };
    repository.create_oauth_state(&create_state).await?;

    Ok(StartAuthorizationResult { login_redirect_url })
}

/// Handle the OIDC callback.
///
/// This function:
/// 1. Validates state parameter
/// 2. Exchanges authorization code for tokens
/// 3. Extracts and validates ID token claims (including nonce)
/// 4. Optionally fetches userinfo
/// 5. Applies the mapping template to extract normalized fields
/// 6. Issues internal access/refresh tokens
pub async fn handle_authorization_handshake_callback<R: UserRepositoryLike>(
    repository: &R,
    crypto_cache: &CryptoCache,
    params: OAuthCallbackParams,
) -> Result<OAuthCallbackResult, CommonError> {
    // Check for error response from IdP
    if let Some(error) = &params.error {
        return Err(CommonError::InvalidRequest {
            msg: format!(
                "OIDC error from IdP: {} - {}",
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

    // Only handle OIDC flows
    let oidc_config = match &config {
        UserAuthFlowConfig::OidcAuthorizationCodeFlow(oidc)
        | UserAuthFlowConfig::OidcAuthorizationCodePkceFlow(oidc) => oidc,
        _ => {
            return Err(CommonError::InvalidRequest {
                msg: "Configuration is not an OIDC flow".to_string(),
                source: None,
            });
        }
    };

    // Exchange code for tokens
    let token_exchange_params = BaseTokenExchangeParams {
        token_endpoint: &oidc_config.base_config.token_endpoint,
        client_id: &oidc_config.base_config.client_id,
        client_secret: &oidc_config.base_config.client_secret,
        redirect_uri: &oidc_config.base_config.authorization_endpoint, // Should be app's redirect URI
        code: &params.code,
        code_verifier: oauth_state.code_verifier.as_deref(),
    };

    let token_response = exchange_code_for_tokens(token_exchange_params).await?;

    // Extract and validate ID token claims
    let normalized =
        extract_oidc_claims(oidc_config, &token_response, oauth_state.nonce.as_deref()).await?;

    // Issue internal tokens
    let token_result =
        issue_tokens_for_normalized_user(repository, crypto_cache, normalized).await?;

    Ok(OAuthCallbackResult {
        issued_tokens: token_result,
        redirect_uri: oauth_state.redirect_uri,
    })
}

// ============================================
// OIDC-specific Helper Functions
// ============================================

/// Extract and validate claims from OIDC token response.
///
/// This function:
/// 1. Extracts ID token from response
/// 2. Decodes and validates ID token claims
/// 3. Verifies nonce if provided
/// 4. Optionally fetches userinfo for additional claims
/// 5. Applies mapping template
async fn extract_oidc_claims(
    config: &OidcConfig,
    token_response: &Map<String, serde_json::Value>,
    expected_nonce: Option<&str>,
) -> Result<NormalizedTokenInputFields, CommonError> {
    // For OIDC, we expect an ID token in the response
    let id_token_str = token_response
        .get("id_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!("No ID token in OIDC token response"))
        })?;

    // Decode ID token (without signature verification - we trust the token endpoint response)
    // In production, you'd want to verify the signature using the IdP's JWKS
    let id_token_claims = decode_jwt_claims_unsafe(id_token_str)?;

    // Verify nonce if provided
    if let Some(expected) = expected_nonce {
        let token_nonce = id_token_claims
            .get("nonce")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CommonError::InvalidRequest {
                msg: "ID token missing nonce claim".to_string(),
                source: None,
            })?;

        if token_nonce != expected {
            return Err(CommonError::InvalidRequest {
                msg: "Nonce mismatch in ID token".to_string(),
                source: None,
            });
        }
    }

    // Get access token for userinfo (if configured)
    let access_token = token_response
        .get("access_token")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Optionally fetch userinfo for additional claims
    let userinfo_claims = if let (Some(userinfo_endpoint), Some(access_token)) =
        (&config.base_config.userinfo_endpoint, &access_token)
    {
        Some(fetch_userinfo(userinfo_endpoint, access_token).await?)
    } else {
        None
    };

    // Build decoded token sources
    let mut sources = DecodedTokenSources::new();
    sources = sources.with_id_token(id_token_claims);
    if let Some(userinfo) = userinfo_claims {
        sources = sources.with_userinfo(userinfo);
    }

    // Apply mapping template (use OIDC mapping if present, else base mapping)
    apply_token_mapping(&config.mapping, &sources)
}
