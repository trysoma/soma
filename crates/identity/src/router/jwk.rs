use axum::extract::{Path, Query, State};
use shared::primitives::PaginationRequest;
use shared::{
    adapters::openapi::{API_VERSION_TAG, JsonResponse},
    error::CommonError,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::logic::jwk::{
    GetJwksResponse, InvalidateJwkParams, InvalidateJwkResponse, ListJwksResponse, get_jwks,
    invalidate_jwk, list_jwks,
};
use crate::service::IdentityService;

use super::{API_VERSION_1, PATH_PREFIX, SERVICE_ROUTE_KEY};

pub fn create_jwk_routes() -> OpenApiRouter<IdentityService> {
    OpenApiRouter::new()
        .routes(routes!(route_invalidate_jwk))
        .routes(routes!(route_list_jwks))
        .routes(routes!(route_get_jwks))
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/jwk/{{kid}}/invalidate", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("kid" = String, Path, description = "Key ID")
    ),
    responses(
        (status = 200, description = "JWK invalidated successfully"),
        (status = 404, description = "JWK not found", body = CommonError),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
)]
async fn route_invalidate_jwk(
    State(service): State<IdentityService>,
    Path(kid): Path<String>,
) -> JsonResponse<InvalidateJwkResponse, CommonError> {
    let params = InvalidateJwkParams { kid };
    let result = invalidate_jwk(
        service.repository.as_ref(),
        &service.internal_jwks_cache,
        params,
    )
    .await;
    JsonResponse::from(result)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/jwk", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        PaginationRequest
    ),
    responses(
        (status = 200, description = "List of JWKs", body = ListJwksResponse),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
)]
async fn route_list_jwks(
    State(service): State<IdentityService>,
    Query(query): Query<PaginationRequest>,
) -> JsonResponse<ListJwksResponse, CommonError> {
    let result = list_jwks(service.repository.as_ref(), &query).await;
    JsonResponse::from(result)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/.well-known/jwks.json", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    responses(
        (status = 200, description = "JWKS (JSON Web Key Set)", body = GetJwksResponse),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
)]
async fn route_get_jwks(
    State(service): State<IdentityService>,
) -> JsonResponse<GetJwksResponse, CommonError> {
    let result = get_jwks(service.repository.as_ref(), &service.internal_jwks_cache).await;
    JsonResponse::from(result)
}
