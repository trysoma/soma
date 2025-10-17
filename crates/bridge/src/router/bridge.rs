use axum::debug_handler;
use axum::extract::{Json, Path, Query, State};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::logic::{
    create_data_encryption_key, create_provider_instance, create_resource_server_credential, create_user_credential, delete_provider_instance, disable_function, enable_function, encrypt_resource_server_configuration, encrypt_user_credential_configuration, invoke_function, list_available_providers, list_data_encryption_keys, resume_user_credential_brokering, start_user_credential_brokering, BrokerAction, BrokerInput, BrokerState, CreateDataEncryptionKeyParams, CreateDataEncryptionKeyResponse, CreateProviderInstanceParams, CreateProviderInstanceParamsInner, CreateProviderInstanceResponse, CreateResourceServerCredentialParams, CreateResourceServerCredentialParamsInner, CreateResourceServerCredentialResponse, CreateUserCredentialParams, CreateUserCredentialParamsInner, CreateUserCredentialResponse, CryptoService, DataEncryptionKey, DecryptionService, DisableFunctionParams, DisableFunctionParamsInner, DisableFunctionResponse, EnableFunctionParams, EnableFunctionParamsInner, EnableFunctionResponse, EncryptConfigurationParams, EncryptCredentialConfigurationParamsInner, EncryptedCredentialConfigurationResponse, EncryptionService, EnvelopeEncryptionKeyContents, EnvelopeEncryptionKeyId, InvokeFunctionParams, InvokeFunctionParamsInner, InvokeFunctionResponse, ListAvailableProvidersResponse, ListDataEncryptionKeysResponse, OnConfigChangeTx, ResumeUserCredentialBrokeringParams, StartUserCredentialBrokeringParams, StartUserCredentialBrokeringParamsInner, UserCredentialBrokeringResponse, UserCredentialSerialized, WithCredentialControllerTypeId, WithFunctionControllerTypeId, WithFunctionInstanceId, WithProviderControllerTypeId, WithProviderInstanceId
};
use crate::repository::Repository;
use shared::{adapters::openapi::JsonResponse, error::CommonError, primitives::PaginationRequest};

pub const PATH_PREFIX: &str = "/api";
pub const API_VERSION_1: &str = "v1";
pub const SERVICE_ROUTE_KEY: &str = "bridge";

pub fn create_router() -> OpenApiRouter<Arc<BridgeService>> {
    OpenApiRouter::new()
        // Provider endpoints
        .routes(routes!(route_list_available_providers))
        // Data encryption key endpoints
        .routes(routes!(route_create_data_encryption_key))
        .routes(routes!(route_list_data_encryption_keys))
        // Configuration endpoints
        .routes(routes!(route_encrypt_resource_server_configuration))
        .routes(routes!(route_encrypt_user_credential_configuration))
        // Resource server credential endpoints
        .routes(routes!(route_create_resource_server_credential))
        // User credential endpoints
        .routes(routes!(route_create_user_credential))
        // User credential brokering endpoints
        .routes(routes!(route_start_user_credential_brokering))
        .routes(routes!(generic_oauth_callback))
        // Provider instance endpoints
        .routes(routes!(route_create_provider_instance))
        .routes(routes!(route_delete_provider_instance))
        // Function endpoints
        .routes(routes!(route_enable_function))
        .routes(routes!(route_disable_function))
        .routes(routes!(route_invoke_function))
}

