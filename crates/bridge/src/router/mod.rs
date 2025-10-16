use axum::extract::{Json, Path, Query, State};
use std::sync::Arc;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    repository::Repository,
};
use shared::{
    adapters::openapi::JsonResponse,
    error::CommonError,
    primitives::{PaginationRequest, WrappedUuidV4},
};

pub const PATH_PREFIX: &str = "/api";
pub const API_VERSION_1: &str = "v1";
pub const SERVICE_ROUTE_KEY: &str = "bridge";

pub fn create_router() -> OpenApiRouter<Arc<BridgeService>> {
    OpenApiRouter::new()
        // .routes(routes!(route_list_tasks))
}

// #[utoipa::path(
//     get,
//     path = format!("{}/{}/{}/provider", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
//     params(
//         PaginationRequest
//     ),
//     responses(
//         (status = 200, description = "List available providers", body = ListAvailableProvidersResponse),
//         (status = 400, description = "Bad Request", body = CommonError),
//         (status = 401, description = "Unauthorized", body = CommonError),
//         (status = 403, description = "Forbidden", body = CommonError),
//         (status = 500, description = "Internal Server Error", body = CommonError),
//         (status = 502, description = "Bad Gateway", body = CommonError),
//     ),
//     operation_id = "list-tasks",
// )]
// async fn route_list_tasks(
//     State(ctx): State<Arc<BridgeService>>,
//     Query(pagination): Query<PaginationRequest>,
// ) -> JsonResponse<ListAvailableProvidersResponse, CommonError> {
//     let res = list_available_providers(pagination).await;
//     JsonResponse::from(res)
// }

// #[utoipa::path(
//     get,
//     path = format!("{}/{}/{}/context", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
//     params(
//         PaginationRequest
//     ),
//     responses(
//         (status = 200, description = "List contexts", body = ListUniqueContextsResponse),
//         (status = 400, description = "Bad Request", body = CommonError),
//         (status = 401, description = "Unauthorized", body = CommonError),
//         (status = 403, description = "Forbidden", body = CommonError),
//         (status = 500, description = "Internal Server Error", body = CommonError),
//         (status = 502, description = "Bad Gateway", body = CommonError),
//     ),
//     operation_id = "list-contexts",
// )]
// async fn route_list_contexts(
//     State(ctx): State<Arc<TaskService>>,
//     Query(pagination): Query<PaginationRequest>,
// ) -> JsonResponse<ListUniqueContextsResponse, CommonError> {
//     let res = list_unique_contexts(&ctx.repository, pagination).await;
//     JsonResponse::from(res)
// }


pub struct BridgeService {
    repository: Repository,
}

impl BridgeService {
    pub fn new(repository: Repository) -> Self {
        Self {
            repository,
        }
    }
}
