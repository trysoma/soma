use schemars::JsonSchema;
use sdk_proto::soma_sdk_service_client::SomaSdkServiceClient;
use serde::{Deserialize, Serialize};
use shared::{
    error::CommonError,
    primitives::{PaginatedResponse, PaginationRequest, WrappedChronoDateTime, WrappedUuidV4},
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Channel;
use tracing::warn;
use utoipa::ToSchema;

use crate::{
    logic::on_change_pubsub::{EnvironmentVariableChangeEvt, EnvironmentVariableChangeTx},
    repository::{
        CreateEnvironmentVariable, EnvironmentVariableRepositoryLike, UpdateEnvironmentVariable,
    },
};

// Domain model for EnvironmentVariable
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct EnvironmentVariable {
    pub id: WrappedUuidV4,
    pub key: String,
    pub value: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

// Request/Response types
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct CreateEnvironmentVariableRequest {
    pub key: String,
    pub value: String,
}

pub type CreateEnvironmentVariableResponse = EnvironmentVariable;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct UpdateEnvironmentVariableRequest {
    pub value: String,
}

pub type UpdateEnvironmentVariableResponse = EnvironmentVariable;

pub type GetEnvironmentVariableResponse = EnvironmentVariable;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct ListEnvironmentVariablesResponse {
    pub environment_variables: Vec<EnvironmentVariable>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct DeleteEnvironmentVariableResponse {
    pub success: bool,
}

// CRUD functions
/// Helper to incrementally sync a single environment variable to SDK
async fn sync_environment_variable_to_sdk_incremental(
    sdk_client: &Arc<Mutex<Option<SomaSdkServiceClient<Channel>>>>,
    key: String,
    value: String,
) {
    let mut sdk_client_guard = sdk_client.lock().await;

    if let Some(ref mut client) = *sdk_client_guard {
        use crate::logic::environment_variable_sync::sync_environment_variable_to_sdk;
        if let Err(e) = sync_environment_variable_to_sdk(client, key.clone(), value).await {
            warn!(
                "Failed to sync environment variable '{}' to SDK: {:?}",
                key, e
            );
        }
    }
}

/// Helper to unset an environment variable in SDK
async fn unset_environment_variable_in_sdk_incremental(
    sdk_client: &Arc<Mutex<Option<SomaSdkServiceClient<Channel>>>>,
    key: String,
) {
    let mut sdk_client_guard = sdk_client.lock().await;

    if let Some(ref mut client) = *sdk_client_guard {
        use crate::logic::environment_variable_sync::unset_environment_variable_in_sdk;
        if let Err(e) = unset_environment_variable_in_sdk(client, key.clone()).await {
            warn!(
                "Failed to unset environment variable '{}' in SDK: {:?}",
                key, e
            );
        }
    }
}

pub async fn create_environment_variable<R: EnvironmentVariableRepositoryLike>(
    on_change_tx: &EnvironmentVariableChangeTx,
    repository: &R,
    sdk_client: &Arc<Mutex<Option<SomaSdkServiceClient<Channel>>>>,
    request: CreateEnvironmentVariableRequest,
    publish_on_change_evt: bool,
) -> Result<CreateEnvironmentVariableResponse, CommonError> {
    let now = WrappedChronoDateTime::now();
    let id = WrappedUuidV4::new();

    let environment_variable = EnvironmentVariable {
        id: id.clone(),
        key: request.key.clone(),
        value: request.value.clone(),
        created_at: now,
        updated_at: now,
    };

    let create_params = CreateEnvironmentVariable {
        id,
        key: request.key,
        value: request.value,
        created_at: now,
        updated_at: now,
    };

    repository
        .create_environment_variable(&create_params)
        .await?;

    // Incrementally sync the new environment variable to SDK
    sync_environment_variable_to_sdk_incremental(
        sdk_client,
        environment_variable.key.clone(),
        environment_variable.value.clone(),
    )
    .await;

    if publish_on_change_evt {
        on_change_tx
            .send(EnvironmentVariableChangeEvt::Created(
                environment_variable.clone(),
            ))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!(
                    "Failed to send environment variable change event: {e}"
                ))
            })?;
    }

    Ok(environment_variable)
}

