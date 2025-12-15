//! Provider and function management routes

use tracing::trace;

use super::{API_VERSION_1, BridgeService, PATH_PREFIX, SERVICE_ROUTE_KEY};
use crate::logic::{
    BrokerAction, BrokerInput, CreateProviderInstanceParamsInner, CreateProviderInstanceResponse,
    CreateResourceServerCredentialParamsInner, CreateResourceServerCredentialResponse,
    CreateUserCredentialParamsInner, CreateUserCredentialResponse, DisableFunctionParamsInner,
    DisableFunctionResponse, EnableFunctionParamsInner, EnableFunctionResponse,
    EncryptCredentialConfigurationParamsInner, EncryptedCredentialConfigurationResponse,
    GetProviderInstanceResponse, InvokeFunctionParamsInner, InvokeFunctionResponse,
    ListAvailableProvidersResponse, ListFunctionInstancesParams, ListFunctionInstancesResponse,
    ListProviderInstancesGroupedByFunctionParams, ListProviderInstancesGroupedByFunctionResponse,
    ListProviderInstancesParams, ListProviderInstancesResponse,
    ResumeUserCredentialBrokeringParams, StartUserCredentialBrokeringParamsInner,
    UpdateProviderInstanceParamsInner, UpdateProviderInstanceResponse,
    UserCredentialBrokeringResponse, UserCredentialSerialized, WithCredentialControllerTypeId,
    WithFunctionControllerTypeId, WithFunctionInstanceId, WithProviderControllerTypeId,
    WithProviderInstanceId, create_provider_instance, create_resource_server_credential,
    create_user_credential, delete_provider_instance, disable_function, enable_function,
    encrypt_resource_server_configuration, encrypt_user_credential_configuration,
    get_function_instances_openapi_spec, get_provider_instance, invoke_function,
    list_available_providers, list_function_instances, list_provider_instances,
    list_provider_instances_grouped_by_function, resume_user_credential_brokering,
    start_user_credential_brokering, update_provider_instance,
};
use crate::repository::ProviderRepositoryLike;
use axum::extract::{Json, Path, Query, State};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use shared::adapters::openapi::{API_VERSION_TAG, JsonResponse};
use shared::error::CommonError;
use shared::primitives::PaginationRequest;
use utoipa::openapi::OpenApi;
use utoipa::{IntoParams, ToSchema};

