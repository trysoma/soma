use std::{collections::HashMap, path::PathBuf, sync::Arc};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, MutexGuard};
use tracing::trace;
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment_variables: Option<HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity: Option<IdentityConfig>,
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

/// Identity configuration for API keys, STS, and user auth flows
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub struct IdentityConfig {
    /// API keys configuration (key is the API key ID)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_keys: Option<HashMap<String, ApiKeyYamlConfig>>,
    /// STS configurations (key is the STS config ID)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sts_configurations: Option<HashMap<String, StsConfigYaml>>,
    /// User auth flow configurations (key is the config ID)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_auth_flows: Option<HashMap<String, UserAuthFlowYamlConfig>>,
}

/// API key configuration stored in soma.yaml
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ApiKeyYamlConfig {
    /// Description of the API key
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The encrypted hashed value of the API key
    pub encrypted_hashed_value: String,
    /// The DEK alias used for encryption
    pub dek_alias: String,
    /// The role assigned to this API key
    pub role: String,
    /// The user ID associated with this API key
    pub user_id: String,
}

/// STS configuration stored in soma.yaml
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum StsConfigYaml {
    /// JWT template configuration for external IdPs
    JwtTemplate(JwtTemplateConfigYaml),
    /// Dev mode configuration (allows any authentication in dev)
    Dev {},
}

/// User auth flow configuration stored in soma.yaml (encrypted)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UserAuthFlowYamlConfig {
    /// OIDC authorization code flow
    OidcAuthorizationCodeFlow(EncryptedOidcYamlConfig),
    /// OAuth authorization code flow
    OauthAuthorizationCodeFlow(EncryptedOauthYamlConfig),
    /// OIDC authorization code flow with PKCE
    OidcAuthorizationCodePkceFlow(EncryptedOidcYamlConfig),
    /// OAuth authorization code flow with PKCE
    OauthAuthorizationCodePkceFlow(EncryptedOauthYamlConfig),
}

/// Encrypted OAuth configuration for YAML storage
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct EncryptedOauthYamlConfig {
    /// Authorization endpoint URL
    pub authorization_endpoint: String,
    /// Token endpoint URL
    pub token_endpoint: String,
    /// JWKS endpoint URL for token verification
    pub jwks_endpoint: String,
    /// OAuth client ID
    pub client_id: String,
    /// Encrypted client secret
    pub encrypted_client_secret: String,
    /// DEK alias used for encryption
    pub dek_alias: String,
    /// OAuth scopes
    pub scopes: Vec<String>,
    /// Token introspection endpoint URL (RFC 7662) - if set, access tokens are treated as opaque
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub introspect_url: Option<String>,
    /// Token mapping configuration (serialized as JSON)
    pub oauth_mapping_config: serde_json::Value,
}

/// Encrypted OIDC configuration for YAML storage
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct EncryptedOidcYamlConfig {
    #[serde(flatten)]
    /// Base OAuth configuration
    pub base_config: EncryptedOauthYamlConfig,
    /// OIDC discovery endpoint (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discovery_endpoint: Option<String>,
    /// Userinfo endpoint URL (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub userinfo_endpoint: Option<String>,
    /// Token introspection endpoint URL (RFC 7662) - if set, access tokens are treated as opaque
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub introspect_url: Option<String>,
    /// Token mapping configuration (serialized as JSON)
    pub oidc_mapping_config: serde_json::Value,
}

/// JWT template configuration for validating external JWTs
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct JwtTemplateConfigYaml {
    /// JWKS URI to fetch public keys from
    pub jwks_uri: String,
    /// Where to find the token in the request
    pub token_location: TokenLocationYaml,
    /// Validation rules
    pub validation: JwtValidationConfigYaml,
    /// Field mapping from JWT claims to internal fields
    pub mapping: JwtMappingConfigYaml,
    /// Group to role mappings
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_to_role_mappings: Option<Vec<GroupToRoleMappingYaml>>,
    /// Scope to role mappings
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_to_role_mappings: Option<Vec<ScopeToRoleMappingYaml>>,
    /// Scope to group mappings (maps scopes to internal groups)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_to_group_mappings: Option<Vec<ScopeToGroupMappingYaml>>,
}

/// Where to find the token in the request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TokenLocationYaml {
    /// Token is in a header (e.g., Authorization: Bearer <token>)
    Header { name: String },
    /// Token is in a cookie
    Cookie { name: String },
}

