// Envelope encryption key management logic
// This module provides high-level operations for envelope encryption key management with event publishing

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, OsRng},
};
use rand::RngCore;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shared::{error::CommonError, primitives::{PaginationRequest, WrappedChronoDateTime}};
use utoipa::ToSchema;
use std::path::PathBuf;

use crate::logic::dek::{DataEncryptionKey, EncryptedDataEncryptionKey, DecryptedDataEncryptionKey};
use crate::repository::{EncryptionKeyRepositoryLike, CreateEnvelopeEncryptionKey, DataEncryptionKeyRepositoryLike};
use super::{EncryptionKeyEventSender, EncryptionKeyEvent};



#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, ToSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum EnvelopeEncryptionKey {
    AwsKms { arn: String, region: String },
    Local { location: String },
}

impl EnvelopeEncryptionKey {
    pub fn id(&self) -> String {
        match self {
            EnvelopeEncryptionKey::AwsKms { arn, region: _ } => arn.clone(),
            EnvelopeEncryptionKey::Local { location } => location.clone(),
        }
    }
}

#[derive(Clone, zeroize::Zeroize, zeroize::ZeroizeOnDrop)]
pub enum EnvelopeEncryptionKeyContents {
    AwsKms { arn: String, region: String },
    Local { location: String, key_bytes: Vec<u8> },
}

impl From<EnvelopeEncryptionKeyContents> for EnvelopeEncryptionKey {
    fn from(contents: EnvelopeEncryptionKeyContents) -> Self {
        match &contents {
            EnvelopeEncryptionKeyContents::AwsKms { arn, region } => {
                EnvelopeEncryptionKey::AwsKms { 
                    arn: arn.clone(),
                    region: region.clone(),
                }
            }
            EnvelopeEncryptionKeyContents::Local {
                location,
                key_bytes: _,
            } => EnvelopeEncryptionKey::Local {
                location: location.clone(),
            },
        }
    }
}

impl TryInto<libsql::Value> for EnvelopeEncryptionKey {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    fn try_into(self) -> Result<libsql::Value, Self::Error> {
        let json_value = serde_json::to_value(&self)?;
        let json_string = serde_json::to_string(&json_value)?;
        Ok(libsql::Value::Text(json_string))
    }
}

impl TryFrom<libsql::Value> for EnvelopeEncryptionKey {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    fn try_from(value: libsql::Value) -> Result<Self, Self::Error> {
        match value {
            libsql::Value::Text(s) => {
                let json_value: EnvelopeEncryptionKey = serde_json::from_str(&s)?;
                Ok(json_value)
            }
            _ => Err("Expected Text value for EnvelopeEncryptionKey".into()),
        }
    }
}

/// Extract AWS region from a KMS ARN
/// ARN format: arn:aws:kms:REGION:ACCOUNT:key/KEY-ID or arn:aws:kms:REGION:ACCOUNT:alias/ALIAS-NAME
pub fn extract_region_from_kms_arn(arn: &str) -> Result<String, CommonError> {
    // ARN format: arn:aws:kms:REGION:ACCOUNT:key/KEY-ID
    let parts: Vec<&str> = arn.split(':').collect();
    if parts.len() >= 4 && parts[0] == "arn" && parts[1] == "aws" && parts[2] == "kms" {
        Ok(parts[3].to_string())
    } else {
        Err(CommonError::Unknown(anyhow::anyhow!(
            "Invalid KMS ARN format: {arn}"
        )))
    }
}

impl libsql::FromValue for EnvelopeEncryptionKey {
    fn from_sql(val: libsql::Value) -> libsql::Result<Self>
    where
        Self: Sized,
    {
        match val {
            libsql::Value::Text(s) => {
                let json_value: EnvelopeEncryptionKey =
                    serde_json::from_str(&s).map_err(|_e| libsql::Error::InvalidColumnType)?;
                Ok(json_value)
            }
            libsql::Value::Null => Err(libsql::Error::NullValue),
            _ => Err(libsql::Error::InvalidColumnType),
        }
    }
}

// Parameter structs for API endpoints

pub type CreateEnvelopeEncryptionKeyParams = EnvelopeEncryptionKey;
pub type CreateEnvelopeEncryptionKeyResponse = EnvelopeEncryptionKey;
pub type ListEnvelopeEncryptionKeysParams = shared::primitives::PaginationRequest;
pub type ListEnvelopeEncryptionKeysResponse = shared::primitives::PaginatedResponse<EnvelopeEncryptionKey>;

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct WithEnvelopeEncryptionKeyId<T> {
    pub envelope_encryption_key_id: String,
    pub inner: T,
}

pub type DeleteEnvelopeEncryptionKeyParams = WithEnvelopeEncryptionKeyId<()>;
pub type DeleteEnvelopeEncryptionKeyResponse = ();

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct MigrateDataEncryptionKeyParams {
    /// ID of the data encryption key to migrate
    pub data_encryption_key_id: String,
    /// New envelope encryption key to migrate to
    pub to_envelope_encryption_key_id: String,
}

pub type MigrateDataEncryptionKeyResponse = ();

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct MigrateAllDataEncryptionKeysParams {
    /// New envelope encryption key to migrate all DEKs to
    pub to_envelope_encryption_key_id: String,
}

pub type MigrateAllDataEncryptionKeysResponse = ();

/// Create a new envelope encryption key
pub async fn create_envelope_encryption_key(
    on_change_tx: &EncryptionKeyEventSender,
    repo: &impl EncryptionKeyRepositoryLike,
    params: CreateEnvelopeEncryptionKeyParams,
    publish_on_change_evt: bool,
) -> Result<CreateEnvelopeEncryptionKeyResponse, CommonError> {
    let now = WrappedChronoDateTime::now();

    // Convert EnvelopeEncryptionKey to repository params using From implementation
    let create_params = CreateEnvelopeEncryptionKey::from((params.clone(), now));

    repo.create_envelope_encryption_key(&create_params).await?;

    // Publish event if publish_on_change_evt is true
    if publish_on_change_evt {
        on_change_tx
            .send(EncryptionKeyEvent::EnvelopeEncryptionKeyAdded(params.clone()))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send encryption key event: {e}"))
            })?;
    }

    Ok(params)
}

/// List envelope encryption keys with pagination
pub async fn list_envelope_encryption_keys<R>(
    repo: &R,
    params: ListEnvelopeEncryptionKeysParams,
) -> Result<ListEnvelopeEncryptionKeysResponse, CommonError>
where
    R: EncryptionKeyRepositoryLike,
{
    repo.list_envelope_encryption_keys_paginated(&params).await
}

