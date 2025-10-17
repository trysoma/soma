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

use crate::{
    providers::google_mail::GoogleMailProviderController, repository::ProviderRepositoryLike,
};

// on change events

pub enum OnConfigChangeEvt {
    DataEncryptionKeyAdded(DataEncryptionKey),
    DataEncryptionKeyRemoved(String),
    ProviderInstanceAdded(ProviderInstanceSerializedWithCredentials),
    ProviderInstanceRemoved(String),
    FunctionInstanceAdded(FunctionInstanceSerialized),
    FunctionInstanceRemoved(String),
}

pub type OnConfigChangeTx = tokio::sync::mpsc::Sender<OnConfigChangeEvt>;
pub type OnConfigChangeRx = tokio::sync::mpsc::Receiver<OnConfigChangeEvt>;

// encrpyion

// encrpyion
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, ToSchema)]
#[serde(transparent)]
pub struct EncryptedDataEncryptionKey(pub String);

#[derive(Debug, Clone, zeroize::Zeroize, zeroize::ZeroizeOnDrop)]
pub struct DecryptedDataEnvelopeKey(pub Vec<u8>);

impl TryInto<libsql::Value> for EncryptedDataEncryptionKey {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    fn try_into(self) -> Result<libsql::Value, Self::Error> {
        Ok(libsql::Value::Text(self.0))
    }
}

impl TryFrom<libsql::Value> for EncryptedDataEncryptionKey {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    fn try_from(value: libsql::Value) -> Result<Self, Self::Error> {
        match value {
            libsql::Value::Text(s) => Ok(EncryptedDataEncryptionKey(s)),
            _ => Err("Expected Text value for EncryptedDataEncryptionKey".into()),
        }
    }
}

impl libsql::FromValue for EncryptedDataEncryptionKey {
    fn from_sql(val: libsql::Value) -> libsql::Result<Self>
    where
        Self: Sized,
    {
        match val {
            libsql::Value::Text(s) => Ok(EncryptedDataEncryptionKey(s)),
            libsql::Value::Null => Err(libsql::Error::NullValue),
            _ => Err(libsql::Error::InvalidColumnType),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, ToSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum EnvelopeEncryptionKeyId {
    AwsKms { arn: String },
    Local { key_id: String },
}

#[derive(Clone, zeroize::Zeroize, zeroize::ZeroizeOnDrop)]
pub enum EnvelopeEncryptionKeyContents {
    AwsKms { arn: String },
    Local { key_id: String, key_bytes: Vec<u8> },
}

impl From<EnvelopeEncryptionKeyContents> for EnvelopeEncryptionKeyId {
    fn from(contents: EnvelopeEncryptionKeyContents) -> Self {
        match &contents {
            EnvelopeEncryptionKeyContents::AwsKms { arn } => {
                EnvelopeEncryptionKeyId::AwsKms { arn: arn.clone() }
            }
            EnvelopeEncryptionKeyContents::Local {
                key_id,
                key_bytes: _,
            } => EnvelopeEncryptionKeyId::Local {
                key_id: key_id.clone(),
            },
        }
    }
}

impl TryInto<libsql::Value> for EnvelopeEncryptionKeyId {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    fn try_into(self) -> Result<libsql::Value, Self::Error> {
        let json_value = serde_json::to_value(&self)?;
        let json_string = serde_json::to_string(&json_value)?;
        Ok(libsql::Value::Text(json_string))
    }
}

impl TryFrom<libsql::Value> for EnvelopeEncryptionKeyId {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    fn try_from(value: libsql::Value) -> Result<Self, Self::Error> {
        match value {
            libsql::Value::Text(s) => {
                let json_value: EnvelopeEncryptionKeyId = serde_json::from_str(&s)?;
                Ok(json_value)
            }
            _ => Err("Expected Text value for EnvelopeEncryptionKeyId".into()),
        }
    }
}

impl libsql::FromValue for EnvelopeEncryptionKeyId {
    fn from_sql(val: libsql::Value) -> libsql::Result<Self>
    where
        Self: Sized,
    {
        match val {
            libsql::Value::Text(s) => {
                let json_value: EnvelopeEncryptionKeyId =
                    serde_json::from_str(&s).map_err(|_e| libsql::Error::InvalidColumnType)?;
                Ok(json_value)
            }
            libsql::Value::Null => Err(libsql::Error::NullValue),
            _ => Err(libsql::Error::InvalidColumnType),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct DataEncryptionKey {
    pub id: String,
    pub envelope_encryption_key_id: EnvelopeEncryptionKeyId,
    pub encrypted_data_encryption_key: EncryptedDataEncryptionKey,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct DataEncryptionKeyListItem {
    pub id: String,
    pub envelope_encryption_key_id: EnvelopeEncryptionKeyId,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

#[derive(Serialize, Deserialize, Clone, JsonSchema, ToSchema)]
#[serde(transparent)]
pub struct EncryptedString(pub String);

#[derive(Clone)]
pub struct CryptoService {
    pub data_encryption_key: DataEncryptionKey,
    cached_decrypted_data_envelope_key: DecryptedDataEnvelopeKey,
}

impl CryptoService {
    pub async fn new(
        envelope_encryption_key_contents: EnvelopeEncryptionKeyContents,
        data_encryption_key: DataEncryptionKey,
    ) -> Result<Self, CommonError> {
        let mut envelop_key_match = false;

        if let EnvelopeEncryptionKeyContents::Local { key_id, key_bytes } =
            &envelope_encryption_key_contents
            && let EnvelopeEncryptionKeyId::Local {
                key_id: data_encryption_key_id,
                ..
            } = &data_encryption_key.envelope_encryption_key_id
        {
            envelop_key_match = key_id == data_encryption_key_id;
        } else if let EnvelopeEncryptionKeyContents::AwsKms { arn } =
            &envelope_encryption_key_contents
            && let EnvelopeEncryptionKeyId::AwsKms {
                arn: data_encryption_key_arn,
                ..
            } = &data_encryption_key.envelope_encryption_key_id
        {
            envelop_key_match = arn == data_encryption_key_arn;
        }

        if !envelop_key_match {
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "Envelope encryption key contents do not match data encryption key"
            )));
        }

        let decrypted_data_envelope_key = decrypt_data_envelope_key(
            &envelope_encryption_key_contents,
            &data_encryption_key.encrypted_data_encryption_key,
        )
        .await?;
        Ok(Self {
            data_encryption_key,
            cached_decrypted_data_envelope_key: decrypted_data_envelope_key,
        })
    }
}

pub struct EncryptionService(CryptoService);

impl EncryptionService {
    pub fn new(crypto_service: CryptoService) -> Self {
        Self(crypto_service)
    }

    pub async fn encrypt_data(&self, data: String) -> Result<EncryptedString, CommonError> {
        use aes_gcm::{
            Aes256Gcm, Nonce,
            aead::{Aead, KeyInit, OsRng},
        };
        use rand::RngCore;

        // Get the decrypted data envelope key as bytes (already Vec<u8>)
        let key_bytes = &self.0.cached_decrypted_data_envelope_key.0;
        if key_bytes.len() != 32 {
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "Invalid key length: expected 32 bytes for AES-256, got {}",
                key_bytes.len()
            )));
        }

        // Create AES-256-GCM cipher
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(key_bytes);
        let cipher = Aes256Gcm::new(key);

        // Generate a random 96-bit (12-byte) nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt the data
        let ciphertext = cipher
            .encrypt(nonce, data.as_bytes())
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Encryption failed: {}", e)))?;

        // Prepend the nonce to the ciphertext: [nonce (12 bytes) | ciphertext]
        let mut result = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        // Base64 encode the result
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &result);
        Ok(EncryptedString(encoded))
    }
}

pub struct DecryptionService(CryptoService);

impl DecryptionService {
    pub fn new(crypto_service: CryptoService) -> Self {
        Self(crypto_service)
    }

    pub async fn decrypt_data(&self, data: EncryptedString) -> Result<String, CommonError> {
        use aes_gcm::{
            Aes256Gcm, Nonce,
            aead::{Aead, KeyInit},
        };

        // Base64 decode the input
        let encrypted_data =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &data.0).map_err(
                |e| CommonError::Unknown(anyhow::anyhow!("Failed to decode base64: {}", e)),
            )?;

        // Ensure we have at least the nonce (12 bytes)
        if encrypted_data.len() < 12 {
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "Invalid encrypted data: too short (expected at least 12 bytes for nonce)"
            )));
        }

        // Extract the nonce (first 12 bytes)
        let nonce = Nonce::from_slice(&encrypted_data[..12]);

        // Extract the ciphertext (remaining bytes)
        let ciphertext = &encrypted_data[12..];

        // Get the decrypted data envelope key as bytes (already Vec<u8>)
        let key_bytes = &self.0.cached_decrypted_data_envelope_key.0;
        if key_bytes.len() != 32 {
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "Invalid key length: expected 32 bytes for AES-256, got {}",
                key_bytes.len()
            )));
        }

        // Create AES-256-GCM cipher
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(key_bytes);
        let cipher = Aes256Gcm::new(key);

        // Decrypt the ciphertext
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Decryption failed: {}", e)))?;

        // Convert to UTF-8 string
        let result = String::from_utf8(plaintext).map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!("Invalid UTF-8 in decrypted data: {}", e))
        })?;

        Ok(result)
    }
}

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

#[derive(Serialize, Deserialize, Clone)]
pub struct Credential<T> {
    pub inner: T,
    pub metadata: Metadata,
    pub id: WrappedUuidV4,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

pub trait RotateableCredentialLike: Send + Sync {
    fn next_rotation_time(&self) -> WrappedChronoDateTime;
}

// Static credential configurations

pub trait StaticCredentialConfigurationLike: Send + Sync {
    fn type_id(&self) -> &'static str;
    fn value(&self) -> WrappedJsonValue;
    fn as_rotateable_credential(&self) -> Option<&dyn RotateableCredentialLike> {
        None
    }
}

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct NoAuthStaticCredentialConfiguration {
    pub metadata: Metadata,
}

impl StaticCredentialConfigurationLike for NoAuthStaticCredentialConfiguration {
    fn type_id(&self) -> &'static str {
        "static_no_auth"
    }
    fn value(&self) -> WrappedJsonValue {
        WrappedJsonValue::new(serde_json::Value::Object(serde_json::Map::new()))
    }
}

// pub type StaticCredential = Credential<Arc<dyn StaticCredentialConfigurationLike>>;

// Resource server credentials

pub trait ResourceServerCredentialLike: Send + Sync {
    fn type_id(&self) -> &'static str;
    fn value(&self) -> WrappedJsonValue;
    fn as_rotateable_credential(&self) -> Option<&dyn RotateableCredentialLike> {
        None
    }
}