/// JWT validation configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct JwtValidationConfigYaml {
    /// Expected issuer (iss claim)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
    /// Valid audiences (aud claim)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_audiences: Option<Vec<String>>,
    /// Required scopes
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_scopes: Option<Vec<String>>,
    /// Required groups
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_groups: Option<Vec<String>>,
}

/// JWT claim field mapping
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct JwtMappingConfigYaml {
    /// Field name for issuer (default: "iss")
    #[serde(default = "default_iss_field")]
    pub issuer_field: String,
    /// Field name for audience (default: "aud")
    #[serde(default = "default_aud_field")]
    pub audience_field: String,
    /// Field name for subject (default: "sub")
    #[serde(default = "default_sub_field")]
    pub sub_field: String,
    /// Field name for email (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email_field: Option<String>,
    /// Field name for groups (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub groups_field: Option<String>,
    /// Field name for scopes (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scopes_field: Option<String>,
}

fn default_iss_field() -> String {
    "iss".to_string()
}

fn default_aud_field() -> String {
    "aud".to_string()
}

fn default_sub_field() -> String {
    "sub".to_string()
}

/// Group to role mapping
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct GroupToRoleMappingYaml {
    /// The group name to match
    pub group: String,
    /// The role to assign when matched
    pub role: String,
}

/// Scope to role mapping
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ScopeToRoleMappingYaml {
    /// The scope to match
    pub scope: String,
    /// The role to assign when matched
    pub role: String,
}

/// Scope to group mapping (maps external scopes to internal groups)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ScopeToGroupMappingYaml {
    /// The scope to match
    pub scope: String,
    /// The internal group to assign when matched
    pub group: String,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<HashMap<String, McpServerConfig>>,
}

/// Configuration for an MCP server instance stored in soma.yaml
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct McpServerConfig {
    /// Display name for the MCP server
    pub name: String,
    /// Functions exposed by this MCP server
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub functions: Option<Vec<McpServerFunctionConfig>>,
}

