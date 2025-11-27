use crate::logic::dek::DataEncryptionKey;
use crate::logic::dek::{
    CreateDataEncryptionKeyResponse, CreateDekInnerParams, CreateDekParams, ImportDekParams,
    ImportDekParamsInner, ImportDekResponse, ListDekParams, ListDekResponse,
    create_data_encryption_key, import_data_encryption_key, list_data_encryption_keys,
};
use crate::logic::dek_alias::{
    CreateAliasInnerParams, CreateAliasParams, CreateAliasResponse, UpdateAliasParams,
    UpdateAliasResponse, create_alias, delete_alias, get_by_alias_or_id, update_alias,
};
use crate::logic::envelope::{
    CreateEnvelopeEncryptionKeyParams, CreateEnvelopeEncryptionKeyResponse,
    ListEnvelopeEncryptionKeysResponse, migrate_all_data_encryption_keys_for_envelope,
    migrate_data_encryption_key_for_envelope,
};
use crate::logic::{
    EncryptionKeyEventSender, create_envelope_encryption_key, list_envelope_encryption_keys,
};
use crate::repository::Repository;
use axum::extract::{Json, Path, Query, State};
use serde::{Deserialize, Serialize};
use shared::{
    adapters::openapi::{API_VERSION_TAG, JsonResponse},
    error::CommonError,
    primitives::PaginationRequest,
};
use std::path::PathBuf;
use std::sync::Arc;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

pub const PATH_PREFIX: &str = "/api";
pub const API_VERSION_1: &str = "v1";
pub const SERVICE_ROUTE_KEY: &str = "encryption";

pub fn create_router() -> OpenApiRouter<EncryptionService> {
    OpenApiRouter::new()
        // Envelope encryption key endpoints
        .routes(routes!(route_create_envelope_encryption_key))
        .routes(routes!(route_list_envelope_encryption_keys))
        // Data encryption key endpoints
        .routes(routes!(route_create_data_encryption_key))
        .routes(routes!(route_import_data_encryption_key))
        .routes(routes!(route_list_data_encryption_keys))
        .routes(routes!(route_migrate_data_encryption_key))
        .routes(routes!(route_migrate_all_data_encryption_keys))
        // Data encryption key alias endpoints
        .routes(routes!(route_create_dek_alias))
        .routes(routes!(route_get_dek_by_alias_or_id))
        .routes(routes!(route_update_dek_alias))
        .routes(routes!(route_delete_dek_alias))
}

// ============================================================================
// Envelope encryption key endpoints
// ============================================================================

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/envelope", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = CreateEnvelopeEncryptionKeyParams,
    responses(
        (status = 200, description = "Create envelope encryption key", body = CreateEnvelopeEncryptionKeyResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
        (status = 502, description = "Bad Gateway", body = CommonError),
    ),
    summary = "Create envelope key",
    description = "Create a new envelope encryption key (master key) for encrypting data encryption keys",
    operation_id = "create-envelope-encryption-key",
)]
async fn route_create_envelope_encryption_key(
    State(ctx): State<EncryptionService>,
    Json(params): Json<CreateEnvelopeEncryptionKeyParams>,
) -> JsonResponse<CreateEnvelopeEncryptionKeyResponse, CommonError> {
    let res = create_envelope_encryption_key(
        ctx.local_envelope_encryption_key_path(),
        ctx.on_change_tx(),
        ctx.repository(),
        params,
        true,
    )
    .await;
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/envelope", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        PaginationRequest
    ),
    responses(
        (status = 200, description = "List envelope encryption keys", body = ListEnvelopeEncryptionKeysResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
        (status = 502, description = "Bad Gateway", body = CommonError),
    ),
    summary = "List envelope keys",
    description = "List all envelope encryption keys (master keys) with pagination",
    operation_id = "list-envelope-encryption-keys",
)]
async fn route_list_envelope_encryption_keys(
    State(ctx): State<EncryptionService>,
    Query(pagination): Query<PaginationRequest>,
) -> JsonResponse<ListEnvelopeEncryptionKeysResponse, CommonError> {
    let res = list_envelope_encryption_keys(ctx.repository(), pagination).await;
    JsonResponse::from(res)
}

