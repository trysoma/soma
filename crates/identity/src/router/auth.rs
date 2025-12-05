use axum::Json;
use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::cookie::CookieJar;
use http::HeaderMap;
use serde::{Deserialize, Serialize};
use shared::{adapters::openapi::API_VERSION_TAG, error::CommonError};
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::logic::auth_client::Identity;
use crate::logic::internal_token_issuance::{
    NormalizedTokenIssuanceResult, RefreshTokenParams, RefreshTokenResult, refresh_access_token,
};
use crate::logic::user_auth_flow::{OAuthCallbackParams, StartAuthorizationParams};
use crate::logic::user_auth_flow::{
    handle_authorization_handshake_callback, start_authorization_handshake,
};
use crate::service::IdentityService;

use super::{
    API_VERSION_1, PATH_PREFIX, REFRESH_TOKEN_COOKIE_NAME, SERVICE_ROUTE_KEY, add_token_cookies,
    add_token_cookies_with_options,
};

pub fn create_auth_routes() -> OpenApiRouter<IdentityService> {
    OpenApiRouter::new()
        .routes(routes!(route_start_authorization))
        .routes(routes!(route_auth_callback))
        .routes(routes!(route_refresh_token))
        .routes(routes!(route_whoami))
}

// ============================================
// Query/Request Types
// ============================================

