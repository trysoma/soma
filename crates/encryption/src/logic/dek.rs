// Data encryption key (DEK) management logic
// This module provides high-level operations for DEK management with event publishing

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shared::{
    error::CommonError,
    primitives::{PaginationRequest, WrappedChronoDateTime},
};
use utoipa::ToSchema;

use super::{EncryptionKeyEvent, EncryptionKeyEventSender};
#[cfg(all(test, feature = "integration_test"))]
use crate::logic::envelope::EnvelopeEncryptionKeyAwsKms;
#[cfg(all(test, feature = "unit_test"))]
use crate::logic::envelope::EnvelopeEncryptionKeyLocal;
use crate::logic::envelope::{
    EnvelopeEncryptionKey, EnvelopeEncryptionKeyContents, WithEnvelopeEncryptionKeyId,
};
use crate::repository::DataEncryptionKeyRepositoryLike;

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct ImportDekParamsInner {
    pub id: Option<String>,
    pub encrypted_data_encryption_key: EncryptedDataEncryptionKey,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateDekInnerParams {
    pub id: Option<String>,
    pub encrypted_dek: Option<String>,
}

pub type CreateDekParams = WithEnvelopeEncryptionKeyId<CreateDekInnerParams>;
pub type CreateDataEncryptionKeyResponse = DataEncryptionKey;
pub type ImportDekParams = WithEnvelopeEncryptionKeyId<ImportDekParamsInner>;
pub type ImportDekResponse = DataEncryptionKey;
pub type ListDekParams = WithEnvelopeEncryptionKeyId<shared::primitives::PaginationRequest>;
pub type ListDekResponse = shared::primitives::PaginatedResponse<DataEncryptionKeyListItem>;
pub type DeleteDekParams = String;
pub type DeleteDekResponse = ();
pub type MigrateDekResponse = ();

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, ToSchema)]
#[serde(transparent)]
pub struct EncryptedDataEncryptionKey(pub String);

#[derive(Debug, Clone, zeroize::Zeroize, zeroize::ZeroizeOnDrop)]
pub struct DecryptedDataEncryptionKey(pub Vec<u8>);

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

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct DataEncryptionKey {
    pub id: String,
    pub envelope_encryption_key_id: EnvelopeEncryptionKey,
    pub encrypted_data_encryption_key: EncryptedDataEncryptionKey,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct DataEncryptionKeyListItem {
    pub id: String,
    pub envelope_encryption_key_id: EnvelopeEncryptionKey,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

// Low-level function to create a DEK (without event publishing)
pub(crate) async fn create_data_encryption_key_internal<R>(
    repo: &R,
    params: CreateDekParams,
    local_envelope_encryption_key_path: &std::path::Path,
) -> Result<CreateDataEncryptionKeyResponse, CommonError>
where
    R: DataEncryptionKeyRepositoryLike + crate::repository::EncryptionKeyRepositoryLike,
{
    use aes_gcm::{
        Aes256Gcm, Nonce,
        aead::{Aead, KeyInit, OsRng},
    };
    use rand::RngCore;
    use shared::primitives::WrappedChronoDateTime;

    // Look up the envelope encryption key from the database
    let envelope_key = repo
        .get_envelope_encryption_key_by_id(&params.envelope_encryption_key_id)
        .await?
        .ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!(
                "Envelope encryption key not found: {}",
                params.envelope_encryption_key_id
            ))
        })?;

    // Get the envelope key contents based on its type
    let key_encryption_key = match &envelope_key {
        EnvelopeEncryptionKey::AwsKms(aws_kms) => EnvelopeEncryptionKeyContents::AwsKms {
            arn: aws_kms.arn.clone(),
            region: aws_kms.region.clone(),
        },
        EnvelopeEncryptionKey::Local(local) => {
            // Resolve the filename relative to local_envelope_encryption_key_path
            let key_path = local_envelope_encryption_key_path.join(&local.file_name);
            crate::logic::envelope::get_or_create_local_envelope_encryption_key(&key_path)?
        }
    };

    let id = params
        .inner
        .id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let encrypted_data_encryption_key = match params.inner.encrypted_dek {
        Some(existing) => EncryptedDataEncryptionKey(existing),
        None => match &key_encryption_key {
            EnvelopeEncryptionKeyContents::AwsKms { arn, region } => {
                // --- AWS KMS path ---
                let mut config = aws_config::load_from_env().await;
                config = config
                    .to_builder()
                    .region(aws_config::Region::new(region.clone()))
                    .build();
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

                let encoded = base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    ciphertext_blob.as_ref(),
                );
                EncryptedDataEncryptionKey(encoded)
            }

            EnvelopeEncryptionKeyContents::Local {
                file_name,
                key_bytes,
            } => {
                // --- Local path (no AWS involved) ---
                if key_bytes.len() != 32 {
                    return Err(CommonError::Unknown(anyhow::anyhow!(
                        "Invalid KEK length in {} (expected 32 bytes, got {})",
                        file_name,
                        key_bytes.len()
                    )));
                }

                // Generate random 32-byte DEK
                let mut dek = [0u8; 32];
                rand::thread_rng().fill_bytes(&mut dek);

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

                let encoded =
                    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &combined);
                EncryptedDataEncryptionKey(encoded)
            }
        },
    };

    let now = WrappedChronoDateTime::now();

    let data_encryption_key = DataEncryptionKey {
        id,
        envelope_encryption_key_id: envelope_key,
        encrypted_data_encryption_key,
        created_at: now,
        updated_at: now,
    };

    DataEncryptionKeyRepositoryLike::create_data_encryption_key(repo, &data_encryption_key).await?;

    Ok(data_encryption_key)
}

