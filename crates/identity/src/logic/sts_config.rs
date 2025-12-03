use serde::{Deserialize, Serialize};
use shared::error::CommonError;
use shared::primitives::{PaginationRequest, WrappedChronoDateTime};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::logic::{OnConfigChangeEvt, OnConfigChangeTx, StsConfigCreatedInfo};
use crate::repository::{CreateStsConfiguration, StsConfiguration, UserRepositoryLike};

/// Valid STS configuration types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum StsConfigType {
    JwtTemplate,
    Dev,
}

impl StsConfigType {
    pub fn as_str(&self) -> &'static str {
        match self {
            StsConfigType::JwtTemplate => "jwt_template",
            StsConfigType::Dev => "dev",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "jwt_template" => Some(StsConfigType::JwtTemplate),
            "dev" => Some(StsConfigType::Dev),
            _ => None,
        }
    }
}

/// Parameters for creating an STS configuration
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateStsConfigParams {
    /// Optional ID (will be generated if not provided)
    pub id: Option<String>,
    /// The configuration type
    #[serde(rename = "type")]
    pub config_type: String,
    /// The configuration value (JSON)
    pub value: Option<String>,
}

/// Response from creating an STS configuration
#[derive(Debug, Serialize, ToSchema)]
pub struct CreateStsConfigResponse {
    /// The STS configuration ID
    pub id: String,
    /// The configuration type
    #[serde(rename = "type")]
    pub config_type: String,
    /// The configuration value (JSON)
    pub value: Option<String>,
}

/// Parameters for deleting an STS configuration
#[derive(Debug, Deserialize, ToSchema)]
pub struct DeleteStsConfigParams {
    /// The ID of the STS configuration to delete
    pub id: String,
}

/// Response from deleting an STS configuration
#[derive(Debug, Serialize, ToSchema)]
pub struct DeleteStsConfigResponse {
    /// Whether the deletion was successful
    pub success: bool,
}

/// Parameters for listing STS configurations
#[derive(Debug)]
pub struct ListStsConfigParams {
    pub pagination: PaginationRequest,
    pub config_type: Option<String>,
}

/// Response from listing STS configurations
#[derive(Debug, Serialize, ToSchema)]
pub struct ListStsConfigResponse {
    pub items: Vec<StsConfiguration>,
    pub next_page_token: Option<String>,
}

/// Parameters for getting an STS configuration
#[derive(Debug, Deserialize, ToSchema)]
pub struct GetStsConfigParams {
    /// The ID of the STS configuration to get
    pub id: String,
}

/// Create a new STS configuration
///
/// This function:
/// 1. Validates the configuration type
/// 2. Generates an ID if not provided
/// 3. Stores the configuration in the repository
/// 4. Optionally broadcasts a config change event
pub async fn create_sts_config<R: UserRepositoryLike>(
    repository: &R,
    on_config_change_tx: &OnConfigChangeTx,
    params: CreateStsConfigParams,
    publish_on_change_evt: bool,
) -> Result<CreateStsConfigResponse, CommonError> {
    // Validate config type
    let config_type = StsConfigType::from_str(&params.config_type).ok_or_else(|| {
        CommonError::InvalidRequest {
            msg: format!(
                "Invalid config type '{}'. Valid types are: jwt_template, dev",
                params.config_type
            ),
            source: None,
        }
    })?;

    // Generate ID if not provided
    let id = params.id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let now = WrappedChronoDateTime::now();

    // Check if config with this ID already exists
    if repository.get_sts_configuration_by_id(&id).await?.is_some() {
        return Err(CommonError::InvalidRequest {
            msg: format!("STS configuration with id '{}' already exists", id),
            source: None,
        });
    }

    // Create the STS configuration
    let create_config = CreateStsConfiguration {
        id: id.clone(),
        config_type: config_type.as_str().to_string(),
        value: params.value.clone(),
        created_at: now,
        updated_at: now,
    };
    repository.create_sts_configuration(&create_config).await?;

    // Broadcast config change event
    if publish_on_change_evt {
        on_config_change_tx
            .send(OnConfigChangeEvt::StsConfigCreated(StsConfigCreatedInfo {
                id: id.clone(),
                config_type: config_type.as_str().to_string(),
                value: params.value.clone(),
            }))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send config change event: {e}"))
            })?;
    }

    Ok(CreateStsConfigResponse {
        id,
        config_type: config_type.as_str().to_string(),
        value: params.value,
    })
}