// ============================================================================
// Data encryption key endpoints
// ============================================================================

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateDataEncryptionKeyParamsRoute {
    pub id: Option<String>,
    pub encrypted_dek: Option<String>,
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/envelope/{{envelope_id}}/dek", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = CreateDataEncryptionKeyParamsRoute,
    params(
        ("envelope_id" = String, Path, description = "Envelope encryption key ID"),
    ),
    responses(
        (status = 200, description = "Create data encryption key", body = CreateDataEncryptionKeyResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
        (status = 502, description = "Bad Gateway", body = CommonError),
    ),
    summary = "Create data key",
    description = "Create a new data encryption key (DEK) encrypted with the specified envelope encryption key",
    operation_id = "create-data-encryption-key",
)]
async fn route_create_data_encryption_key(
    State(ctx): State<EncryptionService>,
    Path(envelope_id): Path<String>,
    Json(params): Json<CreateDataEncryptionKeyParamsRoute>,
) -> JsonResponse<CreateDataEncryptionKeyResponse, CommonError> {
    let create_params = CreateDekParams {
        envelope_encryption_key_id: envelope_id,
        inner: CreateDekInnerParams {
            id: params.id,
            encrypted_dek: params.encrypted_dek,
        },
    };
    let res = create_data_encryption_key(
        ctx.on_change_tx(),
        ctx.repository(),
        create_params,
        ctx.local_envelope_encryption_key_path(),
        true,
    )
    .await;
    JsonResponse::from(res)
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ImportDataEncryptionKeyParamsRoute {
    pub id: Option<String>,
    pub encrypted_data_encryption_key: String,
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/envelope/{{envelope_id}}/dek/import", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = ImportDataEncryptionKeyParamsRoute,
    params(
        ("envelope_id" = String, Path, description = "Envelope encryption key ID"),
    ),
    responses(
        (status = 200, description = "Import data encryption key", body = ImportDekResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
        (status = 502, description = "Bad Gateway", body = CommonError),
    ),
    summary = "Import data key",
    description = "Import an existing pre-encrypted data encryption key into the system",
    operation_id = "import-data-encryption-key",
)]
async fn route_import_data_encryption_key(
    State(ctx): State<EncryptionService>,
    Path(envelope_id): Path<String>,
    Json(params): Json<ImportDataEncryptionKeyParamsRoute>,
) -> JsonResponse<ImportDekResponse, CommonError> {
    let import_params = ImportDekParams {
        envelope_encryption_key_id: envelope_id,
        inner: ImportDekParamsInner {
            id: params.id,
            encrypted_data_encryption_key: crate::logic::dek::EncryptedDataEncryptionKey(
                params.encrypted_data_encryption_key,
            ),
        },
    };
    let res = import_data_encryption_key(
        ctx.on_change_tx(),
        ctx.repository(),
        import_params,
        ctx.local_envelope_encryption_key_path(),
        true,
    )
    .await;
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/envelope/{{envelope_id}}/dek", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("envelope_id" = String, Path, description = "Envelope encryption key ID"),
        PaginationRequest
    ),
    responses(
        (status = 200, description = "List data encryption keys", body = ListDekResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
        (status = 502, description = "Bad Gateway", body = CommonError),
    ),
    summary = "List data keys",
    description = "List all data encryption keys encrypted with the specified envelope encryption key",
    operation_id = "list-data-encryption-keys-by-envelope",
)]
async fn route_list_data_encryption_keys(
    State(ctx): State<EncryptionService>,
    Path(envelope_id): Path<String>,
    Query(pagination): Query<PaginationRequest>,
) -> JsonResponse<ListDekResponse, CommonError> {
    let list_params = ListDekParams {
        envelope_encryption_key_id: envelope_id,
        inner: pagination,
    };
    let res = list_data_encryption_keys(ctx.repository(), list_params).await;
    JsonResponse::from(res)
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct MigrateDataEncryptionKeyParamsRoute {
    pub to_envelope_encryption_key_id: String,
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/envelope/{{envelope_id}}/dek/{{dek_id}}/migrate", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = MigrateDataEncryptionKeyParamsRoute,
    params(
        ("envelope_id" = String, Path, description = "Envelope encryption key ID"),
        ("dek_id" = String, Path, description = "Data encryption key ID"),
    ),
    responses(
        (status = 200, description = "Migrate data encryption key"),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
        (status = 502, description = "Bad Gateway", body = CommonError),
    ),
    summary = "Migrate data key",
    description = "Migrate a data encryption key to be encrypted with a different envelope encryption key",
    operation_id = "migrate-data-encryption-key",
)]
async fn route_migrate_data_encryption_key(
    State(ctx): State<EncryptionService>,
    Path((envelope_id, dek_id)): Path<(String, String)>,
    Json(params): Json<MigrateDataEncryptionKeyParamsRoute>,
) -> JsonResponse<(), CommonError> {
    let res = migrate_data_encryption_key_for_envelope(
        ctx.local_envelope_encryption_key_path(),
        &envelope_id,
        &dek_id,
        &params.to_envelope_encryption_key_id,
        ctx.on_change_tx(),
        ctx.repository(),
        ctx.cache(),
        true,
    )
    .await;
    JsonResponse::from(res)
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct MigrateAllDataEncryptionKeysParamsRoute {
    pub to_envelope_encryption_key_id: String,
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/envelope/{{envelope_id}}/migrate", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = MigrateAllDataEncryptionKeysParamsRoute,
    params(
        ("envelope_id" = String, Path, description = "Envelope encryption key ID"),
    ),
    responses(
        (status = 200, description = "Migrate all data encryption keys"),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
        (status = 502, description = "Bad Gateway", body = CommonError),
    ),
    summary = "Migrate all data keys",
    description = "Migrate all data encryption keys encrypted with the specified envelope key to a new envelope key",
    operation_id = "migrate-all-data-encryption-keys",
)]
async fn route_migrate_all_data_encryption_keys(
    State(ctx): State<EncryptionService>,
    Path(envelope_id): Path<String>,
    Json(params): Json<MigrateAllDataEncryptionKeysParamsRoute>,
) -> JsonResponse<(), CommonError> {
    let res = migrate_all_data_encryption_keys_for_envelope(
        ctx.local_envelope_encryption_key_path(),
        &envelope_id,
        &params.to_envelope_encryption_key_id,
        ctx.on_change_tx(),
        ctx.repository(),
        ctx.cache(),
        true,
    )
    .await;
    JsonResponse::from(res)
}

