use std::{collections::HashMap, path::PathBuf, sync::Arc};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, MutexGuard};
use tracing::info;
use utoipa::ToSchema;

use crate::error::CommonError;
use async_trait::async_trait;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub struct SomaAgentDefinition {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encryption: Option<EncryptionConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bridge: Option<BridgeConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secrets: Option<HashMap<String, SecretConfig>>,
}

/// Configuration for a secret stored in soma.yaml
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SecretConfig {
    /// The encrypted value of the secret
    pub value: String,
    /// The DEK alias used to encrypt this secret
    pub dek_alias: String,
}

/// Top-level encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub struct EncryptionConfig {
    /// Map of envelope key id (ARN or file_name) -> envelope key configuration with nested DEKs
    /// DEKs are stored by their alias name (e.g., "default") rather than UUID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub envelope_keys: Option<HashMap<String, EnvelopeKeyConfig>>,
}

/// Envelope encryption key configuration with nested DEKs
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, ToSchema)]
pub struct EnvelopeKeyConfigAwsKms {
    pub arn: String,
    pub region: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deks: Option<HashMap<String, DekConfig>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, ToSchema)]
pub struct EnvelopeKeyConfigLocal {
    pub file_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deks: Option<HashMap<String, DekConfig>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EnvelopeKeyConfig {
    AwsKms(EnvelopeKeyConfigAwsKms),
    Local(EnvelopeKeyConfigLocal),
}

impl EnvelopeKeyConfig {
    /// Get mutable reference to the DEKs map, creating it if it doesn't exist
    pub fn deks_mut(&mut self) -> &mut HashMap<String, DekConfig> {
        match self {
            EnvelopeKeyConfig::AwsKms(aws_kms) => {
                if aws_kms.deks.is_none() {
                    aws_kms.deks = Some(HashMap::new());
                }
                aws_kms.deks.as_mut().unwrap()
            }
            EnvelopeKeyConfig::Local(local) => {
                if local.deks.is_none() {
                    local.deks = Some(HashMap::new());
                }
                local.deks.as_mut().unwrap()
            }
        }
    }

    /// Get reference to the DEKs map
    pub fn deks(&self) -> Option<&HashMap<String, DekConfig>> {
        match self {
            EnvelopeKeyConfig::AwsKms(aws_kms) => aws_kms.deks.as_ref(),
            EnvelopeKeyConfig::Local(local) => local.deks.as_ref(),
        }
    }
}

/// Data encryption key configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DekConfig {
    pub encrypted_key: String,
}

// Keep old EnvelopeEncryptionKey for backwards compatibility during transition
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, ToSchema)]
pub struct EnvelopeEncryptionKeyAwsKms {
    pub arn: String,
    pub region: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, ToSchema)]
