use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::CookieJar;
use shared::{adapters::openapi::API_VERSION_TAG, error::CommonError};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::logic::internal_token_issuance::NormalizedTokenIssuanceResult;
use crate::logic::sts::exchange::{ExchangeStsTokenParams, exchange_sts_token};
use crate::service::IdentityService;

use super::{API_VERSION_1, PATH_PREFIX, SERVICE_ROUTE_KEY, add_token_cookies_with_options};

pub fn create_sts_routes() -> OpenApiRouter<IdentityService> {
    OpenApiRouter::new().routes(routes!(route_exchange_sts_token))
}

/// Build response with token cookies and JSON body for full token issuance
fn build_token_response(jar: CookieJar, tokens: &NormalizedTokenIssuanceResult) -> Response {
    let jar = add_token_cookies_with_options(jar, tokens, true);

    (jar, Json(tokens)).into_response()
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/sts/{{sts_config_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("sts_config_id" = String, Path, description = "STS configuration ID")
    ),
    responses(
        (status = 200, description = "Token exchange successful", body = NormalizedTokenIssuanceResult),
        (status = 401, description = "Authentication failed", body = CommonError),
        (status = 404, description = "STS config not found", body = CommonError),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
    summary = "Exchange STS token",
    description = "Exchange an external token for internal access and refresh tokens using an STS configuration",
)]
async fn route_exchange_sts_token(
    State(service): State<IdentityService>,
    Path(sts_config_id): Path<String>,
    headers: HeaderMap,
    jar: CookieJar,
) -> impl IntoResponse {
    let params = ExchangeStsTokenParams {
        headers,
        sts_token_config_id: sts_config_id,
    };

    let result = exchange_sts_token(
        service.repository.as_ref(),
        &service.crypto_cache,
        &service.external_jwks_cache,
        params,
    )
    .await;

    match result {
        Ok(token_result) => build_token_response(jar, &token_result),
        Err(error) => error.into_response(),
    }
}
