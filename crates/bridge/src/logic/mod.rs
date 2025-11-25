pub mod controller;
pub mod credential;
pub mod credential_encryption;
pub mod instance;
pub mod mcp;

use std::sync::Arc;

use async_trait::async_trait;
// Re-export encryption types for use within the bridge crate
pub use ::encryption::logic::crypto_services::{DecryptionService, EncryptionService};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shared::{
    error::CommonError,
    primitives::{WrappedChronoDateTime, WrappedJsonValue, WrappedSchema},
};
use utoipa::ToSchema;

// Metadata must be defined before pub use statements so submodules can import it
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, ToSchema, PartialEq, Eq)]
#[serde(transparent)]
pub struct Metadata(pub serde_json::Map<String, serde_json::Value>);

impl Metadata {
    pub fn new() -> Self {
        Self(serde_json::Map::new())
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Self::new()
    }
}

impl TryInto<libsql::Value> for Metadata {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    fn try_into(self) -> Result<libsql::Value, Self::Error> {
        let json_value = serde_json::Value::Object(self.0);
        let json_string = serde_json::to_string(&json_value)?;
        Ok(libsql::Value::Text(json_string))
    }
}

impl TryFrom<libsql::Value> for Metadata {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    fn try_from(value: libsql::Value) -> Result<Self, Self::Error> {
        match value {
            libsql::Value::Text(s) => {
                let json_value: serde_json::Value = serde_json::from_str(&s)?;
                match json_value {
                    serde_json::Value::Object(map) => Ok(Metadata(map)),
                    _ => Err("Expected JSON object for Metadata".into()),
                }
            }
            _ => Err("Expected Text value for Metadata".into()),
        }
    }
}

impl libsql::FromValue for Metadata {
    fn from_sql(val: libsql::Value) -> libsql::Result<Self>
    where
        Self: Sized,
    {
        match val {
            libsql::Value::Text(s) => {
                let json_value: serde_json::Value =
                    serde_json::from_str(&s).map_err(|_e| libsql::Error::InvalidColumnType)?;
                match json_value {
                    serde_json::Value::Object(map) => Ok(Metadata(map)),
                    _ => Err(libsql::Error::InvalidColumnType),
                }
            }
            libsql::Value::Null => Err(libsql::Error::NullValue),
            _ => Err(libsql::Error::InvalidColumnType),
        }
    }
}

// Re-export commonly used types and functions
pub use controller::*;
pub use credential::*;
pub use credential_encryption::*;
pub use instance::*;

// on change events

#[derive(Clone, Debug)]
pub enum OnConfigChangeEvt {
    ProviderInstanceAdded(ProviderInstanceSerializedWithCredentials),
    ProviderInstanceRemoved(String),
    ProviderInstanceUpdated(ProviderInstanceSerializedWithCredentials),
    FunctionInstanceAdded(FunctionInstanceSerialized),
    FunctionInstanceRemoved(String, String, String), // (function_controller_type_id, provider_controller_type_id, provider_instance_id)
}

pub type OnConfigChangeTx = tokio::sync::broadcast::Sender<OnConfigChangeEvt>;
pub type OnConfigChangeRx = tokio::sync::broadcast::Receiver<OnConfigChangeEvt>;

pub trait StaticProviderCredentialControllerLike {
    fn static_type_id() -> &'static str;
}

