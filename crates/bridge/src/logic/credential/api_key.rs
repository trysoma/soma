use async_trait::async_trait;
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::json;
use shared::{
    error::CommonError,
    primitives::{WrappedJsonValue, WrappedSchema},
};

use crate::logic::{
    ConfigurationSchema, Metadata, ProviderCredentialControllerLike, ResourceServerCredentialLike,
    ResourceServerCredentialSerialized, StaticCredentialConfigurationLike,
    StaticProviderCredentialControllerLike, UserCredentialLike, schemars_make_password,
};
use ::encryption::logic::crypto_services::{DecryptionService, EncryptedString, EncryptionService};

// ============================================================================
// Static Credential Configuration
// ============================================================================

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct ApiKeyStaticCredentialConfiguration {
    pub metadata: Metadata,
}

impl StaticCredentialConfigurationLike for ApiKeyStaticCredentialConfiguration {
    fn type_id(&self) -> &'static str {
        "static_api_key"
    }
    fn value(&self) -> WrappedJsonValue {
        WrappedJsonValue::new(json!(self))
    }
}

// ============================================================================
// Resource Server Credentials (Client Credentials)
// ============================================================================

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct ApiKeyResourceServerCredential {
    #[schemars(transform = schemars_make_password)]
    pub api_key: EncryptedString,
    #[serde(default)]
    #[schemars(skip)]
    pub metadata: Metadata,
}

impl ResourceServerCredentialLike for ApiKeyResourceServerCredential {
    fn type_id(&self) -> &'static str {
        "resource_server_api_key"
    }

    fn value(&self) -> WrappedJsonValue {
        WrappedJsonValue::new(json!(self))
    }
}

// ============================================================================
// User Credentials (Access Token + Refresh Token)
// ============================================================================

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct ApiKeyUserCredential {
    #[serde(default)]
    #[schemars(skip)]
    pub metadata: Metadata,
}

impl UserCredentialLike for ApiKeyUserCredential {
    fn type_id(&self) -> &'static str {
        "user_api_key"
    }

    fn value(&self) -> WrappedJsonValue {
        WrappedJsonValue::new(json!(self))
    }
}

pub struct DecryptedApiKeyCredentials {
    pub api_key: String,
    pub metadata: Metadata,
}

// ============================================================================
// OAuth Authorization Code Flow Controller
// ============================================================================

pub struct ApiKeyController {
    pub static_credentials: ApiKeyStaticCredentialConfiguration,
}

const STATIC_TYPE_ID_API_KEY: &str = "api_key";

impl ApiKeyController {
    pub async fn decrypt_api_key_credentials(
        &self,
        crypto_service: &DecryptionService,
        resource_server_cred: &ResourceServerCredentialSerialized,
    ) -> Result<DecryptedApiKeyCredentials, CommonError> {
        let typed_creds: ApiKeyResourceServerCredential =
            serde_json::from_value(resource_server_cred.value.clone().into())?;
        let decrypted_creds: DecryptedApiKeyCredentials = DecryptedApiKeyCredentials {
            api_key: crypto_service.decrypt_data(typed_creds.api_key).await?,
            metadata: typed_creds.metadata,
        };

        Ok(decrypted_creds)
    }
}

impl StaticProviderCredentialControllerLike for ApiKeyController {
    fn static_type_id() -> &'static str {
        STATIC_TYPE_ID_API_KEY
    }
}

#[async_trait]
impl ProviderCredentialControllerLike for ApiKeyController {
    fn static_credentials(&self) -> &dyn StaticCredentialConfigurationLike {
        &self.static_credentials
    }

    fn type_id(&self) -> &'static str {
        STATIC_TYPE_ID_API_KEY
    }

    fn documentation(&self) -> &'static str {
        "API Key - API key authentication"
    }

    fn name(&self) -> &'static str {
        "API Key"
    }

    fn configuration_schema(&self) -> ConfigurationSchema {
        ConfigurationSchema {
            resource_server: WrappedSchema::new(schema_for!(ApiKeyResourceServerCredential)),
            user_credential: WrappedSchema::new(schema_for!(ApiKeyUserCredential)),
        }
    }

    async fn encrypt_resource_server_configuration(
        &self,
        crypto_service: &EncryptionService,
        raw_resource_server_configuration: WrappedJsonValue,
    ) -> Result<Box<dyn ResourceServerCredentialLike>, CommonError> {
        // Parse the raw configuration
        let mut config: ApiKeyResourceServerCredential =
            serde_json::from_value(raw_resource_server_configuration.into())?;

        // Encrypt the client secret
        config.api_key = EncryptedString(crypto_service.encrypt_data(config.api_key.0).await?.0);

        Ok(Box::new(config))
    }

    async fn encrypt_user_credential_configuration(
        &self,
        _crypto_service: &EncryptionService,
        raw_user_credential_configuration: WrappedJsonValue,
    ) -> Result<Box<dyn UserCredentialLike>, CommonError> {
        // Parse the raw configuration
        let config: ApiKeyUserCredential =
            serde_json::from_value(raw_user_credential_configuration.into())?;

        Ok(Box::new(config))
    }

    fn from_serialized_resource_server_configuration(
        &self,
        raw_resource_server_configuration: WrappedJsonValue,
    ) -> Result<(Box<dyn ResourceServerCredentialLike>, Metadata), CommonError> {
        let config: ApiKeyResourceServerCredential =
            serde_json::from_value(raw_resource_server_configuration.into())?;

        let metadata = config.metadata.clone();
        Ok((Box::new(config), metadata))
    }

    fn from_serialized_user_credential_configuration(
        &self,
        raw_user_credential_configuration: WrappedJsonValue,
    ) -> Result<(Box<dyn UserCredentialLike>, Metadata), CommonError> {
        let config: ApiKeyUserCredential =
            serde_json::from_value(raw_user_credential_configuration.into())?;

        let metadata = config.metadata.clone();
        Ok((Box::new(config), metadata))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