pub async fn update_environment_variable<R: EnvironmentVariableRepositoryLike>(
    on_change_tx: &EnvironmentVariableChangeTx,
    repository: &R,
    sdk_client: &Arc<Mutex<Option<SomaSdkServiceClient<Channel>>>>,
    id: WrappedUuidV4,
    request: UpdateEnvironmentVariableRequest,
    publish_on_change_evt: bool,
) -> Result<UpdateEnvironmentVariableResponse, CommonError> {
    // First verify the environment variable exists
    let existing = repository.get_environment_variable_by_id(&id).await?;
    let existing = existing.ok_or_else(|| CommonError::NotFound {
        msg: format!("Environment variable with id {id} not found"),
        lookup_id: id.to_string(),
        source: None,
    })?;

    let now = WrappedChronoDateTime::now();

    let update_params = UpdateEnvironmentVariable {
        id: id.clone(),
        value: request.value.clone(),
        updated_at: now,
    };

    repository
        .update_environment_variable(&update_params)
        .await?;

    // Incrementally sync the updated environment variable to SDK
    sync_environment_variable_to_sdk_incremental(
        sdk_client,
        existing.key.clone(),
        request.value.clone(),
    )
    .await;

    let updated_environment_variable = EnvironmentVariable {
        id,
        key: existing.key,
        value: request.value,
        created_at: existing.created_at,
        updated_at: now,
    };

    if publish_on_change_evt {
        on_change_tx
            .send(EnvironmentVariableChangeEvt::Updated(
                updated_environment_variable.clone(),
            ))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!(
                    "Failed to send environment variable change event: {e}"
                ))
            })?;
    }

    Ok(updated_environment_variable)
}

pub async fn delete_environment_variable<R: EnvironmentVariableRepositoryLike>(
    on_change_tx: &EnvironmentVariableChangeTx,
    repository: &R,
    sdk_client: &Arc<Mutex<Option<SomaSdkServiceClient<Channel>>>>,
    id: WrappedUuidV4,
    publish_on_change_evt: bool,
) -> Result<DeleteEnvironmentVariableResponse, CommonError> {
    // First verify the environment variable exists and get its key
    let existing = repository.get_environment_variable_by_id(&id).await?;
    let existing = existing.ok_or_else(|| CommonError::NotFound {
        msg: format!("Environment variable with id {id} not found"),
        lookup_id: id.to_string(),
        source: None,
    })?;

    repository.delete_environment_variable(&id).await?;

    // Unset the deleted environment variable in SDK
    unset_environment_variable_in_sdk_incremental(sdk_client, existing.key.clone()).await;

    if publish_on_change_evt {
        on_change_tx
            .send(EnvironmentVariableChangeEvt::Deleted {
                id: id.to_string(),
                key: existing.key,
            })
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!(
                    "Failed to send environment variable change event: {e}"
                ))
            })?;
    }

    Ok(DeleteEnvironmentVariableResponse { success: true })
}

pub async fn get_environment_variable_by_id<R: EnvironmentVariableRepositoryLike>(
    repository: &R,
    id: WrappedUuidV4,
) -> Result<GetEnvironmentVariableResponse, CommonError> {
    let environment_variable = repository.get_environment_variable_by_id(&id).await?;
    let environment_variable = environment_variable.ok_or_else(|| CommonError::NotFound {
        msg: format!("Environment variable with id {id} not found"),
        lookup_id: id.to_string(),
        source: None,
    })?;

    Ok(environment_variable)
}

pub async fn get_environment_variable_by_key<R: EnvironmentVariableRepositoryLike>(
    repository: &R,
    key: String,
) -> Result<GetEnvironmentVariableResponse, CommonError> {
    let environment_variable = repository.get_environment_variable_by_key(&key).await?;
    let environment_variable = environment_variable.ok_or_else(|| CommonError::NotFound {
        msg: format!("Environment variable with key {key} not found"),
        lookup_id: key.clone(),
        source: None,
    })?;

    Ok(environment_variable)
}