// ============================================================================
// Provider endpoints
// ============================================================================

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/available-providers", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    params(
        PaginationRequest
    ),
    responses(
        (status = 200, description = "List available providers", body = ListAvailableProvidersResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    operation_id = "list-available-providers",
)]
async fn route_list_available_providers(
    State(_ctx): State<Arc<BridgeService>>,
    Query(pagination): Query<PaginationRequest>,
) -> JsonResponse<ListAvailableProvidersResponse, CommonError> {
    let res = list_available_providers(pagination).await;
    JsonResponse::from(res)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/available-providers/{{provider_controller_type_id}}/available-credentials/{{credential_controller_type_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    request_body = CreateProviderInstanceParamsInner,
    responses(
        (status = 200, description = "Create provider instance", body = CreateProviderInstanceResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    operation_id = "create-provider-instance",
)]
async fn route_create_provider_instance(
    State(ctx): State<Arc<BridgeService>>,
    Path((provider_controller_type_id, credential_controller_type_id)): Path<(String, String)>,
    Json(params): Json<CreateProviderInstanceParamsInner>,
) -> JsonResponse<CreateProviderInstanceResponse, CommonError> {
    let res = create_provider_instance(
        &ctx.on_config_change_tx,
        &ctx.repository,
        WithProviderControllerTypeId {
            provider_controller_type_id: provider_controller_type_id.clone(),
            inner: WithCredentialControllerTypeId {
                credential_controller_type_id: credential_controller_type_id.clone(),
                inner: params,
            },
        },
    )
    .await;
    JsonResponse::from(res)
}

// ============================================================================
// Data encryption key endpoints
// ============================================================================

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/encryption/data-encryption-key", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    request_body = CreateDataEncryptionKeyParams,
    responses(
        (status = 200, description = "Create data encryption key", body = CreateDataEncryptionKeyResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    operation_id = "create-data-encryption-key",
)]
async fn route_create_data_encryption_key(
    State(ctx): State<Arc<BridgeService>>,
    Json(params): Json<CreateDataEncryptionKeyParams>,
) -> JsonResponse<CreateDataEncryptionKeyResponse, CommonError> {
    let res = create_data_encryption_key(
        &ctx.envelope_encryption_key_contents,
        &ctx.on_config_change_tx,
        &ctx.repository,
        params,
    )
    .await;
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/encryption/data-encryption-key", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    params(
        PaginationRequest
    ),
    responses(
        (status = 200, description = "List data encryption keys", body = ListDataEncryptionKeysResponse),
    ),
    operation_id = "list-data-encryption-keys",
)]
async fn route_list_data_encryption_keys(
    State(ctx): State<Arc<BridgeService>>,
    Query(pagination): Query<PaginationRequest>,
) -> JsonResponse<ListDataEncryptionKeysResponse, CommonError> {
    let res = list_data_encryption_keys(&ctx.repository, pagination).await;
    JsonResponse::from(res)
}

// ============================================================================
// Configuration endpoints
// ============================================================================

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/available-providers/{{provider_controller_type_id}}/available-credentials/{{credential_controller_type_id}}/credential/resource-server/encrypt", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    request_body = EncryptCredentialConfigurationParamsInner,
    params(
        ("provider_controller_type_id" = String, Path, description = "Provider controller type ID"),
        ("credential_controller_type_id" = String, Path, description = "Credential controller type ID"),
    ),
    responses(
        (status = 200, description = "Encrypt provider configuration", body = EncryptedCredentialConfigurationResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    operation_id = "encrypt-resource-server-configuration",
)]
async fn route_encrypt_resource_server_configuration(
    State(ctx): State<Arc<BridgeService>>,
    Path((provider_controller_type_id, credential_controller_type_id)): Path<(String, String)>,
    Json(params): Json<EncryptCredentialConfigurationParamsInner>,
) -> JsonResponse<EncryptedCredentialConfigurationResponse, CommonError> {
    let res = encrypt_resource_server_configuration(
        &ctx.envelope_encryption_key_contents,
        &ctx.repository,
        WithProviderControllerTypeId {
            provider_controller_type_id: provider_controller_type_id.clone(),
            inner: WithCredentialControllerTypeId {
                credential_controller_type_id: credential_controller_type_id.clone(),
                inner: params,
            },
        },
    )
    .await;
    JsonResponse::from(res)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/available-providers/{{provider_controller_type_id}}/available-credentials/{{credential_controller_type_id}}/credential/user-credential/encrypt", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
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

    operation_id = "encrypt-user-credential-configuration",
)]
async fn route_encrypt_user_credential_configuration(
    State(ctx): State<Arc<BridgeService>>,
    Path((provider_controller_type_id, credential_controller_type_id)): Path<(String, String)>,
    Json(params): Json<EncryptCredentialConfigurationParamsInner>,
) -> JsonResponse<EncryptedCredentialConfigurationResponse, CommonError> {
    let res = encrypt_user_credential_configuration(
        &ctx.envelope_encryption_key_contents,
        &ctx.repository,
        WithProviderControllerTypeId {
            provider_controller_type_id: provider_controller_type_id.clone(),
            inner: WithCredentialControllerTypeId {
                credential_controller_type_id: credential_controller_type_id.clone(),
                inner: params,
            },
        },
    )
    .await;
    JsonResponse::from(res)
}
// ============================================================================
// Resource server credential endpoints
// ============================================================================

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/available-providers/{{provider_controller_type_id}}/available-credentials/{{credential_controller_type_id}}/credential/resource-server", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
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
    operation_id = "create-resource-server-credential",
)]
async fn route_create_resource_server_credential(
    State(ctx): State<Arc<BridgeService>>,
    Path((provider_controller_type_id, credential_controller_type_id)): Path<(String, String)>,
    Json(params): Json<CreateResourceServerCredentialParamsInner>,
) -> JsonResponse<CreateResourceServerCredentialResponse, CommonError> {
    let res = create_resource_server_credential(
        &ctx.repository,
        WithProviderControllerTypeId {
            provider_controller_type_id: provider_controller_type_id.clone(),
            inner: WithCredentialControllerTypeId {
                credential_controller_type_id: credential_controller_type_id.clone(),
                inner: params,
            },
        },
    )
    .await;
    JsonResponse::from(res)
}