/// Delete an STS configuration
///
/// This function:
/// 1. Verifies the configuration exists
/// 2. Deletes the configuration from the repository
/// 3. Optionally broadcasts a config change event
pub async fn delete_sts_config<R: UserRepositoryLike>(
    repository: &R,
    on_config_change_tx: &OnConfigChangeTx,
    params: DeleteStsConfigParams,
    publish_on_change_evt: bool,
) -> Result<DeleteStsConfigResponse, CommonError> {
    // Verify the config exists
    repository
        .get_sts_configuration_by_id(&params.id)
        .await?
        .ok_or_else(|| CommonError::NotFound {
            msg: "STS configuration not found".to_string(),
            lookup_id: params.id.clone(),
            source: None,
        })?;

    // Delete the configuration
    repository.delete_sts_configuration(&params.id).await?;

    // Broadcast config change event
    if publish_on_change_evt {
        on_config_change_tx
            .send(OnConfigChangeEvt::StsConfigDeleted(params.id.clone()))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send config change event: {e}"))
            })?;
    }

    Ok(DeleteStsConfigResponse { success: true })
}

/// Get an STS configuration by ID
pub async fn get_sts_config<R: UserRepositoryLike>(
    repository: &R,
    params: GetStsConfigParams,
) -> Result<StsConfiguration, CommonError> {
    repository
        .get_sts_configuration_by_id(&params.id)
        .await?
        .ok_or_else(|| CommonError::NotFound {
            msg: "STS configuration not found".to_string(),
            lookup_id: params.id,
            source: None,
        })
}

/// List STS configurations
///
/// This function lists all STS configurations with optional filtering by config_type.
pub async fn list_sts_configs<R: UserRepositoryLike>(
    repository: &R,
    params: ListStsConfigParams,
) -> Result<ListStsConfigResponse, CommonError> {
    let result = repository
        .list_sts_configurations(&params.pagination, params.config_type.as_deref())
        .await?;

    Ok(ListStsConfigResponse {
        items: result.items,
        next_page_token: result.next_page_token,
    })
}

