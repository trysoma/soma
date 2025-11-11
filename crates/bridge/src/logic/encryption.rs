
use std::path::PathBuf;

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, OsRng},
};
use base64::Engine;
use rand::RngCore;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use shared::{
    error::CommonError,
    primitives::{
        PaginatedResponse, PaginationRequest, WrappedChronoDateTime, WrappedJsonValue,
    },
};
use utoipa::ToSchema;

use crate::logic::{controller::{get_credential_controller, get_provider_controller, WithCredentialControllerTypeId, WithProviderControllerTypeId}, OnConfigChangeEvt, OnConfigChangeTx};

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

pub struct EncryptionService(pub CryptoService);

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
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Encryption failed: {e}")))?;

        // Prepend the nonce to the ciphertext: [nonce (12 bytes) | ciphertext]
        let mut result = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        // Base64 encode the result
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &result);
        Ok(EncryptedString(encoded))
    }
}

pub struct DecryptionService(pub CryptoService);

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
                |e| CommonError::Unknown(anyhow::anyhow!("Failed to decode base64: {e}")),
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
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Decryption failed: {e}")))?;

        // Convert to UTF-8 string
        let result = String::from_utf8(plaintext).map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!("Invalid UTF-8 in decrypted data: {e}"))
        })?;

        Ok(result)
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
                        "Failed to encrypt data envelope key with AWS KMS: {e}"
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

            let key = aes_gcm::Key::<Aes256Gcm>::from_slice(key_bytes);
            let cipher = Aes256Gcm::new(key);

            let mut nonce_bytes = [0u8; 12];
            OsRng.fill_bytes(&mut nonce_bytes);
            let nonce = Nonce::from_slice(&nonce_bytes);

            let ciphertext = cipher
                .encrypt(nonce, data_envelope_key.as_bytes())
                .map_err(|e| {
                    CommonError::Unknown(anyhow::anyhow!("Local envelope encryption failed: {e}"))
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
                    "Failed to decode base64 encrypted data envelope key: {e}"
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
                        "Failed to decrypt data envelope key with AWS KMS: {e}"
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

            let key = aes_gcm::Key::<Aes256Gcm>::from_slice(key_bytes);
            let cipher = Aes256Gcm::new(key);

            let encrypted_data = base64::engine::general_purpose::STANDARD
                .decode(&encrypted_data_envelope_key.0)
                .map_err(|e| {
                    CommonError::Unknown(anyhow::anyhow!(
                        "Failed to decode base64 encrypted DEK: {e}"
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
                CommonError::Unknown(anyhow::anyhow!("Local DEK decryption failed: {e}"))
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
    publish_on_change_evt: bool,
) -> Result<CreateDataEncryptionKeyResponse, CommonError> {
    let id = params.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

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
                            "Failed to generate data key with AWS KMS: {e}"
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

                let key = aes_gcm::Key::<Aes256Gcm>::from_slice(key_bytes);
                let cipher = Aes256Gcm::new(key);

                let mut nonce_bytes = [0u8; 12];
                OsRng.fill_bytes(&mut nonce_bytes);
                let nonce = Nonce::from_slice(&nonce_bytes);

                let ciphertext = cipher.encrypt(nonce, dek.as_slice()).map_err(|e| {
                    CommonError::Unknown(anyhow::anyhow!("Failed to encrypt DEK locally: {e}"))
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

    if publish_on_change_evt {
        on_config_change_tx
            .send(OnConfigChangeEvt::DataEncryptionKeyAdded(
                data_encryption_key.clone(),
            ))
            .await?;
    }

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

pub type DeleteDataEncryptionKeyParams = String;
pub type DeleteDataEncryptionKeyResponse = ();

pub async fn delete_data_encryption_key(
    on_config_change_tx: &OnConfigChangeTx,
    repo: &impl crate::repository::ProviderRepositoryLike,
    id: DeleteDataEncryptionKeyParams,
    publish_on_change_evt: bool,
) -> Result<DeleteDataEncryptionKeyResponse, CommonError> {
    repo.delete_data_encryption_key(&id).await?;
    if publish_on_change_evt {
        on_config_change_tx
            .send(OnConfigChangeEvt::DataEncryptionKeyRemoved(id.clone()))
            .await?;
    }
    Ok(())
}


pub async fn get_crypto_service(
    envelope_encryption_key_contents: &EnvelopeEncryptionKeyContents,
    repo: &impl crate::repository::ProviderRepositoryLike,
    data_encryption_key_id: &String,
) -> Result<CryptoService, CommonError> {
    let data_encryption_key = repo
        .get_data_encryption_key_by_id(data_encryption_key_id)
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

pub fn get_encryption_service(
    crypto_service: &CryptoService,
) -> Result<EncryptionService, CommonError> {
    Ok(EncryptionService(crypto_service.clone()))
}

pub fn get_decryption_service(
    crypto_service: &CryptoService,
) -> Result<DecryptionService, CommonError> {
    Ok(DecryptionService(crypto_service.clone()))
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

#[cfg(test)]
mod tests {
    use super::*;
    use shared::primitives::SqlMigrationLoader;

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

    #[tokio::test]
    async fn test_create_data_encryption_key_with_id() {
        shared::setup_test!();

        let repo = { let (_db, conn) = shared::test_utils::repository::setup_in_memory_database(vec![crate::repository::Repository::load_sql_migrations()]).await.unwrap(); crate::repository::Repository::new(conn) };
        let (tx, _rx) = tokio::sync::mpsc::channel(10);

        let kek = EnvelopeEncryptionKeyContents::Local {
            key_id: "test-key".to_string(),
            key_bytes: vec![0u8; 32],
        };

        let params = CreateDataEncryptionKeyParams {
            id: Some("test-dek-id".to_string()),
            encrypted_data_envelope_key: None,
        };

        let result = create_data_encryption_key(&kek, &tx, &repo, params, false).await;
        assert!(result.is_ok());

        let dek = result.unwrap();
        assert_eq!(dek.id, "test-dek-id");
        assert!(!dek.encrypted_data_encryption_key.0.is_empty());
    }

    #[tokio::test]
    async fn test_create_data_encryption_key_generates_id() {
        shared::setup_test!();

        let repo = { let (_db, conn) = shared::test_utils::repository::setup_in_memory_database(vec![crate::repository::Repository::load_sql_migrations()]).await.unwrap(); crate::repository::Repository::new(conn) };
        let (tx, _rx) = tokio::sync::mpsc::channel(10);

        let kek = EnvelopeEncryptionKeyContents::Local {
            key_id: "test-key".to_string(),
            key_bytes: vec![0u8; 32],
        };

        let params = CreateDataEncryptionKeyParams {
            id: None,
            encrypted_data_envelope_key: None,
        };

        let result = create_data_encryption_key(&kek, &tx, &repo, params, false).await;
        assert!(result.is_ok());

        let dek = result.unwrap();
        assert!(!dek.id.is_empty());
        // Should be a valid UUID
        assert!(uuid::Uuid::parse_str(&dek.id).is_ok());
    }

    #[tokio::test]
    async fn test_create_data_encryption_key_with_existing_encrypted_key() {
        shared::setup_test!();

        let repo = { let (_db, conn) = shared::test_utils::repository::setup_in_memory_database(vec![crate::repository::Repository::load_sql_migrations()]).await.unwrap(); crate::repository::Repository::new(conn) };
        let (tx, _rx) = tokio::sync::mpsc::channel(10);

        let kek = EnvelopeEncryptionKeyContents::Local {
            key_id: "test-key".to_string(),
            key_bytes: vec![0u8; 32],
        };

        let existing_encrypted = EncryptedDataEncryptionKey("already-encrypted-value".to_string());

        let params = CreateDataEncryptionKeyParams {
            id: Some("test-id".to_string()),
            encrypted_data_envelope_key: Some(existing_encrypted.clone()),
        };

        let result = create_data_encryption_key(&kek, &tx, &repo, params, false).await;
        assert!(result.is_ok());

        let dek = result.unwrap();
        assert_eq!(dek.encrypted_data_encryption_key.0, existing_encrypted.0);
    }

    #[tokio::test]
    async fn test_delete_data_encryption_key_publishes_event() {
        shared::setup_test!();

        let repo = { let (_db, conn) = shared::test_utils::repository::setup_in_memory_database(vec![crate::repository::Repository::load_sql_migrations()]).await.unwrap(); crate::repository::Repository::new(conn) };
        let (tx, mut rx) = tokio::sync::mpsc::channel(10);

        // Create a DEK first
        let kek = EnvelopeEncryptionKeyContents::Local {
            key_id: "test-key".to_string(),
            key_bytes: vec![0u8; 32],
        };

        let create_params = CreateDataEncryptionKeyParams {
            id: Some("dek-to-delete".to_string()),
            encrypted_data_envelope_key: None,
        };

        create_data_encryption_key(&kek, &tx, &repo, create_params, false).await.unwrap();

        // Delete it with event publishing enabled
        let result = delete_data_encryption_key(&tx, &repo, "dek-to-delete".to_string(), true).await;
        assert!(result.is_ok());

        // Verify event was published
        let event = rx.try_recv();
        assert!(event.is_ok());
        match event.unwrap() {
            crate::logic::OnConfigChangeEvt::DataEncryptionKeyRemoved(id) => {
                assert_eq!(id, "dek-to-delete");
            }
            _ => panic!("Expected DataEncryptionKeyRemoved event"),
        }
    }

    #[tokio::test]
    async fn test_delete_data_encryption_key_no_event_when_disabled() {
        shared::setup_test!();

        let repo = { let (_db, conn) = shared::test_utils::repository::setup_in_memory_database(vec![crate::repository::Repository::load_sql_migrations()]).await.unwrap(); crate::repository::Repository::new(conn) };
        let (tx, mut rx) = tokio::sync::mpsc::channel(10);

        // Create a DEK first
        let kek = EnvelopeEncryptionKeyContents::Local {
            key_id: "test-key".to_string(),
            key_bytes: vec![0u8; 32],
        };

        let create_params = CreateDataEncryptionKeyParams {
            id: Some("dek-to-delete".to_string()),
            encrypted_data_envelope_key: None,
        };

        create_data_encryption_key(&kek, &tx, &repo, create_params, false).await.unwrap();

        // Delete it with event publishing disabled
        let result = delete_data_encryption_key(&tx, &repo, "dek-to-delete".to_string(), false).await;
        assert!(result.is_ok());

        // Verify no event was published
        let event = rx.try_recv();
        assert!(event.is_err());
    }

}