pub struct EnvelopeEncryptionKeyLocal {
    pub file_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, ToSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum EnvelopeEncryptionKey {
    AwsKms(EnvelopeEncryptionKeyAwsKms),
    Local(EnvelopeEncryptionKeyLocal),
}

impl EnvelopeEncryptionKey {
    /// Get the key id (ARN for KMS, file_name for local)
    pub fn key_id(&self) -> String {
        match self {
            EnvelopeEncryptionKey::AwsKms(aws_kms) => aws_kms.arn.clone(),
            EnvelopeEncryptionKey::Local(local) => local.file_name.clone(),
        }
    }
}

impl From<EnvelopeEncryptionKey> for EnvelopeKeyConfig {
    fn from(key: EnvelopeEncryptionKey) -> Self {
        match key {
            EnvelopeEncryptionKey::AwsKms(aws_kms) => {
                EnvelopeKeyConfig::AwsKms(EnvelopeKeyConfigAwsKms {
                    arn: aws_kms.arn,
                    region: aws_kms.region,
                    deks: None,
                })
            }
            EnvelopeEncryptionKey::Local(local) => {
                EnvelopeKeyConfig::Local(EnvelopeKeyConfigLocal {
                    file_name: local.file_name,
                    deks: None,
                })
            }
        }
    }
}

impl From<EnvelopeKeyConfig> for EnvelopeEncryptionKey {
    fn from(config: EnvelopeKeyConfig) -> Self {
        match config {
            EnvelopeKeyConfig::AwsKms(aws_kms) => {
                EnvelopeEncryptionKey::AwsKms(EnvelopeEncryptionKeyAwsKms {
                    arn: aws_kms.arn,
                    region: aws_kms.region,
                })
            }
            EnvelopeKeyConfig::Local(local) => {
                EnvelopeEncryptionKey::Local(EnvelopeEncryptionKeyLocal {
                    file_name: local.file_name,
                })
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BridgeConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub providers: Option<HashMap<String, ProviderConfig>>,
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
    pub dek_alias: String,
}

#[async_trait]
pub trait SomaAgentDefinitionLike: Send + Sync {
    async fn get_definition(&self) -> Result<SomaAgentDefinition, CommonError>;

    // Envelope key operations
    async fn add_envelope_key(
        &self,
        key_id: String,
        config: EnvelopeKeyConfig,
    ) -> Result<(), CommonError>;
    async fn remove_envelope_key(&self, key_id: String) -> Result<(), CommonError>;

    // DEK operations (DEKs are nested under their envelope key, keyed by alias)
    async fn add_dek(
        &self,
        envelope_key_id: String,
        alias: String,
        encrypted_key: String,
    ) -> Result<(), CommonError>;
    async fn remove_dek(&self, envelope_key_id: String, alias: String) -> Result<(), CommonError>;
    /// Rename a DEK from one key (e.g., UUID) to another (e.g., alias)
    async fn rename_dek(
        &self,
        envelope_key_id: String,
        old_key: String,
        new_key: String,
    ) -> Result<(), CommonError>;

    // Provider operations
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

    // Function instance operations
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

    // Secret operations
    async fn add_secret(&self, key: String, config: SecretConfig) -> Result<(), CommonError>;
    async fn update_secret(&self, key: String, config: SecretConfig) -> Result<(), CommonError>;
    async fn remove_secret(&self, key: String) -> Result<(), CommonError>;

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

    fn ensure_encryption_config(definition: &mut SomaAgentDefinition) {
        if definition.encryption.is_none() {
            definition.encryption = Some(EncryptionConfig::default());
        }
    }

    fn ensure_bridge_config(definition: &mut SomaAgentDefinition) {
        if definition.bridge.is_none() {
            definition.bridge = Some(BridgeConfig { providers: None });
        }
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

    async fn add_envelope_key(
        &self,
        key_id: String,
        config: EnvelopeKeyConfig,
    ) -> Result<(), CommonError> {
        let mut definition = self.cached_definition.lock().await;
        Self::ensure_encryption_config(&mut definition);

        let encryption = definition.encryption.as_mut().unwrap();
        if encryption.envelope_keys.is_none() {
            encryption.envelope_keys = Some(HashMap::new());
        }

        encryption
            .envelope_keys
            .as_mut()
            .unwrap()
            .insert(key_id.clone(), config);
        info!("Envelope key added: {:?}", key_id);
        self.save(definition).await?;
        Ok(())
    }

    async fn remove_envelope_key(&self, key_id: String) -> Result<(), CommonError> {
        let mut definition = self.cached_definition.lock().await;

        if let Some(encryption) = &mut definition.encryption {
            if let Some(envelope_keys) = &mut encryption.envelope_keys {
                envelope_keys.remove(&key_id);
                info!("Envelope key removed: {:?}", key_id);
                self.save(definition).await?;
            }
        }
        Ok(())
    }

    async fn add_dek(
        &self,
        envelope_key_id: String,
        alias: String,
        encrypted_key: String,
    ) -> Result<(), CommonError> {
        let mut definition = self.cached_definition.lock().await;
        Self::ensure_encryption_config(&mut definition);

        let encryption = definition.encryption.as_mut().unwrap();
        if encryption.envelope_keys.is_none() {
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "Envelope key {envelope_key_id} not found - cannot add DEK"
            )));
        }

        let envelope_keys = encryption.envelope_keys.as_mut().unwrap();
        let envelope_key = envelope_keys.get_mut(&envelope_key_id).ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!(
                "Envelope key {envelope_key_id} not found - cannot add DEK"
            ))
        })?;