#[derive(Debug, Deserialize, IntoParams)]
pub struct StartAuthorizationQuery {
    /// Optional override for where to redirect after successful login
    #[param(example = "/")]
    redirect_after_login: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct AuthCallbackQuery {
    /// Authorization code from the IdP
    #[param(example = "abc123")]
    code: Option<String>,
    /// State parameter for CSRF validation
    #[param(example = "xyz789")]
    state: Option<String>,
    /// Error from the IdP (if any)
    #[param(example = "access_denied")]
    error: Option<String>,
    /// Error description from the IdP
    #[param(example = "User denied access")]
    error_description: Option<String>,
}

/// Request body for refresh token endpoint
#[derive(Debug, Deserialize, ToSchema)]
pub struct RefreshTokenRequest {
    /// The refresh token. If not provided, will be read from cookie.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
}

/// Response body for token refresh endpoint
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TokenResponse {
    /// The access token
    pub access_token: String,
    /// The refresh token (only present on initial auth, not refresh)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    /// Token expiration time in seconds
    pub expires_in: i64,
    /// Token type (always "Bearer")
    pub token_type: String,
}

// ============================================
// Helper Functions
// ============================================

/// Extract refresh token from cookie jar
fn extract_refresh_token_from_jar(jar: &CookieJar) -> Option<String> {
    jar.get(REFRESH_TOKEN_COOKIE_NAME)
        .map(|cookie| cookie.value().to_string())
}

/// Build response with token cookies and JSON body for refresh (access token only)
fn build_refresh_token_response(jar: CookieJar, tokens: &RefreshTokenResult) -> Response {
    let body = TokenResponse {
        access_token: tokens.access_token.clone(),
        refresh_token: None,
        expires_in: tokens.expires_in,
        token_type: "Bearer".to_string(),
    };

    // Create a temporary NormalizedTokenIssuanceResult just for cookie setting
    // We pass include_refresh_token=false so the refresh_token field isn't used
    let temp_tokens = NormalizedTokenIssuanceResult {
        access_token: tokens.access_token.clone(),
        refresh_token: String::new(), // Won't be used since include_refresh_token=false
        expires_in: tokens.expires_in,
    };

    let jar = add_token_cookies_with_options(jar, &temp_tokens, false);

    (jar, Json(body)).into_response()
}

// ============================================
// Route Handlers
// ============================================

/// Start authorization flow - redirects to the IdP
#[utoipa::path(
    get,
    path = format!("{}/{}/{}/auth/authorize/{{config_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("config_id" = String, Path, description = "ID of the user auth flow configuration to use"),
        StartAuthorizationQuery
    ),
    responses(
        (status = 302, description = "Redirect to IdP authorization endpoint"),
        (status = 404, description = "Configuration not found", body = CommonError),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
    summary = "Start authorization",
    description = "Initiates the OAuth/OIDC authorization flow by redirecting to the external IdP",
)]
async fn route_start_authorization(
    State(service): State<IdentityService>,
    Path(config_id): Path<String>,
    Query(query): Query<StartAuthorizationQuery>,
) -> impl IntoResponse {
    let params = StartAuthorizationParams {
        config_id,
        redirect_after_login: query.redirect_after_login,
    };

    match start_authorization_handshake(
        service.repository.as_ref(),
        &service.crypto_cache,
        &service.base_redirect_uri,
        params,
    )
    .await
    {
        Ok(result) => Redirect::temporary(&result.login_redirect_url).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Authorization callback endpoint - handles the IdP response
#[utoipa::path(
    get,
    path = format!("{}/{}/{}/auth/callback", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        AuthCallbackQuery
    ),
    responses(
        (status = 302, description = "Redirect to post-login URL with tokens set"),
        (status = 400, description = "Invalid request or OAuth error", body = CommonError),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
    summary = "Authorization callback",
    description = "Handles the OAuth/OIDC callback from the external IdP, exchanges the authorization code for tokens",
)]
async fn route_auth_callback(
    State(service): State<IdentityService>,
    Query(query): Query<AuthCallbackQuery>,
    jar: CookieJar,
) -> impl IntoResponse {
    // Check for required parameters
    let state = match query.state {
        Some(s) => s,
        None => {
            return CommonError::InvalidRequest {
                msg: "Missing state parameter".to_string(),
                source: None,
            }
            .into_response();
        }
    };

    // Validate that code is present when there's no error from the IdP
    let code = match (&query.error, query.code) {
        (Some(_), _) => String::new(), // Error case - code doesn't matter
        (None, Some(c)) => c,
        (None, None) => {
            return CommonError::InvalidRequest {
                msg: "Missing authorization code parameter".to_string(),
                source: None,
            }
            .into_response();
        }
    };

    let params = OAuthCallbackParams {
        code,
        state,
        error: query.error,
        error_description: query.error_description,
    };

    match handle_authorization_handshake_callback(
        service.repository.as_ref(),
        &service.crypto_cache,
        &service.external_jwks_cache,
        params,
        &service.base_redirect_uri,
    )
    .await
    {
        Ok(result) => {
            // Add token cookies using the helper function
            let jar = add_token_cookies(jar, &result.issued_tokens);

            // Redirect to post-login URL (default to "/" if no redirect specified)
            let redirect_uri = result.redirect_uri.as_deref().unwrap_or("/");

            (jar, Redirect::to(redirect_uri)).into_response()
        }
        Err(e) => e.into_response(),
    }
}

/// Refresh access token
#[utoipa::path(
    post,
    path = format!("{}/{}/{}/auth/refresh", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body(content = Option<RefreshTokenRequest>, description = "Optional refresh token in body. If not provided, will be read from cookie."),
    responses(
        (status = 200, description = "Token refresh successful", body = TokenResponse),
        (status = 401, description = "Authentication failed", body = CommonError),
        (status = 404, description = "User not found", body = CommonError),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
    summary = "Refresh access token",
    description = "Refreshes an access token using a refresh token from the request body or cookie",
)]
async fn route_refresh_token(
    State(service): State<IdentityService>,
    jar: CookieJar,
    body: Option<Json<RefreshTokenRequest>>,
) -> Response {
    // Get refresh token from body or cookie
    let refresh_token = body
        .and_then(|b| b.refresh_token.clone())
        .or_else(|| extract_refresh_token_from_jar(&jar));

    let Some(refresh_token) = refresh_token else {
        return CommonError::Authentication {
            msg: "No refresh token provided in body or cookie".to_string(),
            source: None,
        }
        .into_response();
    };

    let params = RefreshTokenParams { refresh_token };

    let result = refresh_access_token(
        service.repository.as_ref(),
        &service.crypto_cache,
        &service.internal_jwks_cache,
        params,
    )
    .await;

    match result {
        Ok(token_result) => {
            // Only set access token cookie on refresh, don't update refresh token
            build_refresh_token_response(jar, &token_result)
        }
        Err(error) => error.into_response(),
    }
}

/// Get current authenticated identity
#[utoipa::path(
    get,
    path = format!("{}/{}/{}/auth/whoami", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    responses(
        (status = 200, description = "Current authenticated identity", body = Identity),
        (status = 401, description = "Authentication failed", body = CommonError),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
    summary = "Get current identity",
    description = "Returns the current authenticated identity based on the request headers (Authorization header, cookies, or API key)",
)]
async fn route_whoami(State(service): State<IdentityService>, headers: HeaderMap) -> Response {
    let auth_client = service.auth_client();

    match auth_client.authenticate_from_headers(&headers).await {
        Ok(identity) => Json(identity).into_response(),
        Err(error) => error.into_response(),
    }
}