// ============================================================================
// Provider endpoints
// ============================================================================

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/available-providers", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        PaginationRequest
    ),
    responses(
        (status = 200, description = "List available providers", body = ListAvailableProvidersResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "List providers",
    description = "List all available provider types that can be instantiated",
    operation_id = "list-available-providers",
)]
pub async fn route_list_available_providers(
    State(_ctx): State<BridgeService>,
    Query(pagination): Query<PaginationRequest>,
) -> JsonResponse<ListAvailableProvidersResponse, CommonError> {
    trace!(page_size = pagination.page_size, "Listing available providers");
    let res = list_available_providers(pagination).await;
    trace!(success = res.is_ok(), "Listing available providers completed");
    JsonResponse::from(res)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/available-providers/{{provider_controller_type_id}}/available-credentials/{{credential_controller_type_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = CreateProviderInstanceParamsInner,
    responses(
        (status = 200, description = "Create provider instance", body = CreateProviderInstanceResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Create provider",
    description = "Create a new provider instance with the specified configuration",
    operation_id = "create-provider-instance",
)]
pub async fn route_create_provider_instance(
    State(ctx): State<BridgeService>,
    Path((provider_controller_type_id, credential_controller_type_id)): Path<(String, String)>,
    Json(params): Json<CreateProviderInstanceParamsInner>,
) -> JsonResponse<CreateProviderInstanceResponse, CommonError> {
    trace!(
        provider_type = %provider_controller_type_id,
        credential_type = %credential_controller_type_id,
        display_name = %params.display_name,
        "Creating provider instance"
    );
    let res = create_provider_instance(
        ctx.on_config_change_tx(),
        ctx.repository(),
        WithProviderControllerTypeId {
            provider_controller_type_id: provider_controller_type_id.clone(),
            inner: WithCredentialControllerTypeId {
                credential_controller_type_id: credential_controller_type_id.clone(),
                inner: params,
            },
        },
        true,
    )
    .await;
    trace!(success = res.is_ok(), "Creating provider instance completed");
    JsonResponse::from(res)
}

#[utoipa::path(
    patch,
    path = format!("{}/{}/{}/provider/{{provider_instance_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = UpdateProviderInstanceParamsInner,
    params(
        ("provider_instance_id" = String, Path, description = "Provider instance ID"),
    ),
    responses(
        (status = 200, description = "Update provider instance", body = UpdateProviderInstanceResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Update provider",
    description = "Update an existing provider instance configuration",
    operation_id = "update-provider-instance",
)]
pub async fn route_update_provider_instance(
    State(ctx): State<BridgeService>,
    Path(provider_instance_id): Path<String>,
    Json(params): Json<UpdateProviderInstanceParamsInner>,
) -> JsonResponse<UpdateProviderInstanceResponse, CommonError> {
    trace!(
        provider_instance_id = %provider_instance_id,
        display_name = %params.display_name,
        "Updating provider instance"
    );
    let res = update_provider_instance(
        ctx.on_config_change_tx(),
        ctx.repository(),
        WithProviderInstanceId {
            provider_instance_id: provider_instance_id.clone(),
            inner: params,
        },
        true,
    )
    .await;
    trace!(success = res.is_ok(), "Updating provider instance completed");
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/provider/{{provider_instance_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("provider_instance_id" = String, Path, description = "Provider instance ID"),
    ),
    responses(
        (status = 200, description = "Get provider instance", body = GetProviderInstanceResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Get provider",
    description = "Retrieve a provider instance by its unique identifier",
    operation_id = "get-provider-instance",
)]
pub async fn route_get_provider_instance(
    State(ctx): State<BridgeService>,
    Path(provider_instance_id): Path<String>,
) -> JsonResponse<GetProviderInstanceResponse, CommonError> {
    trace!(provider_instance_id = %provider_instance_id, "Getting provider instance");
    let res = get_provider_instance(
        ctx.repository(),
        WithProviderInstanceId {
            provider_instance_id: provider_instance_id.clone(),
            inner: (),
        },
    )
    .await;
    trace!(success = res.is_ok(), "Getting provider instance completed");
    JsonResponse::from(res)
}

// ============================================================================
// Configuration encryption endpoints
// ============================================================================

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/available-providers/{{provider_controller_type_id}}/available-credentials/{{credential_controller_type_id}}/credential/resource-server/encrypt", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = EncryptCredentialConfigurationParamsInner,
    params(
        ("provider_controller_type_id" = String, Path, description = "Provider controller type ID"),
        ("credential_controller_type_id" = String, Path, description = "Credential controller type ID"),
    ),
    responses(
        (status = 200, description = "Encrypt resource server configuration", body = EncryptedCredentialConfigurationResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Encrypt resource server config",
    description = "Encrypt a resource server credential configuration before storage",
    operation_id = "encrypt-resource-server-configuration",
)]
pub async fn route_encrypt_resource_server_configuration(
    State(ctx): State<BridgeService>,
    Path((provider_controller_type_id, credential_controller_type_id)): Path<(String, String)>,
    Json(params): Json<EncryptCredentialConfigurationParamsInner>,
) -> JsonResponse<EncryptedCredentialConfigurationResponse, CommonError> {
    trace!(
        provider_type = %provider_controller_type_id,
        credential_type = %credential_controller_type_id,
        "Encrypting resource server configuration"
    );
    let res = encrypt_resource_server_configuration(
        ctx.encryption_service(),
        WithProviderControllerTypeId {
            provider_controller_type_id: provider_controller_type_id.clone(),
            inner: WithCredentialControllerTypeId {
                credential_controller_type_id: credential_controller_type_id.clone(),
                inner: params,
            },
        },
    )
    .await;
    trace!(success = res.is_ok(), "Encrypting resource server configuration completed");
    JsonResponse::from(res)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/available-providers/{{provider_controller_type_id}}/available-credentials/{{credential_controller_type_id}}/credential/user-credential/encrypt", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = EncryptCredentialConfigurationParamsInner,
    params(
        ("provider_controller_type_id" = String, Path, description = "Provider controller type ID"),
        ("credential_controller_type_id" = String, Path, description = "Credential controller type ID"),
    ),
    responses(
        (status = 200, description = "Encrypt user credential configuration", body = EncryptedCredentialConfigurationResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Encrypt user credential config",
    description = "Encrypt a user credential configuration before storage",
    operation_id = "encrypt-user-credential-configuration",
)]
pub async fn route_encrypt_user_credential_configuration(
    State(ctx): State<BridgeService>,
    Path((provider_controller_type_id, credential_controller_type_id)): Path<(String, String)>,
    Json(params): Json<EncryptCredentialConfigurationParamsInner>,
) -> JsonResponse<EncryptedCredentialConfigurationResponse, CommonError> {
    trace!(
        provider_type = %provider_controller_type_id,
        credential_type = %credential_controller_type_id,
        "Encrypting user credential configuration"
    );
    let res = encrypt_user_credential_configuration(
        ctx.encryption_service(),
        WithProviderControllerTypeId {
            provider_controller_type_id: provider_controller_type_id.clone(),
            inner: WithCredentialControllerTypeId {
                credential_controller_type_id: credential_controller_type_id.clone(),
                inner: params,
            },
        },
    )
    .await;
    trace!(success = res.is_ok(), "Encrypting user credential configuration completed");
    JsonResponse::from(res)
}

// ============================================================================
// Resource server credential endpoints
// ============================================================================

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/available-providers/{{provider_controller_type_id}}/available-credentials/{{credential_controller_type_id}}/credential/resource-server", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("provider_controller_type_id" = String, Path, description = "Provider controller type ID"),
        ("credential_controller_type_id" = String, Path, description = "Credential controller type ID"),
    ),
    request_body = CreateResourceServerCredentialParamsInner,
    responses(
        (status = 200, description = "Create resource server credential", body = CreateResourceServerCredentialResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Create resource server credential",
    description = "Create a new resource server credential",
    operation_id = "create-resource-server-credential",
)]
pub async fn route_create_resource_server_credential(
    State(ctx): State<BridgeService>,
    Path((provider_controller_type_id, credential_controller_type_id)): Path<(String, String)>,
    Json(params): Json<CreateResourceServerCredentialParamsInner>,
) -> JsonResponse<CreateResourceServerCredentialResponse, CommonError> {
    trace!(
        provider_type = %provider_controller_type_id,
        credential_type = %credential_controller_type_id,
        "Creating resource server credential"
    );
    let res = create_resource_server_credential(
        ctx.repository(),
        WithProviderControllerTypeId {
            provider_controller_type_id: provider_controller_type_id.clone(),
            inner: WithCredentialControllerTypeId {
                credential_controller_type_id: credential_controller_type_id.clone(),
                inner: params,
            },
        },
    )
    .await;
    trace!(success = res.is_ok(), "Creating resource server credential completed");
    JsonResponse::from(res)
}

// ============================================================================
// User credential endpoints
// ============================================================================

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/available-providers/{{provider_controller_type_id}}/available-credentials/{{credential_controller_type_id}}/credential/user-credential", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = CreateUserCredentialParamsInner,
    params(
        ("provider_controller_type_id" = String, Path, description = "Provider controller type ID"),
        ("credential_controller_type_id" = String, Path, description = "Credential controller type ID"),
    ),
    responses(
        (status = 200, description = "Create user credential", body = CreateUserCredentialResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Create user credential",
    description = "Create a new user credential",
    operation_id = "create-user-credential",
)]
pub async fn route_create_user_credential(
    State(ctx): State<BridgeService>,
    Path((provider_controller_type_id, credential_controller_type_id)): Path<(String, String)>,
    Json(params): Json<CreateUserCredentialParamsInner>,
) -> JsonResponse<CreateUserCredentialResponse, CommonError> {
    trace!(
        provider_type = %provider_controller_type_id,
        credential_type = %credential_controller_type_id,
        "Creating user credential"
    );
    let res = create_user_credential(
        ctx.repository(),
        WithProviderControllerTypeId {
            provider_controller_type_id: provider_controller_type_id.clone(),
            inner: WithCredentialControllerTypeId {
                credential_controller_type_id: credential_controller_type_id.clone(),
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
    path = format!("{}/{}/{}/available-providers/{{provider_controller_type_id}}/available-credentials/{{credential_controller_type_id}}/credential/user-credential/broker", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = StartUserCredentialBrokeringParamsInner,
    params(
        ("provider_controller_type_id" = String, Path, description = "Provider controller type ID"),
        ("credential_controller_type_id" = String, Path, description = "Credential controller type ID"),
    ),
    responses(
        (status = 200, description = "Start user credential brokering", body = UserCredentialBrokeringResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Start credential brokering",
    description = "Start the OAuth flow for user credential brokering",
    operation_id = "start-user-credential-brokering",
)]
pub async fn route_start_user_credential_brokering(
    State(ctx): State<BridgeService>,
    Path((provider_controller_type_id, credential_controller_type_id)): Path<(String, String)>,
    Json(params): Json<StartUserCredentialBrokeringParamsInner>,
) -> JsonResponse<UserCredentialBrokeringResponse, CommonError> {
    trace!(
        provider_type = %provider_controller_type_id,
        credential_type = %credential_controller_type_id,
        "Starting user credential brokering"
    );
    let res = start_user_credential_brokering(
        ctx.on_config_change_tx(),
        ctx.repository(),
        WithProviderControllerTypeId {
            provider_controller_type_id: provider_controller_type_id.clone(),
            inner: WithCredentialControllerTypeId {
                credential_controller_type_id: credential_controller_type_id.clone(),
                inner: params,
            },
        },
    )
    .await;
    trace!(success = res.is_ok(), "Starting user credential brokering completed");

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
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "OAuth callback",
    description = "Handle OAuth callback to complete user credential brokering flow",
    operation_id = "resume-user-credential-brokering",
)]
pub async fn generic_oauth_callback(
    State(ctx): State<BridgeService>,
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
// Function endpoints
// ============================================================================

#[utoipa::path(
    delete,
    path = format!("{}/{}/{}/provider/{{provider_instance_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("provider_instance_id" = String, Path, description = "Provider instance ID"),
    ),
    responses(
        (status = 200, description = "Delete provider instance", body = ()),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Delete provider",
    description = "Delete a provider instance by its unique identifier",
    operation_id = "delete-provider-instance",
)]
pub async fn route_delete_provider_instance(
    State(ctx): State<BridgeService>,
    Path(provider_instance_id): Path<String>,
) -> JsonResponse<(), CommonError> {
    trace!(provider_instance_id = %provider_instance_id, "Deleting provider instance");
    let res = delete_provider_instance(
        ctx.on_config_change_tx(),
        ctx.repository(),
        WithProviderInstanceId {
            provider_instance_id: provider_instance_id.clone(),
            inner: (),
        },
        true,
    )
    .await;
    trace!(success = res.is_ok(), "Deleting provider instance completed");
    JsonResponse::from(res)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/provider/{{provider_instance_id}}/function/{{function_controller_type_id}}/enable", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = EnableFunctionParamsInner,
    params(
        ("provider_instance_id" = String, Path, description = "Provider instance ID"),
        ("function_controller_type_id" = String, Path, description = "Function controller type ID"),
    ),
    responses(
        (status = 200, description = "Enable function", body = EnableFunctionResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Enable function",
    description = "Enable a function for a provider instance",
    operation_id = "enable-function",
)]
pub async fn route_enable_function(
    State(ctx): State<BridgeService>,
    Path((provider_instance_id, function_controller_type_id)): Path<(String, String)>,
    Json(params): Json<EnableFunctionParamsInner>,
) -> JsonResponse<EnableFunctionResponse, CommonError> {
    trace!(
        provider_instance_id = %provider_instance_id,
        function_type = %function_controller_type_id,
        "Enabling function"
    );
    let res = enable_function(
        ctx.on_config_change_tx(),
        ctx.repository(),
        WithProviderInstanceId {
            provider_instance_id: provider_instance_id.clone(),
            inner: WithFunctionControllerTypeId {
                function_controller_type_id: function_controller_type_id.clone(),
                inner: params,
            },
        },
        true,
    )
    .await;
    trace!(success = res.is_ok(), "Enabling function completed");

    JsonResponse::from(res)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/provider/{{provider_instance_id}}/function/{{function_controller_type_id}}/disable", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("provider_instance_id" = String, Path, description = "Provider instance ID"),
        ("function_controller_type_id" = String, Path, description = "Function controller type ID"),
    ),
    responses(
        (status = 200, description = "Disable function", body = DisableFunctionResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Disable function",
    description = "Disable a function for a provider instance",
    operation_id = "disable-function",
)]
pub async fn route_disable_function(
    State(ctx): State<BridgeService>,
    Path((provider_instance_id, function_controller_type_id)): Path<(String, String)>,
) -> JsonResponse<DisableFunctionResponse, CommonError> {
    trace!(
        provider_instance_id = %provider_instance_id,
        function_type = %function_controller_type_id,
        "Disabling function"
    );
    let res = disable_function(
        ctx.on_config_change_tx(),
        ctx.repository(),
        WithProviderInstanceId {
            provider_instance_id: provider_instance_id.clone(),
            inner: WithFunctionControllerTypeId {
                function_controller_type_id: function_controller_type_id.clone(),
                inner: DisableFunctionParamsInner {},
            },
        },
        true,
    )
    .await;
    trace!(success = res.is_ok(), "Disabling function completed");

    JsonResponse::from(res)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/provider/{{provider_instance_id}}/function/{{function_controller_type_id}}/invoke", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = InvokeFunctionParamsInner,
    params(
        ("provider_instance_id" = String, Path, description = "Provider instance ID"),
        ("function_controller_type_id" = String, Path, description = "Function controller type ID"),
    ),
    responses(
        (status = 200, description = "Invoke function", body = InvokeFunctionResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Invoke function",
    description = "Invoke a function on a provider instance",
    operation_id = "invoke-function",
)]
pub async fn route_invoke_function(
    State(ctx): State<BridgeService>,
    Path((provider_instance_id, function_controller_type_id)): Path<(String, String)>,
    Json(params): Json<InvokeFunctionParamsInner>,
) -> JsonResponse<InvokeFunctionResponse, CommonError> {
    trace!(
        provider_instance_id = %provider_instance_id,
        function_type = %function_controller_type_id,
        "Invoking function"
    );
    let res = invoke_function(
        ctx.repository(),
        ctx.encryption_service(),
        WithProviderInstanceId {
            provider_instance_id: provider_instance_id.clone(),
            inner: WithFunctionInstanceId {
                function_controller_type_id: function_controller_type_id.clone(),
                inner: params,
            },
        },
    )
    .await;
    trace!(success = res.is_ok(), "Invoking function completed");
    JsonResponse::from(res)
}

#[derive(Serialize, Deserialize, Debug, ToSchema, IntoParams)]
#[into_params(style = Form, parameter_in = Query)]
pub struct ListProviderInstancesQuery {
    // TODO: utoipa doesnt support flattening yet https://github.com/juhaku/utoipa/pull/1426
    pub page_size: i64,
    pub next_page_token: Option<String>,
    pub status: Option<String>,
    pub provider_controller_type_id: Option<String>,
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/provider", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ListProviderInstancesQuery
    ),
    responses(
        (status = 200, description = "List provider instances", body = ListProviderInstancesResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "List provider instances",
    description = "List all provider instances with optional filtering by status and provider type",
    operation_id = "list-provider-instances",
)]
pub async fn route_list_provider_instances(
    State(ctx): State<BridgeService>,
    Query(query): Query<ListProviderInstancesQuery>,
) -> JsonResponse<ListProviderInstancesResponse, CommonError> {
    trace!(
        page_size = query.page_size,
        status = ?query.status,
        provider_type = ?query.provider_controller_type_id,
        "Listing provider instances"
    );
    let res = list_provider_instances(
        ctx.repository(),
        ListProviderInstancesParams {
            pagination: PaginationRequest {
                page_size: query.page_size,
                next_page_token: query.next_page_token,
            },
            status: query.status,
            provider_controller_type_id: query.provider_controller_type_id,
        },
    )
    .await;
    trace!(success = res.is_ok(), "Listing provider instances completed");
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/provider/grouped-by-function", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ListProviderInstancesGroupedByFunctionParams
    ),
    responses(
        (status = 200, description = "List provider instances grouped by function", body = ListProviderInstancesGroupedByFunctionResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "List providers by function",
    description = "List provider instances grouped by their associated functions",
    operation_id = "list-provider-instances-grouped-by-function",
)]
pub async fn route_list_provider_instances_grouped_by_function(
    State(ctx): State<BridgeService>,
    Query(query): Query<ListProviderInstancesGroupedByFunctionParams>,
) -> JsonResponse<ListProviderInstancesGroupedByFunctionResponse, CommonError> {
    trace!("Listing provider instances grouped by function");
    let res = list_provider_instances_grouped_by_function(ctx.repository(), query).await;
    trace!(success = res.is_ok(), "Listing provider instances grouped by function completed");
    JsonResponse::from(res)
}

#[derive(Serialize, Deserialize, Debug, ToSchema, IntoParams)]
#[into_params(style = Form, parameter_in = Query)]
pub struct ListFunctionInstancesQuery {
    // TODO: utoipa doesnt support flattening yet https://github.com/juhaku/utoipa/pull/1426
    pub page_size: i64,
    pub next_page_token: Option<String>,
    pub provider_instance_id: Option<String>,
}
#[utoipa::path(
    get,
    path = format!("{}/{}/{}/function-instances", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ListFunctionInstancesQuery
    ),
    responses(
        (status = 200, description = "List function instances", body = ListFunctionInstancesResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "List function instances",
    description = "List all function instances with optional filtering by provider instance",
    operation_id = "list-function-instances",
)]
pub async fn route_list_function_instances(
    State(ctx): State<BridgeService>,
    Query(query): Query<ListFunctionInstancesQuery>,
) -> JsonResponse<ListFunctionInstancesResponse, CommonError> {
    trace!(
        page_size = query.page_size,
        provider_instance_id = ?query.provider_instance_id,
        "Listing function instances"
    );
    let res = list_function_instances(
        ctx.repository(),
        ListFunctionInstancesParams {
            pagination: PaginationRequest {
                page_size: query.page_size,
                next_page_token: query.next_page_token,
            },
            provider_instance_id: query.provider_instance_id,
        },
    )
    .await;
    trace!(success = res.is_ok(), "Listing function instances completed");
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/function-instances/openapi.json", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(),
    responses(
        (status = 200, description = "Get function instances openapi spec", body = String),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Get function OpenAPI spec",
    description = "Get the OpenAPI specification for all function instances",
    operation_id = "get-function-instances-openapi-spec",
)]
pub async fn route_get_function_instances_openapi_spec(
    State(ctx): State<BridgeService>,
) -> JsonResponse<OpenApi, CommonError> {
    trace!("Getting function instances OpenAPI spec");
    let res = get_function_instances_openapi_spec(ctx.repository()).await;
    trace!(success = res.is_ok(), "Getting function instances OpenAPI spec completed");
    JsonResponse::from(res)
}
