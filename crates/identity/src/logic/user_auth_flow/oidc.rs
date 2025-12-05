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

use crate::logic::internal_token_issuance::{
    NormalizedTokenInputFields, issue_tokens_for_normalized_user,
};
use crate::logic::sts::external_jwk_cache::ExternalJwksCache;
use crate::logic::token_mapping::template::DecodedTokenSources;
use crate::logic::user_auth_flow::oauth::{
    BaseAuthorizationParams, BaseTokenExchangeParams, OAuthState, apply_token_mapping,
    build_authorization_url, exchange_code_for_tokens,
};
use crate::logic::user_auth_flow::{OAuthCallbackParams, OAuthCallbackResult, OauthConfig};
use crate::logic::user_auth_flow::{
    StartAuthorizationParams, StartAuthorizationResult,
    config::{OidcConfig, UserAuthFlowConfig},
};
use crate::logic::{decode_jwt_to_claims_unsafe, fetch_userinfo, introspect_token};
use crate::repository::UserRepositoryLike;
use crate::router::{API_VERSION_1, PATH_PREFIX, SERVICE_ROUTE_KEY};

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
        redirect_uri: &format!("{}{}/{}/{}/auth/callback", base_redirect_uri, PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
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
    let create_state = OAuthState {
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
    external_jwks_cache: &ExternalJwksCache,
    base_redirect_uri: &str,
    params: OAuthCallbackParams,
    config: &OidcConfig,
    oauth_state: &OAuthState,
) -> Result<OAuthCallbackResult, CommonError> {
    tracing::debug!("OIDC callback: exchanging code for tokens");

    // Exchange code for tokens
    let token_exchange_params = BaseTokenExchangeParams {
        token_endpoint: &config.base_config.token_endpoint,
        client_id: &config.base_config.client_id,
        client_secret: &config.base_config.client_secret,
        redirect_uri: &format!("{}{}/{}/{}/auth/callback", base_redirect_uri, PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
        code: &params.code,
        code_verifier: None,
    };

    let token_response = exchange_code_for_tokens(token_exchange_params).await.map_err(|e| {
        tracing::error!("OIDC callback: token exchange failed: {:?}", e);
        e
    })?;

    tracing::debug!("OIDC callback: extracting claims from token response");

    // Extract and validate ID token claims
    let normalized = extract_oidc_claims(
        external_jwks_cache,
        config,
        &token_response,
        oauth_state.nonce.as_deref(),
    )
    .await
    .map_err(|e| {
        tracing::error!("OIDC callback: claim extraction failed: {:?}", e);
        e
    })?;

    tracing::debug!("OIDC callback: issuing internal tokens for user");

    // Issue internal tokens
    let token_result =
        issue_tokens_for_normalized_user(repository, crypto_cache, normalized).await.map_err(|e| {
            tracing::error!("OIDC callback: token issuance failed: {:?}", e);
            e
        })?;

    tracing::info!("OIDC callback: successfully authenticated user");

    Ok(OAuthCallbackResult {
        issued_tokens: token_result,
        redirect_uri: oauth_state.redirect_uri.clone(),
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
    external_jwks_cache: &ExternalJwksCache,
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
    let id_token_claims = decode_jwt_to_claims_unsafe(
        id_token_str,
        &config.base_config.jwks_endpoint,
        external_jwks_cache,
    )
    .await?;

    // Get access token for userinfo (if configured)
    let access_token = match token_response
        .get("access_token")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string()) {
            Some(access_token) => access_token,
            None => return Err(CommonError::InvalidRequest {
                msg: "No access token in OIDC token response".to_string(),
                source: None,
            }),
        };

    // Get access token claims - either via introspection or JWT decoding
    let access_token_claims = if let Some(introspect_url) = &config.introspect_url {
        // If introspect_url is set, use token introspection (RFC 7662)
        // This treats the access token as opaque and validates it via the introspection endpoint
        tracing::debug!("Using token introspection for access token");
        Some(introspect_token(
            introspect_url,
            &access_token,
            &config.base_config.client_id,
            &config.base_config.client_secret,
        ).await?)
    } else {
        // Try to decode access token as JWT
        match decode_jwt_to_claims_unsafe(
            &access_token,
            &config.base_config.jwks_endpoint,
            external_jwks_cache,
        )
        .await {
            Ok(claims) => Some(claims),
            Err(e) => {
                // Access token is not a JWT and no introspection endpoint configured
                // This is an error - we need to be able to get claims from the access token
                tracing::error!("Access token is not a JWT and no introspect_url configured: {:?}", e);
                return Err(e)
            }
        }
    };

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

    
    // Optionally fetch userinfo for additional claims
    let userinfo_claims = match &config.userinfo_endpoint {
        Some(userinfo_endpoint) => {
            Some(fetch_userinfo(&userinfo_endpoint, &access_token).await?)
        }
        None => None,
    };

    // Build decoded token sources
    let mut sources = DecodedTokenSources::new();
    sources = sources.with_id_token(id_token_claims);

    if let Some(claims) = access_token_claims {
        sources = sources.with_access_token(claims);
    }

    if let Some(userinfo) = userinfo_claims {
        sources = sources.with_userinfo(userinfo);
    }

    // Apply mapping template (use OIDC mapping if present, else base mapping)
    apply_token_mapping(&config.mapping, &sources)
}