/// Import an STS configuration (for syncing from soma.yaml)
///
/// This function:
/// 1. Creates the configuration if it doesn't exist
/// 2. Does not broadcast events (used for importing)
pub async fn import_sts_config<R: UserRepositoryLike>(
    repository: &R,
    params: CreateStsConfigParams,
) -> Result<CreateStsConfigResponse, CommonError> {
    // Validate config type
    let config_type = StsConfigType::from_str(&params.config_type).ok_or_else(|| {
        CommonError::InvalidRequest {
            msg: format!(
                "Invalid config type '{}'. Valid types are: jwt_template, dev",
                params.config_type
            ),
            source: None,
        }
    })?;

    // ID is required for import
    let id = params.id.ok_or_else(|| CommonError::InvalidRequest {
        msg: "ID is required for import".to_string(),
        source: None,
    })?;

    // Check if config already exists
    if repository.get_sts_configuration_by_id(&id).await?.is_some() {
        // Config already exists, return success without creating
        return Ok(CreateStsConfigResponse {
            id,
            config_type: config_type.as_str().to_string(),
            value: params.value,
        });
    }

    let now = WrappedChronoDateTime::now();

    // Create the STS configuration
    let create_config = CreateStsConfiguration {
        id: id.clone(),
        config_type: config_type.as_str().to_string(),
        value: params.value.clone(),
        created_at: now,
        updated_at: now,
    };
    repository.create_sts_configuration(&create_config).await?;

    Ok(CreateStsConfigResponse {
        id,
        config_type: config_type.as_str().to_string(),
        value: params.value,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::Repository;
    use shared::primitives::SqlMigrationLoader;
    use shared::test_utils::repository::setup_in_memory_database;
    use tokio::sync::broadcast;

    struct TestContext {
        identity_repo: Repository,
        on_config_change_tx: OnConfigChangeTx,
    }

    async fn setup_test_context() -> TestContext {
        shared::setup_test!();

        // Setup identity database
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let identity_repo = Repository::new(conn);

        let (on_config_change_tx, _rx) = broadcast::channel(100);

        TestContext {
            identity_repo,
            on_config_change_tx,
        }
    }

    #[test]
    fn test_sts_config_type_from_str() {
        assert_eq!(
            StsConfigType::from_str("jwt_template"),
            Some(StsConfigType::JwtTemplate)
        );
        assert_eq!(StsConfigType::from_str("dev"), Some(StsConfigType::Dev));
        assert_eq!(StsConfigType::from_str("invalid"), None);
    }

    #[test]
    fn test_sts_config_type_as_str() {
        assert_eq!(StsConfigType::JwtTemplate.as_str(), "jwt_template");
        assert_eq!(StsConfigType::Dev.as_str(), "dev");
    }

    #[tokio::test]
    async fn test_create_sts_config() {
        let ctx = setup_test_context().await;

        let params = CreateStsConfigParams {
            id: None,
            config_type: "jwt_template".to_string(),
            value: Some(r#"{"issuer":"test"}"#.to_string()),
        };

        let result = create_sts_config(
            &ctx.identity_repo,
            &ctx.on_config_change_tx,
            params,
            false,
        )
        .await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.id.is_empty());
        assert_eq!(response.config_type, "jwt_template");
        assert_eq!(response.value, Some(r#"{"issuer":"test"}"#.to_string()));

        // Verify config was created in repository
        let config = ctx
            .identity_repo
            .get_sts_configuration_by_id(&response.id)
            .await
            .unwrap();
        assert!(config.is_some());
    }

    #[tokio::test]
    async fn test_create_sts_config_with_custom_id() {
        let ctx = setup_test_context().await;

        let params = CreateStsConfigParams {
            id: Some("my-custom-id".to_string()),
            config_type: "dev".to_string(),
            value: None,
        };

        let result = create_sts_config(
            &ctx.identity_repo,
            &ctx.on_config_change_tx,
            params,
            false,
        )
        .await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.id, "my-custom-id");
        assert_eq!(response.config_type, "dev");
    }

    #[tokio::test]
    async fn test_create_sts_config_invalid_type() {
        let ctx = setup_test_context().await;

        let params = CreateStsConfigParams {
            id: None,
            config_type: "invalid-type".to_string(),
            value: None,
        };

        let result = create_sts_config(
            &ctx.identity_repo,
            &ctx.on_config_change_tx,
            params,
            false,
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_sts_config_duplicate_id() {
        let ctx = setup_test_context().await;

        let params = CreateStsConfigParams {
            id: Some("duplicate-id".to_string()),
            config_type: "jwt_template".to_string(),
            value: None,
        };

        // Create first config
        let result = create_sts_config(
            &ctx.identity_repo,
            &ctx.on_config_change_tx,
            params.clone(),
            false,
        )
        .await;
        assert!(result.is_ok());

        // Try to create duplicate
        let result = create_sts_config(
            &ctx.identity_repo,
            &ctx.on_config_change_tx,
            params,
            false,
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_sts_config_broadcasts_event() {
        let ctx = setup_test_context().await;
        let mut rx = ctx.on_config_change_tx.subscribe();

        let params = CreateStsConfigParams {
            id: None,
            config_type: "jwt_template".to_string(),
            value: Some(r#"{"issuer":"test"}"#.to_string()),
        };

        let result = create_sts_config(
            &ctx.identity_repo,
            &ctx.on_config_change_tx,
            params,
            true,
        )
        .await;
        assert!(result.is_ok());

        let response = result.unwrap();

        // Check that event was broadcast
        let event = rx.try_recv();
        assert!(event.is_ok());
        match event.unwrap() {
            OnConfigChangeEvt::StsConfigCreated(info) => {
                assert_eq!(info.id, response.id);
                assert_eq!(info.config_type, "jwt_template");
                assert_eq!(info.value, Some(r#"{"issuer":"test"}"#.to_string()));
            }
            _ => panic!("Expected StsConfigCreated event"),
        }
    }

    #[tokio::test]
    async fn test_delete_sts_config() {
        let ctx = setup_test_context().await;

        // First create a config
        let create_params = CreateStsConfigParams {
            id: Some("to-delete".to_string()),
            config_type: "jwt_template".to_string(),
            value: None,
        };
        create_sts_config(
            &ctx.identity_repo,
            &ctx.on_config_change_tx,
            create_params,
            false,
        )
        .await
        .unwrap();

        // Now delete it
        let delete_params = DeleteStsConfigParams {
            id: "to-delete".to_string(),
        };
        let result = delete_sts_config(
            &ctx.identity_repo,
            &ctx.on_config_change_tx,
            delete_params,
            false,
        )
        .await;
        assert!(result.is_ok());
        assert!(result.unwrap().success);

        // Verify config is gone
        let config = ctx
            .identity_repo
            .get_sts_configuration_by_id("to-delete")
            .await
            .unwrap();
        assert!(config.is_none());
    }

    #[tokio::test]
    async fn test_delete_sts_config_not_found() {
        let ctx = setup_test_context().await;

        let params = DeleteStsConfigParams {
            id: "nonexistent".to_string(),
        };

        let result = delete_sts_config(
            &ctx.identity_repo,
            &ctx.on_config_change_tx,
            params,
            false,
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_sts_config_broadcasts_event() {
        let ctx = setup_test_context().await;

        // First create a config
        let create_params = CreateStsConfigParams {
            id: Some("to-delete-event".to_string()),
            config_type: "jwt_template".to_string(),
            value: None,
        };
        create_sts_config(
            &ctx.identity_repo,
            &ctx.on_config_change_tx,
            create_params,
            false,
        )
        .await
        .unwrap();

        let mut rx = ctx.on_config_change_tx.subscribe();

        // Now delete it with broadcast
        let delete_params = DeleteStsConfigParams {
            id: "to-delete-event".to_string(),
        };
        let result = delete_sts_config(
            &ctx.identity_repo,
            &ctx.on_config_change_tx,
            delete_params,
            true,
        )
        .await;
        assert!(result.is_ok());

        // Check that event was broadcast
        let event = rx.try_recv();
        assert!(event.is_ok());
        match event.unwrap() {
            OnConfigChangeEvt::StsConfigDeleted(id) => {
                assert_eq!(id, "to-delete-event");
            }
            _ => panic!("Expected StsConfigDeleted event"),
        }
    }

    #[tokio::test]
    async fn test_get_sts_config() {
        let ctx = setup_test_context().await;

        // First create a config
        let create_params = CreateStsConfigParams {
            id: Some("to-get".to_string()),
            config_type: "jwt_template".to_string(),
            value: Some(r#"{"issuer":"test"}"#.to_string()),
        };
        create_sts_config(
            &ctx.identity_repo,
            &ctx.on_config_change_tx,
            create_params,
            false,
        )
        .await
        .unwrap();

        // Now get it
        let params = GetStsConfigParams {
            id: "to-get".to_string(),
        };
        let result = get_sts_config(&ctx.identity_repo, params).await;
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.id, "to-get");
        assert_eq!(config.config_type, "jwt_template");
        assert_eq!(config.value, Some(r#"{"issuer":"test"}"#.to_string()));
    }

    #[tokio::test]
    async fn test_get_sts_config_not_found() {
        let ctx = setup_test_context().await;

        let params = GetStsConfigParams {
            id: "nonexistent".to_string(),
        };

        let result = get_sts_config(&ctx.identity_repo, params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_sts_configs() {
        let ctx = setup_test_context().await;

        // Create a few configs
        for i in 1..=3 {
            let params = CreateStsConfigParams {
                id: Some(format!("config-{i}")),
                config_type: if i % 2 == 0 {
                    "dev".to_string()
                } else {
                    "jwt_template".to_string()
                },
                value: None,
            };
            create_sts_config(
                &ctx.identity_repo,
                &ctx.on_config_change_tx,
                params,
                false,
            )
            .await
            .unwrap();
        }

        // List all
        let params = ListStsConfigParams {
            pagination: PaginationRequest {
                page_size: 10,
                next_page_token: None,
            },
            config_type: None,
        };

        let result = list_sts_configs(&ctx.identity_repo, params).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.items.len(), 3);
    }

    #[tokio::test]
    async fn test_list_sts_configs_filter_by_type() {
        let ctx = setup_test_context().await;

        // Create configs of different types
        for i in 1..=4 {
            let params = CreateStsConfigParams {
                id: Some(format!("config-{i}")),
                config_type: if i % 2 == 0 {
                    "dev".to_string()
                } else {
                    "jwt_template".to_string()
                },
                value: None,
            };
            create_sts_config(
                &ctx.identity_repo,
                &ctx.on_config_change_tx,
                params,
                false,
            )
            .await
            .unwrap();
        }

        // List only jwt_template
        let params = ListStsConfigParams {
            pagination: PaginationRequest {
                page_size: 10,
                next_page_token: None,
            },
            config_type: Some("jwt_template".to_string()),
        };

        let result = list_sts_configs(&ctx.identity_repo, params).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.items.len(), 2);
        assert!(response
            .items
            .iter()
            .all(|c| c.config_type == "jwt_template"));
    }

    #[tokio::test]
    async fn test_import_sts_config() {
        let ctx = setup_test_context().await;

        let params = CreateStsConfigParams {
            id: Some("imported-config".to_string()),
            config_type: "jwt_template".to_string(),
            value: Some(r#"{"issuer":"imported"}"#.to_string()),
        };

        let result = import_sts_config(&ctx.identity_repo, params).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.id, "imported-config");

        // Verify config was created
        let config = ctx
            .identity_repo
            .get_sts_configuration_by_id("imported-config")
            .await
            .unwrap();
        assert!(config.is_some());
    }

    #[tokio::test]
    async fn test_import_sts_config_idempotent() {
        let ctx = setup_test_context().await;

        let params = CreateStsConfigParams {
            id: Some("idempotent-config".to_string()),
            config_type: "jwt_template".to_string(),
            value: Some(r#"{"issuer":"test"}"#.to_string()),
        };

        // Import twice
        let result1 = import_sts_config(&ctx.identity_repo, params.clone()).await;
        assert!(result1.is_ok());

        let result2 = import_sts_config(&ctx.identity_repo, params).await;
        assert!(result2.is_ok());

        // Should only have one config
        let params = ListStsConfigParams {
            pagination: PaginationRequest {
                page_size: 10,
                next_page_token: None,
            },
            config_type: None,
        };
        let list = list_sts_configs(&ctx.identity_repo, params).await.unwrap();
        assert_eq!(list.items.len(), 1);
    }

    #[tokio::test]
    async fn test_import_sts_config_requires_id() {
        let ctx = setup_test_context().await;

        let params = CreateStsConfigParams {
            id: None,
            config_type: "jwt_template".to_string(),
            value: None,
        };

        let result = import_sts_config(&ctx.identity_repo, params).await;
        assert!(result.is_err());
    }
}
