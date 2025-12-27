use async_trait::async_trait;
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::json;
use shared::{
    error::CommonError,
    primitives::{WrappedJsonValue, WrappedSchema},
};

use crate::logic::{
    ConfigurationSchema, EncryptionService, Metadata, ProviderCredentialControllerLike,
    ResourceServerCredentialLike, StaticCredentialConfigurationLike,
    StaticProviderCredentialControllerLike, UserCredentialLike,
};

// ============================================================================
// Static Credential Configuration
// ============================================================================

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct NoAuthStaticCredentialConfiguration {
    pub metadata: Metadata,
}

impl StaticCredentialConfigurationLike for NoAuthStaticCredentialConfiguration {
    fn type_id(&self) -> &'static str {
        "static_no_auth"
    }
    fn value(&self) -> WrappedJsonValue {
        WrappedJsonValue::new(json!(self))
    }
}

// ============================================================================
// Resource Server Credentials (Client Credentials)
// ============================================================================

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct NoAuthResourceServerCredential {
    #[serde(default)]
    #[schemars(skip)]
    pub metadata: Metadata,
}

impl ResourceServerCredentialLike for NoAuthResourceServerCredential {
    fn type_id(&self) -> &'static str {
        "resource_server_no_auth"
    }

    fn value(&self) -> WrappedJsonValue {
        WrappedJsonValue::new(json!(self))
    }
}

// ============================================================================
// User Credentials (Access Token + Refresh Token)
// ============================================================================

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct NoAuthUserCredential {
    #[serde(default)]
    #[schemars(skip)]
    pub metadata: Metadata,
}

impl UserCredentialLike for NoAuthUserCredential {
    fn type_id(&self) -> &'static str {
        "user_no_auth"
    }

    fn value(&self) -> WrappedJsonValue {
        WrappedJsonValue::new(json!(self))
    }
}

// ============================================================================
// No Auth Controller
// ============================================================================

const STATIC_TYPE_ID_NO_AUTH: &str = "no_auth";
pub struct NoAuthController {
    pub static_credentials: NoAuthStaticCredentialConfiguration,
}

impl StaticProviderCredentialControllerLike for NoAuthController {
    fn static_type_id() -> &'static str {
        STATIC_TYPE_ID_NO_AUTH
    }
}

#[async_trait]
impl ProviderCredentialControllerLike for NoAuthController {
    fn static_credentials(&self) -> &dyn StaticCredentialConfigurationLike {
        &self.static_credentials
    }

    fn type_id(&self) -> &'static str {
        STATIC_TYPE_ID_NO_AUTH
    }

    fn documentation(&self) -> &'static str {
        "No Auth - No authentication required"
    }

    fn name(&self) -> &'static str {
        "No Auth"
    }

    fn configuration_schema(&self) -> ConfigurationSchema {
        ConfigurationSchema {
            resource_server: WrappedSchema::new(schema_for!(NoAuthResourceServerCredential)),
            user_credential: WrappedSchema::new(schema_for!(NoAuthUserCredential)),
        }
    }

    async fn encrypt_resource_server_configuration(
        &self,
        _crypto_service: &EncryptionService,
        raw_resource_server_configuration: WrappedJsonValue,
    ) -> Result<Box<dyn ResourceServerCredentialLike>, CommonError> {
        // Parse the raw configuration
        let config: NoAuthResourceServerCredential =
            serde_json::from_value(raw_resource_server_configuration.into())?;

        Ok(Box::new(config))
    }

    async fn encrypt_user_credential_configuration(
        &self,
        _crypto_service: &EncryptionService,
        raw_user_credential_configuration: WrappedJsonValue,
    ) -> Result<Box<dyn UserCredentialLike>, CommonError> {
        // Parse the raw configuration
        let config: NoAuthUserCredential =
            serde_json::from_value(raw_user_credential_configuration.into())?;

        // Encrypt sensitive fields
        Ok(Box::new(config))
    }

    fn from_serialized_resource_server_configuration(
        &self,
        raw_resource_server_configuration: WrappedJsonValue,
    ) -> Result<(Box<dyn ResourceServerCredentialLike>, Metadata), CommonError> {
        let config: NoAuthResourceServerCredential =
            serde_json::from_value(raw_resource_server_configuration.into())?;

        let metadata = config.metadata.clone();
        Ok((Box::new(config), metadata))
    }

    fn from_serialized_user_credential_configuration(
        &self,
        raw_user_credential_configuration: WrappedJsonValue,
    ) -> Result<(Box<dyn UserCredentialLike>, Metadata), CommonError> {
        let config: NoAuthUserCredential =
            serde_json::from_value(raw_user_credential_configuration.into())?;

        let metadata = config.metadata.clone();
        Ok((Box::new(config), metadata))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
