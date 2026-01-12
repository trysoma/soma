{change_wanted}

in crates: {crate}

When making changes, adding or removing an api endpoint, first:

1. edit the routes in crates/$crate/src/router.rs or the relevant file in the router folder 
2. For the route body parameters, body response, always define the types in crates/$crate/src/logic.rs or the relevant file in the logic folder 
3. Never define business logic in the actual router method body, always invoke a logic_ function in the body, passing in any dependencies (repository, etc.) as well as parameters for the logic function to execute
4. Always return JsonResponse<$LogicReturnType, CommonError> or impl IntoResponse (if a non-json response like a redirect)
5. Always add trace level logs around every API endpoint, logic function and repository method. To both the start and end of each method.

Note:

Always define an operation ID, summary, description

For example:

`crates/soma-api-server/src/router/secret.rs`:

```rust
#[utoipa::path(
    post,
    path = format!("{}/{}/{}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = CreateSecretRequest,
    responses(
        (status = 200, description = "Create a secret", body = CreateSecretResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Create secret",
    description = "Create a new encrypted secret with the specified key and value",
    operation_id = "create-secret",
)]
async fn route_create_secret(
    State(ctx): State<Arc<SecretService>>,
    Json(request): Json<CreateSecretRequest>,
) -> JsonResponse<CreateSecretResponse, CommonError> {
    let res = create_secret(
        &ctx.on_change_tx,
        &ctx.repository,
        &ctx.crypto_cache,
        &ctx.sdk_client,
        request,
        true,
    )
    .await;
    JsonResponse::from(res)
}
```

`crates/soma-api-server/src/logic/secret.rs`:

```rust

// Domain model for Secret
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct Secret {
    pub id: WrappedUuidV4,
    pub key: String,
    pub encrypted_secret: String,
    pub dek_alias: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

// Request/Response types
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct CreateSecretRequest {
    pub key: String,
    pub raw_value: String,
    pub dek_alias: String,
}

pub type CreateSecretResponse = Secret;



pub async fn create_secret<R: SecretRepositoryLike>(
    on_change_tx: &SecretChangeTx,
    repository: &R,
    crypto_cache: &CryptoCache,
    sdk_client: &Arc<Mutex<Option<SomaSdkServiceClient<Channel>>>>,
    request: CreateSecretRequest,
    publish_on_change_evt: bool,
) -> Result<CreateSecretResponse, CommonError> {
    // Get encryption service for the DEK alias
    let encryption_service = crypto_cache
        .get_encryption_service(&request.dek_alias)
        .await?;

    // Encrypt the raw value
    let encrypted_secret = encryption_service.encrypt_data(request.raw_value).await?;

    let now = WrappedChronoDateTime::now();
    let id = WrappedUuidV4::new();

    let secret = Secret {
        id: id.clone(),
        key: request.key.clone(),
        encrypted_secret: encrypted_secret.0.clone(),
        dek_alias: request.dek_alias.clone(),
        created_at: now,
        updated_at: now,
    };

    let create_params = CreateSecret {
        id,
        key: request.key,
        encrypted_secret: encrypted_secret.0,
        dek_alias: request.dek_alias,
        created_at: now,
        updated_at: now,
    };

    repository.create_secret(&create_params).await?;

    // Incrementally sync the new secret to SDK
    sync_secret_to_sdk_incremental(
        sdk_client,
        crypto_cache,
        secret.key.clone(),
        secret.encrypted_secret.clone(),
        secret.dek_alias.clone(),
    )
    .await;

    if publish_on_change_evt {
        on_change_tx
            .send(SecretChangeEvt::Created(secret.clone()))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send secret change event: {e}"))
            })?;
    }

    Ok(secret)
}


```
