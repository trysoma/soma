use std::{collections::HashMap, path::PathBuf, sync::Arc};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, MutexGuard};
use tracing::info;
use utoipa::ToSchema;

use crate::error::CommonError;
use async_trait::async_trait;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SomaAgentDefinition {
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bridge: Option<BridgeConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BridgeConfig {
    pub encryption: BridgeEncryptionConfig,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub providers: Option<HashMap<String, ProviderConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[serde(transparent)]
pub struct BridgeEncryptionConfig(pub HashMap<String, EncryptionConfiguration>);

// TODO: this is duplicated in the bridge crate
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, ToSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum EnvelopeEncryptionKeyId {
    AwsKms { arn: String, region: String },
    Local { location: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct EncryptionConfiguration {
    pub encrypted_data_encryption_key: String,
    pub envelope_encryption_key_id: EnvelopeEncryptionKeyId,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ProviderConfig {
    pub provider_controller_type_id: String,
    pub credential_controller_type_id: String,
    pub display_name: String,
    pub resource_server_credential: CredentialConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_credential: Option<CredentialConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub functions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CredentialConfig {
    pub id: String,
    pub type_id: String,
    pub metadata: serde_json::Value,
    pub value: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_rotation_time: Option<String>,
    pub data_encryption_key_id: String,
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
    async fn add_provider(
        &self,
        provider_id: String,
        config: ProviderConfig,
    ) -> Result<(), CommonError>;
    async fn remove_provider(&self, provider_id: String) -> Result<(), CommonError>;
    async fn update_provider(
        &self,
        provider_id: String,
        config: ProviderConfig,
    ) -> Result<(), CommonError>;
    async fn add_function_instance(
        &self,
        provider_controller_type_id: String,
        function_controller_type_id: String,
        provider_instance_id: String,
    ) -> Result<(), CommonError>;
    async fn remove_function_instance(
        &self,
        provider_controller_type_id: String,
        function_controller_type_id: String,
        provider_instance_id: String,
    ) -> Result<(), CommonError>;
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

    pub async fn save(
        &self,
        guard: MutexGuard<'_, SomaAgentDefinition>,
    ) -> Result<(), CommonError> {
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
                providers: None,
            });
        }
        let bridge = match &mut definition.bridge {
            Some(bridge) => bridge,
            None => {
                return Err(CommonError::Unknown(anyhow::anyhow!(
                    "Bridge configuration not found"
                )));
            }
        };
        bridge.encryption.0.insert(
            key_id.clone(),
            EncryptionConfiguration {
                encrypted_data_encryption_key: key,
                envelope_encryption_key_id,
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
        Ok(bridge.encryption.0.values().cloned().collect())
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

    async fn add_provider(
        &self,
        provider_id: String,
        config: ProviderConfig,
    ) -> Result<(), CommonError> {
        let mut definition = self.cached_definition.lock().await;
        if definition.bridge.is_none() {
            definition.bridge = Some(BridgeConfig {
                encryption: BridgeEncryptionConfig(HashMap::new()),
                providers: Some(HashMap::new()),
            });
        }
        let bridge = match &mut definition.bridge {
            Some(bridge) => bridge,
            None => {
                return Err(CommonError::Unknown(anyhow::anyhow!(
                    "Bridge configuration not found"
                )));
            }
        };
        if bridge.providers.is_none() {
            bridge.providers = Some(HashMap::new());
        }
        let providers = bridge.providers.as_mut().unwrap();
        providers.insert(provider_id.clone(), config);
        info!("Provider added to bridge: {:?}", provider_id);
        self.save(definition).await?;
        Ok(())
    }

    async fn remove_provider(&self, provider_id: String) -> Result<(), CommonError> {
        let mut definition = self.cached_definition.lock().await;

        let bridge = match &mut definition.bridge {
            Some(bridge) => bridge,
            None => return Ok(()),
        };

        let providers = match &mut bridge.providers {
            Some(providers) => providers,
            None => return Ok(()),
        };

        match providers.remove(&provider_id) {
            Some(_) => (),
            None => return Ok(()),
        };
        info!("Provider removed from bridge: {:?}", provider_id);
        self.save(definition).await?;
        Ok(())
    }

    async fn update_provider(
        &self,
        provider_id: String,
        config: ProviderConfig,
    ) -> Result<(), CommonError> {
        let mut definition = self.cached_definition.lock().await;

        // Create bridge configuration if it doesn't exist (same as add_provider)
        if definition.bridge.is_none() {
            definition.bridge = Some(BridgeConfig {
                encryption: BridgeEncryptionConfig(HashMap::new()),
                providers: Some(HashMap::new()),
            });
        }

        let bridge = match &mut definition.bridge {
            Some(bridge) => bridge,
            None => {
                return Err(CommonError::Unknown(anyhow::anyhow!(
                    "Bridge configuration not found"
                )));
            }
        };

        // Create providers HashMap if it doesn't exist
        if bridge.providers.is_none() {
            bridge.providers = Some(HashMap::new());
        }

        let providers = match &mut bridge.providers {
            Some(providers) => providers,
            None => return Err(CommonError::Unknown(anyhow::anyhow!("Providers not found"))),
        };

        match providers.get_mut(&provider_id) {
            Some(existing_config) => {
                // Update the provider config, preserving functions if not provided in the update
                if config.functions.is_some() {
                    *existing_config = config;
                } else {
                    let functions = existing_config.functions.clone();
                    *existing_config = config;
                    existing_config.functions = functions;
                }
            }
            None => {
                // Provider doesn't exist, add it
                providers.insert(provider_id.clone(), config);
            }
        };

        info!("Provider updated in bridge: {:?}", provider_id);
        self.save(definition).await?;
        Ok(())
    }

    async fn add_function_instance(
        &self,
        provider_controller_type_id: String,
        function_controller_type_id: String,
        provider_instance_id: String,
    ) -> Result<(), CommonError> {
        let mut definition = self.cached_definition.lock().await;
        let bridge = match &mut definition.bridge {
            Some(bridge) => bridge,
            None => {
                return Err(CommonError::Unknown(anyhow::anyhow!(
                    "Bridge configuration not found"
                )));
            }
        };
        let providers = match &mut bridge.providers {
            Some(providers) => providers,
            None => return Err(CommonError::Unknown(anyhow::anyhow!("Providers not found"))),
        };
        let provider = match providers.get_mut(&provider_instance_id) {
            Some(provider) => provider,
            None => return Err(CommonError::Unknown(anyhow::anyhow!("Provider not found"))),
        };
        if provider.functions.is_none() {
            provider.functions = Some(Vec::new());
        }
        let functions = provider.functions.as_mut().unwrap();
        functions.push(function_controller_type_id.clone());
        info!(
            "Function instance added to provider {}: {:?}",
            provider_controller_type_id, function_controller_type_id
        );
        self.save(definition).await?;
        Ok(())
    }

    async fn remove_function_instance(
        &self,
        provider_controller_type_id: String,
        function_controller_type_id: String,
        provider_instance_id: String,
    ) -> Result<(), CommonError> {
        let mut definition = self.cached_definition.lock().await;
        let bridge = match &mut definition.bridge {
            Some(bridge) => bridge,
            None => return Ok(()),
        };
        let providers = match &mut bridge.providers {
            Some(providers) => providers,
            None => return Ok(()),
        };
        let provider = match providers.get_mut(&provider_instance_id) {
            Some(provider) => provider,
            None => return Ok(()),
        };
        let functions = match &mut provider.functions {
            Some(functions) => functions,
            None => return Ok(()),
        };

        functions.retain(|f| *f != function_controller_type_id);

        info!(
            "Function instance ({}) removed from provider ({}, {})",
            function_controller_type_id, provider_controller_type_id, provider_instance_id
        );
        self.save(definition).await?;
        Ok(())
    }
}