/// Delete an envelope encryption key
/// Returns an error if there are data encryption keys still using this envelope key
pub async fn delete_envelope_encryption_key(
    on_change_tx: &EncryptionKeyEventSender,
    repo: &impl EncryptionKeyRepositoryLike,
    params: DeleteEnvelopeEncryptionKeyParams,
    publish_on_change_evt: bool,
) -> Result<DeleteEnvelopeEncryptionKeyResponse, CommonError> {
    use tracing::info;

    // Check if any data encryption keys are using this envelope key
    let envelope_key = repo
        .get_envelope_encryption_key_by_id(&params.envelope_encryption_key_id)
        .await?
        .ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!(
                "Envelope encryption key not found: {}",
                params.envelope_encryption_key_id
            ))
        })?;

    // Check for DEKs using this envelope key
    let mut page_token = None;
    loop {
        let deks = repo
            .list_data_encryption_keys(&PaginationRequest {
                page_size: 100,
                next_page_token: page_token.clone(),
            })
            .await?;

        for dek in &deks.items {
            if matches_envelope_key_id(&dek.envelope_encryption_key_id, &envelope_key) {
                return Err(CommonError::Unknown(anyhow::anyhow!(
                    "Cannot delete envelope encryption key {}: data encryption key {} is still using it",
                    params.envelope_encryption_key_id,
                    dek.id
                )));
            }
        }

        if deks.next_page_token.is_none() {
            break;
        }
        page_token = deks.next_page_token;
    }

    // Safe to delete
    repo.delete_envelope_encryption_key(&params.envelope_encryption_key_id).await?;

    info!("Deleted envelope encryption key: {}", params.envelope_encryption_key_id);

    // Publish event if publish_on_change_evt is true
    if publish_on_change_evt {
        on_change_tx
            .send(EncryptionKeyEvent::EnvelopeEncryptionKeyRemoved(params.envelope_encryption_key_id.clone()))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send encryption key event: {e}"))
            })?;
    }

    Ok(())
}

/// Migrate a data encryption key from one envelope encryption key to another
/// This involves:
/// 1. Decrypting the DEK with the old envelope key
/// 2. Re-encrypting it with the new envelope key
/// 3. Updating the database record
pub async fn migrate_data_encryption_key(
    on_change_tx: &EncryptionKeyEventSender,
    from_envelope_key_contents: &EnvelopeEncryptionKeyContents,
    repo: &(impl EncryptionKeyRepositoryLike + DataEncryptionKeyRepositoryLike),
    cache: &crate::logic::crypto_services::CryptoCache,
    params: MigrateDataEncryptionKeyParams,
    publish_on_change_evt: bool,
) -> Result<MigrateDataEncryptionKeyResponse, CommonError> {
    use tracing::info;

    // Step 1: Get the "to" envelope encryption key from the repository
    let to_envelope_key = repo
        .get_envelope_encryption_key_by_id(&params.to_envelope_encryption_key_id)
        .await?
        .ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!(
                "Envelope encryption key not found: {}",
                params.to_envelope_encryption_key_id
            ))
        })?;

    // Step 2: Convert to EnvelopeEncryptionKeyContents for encryption
    let to_envelope_key_contents = match &to_envelope_key {
        EnvelopeEncryptionKey::AwsKms { arn, region } => {
            EnvelopeEncryptionKeyContents::AwsKms {
                arn: arn.clone(),
                region: region.clone(),
            }
        }
        EnvelopeEncryptionKey::Local { location } => {
            // Load the key bytes from the file
            get_or_create_local_encryption_key(&std::path::PathBuf::from(location))?
        }
    };

    info!(
        "Migrating data encryption key {} to envelope key {}",
        params.data_encryption_key_id,
        params.to_envelope_encryption_key_id
    );

    // Step 3: Get the existing DEK
    let old_dek = DataEncryptionKeyRepositoryLike::get_data_encryption_key_by_id(
        repo,
        &params.data_encryption_key_id,
    )
    .await?
    .ok_or_else(|| {
        CommonError::Unknown(anyhow::anyhow!(
            "Data encryption key not found: {}",
            params.data_encryption_key_id
        ))
    })?;

    // Step 4: Decrypt the DEK with the old envelope key
    let decrypted_dek = decrypt_dek(
        from_envelope_key_contents,
        &old_dek.encrypted_data_encryption_key,
    )
    .await?;

    // Step 5: Re-encrypt the DEK with the new envelope key
    let new_encrypted_dek = match &to_envelope_key_contents {
        EnvelopeEncryptionKeyContents::AwsKms { arn, region } => {
            // Use AWS KMS to encrypt
            let mut config = aws_config::load_from_env().await;
            config = config
                .to_builder()
                .region(aws_config::Region::new(region.clone()))
                .build();
            let kms_client = aws_sdk_kms::Client::new(&config);

            let output = kms_client
                .encrypt()
                .key_id(arn)
                .plaintext(aws_sdk_kms::primitives::Blob::new(decrypted_dek.0.as_slice()))
                .send()
                .await
                .map_err(|e| {
                    CommonError::Unknown(anyhow::anyhow!(
                        "Failed to encrypt DEK with AWS KMS: {e}"
                    ))
                })?;

            let ciphertext_blob = output.ciphertext_blob().ok_or_else(|| {
                CommonError::Unknown(anyhow::anyhow!(
                    "AWS KMS Encrypt response did not contain ciphertext blob"
                ))
            })?;

            let encoded = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                ciphertext_blob.as_ref(),
            );
            EncryptedDataEncryptionKey(encoded)
        }
        EnvelopeEncryptionKeyContents::Local { location: _, key_bytes } => {
            // Use local AES-GCM to encrypt
            if key_bytes.len() != 32 {
                return Err(CommonError::Unknown(anyhow::anyhow!(
                    "Invalid KEK length (expected 32 bytes, got {})",
                    key_bytes.len()
                )));
            }

            let key = aes_gcm::Key::<Aes256Gcm>::from_slice(key_bytes);
            let cipher = Aes256Gcm::new(key);

            let mut nonce_bytes = [0u8; 12];
            rand::thread_rng().fill_bytes(&mut nonce_bytes);
            let nonce = Nonce::from_slice(&nonce_bytes);

            let ciphertext = cipher.encrypt(nonce, decrypted_dek.0.as_slice()).map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to encrypt DEK locally: {e}"))
            })?;

            let mut combined = Vec::with_capacity(12 + ciphertext.len());
            combined.extend_from_slice(&nonce_bytes);
            combined.extend_from_slice(&ciphertext);

            let encoded = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &combined,
            );
            EncryptedDataEncryptionKey(encoded)
        }
    };

    // Step 6: Create new DEK with the re-encrypted key
    let new_dek_id = uuid::Uuid::new_v4().to_string();
    let now = WrappedChronoDateTime::now();

    let new_dek = DataEncryptionKey {
        id: new_dek_id.clone(),
        envelope_encryption_key_id: to_envelope_key.clone(),
        encrypted_data_encryption_key: new_encrypted_dek,
        created_at: now,
        updated_at: now,
    };

    DataEncryptionKeyRepositoryLike::create_data_encryption_key(
        repo,
        &new_dek,
    )
    .await?;

    // Step 7: Delete the old DEK
    DataEncryptionKeyRepositoryLike::delete_data_encryption_key(
        repo,
        &params.data_encryption_key_id,
    )
    .await?;

    info!(
        "Successfully migrated DEK {} to {}",
        params.data_encryption_key_id, new_dek_id
    );

    // Publish events if publish_on_change_evt is true
    if publish_on_change_evt {
        // Send removed event for old DEK
        on_change_tx
            .send(EncryptionKeyEvent::DataEncryptionKeyRemoved(
                params.data_encryption_key_id.clone(),
            ))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send encryption key event: {e}"))
            })?;

        // Send added event for new DEK
        on_change_tx
            .send(EncryptionKeyEvent::DataEncryptionKeyAdded(new_dek.clone()))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send encryption key event: {e}"))
            })?;

        // Send migration event
        on_change_tx
            .send(EncryptionKeyEvent::DataEncryptionKeyMigrated {
                old_dek_id: params.data_encryption_key_id.clone(),
                new_dek_id: new_dek_id.clone(),
                from_envelope_key: old_dek.envelope_encryption_key_id.clone(),
                to_envelope_key: to_envelope_key.clone(),
            })
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send encryption key event: {e}"))
            })?;
    }

    // Invalidate cache for both old and new DEK IDs
    cache.invalidate_cache(&params.data_encryption_key_id);
    cache.invalidate_cache(&new_dek_id);

    Ok(())
}