        envelope_key
            .deks_mut()
            .insert(alias.clone(), DekConfig { encrypted_key });
        info!(
            "DEK '{}' added under envelope key {}",
            alias, envelope_key_id
        );
        self.save(definition).await?;
        Ok(())
    }

    async fn remove_dek(&self, envelope_key_id: String, alias: String) -> Result<(), CommonError> {
        let mut definition = self.cached_definition.lock().await;

        if let Some(encryption) = &mut definition.encryption {
            if let Some(envelope_keys) = &mut encryption.envelope_keys {
                if let Some(envelope_key) = envelope_keys.get_mut(&envelope_key_id) {
                    envelope_key.deks_mut().remove(&alias);
                    info!(
                        "DEK '{}' removed from envelope key {}",
                        alias, envelope_key_id
                    );
                    self.save(definition).await?;
                }
            }
        }
        Ok(())
    }

    async fn rename_dek(
        &self,
        envelope_key_id: String,
        old_key: String,
        new_key: String,
    ) -> Result<(), CommonError> {
        let mut definition = self.cached_definition.lock().await;

        if let Some(encryption) = &mut definition.encryption {
            if let Some(envelope_keys) = &mut encryption.envelope_keys {
                if let Some(envelope_key) = envelope_keys.get_mut(&envelope_key_id) {
                    let deks = envelope_key.deks_mut();
                    if let Some(dek_config) = deks.remove(&old_key) {
                        deks.insert(new_key.clone(), dek_config);
                        info!(
                            "DEK renamed from '{}' to '{}' under envelope key {}",
                            old_key, new_key, envelope_key_id
                        );
                        self.save(definition).await?;
                    }
                }
            }
        }
        Ok(())
    }

    async fn add_provider(
        &self,
        provider_id: String,
        config: ProviderConfig,
    ) -> Result<(), CommonError> {
        let mut definition = self.cached_definition.lock().await;
        Self::ensure_bridge_config(&mut definition);

        let bridge = definition.bridge.as_mut().unwrap();
        if bridge.providers.is_none() {
            bridge.providers = Some(HashMap::new());
        }

        bridge
            .providers
            .as_mut()
            .unwrap()
            .insert(provider_id.clone(), config);
        info!("Provider added to bridge: {:?}", provider_id);
        self.save(definition).await?;
        Ok(())
    }

    async fn remove_provider(&self, provider_id: String) -> Result<(), CommonError> {
        let mut definition = self.cached_definition.lock().await;

        if let Some(bridge) = &mut definition.bridge {
            if let Some(providers) = &mut bridge.providers {
                providers.remove(&provider_id);
                info!("Provider removed from bridge: {:?}", provider_id);
                self.save(definition).await?;
            }
        }
        Ok(())
    }

    async fn update_provider(
        &self,
        provider_id: String,
        config: ProviderConfig,
    ) -> Result<(), CommonError> {
        let mut definition = self.cached_definition.lock().await;
        Self::ensure_bridge_config(&mut definition);

        let bridge = definition.bridge.as_mut().unwrap();
        if bridge.providers.is_none() {
            bridge.providers = Some(HashMap::new());
        }

        let providers = bridge.providers.as_mut().unwrap();

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

    async fn add_secret(&self, key: String, config: SecretConfig) -> Result<(), CommonError> {
        let mut definition = self.cached_definition.lock().await;

        if definition.secrets.is_none() {
            definition.secrets = Some(HashMap::new());
        }

        definition
            .secrets
            .as_mut()
            .unwrap()
            .insert(key.clone(), config);
        info!("Secret added: {:?}", key);
        self.save(definition).await?;
        Ok(())
    }

    async fn update_secret(&self, key: String, config: SecretConfig) -> Result<(), CommonError> {
        let mut definition = self.cached_definition.lock().await;

        if definition.secrets.is_none() {
            definition.secrets = Some(HashMap::new());
        }

        definition
            .secrets
            .as_mut()
            .unwrap()
            .insert(key.clone(), config);
        info!("Secret updated: {:?}", key);
        self.save(definition).await?;
        Ok(())
    }

    async fn remove_secret(&self, key: String) -> Result<(), CommonError> {
        let mut definition = self.cached_definition.lock().await;

        if let Some(secrets) = &mut definition.secrets {
            secrets.remove(&key);
            info!("Secret removed: {:?}", key);
            self.save(definition).await?;
        }
        Ok(())
    }
}