/// Configuration for a function mapping within an MCP server
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct McpServerFunctionConfig {
    /// The function controller type ID
    pub function_controller_type_id: String,
    /// The provider controller type ID
    pub provider_controller_type_id: String,
    /// The provider instance ID
    pub provider_instance_id: String,
    /// The MCP function name exposed to clients
    pub function_name: String,
    /// Optional description for the function
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function_description: Option<String>,
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

    // MCP server instance operations
    async fn add_mcp_server(
        &self,
        mcp_server_id: String,
        config: McpServerConfig,
    ) -> Result<(), CommonError>;
    async fn update_mcp_server(
        &self,
        mcp_server_id: String,
        config: McpServerConfig,
    ) -> Result<(), CommonError>;
    async fn remove_mcp_server(&self, mcp_server_id: String) -> Result<(), CommonError>;

    // MCP server function operations
    async fn add_mcp_server_function(
        &self,
        mcp_server_id: String,
        function_config: McpServerFunctionConfig,
    ) -> Result<(), CommonError>;
    async fn update_mcp_server_function(
        &self,
        mcp_server_id: String,
        function_config: McpServerFunctionConfig,
    ) -> Result<(), CommonError>;
    async fn remove_mcp_server_function(
        &self,
        mcp_server_id: String,
        function_controller_type_id: String,
        provider_controller_type_id: String,
        provider_instance_id: String,
    ) -> Result<(), CommonError>;

    // Secret operations
    async fn add_secret(&self, key: String, config: SecretConfig) -> Result<(), CommonError>;
    async fn update_secret(&self, key: String, config: SecretConfig) -> Result<(), CommonError>;
    async fn remove_secret(&self, key: String) -> Result<(), CommonError>;

    // Environment variable operations
    async fn add_environment_variable(&self, key: String, value: String)
    -> Result<(), CommonError>;
    async fn update_environment_variable(
        &self,
        key: String,
        value: String,
    ) -> Result<(), CommonError>;
    async fn remove_environment_variable(&self, key: String) -> Result<(), CommonError>;

    // Identity operations - API keys
    async fn add_api_key(&self, id: String, config: ApiKeyYamlConfig) -> Result<(), CommonError>;
    async fn remove_api_key(&self, id: String) -> Result<(), CommonError>;

    // Identity operations - STS configurations
    async fn add_sts_config(&self, id: String, config: StsConfigYaml) -> Result<(), CommonError>;
    async fn remove_sts_config(&self, id: String) -> Result<(), CommonError>;

    // Identity operations - User auth flow configurations
    async fn add_user_auth_flow(
        &self,
        id: String,
        config: UserAuthFlowYamlConfig,
    ) -> Result<(), CommonError>;
    async fn remove_user_auth_flow(&self, id: String) -> Result<(), CommonError>;

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
        mut guard: MutexGuard<'_, SomaAgentDefinition>,
    ) -> Result<(), CommonError> {
        // Reload from file first to ensure we preserve all existing fields
        // This prevents overwriting fields that exist in the file but aren't in the cached definition
        let file_definition = Self::load_agent_definition(self.path.clone())?;

        // Merge the file definition with our changes
        // Preserve fields from file that aren't being modified in guard
        if guard.encryption.is_none() && file_definition.encryption.is_some() {
            guard.encryption = file_definition.encryption.clone();
        }
        if guard.bridge.is_none() && file_definition.bridge.is_some() {
            guard.bridge = file_definition.bridge.clone();
        }

        // For secrets, merge: start with file secrets (if any), then apply guard's changes (guard overwrites file for same keys)
        match (&file_definition.secrets, &guard.secrets) {
            (Some(file_secrets), Some(guard_secrets)) => {
                // Both exist: merge (file first, then guard overwrites)
                let mut merged = file_secrets.clone();
                for (key, value) in guard_secrets {
                    merged.insert(key.clone(), value.clone());
                }
                guard.secrets = Some(merged);
            }
            (Some(file_secrets), None) => {
                // Only file has secrets: use file's
                guard.secrets = Some(file_secrets.clone());
            }
            (None, Some(_guard_secrets)) => {
                // Only guard has secrets: use guard's (already set)
            }
            (None, None) => {
                // Neither has secrets: keep None
            }
        }

        // For environment variables, merge similarly
        match (
            &file_definition.environment_variables,
            &guard.environment_variables,
        ) {
            (Some(file_env_vars), Some(guard_env_vars)) => {
                // Both exist: merge (file first, then guard overwrites)
                let mut merged = file_env_vars.clone();
                for (key, value) in guard_env_vars {
                    merged.insert(key.clone(), value.clone());
                }
                guard.environment_variables = Some(merged);
            }
            (Some(file_env_vars), None) => {
                // Only file has env vars: use file's
                guard.environment_variables = Some(file_env_vars.clone());
            }
            (None, Some(_guard_env_vars)) => {
                // Only guard has env vars: use guard's (already set)
            }
            (None, None) => {
                // Neither has env vars: keep None
            }
        }

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
            definition.bridge = Some(BridgeConfig {
                providers: None,
                mcp_servers: None,
            });
        }
    }

    fn ensure_identity_config(definition: &mut SomaAgentDefinition) {
        if definition.identity.is_none() {
            definition.identity = Some(IdentityConfig::default());
        }
    }
}

#[async_trait]
impl SomaAgentDefinitionLike for YamlSomaAgentDefinition {
    async fn reload(&self) -> Result<(), CommonError> {
        trace!(path = %self.path.display(), "Reloading soma definition");
        let definition = Self::load_agent_definition(self.path.clone())?;
        *self.cached_definition.lock().await = definition;
        trace!(path = %self.path.display(), "Soma definition reloaded");
        Ok(())
    }

    async fn get_definition(&self) -> Result<SomaAgentDefinition, CommonError> {
        trace!("Getting soma definition");
        let result = self.cached_definition.lock().await.clone();
        trace!("Retrieved soma definition");
        Ok(result)
    }

    async fn add_envelope_key(
        &self,
        key_id: String,
        config: EnvelopeKeyConfig,
    ) -> Result<(), CommonError> {
        trace!(key_id = %key_id, "Adding envelope key");
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
        self.save(definition).await?;
        trace!(key_id = %key_id, "Envelope key added");
        Ok(())
    }

