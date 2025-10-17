use std::{collections::HashMap, path::PathBuf, sync::Arc};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, MutexGuard};
use tracing::info;
use url::Url;
use utoipa::ToSchema;

use crate::error::CommonError;
use async_trait::async_trait;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SomaAgentDefinition {
    pub project: String,
    pub agent: String,
    pub description: String,
    pub name: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bridge: Option<BridgeConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BridgeConfig {
    pub encryption: BridgeEncryptionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[serde(transparent)]
pub struct BridgeEncryptionConfig(pub HashMap<String, EncryptionConfiguration>);

// TODO: this is duplicated in the bridge crate
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, ToSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum EnvelopeEncryptionKeyId {
    AwsKms { arn: String },
    Local { key_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EncryptionConfiguration {
    pub encrypted_data_encryption_key: String,
    pub envelope_encryption_key_id: EnvelopeEncryptionKeyId,
}

#[async_trait]
pub trait SomaAgentDefinitionLike: Send + Sync {
    async fn get_definition(&self) -> Result<SomaAgentDefinition, CommonError>;
    async fn add_data_encryption_key(
        &self,
        key_id: String,
        key: String,
        envelope_encryption_key_id: EnvelopeEncryptionKeyId,
    ) -> Result<(), CommonError>;
    async fn list_data_encryption_keys(&self) -> Result<Vec<EncryptionConfiguration>, CommonError>;
    async fn remove_data_encryption_key(&self, key_id: String) -> Result<(), CommonError>;
    async fn reload(&self) -> Result<(), CommonError>;
}

#[derive(Debug, Clone)]
pub struct YamlSomaAgentDefinition {
    pub cached_definition: Arc<Mutex<SomaAgentDefinition>>,
    pub path: PathBuf,
}

impl YamlSomaAgentDefinition {

    fn load_agent_definition(path: PathBuf) -> Result<SomaAgentDefinition, CommonError> {
        let yaml_str = std::fs::read_to_string(&path).map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!("Failed to read soma definition: {e:?}"))
        })?;
        let definition = serde_yaml::from_str(&yaml_str).map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!("Failed to parse soma definition: {e:?}"))
        })?;
        Ok(definition)
    }

    pub fn load_from_file(path: PathBuf) -> Result<Self, CommonError> {
        let definition = Self::load_agent_definition(path.clone())?;
        Ok(Self {
            cached_definition: Arc::new(Mutex::new(definition)),
            path,
        })
    }

    pub async fn save(&self, guard: MutexGuard<'_, SomaAgentDefinition>) -> Result<(), CommonError> {
        std::fs::write(
            self.path.clone(),
            serde_yaml::to_string(&*guard).map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!(
                    "Failed to serialize soma definition: {e:?}"
                ))
            })?,
        )
        .map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!("Failed to write soma definition: {e:?}"))
        })?;
        Ok(())
    }
}

#[async_trait]
impl SomaAgentDefinitionLike for YamlSomaAgentDefinition {

    async fn reload(&self) -> Result<(), CommonError> {
        let definition = Self::load_agent_definition(self.path.clone())?;
        *self.cached_definition.lock().await = definition;
        info!("Soma definition reloaded from file: {:?}", self.path);
        Ok(())
    }

    async fn get_definition(&self) -> Result<SomaAgentDefinition, CommonError> {
        Ok(self.cached_definition.lock().await.clone())
    }

    async fn add_data_encryption_key(
        &self,
        key_id: String,
        key: String,
        envelope_encryption_key_id: EnvelopeEncryptionKeyId,
    ) -> Result<(), CommonError> {
        let mut definition = self.cached_definition.lock().await;
        if definition.bridge.is_none() {
            definition.bridge = Some(BridgeConfig {
                encryption: BridgeEncryptionConfig(HashMap::new()),
            });
        }
        let bridge = match &mut definition.bridge {
            Some(bridge) => bridge,
            None => return Err(CommonError::Unknown(anyhow::anyhow!("Bridge configuration not found"))),
        };
        bridge.encryption.0.insert(
            key_id.clone(),
            EncryptionConfiguration {
                encrypted_data_encryption_key: key,
                envelope_encryption_key_id: envelope_encryption_key_id,
            },
        );
        info!("Data encryption key added to bridge: {:?}", key_id);
        self.save(definition).await?;
        Ok(())
    }

    async fn list_data_encryption_keys(&self) -> Result<Vec<EncryptionConfiguration>, CommonError> {
        let definition = self.cached_definition.lock().await;
        let bridge = match &definition.bridge {
            Some(bridge) => bridge,
            None => return Ok(vec![]),
        };
        Ok(bridge
            .encryption
            .0
            .values()
            .cloned().collect())
    }

    async fn remove_data_encryption_key(&self, key_id: String) -> Result<(), CommonError> {
        let mut definition = self.cached_definition.lock().await;

        let bridge = match &mut definition.bridge {
            Some(bridge) => bridge,
            None => return Ok(()),
        };
        
        match bridge.encryption.0.remove(&key_id) {
            Some(_) => (),
            None => return Ok(()),
        };
        info!("Data encryption key removed from bridge: {:?}", key_id);
        self.save(definition).await?;
        Ok(())
    }
}