pub type ResourceServerCredential = Credential<Arc<dyn ResourceServerCredentialLike>>;

#[derive(Serialize, Deserialize, Clone)]
pub struct NoAuthResourceServerCredential {
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

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct Oauth2AuthorizationCodeFlowResourceServerCredential {
    pub client_id: String,
    pub client_secret: EncryptedString,
    pub redirect_uri: String,
    pub metadata: Metadata,
}

impl ResourceServerCredentialLike for Oauth2AuthorizationCodeFlowResourceServerCredential {
    fn type_id(&self) -> &'static str {
        "resource_server_oauth2_authorization_code_flow"
    }
    fn value(&self) -> WrappedJsonValue {
        WrappedJsonValue::new(json!(self))
    }
}

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct Oauth2JwtBearerAssertionFlowResourceServerCredential {
    pub client_id: String,
    pub client_secret: EncryptedString,
    pub redirect_uri: String,
    pub metadata: Metadata,
}

impl ResourceServerCredentialLike for Oauth2JwtBearerAssertionFlowResourceServerCredential {
    fn type_id(&self) -> &'static str {
        "resource_server_oauth2_jwt_bearer_assertion_flow"
    }
    fn value(&self) -> WrappedJsonValue {
        WrappedJsonValue::new(json!(self))
    }
}

// user credentials

pub trait UserCredentialLike: Send + Sync {
    fn type_id(&self) -> &'static str;
    fn value(&self) -> WrappedJsonValue;
    fn as_rotateable_credential(&self) -> Option<&dyn RotateableCredentialLike> {
        None
    }
}

pub type UserCredential = Credential<Arc<dyn UserCredentialLike>>;

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct Oauth2AuthorizationCodeFlowUserCredential {
    pub code: EncryptedString,
    pub access_token: EncryptedString,
    pub refresh_token: EncryptedString,
    pub expiry_time: WrappedChronoDateTime,
    pub sub: String,
    pub metadata: Metadata,
}

impl UserCredentialLike for Oauth2AuthorizationCodeFlowUserCredential {
    fn type_id(&self) -> &'static str {
        "oauth2_authorization_code_flow"
    }
    fn value(&self) -> WrappedJsonValue {
        WrappedJsonValue::new(json!(self))
    }
}

impl RotateableCredentialLike for Oauth2AuthorizationCodeFlowUserCredential {
    fn next_rotation_time(&self) -> WrappedChronoDateTime {
        self.expiry_time
    }
}

// User credentials
#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct NoAuthUserCredential {
    pub metadata: Metadata,
}

impl UserCredentialLike for NoAuthUserCredential {
    fn type_id(&self) -> &'static str {
        "no_auth"
    }
    fn value(&self) -> WrappedJsonValue {
        WrappedJsonValue::new(json!(self))
    }
}

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct Oauth2JwtBearerAssertionFlowUserCredential {
    pub assertion: String,
    pub token: String,
    pub expiry_time: WrappedChronoDateTime,
    pub sub: String,
    pub metadata: Metadata,
}

impl UserCredentialLike for Oauth2JwtBearerAssertionFlowUserCredential {
    fn type_id(&self) -> &'static str {
        "oauth2_jwt_bearer_assertion_flow"
    }
    fn value(&self) -> WrappedJsonValue {
        WrappedJsonValue::new(json!(self))
    }
}

impl RotateableCredentialLike for Oauth2JwtBearerAssertionFlowUserCredential {
    fn next_rotation_time(&self) -> WrappedChronoDateTime {
        self.expiry_time
    }
}