/// Migrate a data encryption key from one envelope encryption key to another using string IDs
/// This is a convenience wrapper that looks up the envelope keys and calls migrate_data_encryption_key
pub async fn migrate_data_encryption_key_for_envelope<R>(
    from_envelope_encryption_key_id: &str,
    data_encryption_key_id: &str,
    to_envelope_encryption_key_id: &str,
    on_change_tx: &EncryptionKeyEventSender,
    repo: &R,
    cache: &crate::logic::crypto_services::CryptoCache,
    publish_on_change_evt: bool,
) -> Result<MigrateDataEncryptionKeyResponse, CommonError>
where
    R: EncryptionKeyRepositoryLike + DataEncryptionKeyRepositoryLike,
{
    // Get the from envelope encryption key
    let from_envelope_key = repo
        .get_envelope_encryption_key_by_id(from_envelope_encryption_key_id)
        .await?
        .ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!(
                "Envelope encryption key not found: {}",
                from_envelope_encryption_key_id
            ))
        })?;

    // Convert to EnvelopeEncryptionKeyContents
    let from_envelope_key_contents = match &from_envelope_key {
        EnvelopeEncryptionKey::AwsKms { arn, region } => {
            EnvelopeEncryptionKeyContents::AwsKms {
                arn: arn.clone(),
                region: region.clone(),
            }
        }
        EnvelopeEncryptionKey::Local { location } => {
            get_or_create_local_encryption_key(&std::path::PathBuf::from(location))?
        }
    };

    let params = MigrateDataEncryptionKeyParams {
        data_encryption_key_id: data_encryption_key_id.to_string(),
        to_envelope_encryption_key_id: to_envelope_encryption_key_id.to_string(),
    };

    migrate_data_encryption_key(
        on_change_tx,
        &from_envelope_key_contents,
        repo,
        cache,
        params,
        publish_on_change_evt,
    )
    .await
}

/// Migrate all data encryption keys for a given envelope encryption key to a new envelope key using string IDs
/// This is a convenience wrapper that looks up the envelope keys and calls migrate_all_data_encryption_keys
pub async fn migrate_all_data_encryption_keys_for_envelope<R>(
    from_envelope_encryption_key_id: &str,
    to_envelope_encryption_key_id: &str,
    on_change_tx: &EncryptionKeyEventSender,
    repo: &R,
    cache: &crate::logic::crypto_services::CryptoCache,
    publish_on_change_evt: bool,
) -> Result<MigrateAllDataEncryptionKeysResponse, CommonError>
where
    R: EncryptionKeyRepositoryLike + DataEncryptionKeyRepositoryLike,
{
    // Get the from envelope encryption key
    let from_envelope_key = repo
        .get_envelope_encryption_key_by_id(from_envelope_encryption_key_id)
        .await?
        .ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!(
                "Envelope encryption key not found: {}",
                from_envelope_encryption_key_id
            ))
        })?;

    // Convert to EnvelopeEncryptionKeyContents
    let from_envelope_key_contents = match &from_envelope_key {
        EnvelopeEncryptionKey::AwsKms { arn, region } => {
            EnvelopeEncryptionKeyContents::AwsKms {
                arn: arn.clone(),
                region: region.clone(),
            }
        }
        EnvelopeEncryptionKey::Local { location } => {
            get_or_create_local_encryption_key(&std::path::PathBuf::from(location))?
        }
    };

    let params = MigrateAllDataEncryptionKeysParams {
        to_envelope_encryption_key_id: to_envelope_encryption_key_id.to_string(),
    };

    migrate_all_data_encryption_keys(
        on_change_tx,
        &from_envelope_key_contents,
        &from_envelope_key,
        repo,
        cache,
        params,
        publish_on_change_evt,
    )
    .await
}