/// Create a new data encryption key
pub async fn create_data_encryption_key<R>(
    on_change_tx: &EncryptionKeyEventSender,
    repo: &R,
    params: CreateDekParams,
    local_envelope_encryption_key_path: &std::path::Path,
    publish_on_change_evt: bool,
) -> Result<CreateDataEncryptionKeyResponse, CommonError>
where
    R: DataEncryptionKeyRepositoryLike + crate::repository::EncryptionKeyRepositoryLike,
{
    let dek = create_data_encryption_key_internal(repo, params, local_envelope_encryption_key_path)
        .await?;

    // Publish event if requested - include encrypted key value in event
    if publish_on_change_evt {
        on_change_tx
            .send(EncryptionKeyEvent::DataEncryptionKeyAdded(dek.clone()))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send encryption key event: {e}"))
            })?;
    }

    Ok(dek)
}

/// Import an already encrypted data encryption key
pub async fn import_data_encryption_key<R>(
    on_change_tx: &EncryptionKeyEventSender,
    repo: &R,
    params: ImportDekParams,
    local_envelope_encryption_key_path: &std::path::Path,
    publish_on_change_evt: bool,
) -> Result<ImportDekResponse, CommonError>
where
    R: DataEncryptionKeyRepositoryLike + crate::repository::EncryptionKeyRepositoryLike,
{
    use shared::primitives::WrappedChronoDateTime;

    // Look up the envelope encryption key from the database
    let envelope_key = repo
        .get_envelope_encryption_key_by_id(&params.envelope_encryption_key_id)
        .await?
        .ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!(
                "Envelope encryption key not found: {}",
                params.envelope_encryption_key_id
            ))
        })?;

    // Get the envelope key contents based on its type
    let key_encryption_key = match &envelope_key {
        EnvelopeEncryptionKey::AwsKms(aws_kms) => EnvelopeEncryptionKeyContents::AwsKms {
            arn: aws_kms.arn.clone(),
            region: aws_kms.region.clone(),
        },
        EnvelopeEncryptionKey::Local(local) => {
            // Resolve the filename relative to local_envelope_encryption_key_path
            let key_path = local_envelope_encryption_key_path.join(&local.file_name);
            crate::logic::envelope::get_local_envelope_encryption_key(&key_path)?
        }
    };

    // Attempt to decrypt the DEK to validate it before saving
    crate::logic::envelope::decrypt_dek(&key_encryption_key, &params.inner.encrypted_data_encryption_key)
        .await
        .map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!(
                "Failed to decrypt imported data encryption key - it may be encrypted with a different envelope key: {e}"
            ))
        })?;

    let id = params
        .inner
        .id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let now = WrappedChronoDateTime::now();

    let data_encryption_key = DataEncryptionKey {
        id: id.clone(),
        envelope_encryption_key_id: envelope_key,
        encrypted_data_encryption_key: params.inner.encrypted_data_encryption_key,
        created_at: now,
        updated_at: now,
    };

    DataEncryptionKeyRepositoryLike::create_data_encryption_key(repo, &data_encryption_key).await?;

    // Publish event if requested - include encrypted key value in event
    if publish_on_change_evt {
        on_change_tx
            .send(EncryptionKeyEvent::DataEncryptionKeyAdded(
                data_encryption_key.clone(),
            ))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send encryption key event: {e}"))
            })?;
    }

    Ok(data_encryption_key)
}