// Brokering user credentials

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct BrokerState {
    pub id: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
    pub resource_server_cred_id: WrappedUuidV4,
    pub provider_controller_type_id: String,
    pub credential_controller_type_id: String,
    pub metadata: Metadata,
    pub action: BrokerAction,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub enum BrokerAction {
    Redirect { url: String },
    None,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BrokerInput {
    Oauth2AuthorizationCodeFlow { code: String },
    Oauth2AuthorizationCodeFlowWithPkce { code: String, code_verifier: String },
}

pub enum BrokerOutcome {
    Success {
        user_credential: Box<dyn UserCredentialLike>,
        metadata: Metadata,
    },
    Continue {
        metadata: Metadata,
    },
}

#[async_trait]
pub trait UserCredentialBrokerLike: Send + Sync {
    /// Called when the user (or system) initiates credential brokering.
    async fn start(
        &self,
        resource_server_cred: &Credential<Box<dyn ResourceServerCredentialLike>>,
    ) -> Result<(BrokerAction, BrokerOutcome), CommonError>;

    /// Called after an external event (callback, redirect, etc.) returns data to the platform.
    async fn resume(
        &self,
        state: &BrokerState,
        input: BrokerInput,
    ) -> Result<(BrokerAction, BrokerOutcome), CommonError>;
}

#[derive(Serialize, Deserialize, Clone, ToSchema, JsonSchema)]
pub struct ConfigurationSchema {
    pub resource_server: WrappedSchema,
    pub user_credential: WrappedSchema,
}

// #[derive(Serialize, Deserialize, Clone, ToSchema, JsonSchema)]
// #[serde(transparent)]
// pub struct ConfigurationSchema(pub HashMap<String, ConfigurationSchemaItem>);

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

// serialized versions of the above types

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct StaticCredentialSerialized {
    // not UUID as some ID's will be deterministic
    pub type_id: String,
    pub metadata: Metadata,

    // this is the serialized version of the actual configuration fields
    pub value: WrappedJsonValue,
}

impl From<Credential<Arc<dyn StaticCredentialConfigurationLike>>> for StaticCredentialSerialized {
    fn from(static_cred: Credential<Arc<dyn StaticCredentialConfigurationLike>>) -> Self {
        StaticCredentialSerialized {
            type_id: static_cred.inner.type_id().to_string(),
            metadata: static_cred.metadata.clone(),
            value: static_cred.inner.value(),
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct ResourceServerCredentialSerialized {
    pub id: WrappedUuidV4,
    pub type_id: String,
    pub metadata: Metadata,
    pub value: WrappedJsonValue,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
    pub next_rotation_time: Option<WrappedChronoDateTime>,
    pub data_encryption_key_id: String,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct UserCredentialSerialized {
    pub id: WrappedUuidV4,
    pub type_id: String,
    pub metadata: Metadata,
    pub value: WrappedJsonValue,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
    pub next_rotation_time: Option<WrappedChronoDateTime>,
    pub data_encryption_key_id: String,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct ProviderCredentialControllerSerialized {
    pub type_id: String,
    pub configuration_schema: ConfigurationSchema,
    pub name: String,
    pub documentation: String,
    pub requires_brokering: bool,
    pub requires_resource_server_credential_refreshing: bool,
    pub requires_user_credential_refreshing: bool,
}

impl From<Arc<dyn ProviderCredentialControllerLike>> for ProviderCredentialControllerSerialized {
    fn from(credential_controller: Arc<dyn ProviderCredentialControllerLike>) -> Self {
        ProviderCredentialControllerSerialized {
            type_id: credential_controller.type_id().to_string(),
            configuration_schema: credential_controller.configuration_schema(),
            name: credential_controller.name().to_string(),
            documentation: credential_controller.documentation().to_string(),
            requires_brokering: credential_controller.as_user_credential_broker().is_some(),
            requires_resource_server_credential_refreshing: credential_controller
                .as_rotateable_controller_resource_server_credential()
                .is_some(),
            requires_user_credential_refreshing: credential_controller
                .as_rotateable_controller_user_credential()
                .is_some(),
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct ProviderInstanceSerialized {
    // not UUID as some ID's will be deterministic
    pub id: String,
    pub display_name: String,
    pub resource_server_credential_id: WrappedUuidV4,
    pub user_credential_id: WrappedUuidV4,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
    pub provider_controller_type_id: String,
    pub credential_controller_type_id: String,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct ProviderInstanceSerializedWithCredentials {
    pub provider_instance: ProviderInstanceSerialized,
    pub resource_server_credential: ResourceServerCredentialSerialized,
    pub user_credential: UserCredentialSerialized,
}

// we shouldn't need this besides the fact that we want to keep track of functions intentionally enabled
// by users. if all functions were enabled, always, we could drop this struct.
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct FunctionInstanceSerialized {
    pub id: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
    pub provider_instance_id: String,
    pub function_controller_type_id: String,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct FunctionInstanceSerializedWithCredentials {
    pub function_instance: FunctionInstanceSerialized,
    pub provider_instance: ProviderInstanceSerialized,
    pub resource_server_credential: ResourceServerCredentialSerialized,
    pub user_credential: UserCredentialSerialized,
}

// api method modelling

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct FunctionControllerSerialized {
    pub type_id: String,
    pub name: String,
    pub documentation: String,
    pub parameters: WrappedSchema,
    pub output: WrappedSchema,
    pub categories: Vec<String>, // TODO: change to Vec<&'static str>
}

impl From<Arc<dyn FunctionControllerLike>> for FunctionControllerSerialized {
    fn from(function: Arc<dyn FunctionControllerLike>) -> Self {
        FunctionControllerSerialized {
            type_id: function.type_id().to_string(),
            name: function.name().to_string(),
            documentation: function.documentation().to_string(),
            parameters: function.parameters(),
            output: function.output(),
            categories: function
                .categories()
                .into_iter()
                .map(|c| c.to_string())
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct ProviderControllerSerialized {
    pub type_id: String,
    pub name: String,
    pub categories: Vec<String>,
    pub documentation: String,
    pub functions: Vec<FunctionControllerSerialized>,
    pub credential_controllers: Vec<ProviderCredentialControllerSerialized>,
}

impl From<&dyn ProviderControllerLike> for ProviderControllerSerialized {
    fn from(provider: &dyn ProviderControllerLike) -> Self {
        ProviderControllerSerialized {
            type_id: provider.type_id().to_string(),
            name: provider.name().to_string(),
            categories: provider
                .categories()
                .into_iter()
                .map(|c| c.to_string())
                .collect(),
            documentation: provider.documentation().to_string(),
            credential_controllers: provider
                .credential_controllers()
                .into_iter()
                .map(|c| c.into())
                .collect::<Vec<ProviderCredentialControllerSerialized>>(),
            functions: provider
                .functions()
                .into_iter()
                .map(|f| f.into())
                .collect::<Vec<FunctionControllerSerialized>>(),
        }
    }
}

// encryption functions

/// Generate or load a local encryption key from a file path.
/// If the file already exists, it reads and returns the key.
/// If the file doesn't exist, it generates a new 32-byte key, saves it, and returns it.
pub fn get_or_create_local_encryption_key(
    file_path: &PathBuf,
) -> Result<EnvelopeEncryptionKeyContents, CommonError> {
    use rand::RngCore;

    // If file exists, read and return the key
    if file_path.exists() {
        let key_bytes = std::fs::read(file_path.clone()).map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!(
                "Failed to read local KEK file at {}: {}",
                file_path.display(),
                e
            ))
        })?;

        if key_bytes.len() != 32 {
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "Invalid local KEK length in file {}: expected 32 bytes, got {}",
                file_path.display(),
                key_bytes.len()
            )));
        }

        return Ok(EnvelopeEncryptionKeyContents::Local {
            key_id: file_path.to_string_lossy().to_string(),
            key_bytes,
        });
    }

    // File doesn't exist - generate new key
    let mut key_bytes = vec![0u8; 32];
    rand::thread_rng().fill_bytes(&mut key_bytes);

    // Write the key to file
    std::fs::write(file_path, &key_bytes).map_err(|e| {
        CommonError::Unknown(anyhow::anyhow!(
            "Failed to write local KEK file at {}: {}",
            file_path.display(),
            e
        ))
    })?;

    Ok(EnvelopeEncryptionKeyContents::Local {
        key_id: file_path.to_string_lossy().to_string(),
        key_bytes,
    })
}

pub async fn encrypt_data_envelope_key(
    parent_encryption_key: &EnvelopeEncryptionKeyContents,
    data_envelope_key: String,
) -> Result<EncryptedDataEncryptionKey, CommonError> {
    match parent_encryption_key {
        EnvelopeEncryptionKeyContents::AwsKms { arn } => {
            // Create AWS KMS client
            let config = aws_config::load_from_env().await;
            let kms_client = aws_sdk_kms::Client::new(&config);

            // Encrypt the data envelope key using AWS KMS
            let encrypt_output = kms_client
                .encrypt()
                .key_id(arn)
                .plaintext(aws_sdk_kms::primitives::Blob::new(
                    data_envelope_key.as_bytes(),
                ))
                .send()
                .await
                .map_err(|e| {
                    CommonError::Unknown(anyhow::anyhow!(
                        "Failed to encrypt data envelope key with AWS KMS: {}",
                        e
                    ))
                })?;

            // Get the encrypted ciphertext blob
            let ciphertext_blob = encrypt_output.ciphertext_blob().ok_or_else(|| {
                CommonError::Unknown(anyhow::anyhow!(
                    "AWS KMS encrypt response did not contain ciphertext blob"
                ))
            })?;

            // Encode to base64 for storage
            let encrypted_key = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                ciphertext_blob.as_ref(),
            );

            Ok(EncryptedDataEncryptionKey(encrypted_key))
        }
        EnvelopeEncryptionKeyContents::Local {
            key_id: _,
            key_bytes,
        } => {
            // --- Local AES-GCM path ---
            if key_bytes.len() != 32 {
                return Err(CommonError::Unknown(anyhow::anyhow!(
                    "Invalid local KEK length: expected 32 bytes, got {}",
                    key_bytes.len()
                )));
            }

            let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
            let cipher = Aes256Gcm::new(key);

            let mut nonce_bytes = [0u8; 12];
            OsRng.fill_bytes(&mut nonce_bytes);
            let nonce = Nonce::from_slice(&nonce_bytes);

            let ciphertext = cipher
                .encrypt(nonce, data_envelope_key.as_bytes())
                .map_err(|e| {
                    CommonError::Unknown(anyhow::anyhow!("Local envelope encryption failed: {}", e))
                })?;

            // Combine nonce + ciphertext
            let mut combined = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
            combined.extend_from_slice(&nonce_bytes);
            combined.extend_from_slice(&ciphertext);

            let encoded = base64::engine::general_purpose::STANDARD.encode(&combined);
            Ok(EncryptedDataEncryptionKey(encoded))
        }
    }
}

pub async fn decrypt_data_envelope_key(
    parent_encryption_key: &EnvelopeEncryptionKeyContents,
    encrypted_data_envelope_key: &EncryptedDataEncryptionKey,
) -> Result<DecryptedDataEnvelopeKey, CommonError> {
    match parent_encryption_key {
        EnvelopeEncryptionKeyContents::AwsKms { arn } => {
            // Decode the base64 encrypted key
            let ciphertext_blob = base64::Engine::decode(
                &base64::engine::general_purpose::STANDARD,
                &encrypted_data_envelope_key.0,
            )
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!(
                    "Failed to decode base64 encrypted data envelope key: {}",
                    e
                ))
            })?;

            // Create AWS KMS client
            let config = aws_config::load_from_env().await;
            let kms_client = aws_sdk_kms::Client::new(&config);

            // Decrypt the data envelope key using AWS KMS
            let decrypt_output = kms_client
                .decrypt()
                .key_id(arn)
                .ciphertext_blob(aws_sdk_kms::primitives::Blob::new(ciphertext_blob))
                .send()
                .await
                .map_err(|e| {
                    CommonError::Unknown(anyhow::anyhow!(
                        "Failed to decrypt data envelope key with AWS KMS: {}",
                        e
                    ))
                })?;

            // Get the decrypted plaintext as raw bytes
            let plaintext = decrypt_output.plaintext().ok_or_else(|| {
                CommonError::Unknown(anyhow::anyhow!(
                    "AWS KMS decrypt response did not contain plaintext"
                ))
            })?;

            // Store as raw bytes (no UTF-8 conversion needed for key material)
            Ok(DecryptedDataEnvelopeKey(plaintext.as_ref().to_vec()))
        }
        EnvelopeEncryptionKeyContents::Local {
            key_id: _,
            key_bytes,
        } => {
            // --- Local AES-GCM path ---
            if key_bytes.len() != 32 {
                return Err(CommonError::Unknown(anyhow::anyhow!(
                    "Invalid local KEK length: expected 32 bytes, got {}",
                    key_bytes.len()
                )));
            }

            let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
            let cipher = Aes256Gcm::new(key);

            let encrypted_data = base64::engine::general_purpose::STANDARD
                .decode(&encrypted_data_envelope_key.0)
                .map_err(|e| {
                    CommonError::Unknown(anyhow::anyhow!(
                        "Failed to decode base64 encrypted DEK: {}",
                        e
                    ))
                })?;

            if encrypted_data.len() < 12 {
                return Err(CommonError::Unknown(anyhow::anyhow!(
                    "Invalid encrypted DEK format: missing nonce"
                )));
            }

            let nonce = Nonce::from_slice(&encrypted_data[..12]);
            let ciphertext = &encrypted_data[12..];

            let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Local DEK decryption failed: {}", e))
            })?;

            Ok(DecryptedDataEnvelopeKey(plaintext))
        }
    }
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct CreateDataEncryptionKeyParams {
    // pub envelope_encryption_key_id: EnvelopeEncryptionKeyId,
    pub id: Option<String>,
    pub encrypted_data_envelope_key: Option<EncryptedDataEncryptionKey>,
}

pub type CreateDataEncryptionKeyResponse = DataEncryptionKey;

pub async fn create_data_encryption_key(
    key_encryption_key: &EnvelopeEncryptionKeyContents,
    on_config_change_tx: &OnConfigChangeTx,
    repo: &impl crate::repository::ProviderRepositoryLike,
    params: CreateDataEncryptionKeyParams,
) -> Result<CreateDataEncryptionKeyResponse, CommonError> {
    let id = match params.id {
        Some(id) => {
            // overwrite existing DEK if same ID exists
            repo.delete_data_encryption_key(&id).await?;
            on_config_change_tx
                .send(OnConfigChangeEvt::DataEncryptionKeyRemoved(id.clone()))
                .await?;
            id
        }
        None => uuid::Uuid::new_v4().to_string(),
    };

    let key_encryption_key = key_encryption_key.clone();
    let encrypted_data_encryption_key = match params.encrypted_data_envelope_key {
        Some(existing) => existing,
        None => match &key_encryption_key {
            EnvelopeEncryptionKeyContents::AwsKms { arn } => {
                // --- AWS KMS path ---
                let config = aws_config::load_from_env().await;
                let kms_client = aws_sdk_kms::Client::new(&config);

                let output = kms_client
                    .generate_data_key()
                    .key_id(arn)
                    .key_spec(aws_sdk_kms::types::DataKeySpec::Aes256)
                    .send()
                    .await
                    .map_err(|e| {
                        CommonError::Unknown(anyhow::anyhow!(
                            "Failed to generate data key with AWS KMS: {}",
                            e
                        ))
                    })?;

                let ciphertext_blob = output.ciphertext_blob().ok_or_else(|| {
                    CommonError::Unknown(anyhow::anyhow!(
                        "AWS KMS GenerateDataKey response did not contain ciphertext blob"
                    ))
                })?;

                let encoded = base64::engine::general_purpose::STANDARD.encode(ciphertext_blob);
                EncryptedDataEncryptionKey(encoded)
            }

            EnvelopeEncryptionKeyContents::Local { key_id, key_bytes } => {
                // --- Local path (no AWS involved) ---
                if key_bytes.len() != 32 {
                    return Err(CommonError::Unknown(anyhow::anyhow!(
                        "Invalid KEK length in {} (expected 32 bytes, got {})",
                        key_id,
                        key_bytes.len()
                    )));
                }

                // Generate random 32-byte DEK
                let mut dek = [0u8; 32];
                rand::thread_rng().fill_bytes(&mut dek);

                // Encrypt DEK with local KEK using AES-GCM
                use aes_gcm::{
                    Aes256Gcm, Nonce,
                    aead::{Aead, KeyInit, OsRng},
                };

                let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
                let cipher = Aes256Gcm::new(key);

                let mut nonce_bytes = [0u8; 12];
                OsRng.fill_bytes(&mut nonce_bytes);
                let nonce = Nonce::from_slice(&nonce_bytes);

                let ciphertext = cipher.encrypt(nonce, dek.as_slice()).map_err(|e| {
                    CommonError::Unknown(anyhow::anyhow!("Failed to encrypt DEK locally: {}", e))
                })?;

                let mut combined = Vec::with_capacity(12 + ciphertext.len());
                combined.extend_from_slice(&nonce_bytes);
                combined.extend_from_slice(&ciphertext);

                let encoded = base64::engine::general_purpose::STANDARD.encode(&combined);
                EncryptedDataEncryptionKey(encoded)
            }
        },
    };

    let now = WrappedChronoDateTime::now();

    let data_encryption_key = DataEncryptionKey {
        id,
        envelope_encryption_key_id: key_encryption_key.into(),
        encrypted_data_encryption_key,
        created_at: now,
        updated_at: now,
    };

    repo.create_data_encryption_key(&data_encryption_key.clone().into())
        .await?;

    on_config_change_tx
        .send(OnConfigChangeEvt::DataEncryptionKeyAdded(
            data_encryption_key.clone(),
        ))
        .await?;

    Ok(data_encryption_key)
}

pub type ListDataEncryptionKeysParams = PaginationRequest;
pub type ListDataEncryptionKeysResponse = PaginatedResponse<DataEncryptionKeyListItem>;

pub async fn list_data_encryption_keys(
    repo: &impl crate::repository::ProviderRepositoryLike,
    params: ListDataEncryptionKeysParams,
) -> Result<ListDataEncryptionKeysResponse, CommonError> {
    let data_encryption_keys = repo.list_data_encryption_keys(&params).await?;
    Ok(data_encryption_keys)
}

async fn get_crypto_service(
    envelope_encryption_key_contents: &EnvelopeEncryptionKeyContents,
    repo: &impl crate::repository::ProviderRepositoryLike,
    data_encryption_key_id: &String,
) -> Result<CryptoService, CommonError> {
    let data_encryption_key = repo
        .get_data_encryption_key_by_id(&data_encryption_key_id)
        .await?;

    let data_encryption_key = match data_encryption_key {
        Some(data_encryption_key) => data_encryption_key,
        None => {
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "Data encryption key not found"
            )));
        }
    };

    let crypto_service = CryptoService::new(
        envelope_encryption_key_contents.clone(),
        data_encryption_key,
    )
    .await?;
    Ok(crypto_service)
}

fn get_encryption_service(
    crypto_service: &CryptoService,
) -> Result<EncryptionService, CommonError> {
    Ok(EncryptionService(crypto_service.clone()))
}

fn get_decryption_service(
    crypto_service: &CryptoService,
) -> Result<DecryptionService, CommonError> {
    Ok(DecryptionService(crypto_service.clone()))
}

// everything else functions

pub const CATEGORY_EMAIL: &str = "email";

pub static PROVIDER_REGISTRY: Lazy<RwLock<Vec<Arc<dyn ProviderControllerLike>>>> =
    Lazy::new(|| RwLock::new(Vec::new()));

pub type ListAvailableProvidersParams = PaginationRequest;
pub type ListAvailableProvidersResponse = PaginatedResponse<ProviderControllerSerialized>;
pub async fn list_available_providers(
    pagination: ListAvailableProvidersParams,
) -> Result<ListAvailableProvidersResponse, CommonError> {
    let providers = PROVIDER_REGISTRY
        .read()
        .map_err(|_e| CommonError::Unknown(anyhow::anyhow!("Poison error")))?
        .iter()
        .map(|p| p.as_ref().into())
        .collect::<Vec<ProviderControllerSerialized>>();

    Ok(ListAvailableProvidersResponse::from_items_with_extra(
        providers,
        &pagination,
        |p| vec![p.type_id.to_string()],
    ))
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct WithProviderControllerTypeId<T> {
    pub provider_controller_type_id: String,
    pub inner: T,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct WithCredentialControllerTypeId<T> {
    pub credential_controller_type_id: String,
    pub inner: T,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct EncryptCredentialConfigurationParamsInner {
    pub value: WrappedJsonValue,
    pub data_encryption_key_id: String,
}

pub type EncryptedCredentialConfigurationResponse = WrappedJsonValue;

pub type EncryptConfigurationParams = WithProviderControllerTypeId<
    WithCredentialControllerTypeId<EncryptCredentialConfigurationParamsInner>,
>;

fn get_provider_controller(
    provider_controller_type_id: &str,
) -> Result<Arc<dyn ProviderControllerLike>, CommonError> {
    let provider_controller = PROVIDER_REGISTRY
        .read()
        .map_err(|_e| CommonError::Unknown(anyhow::anyhow!("Poison error")))?
        .iter()
        .find(|p| p.type_id() == provider_controller_type_id)
        .ok_or(CommonError::Unknown(anyhow::anyhow!(
            "Provider controller not found"
        )))?
        .clone();

    Ok(provider_controller)
}

fn get_credential_controller(
    provider_controller: &Arc<dyn ProviderControllerLike>,
    credential_controller_type_id: &str,
) -> Result<Arc<dyn ProviderCredentialControllerLike>, CommonError> {
    let credential_controller = provider_controller
        .credential_controllers()
        .iter()
        .find(|c| c.type_id() == credential_controller_type_id)
        .ok_or(CommonError::Unknown(anyhow::anyhow!(
            "Credential controller not found"
        )))?
        .clone();

    Ok(credential_controller)
}

fn get_function_controller(
    provider_controller: &Arc<dyn ProviderControllerLike>,
    function_controller_type_id: &str,
) -> Result<Arc<dyn FunctionControllerLike>, CommonError> {
    let function_controller = provider_controller
        .functions()
        .iter()
        .find(|f| f.type_id() == function_controller_type_id)
        .ok_or(CommonError::Unknown(anyhow::anyhow!(
            "Function controller not found"
        )))?
        .clone();
    Ok(function_controller)
}

pub async fn encrypt_resource_server_configuration(
    envelope_encryption_key_contents: &EnvelopeEncryptionKeyContents,
    repo: &impl crate::repository::ProviderRepositoryLike,
    params: EncryptConfigurationParams,
) -> Result<EncryptedCredentialConfigurationResponse, CommonError> {
    let crypto_service = get_crypto_service(
        envelope_encryption_key_contents,
        repo,
        &params.inner.inner.data_encryption_key_id,
    )
    .await?;
    let encryption_service = get_encryption_service(&crypto_service)?;
    let provider_controller = get_provider_controller(&params.provider_controller_type_id)?;
    let credential_controller = get_credential_controller(
        &provider_controller,
        &params.inner.credential_controller_type_id,
    )?;
    let resource_server_configuration = params.inner.inner.value;

    let encrypted_resource_server_configuration = credential_controller
        .encrypt_resource_server_configuration(&encryption_service, resource_server_configuration)
        .await?;

    Ok(encrypted_resource_server_configuration.value())
}

pub async fn encrypt_user_credential_configuration(
    envelope_encryption_key_contents: &EnvelopeEncryptionKeyContents,
    repo: &impl crate::repository::ProviderRepositoryLike,
    params: EncryptConfigurationParams,
) -> Result<EncryptedCredentialConfigurationResponse, CommonError> {
    let crypto_service = get_crypto_service(
        envelope_encryption_key_contents,
        repo,
        &params.inner.inner.data_encryption_key_id,
    )
    .await?;
    let encryption_service = get_encryption_service(&crypto_service)?;
    let provider_controller = get_provider_controller(&params.provider_controller_type_id)?;
    let credential_controller = get_credential_controller(
        &provider_controller,
        &params.inner.credential_controller_type_id,
    )?;
    let user_credential_configuration = params.inner.inner.value;

    let encrypted_user_credential_configuration = credential_controller
        .encrypt_user_credential_configuration(&encryption_service, user_credential_configuration)
        .await?;

    Ok(encrypted_user_credential_configuration.value())
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct CreateResourceServerCredentialParamsInner {
    // NOTE: serialized values are always already encrypted, only encrypt_provider_configuration accepts raw values
    pub resource_server_configuration: WrappedJsonValue,
    pub metadata: Option<Metadata>,
    pub data_encryption_key_id: String,
}
pub type CreateResourceServerCredentialParams = WithProviderControllerTypeId<
    WithCredentialControllerTypeId<CreateResourceServerCredentialParamsInner>,
>;
pub type CreateResourceServerCredentialResponse = ResourceServerCredentialSerialized;

pub async fn create_resource_server_credential(
    repo: &impl crate::repository::ProviderRepositoryLike,
    params: CreateResourceServerCredentialParams,
) -> Result<CreateResourceServerCredentialResponse, CommonError> {
    let provider_controller = get_provider_controller(&params.provider_controller_type_id)?;

    let credential_controller = get_credential_controller(
        &provider_controller,
        &params.inner.credential_controller_type_id,
    )?;

    let (resource_server_credential, mut core_metadata) = credential_controller
        .from_serialized_resource_server_configuration(
            params.inner.inner.resource_server_configuration,
        )?;

    if let Some(metadata) = params.inner.inner.metadata {
        core_metadata.0.extend(metadata.0);
    }

    let next_rotation_time = if let Some(resource_server_credential) =
        resource_server_credential.as_rotateable_credential()
    {
        Some(resource_server_credential.next_rotation_time())
    } else {
        None
    };

    let now = WrappedChronoDateTime::now();
    let resource_server_credential_serialized = ResourceServerCredentialSerialized {
        id: WrappedUuidV4::new(),
        type_id: resource_server_credential.type_id().to_string(),
        metadata: core_metadata,
        value: resource_server_credential.value(),
        created_at: now,
        updated_at: now,
        next_rotation_time: next_rotation_time,
        data_encryption_key_id: params.inner.inner.data_encryption_key_id,
    };

    // Save to database
    repo.create_resource_server_credential(
        &crate::repository::CreateResourceServerCredential::from(
            resource_server_credential_serialized.clone(),
        ),
    )
    .await?;

    Ok(resource_server_credential_serialized)
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct CreateUserCredentialParamsInner {
    pub user_credential_configuration: WrappedJsonValue,
    pub metadata: Option<Metadata>,
    pub data_encryption_key_id: String,
}
pub type CreateUserCredentialParams =
    WithProviderControllerTypeId<WithCredentialControllerTypeId<CreateUserCredentialParamsInner>>;
pub type CreateUserCredentialResponse = UserCredentialSerialized;

pub async fn create_user_credential(
    repo: &impl crate::repository::ProviderRepositoryLike,
    params: CreateUserCredentialParams,
) -> Result<CreateUserCredentialResponse, CommonError> {
    let provider_controller = get_provider_controller(&params.provider_controller_type_id)?;

    let credential_controller = get_credential_controller(
        &provider_controller,
        &params.inner.credential_controller_type_id,
    )?;

    let (user_credential, mut core_metadata) = credential_controller
        .from_serialized_user_credential_configuration(
            params.inner.inner.user_credential_configuration,
        )?;

    if let Some(metadata) = params.inner.inner.metadata {
        core_metadata.0.extend(metadata.0);
    }

    let next_rotation_time =
        if let Some(user_credential) = user_credential.as_rotateable_credential() {
            Some(user_credential.next_rotation_time())
        } else {
            None
        };

    let now = WrappedChronoDateTime::now();
    let user_credential_serialized = UserCredentialSerialized {
        id: WrappedUuidV4::new(),
        type_id: user_credential.type_id().to_string(),
        metadata: core_metadata,
        value: user_credential.value(),
        created_at: now,
        updated_at: now,
        next_rotation_time: next_rotation_time,
        data_encryption_key_id: params.inner.inner.data_encryption_key_id,
    };

    // Save to database
    repo.create_user_credential(&crate::repository::CreateUserCredential::from(
        user_credential_serialized.clone(),
    ))
    .await?;

    Ok(user_credential_serialized)
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct CreateProviderInstanceParamsInner {
    pub resource_server_credential_id: WrappedUuidV4,
    pub user_credential_id: WrappedUuidV4,
    pub provider_instance_id: Option<String>,
    pub display_name: String,
}
pub type CreateProviderInstanceParams =
    WithProviderControllerTypeId<WithCredentialControllerTypeId<CreateProviderInstanceParamsInner>>;
pub type CreateProviderInstanceResponse = ProviderInstanceSerialized;

pub async fn create_provider_instance(
    on_config_change_tx: &OnConfigChangeTx,
    repo: &impl crate::repository::ProviderRepositoryLike,
    params: CreateProviderInstanceParams,
) -> Result<CreateProviderInstanceResponse, CommonError> {
    let provider_controller = get_provider_controller(&params.provider_controller_type_id)?;

    let credential_controller = get_credential_controller(
        &provider_controller,
        &params.inner.credential_controller_type_id,
    )?;

    // Verify resource server credential exists
    let resource_server_credential = repo
        .get_resource_server_credential_by_id(&params.inner.inner.resource_server_credential_id)
        .await?
        .ok_or(CommonError::Unknown(anyhow::anyhow!(
            "Resource server credential not found"
        )))?;

    // Verify user credential exists
    let user_credential = repo
        .get_user_credential_by_id(&params.inner.inner.user_credential_id)
        .await?
        .ok_or(CommonError::Unknown(anyhow::anyhow!(
            "User credential not found"
        )))?;

    let provider_instance_id = match params.inner.inner.provider_instance_id {
        Some(provider_instance_id) => provider_instance_id,
        None => uuid::Uuid::new_v4().to_string(),
    };
    let now = WrappedChronoDateTime::now();
    let provider_instance_serialized = ProviderInstanceSerialized {
        id: provider_instance_id,
        display_name: params.inner.inner.display_name,
        resource_server_credential_id: params.inner.inner.resource_server_credential_id,
        user_credential_id: params.inner.inner.user_credential_id,
        created_at: now,
        updated_at: now,
        provider_controller_type_id: params.provider_controller_type_id,
        credential_controller_type_id: params.inner.credential_controller_type_id,
    };

    // Save to database
    repo.create_provider_instance(&crate::repository::CreateProviderInstance::from(
        provider_instance_serialized.clone(),
    ))
    .await?;

    let provider_instance_with_credentials = ProviderInstanceSerializedWithCredentials {
        provider_instance: provider_instance_serialized.clone(),
        resource_server_credential: resource_server_credential.clone(),
        user_credential: user_credential.clone(),
    };
    on_config_change_tx
        .send(OnConfigChangeEvt::ProviderInstanceAdded(
            provider_instance_with_credentials,
        ))
        .await?;

    Ok(provider_instance_serialized)
}

pub type DeleteProviderInstanceParams = WithProviderInstanceId<()>;
pub type DeleteProviderInstanceResponse = ();

pub async fn delete_provider_instance(
    on_config_change_tx: &OnConfigChangeTx,
    repo: &impl crate::repository::ProviderRepositoryLike,
    params: DeleteProviderInstanceParams,
) -> Result<DeleteProviderInstanceResponse, CommonError> {
    repo.delete_provider_instance(&params.provider_instance_id)
        .await?;
    on_config_change_tx
        .send(OnConfigChangeEvt::ProviderInstanceRemoved(
            params.provider_instance_id.clone(),
        ))
        .await?;
    Ok(())
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct WithProviderInstanceId<T> {
    pub provider_instance_id: String,
    pub inner: T,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct WithFunctionControllerTypeId<T> {
    pub function_controller_type_id: String,
    pub inner: T,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct WithFunctionInstanceId<T> {
    pub function_instance_id: String,
    pub inner: T,
}

// TODO: list provider instances potentially? but assuming a clean database on startup for now.

#[derive(Serialize, Deserialize, Clone, ToSchema, JsonSchema)]
pub struct EnableFunctionParamsInner {
    pub function_instance_id: Option<String>,
}
pub type EnableFunctionParams =
    WithProviderInstanceId<WithFunctionControllerTypeId<EnableFunctionParamsInner>>;
pub type EnableFunctionResponse = FunctionInstanceSerialized;

pub async fn enable_function(
    on_config_change_tx: &OnConfigChangeTx,
    repo: &crate::repository::Repository,
    params: EnableFunctionParams,
) -> Result<EnableFunctionResponse, CommonError> {
    // Verify provider instance exists
    let provider_instance = repo
        .get_provider_instance_by_id(&params.provider_instance_id)
        .await?
        .ok_or(CommonError::Unknown(anyhow::anyhow!(
            "Provider instance not found"
        )))?;

    // // Verify function exists in provider controller
    let provider_controller = get_provider_controller(&params.provider_instance_id)?;
    let _function_controller = get_function_controller(
        &provider_controller,
        &params.inner.function_controller_type_id,
    )?;

    let function_instance_id = match params.inner.inner.function_instance_id {
        Some(function_instance_id) => function_instance_id,
        None => uuid::Uuid::new_v4().to_string(),
    };
    let now = WrappedChronoDateTime::now();
    let function_instance_serialized = FunctionInstanceSerialized {
        id: function_instance_id,
        created_at: now,
        updated_at: now,
        provider_instance_id: params.provider_instance_id.clone(),
        function_controller_type_id: params.inner.function_controller_type_id.clone(),
    };

    // Save to database
    let create_params =
        crate::repository::CreateFunctionInstance::from(function_instance_serialized.clone());
    repo.create_function_instance(&create_params).await?;

    on_config_change_tx
        .send(OnConfigChangeEvt::FunctionInstanceAdded(
            function_instance_serialized.clone(),
        ))
        .await?;

    Ok(function_instance_serialized)
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct InvokeFunctionParamsInner {
    pub params: WrappedJsonValue,
}
pub type InvokeFunctionParams =
    WithProviderInstanceId<WithFunctionInstanceId<InvokeFunctionParamsInner>>;
pub type InvokeFunctionResponse = WrappedJsonValue;

pub async fn invoke_function(
    repo: &crate::repository::Repository,
    envelope_encryption_key_contents: &EnvelopeEncryptionKeyContents,
    params: InvokeFunctionParams,
) -> Result<InvokeFunctionResponse, CommonError> {
    let function_instance_with_credentials = repo
        .get_function_instance_with_credentials(&params.inner.function_instance_id)
        .await?
        .ok_or(CommonError::Unknown(anyhow::anyhow!(
            "Function instance not found"
        )))?;

    // TODO: we assume user and resource credentials are encrypted with the same data encryption key
    // this could change in future as the sql tables permit different data encryption keys for user and resource credentials
    let crypto_service = get_crypto_service(
        envelope_encryption_key_contents,
        repo,
        &function_instance_with_credentials
            .resource_server_credential
            .data_encryption_key_id,
    )
    .await?;
    let decryption_service = get_decryption_service(&crypto_service)?;
    let provder_controller = get_provider_controller(
        &function_instance_with_credentials
            .provider_instance
            .provider_controller_type_id,
    )?;
    let function_controller = get_function_controller(
        &provder_controller,
        &function_instance_with_credentials
            .function_instance
            .function_controller_type_id,
    )?;

    // TODO: I think the credential controller should manage decrypting the resource server credential and user credential and static credentials
    // and pass a single return type to the function invocation that implements a DecryptedFullCredentialLike trait?
    let credential_controller = get_credential_controller(
        &provder_controller,
        &function_instance_with_credentials
            .provider_instance
            .credential_controller_type_id,
    )?;
    let static_credentials = credential_controller.static_credentials();

    let response = function_controller
        .invoke(
            &decryption_service,
            &credential_controller,
            &static_credentials,
            &function_instance_with_credentials.resource_server_credential,
            &function_instance_with_credentials.user_credential,
            params.inner.inner.params,
        )
        .await?;
    Ok(response)
}

#[derive(Serialize, Deserialize, Clone, ToSchema, JsonSchema)]
pub struct DisableFunctionParamsInner {
    pub function_instance_id: String,
}
pub type DisableFunctionParams = WithProviderInstanceId<DisableFunctionParamsInner>;
pub type DisableFunctionResponse = ();

pub async fn disable_function(
    on_config_change_tx: &OnConfigChangeTx,
    repo: &crate::repository::Repository,
    params: DisableFunctionParams,
) -> Result<DisableFunctionResponse, CommonError> {
    // Delete from database
    repo.delete_function_instance(&params.inner.function_instance_id)
        .await?;
    on_config_change_tx
        .send(OnConfigChangeEvt::FunctionInstanceRemoved(
            params.inner.function_instance_id.clone(),
        ))
        .await?;
    Ok(())
}

// TODO: list functions potentially? but assuming a clean database on startup for now.

async fn process_broker_outcome(
    repo: &impl crate::repository::ProviderRepositoryLike,
    provider_controller: &Arc<dyn ProviderControllerLike>,
    credential_controller: &Arc<dyn ProviderCredentialControllerLike>,
    resource_server_cred_id: WrappedUuidV4,
    broker_action: &BrokerAction,
    outcome: BrokerOutcome,
) -> Result<UserCredentialBrokeringResponse, CommonError> {
    let response = match outcome {
        BrokerOutcome::Success {
            user_credential,
            metadata,
        } => {
            let resource_server_cred = repo
                .get_resource_server_credential_by_id(&resource_server_cred_id)
                .await?;

            let resource_server_cred = match resource_server_cred {
                Some(resource_server_cred) => resource_server_cred,
                None => {
                    return Err(CommonError::Unknown(anyhow::anyhow!(
                        "Resource server credential not found"
                    )));
                }
            };
            let data_encryption_key_id = resource_server_cred.data_encryption_key_id;
            let user_credential = create_user_credential(
                repo,
                CreateUserCredentialParams {
                    provider_controller_type_id: provider_controller.type_id().to_string(),
                    inner: WithCredentialControllerTypeId {
                        credential_controller_type_id: credential_controller.type_id().to_string(),
                        inner: CreateUserCredentialParamsInner {
                            data_encryption_key_id: data_encryption_key_id,
                            user_credential_configuration: user_credential.value(),
                            metadata: Some(metadata),
                        },
                    },
                },
            )
            .await?;

            UserCredentialBrokeringResponse::UserCredential(user_credential)
        }
        BrokerOutcome::Continue { metadata } => {
            let broker_state = BrokerState {
                id: uuid::Uuid::new_v4().to_string(),
                created_at: WrappedChronoDateTime::now(),
                updated_at: WrappedChronoDateTime::now(),
                resource_server_cred_id: resource_server_cred_id,
                provider_controller_type_id: provider_controller.type_id().to_string(),
                metadata: metadata,
                action: broker_action.clone(),
                credential_controller_type_id: credential_controller.type_id().to_string(),
            };

            // Save broker state to database
            repo.create_broker_state(&crate::repository::CreateBrokerState::from(
                broker_state.clone(),
            ))
            .await?;

            UserCredentialBrokeringResponse::BrokerState(broker_state)
        }
    };

    Ok(response)
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct StartUserCredentialBrokeringParamsInner {
    pub resource_server_cred_id: WrappedUuidV4,
}
pub type StartUserCredentialBrokeringParams = WithProviderControllerTypeId<
    WithCredentialControllerTypeId<StartUserCredentialBrokeringParamsInner>,
>;

#[derive(Serialize, Deserialize, Clone, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UserCredentialBrokeringResponse {
    BrokerState(BrokerState),
    UserCredential(UserCredentialSerialized),
}
pub async fn start_user_credential_brokering(
    repo: &impl crate::repository::ProviderRepositoryLike,
    params: StartUserCredentialBrokeringParams,
) -> Result<UserCredentialBrokeringResponse, CommonError> {
    let provider_controller = get_provider_controller(&params.provider_controller_type_id)?;
    let credential_controller = get_credential_controller(
        &provider_controller,
        &params.inner.credential_controller_type_id,
    )?;
    let user_credential_broker = match credential_controller.as_user_credential_broker() {
        Some(broker) => broker,
        None => {
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "Provider controller does not support user credential brokering"
            )));
        }
    };

    // Fetch resource server credential from database
    let resource_server_cred = repo
        .get_resource_server_credential_by_id(&params.inner.inner.resource_server_cred_id)
        .await?
        .ok_or(CommonError::Unknown(anyhow::anyhow!(
            "Resource server credential not found"
        )))?;

    let (inner, metadata) = credential_controller
        .from_serialized_resource_server_configuration(resource_server_cred.value)?;
    let resource_server_cred = Credential {
        inner: inner,
        metadata: metadata,
        id: resource_server_cred.id,
        created_at: resource_server_cred.created_at,
        updated_at: resource_server_cred.updated_at,
    };
    let (action, outcome) = user_credential_broker.start(&resource_server_cred).await?;

    let response = process_broker_outcome(
        repo,
        &provider_controller,
        &credential_controller,
        params.inner.inner.resource_server_cred_id,
        &action,
        outcome,
    )
    .await?;
    Ok(response)
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct ResumeUserCredentialBrokeringParams {
    pub broker_state_id: String,
    pub input: BrokerInput,
}

pub async fn resume_user_credential_brokering(
    repo: &impl crate::repository::ProviderRepositoryLike,
    params: ResumeUserCredentialBrokeringParams,
) -> Result<UserCredentialBrokeringResponse, CommonError> {
    // Fetch broker state from database
    let broker_state = repo
        .get_broker_state_by_id(&params.broker_state_id)
        .await?
        .ok_or(CommonError::Unknown(anyhow::anyhow!(
            "Broker state not found"
        )))?;

    let provider_controller = get_provider_controller(&broker_state.provider_controller_type_id)?;
    let credential_controller = get_credential_controller(
        &provider_controller,
        &broker_state.credential_controller_type_id,
    )?;

    let user_credential_broker = match credential_controller.as_user_credential_broker() {
        Some(broker) => broker,
        None => {
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "Provider controller does not support user credential brokering"
            )));
        }
    };

    let (action, outcome) = user_credential_broker
        .resume(&broker_state, params.input)
        .await?;

    let response = process_broker_outcome(
        repo,
        &provider_controller,
        &credential_controller,
        broker_state.resource_server_cred_id,
        &action,
        outcome,
    )
    .await?;
    Ok(response)
}

pub async fn register_all_bridge_providers() -> Result<(), CommonError> {
    let mut registry = PROVIDER_REGISTRY.write().map_err(|e| {
        CommonError::Unknown(anyhow::anyhow!("Failed to write provider registry: {}", e))
    })?;
    registry.push(Arc::new(GoogleMailProviderController));
    drop(registry);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_KMS_KEY_ARN: &str =
        "arn:aws:kms:us-east-1:855806899624:key/0155f7f0-b3a2-4e5a-afdc-9070c2cd4059";

    #[tokio::test]
    async fn test_encrypt_data_envelope_key_with_aws_kms() {
        shared::setup_test!();

        // Test data
        let test_data = "This is a test data encryption key for envelope encryption";
        let parent_key = EnvelopeEncryptionKeyContents::AwsKms {
            arn: TEST_KMS_KEY_ARN.to_string(),
        };

        // Encrypt the data envelope key
        let result = encrypt_data_envelope_key(&parent_key, test_data.to_string()).await;

        // Verify encryption succeeded
        assert!(result.is_ok(), "Encryption should succeed");
        let encrypted_key = result.unwrap();

        // Verify the encrypted key is not empty
        assert!(
            !encrypted_key.0.is_empty(),
            "Encrypted key should not be empty"
        );

        // Verify the encrypted key is base64 encoded
        let decode_result =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &encrypted_key.0);
        assert!(
            decode_result.is_ok(),
            "Encrypted key should be valid base64"
        );

        // Verify the encrypted key is different from the original
        assert_ne!(
            encrypted_key.0, test_data,
            "Encrypted key should be different from plaintext"
        );
    }

    #[tokio::test]
    async fn test_decrypt_data_envelope_key_with_aws_kms() {
        shared::setup_test!();

        // Test data
        let test_data = "This is a test data encryption key for envelope encryption";
        let parent_key = EnvelopeEncryptionKeyContents::AwsKms {
            arn: TEST_KMS_KEY_ARN.to_string(),
        };

        // First, encrypt the data
        let encrypted_key = encrypt_data_envelope_key(&parent_key, test_data.to_string())
            .await
            .expect("Encryption should succeed");

        // Now decrypt it
        let result = decrypt_data_envelope_key(&parent_key, &encrypted_key).await;

        // Verify decryption succeeded
        assert!(result.is_ok(), "Decryption should succeed");
        let decrypted_key = result.unwrap();

        // Verify the decrypted key matches the original
        assert_eq!(
            decrypted_key.0,
            test_data.as_bytes(),
            "Decrypted key should match original plaintext"
        );
    }

    #[tokio::test]
    async fn test_encrypt_decrypt_roundtrip() {
        shared::setup_test!();

        // Test multiple different data strings
        let long_key = "A".repeat(1000);
        let test_cases = vec![
            "Simple test key",
            "Key with special characters: !@#$%^&*()_+-=[]{}|;:',.<>?",
            "Multi\nline\nkey\nwith\nnewlines",
            "Unicode characters: 你好世界 🌍🔐",
            long_key.as_str(), // Long key
        ];

        let parent_key = EnvelopeEncryptionKeyContents::AwsKms {
            arn: TEST_KMS_KEY_ARN.to_string(),
        };

        for test_data in test_cases {
            // Encrypt
            let encrypted = encrypt_data_envelope_key(&parent_key, test_data.to_string())
                .await
                .expect("Encryption should succeed");

            // Decrypt
            let decrypted = decrypt_data_envelope_key(&parent_key, &encrypted)
                .await
                .expect("Decryption should succeed");

            // Verify
            assert_eq!(
                decrypted.0,
                test_data.as_bytes(),
                "Roundtrip should preserve data for: {}",
                test_data
            );
        }
    }

    #[tokio::test]
    async fn test_decrypt_with_invalid_base64() {
        shared::setup_test!();

        let parent_key = EnvelopeEncryptionKeyContents::AwsKms {
            arn: TEST_KMS_KEY_ARN.to_string(),
        };

        // Create an invalid base64 encrypted key
        let invalid_encrypted_key = EncryptedDataEncryptionKey("Not valid base64!!!".to_string());

        // Try to decrypt
        let result = decrypt_data_envelope_key(&parent_key, &invalid_encrypted_key).await;

        // Should fail with a base64 decode error
        assert!(result.is_err(), "Should fail with invalid base64");
        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(
            error_msg.contains("Failed to decode base64"),
            "Error should mention base64 decode failure"
        );
    }

    #[tokio::test]
    async fn test_decrypt_with_invalid_ciphertext() {
        shared::setup_test!();

        let parent_key = EnvelopeEncryptionKeyContents::AwsKms {
            arn: TEST_KMS_KEY_ARN.to_string(),
        };

        // Create a valid base64 string but invalid ciphertext
        let invalid_ciphertext = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            b"This is not a valid KMS ciphertext",
        );
        let invalid_encrypted_key = EncryptedDataEncryptionKey(invalid_ciphertext);

        // Try to decrypt
        let result = decrypt_data_envelope_key(&parent_key, &invalid_encrypted_key).await;

        // Should fail with a KMS error
        assert!(result.is_err(), "Should fail with invalid ciphertext");
        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(
            error_msg.contains("Failed to decrypt data envelope key with AWS KMS"),
            "Error should mention KMS decrypt failure"
        );
    }

    #[tokio::test]
    async fn test_encrypt_multiple_times_produces_different_ciphertext() {
        shared::setup_test!();

        let test_data = "Same plaintext data";
        let parent_key = EnvelopeEncryptionKeyContents::AwsKms {
            arn: TEST_KMS_KEY_ARN.to_string(),
        };

        // Encrypt the same data multiple times
        let encrypted1 = encrypt_data_envelope_key(&parent_key, test_data.to_string())
            .await
            .expect("First encryption should succeed");

        let encrypted2 = encrypt_data_envelope_key(&parent_key, test_data.to_string())
            .await
            .expect("Second encryption should succeed");

        // The ciphertexts should be different (due to random IV in encryption)
        assert_ne!(
            encrypted1.0, encrypted2.0,
            "Multiple encryptions of same plaintext should produce different ciphertexts"
        );

        // But both should decrypt to the same plaintext
        let decrypted1 = decrypt_data_envelope_key(&parent_key, &encrypted1)
            .await
            .expect("First decryption should succeed");

        let decrypted2 = decrypt_data_envelope_key(&parent_key, &encrypted2)
            .await
            .expect("Second decryption should succeed");

        assert_eq!(
            decrypted1.0,
            test_data.as_bytes(),
            "First decryption should match original"
        );
        assert_eq!(
            decrypted2.0,
            test_data.as_bytes(),
            "Second decryption should match original"
        );
    }

    #[tokio::test]
    async fn test_encrypt_empty_string() {
        shared::setup_test!();

        let test_data = "";
        let parent_key = EnvelopeEncryptionKeyContents::AwsKms {
            arn: TEST_KMS_KEY_ARN.to_string(),
        };

        // AWS KMS does not allow encrypting empty strings (plaintext must be 1-4096 bytes)
        // This test verifies that we get an appropriate error
        let result = encrypt_data_envelope_key(&parent_key, test_data.to_string()).await;

        // Should fail with a KMS error
        assert!(result.is_err(), "Encrypting empty string should fail");
        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(
            error_msg.contains("Failed to encrypt data envelope key with AWS KMS"),
            "Error should mention KMS encrypt failure"
        );
    }

    #[tokio::test]
    async fn test_encrypt_large_data() {
        shared::setup_test!();

        // AWS KMS has a 4KB limit for direct encryption
        // This test ensures we handle data close to that limit
        let test_data = "A".repeat(4000); // 4000 bytes
        let parent_key = EnvelopeEncryptionKeyContents::AwsKms {
            arn: TEST_KMS_KEY_ARN.to_string(),
        };

        // Encrypt
        let encrypted = encrypt_data_envelope_key(&parent_key, test_data.clone())
            .await
            .expect("Encrypting large data should succeed");

        // Decrypt
        let decrypted = decrypt_data_envelope_key(&parent_key, &encrypted)
            .await
            .expect("Decrypting should succeed");

        // Verify
        assert_eq!(
            decrypted.0,
            test_data.as_bytes(),
            "Large data should roundtrip correctly"
        );
    }

    #[tokio::test]
    async fn test_encrypt_with_invalid_kms_arn() {
        shared::setup_test!();

        let test_data = "Test data";
        let invalid_parent_key = EnvelopeEncryptionKeyContents::AwsKms {
            arn: "arn:aws:kms:us-east-1:123456789012:key/invalid-key-id".to_string(),
        };

        // Try to encrypt with invalid ARN
        let result = encrypt_data_envelope_key(&invalid_parent_key, test_data.to_string()).await;

        // Should fail
        assert!(result.is_err(), "Should fail with invalid KMS key ARN");
        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(
            error_msg.contains("Failed to encrypt data envelope key with AWS KMS"),
            "Error should mention KMS encrypt failure"
        );
    }

    #[tokio::test]
    async fn test_encryption_service_aes_gcm_roundtrip() {
        shared::setup_test!();

        // Generate a 32-byte (256-bit) key using AWS KMS
        let parent_key = EnvelopeEncryptionKeyContents::AwsKms {
            arn: TEST_KMS_KEY_ARN.to_string(),
        };

        let config = aws_config::load_from_env().await;
        let kms_client = aws_sdk_kms::Client::new(&config);

        // Generate a 256-bit data key using AWS KMS
        let generate_output = kms_client
            .generate_data_key()
            .key_id(TEST_KMS_KEY_ARN)
            .key_spec(aws_sdk_kms::types::DataKeySpec::Aes256)
            .send()
            .await
            .expect("Failed to generate data key with AWS KMS");

        // Get the encrypted data key (ciphertext blob)
        let ciphertext_blob = generate_output
            .ciphertext_blob()
            .expect("AWS KMS GenerateDataKey response did not contain ciphertext blob");

        // Encode to base64 for storage
        let encrypted_key = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            ciphertext_blob.as_ref(),
        );

        let now = WrappedChronoDateTime::now();
        let data_encryption_key = DataEncryptionKey {
            id: uuid::Uuid::new_v4().to_string(),
            envelope_encryption_key_id: EnvelopeEncryptionKeyId::from(parent_key.clone()),
            encrypted_data_encryption_key: EncryptedDataEncryptionKey(encrypted_key),
            created_at: now,
            updated_at: now,
        };

        // Create crypto service
        let crypto_service = CryptoService::new(parent_key, data_encryption_key.clone())
            .await
            .expect("Failed to create crypto service");

        let encryption_service = EncryptionService::new(crypto_service.clone());
        let decryption_service = DecryptionService::new(crypto_service);

        // Test cases
        let long_data = "A".repeat(1000);
        let test_cases = vec![
            "Simple plaintext",
            "Data with special characters: !@#$%^&*()_+-=[]{}|;:',.<>?",
            "Multi\nline\ndata\nwith\nnewlines",
            "Unicode characters: 你好世界 🌍🔐",
            long_data.as_str(), // Long data
        ];

        for test_data in test_cases {
            // Encrypt
            let encrypted = encryption_service
                .encrypt_data(test_data.to_string())
                .await
                .expect(&format!("Encryption should succeed for: {}", test_data));

            // Verify encrypted is different from plaintext
            assert_ne!(
                encrypted.0, test_data,
                "Encrypted data should differ from plaintext"
            );

            // Verify encrypted is base64
            let decode_result =
                base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &encrypted.0);
            assert!(
                decode_result.is_ok(),
                "Encrypted data should be valid base64"
            );

            // Decrypt
            let decrypted = decryption_service
                .decrypt_data(encrypted)
                .await
                .expect(&format!("Decryption should succeed for: {}", test_data));

            // Verify roundtrip
            assert_eq!(
                decrypted, test_data,
                "Decrypted data should match original plaintext"
            );
        }
    }

    #[tokio::test]
    async fn test_decryption_service_with_invalid_data() {
        shared::setup_test!();

        // Generate a 32-byte (256-bit) key using AWS KMS
        let parent_key = EnvelopeEncryptionKeyContents::AwsKms {
            arn: TEST_KMS_KEY_ARN.to_string(),
        };

        let config = aws_config::load_from_env().await;
        let kms_client = aws_sdk_kms::Client::new(&config);

        let generate_output = kms_client
            .generate_data_key()
            .key_id(TEST_KMS_KEY_ARN)
            .key_spec(aws_sdk_kms::types::DataKeySpec::Aes256)
            .send()
            .await
            .expect("Failed to generate data key with AWS KMS");

        let ciphertext_blob = generate_output
            .ciphertext_blob()
            .expect("AWS KMS GenerateDataKey response did not contain ciphertext blob");

        let encrypted_key = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            ciphertext_blob.as_ref(),
        );

        let now = WrappedChronoDateTime::now();
        let data_encryption_key = DataEncryptionKey {
            id: uuid::Uuid::new_v4().to_string(),
            envelope_encryption_key_id: EnvelopeEncryptionKeyId::from(parent_key.clone()),
            encrypted_data_encryption_key: EncryptedDataEncryptionKey(encrypted_key),
            created_at: now,
            updated_at: now,
        };

        let crypto_service = CryptoService::new(parent_key, data_encryption_key)
            .await
            .expect("Failed to create crypto service");

        let decryption_service = DecryptionService::new(crypto_service);

        // Test with invalid base64
        let result = decryption_service
            .decrypt_data(EncryptedString("Not valid base64!!!".to_string()))
            .await;
        assert!(result.is_err(), "Should fail with invalid base64");

        // Test with too short data (less than nonce size)
        let short_data =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &[0u8; 5]);
        let result = decryption_service
            .decrypt_data(EncryptedString(short_data))
            .await;
        assert!(result.is_err(), "Should fail with data too short");
        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(
            error_msg.contains("too short"),
            "Error should mention data is too short"
        );
    }

    #[tokio::test]
    async fn test_encryption_produces_different_ciphertexts() {
        shared::setup_test!();

        // Generate a 32-byte (256-bit) key using AWS KMS
        let parent_key = EnvelopeEncryptionKeyContents::AwsKms {
            arn: TEST_KMS_KEY_ARN.to_string(),
        };

        let config = aws_config::load_from_env().await;
        let kms_client = aws_sdk_kms::Client::new(&config);

        let generate_output = kms_client
            .generate_data_key()
            .key_id(TEST_KMS_KEY_ARN)
            .key_spec(aws_sdk_kms::types::DataKeySpec::Aes256)
            .send()
            .await
            .expect("Failed to generate data key with AWS KMS");

        let ciphertext_blob = generate_output
            .ciphertext_blob()
            .expect("AWS KMS GenerateDataKey response did not contain ciphertext blob");

        let encrypted_key = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            ciphertext_blob.as_ref(),
        );

        let now = WrappedChronoDateTime::now();
        let data_encryption_key = DataEncryptionKey {
            id: uuid::Uuid::new_v4().to_string(),
            envelope_encryption_key_id: EnvelopeEncryptionKeyId::from(parent_key.clone()),
            encrypted_data_encryption_key: EncryptedDataEncryptionKey(encrypted_key),
            created_at: now,
            updated_at: now,
        };

        let crypto_service = CryptoService::new(parent_key, data_encryption_key)
            .await
            .expect("Failed to create crypto service");

        let encryption_service = EncryptionService::new(crypto_service.clone());
        let decryption_service = DecryptionService::new(crypto_service);

        let test_data = "Same plaintext for both encryptions";

        // Encrypt same data twice
        let encrypted1 = encryption_service
            .encrypt_data(test_data.to_string())
            .await
            .expect("First encryption should succeed");

        let encrypted2 = encryption_service
            .encrypt_data(test_data.to_string())
            .await
            .expect("Second encryption should succeed");

        // Ciphertexts should be different (due to random nonce)
        assert_ne!(
            encrypted1.0, encrypted2.0,
            "Multiple encryptions should produce different ciphertexts"
        );

        // But both should decrypt to same plaintext
        let decrypted1 = decryption_service
            .decrypt_data(encrypted1)
            .await
            .expect("First decryption should succeed");

        let decrypted2 = decryption_service
            .decrypt_data(encrypted2)
            .await
            .expect("Second decryption should succeed");

        assert_eq!(decrypted1, test_data);
        assert_eq!(decrypted2, test_data);
    }

    // Helper function to create a temporary KEK file for local encryption tests
    fn create_temp_kek_file() -> (tempfile::NamedTempFile, EnvelopeEncryptionKeyContents) {
        use rand::RngCore;
        let mut kek_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut kek_bytes);

        let temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        std::fs::write(temp_file.path(), &kek_bytes).expect("Failed to write KEK to temp file");

        let key_id = temp_file
            .path()
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("test-key")
            .to_string();

        let contents = EnvelopeEncryptionKeyContents::Local {
            key_id,
            key_bytes: kek_bytes.to_vec(),
        };

        (temp_file, contents)
    }

    #[tokio::test]
    async fn test_encrypt_data_envelope_key_with_local() {
        shared::setup_test!();

        let (_temp_file, parent_key) = create_temp_kek_file();
        let test_data = "This is a test data encryption key for local envelope encryption";

        // Encrypt the data envelope key
        let result = encrypt_data_envelope_key(&parent_key, test_data.to_string()).await;

        // Verify encryption succeeded
        assert!(result.is_ok(), "Encryption should succeed");
        let encrypted_key = result.unwrap();

        // Verify the encrypted key is not empty
        assert!(
            !encrypted_key.0.is_empty(),
            "Encrypted key should not be empty"
        );

        // Verify the encrypted key is base64 encoded
        let decode_result =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &encrypted_key.0);
        assert!(
            decode_result.is_ok(),
            "Encrypted key should be valid base64"
        );

        // Verify the encrypted key is different from the original
        assert_ne!(
            encrypted_key.0, test_data,
            "Encrypted key should be different from plaintext"
        );
    }

    #[tokio::test]
    async fn test_decrypt_data_envelope_key_with_local() {
        shared::setup_test!();

        let (_temp_file, parent_key) = create_temp_kek_file();
        let test_data = "This is a test data encryption key for local envelope encryption";

        // First, encrypt the data
        let encrypted_key = encrypt_data_envelope_key(&parent_key, test_data.to_string())
            .await
            .expect("Encryption should succeed");

        // Now decrypt it
        let result = decrypt_data_envelope_key(&parent_key, &encrypted_key).await;

        // Verify decryption succeeded
        assert!(result.is_ok(), "Decryption should succeed");
        let decrypted_key = result.unwrap();

        // Verify the decrypted key matches the original
        assert_eq!(
            decrypted_key.0,
            test_data.as_bytes(),
            "Decrypted key should match original plaintext"
        );
    }

    #[tokio::test]
    async fn test_local_encrypt_decrypt_roundtrip() {
        shared::setup_test!();

        let (_temp_file, parent_key) = create_temp_kek_file();

        // Test multiple different data strings
        let long_key = "A".repeat(1000);
        let test_cases = vec![
            "Simple test key",
            "Key with special characters: !@#$%^&*()_+-=[]{}|;:',.<>?",
            "Multi\nline\nkey\nwith\nnewlines",
            "Unicode characters: 你好世界 🌍🔐",
            long_key.as_str(),
        ];

        for test_data in test_cases {
            // Encrypt
            let encrypted = encrypt_data_envelope_key(&parent_key, test_data.to_string())
                .await
                .expect("Encryption should succeed");

            // Decrypt
            let decrypted = decrypt_data_envelope_key(&parent_key, &encrypted)
                .await
                .expect("Decryption should succeed");

            // Verify
            assert_eq!(
                decrypted.0,
                test_data.as_bytes(),
                "Roundtrip should preserve data for: {}",
                test_data
            );
        }
    }

    #[tokio::test]
    async fn test_local_encrypt_multiple_times_produces_different_ciphertext() {
        shared::setup_test!();

        let (_temp_file, parent_key) = create_temp_kek_file();
        let test_data = "Same plaintext data";

        // Encrypt the same data multiple times
        let encrypted1 = encrypt_data_envelope_key(&parent_key, test_data.to_string())
            .await
            .expect("First encryption should succeed");

        let encrypted2 = encrypt_data_envelope_key(&parent_key, test_data.to_string())
            .await
            .expect("Second encryption should succeed");

        // The ciphertexts should be different (due to random nonce in encryption)
        assert_ne!(
            encrypted1.0, encrypted2.0,
            "Multiple encryptions of same plaintext should produce different ciphertexts"
        );

        // But both should decrypt to the same plaintext
        let decrypted1 = decrypt_data_envelope_key(&parent_key, &encrypted1)
            .await
            .expect("First decryption should succeed");

        let decrypted2 = decrypt_data_envelope_key(&parent_key, &encrypted2)
            .await
            .expect("Second decryption should succeed");

        assert_eq!(
            decrypted1.0,
            test_data.as_bytes(),
            "First decryption should match original"
        );
        assert_eq!(
            decrypted2.0,
            test_data.as_bytes(),
            "Second decryption should match original"
        );
    }

    #[tokio::test]
    async fn test_local_decrypt_with_invalid_base64() {
        shared::setup_test!();

        let (_temp_file, parent_key) = create_temp_kek_file();

        // Create an invalid base64 encrypted key
        let invalid_encrypted_key = EncryptedDataEncryptionKey("Not valid base64!!!".to_string());

        // Try to decrypt
        let result = decrypt_data_envelope_key(&parent_key, &invalid_encrypted_key).await;

        // Should fail with a base64 decode error
        assert!(result.is_err(), "Should fail with invalid base64");
        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(
            error_msg.contains("Failed to decode base64"),
            "Error should mention base64 decode failure"
        );
    }

    #[tokio::test]
    async fn test_local_decrypt_with_invalid_ciphertext() {
        shared::setup_test!();

        let (_temp_file, parent_key) = create_temp_kek_file();

        // Create a valid base64 string but invalid ciphertext (wrong nonce or corrupted data)
        let invalid_ciphertext = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            b"This is not valid encrypted data with proper nonce",
        );
        let invalid_encrypted_key = EncryptedDataEncryptionKey(invalid_ciphertext);

        // Try to decrypt
        let result = decrypt_data_envelope_key(&parent_key, &invalid_encrypted_key).await;

        // Should fail with a decryption error
        assert!(result.is_err(), "Should fail with invalid ciphertext");
        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(
            error_msg.contains("Local DEK decryption failed"),
            "Error should mention local decryption failure"
        );
    }

    #[tokio::test]
    async fn test_local_decrypt_with_missing_nonce() {
        shared::setup_test!();

        let (_temp_file, parent_key) = create_temp_kek_file();

        // Create encrypted data that's too short (less than 12 bytes for nonce)
        let short_data =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b"short");
        let invalid_encrypted_key = EncryptedDataEncryptionKey(short_data);

        // Try to decrypt
        let result = decrypt_data_envelope_key(&parent_key, &invalid_encrypted_key).await;

        // Should fail
        assert!(result.is_err(), "Should fail with missing nonce");
        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(
            error_msg.contains("missing nonce"),
            "Error should mention missing nonce"
        );
    }

    #[tokio::test]
    async fn test_local_encrypt_with_nonexistent_key_file() {
        shared::setup_test!();

        // Create a Local variant with empty key_bytes to test error handling
        let parent_key = EnvelopeEncryptionKeyContents::Local {
            key_id: "test-key-1".to_string(),
            key_bytes: vec![], // Empty bytes to trigger encryption failure
        };

        let test_data = "Test data";

        // Try to encrypt with invalid key (empty bytes)
        let result = encrypt_data_envelope_key(&parent_key, test_data.to_string()).await;

        // Should fail
        assert!(result.is_err(), "Should fail with invalid key");
        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(
            error_msg.contains("Invalid local KEK length")
                || error_msg.contains("Local DEK encryption failed"),
            "Error should mention invalid key or encryption failure"
        );
    }

    #[tokio::test]
    async fn test_local_encrypt_with_invalid_key_length() {
        shared::setup_test!();

        // Create a Local variant with wrong key length (16 bytes instead of 32)
        let parent_key = EnvelopeEncryptionKeyContents::Local {
            key_id: "test-key-1".to_string(),
            key_bytes: vec![0u8; 16], // Wrong length
        };

        let test_data = "Test data";

        // Try to encrypt with invalid key length
        let result = encrypt_data_envelope_key(&parent_key, test_data.to_string()).await;

        // Should fail
        assert!(result.is_err(), "Should fail with invalid key length");
        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(
            error_msg.contains("Invalid local KEK length")
                || error_msg.contains("Local DEK encryption failed"),
            "Error should mention invalid key length or encryption failure"
        );
    }

    #[tokio::test]
    async fn test_local_encryption_service_aes_gcm_roundtrip() {
        shared::setup_test!();

        let (_temp_file, parent_key) = create_temp_kek_file();

        // Generate a DEK using local encryption
        let dek_plaintext = "A".repeat(32); // 32-byte DEK
        let encrypted_dek = encrypt_data_envelope_key(&parent_key, dek_plaintext)
            .await
            .expect("Failed to encrypt DEK with local KEK");

        let now = WrappedChronoDateTime::now();
        let data_encryption_key = DataEncryptionKey {
            id: uuid::Uuid::new_v4().to_string(),
            envelope_encryption_key_id: EnvelopeEncryptionKeyId::from(parent_key.clone()),
            encrypted_data_encryption_key: encrypted_dek,
            created_at: now,
            updated_at: now,
        };

        // Create crypto service
        let crypto_service = CryptoService::new(parent_key, data_encryption_key.clone())
            .await
            .expect("Failed to create crypto service");

        let encryption_service = EncryptionService::new(crypto_service.clone());
        let decryption_service = DecryptionService::new(crypto_service);

        // Test cases
        let long_data = "A".repeat(1000);
        let test_cases = vec![
            "Simple plaintext",
            "Data with special characters: !@#$%^&*()_+-=[]{}|;:',.<>?",
            "Multi\nline\ndata\nwith\nnewlines",
            "Unicode characters: 你好世界 🌍🔐",
            long_data.as_str(),
        ];

        for test_data in test_cases {
            // Encrypt
            let encrypted = encryption_service
                .encrypt_data(test_data.to_string())
                .await
                .expect(&format!("Encryption should succeed for: {}", test_data));

            // Verify encrypted is different from plaintext
            assert_ne!(
                encrypted.0, test_data,
                "Encrypted should differ from plaintext"
            );

            // Decrypt
            let decrypted = decryption_service
                .decrypt_data(encrypted)
                .await
                .expect(&format!("Decryption should succeed for: {}", test_data));

            // Verify roundtrip
            assert_eq!(decrypted, test_data, "Roundtrip should preserve data");
        }
    }

    #[tokio::test]
    async fn test_local_encryption_produces_different_ciphertexts() {
        shared::setup_test!();

        let (_temp_file, parent_key) = create_temp_kek_file();

        // Generate a DEK using local encryption
        let dek_plaintext = "B".repeat(32); // 32-byte DEK
        let encrypted_dek = encrypt_data_envelope_key(&parent_key, dek_plaintext)
            .await
            .expect("Failed to encrypt DEK with local KEK");

        let now = WrappedChronoDateTime::now();
        let data_encryption_key = DataEncryptionKey {
            id: uuid::Uuid::new_v4().to_string(),
            envelope_encryption_key_id: EnvelopeEncryptionKeyId::from(parent_key.clone()),
            encrypted_data_encryption_key: encrypted_dek,
            created_at: now,
            updated_at: now,
        };

        let crypto_service = CryptoService::new(parent_key, data_encryption_key)
            .await
            .expect("Failed to create crypto service");

        let encryption_service = EncryptionService::new(crypto_service.clone());
        let decryption_service = DecryptionService::new(crypto_service);

        let test_data = "Same data encrypted twice";

        // Encrypt twice
        let encrypted1 = encryption_service
            .encrypt_data(test_data.to_string())
            .await
            .expect("First encryption should succeed");

        let encrypted2 = encryption_service
            .encrypt_data(test_data.to_string())
            .await
            .expect("Second encryption should succeed");

        // Ciphertexts should be different (due to random nonce)
        assert_ne!(
            encrypted1.0, encrypted2.0,
            "Multiple encryptions should produce different ciphertexts"
        );

        // But both should decrypt to same plaintext
        let decrypted1 = decryption_service
            .decrypt_data(encrypted1)
            .await
            .expect("First decryption should succeed");

        let decrypted2 = decryption_service
            .decrypt_data(encrypted2)
            .await
            .expect("Second decryption should succeed");

        assert_eq!(decrypted1, test_data);
        assert_eq!(decrypted2, test_data);
    }
}