// ============================================================================
// Data encryption key alias endpoints
// ============================================================================

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateDekAliasRequest {
    pub dek_id: String,
    pub alias: String,
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/dek/alias", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = CreateDekAliasRequest,
    responses(
        (status = 200, description = "Create DEK alias", body = CreateAliasResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
        (status = 502, description = "Bad Gateway", body = CommonError),
    ),
    summary = "Create DEK alias",
    description = "Create an alias for a data encryption key to enable lookup by friendly name",
    operation_id = "create-dek-alias",
)]
async fn route_create_dek_alias(
    State(ctx): State<EncryptionService>,
    Json(req): Json<CreateDekAliasRequest>,
) -> JsonResponse<CreateAliasResponse, CommonError> {
    let params = CreateAliasParams {
        dek_id: req.dek_id,
        inner: CreateAliasInnerParams { alias: req.alias },
    };
    let res = create_alias(ctx.on_change_tx(), ctx.repository(), ctx.cache(), params).await;
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/dek/alias/{{alias}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("alias" = String, Path, description = "DEK alias or ID"),
    ),
    responses(
        (status = 200, description = "Get DEK by alias or ID", body = DataEncryptionKey),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
        (status = 502, description = "Bad Gateway", body = CommonError),
    ),
    summary = "Get DEK by alias",
    description = "Retrieve a data encryption key by its alias or ID",
    operation_id = "get-dek-by-alias-or-id",
)]
async fn route_get_dek_by_alias_or_id(
    State(ctx): State<EncryptionService>,
    Path(alias_or_id): Path<String>,
) -> JsonResponse<DataEncryptionKey, CommonError> {
    let res = get_by_alias_or_id(ctx.repository(), &alias_or_id).await;
    JsonResponse::from(res)
}