#[async_trait]
pub trait ProviderCredentialControllerLike: Send + Sync {
    fn type_id(&self) -> &'static str;
    fn documentation(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn configuration_schema(&self) -> ConfigurationSchema;
    fn static_credentials(&self) -> &dyn StaticCredentialConfigurationLike;
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_rotateable_controller_resource_server_credential(
        &self,
    ) -> Option<&dyn RotateableControllerResourceServerCredentialLike> {
        None
    }
    fn as_rotateable_controller_user_credential(
        &self,
    ) -> Option<&dyn RotateableControllerUserCredentialLike> {
        None
    }
    fn as_user_credential_broker(&self) -> Option<&dyn UserCredentialBrokerLike> {
        None
    }
    // TODO: need to pass in the encryption provider here to do the actual encryption
    async fn encrypt_resource_server_configuration(
        &self,
        crypto_service: &EncryptionService,
        raw_resource_server_configuration: WrappedJsonValue,
    ) -> Result<Box<dyn ResourceServerCredentialLike>, CommonError>;
    async fn encrypt_user_credential_configuration(
        &self,
        crypto_service: &EncryptionService,
        raw_user_credential_configuration: WrappedJsonValue,
    ) -> Result<Box<dyn UserCredentialLike>, CommonError>;

    // NOTE: serialized values are always already encrypted
    #[allow(clippy::wrong_self_convention)]
    fn from_serialized_resource_server_configuration(
        &self,
        raw_resource_server_configuration: WrappedJsonValue,
    ) -> Result<(Box<dyn ResourceServerCredentialLike>, Metadata), CommonError>;

    #[allow(clippy::wrong_self_convention)]
    fn from_serialized_user_credential_configuration(
        &self,
        raw_user_credential_configuration: WrappedJsonValue,
    ) -> Result<(Box<dyn UserCredentialLike>, Metadata), CommonError>;
}

#[async_trait]
pub trait ProviderControllerLike: Send + Sync {
    fn type_id(&self) -> String;
    fn documentation(&self) -> String;
    fn name(&self) -> String;
    fn categories(&self) -> Vec<String>;
    fn functions(&self) -> Vec<Arc<dyn FunctionControllerLike>>;
    fn credential_controllers(&self) -> Vec<Arc<dyn ProviderCredentialControllerLike>>;
    fn metadata(&self) -> Metadata;
}

pub trait ProviderInstanceLike {
    fn provider_controller_type_id(&self) -> String;
    fn type_id(&self) -> String;
    fn credential_controller_type_id(&self) -> String;

    fn static_credential_value(&self) -> WrappedJsonValue;
    fn resource_server_credential_value(&self) -> WrappedJsonValue;
    fn user_credential_value(&self) -> WrappedJsonValue;
}

#[async_trait]
pub trait RotateableControllerResourceServerCredentialLike {
    async fn rotate_resource_server_credential(
        &self,
        decryption_service: &DecryptionService,
        encryption_service: &EncryptionService,
        static_credentials: &dyn StaticCredentialConfigurationLike,
        resource_server_cred: &ResourceServerCredentialSerialized,
    ) -> Result<ResourceServerCredentialSerialized, CommonError>;
    fn next_resource_server_credential_rotation_time(
        &self,
        decryption_service: &DecryptionService,
        encryption_service: &EncryptionService,
        static_credentials: &dyn StaticCredentialConfigurationLike,
        resource_server_cred: &ResourceServerCredentialSerialized,
    ) -> Result<WrappedChronoDateTime, CommonError>;
}

#[async_trait]
pub trait RotateableControllerUserCredentialLike {
    async fn rotate_user_credential(
        &self,
        decryption_service: &DecryptionService,
        encryption_service: &EncryptionService,
        static_credentials: &dyn StaticCredentialConfigurationLike,
        resource_server_cred: &ResourceServerCredentialSerialized,
        user_cred: &UserCredentialSerialized,
    ) -> Result<UserCredentialSerialized, CommonError>;
    async fn next_user_credential_rotation_time(
        &self,
        static_credentials: &dyn StaticCredentialConfigurationLike,
        resource_server_cred: &ResourceServerCredentialSerialized,
        user_cred: &UserCredentialSerialized,
        decryption_service: &DecryptionService,
        encryption_service: &EncryptionService,
    ) -> Result<WrappedChronoDateTime, CommonError>;
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct InvokeError {
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InvokeResult {
    Success(WrappedJsonValue),
    Error(InvokeError),
}

#[async_trait]
pub trait FunctionControllerLike: Send + Sync {
    fn type_id(&self) -> String;
    fn name(&self) -> String;
    fn documentation(&self) -> String;
    fn parameters(&self) -> WrappedSchema;
    fn output(&self) -> WrappedSchema;
    fn categories(&self) -> Vec<String>;
    async fn invoke(
        &self,
        crypto_service: &DecryptionService,
        credential_controller: &Arc<dyn ProviderCredentialControllerLike>,
        static_credentials: &dyn StaticCredentialConfigurationLike,
        resource_server_credential: &ResourceServerCredentialSerialized,
        user_credential: &UserCredentialSerialized,
        params: WrappedJsonValue,
    ) -> Result<InvokeResult, CommonError>;
}
