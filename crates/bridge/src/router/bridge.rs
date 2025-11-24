use crate::logic::{
    BrokerAction, BrokerInput, CreateDataEncryptionKeyResponse,
    CreateProviderInstanceParamsInner, CreateProviderInstanceResponse,
    CreateResourceServerCredentialParamsInner, CreateResourceServerCredentialResponse,
    CreateUserCredentialParamsInner, CreateUserCredentialResponse, DisableFunctionParamsInner,
    DisableFunctionResponse, EnableFunctionParamsInner, EnableFunctionResponse,
    EncryptCredentialConfigurationParamsInner, EncryptedCredentialConfigurationResponse,
    EnvelopeEncryptionKeyContents, GetProviderInstanceResponse, InvokeFunctionParamsInner,
    InvokeFunctionResponse, ListAvailableProvidersResponse, ListDataEncryptionKeysResponse,
    ListFunctionInstancesParams, ListFunctionInstancesResponse,
    ListProviderInstancesGroupedByFunctionParams, ListProviderInstancesGroupedByFunctionResponse,
    ListProviderInstancesParams, ListProviderInstancesResponse, MigrateEncryptionKeyParams,
    MigrateEncryptionKeyResponse, OnConfigChangeTx, ResumeUserCredentialBrokeringParams,
    StartUserCredentialBrokeringParamsInner, UpdateProviderInstanceParamsInner,
    UpdateProviderInstanceResponse, UserCredentialBrokeringResponse, UserCredentialSerialized,
    WithCredentialControllerTypeId, WithFunctionControllerTypeId, WithFunctionInstanceId,
    WithProviderControllerTypeId, WithProviderInstanceId, create_data_encryption_key,
    create_provider_instance, create_resource_server_credential, create_user_credential,
    delete_provider_instance, disable_function, enable_function,
    encrypt_resource_server_configuration, encrypt_user_credential_configuration,
    get_function_instances_openapi_spec, get_provider_instance, invoke_function,
    list_available_providers, list_data_encryption_keys, list_function_instances,
    list_provider_instances, list_provider_instances_grouped_by_function,
    migrate_encryption_key, process_credential_rotations_with_window,
    resume_user_credential_brokering, start_user_credential_brokering, update_provider_instance,
};
use crate::logic::encryption::{
    CreateDataEncryptionKeyParamsBridge, DeleteDataEncryptionKeyByIdentifierParams,
    MigrateEncryptionKeyByIdentifierParams, migrate_encryption_key_by_identifier,
    find_envelope_encryption_key_by_arn, find_envelope_encryption_key_by_location,
    CreateDataEncryptionKeyParams,
};
use crate::repository::ProviderRepositoryLike;
use crate::repository::Repository;
use axum::Extension;
use axum::extract::{Json, NestedPath, Path, Query, State};
use axum::response::sse::{Event, KeepAlive};
use axum::response::{IntoResponse, Response, Sse};
use http::StatusCode;
use http::request::Parts;
use rmcp::{
    model::ClientJsonRpcMessage,
    transport::{
        common::server_side_http::session_id,
        sse_server::{PostEventQuery, SseServerTransport},
    },
};
use serde::{Deserialize, Serialize};
use shared::{adapters::openapi::JsonResponse, error::CommonError, primitives::PaginationRequest};
use std::collections::HashMap;
use std::io;
use std::sync::Arc;
use std::time::Duration;
use arc_swap::ArcSwap;
use tracing::info;
use utoipa::openapi::OpenApi;
use utoipa::{IntoParams, PartialSchema, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

pub const PATH_PREFIX: &str = "/api";
pub const API_VERSION_1: &str = "v1";
pub const SERVICE_ROUTE_KEY: &str = "bridge";

pub fn create_router() -> OpenApiRouter<BridgeService> {
    OpenApiRouter::new()
        // Provider endpoints
        .routes(routes!(route_list_available_providers))
        // Data encryption key endpoints
        .routes(routes!(route_create_data_encryption_key))
        .routes(routes!(route_list_data_encryption_keys))
        .routes(routes!(route_migrate_encryption_key))
        .routes(routes!(route_migrate_encryption_key_by_identifier))
        .routes(routes!(route_delete_data_encryption_key_by_identifier))
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
        .routes(routes!(route_update_provider_instance))
        .routes(routes!(route_delete_provider_instance))
        .routes(routes!(route_get_provider_instance))
        .routes(routes!(route_list_provider_instances))
        .routes(routes!(route_list_provider_instances_grouped_by_function))
        // Function endpoints
        .routes(routes!(route_enable_function))
        .routes(routes!(route_disable_function))
        .routes(routes!(route_invoke_function))
        .routes(routes!(route_list_function_instances))
        .routes(routes!(route_get_function_instances_openapi_spec))
        // mcp endpoints
        .routes(routes!(mcp_sse))
        .routes(routes!(mcp_message))
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
    State(_ctx): State<BridgeService>,
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
    State(ctx): State<BridgeService>,
    Path((provider_controller_type_id, credential_controller_type_id)): Path<(String, String)>,
    Json(params): Json<CreateProviderInstanceParamsInner>,
) -> JsonResponse<CreateProviderInstanceResponse, CommonError> {
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
    JsonResponse::from(res)
}

#[utoipa::path(
    patch,
    path = format!("{}/{}/{}/provider/{{provider_instance_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    request_body = UpdateProviderInstanceParamsInner,
    params(
        ("provider_instance_id" = String, Path, description = "Provider instance ID"),
    ),
    responses(
        (status = 200, description = "Update provider instance", body = UpdateProviderInstanceResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    operation_id = "update-provider-instance",
)]
async fn route_update_provider_instance(
    State(ctx): State<BridgeService>,
    Path(provider_instance_id): Path<String>,
    Json(params): Json<UpdateProviderInstanceParamsInner>,
) -> JsonResponse<UpdateProviderInstanceResponse, CommonError> {
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
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/provider/{{provider_instance_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    params(
        ("provider_instance_id" = String, Path, description = "Provider instance ID"),
    ),
    responses(
        (status = 200, description = "Get provider instance", body = GetProviderInstanceResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    operation_id = "get-provider-instance",
)]
async fn route_get_provider_instance(
    State(ctx): State<BridgeService>,
    Path(provider_instance_id): Path<String>,
) -> JsonResponse<GetProviderInstanceResponse, CommonError> {
    let res = get_provider_instance(
        ctx.repository(),
        WithProviderInstanceId {
            provider_instance_id: provider_instance_id.clone(),
            inner: (),
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
    request_body = CreateDataEncryptionKeyParamsBridge,
    responses(
        (status = 200, description = "Create data encryption key", body = CreateDataEncryptionKeyResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    operation_id = "create-data-encryption-key",
)]
async fn route_create_data_encryption_key(
    State(ctx): State<BridgeService>,
    Json(params): Json<CreateDataEncryptionKeyParamsBridge>,
) -> JsonResponse<CreateDataEncryptionKeyResponse, CommonError> {
    // Determine which envelope encryption key to use
    let envelope_key = if let Some(ref identifier) = params.envelope_encryption_key_identifier {
        // Try to get the key from the map
        match ctx.get_envelope_encryption_key_contents(identifier) {
            Ok(key) => key,
            Err(_) => {
                // Key not in map, need to add it
                // For AWS KMS, construct from ARN and region
                // For local, load from file
                let key_contents = if identifier.starts_with("arn:aws:kms:") {
                    // Use provided region if available, otherwise extract from ARN
                    let region = if let Some(ref provided_region) = params.aws_region {
                        provided_region.clone()
                    } else {
                        match encryption::extract_region_from_kms_arn(identifier) {
                            Ok(region_from_arn) => region_from_arn,
                            Err(_) => {
                                return JsonResponse::from(Err(CommonError::Unknown(anyhow::anyhow!(
                                    "Invalid AWS KMS ARN format: {identifier}. Please provide ARN in format: arn:aws:kms:REGION:ACCOUNT:key/KEY-ID or provide aws_region parameter"
                                ))));
                            }
                        }
                    };
                    EnvelopeEncryptionKeyContents::AwsKms {
                        arn: identifier.clone(),
                        region,
                    }
                } else {
                    // Local key - load from file
                    match encryption::get_or_create_local_encryption_key(
                        &std::path::PathBuf::from(identifier),
                    ) {
                        Ok(key) => key,
                        Err(e) => return JsonResponse::from(Err(e)),
                    }
                };
                // Add to map
                if let Err(e) = ctx.add_envelope_encryption_key_contents(key_contents.clone()) {
                    return JsonResponse::from(Err(e));
                }
                key_contents
            }
        }
    } else {
        // Use default key (first one in map)
        match ctx.0.envelope_encryption_key_contents.load().values().next() {
            Some(key) => key.clone(),
            None => {
                return JsonResponse::from(Err(CommonError::Unknown(anyhow::anyhow!(
                    "No envelope encryption key configured and none specified"
                ))));
            }
        }
    };

    // Convert bridge params to encryption params
    let encryption_params = CreateDataEncryptionKeyParams {
        id: params.id.clone(),
        encrypted_data_envelope_key: params.encrypted_data_envelope_key.clone(),
    };

    let res = create_data_encryption_key(
        &envelope_key,
        ctx.on_config_change_tx(),
        ctx.repository(),
        encryption_params,
        true,
    )
    .await;

    // After creating the DEK, add the parent encryption key to the map if not already present
    if let Ok(dek) = &res {
        let identifier = get_envelope_key_id_identifier(&dek.envelope_encryption_key_id);
        // Check if key exists, if not, we need to construct it from the DEK's envelope_encryption_key_id
        if ctx.get_envelope_encryption_key_contents(&identifier).is_err() {
            // Key not in map, need to add it
            // For AWS KMS, we have ARN and region from the DEK
            // For local, we need to load the key bytes from the file
            let key_contents = match &dek.envelope_encryption_key_id {
                crate::logic::encryption::EnvelopeEncryptionKeyId::AwsKms { arn, region } => {
                    EnvelopeEncryptionKeyContents::AwsKms {
                        arn: arn.clone(),
                        region: region.clone(),
                    }
                }
                crate::logic::encryption::EnvelopeEncryptionKeyId::Local { location } => {
                    match encryption::get_or_create_local_encryption_key(
                        &std::path::PathBuf::from(location),
                    ) {
                        Ok(key) => key,
                        Err(e) => return JsonResponse::from(Err(e)),
                    }
                }
            };
            if let Err(e) = ctx.add_envelope_encryption_key_contents(key_contents) {
                return JsonResponse::from(Err(e));
            }
        }
    }

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
    State(ctx): State<BridgeService>,
    Query(pagination): Query<PaginationRequest>,
) -> JsonResponse<ListDataEncryptionKeysResponse, CommonError> {
    let res = list_data_encryption_keys(ctx.repository(), pagination).await;
    JsonResponse::from(res)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/encryption/migrate", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    request_body = MigrateEncryptionKeyParams,
    responses(
        (status = 200, description = "Migrate encryption key", body = MigrateEncryptionKeyResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    operation_id = "migrate-encryption-key",
)]
async fn route_migrate_encryption_key(
    State(ctx): State<BridgeService>,
    Json(params): Json<MigrateEncryptionKeyParams>,
) -> JsonResponse<MigrateEncryptionKeyResponse, CommonError> {
    // Get the envelope encryption key contents from the IDs
    let from_envelope_key = match ctx.get_envelope_encryption_key_contents_by_id(
        &params.from_envelope_encryption_key_id,
    ) {
        Ok(key) => key,
        Err(e) => return JsonResponse::from(Err(e)),
    };
    let to_envelope_key = match ctx.get_envelope_encryption_key_contents_by_id(
        &params.to_envelope_encryption_key_id,
    ) {
        Ok(key) => key,
        Err(e) => return JsonResponse::from(Err(e)),
    };

    let res = migrate_encryption_key(
        &from_envelope_key,
        &to_envelope_key,
        ctx.on_config_change_tx(),
        ctx.repository(),
        params,
    )
    .await;

    JsonResponse::from(res)
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct DeleteDataEncryptionKeyByIdentifierResponse {
    pub deleted_count: usize,
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/encryption/migrate-by-identifier", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    request_body = MigrateEncryptionKeyByIdentifierParams,
    responses(
        (status = 200, description = "Migrate encryption key by identifier", body = MigrateEncryptionKeyResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    operation_id = "migrate-encryption-key-by-identifier",
)]
async fn route_migrate_encryption_key_by_identifier(
    State(ctx): State<BridgeService>,
    Json(params): Json<MigrateEncryptionKeyByIdentifierParams>,
) -> JsonResponse<MigrateEncryptionKeyResponse, CommonError> {
    // Use a default key for the bridge parameter (not used in the actual migration logic)
    let default_key = match ctx.0.envelope_encryption_key_contents.load().keys().next() {
        Some(key_id) => {
            match ctx.get_envelope_encryption_key_contents(key_id) {
                Ok(key) => key,
                Err(_) => {
                    // Fallback: create a dummy key (shouldn't happen in practice)
                    EnvelopeEncryptionKeyContents::Local {
                        location: "/tmp/dummy".to_string(),
                        key_bytes: vec![0u8; 32],
                    }
                }
            }
        }
        None => {
            return JsonResponse::from(Err(CommonError::Unknown(anyhow::anyhow!(
                "No envelope encryption key configured"
            ))));
        }
    };

    let res = migrate_encryption_key_by_identifier(
        &default_key,
        ctx.on_config_change_tx(),
        ctx.repository(),
        params,
    )
    .await;

    JsonResponse::from(res)
}

#[utoipa::path(
    delete,
    path = format!("{}/{}/{}/encryption/data-encryption-key/by-identifier", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    request_body = DeleteDataEncryptionKeyByIdentifierParams,
    responses(
        (status = 200, description = "Delete data encryption keys by identifier", body = DeleteDataEncryptionKeyByIdentifierResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    operation_id = "delete-data-encryption-key-by-identifier",
)]
async fn route_delete_data_encryption_key_by_identifier(
    State(ctx): State<BridgeService>,
    Json(params): Json<DeleteDataEncryptionKeyByIdentifierParams>,
) -> JsonResponse<DeleteDataEncryptionKeyByIdentifierResponse, CommonError> {
    // Check if any DEKs are using this parent encryption key
    let envelope_key_id = if params.identifier.starts_with("arn:aws:kms:") {
        match find_envelope_encryption_key_by_arn(ctx.repository(), &params.identifier).await {
            Ok(Some(key_id)) => key_id,
            Ok(None) => {
                return JsonResponse::from(Err(CommonError::Unknown(anyhow::anyhow!(
                    "No data encryption key found with ARN: {}",
                    params.identifier
                ))));
            }
            Err(e) => return JsonResponse::from(Err(e)),
        }
    } else {
        match find_envelope_encryption_key_by_location(ctx.repository(), &params.identifier).await {
            Ok(Some(key_id)) => key_id,
            Ok(None) => {
                return JsonResponse::from(Err(CommonError::Unknown(anyhow::anyhow!(
                    "No data encryption key found with location: {}",
                    params.identifier
                ))));
            }
            Err(e) => return JsonResponse::from(Err(e)),
        }
    };

    // Check if any DEKs are using this envelope encryption key
    use shared::primitives::PaginationRequest;
    let mut has_deks = false;
    let mut page_token = None;
    loop {
        let deks = match crate::logic::encryption::list_data_encryption_keys(
            ctx.repository(),
            PaginationRequest {
                page_size: 100,
                next_page_token: page_token.clone(),
            },
        )
        .await
        {
            Ok(deks) => deks,
            Err(e) => return JsonResponse::from(Err(e)),
        };

        for dek_item in &deks.items {
            if crate::logic::encryption::matches_envelope_key_id(&dek_item.envelope_encryption_key_id, &envelope_key_id) {
                has_deks = true;
                break;
            }
        }

        if has_deks || deks.next_page_token.is_none() {
            break;
        }
        page_token = deks.next_page_token;
    }

    if has_deks {
        return JsonResponse::from(Err(CommonError::Unknown(anyhow::anyhow!(
            "Cannot remove parent encryption key '{}': some database secrets are currently encrypted with data encryption keys that use this parent encryption key. Please migrate the encryption keys first.",
            params.identifier
        ))));
    }

    // No DEKs using this key, safe to remove
    // Remove from the map
    match ctx.remove_envelope_encryption_key_contents(&params.identifier) {
        Ok(_) => {}
        Err(e) => return JsonResponse::from(Err(e)),
    }

    // Return success (no DEKs to delete since we're removing the parent key, not DEKs)
    JsonResponse::from(Ok(DeleteDataEncryptionKeyByIdentifierResponse {
        deleted_count: 0,
    }))
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
    State(ctx): State<BridgeService>,
    Path((provider_controller_type_id, credential_controller_type_id)): Path<(String, String)>,
    Json(params): Json<EncryptCredentialConfigurationParamsInner>,
) -> JsonResponse<EncryptedCredentialConfigurationResponse, CommonError> {
    // Get the envelope encryption key contents from the DEK ID
    let envelope_key = match ctx
        .get_envelope_encryption_key_contents_from_dek_id(&params.data_encryption_key_id)
        .await
    {
        Ok(key) => key,
        Err(e) => return JsonResponse::from(Err(e)),
    };

    let res = encrypt_resource_server_configuration(
        &envelope_key,
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
    State(ctx): State<BridgeService>,
    Path((provider_controller_type_id, credential_controller_type_id)): Path<(String, String)>,
    Json(params): Json<EncryptCredentialConfigurationParamsInner>,
) -> JsonResponse<EncryptedCredentialConfigurationResponse, CommonError> {
    // Get the envelope encryption key contents from the DEK ID
    let envelope_key = match ctx
        .get_envelope_encryption_key_contents_from_dek_id(&params.data_encryption_key_id)
        .await
    {
        Ok(key) => key,
        Err(e) => return JsonResponse::from(Err(e)),
    };

    let res = encrypt_user_credential_configuration(
        &envelope_key,
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
    State(ctx): State<BridgeService>,
    Path((provider_controller_type_id, credential_controller_type_id)): Path<(String, String)>,
    Json(params): Json<CreateResourceServerCredentialParamsInner>,
) -> JsonResponse<CreateResourceServerCredentialResponse, CommonError> {
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
    State(ctx): State<BridgeService>,
    Path((provider_controller_type_id, credential_controller_type_id)): Path<(String, String)>,
    Json(params): Json<CreateUserCredentialParamsInner>,
) -> JsonResponse<CreateUserCredentialResponse, CommonError> {
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
        UserCredentialBrokeringResponse::Redirect(url) => {
            axum::response::Redirect::to(url.as_str()).into_response()
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
    State(ctx): State<BridgeService>,
    Path((provider_controller_type_id, credential_controller_type_id)): Path<(String, String)>,
    Json(params): Json<StartUserCredentialBrokeringParamsInner>,
) -> JsonResponse<UserCredentialBrokeringResponse, CommonError> {
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
    State(ctx): State<BridgeService>,
    Query(params): Query<GenericOAuthCallbackParams>,
) -> impl IntoResponse {
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

    // Get the envelope encryption key from the broker state's DEK
    // First, get the broker state to find the DEK
    let broker_state = match ctx.repository().get_broker_state_by_id(&state).await {
        Ok(Some(state)) => state,
        Ok(None) => {
            return respond_err!(CommonError::Unknown(anyhow::anyhow!(
                "Broker state not found: {state}"
            )));
        }
        Err(e) => return respond_err!(e),
    };
    
    // Get the provider instance to find the resource server credential's DEK
    let provider_instance = match ctx
        .repository()
        .get_provider_instance_by_id(&broker_state.provider_instance_id)
        .await
    {
        Ok(Some(instance)) => instance,
        Ok(None) => {
            return respond_err!(CommonError::Unknown(anyhow::anyhow!(
                "Provider instance not found: {}",
                broker_state.provider_instance_id
            )));
        }
        Err(e) => return respond_err!(e),
    };
    
    let resource_server_cred = match ctx
        .repository()
        .get_resource_server_credential_by_id(
            &provider_instance.provider_instance.resource_server_credential_id,
        )
        .await
    {
        Ok(Some(cred)) => cred,
        Ok(None) => {
            return respond_err!(CommonError::Unknown(anyhow::anyhow!(
                "Resource server credential not found"
            )));
        }
        Err(e) => return respond_err!(e),
    };

    let envelope_key = match ctx
        .get_envelope_encryption_key_contents_from_dek_id(&resource_server_cred.data_encryption_key_id)
        .await
    {
        Ok(key) => key,
        Err(e) => return respond_err!(e),
    };

    // Resume the user credential brokering flow
    let res = resume_user_credential_brokering(
        ctx.on_config_change_tx(),
        ctx.repository(),
        &envelope_key,
        ResumeUserCredentialBrokeringParams {
            broker_state_id: state,
            input: broker_input,
        },
    )
    .await;

    handle_user_credential_brokering_response(res).into_response()
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
    State(ctx): State<BridgeService>,
    Path(provider_instance_id): Path<String>,
) -> JsonResponse<(), CommonError> {
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
    JsonResponse::from(res)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/provider/{{provider_instance_id}}/function/{{function_controller_type_id}}/enable", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
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
    State(ctx): State<BridgeService>,
    Path((provider_instance_id, function_controller_type_id)): Path<(String, String)>,
    Json(params): Json<EnableFunctionParamsInner>,
) -> JsonResponse<EnableFunctionResponse, CommonError> {
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

    // Trigger bridge client generation if function was enabled successfully
    if res.is_ok() {
        let mut sdk_client_guard = ctx.sdk_client().lock().await;
        if let Some(ref mut client) = *sdk_client_guard {
            if let Err(e) =
                crate::logic::codegen::trigger_bridge_client_generation(client, ctx.repository())
                    .await
            {
                tracing::warn!("Failed to trigger bridge client generation: {:?}", e);
            }
        }
    }

    JsonResponse::from(res)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/provider/{{provider_instance_id}}/function/{{function_controller_type_id}}/disable", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    params(
        ("provider_instance_id" = String, Path, description = "Provider instance ID"),
        ("function_controller_type_id" = String, Path, description = "Function controller type ID"),
    ),
    responses(
        (status = 200, description = "Disable function", body = DisableFunctionResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    operation_id = "disable-function",
)]
async fn route_disable_function(
    State(ctx): State<BridgeService>,
    Path((provider_instance_id, function_controller_type_id)): Path<(String, String)>,
) -> JsonResponse<DisableFunctionResponse, CommonError> {
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

    // Trigger bridge client generation if function was disabled successfully
    if res.is_ok() {
        let mut sdk_client_guard = ctx.sdk_client().lock().await;
        if let Some(ref mut client) = *sdk_client_guard {
            if let Err(e) =
                crate::logic::codegen::trigger_bridge_client_generation(client, ctx.repository())
                    .await
            {
                tracing::warn!("Failed to trigger bridge client generation: {:?}", e);
            }
        }
    }

    JsonResponse::from(res)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/provider/{{provider_instance_id}}/function/{{function_controller_type_id}}/invoke", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
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
    operation_id = "invoke-function",
)]
async fn route_invoke_function(
    State(ctx): State<BridgeService>,
    Path((provider_instance_id, function_controller_type_id)): Path<(String, String)>,
    Json(params): Json<InvokeFunctionParamsInner>,
) -> JsonResponse<InvokeFunctionResponse, CommonError> {
    // Get the envelope encryption key from the function instance's DEK
    // The invoke_function will get the DEK from the function instance, but we need the parent key
    // For now, we'll get it from the provider instance's resource server credential
    let provider_instance = match ctx
        .repository()
        .get_provider_instance_by_id(&provider_instance_id)
        .await
    {
        Ok(Some(instance)) => instance,
        Ok(None) => {
            return JsonResponse::from(Err(CommonError::Unknown(anyhow::anyhow!(
                "Provider instance not found: {provider_instance_id}"
            ))));
        }
        Err(e) => return JsonResponse::from(Err(e)),
    };
    
    let resource_server_cred = match ctx
        .repository()
        .get_resource_server_credential_by_id(
            &provider_instance.provider_instance.resource_server_credential_id,
        )
        .await
    {
        Ok(Some(cred)) => cred,
        Ok(None) => {
            return JsonResponse::from(Err(CommonError::Unknown(anyhow::anyhow!(
                "Resource server credential not found"
            ))));
        }
        Err(e) => return JsonResponse::from(Err(e)),
    };

    let envelope_key = match ctx
        .get_envelope_encryption_key_contents_from_dek_id(&resource_server_cred.data_encryption_key_id)
        .await
    {
        Ok(key) => key,
        Err(e) => return JsonResponse::from(Err(e)),
    };

    let res = invoke_function(
        ctx.repository(),
        &envelope_key,
        WithProviderInstanceId {
            provider_instance_id: provider_instance_id.clone(),
            inner: WithFunctionInstanceId {
                function_controller_type_id: function_controller_type_id.clone(),
                inner: params,
            },
        },
    )
    .await;
    JsonResponse::from(res)
}

#[derive(Serialize, Deserialize, Debug, ToSchema, IntoParams)]
#[into_params(style = Form, parameter_in = Query)]
struct ListProviderInstancesQuery {
    // TODO: utoipa doesnt support flattening yet https://github.com/juhaku/utoipa/pull/1426
    pub page_size: i64,
    pub next_page_token: Option<String>,
    pub status: Option<String>,
    pub provider_controller_type_id: Option<String>,
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/provider", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    params(
        ListProviderInstancesQuery
    ),
    responses(
        (status = 200, description = "List provider instances", body = ListProviderInstancesResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    operation_id = "list-provider-instances",
)]
async fn route_list_provider_instances(
    State(ctx): State<BridgeService>,
    Query(query): Query<ListProviderInstancesQuery>,
) -> JsonResponse<ListProviderInstancesResponse, CommonError> {
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
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/provider/grouped-by-function", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    params(
        ListProviderInstancesGroupedByFunctionParams
    ),
    responses(
        (status = 200, description = "List provider instances grouped by function", body = ListProviderInstancesGroupedByFunctionResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    operation_id = "list-provider-instances-grouped-by-function",
)]
async fn route_list_provider_instances_grouped_by_function(
    State(ctx): State<BridgeService>,
    Query(query): Query<ListProviderInstancesGroupedByFunctionParams>,
) -> JsonResponse<ListProviderInstancesGroupedByFunctionResponse, CommonError> {
    let res = list_provider_instances_grouped_by_function(ctx.repository(), query).await;
    JsonResponse::from(res)
}

#[derive(Serialize, Deserialize, Debug, ToSchema, IntoParams)]
#[into_params(style = Form, parameter_in = Query)]
struct ListFunctionInstancesQuery {
    // TODO: utoipa doesnt support flattening yet https://github.com/juhaku/utoipa/pull/1426
    pub page_size: i64,
    pub next_page_token: Option<String>,
    pub provider_instance_id: Option<String>,
}
#[utoipa::path(
    get,
    path = format!("{}/{}/{}/function-instances", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    params(
        ListFunctionInstancesQuery
    ),
    responses(
        (status = 200, description = "List function instances", body = ListFunctionInstancesResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    operation_id = "list-function-instances",
)]
async fn route_list_function_instances(
    State(ctx): State<BridgeService>,
    Query(query): Query<ListFunctionInstancesQuery>,
) -> JsonResponse<ListFunctionInstancesResponse, CommonError> {
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
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/function-instances/openapi.json", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    params(),
    responses(
        (status = 200, description = "Get function instances openapi spec", body = String),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    operation_id = "get-function-instances-openapi-spec",
)]
async fn route_get_function_instances_openapi_spec(
    State(ctx): State<BridgeService>,
) -> JsonResponse<OpenApi, CommonError> {
    let res = get_function_instances_openapi_spec(ctx.repository()).await;
    JsonResponse::from(res)
}

// ============================================================================
// Service
// ============================================================================

/// Get identifier (ARN or location) from EnvelopeEncryptionKeyContents
fn get_envelope_key_identifier(key: &EnvelopeEncryptionKeyContents) -> String {
    match key {
        EnvelopeEncryptionKeyContents::AwsKms { arn, .. } => arn.clone(),
        EnvelopeEncryptionKeyContents::Local { location, .. } => location.clone(),
    }
}

/// Get identifier (ARN or location) from EnvelopeEncryptionKeyId
fn get_envelope_key_id_identifier(key_id: &crate::logic::encryption::EnvelopeEncryptionKeyId) -> String {
    match key_id {
        crate::logic::encryption::EnvelopeEncryptionKeyId::AwsKms { arn, .. } => arn.clone(),
        crate::logic::encryption::EnvelopeEncryptionKeyId::Local { location } => location.clone(),
    }
}

pub struct BridgeServiceInner {
    pub repository: Repository,
    pub on_config_change_tx: OnConfigChangeTx,
    /// Map of envelope encryption key contents by identifier (ARN for AWS KMS, location for local)
    pub envelope_encryption_key_contents: ArcSwap<HashMap<String, EnvelopeEncryptionKeyContents>>,
    pub mcp_sessions: rmcp::transport::sse_server::TxStore,
    pub mcp_transport_tx:
        tokio::sync::mpsc::UnboundedSender<rmcp::transport::sse_server::SseServerTransport>,
    pub mcp_sse_ping_interval: Duration,
    pub sdk_client: Arc<
        tokio::sync::Mutex<
            Option<
                sdk_proto::soma_sdk_service_client::SomaSdkServiceClient<tonic::transport::Channel>,
            >,
        >,
    >,
}

impl BridgeServiceInner {
    pub fn new(
        repository: Repository,
        on_config_change_tx: OnConfigChangeTx,
        envelope_encryption_key_contents: EnvelopeEncryptionKeyContents,
        mcp_transport_tx: tokio::sync::mpsc::UnboundedSender<
            rmcp::transport::sse_server::SseServerTransport,
        >,
        mcp_sse_ping_interval: Duration,
        sdk_client: Arc<
            tokio::sync::Mutex<
                Option<
                    sdk_proto::soma_sdk_service_client::SomaSdkServiceClient<
                        tonic::transport::Channel,
                    >,
                >,
            >,
        >,
    ) -> Self {
        // Initialize map with the initial envelope encryption key
        let identifier = get_envelope_key_identifier(&envelope_encryption_key_contents);
        let mut map = HashMap::new();
        map.insert(identifier, envelope_encryption_key_contents);
        
        Self {
            repository,
            on_config_change_tx,
            envelope_encryption_key_contents: ArcSwap::from_pointee(map),
            mcp_sessions: Default::default(),
            mcp_transport_tx,
            mcp_sse_ping_interval,
            sdk_client,
        }
    }
}

#[derive(Clone)]
pub struct BridgeService(pub Arc<BridgeServiceInner>);

impl BridgeService {
    pub async fn new(
        repository: Repository,
        on_config_change_tx: OnConfigChangeTx,
        envelope_encryption_key_contents: EnvelopeEncryptionKeyContents,
        mcp_transport_tx: tokio::sync::mpsc::UnboundedSender<
            rmcp::transport::sse_server::SseServerTransport,
        >,
        mcp_sse_ping_interval: Duration,
        sdk_client: Arc<
            tokio::sync::Mutex<
                Option<
                    sdk_proto::soma_sdk_service_client::SomaSdkServiceClient<
                        tonic::transport::Channel,
                    >,
                >,
            >,
        >,
    ) -> Result<Self, CommonError> {
        // Initialize the service inner first to get the map
        let inner = BridgeServiceInner::new(
            repository,
            on_config_change_tx,
            envelope_encryption_key_contents,
            mcp_transport_tx,
            mcp_sse_ping_interval,
            sdk_client,
        );
        
        // Get the initial key for credential rotation check
        let initial_key_identifier = {
            let map = inner.envelope_encryption_key_contents.load();
            // Get the first (and only) key from the map
            map.keys().next().ok_or_else(|| {
                CommonError::Unknown(anyhow::anyhow!(
                    "No envelope encryption key found in map during initialization"
                ))
            })?.clone()
        };
        let initial_key = inner.envelope_encryption_key_contents.load()
            .get(&initial_key_identifier).ok_or_else(|| {
                CommonError::Unknown(anyhow::anyhow!(
                    "Initial envelope encryption key not found"
                ))
            })?.clone();
        
        // Run initial credential rotation check for expired and soon-to-expire credentials (30 min window)
        info!("Running initial credential rotation check...");
        process_credential_rotations_with_window(
            &inner.repository,
            &inner.on_config_change_tx,
            &initial_key,
            30,
        )
        .await?;
        info!("Initial credential rotation check complete");

        Ok(Self(Arc::new(inner)))
    }

    pub fn repository(&self) -> &Repository {
        &self.0.repository
    }

    pub fn on_config_change_tx(&self) -> &OnConfigChangeTx {
        &self.0.on_config_change_tx
    }

    /// Get envelope encryption key contents by identifier (ARN or location)
    /// Returns an error if the key is not found in the map
    pub fn get_envelope_encryption_key_contents(
        &self,
        identifier: &str,
    ) -> Result<EnvelopeEncryptionKeyContents, CommonError> {
        self.0
            .envelope_encryption_key_contents
            .load()
            .get(identifier)
            .cloned()
            .ok_or_else(|| {
                CommonError::Unknown(anyhow::anyhow!(
                    "Envelope encryption key not found for identifier: {identifier}"
                ))
            })
    }

    /// Get envelope encryption key contents by EnvelopeEncryptionKeyId
    pub fn get_envelope_encryption_key_contents_by_id(
        &self,
        key_id: &crate::logic::encryption::EnvelopeEncryptionKeyId,
    ) -> Result<EnvelopeEncryptionKeyContents, CommonError> {
        let identifier = get_envelope_key_id_identifier(key_id);
        self.get_envelope_encryption_key_contents(&identifier)
    }

    /// Get envelope encryption key contents from a data encryption key ID
    /// Looks up the DEK, extracts its envelope encryption key ID, and returns the key contents
    pub async fn get_envelope_encryption_key_contents_from_dek_id(
        &self,
        dek_id: &str,
    ) -> Result<EnvelopeEncryptionKeyContents, CommonError> {
        use crate::logic::encryption::DataEncryptionKeyRepositoryLike;
        let dek = DataEncryptionKeyRepositoryLike::get_data_encryption_key_by_id(
            self.repository(),
            dek_id,
        )
        .await?
        .ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!(
                "Data encryption key not found: {dek_id}"
            ))
        })?;
        self.get_envelope_encryption_key_contents_by_id(&dek.envelope_encryption_key_id)
    }

    /// Add or update envelope encryption key contents in the map
    pub fn add_envelope_encryption_key_contents(
        &self,
        key: EnvelopeEncryptionKeyContents,
    ) -> Result<(), CommonError> {
        let identifier = get_envelope_key_identifier(&key);
        // Copy-modify-store pattern for ArcSwap
        let current_map = self.0.envelope_encryption_key_contents.load();
        let mut new_map = HashMap::clone(&*current_map);
        new_map.insert(identifier, key);
        self.0.envelope_encryption_key_contents.store(Arc::new(new_map));
        Ok(())
    }

    /// Remove envelope encryption key contents from the map
    pub fn remove_envelope_encryption_key_contents(
        &self,
        identifier: &str,
    ) -> Result<(), CommonError> {
        // Copy-modify-store pattern for ArcSwap
        let current_map = self.0.envelope_encryption_key_contents.load();
        let mut new_map = HashMap::clone(&*current_map);
        new_map.remove(identifier);
        self.0.envelope_encryption_key_contents.store(Arc::new(new_map));
        Ok(())
    }

    pub fn mcp_transport_tx(
        &self,
    ) -> &tokio::sync::mpsc::UnboundedSender<rmcp::transport::sse_server::SseServerTransport> {
        &self.0.mcp_transport_tx
    }

    pub fn mcp_sse_ping_interval(&self) -> &Duration {
        &self.0.mcp_sse_ping_interval
    }

    pub fn mcp_sessions(&self) -> &rmcp::transport::sse_server::TxStore {
        &self.0.mcp_sessions
    }

    pub fn sdk_client(
        &self,
    ) -> &Arc<
        tokio::sync::Mutex<
            Option<
                sdk_proto::soma_sdk_service_client::SomaSdkServiceClient<tonic::transport::Channel>,
            >,
        >,
    > {
        &self.0.sdk_client
    }
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/mcp", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    params(
    ),
    tag = SERVICE_ROUTE_KEY,
    responses(
        (status = 200, description = "MCP server running"),
    ),
    operation_id = "listen-to-mcp-sse",
)]
pub async fn mcp_sse(
    State(ctx): State<BridgeService>,
    nested_path: Option<Extension<NestedPath>>,
    parts: Parts,
) -> impl IntoResponse {
    // taken from rmcp sse_handler source code.
    let session = session_id();
    tracing::info!(%session, ?parts, "sse connection");
    use tokio_stream::{StreamExt, wrappers::ReceiverStream};
    use tokio_util::sync::PollSender;
    let (from_client_tx, from_client_rx) = tokio::sync::mpsc::channel(64);
    let (to_client_tx, to_client_rx) = tokio::sync::mpsc::channel(64);
    let to_client_tx_clone = to_client_tx.clone();

    ctx.mcp_sessions()
        .write()
        .await
        .insert(session.clone(), from_client_tx);
    let session = session.clone();
    let stream = ReceiverStream::new(from_client_rx);
    let sink = PollSender::new(to_client_tx);
    let transport = SseServerTransport {
        stream,
        sink,
        session_id: session.clone(),
        // tx_store: app.txs.clone(),
        tx_store: ctx.mcp_sessions().clone(),
    };
    let transport_send_result = ctx.mcp_transport_tx().send(transport);
    if transport_send_result.is_err() {
        tracing::warn!("send transport out error");
        let mut response =
            Response::new("fail to send out transport, it seems server is closed".to_string());
        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
        return Err(response);
    }
    let nested_path = nested_path.as_deref().map(NestedPath::as_str).unwrap_or("");
    // let post_path = app.post_path.as_ref();
    // let post_path = app.mcp_post_path.clone();
    let post_path = parts.uri.path();
    // let ping_interval = app.sse_ping_interval;
    let ping_interval = ctx.mcp_sse_ping_interval();
    let stream = futures::stream::once(futures::future::ok(
        Event::default()
            .event("endpoint")
            .data(format!("{nested_path}{post_path}?sessionId={session}")),
    ))
    .chain(ReceiverStream::new(to_client_rx).map(|message| {
        match serde_json::to_string(&message) {
            Ok(bytes) => Ok(Event::default().event("message").data(&bytes)),
            Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
        }
    }));
    let tx_store = ctx.mcp_sessions().clone();
    tokio::spawn(async move {
        // Wait for connection closure
        to_client_tx_clone.closed().await;

        // Clean up session
        let session_id = session.clone();
        // let tx_store = app.txs.clone();
        let mut txs = tx_store.write().await;
        txs.remove(&session_id);
        tracing::debug!(%session_id, "Closed session and cleaned up resources");
    });

    Ok(Sse::new(stream).keep_alive(KeepAlive::new().interval(*ping_interval)))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(transparent)]
pub struct WrappedClientJsonRpcMessage(ClientJsonRpcMessage);

// TODO: implement ToSchema and PartialSchema
impl ToSchema for WrappedClientJsonRpcMessage {}

impl PartialSchema for WrappedClientJsonRpcMessage {
    // TODO: Implement schema generation for AgentCard
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::RefOr::T(utoipa::openapi::Schema::Object(
            utoipa::openapi::ObjectBuilder::new().build(),
        ))
    }
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/mcp", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tag = SERVICE_ROUTE_KEY,
    params(
    ),
    responses(
        (status = 200, description = "MCP server running"),
    ),
    operation_id = "trigger-mcp-message",
)]
pub async fn mcp_message(
    State(ctx): State<BridgeService>,
    Query(PostEventQuery { session_id }): Query<PostEventQuery>,
    parts: Parts,
    Json(message): Json<WrappedClientJsonRpcMessage>,
) -> impl IntoResponse {
    let mut message = message.0;
    tracing::debug!(session_id, ?parts, ?message, "new client message");
    let tx = {
        // let rg = app.txs.read().await;
        let rg = ctx.mcp_sessions().read().await;
        rg.get(session_id.as_str())
            .ok_or(StatusCode::NOT_FOUND)?
            .clone()
    };
    message.insert_extension(parts);

    if tx.send(message).await.is_err() {
        tracing::error!("send message error");
        return Err(StatusCode::GONE);
    }
    Ok(StatusCode::ACCEPTED)
}