/// List data encryption keys filtered by envelope encryption key ID
pub async fn list_data_encryption_keys<R>(
    repo: &R,
    params: ListDekParams,
) -> Result<ListDekResponse, CommonError>
where
    R: DataEncryptionKeyRepositoryLike + crate::repository::EncryptionKeyRepositoryLike,
{
    // Look up the envelope encryption key from the database
    let envelope_key = repo
        .get_envelope_encryption_key_by_id(&params.envelope_encryption_key_id)
        .await?
        .ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!(
                "Envelope encryption key not found: {}",
                params.envelope_encryption_key_id
            ))
        })?;

    list_data_encryption_keys_by_envelope_key_id(repo, &envelope_key, params.inner).await
}

/// List data encryption keys filtered by envelope encryption key ID
/// Note: This function paginates through all DEKs and filters by envelope key ID.
/// For better performance, consider adding a repository method that filters at the database level.
pub async fn list_data_encryption_keys_by_envelope_key_id<R>(
    repo: &R,
    envelope_encryption_key_id: &EnvelopeEncryptionKey,
    params: PaginationRequest,
) -> Result<ListDekResponse, CommonError>
where
    R: DataEncryptionKeyRepositoryLike,
{
    // Get all DEKs and filter by envelope key ID
    // We'll need to paginate through all results to filter properly
    let mut page_token = None;
    let mut all_matching_items = Vec::new();

    loop {
        let deks = repo
            .list_data_encryption_keys(&PaginationRequest {
                page_size: 100,
                next_page_token: page_token.clone(),
            })
            .await?;

        for dek_item in &deks.items {
            if matches_envelope_key_id(
                &dek_item.envelope_encryption_key_id,
                envelope_encryption_key_id,
            ) {
                all_matching_items.push(dek_item.clone());
            }
        }

        if deks.next_page_token.is_none() {
            break;
        }
        page_token = deks.next_page_token;
    }

    // Apply pagination manually
    let page_size = params.page_size as usize;
    let start_idx = if let Some(token) = &params.next_page_token {
        token.parse::<usize>().unwrap_or(0)
    } else {
        0
    };

    let end_idx = (start_idx + page_size).min(all_matching_items.len());
    let items = all_matching_items[start_idx..end_idx].to_vec();
    let next_page_token = if end_idx < all_matching_items.len() {
        Some(end_idx.to_string())
    } else {
        None
    };

    Ok(ListDekResponse {
        items,
        next_page_token,
    })
}