#[utoipa::path(
    put,
    path = format!("{}/{}/{}/dek/alias/{{alias}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = UpdateAliasParams,
    params(
        ("alias" = String, Path, description = "DEK alias"),
    ),
    responses(
        (status = 200, description = "Update DEK alias", body = UpdateAliasResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
        (status = 502, description = "Bad Gateway", body = CommonError),
    ),
    summary = "Update DEK alias",
    description = "Update the alias for a data encryption key",
    operation_id = "update-dek-alias",
)]
async fn route_update_dek_alias(
    State(ctx): State<EncryptionService>,
    Path(alias): Path<String>,
    Json(params): Json<UpdateAliasParams>,
) -> JsonResponse<UpdateAliasResponse, CommonError> {
    let res = update_alias(
        ctx.on_change_tx(),
        ctx.repository(),
        ctx.cache(),
        alias,
        params,
    )
    .await;
    JsonResponse::from(res)
}

#[utoipa::path(
    delete,
    path = format!("{}/{}/{}/dek/alias/{{alias}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("alias" = String, Path, description = "DEK alias"),
    ),
    responses(
        (status = 200, description = "Delete DEK alias"),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
        (status = 502, description = "Bad Gateway", body = CommonError),
    ),
    summary = "Delete DEK alias",
    description = "Delete an alias for a data encryption key",
    operation_id = "delete-dek-alias",
)]
async fn route_delete_dek_alias(
    State(ctx): State<EncryptionService>,
    Path(alias): Path<String>,
) -> JsonResponse<(), CommonError> {
    let res = delete_alias(ctx.on_change_tx(), ctx.repository(), ctx.cache(), alias).await;
    JsonResponse::from(res)
}

// ============================================================================
// Service
// ============================================================================

#[derive(Clone)]
pub struct EncryptionServiceInner {
    pub repository: Repository,
    pub on_change_tx: EncryptionKeyEventSender,
    pub cache: Arc<crate::logic::crypto_services::CryptoCache>,
    pub local_envelope_encryption_key_path: PathBuf,
}

#[derive(Clone)]
pub struct EncryptionService(pub Arc<EncryptionServiceInner>);

impl EncryptionService {
    pub fn new(
        repository: Repository,
        on_change_tx: EncryptionKeyEventSender,
        cache: crate::logic::crypto_services::CryptoCache,
        local_envelope_encryption_key_path: PathBuf,
    ) -> Self {
        Self(Arc::new(EncryptionServiceInner {
            repository,
            on_change_tx,
            cache: Arc::new(cache),
            local_envelope_encryption_key_path,
        }))
    }

    pub fn repository(&self) -> &Repository {
        &self.0.repository
    }

    pub fn on_change_tx(&self) -> &EncryptionKeyEventSender {
        &self.0.on_change_tx
    }

    pub fn cache(&self) -> &crate::logic::crypto_services::CryptoCache {
        self.0.cache.as_ref()
    }

    pub fn local_envelope_encryption_key_path(&self) -> &PathBuf {
        &self.0.local_envelope_encryption_key_path
    }
}