pub async fn list_environment_variables<R: EnvironmentVariableRepositoryLike>(
    repository: &R,
    pagination: PaginationRequest,
) -> Result<ListEnvironmentVariablesResponse, CommonError> {
    let paginated: PaginatedResponse<EnvironmentVariable> =
        repository.get_environment_variables(&pagination).await?;

    Ok(ListEnvironmentVariablesResponse {
        environment_variables: paginated.items,
        next_page_token: paginated.next_page_token,
    })
}

// Request type for importing environment variables (used by sync_yaml_to_api_on_start)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct ImportEnvironmentVariableRequest {
    pub key: String,
    pub value: String,
}

/// Import an environment variable (used for syncing from soma.yaml)
/// This does NOT publish change events since it's used for initial sync
pub async fn import_environment_variable<R: EnvironmentVariableRepositoryLike>(
    repository: &R,
    request: ImportEnvironmentVariableRequest,
) -> Result<EnvironmentVariable, CommonError> {
    let now = WrappedChronoDateTime::now();
    let id = WrappedUuidV4::new();

    let environment_variable = EnvironmentVariable {
        id: id.clone(),
        key: request.key.clone(),
        value: request.value.clone(),
        created_at: now,
        updated_at: now,
    };

    let create_params = CreateEnvironmentVariable {
        id,
        key: request.key,
        value: request.value,
        created_at: now,
        updated_at: now,
    };

    repository
        .create_environment_variable(&create_params)
        .await?;

    Ok(environment_variable)
}

#[cfg(all(test, feature = "unit_test"))]
mod unit_test {
    use super::*;
    use crate::repository::Repository;
    use shared::primitives::SqlMigrationLoader;

    fn create_test_sdk_client() -> Arc<Mutex<Option<SomaSdkServiceClient<Channel>>>> {
        Arc::new(Mutex::new(None::<SomaSdkServiceClient<Channel>>))
    }

    async fn setup_test_repository() -> Repository {
        let (_db, conn) = shared::test_utils::repository::setup_in_memory_database(vec![
            <Repository as SqlMigrationLoader>::load_sql_migrations(),
        ])
        .await
        .expect("Failed to setup test database");
        Repository::new(conn)
    }

    #[tokio::test]
    async fn test_create_environment_variable() {
        let repository = setup_test_repository().await;
        let (on_change_tx, mut on_change_rx) = tokio::sync::broadcast::channel(10);

        let request = CreateEnvironmentVariableRequest {
            key: "MY_ENV_VAR".to_string(),
            value: "my-value".to_string(),
        };

        let sdk_client = create_test_sdk_client();
        let result = create_environment_variable(
            &on_change_tx,
            &repository,
            &sdk_client,
            request.clone(),
            true,
        )
        .await;

        assert!(result.is_ok());
        let env_var = result.unwrap();
        assert_eq!(env_var.key, "MY_ENV_VAR");
        assert_eq!(env_var.value, "my-value");

        // Check event was published
        let event = on_change_rx.try_recv();
        assert!(event.is_ok());
        match event.unwrap() {
            EnvironmentVariableChangeEvt::Created(e) => {
                assert_eq!(e.key, "MY_ENV_VAR");
            }
            _ => panic!("Expected Created event"),
        }
    }

    #[tokio::test]
    async fn test_update_environment_variable() {
        let repository = setup_test_repository().await;
        let (on_change_tx, mut on_change_rx) = tokio::sync::broadcast::channel(10);
        let sdk_client = create_test_sdk_client();

        // Create an environment variable first
        let create_request = CreateEnvironmentVariableRequest {
            key: "MY_ENV_VAR".to_string(),
            value: "original-value".to_string(),
        };

        let created = create_environment_variable(
            &on_change_tx,
            &repository,
            &sdk_client,
            create_request,
            false,
        )
        .await
        .unwrap();

        // Update the environment variable
        let update_request = UpdateEnvironmentVariableRequest {
            value: "updated-value".to_string(),
        };

        let result = update_environment_variable(
            &on_change_tx,
            &repository,
            &sdk_client,
            created.id.clone(),
            update_request,
            true,
        )
        .await;

        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.key, "MY_ENV_VAR");
        assert_eq!(updated.value, "updated-value");

