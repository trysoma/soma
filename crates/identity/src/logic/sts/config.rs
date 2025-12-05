use serde::{Deserialize, Serialize};
use shared::error::CommonError;
use shared::primitives::{PaginatedResponse, PaginationRequest, WrappedChronoDateTime};
use utoipa::ToSchema;

use crate::logic::token_mapping::template::{
    JwtTokenTemplateConfig, JwtTokenTemplateValidationConfig,
};
use crate::logic::{OnConfigChangeEvt, OnConfigChangeTx};
use crate::repository::{StsConfigurationDb, UserRepositoryLike};

pub type StsConfigId = String;

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct DevModeConfig {
    pub id: StsConfigId,
}

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct JwtTemplateModeConfig {
    pub id: StsConfigId,
    pub mapping_template: JwtTokenTemplateConfig,
    pub validation_template: JwtTokenTemplateValidationConfig,
}

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum StsTokenConfig {
    JwtTemplate(JwtTemplateModeConfig),
    DevMode(DevModeConfig),
}

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StsTokenConfigType {
    JwtTemplate,
    DevMode,
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
    params: StsTokenConfig,
    publish_on_change_evt: bool,
) -> Result<StsTokenConfig, CommonError> {
    let now = WrappedChronoDateTime::now();
    let id = match &params {
        StsTokenConfig::JwtTemplate(config) => config.id.clone(),
        StsTokenConfig::DevMode(config) => config.id.clone(),
    };

    // Check if config with this ID already exists
    if repository.get_sts_configuration_by_id(&id).await?.is_some() {
        return Err(CommonError::InvalidRequest {
            msg: format!("STS configuration with id '{id}' already exists"),
            source: None,
        });
    }

    repository
        .create_sts_configuration(&StsConfigurationDb {
            config: params.clone(),
            created_at: now,
            updated_at: now,
        })
        .await?;

    // Broadcast config change event
    if publish_on_change_evt {
        on_config_change_tx
            .send(OnConfigChangeEvt::StsConfigCreated(params.clone()))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send config change event: {e}"))
            })?;
    }

    Ok(params)
}

/// Parameters for deleting an STS configuration
#[derive(Debug, Deserialize, ToSchema)]
pub struct DeleteStsConfigParams {
    /// The ID of the STS configuration to delete
    pub id: String,
}

/// Response from deleting an STS configuration
pub type DeleteStsConfigResponse = ();

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

    Ok(())
}

/// Parameters for getting an STS configuration
#[derive(Debug, Deserialize, ToSchema)]
pub struct GetStsConfigParams {
    /// The ID of the STS configuration to get
    pub id: String,
}

/// Get an STS configuration by ID
pub async fn get_sts_config<R: UserRepositoryLike>(
    repository: &R,
    params: GetStsConfigParams,
) -> Result<StsTokenConfig, CommonError> {
    repository
        .get_sts_configuration_by_id(&params.id)
        .await?
        .ok_or_else(|| CommonError::NotFound {
            msg: "STS configuration not found".to_string(),
            lookup_id: params.id,
            source: None,
        })
        .map(|config| config.config)
}

/// Response from listing STS configurations
pub type ListStsConfigResponse = PaginatedResponse<StsTokenConfig>;

/// List STS configurations
///
/// This function lists all STS configurations with optional filtering by config_type.
pub async fn list_sts_configs<R: UserRepositoryLike>(
    repository: &R,
    pagination: &PaginationRequest,
) -> Result<ListStsConfigResponse, CommonError> {
    let result = repository.list_sts_configurations(pagination, None).await?;

    Ok(ListStsConfigResponse {
        items: result
            .items
            .into_iter()
            .map(|config| config.config)
            .collect(),
        next_page_token: result.next_page_token,
    })
}