/// Helper function to check if two envelope encryption keys match
fn matches_envelope_key_id(id1: &EnvelopeEncryptionKey, id2: &EnvelopeEncryptionKey) -> bool {
    match (id1, id2) {
        (EnvelopeEncryptionKey::AwsKms(aws_kms1), EnvelopeEncryptionKey::AwsKms(aws_kms2)) => {
            aws_kms1.arn == aws_kms2.arn && aws_kms1.region == aws_kms2.region
        }
        (EnvelopeEncryptionKey::Local(local1), EnvelopeEncryptionKey::Local(local2)) => {
            local1.file_name == local2.file_name
        }
        _ => false,
    }
}

/// Get a data encryption key by ID
pub async fn get_data_encryption_key_by_id<R>(
    repo: &R,
    id: &str,
) -> Result<Option<DataEncryptionKey>, CommonError>
where
    R: DataEncryptionKeyRepositoryLike,
{
    repo.get_data_encryption_key_by_id(id).await
}

/// Delete a data encryption key
pub async fn delete_data_encryption_key<R>(
    on_change_tx: &EncryptionKeyEventSender,
    repo: &R,
    id: DeleteDekParams,
    publish_on_change_evt: bool,
) -> Result<DeleteDekResponse, CommonError>
where
    R: DataEncryptionKeyRepositoryLike,
{
    repo.delete_data_encryption_key(&id).await?;

    // Publish event if requested
    if publish_on_change_evt {
        on_change_tx
            .send(EncryptionKeyEvent::DataEncryptionKeyRemoved(id))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send encryption key event: {e}"))
            })?;
    }

    Ok(())
}

#[cfg(all(test, feature = "unit_test"))]
mod unit_test {
    use super::*;
    use crate::repository::{EncryptionKeyRepositoryLike, Repository};
    use rand::RngCore;
    use shared::primitives::{PaginationRequest, SqlMigrationLoader};
    use shared::test_utils::repository::setup_in_memory_database;
    use tokio::sync::broadcast;

    /// Helper function to create a temporary local key file
    fn create_temp_local_key() -> (tempfile::NamedTempFile, EnvelopeEncryptionKeyContents) {
        let mut kek_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut kek_bytes);

        let temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        std::fs::write(temp_file.path(), kek_bytes).expect("Failed to write KEK to temp file");

        let file_name = temp_file.path().to_string_lossy().to_string();

        let contents = EnvelopeEncryptionKeyContents::Local {
            file_name: file_name.clone(),
            key_bytes: kek_bytes.to_vec(),
        };

