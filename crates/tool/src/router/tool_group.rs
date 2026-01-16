//! Tool group and tool management routes

use http::HeaderMap;
use tracing::trace;

use super::{API_VERSION_1, McpService, PATH_PREFIX, SERVICE_ROUTE_KEY};
use crate::logic::{
    BrokerAction, BrokerInput, CreateToolGroupParamsInner, CreateToolGroupInstanceResponse,
    CreateResourceServerCredentialParamsInner, CreateResourceServerCredentialResponse,
    CreateUserCredentialParamsInner, CreateUserCredentialResponse, DisableToolParamsInner,
    DisableToolResponse, EnableToolParamsInner, EnableToolResponse,
    EncryptCredentialConfigurationParamsInner, EncryptedCredentialConfigurationResponse,
    GetToolGroupInstanceResponse, InvokeToolParamsInner, InvokeToolResponse,
    ListToolsParams, ListToolInstancesResponse,
    ListToolGroupInstancesGroupedByFunctionParams, ListToolGroupInstancesGroupedByFunctionResponse,
    ListToolGroupsParams, ListToolGroupInstancesResponse,
    ResumeUserCredentialBrokeringParams, StartUserCredentialBrokeringParamsInner,
    UpdateToolGroupParamsInner, UpdateToolGroupInstanceResponse,
    UserCredentialBrokeringResponse, UserCredentialSerialized, WithCredentialDeploymentTypeId,
    WithToolDeploymentTypeId, WithToolInstanceId, WithToolGroupDeploymentTypeId,
    WithToolGroupInstanceId, create_tool_group_instance, create_resource_server_credential,
    create_user_credential, delete_tool_group_instance, disable_tool, enable_tool,
    encrypt_resource_server_configuration, encrypt_user_credential_configuration,
    get_tool_instances_openapi_spec, get_tool_group_instance, invoke_tool,
    list_tool_instances, list_tool_group_instances,
    list_tool_group_instances_grouped_by_function, resume_user_credential_brokering,
    start_user_credential_brokering, update_tool_group_instance,
    ToolGroupDeploymentSerialized,
};
use crate::repository::ProviderRepositoryLike;
use axum::extract::{Json, Path, Query, State};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use shared::adapters::openapi::{API_VERSION_TAG, JsonResponse};
use shared::error::CommonError;
use shared::primitives::{PaginatedResponse, PaginationRequest};
use utoipa::openapi::OpenApi;
use utoipa::{IntoParams, ToSchema};

