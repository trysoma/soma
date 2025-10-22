
pub mod controller;
pub mod encryption;
pub mod instance;
pub mod credential;
pub mod mcp;

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, OsRng},
};
use async_trait::async_trait;
use base64::Engine;
use enum_dispatch::enum_dispatch;
use once_cell::sync::Lazy;
use rand::RngCore;
use reqwest::Request;
use schemars::{JsonSchema, Schema};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::json;
use sha2::{Digest, Sha256};
use shared::{
    error::CommonError,
    primitives::{
        PaginatedResponse, PaginationRequest, WrappedChronoDateTime, WrappedJsonValue,
        WrappedSchema, WrappedUuidV4,
    },
};
use std::fs;
use std::path::Path;
use std::sync::RwLock;
use utoipa::ToSchema;

// Metadata must be defined before pub use statements so submodules can import it
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, ToSchema)]
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
pub use encryption::*;
pub use instance::*;

use crate::{
    logic::instance::{FunctionInstanceSerialized, ProviderInstanceSerializedWithCredentials}, providers::google_mail::GoogleMailProviderController, repository::ProviderRepositoryLike
};
use encryption::*;
use credential::*;

// on change events

pub enum OnConfigChangeEvt {
    DataEncryptionKeyAdded(DataEncryptionKey),
    DataEncryptionKeyRemoved(String),
    ProviderInstanceAdded(ProviderInstanceSerializedWithCredentials),
    ProviderInstanceRemoved(String),
    FunctionInstanceAdded(FunctionInstanceSerialized),
    FunctionInstanceRemoved(String, String, String), // (function_controller_type_id, provider_controller_type_id, provider_instance_id)
}

pub type OnConfigChangeTx = tokio::sync::mpsc::Sender<OnConfigChangeEvt>;
pub type OnConfigChangeRx = tokio::sync::mpsc::Receiver<OnConfigChangeEvt>;

pub trait StaticProviderCredentialControllerLike {
    fn static_type_id() -> &'static str;
}

#[async_trait]
pub trait ProviderCredentialControllerLike: Send + Sync {
    fn type_id(&self) -> &'static str;
    fn documentation(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn configuration_schema(&self) -> ConfigurationSchema;
    fn static_credentials(&self) -> Box<dyn StaticCredentialConfigurationLike>;
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
    fn from_serialized_resource_server_configuration(
        &self,
        raw_resource_server_configuration: WrappedJsonValue,
    ) -> Result<(Box<dyn ResourceServerCredentialLike>, Metadata), CommonError>;
    fn from_serialized_user_credential_configuration(
        &self,
        raw_user_credential_configuration: WrappedJsonValue,
    ) -> Result<(Box<dyn UserCredentialLike>, Metadata), CommonError>;
}

#[async_trait]
pub trait ProviderControllerLike: Send + Sync {
    fn type_id(&self) -> &'static str;
    fn documentation(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn categories(&self) -> Vec<&'static str>;
    fn functions(&self) -> Vec<Arc<dyn FunctionControllerLike>>;
    fn credential_controllers(&self) -> Vec<Arc<dyn ProviderCredentialControllerLike>>;
}

pub trait ProviderInstanceLike {
    fn provider_controller_type_id(&self) -> &'static str;
    fn type_id(&self) -> &'static str;
    fn credential_controller_type_id(&self) -> &'static str;

    fn static_credential_value(&self) -> WrappedJsonValue;
    fn resource_server_credential_value(&self) -> WrappedJsonValue;
    fn user_credential_value(&self) -> WrappedJsonValue;
}

#[async_trait]
pub trait RotateableControllerResourceServerCredentialLike {
    async fn rotate_resource_server_credential(
        &self,
        resource_server_cred: &Credential<Arc<dyn ResourceServerCredentialLike>>,
    ) -> Result<Credential<Arc<dyn ResourceServerCredentialLike>>, CommonError>;
    fn next_resource_server_credential_rotation_time(
        &self,
        resource_server_cred: &Credential<Arc<dyn ResourceServerCredentialLike>>,
    ) -> WrappedChronoDateTime;
}

#[async_trait]
pub trait RotateableControllerUserCredentialLike {
    async fn rotate_user_credential(
        &self,
        resource_server_cred: &ResourceServerCredential,
        user_cred: &Credential<Arc<dyn UserCredentialLike>>,
    ) -> Result<Credential<Arc<dyn UserCredentialLike>>, CommonError>;
    async fn next_user_credential_rotation_time(
        &self,
        resource_server_cred: &ResourceServerCredential,
        user_cred: &Credential<Arc<dyn UserCredentialLike>>,
    ) -> WrappedChronoDateTime;
}

#[async_trait]
pub trait FunctionControllerLike: Send + Sync {
    fn type_id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn documentation(&self) -> &'static str;
    fn parameters(&self) -> WrappedSchema;
    fn output(&self) -> WrappedSchema;
    fn categories(&self) -> Vec<&'static str>;
    async fn invoke(
        &self,
        crypto_service: &DecryptionService,
        credential_controller: &Arc<dyn ProviderCredentialControllerLike>,
        static_credentials: &Box<dyn StaticCredentialConfigurationLike>,
        resource_server_credential: &ResourceServerCredentialSerialized,
        user_credential: &UserCredentialSerialized,
        params: WrappedJsonValue,
    ) -> Result<WrappedJsonValue, CommonError>;
}