/// Migrate all data encryption keys for a given envelope encryption key to a new envelope key
pub async fn migrate_all_data_encryption_keys<R>(
    on_change_tx: &EncryptionKeyEventSender,
    from_envelope_key_contents: &EnvelopeEncryptionKeyContents,
    from_envelope_key_id: &EnvelopeEncryptionKey,
    repo: &R,
    cache: &crate::logic::crypto_services::CryptoCache,
    params: MigrateAllDataEncryptionKeysParams,
    publish_on_change_evt: bool,
) -> Result<MigrateAllDataEncryptionKeysResponse, CommonError>
where
    R: EncryptionKeyRepositoryLike + DataEncryptionKeyRepositoryLike,
{
    use tracing::info;
    use shared::primitives::PaginationRequest;

    // Get the "to" envelope encryption key from the repository
    let to_envelope_key = repo
        .get_envelope_encryption_key_by_id(&params.to_envelope_encryption_key_id)
        .await?
        .ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!(
                "Envelope encryption key not found: {}",
                params.to_envelope_encryption_key_id
            ))
        })?;

    // Convert to EnvelopeEncryptionKeyContents for encryption
    let _to_envelope_key_contents = match &to_envelope_key {
        EnvelopeEncryptionKey::AwsKms { arn, region } => {
            EnvelopeEncryptionKeyContents::AwsKms {
                arn: arn.clone(),
                region: region.clone(),
            }
        }
        EnvelopeEncryptionKey::Local { location } => {
            get_or_create_local_encryption_key(&std::path::PathBuf::from(location))?
        }
    };

    info!(
        "Migrating all data encryption keys from envelope key {} to {}",
        match from_envelope_key_id {
            EnvelopeEncryptionKey::AwsKms { arn, .. } => arn.clone(),
            EnvelopeEncryptionKey::Local { location } => location.clone(),
        },
        params.to_envelope_encryption_key_id
    );

    // Get all DEKs for the from envelope key
    use crate::repository::DataEncryptionKeyRepositoryLike;
    let mut page_token = None;
    let mut all_deks = Vec::new();

    loop {
        let deks = DataEncryptionKeyRepositoryLike::list_data_encryption_keys(
            repo,
            &PaginationRequest {
                page_size: 100,
                next_page_token: page_token.clone(),
            },
        )
        .await?;

        for dek_item in &deks.items {
            // Get full DEK to check envelope key match
            if let Some(dek) = DataEncryptionKeyRepositoryLike::get_data_encryption_key_by_id(
                repo,
                &dek_item.id,
            )
            .await?
            {
                if matches_envelope_key_id(&dek.envelope_encryption_key_id, from_envelope_key_id) {
                    all_deks.push(dek);
                }
            }
        }

        if deks.next_page_token.is_none() {
            break;
        }
        page_token = deks.next_page_token;
    }

    info!("Found {} DEKs to migrate", all_deks.len());

    // Migrate each DEK
    for old_dek in all_deks {
        let migrate_params = MigrateDataEncryptionKeyParams {
            data_encryption_key_id: old_dek.id.clone(),
            to_envelope_encryption_key_id: params.to_envelope_encryption_key_id.clone(),
        };

        migrate_data_encryption_key(
            on_change_tx,
            from_envelope_key_contents,
            repo,
            cache,
            migrate_params,
            publish_on_change_evt,
        )
        .await?;
    }

    info!("Successfully migrated all DEKs");

    Ok(())
}

/// Helper function to check if two envelope encryption keys match
pub fn matches_envelope_key_id(
    id1: &EnvelopeEncryptionKey,
    id2: &EnvelopeEncryptionKey,
) -> bool {
    match (id1, id2) {
        (
            EnvelopeEncryptionKey::AwsKms { arn: arn1, region: region1 },
            EnvelopeEncryptionKey::AwsKms { arn: arn2, region: region2 },
        ) => arn1 == arn2 && region1 == region2,
        (
            EnvelopeEncryptionKey::Local { location: loc1 },
            EnvelopeEncryptionKey::Local { location: loc2 },
        ) => loc1 == loc2,
        _ => false,
    }
}

/// Find envelope encryption key by ARN (for AWS KMS keys)
pub async fn find_envelope_encryption_key_by_arn<R>(
    repo: &R,
    arn: &str,
) -> Result<Option<EnvelopeEncryptionKey>, CommonError>
where
    R: EncryptionKeyRepositoryLike,
{
    let keys = repo.list_envelope_encryption_keys().await?;

    for key in keys {
        if let EnvelopeEncryptionKey::AwsKms { arn: stored_arn, .. } = &key {
            if stored_arn == arn {
                return Ok(Some(key));
            }
        }
    }

    Ok(None)
}

/// Find envelope encryption key by location (for local keys)
pub async fn find_envelope_encryption_key_by_location<R>(
    repo: &R,
    location: &str,
) -> Result<Option<EnvelopeEncryptionKey>, CommonError>
where
    R: EncryptionKeyRepositoryLike,
{
    let keys = repo.list_envelope_encryption_keys().await?;

    for key in keys {
        if let EnvelopeEncryptionKey::Local { location: stored_location } = &key {
            if stored_location == location {
                return Ok(Some(key));
            }
        }
    }

    Ok(None)
}

/// Generate or load a local encryption key from a file path.
/// If the file already exists, it reads and returns the key.
/// If the file doesn't exist, it generates a new 32-byte key, saves it, and returns it.
pub fn get_or_create_local_encryption_key(
    file_path: &PathBuf,
) -> Result<EnvelopeEncryptionKeyContents, CommonError> {
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
            location: file_path.to_string_lossy().to_string(),
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
        location: file_path.to_string_lossy().to_string(),
        key_bytes,
    })
}