        (temp_file, contents)
    }

    #[tokio::test]
    async fn test_create_data_encryption_key_with_local_key() {
        shared::setup_test!();

        // Setup in-memory database
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);
        let (tx, _rx) = broadcast::channel(100);

        // Create a local key
        let (_temp_file, local_key) = create_temp_local_key();

        // Create envelope key first
        let envelope_key = EnvelopeEncryptionKey::Local(EnvelopeEncryptionKeyLocal {
            file_name: match &local_key {
                EnvelopeEncryptionKeyContents::Local { file_name, .. } => file_name.clone(),
                _ => panic!("Expected local key"),
            },
        });
        let create_params = crate::repository::CreateEnvelopeEncryptionKey::from((
            envelope_key.clone(),
            shared::primitives::WrappedChronoDateTime::now(),
        ));
        repo.create_envelope_encryption_key(&create_params)
            .await
            .unwrap();

        // Create a data encryption key
        let dek = create_data_encryption_key(
            &tx,
            &repo,
            CreateDekParams {
                envelope_encryption_key_id: envelope_key.id(),
                inner: CreateDekInnerParams {
                    id: Some("test-dek-local".to_string()),
                    encrypted_dek: None,
                },
            },
            &std::path::PathBuf::from("/tmp/test-keys"),
            false,
        )
        .await
        .unwrap();

        assert_eq!(dek.id, "test-dek-local");
        assert!(matches!(
            dek.envelope_encryption_key_id,
            EnvelopeEncryptionKey::Local(_)
        ));

        // Verify the DEK exists in the database
        let retrieved_dek = get_data_encryption_key_by_id(&repo, &dek.id).await.unwrap();
        assert!(retrieved_dek.is_some());
    }

    #[tokio::test]
    async fn test_delete_data_encryption_key_with_local_key() {
        shared::setup_test!();

        // Setup in-memory database
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);
        let (tx, _rx) = broadcast::channel(100);

        // Create a local key
        let (_temp_file, local_key) = create_temp_local_key();

        // Create envelope key first
        let envelope_key = EnvelopeEncryptionKey::Local(EnvelopeEncryptionKeyLocal {
            file_name: match &local_key {
                EnvelopeEncryptionKeyContents::Local { file_name, .. } => file_name.clone(),
                _ => panic!("Expected local key"),
            },
        });
        let create_params = crate::repository::CreateEnvelopeEncryptionKey::from((
            envelope_key.clone(),
            shared::primitives::WrappedChronoDateTime::now(),
        ));
        repo.create_envelope_encryption_key(&create_params)
            .await
            .unwrap();

        // Create a data encryption key
        let dek = create_data_encryption_key(
            &tx,
            &repo,
            CreateDekParams {
                envelope_encryption_key_id: envelope_key.id(),
                inner: CreateDekInnerParams {
                    id: Some("test-dek-local-delete".to_string()),
                    encrypted_dek: None,
                },
            },
            &std::path::PathBuf::from("/tmp/test-keys"),
            false,
        )
        .await
        .unwrap();

        // Delete the DEK
        delete_data_encryption_key(&tx, &repo, dek.id.clone(), false)
            .await
            .unwrap();

        // Verify the DEK is deleted
        let deleted_dek = get_data_encryption_key_by_id(&repo, &dek.id).await.unwrap();
        assert!(deleted_dek.is_none());
    }

    #[tokio::test]
    async fn test_get_data_encryption_key_by_id() {
        shared::setup_test!();

        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);
        let (tx, _rx) = broadcast::channel(100);

        let (_temp_file, local_key_contents) = create_temp_local_key();
        let envelope_key =
            if let EnvelopeEncryptionKeyContents::Local { file_name, .. } = &local_key_contents {
                EnvelopeEncryptionKey::Local(EnvelopeEncryptionKeyLocal {
                    file_name: file_name.clone(),
                })
            } else {
                panic!("Expected local key");
            };
        let create_params = crate::repository::CreateEnvelopeEncryptionKey::from((
            envelope_key.clone(),
            shared::primitives::WrappedChronoDateTime::now(),
        ));
        repo.create_envelope_encryption_key(&create_params)
            .await
            .unwrap();

        let dek = create_data_encryption_key(
            &tx,
            &repo,
            CreateDekParams {
                envelope_encryption_key_id: envelope_key.id(),
                inner: CreateDekInnerParams {
                    id: Some("test-dek-get".to_string()),
                    encrypted_dek: None,
                },
            },
            &std::path::PathBuf::from("/tmp/test-keys"),
            false,
        )
        .await
        .unwrap();

        // Test getting existing DEK
        let retrieved = get_data_encryption_key_by_id(&repo, &dek.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, dek.id);

        // Test getting non-existent DEK
        let not_found = get_data_encryption_key_by_id(&repo, "non-existent")
            .await
            .unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_list_data_encryption_keys() {
        shared::setup_test!();

        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);
        let (tx, _rx) = broadcast::channel(100);

        // Create a local key
        let (_temp_file, local_key) = create_temp_local_key();

        // Create envelope key first
        let envelope_key = EnvelopeEncryptionKey::Local(EnvelopeEncryptionKeyLocal {
            file_name: match &local_key {
                EnvelopeEncryptionKeyContents::Local { file_name, .. } => file_name.clone(),
                _ => panic!("Expected local key"),
            },
        });
        let create_params = crate::repository::CreateEnvelopeEncryptionKey::from((
            envelope_key.clone(),
            shared::primitives::WrappedChronoDateTime::now(),
        ));
        repo.create_envelope_encryption_key(&create_params)
            .await
            .unwrap();

        // Create multiple DEKs
        let dek1 = create_data_encryption_key(
            &tx,
            &repo,
            CreateDekParams {
                envelope_encryption_key_id: envelope_key.id(),
                inner: CreateDekInnerParams {
                    id: Some("test-dek-1".to_string()),
                    encrypted_dek: None,
                },
            },
            &std::path::PathBuf::from("/tmp/test-keys"),
            false,
        )
        .await
        .unwrap();

        let dek2 = create_data_encryption_key(
            &tx,
            &repo,
            CreateDekParams {
                envelope_encryption_key_id: envelope_key.id(),
                inner: CreateDekInnerParams {
                    id: Some("test-dek-2".to_string()),
                    encrypted_dek: None,
                },
            },
            &std::path::PathBuf::from("/tmp/test-keys"),
            false,
        )
        .await
        .unwrap();

        // List DEKs
        let deks = list_data_encryption_keys(
            &repo,
            ListDekParams {
                envelope_encryption_key_id: envelope_key.id(),
                inner: PaginationRequest {
                    page_size: 100,
                    next_page_token: None,
                },
            },
        )
        .await
        .unwrap();

        assert!(deks.items.len() >= 2);
        let ids: Vec<String> = deks.items.iter().map(|d| d.id.clone()).collect();
        assert!(ids.contains(&dek1.id));
        assert!(ids.contains(&dek2.id));
    }
}