// ============================================================================
// User credential endpoints
// ============================================================================

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/available-providers/{{provider_controller_type_id}}/available-credentials/{{credential_controller_type_id}}/credential/user-credential", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
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
    operation_id = "create-user-credential",
)]
async fn route_create_user_credential(
    State(ctx): State<Arc<BridgeService>>,
    Path((provider_controller_type_id, credential_controller_type_id)): Path<(String, String)>,
    Json(params): Json<CreateUserCredentialParamsInner>,
) -> JsonResponse<CreateUserCredentialResponse, CommonError> {
    let res = create_user_credential(
        &ctx.repository,
        WithProviderControllerTypeId {
            provider_controller_type_id: provider_controller_type_id.clone(),
            inner: WithCredentialControllerTypeId {
                credential_controller_type_id: credential_controller_type_id.clone(),
                inner: params,
            },
        },
    )
    .await;
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
            BrokerAction::Redirect { url } => {
                axum::response::Redirect::to(url.as_str()).into_response()
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
    }
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/available-providers/{{provider_controller_type_id}}/available-credentials/{{credential_controller_type_id}}/credential/user-credential/broker", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
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
    operation_id = "start-user-credential-brokering",
)]
async fn route_start_user_credential_brokering(
    State(ctx): State<Arc<BridgeService>>,
    Path((provider_controller_type_id, credential_controller_type_id)): Path<(String, String)>,
    Json(params): Json<StartUserCredentialBrokeringParamsInner>,
) -> impl IntoResponse {
    let res = start_user_credential_brokering(
        &ctx.repository,
        WithProviderControllerTypeId {
            provider_controller_type_id: provider_controller_type_id.clone(),
            inner: WithCredentialControllerTypeId {
                credential_controller_type_id: credential_controller_type_id.clone(),
                inner: params,
            },
        },
    )
    .await;

    return handle_user_credential_brokering_response(res).into_response();
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
    operation_id = "resume-user-credential-brokering",
)]
async fn generic_oauth_callback(
    State(ctx): State<Arc<BridgeService>>,
    Query(params): Query<GenericOAuthCallbackParams>,
) -> impl IntoResponse {
    // Check for OAuth errors first
    if let Some(error) = params.error {
        let error_desc = params
            .error_description
            .unwrap_or_else(|| "No description provided".to_string());
        return respond_err!(CommonError::Unknown(anyhow::anyhow!(
            "OAuth error: {} - {}",
            error,
            error_desc
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

    // Resume the user credential brokering flow
    let res = resume_user_credential_brokering(
        &ctx.repository,
        ResumeUserCredentialBrokeringParams {
            broker_state_id: state,
            input: broker_input,
        },
    )
    .await;

    return handle_user_credential_brokering_response(res).into_response();
}

// ============================================================================
// Function endpoints
// ============================================================================

#[utoipa::path(
    delete,
    path = format!("{}/{}/{}/provider/{{provider_instance_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    params(
        ("provider_instance_id" = String, Path, description = "Provider instance ID"),
    ),
    responses(
        (status = 200, description = "Delete provider instance", body = ()),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    operation_id = "delete-provider-instance",
)]
async fn route_delete_provider_instance(
    State(ctx): State<Arc<BridgeService>>,
    Path(provider_instance_id): Path<String>,
) -> JsonResponse<(), CommonError> {
    let res = delete_provider_instance(
        &ctx.on_config_change_tx,
        &ctx.repository,
        WithProviderInstanceId {
            provider_instance_id: provider_instance_id.clone(),
            inner: (),
        },
    )
    .await;
    JsonResponse::from(res)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/provider/{{provider_instance_id}}/available-functions/{{function_controller_type_id}}/enable", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
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
    operation_id = "enable-function",
)]
async fn route_enable_function(
    State(ctx): State<Arc<BridgeService>>,
    Path((provider_instance_id, function_controller_type_id)): Path<(String, String)>,
    Json(params): Json<EnableFunctionParamsInner>,
) -> JsonResponse<EnableFunctionResponse, CommonError> {
    let res = enable_function(
        &ctx.on_config_change_tx,
        &ctx.repository,
        WithProviderInstanceId {
            provider_instance_id: provider_instance_id.clone(),
            inner: WithFunctionControllerTypeId {
                function_controller_type_id: function_controller_type_id.clone(),
                inner: params,
            },
        },
    )
    .await;
    JsonResponse::from(res)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/provider/{{provider_instance_id}}/function/{{function_instance_id}}/disable", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    params(
        ("provider_instance_id" = String, Path, description = "Provider instance ID"),
        ("function_instance_id" = String, Path, description = "Function instance ID"),
    ),
    responses(
        (status = 200, description = "Disable function", body = DisableFunctionResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    operation_id = "disable-function",
)]
async fn route_disable_function(
    State(ctx): State<Arc<BridgeService>>,
    Path((provider_instance_id, function_instance_id)): Path<(String, String)>,
) -> JsonResponse<DisableFunctionResponse, CommonError> {
    let res = disable_function(
        &ctx.on_config_change_tx,
        &ctx.repository,
        WithProviderInstanceId {
            provider_instance_id: provider_instance_id.clone(),
            inner: DisableFunctionParamsInner {
                function_instance_id: function_instance_id.clone(),
            },
        },
    )
    .await;
    JsonResponse::from(res)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/provider/{{provider_instance_id}}/function/{{function_instance_id}}/invoke", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    request_body = InvokeFunctionParamsInner,
    params(
        ("provider_instance_id" = String, Path, description = "Provider instance ID"),
        ("function_instance_id" = String, Path, description = "Function instance ID"),
    ),
    responses(
        (status = 200, description = "Invoke function", body = InvokeFunctionResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    operation_id = "invoke-function",
)]
async fn route_invoke_function(
    State(ctx): State<Arc<BridgeService>>,
    Path((provider_instance_id, function_instance_id)): Path<(String, String)>,
    Json(params): Json<InvokeFunctionParamsInner>,
) -> JsonResponse<InvokeFunctionResponse, CommonError> {
    let res = invoke_function(
        &ctx.repository,
        &ctx.envelope_encryption_key_contents,
        WithProviderInstanceId {
            provider_instance_id: provider_instance_id.clone(),
            inner: WithFunctionInstanceId {
                function_instance_id: function_instance_id.clone(),
                inner: params,
            },
        },
    )
    .await;
    JsonResponse::from(res)
}

// ============================================================================
// Service
// ============================================================================

pub struct BridgeService {
    repository: Repository,
    on_config_change_tx: OnConfigChangeTx,
    envelope_encryption_key_contents: EnvelopeEncryptionKeyContents,
}

impl BridgeService {
    pub fn new(
        repository: Repository,
        on_config_change_tx: OnConfigChangeTx,
        envelope_encryption_key_contents: EnvelopeEncryptionKeyContents,
    ) -> Self {
        Self {
            repository,
            on_config_change_tx,
            envelope_encryption_key_contents,
        }
    }
}
