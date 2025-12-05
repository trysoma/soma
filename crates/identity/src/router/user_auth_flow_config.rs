use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use shared::{
    adapters::openapi::{API_VERSION_TAG, JsonResponse},
    error::CommonError,
};
use utoipa::IntoParams;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::logic::user_auth_flow::{
    CreateUserAuthFlowConfigParams, CreateUserAuthFlowConfigResponse,
    DeleteUserAuthFlowConfigParams, DeleteUserAuthFlowConfigResponse, GetUserAuthFlowConfigParams,
    GetUserAuthFlowConfigResponse, ImportUserAuthFlowConfigParams,
    ImportUserAuthFlowConfigResponse, ListUserAuthFlowConfigParams, ListUserAuthFlowConfigResponse,
    create_user_auth_flow_config, delete_user_auth_flow_config, get_user_auth_flow_config,
    import_user_auth_flow_config, list_user_auth_flow_configs,
};
use crate::service::IdentityService;

use super::{API_VERSION_1, PATH_PREFIX, SERVICE_ROUTE_KEY};

pub fn create_user_auth_flow_config_routes() -> OpenApiRouter<IdentityService> {
    OpenApiRouter::new()
        .routes(routes!(route_create_user_auth_flow_config))
        .routes(routes!(route_get_user_auth_flow_config))
        .routes(routes!(route_delete_user_auth_flow_config))
        .routes(routes!(route_list_user_auth_flow_configs))
        .routes(routes!(route_import_user_auth_flow_config))
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListUserAuthFlowConfigsQuery {
    #[param(example = "10")]
    page_size: Option<u32>,
    #[param(example = "")]
    next_page_token: Option<String>,
    /// Filter by configuration type (oidc_authorization_code_flow, oauth_authorization_code_flow, etc.)
    #[param(example = "oidc_authorization_code_flow")]
    #[serde(rename = "type")]
    config_type: Option<String>,
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/user-auth-flow-config", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = CreateUserAuthFlowConfigParams,
    responses(
        (status = 201, description = "User auth flow configuration created successfully", body = CreateUserAuthFlowConfigResponse),
        (status = 400, description = "Invalid request", body = CommonError),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
    summary = "Create user auth flow configuration",
    description = "Create a new user auth flow configuration for OAuth/OIDC authorization flows. The configuration will be encrypted before storage.",
)]
async fn route_create_user_auth_flow_config(
    State(service): State<IdentityService>,
    Json(params): Json<CreateUserAuthFlowConfigParams>,
) -> JsonResponse<CreateUserAuthFlowConfigResponse, CommonError> {
    let result = create_user_auth_flow_config(
        service.repository.as_ref(),
        &service.crypto_cache,
        &service.on_config_change_tx,
        params,
        true, // publish_on_change_evt
    )
    .await;
    JsonResponse::from(result)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/user-auth-flow-config/{{id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("id" = String, Path, description = "ID of the user auth flow configuration to retrieve")
    ),
    responses(
        (status = 200, description = "User auth flow configuration found", body = GetUserAuthFlowConfigResponse),
        (status = 404, description = "User auth flow configuration not found", body = CommonError),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
    summary = "Get user auth flow configuration",
    description = "Get a user auth flow configuration by ID. Returns the encrypted configuration.",
)]
async fn route_get_user_auth_flow_config(
    State(service): State<IdentityService>,
    Path(id): Path<String>,
) -> JsonResponse<GetUserAuthFlowConfigResponse, CommonError> {
    let params = GetUserAuthFlowConfigParams { id };
    let result = get_user_auth_flow_config(service.repository.as_ref(), params).await;
    JsonResponse::from(result)
}

#[utoipa::path(
    delete,
    path = format!("{}/{}/{}/user-auth-flow-config/{{id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("id" = String, Path, description = "ID of the user auth flow configuration to delete")
    ),
    responses(
        (status = 200, description = "User auth flow configuration deleted successfully", body = DeleteUserAuthFlowConfigResponse),
        (status = 404, description = "User auth flow configuration not found", body = CommonError),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
    summary = "Delete user auth flow configuration",
    description = "Delete a user auth flow configuration by ID",
)]
async fn route_delete_user_auth_flow_config(
    State(service): State<IdentityService>,
    Path(id): Path<String>,
) -> JsonResponse<DeleteUserAuthFlowConfigResponse, CommonError> {
    let params = DeleteUserAuthFlowConfigParams { id };
    let result = delete_user_auth_flow_config(
        service.repository.as_ref(),
        &service.on_config_change_tx,
        params,
        true, // publish_on_change_evt
    )
    .await;
    JsonResponse::from(result)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/user-auth-flow-config", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ListUserAuthFlowConfigsQuery
    ),
    responses(
        (status = 200, description = "List of user auth flow configurations", body = ListUserAuthFlowConfigResponse),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
    summary = "List user auth flow configurations",
    description = "List all user auth flow configurations with optional filtering by type",
)]
async fn route_list_user_auth_flow_configs(
    State(service): State<IdentityService>,
    Query(query): Query<ListUserAuthFlowConfigsQuery>,
) -> JsonResponse<ListUserAuthFlowConfigResponse, CommonError> {
    use shared::primitives::PaginationRequest;
    let params = ListUserAuthFlowConfigParams {
        pagination: PaginationRequest {
            page_size: query.page_size.unwrap_or(10) as i64,
            next_page_token: query.next_page_token,
        },
        config_type: query.config_type,
    };
    let result = list_user_auth_flow_configs(service.repository.as_ref(), params).await;
    JsonResponse::from(result)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/user-auth-flow-config/import", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = ImportUserAuthFlowConfigParams,
    responses(
        (status = 201, description = "User auth flow configuration imported successfully", body = ImportUserAuthFlowConfigResponse),
        (status = 400, description = "Invalid request", body = CommonError),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
    summary = "Import user auth flow configuration",
    description = "Import an already encrypted user auth flow configuration (idempotent, used for syncing from soma.yaml)",
)]
async fn route_import_user_auth_flow_config(
    State(service): State<IdentityService>,
    Json(params): Json<ImportUserAuthFlowConfigParams>,
) -> JsonResponse<ImportUserAuthFlowConfigResponse, CommonError> {
    tracing::info!("Importing user auth flow config: {:?}", params.config);
    let result = import_user_auth_flow_config(service.repository.as_ref(), params).await;
    JsonResponse::from(result)
}