        // Check event was published
        let event = on_change_rx.try_recv();
        assert!(event.is_ok());
        match event.unwrap() {
            EnvironmentVariableChangeEvt::Updated(e) => {
                assert_eq!(e.key, "MY_ENV_VAR");
            }
            _ => panic!("Expected Updated event"),
        }
    }

    #[tokio::test]
    async fn test_delete_environment_variable() {
        let repository = setup_test_repository().await;
        let (on_change_tx, mut on_change_rx) = tokio::sync::broadcast::channel(10);
        let sdk_client = create_test_sdk_client();

        // Create an environment variable first
        let create_request = CreateEnvironmentVariableRequest {
            key: "MY_ENV_VAR".to_string(),
            value: "my-value".to_string(),
        };

        let created = create_environment_variable(
            &on_change_tx,
            &repository,
            &sdk_client,
            create_request,
            false,
        )
        .await
        .unwrap();

        // Delete the environment variable
        let result = delete_environment_variable(
            &on_change_tx,
            &repository,
            &sdk_client,
            created.id.clone(),
            true,
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.success);

        // Check event was published
        let event = on_change_rx.try_recv();
        assert!(event.is_ok());
        match event.unwrap() {
            EnvironmentVariableChangeEvt::Deleted { id, key } => {
                assert_eq!(id, created.id.to_string());
                assert_eq!(key, "MY_ENV_VAR");
            }
            _ => panic!("Expected Deleted event"),
        }

        // Verify it's deleted
        let get_result = get_environment_variable_by_id(&repository, created.id).await;
        assert!(get_result.is_err());
    }

    #[tokio::test]
    async fn test_get_environment_variable_by_id() {
        let repository = setup_test_repository().await;
        let (on_change_tx, _on_change_rx) = tokio::sync::broadcast::channel(10);

        // Create an environment variable first
        let create_request = CreateEnvironmentVariableRequest {
            key: "MY_ENV_VAR".to_string(),
            value: "my-value".to_string(),
        };

        let sdk_client = create_test_sdk_client();
        let created = create_environment_variable(
            &on_change_tx,
            &repository,
            &sdk_client,
            create_request,
            false,
        )
        .await
        .unwrap();

        // Get by id
        let result = get_environment_variable_by_id(&repository, created.id.clone()).await;

        assert!(result.is_ok());
        let env_var = result.unwrap();
        assert_eq!(env_var.key, "MY_ENV_VAR");
        assert_eq!(env_var.id, created.id);
    }

    #[tokio::test]
    async fn test_get_environment_variable_by_key() {
        let repository = setup_test_repository().await;
        let (on_change_tx, _on_change_rx) = tokio::sync::broadcast::channel(10);

        // Create an environment variable first
        let create_request = CreateEnvironmentVariableRequest {
            key: "MY_ENV_VAR".to_string(),
            value: "my-value".to_string(),
        };

        let sdk_client = create_test_sdk_client();
        let created = create_environment_variable(
            &on_change_tx,
            &repository,
            &sdk_client,
            create_request,
            false,
        )
        .await
        .unwrap();

        // Get by key
        let result = get_environment_variable_by_key(&repository, "MY_ENV_VAR".to_string()).await;

        assert!(result.is_ok());
        let env_var = result.unwrap();
        assert_eq!(env_var.id, created.id);
        assert_eq!(env_var.key, "MY_ENV_VAR");
    }

    #[tokio::test]
    async fn test_list_environment_variables() {
        let repository = setup_test_repository().await;
        let (on_change_tx, _on_change_rx) = tokio::sync::broadcast::channel(10);

        // Create multiple environment variables
        for i in 0..3 {
            let create_request = CreateEnvironmentVariableRequest {
                key: format!("ENV_VAR_{i}"),
                value: format!("value-{i}"),
            };

            let sdk_client = create_test_sdk_client();
            create_environment_variable(
                &on_change_tx,
                &repository,
                &sdk_client,
                create_request,
                false,
            )
            .await
            .unwrap();
        }

        // List environment variables
        let result = list_environment_variables(
            &repository,
            PaginationRequest {
                page_size: 10,
                next_page_token: None,
            },
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.environment_variables.len(), 3);
    }

    #[tokio::test]
    async fn test_get_environment_variable_not_found() {
        let repository = setup_test_repository().await;

        let result = get_environment_variable_by_id(&repository, WrappedUuidV4::new()).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            CommonError::NotFound { .. } => {}
            e => panic!("Expected NotFound error, got: {e:?}"),
        }
    }

    #[tokio::test]
    async fn test_create_environment_variable_no_publish() {
        let repository = setup_test_repository().await;
        let (on_change_tx, mut on_change_rx) = tokio::sync::broadcast::channel(10);
        let sdk_client = create_test_sdk_client();

        let request = CreateEnvironmentVariableRequest {
            key: "MY_ENV_VAR".to_string(),
            value: "my-value".to_string(),
        };

        let result = create_environment_variable(
            &on_change_tx,
            &repository,
            &sdk_client,
            request,
            false, // Don't publish
        )
        .await;

        assert!(result.is_ok());

        // Should be no event
        let event = on_change_rx.try_recv();
        assert!(event.is_err());
    }

    #[tokio::test]
    async fn test_import_environment_variable() {
        let repository = setup_test_repository().await;

        let request = ImportEnvironmentVariableRequest {
            key: "IMPORTED_VAR".to_string(),
            value: "imported-value".to_string(),
        };

        let result = import_environment_variable(&repository, request).await;

        assert!(result.is_ok());
        let env_var = result.unwrap();
        assert_eq!(env_var.key, "IMPORTED_VAR");
        assert_eq!(env_var.value, "imported-value");

        // Verify it was actually saved
        let fetched =
            get_environment_variable_by_key(&repository, "IMPORTED_VAR".to_string()).await;
        assert!(fetched.is_ok());
        assert_eq!(fetched.unwrap().id, env_var.id);
    }

    #[tokio::test]
    async fn test_update_environment_variable_not_found() {
        let repository = setup_test_repository().await;
        let (on_change_tx, _on_change_rx) = tokio::sync::broadcast::channel(10);

        let update_request = UpdateEnvironmentVariableRequest {
            value: "updated-value".to_string(),
        };

        let sdk_client = create_test_sdk_client();
        let result = update_environment_variable(
            &on_change_tx,
            &repository,
            &sdk_client,
            WrappedUuidV4::new(),
            update_request,
            true,
        )
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            CommonError::NotFound { .. } => {}
            e => panic!("Expected NotFound error, got: {e:?}"),
        }
    }

    #[tokio::test]
    async fn test_delete_environment_variable_not_found() {
        let repository = setup_test_repository().await;
        let (on_change_tx, _on_change_rx) = tokio::sync::broadcast::channel(10);

        let sdk_client = create_test_sdk_client();
        let result = delete_environment_variable(
            &on_change_tx,
            &repository,
            &sdk_client,
            WrappedUuidV4::new(),
            true,
        )
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            CommonError::NotFound { .. } => {}
            e => panic!("Expected NotFound error, got: {e:?}"),
        }
    }

    #[tokio::test]
    async fn test_get_environment_variable_by_key_not_found() {
        let repository = setup_test_repository().await;

        let result =
            get_environment_variable_by_key(&repository, "NON_EXISTENT_KEY".to_string()).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            CommonError::NotFound { .. } => {}
            e => panic!("Expected NotFound error, got: {e:?}"),
        }
    }

    #[tokio::test]
    async fn test_list_environment_variables_pagination() {
        let repository = setup_test_repository().await;
        let (on_change_tx, _on_change_rx) = tokio::sync::broadcast::channel(10);

        // Create 5 environment variables
        for i in 0..5 {
            let create_request = CreateEnvironmentVariableRequest {
                key: format!("ENV_VAR_{i}"),
                value: format!("value-{i}"),
            };

            let sdk_client = create_test_sdk_client();
            create_environment_variable(
                &on_change_tx,
                &repository,
                &sdk_client,
                create_request,
                false,
            )
            .await
            .unwrap();
        }

        // First page
        let result = list_environment_variables(
            &repository,
            PaginationRequest {
                page_size: 3,
                next_page_token: None,
            },
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.environment_variables.len(), 3);
        assert!(response.next_page_token.is_some());

        // Second page
        let result = list_environment_variables(
            &repository,
            PaginationRequest {
                page_size: 3,
                next_page_token: response.next_page_token,
            },
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.environment_variables.len(), 2);
    }
}