    async fn remove_envelope_key(&self, key_id: String) -> Result<(), CommonError> {
        trace!(key_id = %key_id, "Removing envelope key");
        let mut definition = self.cached_definition.lock().await;

        if let Some(encryption) = &mut definition.encryption {
            if let Some(envelope_keys) = &mut encryption.envelope_keys {
                envelope_keys.remove(&key_id);
                self.save(definition).await?;
                trace!(key_id = %key_id, "Envelope key removed");
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
        trace!(envelope_key_id = %envelope_key_id, alias = %alias, "Adding DEK");
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

        self.save(definition).await?;
        trace!(envelope_key_id = %envelope_key_id, alias = %alias, "DEK added");
        Ok(())
    }

    async fn remove_dek(&self, envelope_key_id: String, alias: String) -> Result<(), CommonError> {
        trace!(envelope_key_id = %envelope_key_id, alias = %alias, "Removing DEK");
        let mut definition = self.cached_definition.lock().await;

        if let Some(encryption) = &mut definition.encryption {
            if let Some(envelope_keys) = &mut encryption.envelope_keys {
                if let Some(envelope_key) = envelope_keys.get_mut(&envelope_key_id) {
                    envelope_key.deks_mut().remove(&alias);
                    self.save(definition).await?;
                    trace!(envelope_key_id = %envelope_key_id, alias = %alias, "DEK removed");
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
        trace!(envelope_key_id = %envelope_key_id, old_key = %old_key, new_key = %new_key, "Renaming DEK");
        let mut definition = self.cached_definition.lock().await;

        if let Some(encryption) = &mut definition.encryption {
            if let Some(envelope_keys) = &mut encryption.envelope_keys {
                if let Some(envelope_key) = envelope_keys.get_mut(&envelope_key_id) {
                    let deks = envelope_key.deks_mut();
                    if let Some(dek_config) = deks.remove(&old_key) {
                        deks.insert(new_key.clone(), dek_config);
                        self.save(definition).await?;
                        trace!(envelope_key_id = %envelope_key_id, old_key = %old_key, new_key = %new_key, "DEK renamed");
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
        trace!(provider_id = %provider_id, "Adding provider");
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
        self.save(definition).await?;
        trace!(provider_id = %provider_id, "Provider added");
        Ok(())
    }

    async fn remove_provider(&self, provider_id: String) -> Result<(), CommonError> {
        trace!(provider_id = %provider_id, "Removing provider");
        let mut definition = self.cached_definition.lock().await;

        if let Some(bridge) = &mut definition.bridge {
            if let Some(providers) = &mut bridge.providers {
                providers.remove(&provider_id);
                self.save(definition).await?;
                trace!(provider_id = %provider_id, "Provider removed");
            }
        }
        Ok(())
    }

    async fn update_provider(
        &self,
        provider_id: String,
        config: ProviderConfig,
    ) -> Result<(), CommonError> {
        trace!(provider_id = %provider_id, "Updating provider");
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

        self.save(definition).await?;
        trace!(provider_id = %provider_id, "Provider updated");
        Ok(())
    }

    async fn add_function_instance(
        &self,
        provider_controller_type_id: String,
        function_controller_type_id: String,
        provider_instance_id: String,
    ) -> Result<(), CommonError> {
        trace!(
            provider_controller_type_id = %provider_controller_type_id,
            function_controller_type_id = %function_controller_type_id,
            provider_instance_id = %provider_instance_id,
            "Adding function instance"
        );
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
        self.save(definition).await?;
        trace!(
            provider_controller_type_id = %provider_controller_type_id,
            function_controller_type_id = %function_controller_type_id,
            provider_instance_id = %provider_instance_id,
            "Function instance added"
        );
        Ok(())
    }

    async fn remove_function_instance(
        &self,
        provider_controller_type_id: String,
        function_controller_type_id: String,
        provider_instance_id: String,
    ) -> Result<(), CommonError> {
        trace!(
            provider_controller_type_id = %provider_controller_type_id,
            function_controller_type_id = %function_controller_type_id,
            provider_instance_id = %provider_instance_id,
            "Removing function instance"
        );
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

        self.save(definition).await?;
        trace!(
            provider_controller_type_id = %provider_controller_type_id,
            function_controller_type_id = %function_controller_type_id,
            provider_instance_id = %provider_instance_id,
            "Function instance removed"
        );
        Ok(())
    }

    async fn add_mcp_server(
        &self,
        mcp_server_id: String,
        config: McpServerConfig,
    ) -> Result<(), CommonError> {
        trace!(mcp_server_id = %mcp_server_id, "Adding MCP server");
        let mut definition = self.cached_definition.lock().await;
        Self::ensure_bridge_config(&mut definition);

        let bridge = definition.bridge.as_mut().unwrap();
        if bridge.mcp_servers.is_none() {
            bridge.mcp_servers = Some(HashMap::new());
        }

        bridge
            .mcp_servers
            .as_mut()
            .unwrap()
            .insert(mcp_server_id.clone(), config);
        self.save(definition).await?;
        trace!(mcp_server_id = %mcp_server_id, "MCP server added");
        Ok(())
    }

    async fn update_mcp_server(
        &self,
        mcp_server_id: String,
        config: McpServerConfig,
    ) -> Result<(), CommonError> {
        trace!(mcp_server_id = %mcp_server_id, "Updating MCP server");
        let mut definition = self.cached_definition.lock().await;
        Self::ensure_bridge_config(&mut definition);

        let bridge = definition.bridge.as_mut().unwrap();
        if bridge.mcp_servers.is_none() {
            bridge.mcp_servers = Some(HashMap::new());
        }

        let mcp_servers = bridge.mcp_servers.as_mut().unwrap();
        match mcp_servers.get_mut(&mcp_server_id) {
            Some(existing_config) => {
                // Update name but preserve functions if not provided
                existing_config.name = config.name;
                if config.functions.is_some() {
                    existing_config.functions = config.functions;
                }
            }
            None => {
                mcp_servers.insert(mcp_server_id.clone(), config);
            }
        }

        self.save(definition).await?;
        trace!(mcp_server_id = %mcp_server_id, "MCP server updated");
        Ok(())
    }

    async fn remove_mcp_server(&self, mcp_server_id: String) -> Result<(), CommonError> {
        trace!(mcp_server_id = %mcp_server_id, "Removing MCP server");
        let mut definition = self.cached_definition.lock().await;

        if let Some(bridge) = &mut definition.bridge {
            if let Some(mcp_servers) = &mut bridge.mcp_servers {
                mcp_servers.remove(&mcp_server_id);
                self.save(definition).await?;
                trace!(mcp_server_id = %mcp_server_id, "MCP server removed");
            }
        }
        Ok(())
    }

    async fn add_mcp_server_function(
        &self,
        mcp_server_id: String,
        function_config: McpServerFunctionConfig,
    ) -> Result<(), CommonError> {
        trace!(
            mcp_server_id = %mcp_server_id,
            function_name = %function_config.function_name,
            "Adding MCP server function"
        );
        let mut definition = self.cached_definition.lock().await;
        let bridge = match &mut definition.bridge {
            Some(bridge) => bridge,
            None => {
                return Err(CommonError::Unknown(anyhow::anyhow!(
                    "Bridge configuration not found"
                )));
            }
        };
        let mcp_servers = match &mut bridge.mcp_servers {
            Some(mcp_servers) => mcp_servers,
            None => {
                return Err(CommonError::Unknown(anyhow::anyhow!(
                    "MCP servers not found"
                )));
            }
        };
        let mcp_server = match mcp_servers.get_mut(&mcp_server_id) {
            Some(mcp_server) => mcp_server,
            None => {
                return Err(CommonError::Unknown(anyhow::anyhow!(
                    "MCP server not found: {mcp_server_id}"
                )));
            }
        };
        if mcp_server.functions.is_none() {
            mcp_server.functions = Some(Vec::new());
        }
        let functions = mcp_server.functions.as_mut().unwrap();
        functions.push(function_config.clone());
        self.save(definition).await?;
        trace!(
            mcp_server_id = %mcp_server_id,
            function_name = %function_config.function_name,
            "MCP server function added"
        );
        Ok(())
    }

    async fn update_mcp_server_function(
        &self,
        mcp_server_id: String,
        function_config: McpServerFunctionConfig,
    ) -> Result<(), CommonError> {
        trace!(
            mcp_server_id = %mcp_server_id,
            function_name = %function_config.function_name,
            "Updating MCP server function"
        );
        let mut definition = self.cached_definition.lock().await;
        let bridge = match &mut definition.bridge {
            Some(bridge) => bridge,
            None => {
                return Err(CommonError::Unknown(anyhow::anyhow!(
                    "Bridge configuration not found"
                )));
            }
        };
        let mcp_servers = match &mut bridge.mcp_servers {
            Some(mcp_servers) => mcp_servers,
            None => {
                return Err(CommonError::Unknown(anyhow::anyhow!(
                    "MCP servers not found"
                )));
            }
        };
        let mcp_server = match mcp_servers.get_mut(&mcp_server_id) {
            Some(mcp_server) => mcp_server,
            None => {
                return Err(CommonError::Unknown(anyhow::anyhow!(
                    "MCP server not found: {mcp_server_id}"
                )));
            }
        };
        let functions = match &mut mcp_server.functions {
            Some(functions) => functions,
            None => {
                return Err(CommonError::Unknown(anyhow::anyhow!(
                    "No functions in MCP server"
                )));
            }
        };

        // Find and update the function
        if let Some(func) = functions.iter_mut().find(|f| {
            f.function_controller_type_id == function_config.function_controller_type_id
                && f.provider_controller_type_id == function_config.provider_controller_type_id
                && f.provider_instance_id == function_config.provider_instance_id
        }) {
            func.function_name = function_config.function_name.clone();
            func.function_description = function_config.function_description.clone();
        }

        self.save(definition).await?;
        trace!(
            mcp_server_id = %mcp_server_id,
            function_name = %function_config.function_name,
            "MCP server function updated"
        );
        Ok(())
    }

    async fn remove_mcp_server_function(
        &self,
        mcp_server_id: String,
        function_controller_type_id: String,
        provider_controller_type_id: String,
        provider_instance_id: String,
    ) -> Result<(), CommonError> {
        trace!(
            mcp_server_id = %mcp_server_id,
            function_controller_type_id = %function_controller_type_id,
            provider_controller_type_id = %provider_controller_type_id,
            provider_instance_id = %provider_instance_id,
            "Removing MCP server function"
        );
        let mut definition = self.cached_definition.lock().await;
        let bridge = match &mut definition.bridge {
            Some(bridge) => bridge,
            None => return Ok(()),
        };
        let mcp_servers = match &mut bridge.mcp_servers {
            Some(mcp_servers) => mcp_servers,
            None => return Ok(()),
        };
        let mcp_server = match mcp_servers.get_mut(&mcp_server_id) {
            Some(mcp_server) => mcp_server,
            None => return Ok(()),
        };
        let functions = match &mut mcp_server.functions {
            Some(functions) => functions,
            None => return Ok(()),
        };

        functions.retain(|f| {
            !(f.function_controller_type_id == function_controller_type_id
                && f.provider_controller_type_id == provider_controller_type_id
                && f.provider_instance_id == provider_instance_id)
        });

        self.save(definition).await?;
        trace!(
            mcp_server_id = %mcp_server_id,
            function_controller_type_id = %function_controller_type_id,
            provider_controller_type_id = %provider_controller_type_id,
            provider_instance_id = %provider_instance_id,
            "MCP server function removed"
        );
        Ok(())
    }

    async fn add_secret(&self, key: String, config: SecretConfig) -> Result<(), CommonError> {
        trace!(key = %key, "Adding secret");
        let mut definition = self.cached_definition.lock().await;

        if definition.secrets.is_none() {
            definition.secrets = Some(HashMap::new());
        }

        definition
            .secrets
            .as_mut()
            .unwrap()
            .insert(key.clone(), config);
        self.save(definition).await?;
        trace!(key = %key, "Secret added");
        Ok(())
    }

    async fn update_secret(&self, key: String, config: SecretConfig) -> Result<(), CommonError> {
        trace!(key = %key, "Updating secret");
        let mut definition = self.cached_definition.lock().await;

        if definition.secrets.is_none() {
            definition.secrets = Some(HashMap::new());
        }

        definition
            .secrets
            .as_mut()
            .unwrap()
            .insert(key.clone(), config);
        self.save(definition).await?;
        trace!(key = %key, "Secret updated");
        Ok(())
    }

    async fn remove_secret(&self, key: String) -> Result<(), CommonError> {
        trace!(key = %key, "Removing secret");
        let mut definition = self.cached_definition.lock().await;

        if let Some(secrets) = &mut definition.secrets {
            secrets.remove(&key);
            self.save(definition).await?;
            trace!(key = %key, "Secret removed");
        }
        Ok(())
    }

    async fn add_environment_variable(
        &self,
        key: String,
        value: String,
    ) -> Result<(), CommonError> {
        trace!(key = %key, "Adding environment variable");
        let mut definition = self.cached_definition.lock().await;

        if definition.environment_variables.is_none() {
            definition.environment_variables = Some(HashMap::new());
        }

        definition
            .environment_variables
            .as_mut()
            .unwrap()
            .insert(key.clone(), value);
        self.save(definition).await?;
        trace!(key = %key, "Environment variable added");
        Ok(())
    }

    async fn update_environment_variable(
        &self,
        key: String,
        value: String,
    ) -> Result<(), CommonError> {
        trace!(key = %key, "Updating environment variable");
        let mut definition = self.cached_definition.lock().await;

        if definition.environment_variables.is_none() {
            definition.environment_variables = Some(HashMap::new());
        }

        definition
            .environment_variables
            .as_mut()
            .unwrap()
            .insert(key.clone(), value);
        self.save(definition).await?;
        trace!(key = %key, "Environment variable updated");
        Ok(())
    }

    async fn remove_environment_variable(&self, key: String) -> Result<(), CommonError> {
        trace!(key = %key, "Removing environment variable");
        let mut definition = self.cached_definition.lock().await;

        if let Some(env_vars) = &mut definition.environment_variables {
            env_vars.remove(&key);
            self.save(definition).await?;
            trace!(key = %key, "Environment variable removed");
        }
        Ok(())
    }

    async fn add_api_key(&self, id: String, config: ApiKeyYamlConfig) -> Result<(), CommonError> {
        trace!(id = %id, "Adding API key");
        let mut definition = self.cached_definition.lock().await;
        Self::ensure_identity_config(&mut definition);

        let identity = definition.identity.as_mut().unwrap();
        if identity.api_keys.is_none() {
            identity.api_keys = Some(HashMap::new());
        }

        identity
            .api_keys
            .as_mut()
            .unwrap()
            .insert(id.clone(), config);
        self.save(definition).await?;
        trace!(id = %id, "API key added");
        Ok(())
    }

    async fn remove_api_key(&self, id: String) -> Result<(), CommonError> {
        trace!(id = %id, "Removing API key");
        let mut definition = self.cached_definition.lock().await;

        if let Some(identity) = &mut definition.identity {
            if let Some(api_keys) = &mut identity.api_keys {
                api_keys.remove(&id);
                self.save(definition).await?;
                trace!(id = %id, "API key removed");
            }
        }
        Ok(())
    }

    async fn add_sts_config(&self, id: String, config: StsConfigYaml) -> Result<(), CommonError> {
        trace!(id = %id, "Adding STS configuration");
        let mut definition = self.cached_definition.lock().await;
        Self::ensure_identity_config(&mut definition);

        let identity = definition.identity.as_mut().unwrap();
        if identity.sts_configurations.is_none() {
            identity.sts_configurations = Some(HashMap::new());
        }

        identity
            .sts_configurations
            .as_mut()
            .unwrap()
            .insert(id.clone(), config);
        self.save(definition).await?;
        trace!(id = %id, "STS configuration added");
        Ok(())
    }

    async fn remove_sts_config(&self, id: String) -> Result<(), CommonError> {
        trace!(id = %id, "Removing STS configuration");
        let mut definition = self.cached_definition.lock().await;

        if let Some(identity) = &mut definition.identity {
            if let Some(sts_configs) = &mut identity.sts_configurations {
                sts_configs.remove(&id);
                self.save(definition).await?;
                trace!(id = %id, "STS configuration removed");
            }
        }
        Ok(())
    }

    async fn add_user_auth_flow(
        &self,
        id: String,
        config: UserAuthFlowYamlConfig,
    ) -> Result<(), CommonError> {
        trace!(id = %id, "Adding user auth flow configuration");
        let mut definition = self.cached_definition.lock().await;
        Self::ensure_identity_config(&mut definition);

        let identity = definition.identity.as_mut().unwrap();
        if identity.user_auth_flows.is_none() {
            identity.user_auth_flows = Some(HashMap::new());
        }

        identity
            .user_auth_flows
            .as_mut()
            .unwrap()
            .insert(id.clone(), config);
        self.save(definition).await?;
        trace!(id = %id, "User auth flow configuration added");
        Ok(())
    }

    async fn remove_user_auth_flow(&self, id: String) -> Result<(), CommonError> {
        trace!(id = %id, "Removing user auth flow configuration");
        let mut definition = self.cached_definition.lock().await;

        if let Some(identity) = &mut definition.identity {
            if let Some(user_auth_flows) = &mut identity.user_auth_flows {
                user_auth_flows.remove(&id);
                self.save(definition).await?;
                trace!(id = %id, "User auth flow configuration removed");
            }
        }
        Ok(())
    }
}
