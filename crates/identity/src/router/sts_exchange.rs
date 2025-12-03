use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header::SET_COOKIE},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use shared::{adapters::openapi::API_VERSION_TAG, error::CommonError};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::logic::sts_exchange::{
    ExchangeStsTokenParams, RefreshTokenParams, exchange_sts_token, refresh_access_token,
};
use crate::service::IdentityService;

use super::{
    ACCESS_TOKEN_COOKIE_NAME, API_VERSION_1, PATH_PREFIX, REFRESH_TOKEN_COOKIE_NAME,
    SERVICE_ROUTE_KEY,
};

/// Access token expiration in seconds (1 hour)
const ACCESS_TOKEN_MAX_AGE_SECONDS: i64 = 3600;
/// Refresh token expiration in seconds (7 days)
const REFRESH_TOKEN_MAX_AGE_SECONDS: i64 = 86400 * 7;

pub fn create_sts_routes() -> OpenApiRouter<IdentityService> {
    OpenApiRouter::new()
        .routes(routes!(route_exchange_sts_token))
        .routes(routes!(route_refresh_token))
}

/// Response body for token exchange and refresh endpoints
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TokenResponse {
    /// The access token
    pub access_token: String,
    /// The refresh token (only present on exchange, not refresh)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    /// Token expiration time in seconds
    pub expires_in: i64,
    /// Token type (always "Bearer")
    pub token_type: String,
}

/// Request body for refresh token endpoint
#[derive(Debug, Deserialize, ToSchema)]
pub struct RefreshTokenRequest {
    /// The refresh token. If not provided, will be read from cookie.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
}

/// Build Set-Cookie header value for a token
fn build_cookie_header(name: &str, value: &str, max_age_seconds: i64) -> String {
    format!(
        "{}={}; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age={}",
        name, value, max_age_seconds
    )
}

/// Build response with token cookies and JSON body
fn build_token_response(
    access_token: String,
    refresh_token: Option<String>,
    expires_in: i64,
) -> Response {
    let body = TokenResponse {
        access_token: access_token.clone(),
        refresh_token: refresh_token.clone(),
        expires_in,
        token_type: "Bearer".to_string(),
    };

    let mut response = Json(body).into_response();

    // Set access token cookie
    response.headers_mut().append(
        SET_COOKIE,
        build_cookie_header(ACCESS_TOKEN_COOKIE_NAME, &access_token, ACCESS_TOKEN_MAX_AGE_SECONDS)
            .parse()
            .unwrap(),
    );

    // Set refresh token cookie if present
    if let Some(ref rt) = refresh_token {
        response.headers_mut().append(
            SET_COOKIE,
            build_cookie_header(REFRESH_TOKEN_COOKIE_NAME, rt, REFRESH_TOKEN_MAX_AGE_SECONDS)
                .parse()
                .unwrap(),
        );
    }

    response
}

/// Build error response from CommonError
fn build_error_response(error: CommonError) -> Response {
    let status = match &error {
        CommonError::NotFound { .. } => StatusCode::NOT_FOUND,
        CommonError::Authentication { .. } => StatusCode::UNAUTHORIZED,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };

    let body = serde_json::json!({
        "error": error.to_string(),
    });

    (status, Json(body)).into_response()
}

/// Extract refresh token from cookie header
fn extract_refresh_token_from_cookies(headers: &HeaderMap) -> Option<String> {
    let cookie_header = headers.get("cookie")?.to_str().ok()?;

    for cookie in cookie_header.split(';') {
        let cookie = cookie.trim();
        if let Some((name, value)) = cookie.split_once('=') {
            if name.trim() == REFRESH_TOKEN_COOKIE_NAME {
                return Some(value.trim().to_string());
            }
        }
    }

    None
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/sts/{{sts_config_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("sts_config_id" = String, Path, description = "STS configuration ID")
    ),
    responses(
        (status = 200, description = "Token exchange successful", body = TokenResponse),
        (status = 401, description = "Authentication failed", body = CommonError),
        (status = 404, description = "STS config not found", body = CommonError),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
)]
async fn route_exchange_sts_token(
    State(service): State<IdentityService>,
    Path(sts_config_id): Path<String>,
    headers: HeaderMap,
) -> Response {
    let params = ExchangeStsTokenParams {
        headers,
        sts_token_config_id: sts_config_id,
    };

    let auth_config = service.auth_middleware_config.load();

    let result = exchange_sts_token(
        service.repository.as_ref(),
        &service.crypto_cache,
        &service.jwks_cache,
        &service.external_jwks_cache,
        &auth_config.sts_token_config,
        params,
    )
    .await;

    match result {
        Ok(token_result) => build_token_response(
            token_result.access_token,
            token_result.refresh_token,
            token_result.expires_in,
        ),
        Err(error) => build_error_response(error),
    }
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/sts/refresh", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body(content = Option<RefreshTokenRequest>, description = "Optional refresh token in body. If not provided, will be read from cookie."),
    responses(
        (status = 200, description = "Token refresh successful", body = TokenResponse),
        (status = 401, description = "Authentication failed", body = CommonError),
        (status = 404, description = "User not found", body = CommonError),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
)]
async fn route_refresh_token(
    State(service): State<IdentityService>,
    headers: HeaderMap,
    body: Option<Json<RefreshTokenRequest>>,
) -> Response {
    // Get refresh token from body or cookie
    let refresh_token = body
        .and_then(|b| b.refresh_token.clone())
        .or_else(|| extract_refresh_token_from_cookies(&headers));

    let Some(refresh_token) = refresh_token else {
        return build_error_response(CommonError::Authentication {
            msg: "No refresh token provided in body or cookie".to_string(),
            source: None,
        });
    };

    let params = RefreshTokenParams { refresh_token };

    let result = refresh_access_token(
        service.repository.as_ref(),
        &service.crypto_cache,
        &service.jwks_cache,
        params,
    )
    .await;

    match result {
        Ok(token_result) => {
            // Only set access token cookie on refresh, don't update refresh token
            build_token_response(token_result.access_token, None, token_result.expires_in)
        }
        Err(error) => build_error_response(error),
    }
}