#[cfg(all(test, feature = "integration_test"))]
mod integration_test {
    use super::*;
    use crate::repository::{EncryptionKeyRepositoryLike, Repository};
    use shared::primitives::SqlMigrationLoader;
    use shared::test_utils::repository::setup_in_memory_database;
    use tokio::sync::broadcast;

    const TEST_KMS_KEY_ARN: &str =
        "arn:aws:kms:eu-west-2:914788356809:alias/unsafe-github-action-soma-test-key";
    const TEST_KMS_REGION: &str = "eu-west-2";

    #[tokio::test]
    async fn test_create_data_encryption_key_with_aws_kms() {
        shared::setup_test!();

        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);
        let (tx, _rx) = broadcast::channel(100);

        // Create envelope key first
        let envelope_key = EnvelopeEncryptionKey::AwsKms(EnvelopeEncryptionKeyAwsKms {
            arn: TEST_KMS_KEY_ARN.to_string(),
            region: TEST_KMS_REGION.to_string(),
        });
        let create_params = crate::repository::CreateEnvelopeEncryptionKey::from((
            envelope_key.clone(),
            shared::primitives::WrappedChronoDateTime::now(),
        ));
        repo.create_envelope_encryption_key(&create_params)
            .await
            .unwrap();

        // Create a data encryption key
        let dek = create_data_encryption_key(
            &tx,
            &repo,
            CreateDekParams {
                envelope_encryption_key_id: envelope_key.id(),
                inner: CreateDekInnerParams {
                    id: Some("test-dek-aws".to_string()),
                    encrypted_dek: None,
                },
            },
            &std::path::PathBuf::from("/tmp/test-keys"),
            false,
        )
        .await
        .unwrap();

        assert_eq!(dek.id, "test-dek-aws");
        assert!(matches!(
            dek.envelope_encryption_key_id,
            EnvelopeEncryptionKey::AwsKms(_)
        ));

        // Verify the DEK exists in the database
        let retrieved_dek = get_data_encryption_key_by_id(&repo, &dek.id).await.unwrap();
        assert!(retrieved_dek.is_some());
    }
}
