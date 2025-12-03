use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use axum::Json;
use serde::Deserialize;
use shared::{adapters::openapi::API_VERSION_TAG, error::CommonError};
use utoipa::IntoParams;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::logic::oauth::{handle_callback, start_authorization, OAuthCallbackParams, StartAuthorizationParams};
use crate::service::IdentityService;

use super::{
    ACCESS_TOKEN_COOKIE_NAME, API_VERSION_1, PATH_PREFIX, REFRESH_TOKEN_COOKIE_NAME,
    SERVICE_ROUTE_KEY,
};

pub fn create_oauth_routes() -> OpenApiRouter<IdentityService> {
    OpenApiRouter::new()
        .routes(routes!(route_start_authorization))
        .routes(routes!(route_oauth_callback))
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct StartAuthorizationQuery {
    /// Optional override for where to redirect after successful login
    #[param(example = "/dashboard")]
    redirect_after_login: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct OAuthCallbackQuery {
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

/// Start OAuth authorization flow - redirects to the IdP
#[utoipa::path(
    get,
    path = format!("{}/{}/{}/oauth/authorize/{{config_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("config_id" = String, Path, description = "ID of the IdP configuration to use"),
        StartAuthorizationQuery
    ),
    responses(
        (status = 302, description = "Redirect to IdP authorization endpoint"),
        (status = 404, description = "IdP configuration not found", body = CommonError),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
    summary = "Start OAuth authorization",
    description = "Initiates the OAuth/OIDC authorization flow by redirecting to the external IdP",
)]
async fn route_start_authorization(
    State(service): State<IdentityService>,
    Path(config_id): Path<String>,
    Query(query): Query<StartAuthorizationQuery>,
) -> Response {
    let params = StartAuthorizationParams {
        config_id,
        redirect_after_login: query.redirect_after_login,
    };

    match start_authorization(service.repository.as_ref(), &service.crypto_cache, params).await {
        Ok(result) => Redirect::temporary(&result.authorization_url).into_response(),
        Err(e) => {
            let status = match &e {
                CommonError::NotFound { .. } => StatusCode::NOT_FOUND,
                CommonError::InvalidRequest { .. } => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(e)).into_response()
        }
    }
}

/// OAuth callback endpoint - handles the IdP response
#[utoipa::path(
    get,
    path = format!("{}/{}/{}/oauth/callback/{{config_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("config_id" = String, Path, description = "ID of the IdP configuration"),
        OAuthCallbackQuery
    ),
    responses(
        (status = 302, description = "Redirect to post-login URL with tokens set"),
        (status = 400, description = "Invalid request or OAuth error", body = CommonError),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
    summary = "OAuth callback",
    description = "Handles the OAuth/OIDC callback from the external IdP, exchanges the authorization code for tokens",
)]
async fn route_oauth_callback(
    State(service): State<IdentityService>,
    Path(_config_id): Path<String>,
    Query(query): Query<OAuthCallbackQuery>,
) -> Response {
    // Check for required parameters
    let state = match query.state {
        Some(s) => s,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CommonError::InvalidRequest {
                    msg: "Missing state parameter".to_string(),
                    source: None,
                }),
            )
                .into_response();
        }
    };

    let code = query.code.unwrap_or_default();

    let params = OAuthCallbackParams {
        code,
        state,
        error: query.error,
        error_description: query.error_description,
    };

    match handle_callback(
        service.repository.as_ref(),
        &service.crypto_cache,
        params,
    )
    .await
    {
        Ok(result) => {
            // Set cookies and redirect
            let mut headers = HeaderMap::new();

            // Access token cookie (manually build cookie string)
            let access_cookie = format!(
                "{}={}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age={}",
                ACCESS_TOKEN_COOKIE_NAME, result.access_token, result.expires_in
            );
            headers.insert(header::SET_COOKIE, access_cookie.parse().unwrap());

            // Refresh token cookie (if present)
            if let Some(ref refresh_token) = result.refresh_token {
                let refresh_cookie = format!(
                    "{}={}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age={}",
                    REFRESH_TOKEN_COOKIE_NAME,
                    refresh_token,
                    86400 * 7 // 7 days in seconds
                );
                headers.append(header::SET_COOKIE, refresh_cookie.parse().unwrap());
            }

            // Redirect to post-login URL
            headers.insert(header::LOCATION, result.redirect_uri.parse().unwrap());

            (StatusCode::FOUND, headers).into_response()
        }
        Err(e) => {
            let status = match &e {
                CommonError::NotFound { .. } => StatusCode::NOT_FOUND,
                CommonError::InvalidRequest { .. } => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(e)).into_response()
        }
    }
}