// ============================================================================
// Tool group endpoints
// ============================================================================

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/available-tool-groups", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        PaginationRequest
    ),
    responses(
        (status = 200, description = "List available tool groups"),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "List tool groups",
    description = "List all available tool group types that can be instantiated",
    operation_id = "list-available-tool-groups",
    security(
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
pub async fn route_list_available_tool_groups(
    State(_ctx): State<McpService>,
    _headers: HeaderMap,
    Query(pagination): Query<PaginationRequest>,
) -> JsonResponse<PaginatedResponse<ToolGroupDeploymentSerialized>, CommonError> {
    trace!(
        page_size = pagination.page_size,
        "Listing available tool groups"
    );

    // TODO: Fetch tool groups from repository instead of hardcoded list
    // Tool groups should be registered via API and stored in the database
    let tool_groups: Vec<ToolGroupDeploymentSerialized> = vec![];

    let response = PaginatedResponse::from_items_with_extra(
        tool_groups,
        &pagination,
        |p| vec![p.type_id.to_string()],
    );

    trace!("Listing available tool groups completed");
    JsonResponse::from(Ok(response))
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/available-tool-groups/{{tool_group_deployment_type_id}}/available-credentials/{{credential_deployment_type_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = CreateToolGroupParamsInner,
    responses(
        (status = 200, description = "Create tool group instance", body = CreateToolGroupInstanceResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Create tool group",
    description = "Create a new tool group instance with the specified configuration",
    operation_id = "create-tool-group-instance",
    security(
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
pub async fn route_create_tool_group(
    State(ctx): State<McpService>,
    headers: HeaderMap,
    Path((tool_group_deployment_type_id, credential_deployment_type_id)): Path<(String, String)>,
    Json(params): Json<CreateToolGroupParamsInner>,
) -> JsonResponse<CreateToolGroupInstanceResponse, CommonError> {
    trace!(
        tool_group_type = %tool_group_deployment_type_id,
        credential_type = %credential_deployment_type_id,
        display_name = %params.display_name,
        "Creating tool group instance"
    );
    let res = create_tool_group(
        ctx.auth_client().clone(),
        headers,
        ctx.on_config_change_tx(),
        ctx.repository(),
        WithToolGroupDeploymentTypeId {
            tool_group_deployment_type_id: tool_group_deployment_type_id.clone(),
            inner: WithCredentialDeploymentTypeId {
                credential_deployment_type_id: credential_deployment_type_id.clone(),
                inner: params,
            },
        },
        true,
    )
    .await;
    trace!(
        success = res.is_ok(),
        "Creating tool group instance completed"
    );
    JsonResponse::from(res)
}

#[utoipa::path(
    patch,
    path = format!("{}/{}/{}/tool-group/{{tool_group_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = UpdateToolGroupParamsInner,
    params(
        ("tool_group_id" = String, Path, description = "Tool group ID"),
    ),
    responses(
        (status = 200, description = "Update tool group instance", body = UpdateToolGroupInstanceResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Update tool group",
    description = "Update an existing tool group instance configuration",
    operation_id = "update-tool-group-instance",
    security(
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
pub async fn route_update_tool_group(
    State(ctx): State<McpService>,
    headers: HeaderMap,
    Path(tool_group_id): Path<String>,
    Json(params): Json<UpdateToolGroupParamsInner>,
) -> JsonResponse<UpdateToolGroupInstanceResponse, CommonError> {
    trace!(
        tool_group_id = %tool_group_id,
        display_name = %params.display_name,
        "Updating tool group instance"
    );
    let res = update_tool_group(
        ctx.auth_client().clone(),
        headers,
        ctx.on_config_change_tx(),
        ctx.repository(),
        WithToolGroupInstanceId {
            tool_group_id: tool_group_id.clone(),
            inner: params,
        },
        true,
    )
    .await;
    trace!(
        success = res.is_ok(),
        "Updating tool group instance completed"
    );
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/tool-group/{{tool_group_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("tool_group_id" = String, Path, description = "Tool group ID"),
    ),
    responses(
        (status = 200, description = "Get tool group instance", body = GetToolGroupInstanceResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Get tool group",
    description = "Retrieve a tool group instance by its unique identifier",
    operation_id = "get-tool-group-instance",
    security(
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
pub async fn route_get_tool_group(
    State(ctx): State<McpService>,
    headers: HeaderMap,
    Path(tool_group_id): Path<String>,
) -> JsonResponse<GetToolGroupInstanceResponse, CommonError> {
    trace!(tool_group_id = %tool_group_id, "Getting tool group instance");
    let res = get_tool_group(
        ctx.auth_client().clone(),
        headers,
        ctx.repository(),
        WithToolGroupInstanceId {
            tool_group_id: tool_group_id.clone(),
            inner: (),
        },
    )
    .await;
    trace!(success = res.is_ok(), "Getting tool group instance completed");
    JsonResponse::from(res)
}

// ============================================================================
// Configuration encryption endpoints
// ============================================================================

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/available-tool-groups/{{tool_group_deployment_type_id}}/available-credentials/{{credential_deployment_type_id}}/credential/resource-server/encrypt", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = EncryptCredentialConfigurationParamsInner,
    params(
        ("tool_group_deployment_type_id" = String, Path, description = "Tool group source type ID"),
        ("credential_deployment_type_id" = String, Path, description = "Credential source type ID"),
    ),
    responses(
        (status = 200, description = "Encrypt resource server configuration", body = EncryptedCredentialConfigurationResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Encrypt resource server config",
    description = "Encrypt a resource server credential configuration before storage",
    operation_id = "encrypt-resource-server-configuration",
    security(
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
pub async fn route_encrypt_resource_server_configuration(
    State(ctx): State<McpService>,
    headers: HeaderMap,
    Path((tool_group_deployment_type_id, credential_deployment_type_id)): Path<(String, String)>,
    Json(params): Json<EncryptCredentialConfigurationParamsInner>,
) -> JsonResponse<EncryptedCredentialConfigurationResponse, CommonError> {
    trace!(
        tool_group_type = %tool_group_deployment_type_id,
        credential_type = %credential_deployment_type_id,
        "Encrypting resource server configuration"
    );
    let res = encrypt_resource_server_configuration(
        ctx.auth_client().clone(),
        headers,
        ctx.encryption_service(),
        WithToolGroupDeploymentTypeId {
            tool_group_deployment_type_id: tool_group_deployment_type_id.clone(),
            inner: WithCredentialDeploymentTypeId {
                credential_deployment_type_id: credential_deployment_type_id.clone(),
                inner: params,
            },
        },
    )
    .await;
    trace!(
        success = res.is_ok(),
        "Encrypting resource server configuration completed"
    );
    JsonResponse::from(res)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/available-tool-groups/{{tool_group_deployment_type_id}}/available-credentials/{{credential_deployment_type_id}}/credential/user-credential/encrypt", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = EncryptCredentialConfigurationParamsInner,
    params(
        ("tool_group_deployment_type_id" = String, Path, description = "Tool group source type ID"),
        ("credential_deployment_type_id" = String, Path, description = "Credential source type ID"),
    ),
    responses(
        (status = 200, description = "Encrypt user credential configuration", body = EncryptedCredentialConfigurationResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Encrypt user credential config",
    description = "Encrypt a user credential configuration before storage",
    operation_id = "encrypt-user-credential-configuration",
    security(
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
pub async fn route_encrypt_user_credential_configuration(
    State(ctx): State<McpService>,
    headers: HeaderMap,
    Path((tool_group_deployment_type_id, credential_deployment_type_id)): Path<(String, String)>,
    Json(params): Json<EncryptCredentialConfigurationParamsInner>,
) -> JsonResponse<EncryptedCredentialConfigurationResponse, CommonError> {
    trace!(
        tool_group_type = %tool_group_deployment_type_id,
        credential_type = %credential_deployment_type_id,
        "Encrypting user credential configuration"
    );
    let res = encrypt_user_credential_configuration(
        ctx.auth_client().clone(),
        headers,
        ctx.encryption_service(),
        WithToolGroupDeploymentTypeId {
            tool_group_deployment_type_id: tool_group_deployment_type_id.clone(),
            inner: WithCredentialDeploymentTypeId {
                credential_deployment_type_id: credential_deployment_type_id.clone(),
                inner: params,
            },
        },
    )
    .await;
    trace!(
        success = res.is_ok(),
        "Encrypting user credential configuration completed"
    );
    JsonResponse::from(res)
}

// ============================================================================
// Resource server credential endpoints
// ============================================================================

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/available-tool-groups/{{tool_group_deployment_type_id}}/available-credentials/{{credential_deployment_type_id}}/credential/resource-server", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("tool_group_deployment_type_id" = String, Path, description = "Tool group source type ID"),
        ("credential_deployment_type_id" = String, Path, description = "Credential source type ID"),
    ),
    request_body = CreateResourceServerCredentialParamsInner,
    responses(
        (status = 200, description = "Create resource server credential", body = CreateResourceServerCredentialResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Create resource server credential",
    description = "Create a new resource server credential",
    operation_id = "create-resource-server-credential",
    security(
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
pub async fn route_create_resource_server_credential(
    State(ctx): State<McpService>,
    headers: HeaderMap,
    Path((tool_group_deployment_type_id, credential_deployment_type_id)): Path<(String, String)>,
    Json(params): Json<CreateResourceServerCredentialParamsInner>,
) -> JsonResponse<CreateResourceServerCredentialResponse, CommonError> {
    trace!(
        tool_group_type = %tool_group_deployment_type_id,
        credential_type = %credential_deployment_type_id,
        "Creating resource server credential"
    );
    let res = create_resource_server_credential(
        ctx.auth_client().clone(),
        headers,
        ctx.repository(),
        WithToolGroupDeploymentTypeId {
            tool_group_deployment_type_id: tool_group_deployment_type_id.clone(),
            inner: WithCredentialDeploymentTypeId {
                credential_deployment_type_id: credential_deployment_type_id.clone(),
                inner: params,
            },
        },
    )
    .await;
    trace!(
        success = res.is_ok(),
        "Creating resource server credential completed"
    );
    JsonResponse::from(res)
}

// ============================================================================
// User credential endpoints
// ============================================================================

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/available-tool-groups/{{tool_group_deployment_type_id}}/available-credentials/{{credential_deployment_type_id}}/credential/user-credential", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = CreateUserCredentialParamsInner,
    params(
        ("tool_group_deployment_type_id" = String, Path, description = "Tool group source type ID"),
        ("credential_deployment_type_id" = String, Path, description = "Credential source type ID"),
    ),
    responses(
        (status = 200, description = "Create user credential", body = CreateUserCredentialResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Create user credential",
    description = "Create a new user credential",
    operation_id = "create-user-credential",
    security(
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
pub async fn route_create_user_credential(
    State(ctx): State<McpService>,
    headers: HeaderMap,
    Path((tool_group_deployment_type_id, credential_deployment_type_id)): Path<(String, String)>,
    Json(params): Json<CreateUserCredentialParamsInner>,
) -> JsonResponse<CreateUserCredentialResponse, CommonError> {
    trace!(
        tool_group_type = %tool_group_deployment_type_id,
        credential_type = %credential_deployment_type_id,
        "Creating user credential"
    );
    let res = create_user_credential(
        ctx.auth_client().clone(),
        headers,
        ctx.repository(),
        WithToolGroupDeploymentTypeId {
            tool_group_deployment_type_id: tool_group_deployment_type_id.clone(),
            inner: WithCredentialDeploymentTypeId {
                credential_deployment_type_id: credential_deployment_type_id.clone(),
                inner: params,
            },
        },
    )
    .await;
    trace!(success = res.is_ok(), "Creating user credential completed");
    JsonResponse::from(res)
}

// ============================================================================
// User credential brokering endpoints
// ============================================================================

macro_rules! respond_err {
    ($expr:expr) => {{
        let data = $expr.into();
        let res: JsonResponse<(), CommonError> = JsonResponse::new_error(data);
        res.into_response()
    }};
}

fn handle_user_credential_brokering_response(
    response: Result<UserCredentialBrokeringResponse, CommonError>,
) -> impl IntoResponse {
    let response = match response {
        Ok(response) => response,
        Err(e) => {
            return respond_err!(e);
        }
    };

    match response {
        UserCredentialBrokeringResponse::BrokerState(broker_state) => match broker_state.action {
            BrokerAction::Redirect(redirect) => {
                axum::response::Redirect::to(redirect.url.as_str()).into_response()
            }
            BrokerAction::None => {
                let res: JsonResponse<(), CommonError> = JsonResponse::new_ok(());
                res.into_response()
            }
        },
        UserCredentialBrokeringResponse::UserCredential(user_cred) => {
            let res: JsonResponse<UserCredentialSerialized, CommonError> =
                JsonResponse::new_ok(user_cred);
            res.into_response()
        }
        UserCredentialBrokeringResponse::Redirect(url) => {
            axum::response::Redirect::to(url.as_str()).into_response()
        }
    }
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/available-tool-groups/{{tool_group_deployment_type_id}}/available-credentials/{{credential_deployment_type_id}}/credential/user-credential/broker", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = StartUserCredentialBrokeringParamsInner,
    params(
        ("tool_group_deployment_type_id" = String, Path, description = "Tool group source type ID"),
        ("credential_deployment_type_id" = String, Path, description = "Credential source type ID"),
    ),
    responses(
        (status = 200, description = "Start user credential brokering", body = UserCredentialBrokeringResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Start credential brokering",
    description = "Start the OAuth flow for user credential brokering",
    operation_id = "start-user-credential-brokering",
    security(
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
pub async fn route_start_user_credential_brokering(
    State(ctx): State<McpService>,
    headers: HeaderMap,
    Path((tool_group_deployment_type_id, credential_deployment_type_id)): Path<(String, String)>,
    Json(params): Json<StartUserCredentialBrokeringParamsInner>,
) -> JsonResponse<UserCredentialBrokeringResponse, CommonError> {
    trace!(
        tool_group_type = %tool_group_deployment_type_id,
        credential_type = %credential_deployment_type_id,
        "Starting user credential brokering"
    );
    let res = start_user_credential_brokering(
        ctx.auth_client().clone(),
        headers,
        ctx.on_config_change_tx(),
        ctx.repository(),
        WithToolGroupDeploymentTypeId {
            tool_group_deployment_type_id: tool_group_deployment_type_id.clone(),
            inner: WithCredentialDeploymentTypeId {
                credential_deployment_type_id: credential_deployment_type_id.clone(),
                inner: params,
            },
        },
    )
    .await;
    trace!(
        success = res.is_ok(),
        "Starting user credential brokering completed"
    );

    JsonResponse::from(res)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GenericOAuthCallbackParams {
    pub state: Option<String>,
    pub code: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/generic-oauth-callback", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("state" = Option<String>, Query, description = "OAuth state parameter"),
        ("code" = Option<String>, Query, description = "OAuth authorization code"),
        ("error" = Option<String>, Query, description = "OAuth error code"),
        ("error_description" = Option<String>, Query, description = "OAuth error description"),
    ),
    responses(
        (status = 200, description = "Generic OAuth callback", body = UserCredentialBrokeringResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "OAuth callback",
    description = "Handle OAuth callback to complete user credential brokering flow",
    operation_id = "resume-user-credential-brokering",
    security(
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
pub async fn generic_oauth_callback(
    State(ctx): State<McpService>,
    Query(params): Query<GenericOAuthCallbackParams>,
) -> impl IntoResponse {
    trace!(
        has_state = params.state.is_some(),
        has_code = params.code.is_some(),
        has_error = params.error.is_some(),
        "Handling OAuth callback"
    );
    // Check for OAuth errors first
    if let Some(error) = params.error {
        let error_desc = params
            .error_description
            .unwrap_or_else(|| "No description provided".to_string());
        return respond_err!(CommonError::Unknown(anyhow::anyhow!(
            "OAuth error: {error} - {error_desc}"
        )));
    }

    // Extract state parameter
    let state = match params.state {
        Some(s) => s,
        None => {
            return respond_err!(CommonError::Unknown(anyhow::anyhow!(
                "Missing 'state' parameter in OAuth callback"
            )));
        }
    };

    // Extract authorization code
    let code = match params.code {
        Some(c) => c,
        None => {
            return respond_err!(CommonError::Unknown(anyhow::anyhow!(
                "Missing 'code' parameter in OAuth callback"
            )));
        }
    };

    // Create broker input
    let broker_input = BrokerInput::Oauth2AuthorizationCodeFlow { code };

    // Validate that the broker state exists (fetched for validation only)
    let _broker_state = match ctx.repository().get_broker_state_by_id(&state).await {
        Ok(Some(state)) => state,
        Ok(None) => {
            return respond_err!(CommonError::Unknown(anyhow::anyhow!(
                "Broker state not found: {state}"
            )));
        }
        Err(e) => return respond_err!(e),
    };

    // Resume the user credential brokering flow
    let res = resume_user_credential_brokering(
        ctx.on_config_change_tx(),
        ctx.repository(),
        ctx.encryption_service(),
        ResumeUserCredentialBrokeringParams {
            broker_state_id: state,
            input: broker_input,
        },
    )
    .await;
    trace!(success = res.is_ok(), "OAuth callback handling completed");

    handle_user_credential_brokering_response(res).into_response()
}

// ============================================================================
// Tool endpoints
// ============================================================================

#[utoipa::path(
    delete,
    path = format!("{}/{}/{}/tool-group/{{tool_group_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("tool_group_id" = String, Path, description = "Tool group ID"),
    ),
    responses(
        (status = 200, description = "Delete tool group instance", body = ()),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Delete tool group",
    description = "Delete a tool group instance by its unique identifier",
    operation_id = "delete-tool-group-instance",
    security(
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
pub async fn route_delete_tool_group(
    State(ctx): State<McpService>,
    headers: HeaderMap,
    Path(tool_group_id): Path<String>,
) -> JsonResponse<(), CommonError> {
    trace!(tool_group_id = %tool_group_id, "Deleting tool group instance");
    let res = delete_tool_group(
        ctx.auth_client().clone(),
        headers,
        ctx.on_config_change_tx(),
        ctx.repository(),
        WithToolGroupInstanceId {
            tool_group_id: tool_group_id.clone(),
            inner: (),
        },
        true,
    )
    .await;
    trace!(
        success = res.is_ok(),
        "Deleting tool group instance completed"
    );
    JsonResponse::from(res)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/tool-group/{{tool_group_id}}/tool/{{tool_deployment_type_id}}/enable", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = EnableToolParamsInner,
    params(
        ("tool_group_id" = String, Path, description = "Tool group ID"),
        ("tool_deployment_type_id" = String, Path, description = "Tool source type ID"),
    ),
    responses(
        (status = 200, description = "Enable tool", body = EnableToolResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Enable tool",
    description = "Enable a tool for a tool group instance",
    operation_id = "enable-tool",
    security(
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
pub async fn route_enable_tool(
    State(ctx): State<McpService>,
    headers: HeaderMap,
    Path((tool_group_id, tool_deployment_type_id)): Path<(String, String)>,
    Json(params): Json<EnableToolParamsInner>,
) -> JsonResponse<EnableToolResponse, CommonError> {
    trace!(
        tool_group_id = %tool_group_id,
        tool_type = %tool_deployment_type_id,
        "Enabling tool"
    );
    let res = enable_tool(
        ctx.auth_client().clone(),
        headers,
        ctx.on_config_change_tx(),
        ctx.repository(),
        WithToolGroupInstanceId {
            tool_group_id: tool_group_id.clone(),
            inner: WithToolDeploymentTypeId {
                tool_deployment_type_id: tool_deployment_type_id.clone(),
                inner: params,
            },
        },
        true,
    )
    .await;
    trace!(success = res.is_ok(), "Enabling tool completed");

    JsonResponse::from(res)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/tool-group/{{tool_group_id}}/tool/{{tool_deployment_type_id}}/disable", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("tool_group_id" = String, Path, description = "Tool group ID"),
        ("tool_deployment_type_id" = String, Path, description = "Tool source type ID"),
    ),
    responses(
        (status = 200, description = "Disable tool", body = DisableToolResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Disable tool",
    description = "Disable a tool for a tool group instance",
    operation_id = "disable-tool",
    security(
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
pub async fn route_disable_tool(
    State(ctx): State<McpService>,
    headers: HeaderMap,
    Path((tool_group_id, tool_deployment_type_id)): Path<(String, String)>,
) -> JsonResponse<DisableToolResponse, CommonError> {
    trace!(
        tool_group_id = %tool_group_id,
        tool_type = %tool_deployment_type_id,
        "Disabling tool"
    );
    let res = disable_tool(
        ctx.auth_client().clone(),
        headers,
        ctx.on_config_change_tx(),
        ctx.repository(),
        WithToolGroupInstanceId {
            tool_group_id: tool_group_id.clone(),
            inner: WithToolDeploymentTypeId {
                tool_deployment_type_id: tool_deployment_type_id.clone(),
                inner: DisableToolParamsInner {},
            },
        },
        true,
    )
    .await;
    trace!(success = res.is_ok(), "Disabling tool completed");

    JsonResponse::from(res)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/tool-group/{{tool_group_id}}/tool/{{tool_deployment_type_id}}/invoke", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = InvokeToolParamsInner,
    params(
        ("tool_group_id" = String, Path, description = "Tool group ID"),
        ("tool_deployment_type_id" = String, Path, description = "Tool source type ID"),
    ),
    responses(
        (status = 200, description = "Invoke tool", body = InvokeToolResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Invoke tool",
    description = "Invoke a tool on a tool group instance",
    operation_id = "invoke-tool",
    security(
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
pub async fn route_invoke_tool(
    State(ctx): State<McpService>,
    headers: HeaderMap,
    Path((tool_group_id, tool_deployment_type_id)): Path<(String, String)>,
    Json(params): Json<InvokeToolParamsInner>,
) -> JsonResponse<InvokeToolResponse, CommonError> {
    trace!(
        tool_group_id = %tool_group_id,
        tool_type = %tool_deployment_type_id,
        "Invoking tool"
    );
    let res = invoke_tool(
        ctx.auth_client().clone(),
        headers,
        ctx.repository(),
        ctx.encryption_service(),
        WithToolGroupInstanceId {
            tool_group_id: tool_group_id.clone(),
            inner: WithToolInstanceId {
                tool_deployment_type_id: tool_deployment_type_id.clone(),
                inner: params,
            },
        },
    )
    .await;
    trace!(success = res.is_ok(), "Invoking tool completed");
    JsonResponse::from(res)
}

#[derive(Serialize, Deserialize, Debug, ToSchema, IntoParams)]
#[into_params(style = Form, parameter_in = Query)]
pub struct ListToolGroupInstancesQuery {
    // TODO: utoipa doesnt support flattening yet https://github.com/juhaku/utoipa/pull/1426
    pub page_size: i64,
    pub next_page_token: Option<String>,
    pub status: Option<String>,
    pub tool_group_deployment_type_id: Option<String>,
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/tool-group", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ListToolGroupInstancesQuery
    ),
    responses(
        (status = 200, description = "List tool group instances", body = ListToolGroupInstancesResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "List tool group instances",
    description = "List all tool group instances with optional filtering by status and tool group type",
    operation_id = "list-tool-group-instances",
    security(
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
pub async fn route_list_tool_groups(
    State(ctx): State<McpService>,
    headers: HeaderMap,
    Query(query): Query<ListToolGroupInstancesQuery>,
) -> JsonResponse<ListToolGroupInstancesResponse, CommonError> {
    trace!(
        page_size = query.page_size,
        status = ?query.status,
        tool_group_type = ?query.tool_group_deployment_type_id,
        "Listing tool group instances"
    );
    let res = list_tool_groups(
        ctx.auth_client().clone(),
        headers,
        ctx.repository(),
        ListToolGroupsParams {
            pagination: PaginationRequest {
                page_size: query.page_size,
                next_page_token: query.next_page_token,
            },
            status: query.status,
            tool_group_deployment_type_id: query.tool_group_deployment_type_id,
        },
    )
    .await;
    trace!(
        success = res.is_ok(),
        "Listing tool group instances completed"
    );
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/tool-group/grouped-by-function", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ListToolGroupInstancesGroupedByFunctionParams
    ),
    responses(
        (status = 200, description = "List tool group instances grouped by function", body = ListToolGroupInstancesGroupedByFunctionResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "List tool groups by function",
    description = "List tool group instances grouped by their associated functions",
    operation_id = "list-tool-group-instances-grouped-by-function",
    security(
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
pub async fn route_list_tool_groups_grouped_by_function(
    State(ctx): State<McpService>,
    headers: HeaderMap,
    Query(query): Query<ListToolGroupInstancesGroupedByFunctionParams>,
) -> JsonResponse<ListToolGroupInstancesGroupedByFunctionResponse, CommonError> {
    trace!("Listing tool group instances grouped by function");
    let res = list_tool_groups_grouped_by_function(
        ctx.auth_client().clone(),
        headers,
        ctx.repository(),
        query,
    )
    .await;
    trace!(
        success = res.is_ok(),
        "Listing tool group instances grouped by function completed"
    );
    JsonResponse::from(res)
}

#[derive(Serialize, Deserialize, Debug, ToSchema, IntoParams)]
#[into_params(style = Form, parameter_in = Query)]
pub struct ListToolInstancesQuery {
    // TODO: utoipa doesnt support flattening yet https://github.com/juhaku/utoipa/pull/1426
    pub page_size: i64,
    pub next_page_token: Option<String>,
    pub tool_group_id: Option<String>,
}
#[utoipa::path(
    get,
    path = format!("{}/{}/{}/tools", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ListToolInstancesQuery
    ),
    responses(
        (status = 200, description = "List tool instances", body = ListToolInstancesResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "List tool instances",
    description = "List all tool instances with optional filtering by tool group instance",
    operation_id = "list-tools",
    security(
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
pub async fn route_list_tools(
    State(ctx): State<McpService>,
    headers: HeaderMap,
    Query(query): Query<ListToolInstancesQuery>,
) -> JsonResponse<ListToolInstancesResponse, CommonError> {
    trace!(
        page_size = query.page_size,
        tool_group_id = ?query.tool_group_id,
        "Listing tool instances"
    );
    let res = list_tools(
        ctx.auth_client().clone(),
        headers,
        ctx.repository(),
        ListToolsParams {
            pagination: PaginationRequest {
                page_size: query.page_size,
                next_page_token: query.next_page_token,
            },
            tool_group_id: query.tool_group_id,
        },
    )
    .await;
    trace!(
        success = res.is_ok(),
        "Listing tool instances completed"
    );
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/tools/openapi.json", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(),
    responses(
        (status = 200, description = "Get tool instances openapi spec", body = String),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Get tool OpenAPI spec",
    description = "Get the OpenAPI specification for all tool instances",
    operation_id = "get-tools-openapi-spec",
    security(
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
pub async fn route_get_tools_openapi_spec(
    State(ctx): State<McpService>,
    headers: HeaderMap,
) -> JsonResponse<OpenApi, CommonError> {
    trace!("Getting tool instances OpenAPI spec");
    let res =
        get_tools_openapi_spec(ctx.auth_client().clone(), headers, ctx.repository())
            .await;
    trace!(
        success = res.is_ok(),
        "Getting tool instances OpenAPI spec completed"
    );
    JsonResponse::from(res)
}