pub async fn encrypt_dek(
    parent_encryption_key: &EnvelopeEncryptionKeyContents,
    dek: String,
) -> Result<EncryptedDataEncryptionKey, CommonError> {
    match parent_encryption_key {
        EnvelopeEncryptionKeyContents::AwsKms { arn, region } => {
            // Create AWS KMS client with specific region
            let mut config = aws_config::load_from_env().await;
            config = config.to_builder().region(aws_config::Region::new(region.clone())).build();
            let kms_client = aws_sdk_kms::Client::new(&config);

            // Encrypt the DEK using AWS KMS
            let encrypt_output = kms_client
                .encrypt()
                .key_id(arn)
                .plaintext(aws_sdk_kms::primitives::Blob::new(
                    dek.as_bytes(),
                ))
                .send()
                .await
                .map_err(|e| {
                    CommonError::Unknown(anyhow::anyhow!(
                        "Failed to encrypt DEK with AWS KMS: {e}"
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
            location: _,
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
                .encrypt(nonce, dek.as_bytes())
                .map_err(|e| {
                    CommonError::Unknown(anyhow::anyhow!("Local DEK encryption failed: {e}"))
                })?;

            // Combine nonce + ciphertext
            let mut combined = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
            combined.extend_from_slice(&nonce_bytes);
            combined.extend_from_slice(&ciphertext);

            let encoded = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &combined,
            );
            Ok(EncryptedDataEncryptionKey(encoded))
        }
    }
}

pub async fn decrypt_dek(
    parent_encryption_key: &EnvelopeEncryptionKeyContents,
    encrypted_dek: &EncryptedDataEncryptionKey,
) -> Result<DecryptedDataEncryptionKey, CommonError> {
    match parent_encryption_key {
        EnvelopeEncryptionKeyContents::AwsKms { arn, region } => {
            // Create AWS KMS client with specific region
            let mut config = aws_config::load_from_env().await;
            config = config.to_builder().region(aws_config::Region::new(region.clone())).build();
            let kms_client = aws_sdk_kms::Client::new(&config);
            
            // Decode the base64 encrypted DEK
            let ciphertext_blob = base64::Engine::decode(
                &base64::engine::general_purpose::STANDARD,
                &encrypted_dek.0,
            )
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!(
                    "Failed to decode base64 encrypted DEK: {e}"
                ))
            })?;

            // Decrypt the DEK using AWS KMS
            let decrypt_output = kms_client
                .decrypt()
                .key_id(arn)
                .ciphertext_blob(aws_sdk_kms::primitives::Blob::new(ciphertext_blob))
                .send()
                .await
                .map_err(|e| {
                    CommonError::Unknown(anyhow::anyhow!(
                        "Failed to decrypt DEK with AWS KMS: {e}"
                    ))
                })?;

            // Get the decrypted plaintext as raw bytes
            let plaintext = decrypt_output.plaintext().ok_or_else(|| {
                CommonError::Unknown(anyhow::anyhow!(
                    "AWS KMS decrypt response did not contain plaintext"
                ))
            })?;

            // Store as raw bytes (no UTF-8 conversion needed for key material)
            Ok(DecryptedDataEncryptionKey(plaintext.as_ref().to_vec()))
        }
        EnvelopeEncryptionKeyContents::Local {
            location: _,
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

            let encrypted_data = base64::Engine::decode(
                &base64::engine::general_purpose::STANDARD,
                &encrypted_dek.0,
            )
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

            Ok(DecryptedDataEncryptionKey(plaintext))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::Repository;
    use shared::primitives::SqlMigrationLoader;
    use shared::test_utils::repository::setup_in_memory_database;
    use tokio::sync::broadcast;
    use crate::logic::dek;

    const TEST_KMS_KEY_ARN: &str =
        "arn:aws:kms:eu-west-2:914788356809:alias/unsafe-github-action-soma-test-key";
    const TEST_KMS_REGION: &str = "eu-west-2";

    /// Helper function to create a temporary local key file
    fn create_temp_local_key() -> (tempfile::NamedTempFile, EnvelopeEncryptionKeyContents) {
        let mut kek_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut kek_bytes);

        let temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        std::fs::write(temp_file.path(), kek_bytes).expect("Failed to write KEK to temp file");

        let location = temp_file.path().to_string_lossy().to_string();

        let contents = EnvelopeEncryptionKeyContents::Local {
            location: location.clone(),
            key_bytes: kek_bytes.to_vec(),
        };

        (temp_file, contents)
    }

    /// Helper function to get AWS KMS key
    fn get_aws_kms_key() -> EnvelopeEncryptionKeyContents {
        EnvelopeEncryptionKeyContents::AwsKms {
            arn: TEST_KMS_KEY_ARN.to_string(),
            region: TEST_KMS_REGION.to_string(),
        }
    }

    #[tokio::test]
    async fn test_extract_region_from_kms_arn() {
        shared::setup_test!();

        // Test valid ARN with key ID
        let arn = "arn:aws:kms:eu-west-2:914788356809:key/12345678-1234-1234-1234-123456789012";
        let region = extract_region_from_kms_arn(arn).unwrap();
        assert_eq!(region, "eu-west-2");

        // Test valid ARN with alias
        let arn_alias = "arn:aws:kms:us-east-1:123456789012:alias/my-key";
        let region_alias = extract_region_from_kms_arn(arn_alias).unwrap();
        assert_eq!(region_alias, "us-east-1");

        // Test invalid ARN
        let invalid = "not-an-arn";
        assert!(extract_region_from_kms_arn(invalid).is_err());

        // Test ARN with wrong service
        let wrong_service = "arn:aws:s3:eu-west-2:123456789012:bucket/my-bucket";
        assert!(extract_region_from_kms_arn(wrong_service).is_err());
    }

    #[tokio::test]
    async fn test_matches_envelope_key_id() {
        shared::setup_test!();

        // Test AWS KMS keys match
        let key1 = EnvelopeEncryptionKey::AwsKms {
            arn: "arn:aws:kms:eu-west-2:123456789012:key/123".to_string(),
            region: "eu-west-2".to_string(),
        };
        let key2 = EnvelopeEncryptionKey::AwsKms {
            arn: "arn:aws:kms:eu-west-2:123456789012:key/123".to_string(),
            region: "eu-west-2".to_string(),
        };
        assert!(matches_envelope_key_id(&key1, &key2));

        // Test AWS KMS keys don't match (different ARN)
        let key3 = EnvelopeEncryptionKey::AwsKms {
            arn: "arn:aws:kms:eu-west-2:123456789012:key/456".to_string(),
            region: "eu-west-2".to_string(),
        };
        assert!(!matches_envelope_key_id(&key1, &key3));

        // Test AWS KMS keys don't match (different region)
        let key4 = EnvelopeEncryptionKey::AwsKms {
            arn: "arn:aws:kms:eu-west-2:123456789012:key/123".to_string(),
            region: "us-east-1".to_string(),
        };
        assert!(!matches_envelope_key_id(&key1, &key4));

        // Test local keys match
        let local1 = EnvelopeEncryptionKey::Local {
            location: "/path/to/key".to_string(),
        };
        let local2 = EnvelopeEncryptionKey::Local {
            location: "/path/to/key".to_string(),
        };
        assert!(matches_envelope_key_id(&local1, &local2));

        // Test local keys don't match
        let local3 = EnvelopeEncryptionKey::Local {
            location: "/different/path".to_string(),
        };
        assert!(!matches_envelope_key_id(&local1, &local3));

        // Test mixed types don't match
        assert!(!matches_envelope_key_id(&key1, &local1));
    }

    #[tokio::test]
    async fn test_find_envelope_encryption_key_by_arn() {
        shared::setup_test!();

        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);
        let (tx, _rx) = broadcast::channel(100);

        let aws_key = EnvelopeEncryptionKey::AwsKms {
            arn: TEST_KMS_KEY_ARN.to_string(),
            region: TEST_KMS_REGION.to_string(),
        };
        create_envelope_encryption_key(&tx, &repo, aws_key.clone(), false).await.unwrap();

        // Test finding existing key
        let found = find_envelope_encryption_key_by_arn(&repo, TEST_KMS_KEY_ARN).await.unwrap();
        assert!(found.is_some());
        assert!(matches_envelope_key_id(&found.unwrap(), &aws_key));

        // Test finding non-existent key
        let not_found = find_envelope_encryption_key_by_arn(&repo, "arn:aws:kms:us-east-1:123456789012:key/nonexistent")
            .await
            .unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_find_envelope_encryption_key_by_location() {
        shared::setup_test!();

        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);
        let (tx, _rx) = broadcast::channel(100);

        let (_temp_file, local_key_contents) = create_temp_local_key();
        let location = if let EnvelopeEncryptionKeyContents::Local { location, .. } = &local_key_contents {
            location.clone()
        } else {
            panic!("Expected local key");
        };
        let local_key = EnvelopeEncryptionKey::Local {
            location: location.clone(),
        };
        create_envelope_encryption_key(&tx, &repo, local_key.clone(), false).await.unwrap();

        // Test finding existing key
        let found = find_envelope_encryption_key_by_location(&repo, &location).await.unwrap();
        assert!(found.is_some());
        assert!(matches_envelope_key_id(&found.unwrap(), &local_key));

        // Test finding non-existent key
        let not_found = find_envelope_encryption_key_by_location(&repo, "/nonexistent/path")
            .await
            .unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_get_or_create_local_encryption_key() {
        shared::setup_test!();

        // Use a persistent temp directory so the file doesn't get deleted
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test-key");

        // Test creating new key
        let key1 = get_or_create_local_encryption_key(&path).unwrap();
        assert!(path.exists());
        assert!(matches!(key1, EnvelopeEncryptionKeyContents::Local { .. }));
        if let EnvelopeEncryptionKeyContents::Local { location, key_bytes } = &key1 {
            assert_eq!(location, &path.to_string_lossy().to_string());
            assert_eq!(key_bytes.len(), 32);
        }

        // Test loading existing key
        let key2 = get_or_create_local_encryption_key(&path).unwrap();
        assert!(matches!(key2, EnvelopeEncryptionKeyContents::Local { .. }));
        if let EnvelopeEncryptionKeyContents::Local { location: loc2, key_bytes: bytes2 } = &key2 {
            if let EnvelopeEncryptionKeyContents::Local { location: loc1, key_bytes: bytes1 } = &key1 {
                assert_eq!(loc1, loc2);
                assert_eq!(bytes1, bytes2); // Should be the same key
            }
        }
    }

    #[tokio::test]
    async fn test_create_envelope_encryption_key_local() {
        shared::setup_test!();

        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);
        let (tx, _rx) = broadcast::channel(100);

        let (_temp_file, local_key_contents) = create_temp_local_key();
        let location = match &local_key_contents {
            EnvelopeEncryptionKeyContents::Local { location, .. } => location.clone(),
            _ => panic!("Expected local key"),
        };

        let envelope_key = EnvelopeEncryptionKey::Local {
            location: location.clone(),
        };

        let result = create_envelope_encryption_key(
            &tx,
            &repo,
            envelope_key.clone(),
            false,
        )
        .await;

        assert!(result.is_ok());
        let created = result.unwrap();
        assert!(matches!(created, EnvelopeEncryptionKey::Local { .. }));

        // Verify it exists in the database
        let retrieved = repo.get_envelope_encryption_key_by_id(&location).await.unwrap();
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_create_envelope_encryption_key_aws() {
        shared::setup_test!();

        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);
        let (tx, _rx) = broadcast::channel(100);

        let envelope_key = EnvelopeEncryptionKey::AwsKms {
            arn: TEST_KMS_KEY_ARN.to_string(),
            region: TEST_KMS_REGION.to_string(),
        };

        let result = create_envelope_encryption_key(
            &tx,
            &repo,
            envelope_key.clone(),
            false,
        )
        .await;

        assert!(result.is_ok());
        let created = result.unwrap();
        assert!(matches!(created, EnvelopeEncryptionKey::AwsKms { .. }));

        // Verify it exists in the database
        let retrieved = repo.get_envelope_encryption_key_by_id(TEST_KMS_KEY_ARN).await.unwrap();
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_delete_envelope_encryption_key() {
        shared::setup_test!();

        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);
        let (tx, _rx) = broadcast::channel(100);

        let (_temp_file, local_key_contents) = create_temp_local_key();
        let location = match &local_key_contents {
            EnvelopeEncryptionKeyContents::Local { location, .. } => location.clone(),
            _ => panic!("Expected local key"),
        };

        let envelope_key = EnvelopeEncryptionKey::Local {
            location: location.clone(),
        };

        // Create the key
        create_envelope_encryption_key(&tx, &repo, envelope_key.clone(), false)
            .await
            .unwrap();

        // Delete it
        let result = delete_envelope_encryption_key(
            &tx,
            &repo,
            DeleteEnvelopeEncryptionKeyParams {
                envelope_encryption_key_id: location.clone(),
                inner: (),
            },
            false,
        )
        .await;

        assert!(result.is_ok());

        // Verify it's deleted
        let retrieved = repo.get_envelope_encryption_key_by_id(&location).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_delete_envelope_encryption_key_with_dek_fails() {
        shared::setup_test!();

        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);
        let (tx, _rx) = broadcast::channel(100);

        let (_temp_file, local_key_contents) = create_temp_local_key();
        let location = match &local_key_contents {
            EnvelopeEncryptionKeyContents::Local { location, .. } => location.clone(),
            _ => panic!("Expected local key"),
        };

        let envelope_key = EnvelopeEncryptionKey::Local {
            location: location.clone(),
        };

        // Create the envelope key
        create_envelope_encryption_key(&tx, &repo, envelope_key.clone(), false)
            .await
            .unwrap();

        // Create a DEK using this envelope key
        dek::create_data_encryption_key(
            &tx,
            &repo,
            crate::logic::dek::CreateDekParams {
                envelope_encryption_key_id: envelope_key.id(),
                inner: crate::logic::dek::CreateDekInnerParams {
                    id: Some("test-dek".to_string()),
                    encrypted_dek: None,
                },
            },
            false,
        )
        .await
        .unwrap();

        // Try to delete the envelope key - should fail
        let result = delete_envelope_encryption_key(
            &tx,
            &repo,
            DeleteEnvelopeEncryptionKeyParams {
                envelope_encryption_key_id: location.clone(),
                inner: (),
            },
            false,
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        // Check the error message using Debug format which includes the full anyhow error
        let err_msg = format!("{:?}", err);
        assert!(
            err_msg.contains("still using it") || err_msg.contains("is still using it") || err_msg.contains("Cannot delete"),
            "Error message should mention DEK is still using the envelope key. Got: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn test_migrate_data_encryption_key_local_to_local() {
        shared::setup_test!();

        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);
        let (tx, _rx) = broadcast::channel(100);

        // Create two local keys
        let (_temp_file1, local_key1_contents) = create_temp_local_key();
        let location1 = match &local_key1_contents {
            EnvelopeEncryptionKeyContents::Local { location, .. } => location.clone(),
            _ => panic!("Expected local key"),
        };
        let envelope_key1 = EnvelopeEncryptionKey::Local {
            location: location1.clone(),
        };

        let (_temp_file2, local_key2_contents) = create_temp_local_key();
        let location2 = match &local_key2_contents {
            EnvelopeEncryptionKeyContents::Local { location, .. } => location.clone(),
            _ => panic!("Expected local key"),
        };
        let envelope_key2 = EnvelopeEncryptionKey::Local {
            location: location2.clone(),
        };

        // Create both envelope keys
        create_envelope_encryption_key(&tx, &repo, envelope_key1.clone(), false)
            .await
            .unwrap();
        create_envelope_encryption_key(&tx, &repo, envelope_key2.clone(), false)
            .await
            .unwrap();

        // Create a DEK with the first key
        let dek = dek::create_data_encryption_key(
            &tx,
            &repo,
            crate::logic::dek::CreateDekParams {
                envelope_encryption_key_id: envelope_key1.id(),
                inner: crate::logic::dek::CreateDekInnerParams {
                    id: Some("test-dek-migration".to_string()),
                    encrypted_dek: None,
                },
            },
            false,
        )
        .await
        .unwrap();

        // Create cache
        let cache = crate::logic::crypto_services::CryptoCache::new(repo.clone());
        crate::logic::crypto_services::init_crypto_cache(&cache).await.unwrap();

        // Migrate to the second key
        let result = migrate_data_encryption_key(
            &tx,
            &local_key1_contents,
            &repo,
            &cache,
            MigrateDataEncryptionKeyParams {
                data_encryption_key_id: dek.id.clone(),
                to_envelope_encryption_key_id: location2.clone(),
            },
            false,
        )
        .await;

        assert!(result.is_ok());

        // Verify the old DEK is gone
        let old_dek = dek::get_data_encryption_key_by_id(&repo, &dek.id).await.unwrap();
        assert!(old_dek.is_none());

        // Verify a new DEK exists with the new envelope key
        let deks = dek::list_data_encryption_keys(
            &repo,
            crate::logic::dek::ListDekParams {
                envelope_encryption_key_id: envelope_key2.id(),
                inner: shared::primitives::PaginationRequest {
                    page_size: 100,
                    next_page_token: None,
                },
            },
        )
        .await
        .unwrap();

        let migrated_dek = deks.items.iter().find(|d| {
            matches_envelope_key_id(&d.envelope_encryption_key_id, &envelope_key2)
        });
        assert!(migrated_dek.is_some());
    }

    #[tokio::test]
    async fn test_migrate_data_encryption_key_local_to_aws() {
        shared::setup_test!();

        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);
        let (tx, _rx) = broadcast::channel(100);

        // Create local key
        let (_temp_file, local_key_contents) = create_temp_local_key();
        let location = match &local_key_contents {
            EnvelopeEncryptionKeyContents::Local { location, .. } => location.clone(),
            _ => panic!("Expected local key"),
        };
        let envelope_key_local = EnvelopeEncryptionKey::Local {
            location: location.clone(),
        };

        // Create AWS KMS key
        let aws_key_contents = get_aws_kms_key();
        let envelope_key_aws = EnvelopeEncryptionKey::AwsKms {
            arn: TEST_KMS_KEY_ARN.to_string(),
            region: TEST_KMS_REGION.to_string(),
        };

        // Create both envelope keys
        create_envelope_encryption_key(&tx, &repo, envelope_key_local.clone(), false)
            .await
            .unwrap();
        create_envelope_encryption_key(&tx, &repo, envelope_key_aws.clone(), false)
            .await
            .unwrap();

        // Create a DEK with the local key
        let dek = dek::create_data_encryption_key(
            &tx,
            &repo,
            crate::logic::dek::CreateDekParams {
                envelope_encryption_key_id: envelope_key_local.id(),
                inner: crate::logic::dek::CreateDekInnerParams {
                    id: Some("test-dek-local-to-aws".to_string()),
                    encrypted_dek: None,
                },
            },
            false,
        )
        .await
        .unwrap();

        // Create cache
        let cache = crate::logic::crypto_services::CryptoCache::new(repo.clone());
        crate::logic::crypto_services::init_crypto_cache(&cache).await.unwrap();

        // Migrate to AWS KMS
        let result = migrate_data_encryption_key(
            &tx,
            &local_key_contents,
            &repo,
            &cache,
            MigrateDataEncryptionKeyParams {
                data_encryption_key_id: dek.id.clone(),
                to_envelope_encryption_key_id: TEST_KMS_KEY_ARN.to_string(),
            },
            false,
        )
        .await;

        assert!(result.is_ok());

        // Verify the old DEK is gone
        let old_dek = dek::get_data_encryption_key_by_id(&repo, &dek.id).await.unwrap();
        assert!(old_dek.is_none());

        // Verify a new DEK exists with AWS KMS
        let deks = dek::list_data_encryption_keys(
            &repo,
            crate::logic::dek::ListDekParams {
                envelope_encryption_key_id: envelope_key_aws.id(),
                inner: shared::primitives::PaginationRequest {
                    page_size: 100,
                    next_page_token: None,
                },
            },
        )
        .await
        .unwrap();

        let migrated_dek = deks.items.iter().find(|d| {
            matches_envelope_key_id(&d.envelope_encryption_key_id, &envelope_key_aws)
        });
        assert!(migrated_dek.is_some());
    }

    #[tokio::test]
    async fn test_migrate_data_encryption_key_aws_to_aws() {
        shared::setup_test!();

        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);
        let (tx, _rx) = broadcast::channel(100);

        // Create AWS KMS key (same ARN, but we'll use it for both)
        let aws_key_contents = get_aws_kms_key();
        let envelope_key_aws = EnvelopeEncryptionKey::AwsKms {
            arn: TEST_KMS_KEY_ARN.to_string(),
            region: TEST_KMS_REGION.to_string(),
        };

        // Create envelope key
        create_envelope_encryption_key(&tx, &repo, envelope_key_aws.clone(), false)
            .await
            .unwrap();

        // Create a DEK with AWS KMS
        let dek = dek::create_data_encryption_key(
            &tx,
            &repo,
            crate::logic::dek::CreateDekParams {
                envelope_encryption_key_id: envelope_key_aws.id(),
                inner: crate::logic::dek::CreateDekInnerParams {
                    id: Some("test-dek-aws-to-aws".to_string()),
                    encrypted_dek: None,
                },
            },
            false,
        )
        .await
        .unwrap();

        // Create cache
        let cache = crate::logic::crypto_services::CryptoCache::new(repo.clone());
        crate::logic::crypto_services::init_crypto_cache(&cache).await.unwrap();

        // Migrate to the same AWS KMS key (re-encrypt)
        let result = migrate_data_encryption_key(
            &tx,
            &aws_key_contents,
            &repo,
            &cache,
            MigrateDataEncryptionKeyParams {
                data_encryption_key_id: dek.id.clone(),
                to_envelope_encryption_key_id: TEST_KMS_KEY_ARN.to_string(),
            },
            false,
        )
        .await;

        assert!(result.is_ok());

        // Verify the old DEK is gone
        let old_dek = dek::get_data_encryption_key_by_id(&repo, &dek.id).await.unwrap();
        assert!(old_dek.is_none());

        // Verify a new DEK exists
        let deks = dek::list_data_encryption_keys(
            &repo,
            crate::logic::dek::ListDekParams {
                envelope_encryption_key_id: envelope_key_aws.id(),
                inner: shared::primitives::PaginationRequest {
                    page_size: 100,
                    next_page_token: None,
                },
            },
        )
        .await
        .unwrap();

        let migrated_dek = deks.items.iter().find(|d| {
            matches_envelope_key_id(&d.envelope_encryption_key_id, &envelope_key_aws)
        });
        assert!(migrated_dek.is_some());
    }

    #[tokio::test]
    async fn test_migrate_data_encryption_key_aws_to_local() {
        shared::setup_test!();

        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);
        let (tx, _rx) = broadcast::channel(100);

        // Create AWS KMS key
        let aws_key_contents = get_aws_kms_key();
        let envelope_key_aws = EnvelopeEncryptionKey::AwsKms {
            arn: TEST_KMS_KEY_ARN.to_string(),
            region: TEST_KMS_REGION.to_string(),
        };

        // Create local key
        let (_temp_file, local_key_contents) = create_temp_local_key();
        let location = match &local_key_contents {
            EnvelopeEncryptionKeyContents::Local { location, .. } => location.clone(),
            _ => panic!("Expected local key"),
        };
        let envelope_key_local = EnvelopeEncryptionKey::Local {
            location: location.clone(),
        };

        // Create both envelope keys
        create_envelope_encryption_key(&tx, &repo, envelope_key_aws.clone(), false)
            .await
            .unwrap();
        create_envelope_encryption_key(&tx, &repo, envelope_key_local.clone(), false)
            .await
            .unwrap();

        // Create a DEK with AWS KMS
        let dek = dek::create_data_encryption_key(
            &tx,
            &repo,
            crate::logic::dek::CreateDekParams {
                envelope_encryption_key_id: envelope_key_aws.id(),
                inner: crate::logic::dek::CreateDekInnerParams {
                    id: Some("test-dek-aws-to-local".to_string()),
                    encrypted_dek: None,
                },
            },
            false,
        )
        .await
        .unwrap();

        // Create cache
        let cache = crate::logic::crypto_services::CryptoCache::new(repo.clone());
        crate::logic::crypto_services::init_crypto_cache(&cache).await.unwrap();

        // Migrate to local key
        let result = migrate_data_encryption_key(
            &tx,
            &aws_key_contents,
            &repo,
            &cache,
            MigrateDataEncryptionKeyParams {
                data_encryption_key_id: dek.id.clone(),
                to_envelope_encryption_key_id: location.clone(),
            },
            false,
        )
        .await;

        assert!(result.is_ok());

        // Verify the old DEK is gone
        let old_dek = dek::get_data_encryption_key_by_id(&repo, &dek.id).await.unwrap();
        assert!(old_dek.is_none());

        // Verify a new DEK exists with local key
        let deks = dek::list_data_encryption_keys(
            &repo,
            crate::logic::dek::ListDekParams {
                envelope_encryption_key_id: envelope_key_local.id(),
                inner: shared::primitives::PaginationRequest {
                    page_size: 100,
                    next_page_token: None,
                },
            },
        )
        .await
        .unwrap();

        let migrated_dek = deks.items.iter().find(|d| {
            matches_envelope_key_id(&d.envelope_encryption_key_id, &envelope_key_local)
        });
        assert!(migrated_dek.is_some());
    }

    #[tokio::test]
    async fn test_migrate_data_encryption_key_invalidates_cache() {
        shared::setup_test!();

        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);
        let (tx, _rx) = broadcast::channel(100);

        // Create two local keys
        let (_temp_file1, local_key1_contents) = create_temp_local_key();
        let location1 = match &local_key1_contents {
            EnvelopeEncryptionKeyContents::Local { location, .. } => location.clone(),
            _ => panic!("Expected local key"),
        };
        let envelope_key1 = EnvelopeEncryptionKey::Local {
            location: location1.clone(),
        };

        let (_temp_file2, local_key2_contents) = create_temp_local_key();
        let location2 = match &local_key2_contents {
            EnvelopeEncryptionKeyContents::Local { location, .. } => location.clone(),
            _ => panic!("Expected local key"),
        };
        let envelope_key2 = EnvelopeEncryptionKey::Local {
            location: location2.clone(),
        };

        // Create both envelope keys
        create_envelope_encryption_key(&tx, &repo, envelope_key1.clone(), false)
            .await
            .unwrap();
        create_envelope_encryption_key(&tx, &repo, envelope_key2.clone(), false)
            .await
            .unwrap();

        // Create a DEK with the first key
        let dek = dek::create_data_encryption_key(
            &tx,
            &repo,
            crate::logic::dek::CreateDekParams {
                envelope_encryption_key_id: envelope_key1.id(),
                inner: crate::logic::dek::CreateDekInnerParams {
                    id: Some("test-dek-cache-invalidation".to_string()),
                    encrypted_dek: None,
                },
            },
            false,
        )
        .await
        .unwrap();

        // Create and initialize cache
        let cache = crate::logic::crypto_services::CryptoCache::new(repo.clone());
        crate::logic::crypto_services::init_crypto_cache(&cache).await.unwrap();

        // Get encryption service - this should cache it
        let encryption_service1 = crate::logic::crypto_services::get_encryption_service(&cache, &dek.id).await.unwrap();
        let _encrypted1 = encryption_service1.encrypt_data("test message".to_string()).await.unwrap();

        // Verify it's cached by getting it again (should be the same instance)
        let encryption_service2 = crate::logic::crypto_services::get_encryption_service(&cache, &dek.id).await.unwrap();
        let _encrypted2 = encryption_service2.encrypt_data("test message 2".to_string()).await.unwrap();

        // Migrate to the second key
        let result = migrate_data_encryption_key(
            &tx,
            &local_key1_contents,
            &repo,
            &cache,
            MigrateDataEncryptionKeyParams {
                data_encryption_key_id: dek.id.clone(),
                to_envelope_encryption_key_id: location2.clone(),
            },
            false,
        )
        .await;

        assert!(result.is_ok());

        // Find the new DEK ID
        let deks = dek::list_data_encryption_keys(
            &repo,
            crate::logic::dek::ListDekParams {
                envelope_encryption_key_id: envelope_key2.id(),
                inner: shared::primitives::PaginationRequest {
                    page_size: 100,
                    next_page_token: None,
                },
            },
        )
        .await
        .unwrap();

        let migrated_dek = deks.items.iter().find(|d| {
            matches_envelope_key_id(&d.envelope_encryption_key_id, &envelope_key2)
        }).unwrap();

        // Verify old DEK cache is invalidated (should not be accessible)
        let old_dek_result = crate::logic::crypto_services::get_encryption_service(&cache, &dek.id).await;
        assert!(old_dek_result.is_err() || old_dek_result.unwrap_err().to_string().contains("not found"));

        // Verify new DEK can be accessed (cache miss, will load from DB)
        let new_encryption_service = crate::logic::crypto_services::get_encryption_service(&cache, &migrated_dek.id).await.unwrap();
        let new_encrypted = new_encryption_service.encrypt_data("new test message".to_string()).await.unwrap();
        assert!(!new_encrypted.0.is_empty());

        // Verify decryption works with new service
        let decryption_service = crate::logic::crypto_services::get_decryption_service(&cache, &migrated_dek.id).await.unwrap();
        let decrypted = decryption_service.decrypt_data(new_encrypted).await.unwrap();
        assert_eq!(decrypted, "new test message");
    }
}

