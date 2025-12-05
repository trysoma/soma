use dashmap::DashMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shared::error::CommonError;
use std::sync::Arc;
use utoipa::ToSchema;

use crate::logic::dek::{DataEncryptionKey, DecryptedDataEncryptionKey};
use crate::logic::envelope::{
    EnvelopeEncryptionKey, EnvelopeEncryptionKeyContents, decrypt_dek,
    get_local_envelope_encryption_key, get_or_create_local_envelope_encryption_key,
};
use crate::repository::DataEncryptionKeyRepositoryLike;

#[derive(Serialize, Deserialize, Clone, JsonSchema, ToSchema)]
#[serde(transparent)]
pub struct EncryptedString(pub String);

impl std::fmt::Debug for EncryptedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EncryptedString(************)")
    }
}

// encryption services

#[derive(Clone, Debug)]
pub struct CryptoService {
    pub data_encryption_key: DataEncryptionKey,
    cached_decrypted_dek: DecryptedDataEncryptionKey,
}

impl CryptoService {
    pub async fn new(
        envelope_encryption_key_contents: EnvelopeEncryptionKeyContents,
        data_encryption_key: DataEncryptionKey,
    ) -> Result<Self, CommonError> {
        let mut envelop_key_match = false;

        if let EnvelopeEncryptionKeyContents::Local {
            file_name,
            key_bytes: _,
        } = &envelope_encryption_key_contents
            && let EnvelopeEncryptionKey::Local(local) =
                &data_encryption_key.envelope_encryption_key_id
        {
            envelop_key_match = file_name == &local.file_name;
        } else if let EnvelopeEncryptionKeyContents::AwsKms { arn, region } =
            &envelope_encryption_key_contents
            && let EnvelopeEncryptionKey::AwsKms(aws_kms) =
                &data_encryption_key.envelope_encryption_key_id
        {
            envelop_key_match = arn == &aws_kms.arn && region == &aws_kms.region;
        }

        if !envelop_key_match {
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "Envelope encryption key contents do not match data encryption key"
            )));
        }

        let decrypted_dek = decrypt_dek(
            &envelope_encryption_key_contents,
            &data_encryption_key.encrypted_data_encryption_key,
        )
        .await?;
        Ok(Self {
            data_encryption_key,
            cached_decrypted_dek: decrypted_dek,
        })
    }
}

#[derive(Clone, Debug)]
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

        // Get the decrypted DEK as bytes (already Vec<u8>)
        let key_bytes = &self.0.cached_decrypted_dek.0;
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

#[derive(Clone, Debug)]
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

        // Get the decrypted DEK as bytes (already Vec<u8>)
        let key_bytes = &self.0.cached_decrypted_dek.0;
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

/// Get a crypto service for a given data encryption key (by ID or alias)
pub async fn get_crypto_service<R: DataEncryptionKeyRepositoryLike>(
    envelope_encryption_key_contents: &EnvelopeEncryptionKeyContents,
    repo: &R,
    data_encryption_key_id_or_alias: &str,
) -> Result<CryptoService, CommonError> {
    // Use get_by_alias_or_id to support both aliases and direct IDs
    let data_encryption_key =
        crate::logic::dek_alias::get_by_alias_or_id(repo, data_encryption_key_id_or_alias).await?;

    let crypto_service = CryptoService::new(
        envelope_encryption_key_contents.clone(),
        data_encryption_key,
    )
    .await?;
    Ok(crypto_service)
}

/// Get an encryption service from a crypto service
pub fn get_encryption_service_from_crypto(
    crypto_service: &CryptoService,
) -> Result<EncryptionService, CommonError> {
    Ok(EncryptionService(crypto_service.clone()))
}

/// Get a decryption service from a crypto service
pub fn get_decryption_service_from_crypto(
    crypto_service: &CryptoService,
) -> Result<DecryptionService, CommonError> {
    Ok(DecryptionService(crypto_service.clone()))
}

/// Get an encryption service from the cache by DEK ID
pub async fn get_encryption_service(
    cache: &CryptoCache,
    dek_id: &str,
) -> Result<EncryptionService, CommonError> {
    get_encryption_service_cached(cache, dek_id).await
}

/// Get a decryption service from the cache by DEK ID
pub async fn get_decryption_service(
    cache: &CryptoCache,
    dek_id: &str,
) -> Result<DecryptionService, CommonError> {
    get_decryption_service_cached(cache, dek_id).await
}

/// Crypto cache structure for managing encryption and decryption services
#[derive(Clone)]
pub struct CryptoCache {
    encryption_services: DashMap<String, EncryptionService>,
    decryption_services: DashMap<String, DecryptionService>,
    repository: Arc<dyn DataEncryptionKeyRepositoryLike + Send + Sync>,
    local_envelope_encryption_key_path: std::path::PathBuf,
}

impl CryptoCache {
    /// Create a new empty crypto cache with the given repository and local key path
    pub fn new<R>(repo: R, local_envelope_encryption_key_path: std::path::PathBuf) -> Self
    where
        R: DataEncryptionKeyRepositoryLike + Send + Sync + 'static,
    {
        Self {
            encryption_services: DashMap::new(),
            decryption_services: DashMap::new(),
            repository: Arc::new(repo),
            local_envelope_encryption_key_path,
        }
    }

    /// Invalidate the cache for a specific DEK ID
    /// This removes both encryption and decryption services for the given DEK,
    /// forcing them to be recreated on the next access
    pub fn invalidate_cache(&self, dek_id: &str) {
        let encryption_key = format!("encryption.{dek_id}");
        let decryption_key = format!("decryption.{dek_id}");

        self.encryption_services.remove(&encryption_key);
        self.decryption_services.remove(&decryption_key);
    }

    /// Clear the entire cache
    /// This is useful when alias mappings change, as we don't know which DEK IDs are affected
    pub fn clear_cache(&self) {
        self.encryption_services.clear();
        self.decryption_services.clear();
    }

    /// Get an encryption service from the cache by DEK ID or alias
    pub async fn get_encryption_service(
        &self,
        dek_id_or_alias: &str,
    ) -> Result<EncryptionService, CommonError> {
        get_encryption_service_cached(self, dek_id_or_alias).await
    }

    /// Get a decryption service from the cache by DEK ID or alias
    pub async fn get_decryption_service(
        &self,
        dek_id_or_alias: &str,
    ) -> Result<DecryptionService, CommonError> {
        get_decryption_service_cached(self, dek_id_or_alias).await
    }
}

/// Initialize the crypto cache with all data encryption keys and their services
pub async fn init_crypto_cache(cache: &CryptoCache) -> Result<(), CommonError> {
    use shared::primitives::PaginationRequest;

    // Get all data encryption keys by paginating through them
    let mut page_token = None;
    let mut all_deks = Vec::new();

    loop {
        let deks = cache
            .repository
            .list_data_encryption_keys(&PaginationRequest {
                page_size: 100,
                next_page_token: page_token.clone(),
            })
            .await?;

        for dek_item in &deks.items {
            // Get full DEK with encrypted key
            if let Some(dek) = cache
                .repository
                .get_data_encryption_key_by_id(&dek_item.id)
                .await?
            {
                all_deks.push(dek);
            }
        }

        if deks.next_page_token.is_none() {
            break;
        }
        page_token = deks.next_page_token;
    }

    // Initialize services for each DEK
    for dek in all_deks {
        // Convert EnvelopeEncryptionKey to EnvelopeEncryptionKeyContents
        let envelope_key_contents = match &dek.envelope_encryption_key_id {
            EnvelopeEncryptionKey::AwsKms(aws_kms) => EnvelopeEncryptionKeyContents::AwsKms {
                arn: aws_kms.arn.clone(),
                region: aws_kms.region.clone(),
            },
            EnvelopeEncryptionKey::Local(local) => {
                // Resolve the filename relative to local_envelope_encryption_key_path
                let key_path = cache
                    .local_envelope_encryption_key_path
                    .join(&local.file_name);
                get_local_envelope_encryption_key(&key_path)?
            }
        };

        // Create crypto service
        let crypto_service = CryptoService::new(envelope_key_contents.clone(), dek.clone()).await?;

        // Create and cache encryption service
        let encryption_service = EncryptionService::new(crypto_service.clone());
        cache.encryption_services.insert(
            format!("encryption.{}", crypto_service.data_encryption_key.id),
            encryption_service,
        );

        // Create and cache decryption service
        let decryption_service = DecryptionService::new(crypto_service);
        cache
            .decryption_services
            .insert(format!("decryption.{}", dek.id), decryption_service);
    }

    Ok(())
}

/// Get an encryption service from the cache by DEK ID or alias
pub async fn get_encryption_service_cached(
    cache: &CryptoCache,
    dek_id_or_alias: &str,
) -> Result<EncryptionService, CommonError> {
    // Try to get from cache using the provided key (could be ID or alias)
    let cache_key = format!("encryption.{dek_id_or_alias}");
    if let Some(service) = cache.encryption_services.get(&cache_key) {
        return Ok((*service).clone());
    }

    // Not in cache, resolve the alias or ID to get the actual DEK
    let dek =
        crate::logic::dek_alias::get_by_alias_or_id(cache.repository.as_ref(), dek_id_or_alias)
            .await?;

    // Get envelope encryption key contents
    let envelope_key_contents = match &dek.envelope_encryption_key_id {
        EnvelopeEncryptionKey::AwsKms(aws_kms) => EnvelopeEncryptionKeyContents::AwsKms {
            arn: aws_kms.arn.clone(),
            region: aws_kms.region.clone(),
        },
        EnvelopeEncryptionKey::Local(local) => {
            // Resolve the filename relative to local_envelope_encryption_key_path
            let key_path = cache
                .local_envelope_encryption_key_path
                .join(&local.file_name);
            get_or_create_local_envelope_encryption_key(&key_path)?
        }
    };

    // Create crypto service
    let crypto_service = CryptoService::new(envelope_key_contents, dek.clone()).await?;

    // Create encryption service
    let encryption_service = EncryptionService::new(crypto_service.clone());

    // Cache it using the key that was requested (could be alias or ID)
    cache
        .encryption_services
        .insert(cache_key, encryption_service.clone());

    Ok(encryption_service)
}

/// Get a decryption service from the cache by DEK ID or alias
pub async fn get_decryption_service_cached(
    cache: &CryptoCache,
    dek_id_or_alias: &str,
) -> Result<DecryptionService, CommonError> {
    // Try to get from cache using the provided key (could be ID or alias)
    let cache_key = format!("decryption.{dek_id_or_alias}");
    if let Some(service) = cache.decryption_services.get(&cache_key) {
        return Ok((*service).clone());
    }

    // Not in cache, resolve the alias or ID to get the actual DEK
    let dek =
        crate::logic::dek_alias::get_by_alias_or_id(cache.repository.as_ref(), dek_id_or_alias)
            .await?;

    // Get envelope encryption key contents
    let envelope_key_contents = match &dek.envelope_encryption_key_id {
        EnvelopeEncryptionKey::AwsKms(aws_kms) => EnvelopeEncryptionKeyContents::AwsKms {
            arn: aws_kms.arn.clone(),
            region: aws_kms.region.clone(),
        },
        EnvelopeEncryptionKey::Local(local) => {
            // Resolve the filename relative to local_envelope_encryption_key_path
            let key_path = cache
                .local_envelope_encryption_key_path
                .join(&local.file_name);
            get_or_create_local_envelope_encryption_key(&key_path)?
        }
    };

    // Create crypto service
    let crypto_service = CryptoService::new(envelope_key_contents, dek.clone()).await?;

    // Create decryption service
    let decryption_service = DecryptionService::new(crypto_service);

    // Cache it using the key that was requested (could be alias or ID)
    cache
        .decryption_services
        .insert(cache_key, decryption_service.clone());

    Ok(decryption_service)
}

#[cfg(all(test, feature = "unit_test"))]
mod unit_test {
    use super::*;
    use crate::logic::dek::{CreateDekInnerParams, CreateDekParams};
    use crate::logic::envelope::{encrypt_dek, get_or_create_local_envelope_encryption_key};
    use crate::repository::{EncryptionKeyRepositoryLike, Repository};
    use shared::primitives::{SqlMigrationLoader, WrappedChronoDateTime};
    use shared::test_utils::repository::setup_in_memory_database;
    use tokio::sync::broadcast;

    #[tokio::test]
    async fn test_get_crypto_service() {
        shared::setup_test!();

        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);
        let (tx, _rx) = broadcast::channel(100);

        // Create envelope key
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
        let key_path = temp_dir.path().join("test-key");
        let envelope_key_contents = get_or_create_local_envelope_encryption_key(&key_path).unwrap();
        let envelope_key =
            crate::logic::envelope::EnvelopeEncryptionKey::from(envelope_key_contents.clone());
        let create_params = crate::repository::CreateEnvelopeEncryptionKey::from((
            envelope_key.clone(),
            WrappedChronoDateTime::now(),
        ));
        EncryptionKeyRepositoryLike::create_envelope_encryption_key(&repo, &create_params)
            .await
            .unwrap();

        // Create DEK - use the same temp_dir as base path so keys can be found
        let dek = crate::logic::dek::create_data_encryption_key(
            &tx,
            &repo,
            CreateDekParams {
                envelope_encryption_key_id: envelope_key.id(),
                inner: CreateDekInnerParams {
                    id: Some("test-dek-crypto-service".to_string()),
                    encrypted_dek: None,
                },
            },
            temp_dir.path(),
            false,
        )
        .await
        .unwrap();

        // Test getting crypto service
        let crypto_service = get_crypto_service(&envelope_key_contents, &repo, &dek.id)
            .await
            .unwrap();
        assert_eq!(crypto_service.data_encryption_key.id, dek.id);

        // Test getting crypto service for non-existent DEK
        let result = get_crypto_service(&envelope_key_contents, &repo, "non-existent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_encryption_service() {
        shared::setup_test!();

        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test-key");
        let envelope_key_contents = get_or_create_local_envelope_encryption_key(&path).unwrap();

        // Create a test DEK - generate 32 random bytes
        use rand::RngCore;
        let mut dek_bytes = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut dek_bytes);
        // Convert to string for encrypt_dek (it will be decrypted back to bytes)
        let dek_string = unsafe { String::from_utf8_unchecked(dek_bytes.clone()) };

        let encrypted_dek = encrypt_dek(&envelope_key_contents, dek_string)
            .await
            .unwrap();
        let dek = crate::logic::dek::DataEncryptionKey {
            id: "test-dek".to_string(),
            envelope_encryption_key_id: crate::logic::envelope::EnvelopeEncryptionKey::from(
                envelope_key_contents.clone(),
            ),
            encrypted_data_encryption_key: encrypted_dek,
            created_at: WrappedChronoDateTime::now(),
            updated_at: WrappedChronoDateTime::now(),
        };

        // Create crypto service
        let crypto_service = CryptoService::new(envelope_key_contents.clone(), dek)
            .await
            .unwrap();

        // Test getting encryption service
        let encryption_service = get_encryption_service_from_crypto(&crypto_service).unwrap();
        assert!(matches!(encryption_service, EncryptionService(_)));

        // Test encryption service can encrypt data
        let encrypted = encryption_service
            .encrypt_data("test message".to_string())
            .await
            .unwrap();
        assert!(!encrypted.0.is_empty());
    }

    #[tokio::test]
    async fn test_get_decryption_service() {
        shared::setup_test!();

        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test-key");
        let envelope_key_contents = get_or_create_local_envelope_encryption_key(&path).unwrap();

        // Create a test DEK - generate 32 random bytes
        use rand::RngCore;
        let mut dek_bytes = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut dek_bytes);
        // Convert to string for encrypt_dek (it will be decrypted back to bytes)
        let dek_string = unsafe { String::from_utf8_unchecked(dek_bytes.clone()) };

        let encrypted_dek = encrypt_dek(&envelope_key_contents, dek_string)
            .await
            .unwrap();
        let dek = crate::logic::dek::DataEncryptionKey {
            id: "test-dek".to_string(),
            envelope_encryption_key_id: crate::logic::envelope::EnvelopeEncryptionKey::from(
                envelope_key_contents.clone(),
            ),
            encrypted_data_encryption_key: encrypted_dek,
            created_at: WrappedChronoDateTime::now(),
            updated_at: WrappedChronoDateTime::now(),
        };

        // Create crypto service
        let crypto_service = CryptoService::new(envelope_key_contents.clone(), dek)
            .await
            .unwrap();

        // Test getting decryption service
        let decryption_service = get_decryption_service_from_crypto(&crypto_service).unwrap();
        assert!(matches!(decryption_service, DecryptionService(_)));

        // Test decryption service can decrypt data
        let encryption_service = get_encryption_service_from_crypto(&crypto_service).unwrap();
        let encrypted = encryption_service
            .encrypt_data("test message".to_string())
            .await
            .unwrap();
        let decrypted = decryption_service.decrypt_data(encrypted).await.unwrap();
        assert_eq!(decrypted, "test message");
    }

    #[tokio::test]
    async fn test_init_crypto_cache() {
        shared::setup_test!();

        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);
        let (tx, _rx) = broadcast::channel(100);

        // Create envelope key
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test-key");
        let envelope_key_contents = get_or_create_local_envelope_encryption_key(&path).unwrap();
        let envelope_key =
            crate::logic::envelope::EnvelopeEncryptionKey::from(envelope_key_contents.clone());
        let create_params = crate::repository::CreateEnvelopeEncryptionKey::from((
            envelope_key.clone(),
            WrappedChronoDateTime::now(),
        ));
        EncryptionKeyRepositoryLike::create_envelope_encryption_key(&repo, &create_params)
            .await
            .unwrap();

        // Create multiple DEKs - use temp_dir as base path
        let dek1 = crate::logic::dek::create_data_encryption_key(
            &tx,
            &repo,
            CreateDekParams {
                envelope_encryption_key_id: envelope_key.id(),
                inner: CreateDekInnerParams {
                    id: Some("test-dek-cache-1".to_string()),
                    encrypted_dek: None,
                },
            },
            temp_dir.path(),
            false,
        )
        .await
        .unwrap();

        let dek2 = crate::logic::dek::create_data_encryption_key(
            &tx,
            &repo,
            CreateDekParams {
                envelope_encryption_key_id: envelope_key.id(),
                inner: CreateDekInnerParams {
                    id: Some("test-dek-cache-2".to_string()),
                    encrypted_dek: None,
                },
            },
            temp_dir.path(),
            false,
        )
        .await
        .unwrap();

        // Initialize cache - use temp_dir as base path
        let cache = CryptoCache::new(repo.clone(), temp_dir.path().to_path_buf());
        init_crypto_cache(&cache).await.unwrap();

        // Test getting encryption service from cache
        let encryption_service1 = get_encryption_service(&cache, &dek1.id).await.unwrap();
        let encryption_service2 = get_encryption_service(&cache, &dek2.id).await.unwrap();

        // Verify they work
        let encrypted1 = encryption_service1
            .encrypt_data("message 1".to_string())
            .await
            .unwrap();
        let encrypted2 = encryption_service2
            .encrypt_data("message 2".to_string())
            .await
            .unwrap();
        assert!(!encrypted1.0.is_empty());
        assert!(!encrypted2.0.is_empty());
        assert_ne!(encrypted1.0, encrypted2.0); // Different nonces should produce different ciphertexts

        // Test getting decryption service from cache
        let decryption_service1 = get_decryption_service(&cache, &dek1.id).await.unwrap();
        let decryption_service2 = get_decryption_service(&cache, &dek2.id).await.unwrap();

        // Verify decryption works
        let decrypted1 = decryption_service1.decrypt_data(encrypted1).await.unwrap();
        let decrypted2 = decryption_service2.decrypt_data(encrypted2).await.unwrap();
        assert_eq!(decrypted1, "message 1");
        assert_eq!(decrypted2, "message 2");
    }

    #[tokio::test]
    async fn test_get_encryption_service_cached_miss() {
        shared::setup_test!();

        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);
        let (tx, _rx) = broadcast::channel(100);

        // Create envelope key
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test-key");
        let envelope_key_contents = get_or_create_local_envelope_encryption_key(&path).unwrap();
        let envelope_key =
            crate::logic::envelope::EnvelopeEncryptionKey::from(envelope_key_contents.clone());
        let create_params = crate::repository::CreateEnvelopeEncryptionKey::from((
            envelope_key.clone(),
            WrappedChronoDateTime::now(),
        ));
        EncryptionKeyRepositoryLike::create_envelope_encryption_key(&repo, &create_params)
            .await
            .unwrap();

        // Initialize cache - use temp_dir as base path
        let cache = CryptoCache::new(repo.clone(), temp_dir.path().to_path_buf());
        init_crypto_cache(&cache).await.unwrap();

        // Create a new DEK after cache initialization - use temp_dir as base path
        let dek2 = crate::logic::dek::create_data_encryption_key(
            &tx,
            &repo,
            CreateDekParams {
                envelope_encryption_key_id: envelope_key.id(),
                inner: CreateDekInnerParams {
                    id: Some("test-dek-cache-miss-2".to_string()),
                    encrypted_dek: None,
                },
            },
            temp_dir.path(),
            false,
        )
        .await
        .unwrap();

        // Should be able to get it (cache miss, loads from DB)
        let encryption_service = get_encryption_service(&cache, &dek2.id).await.unwrap();
        let encrypted = encryption_service
            .encrypt_data("test".to_string())
            .await
            .unwrap();
        assert!(!encrypted.0.is_empty());

        // Now it should be cached
        let encryption_service2 = get_encryption_service(&cache, &dek2.id).await.unwrap();
        let encrypted2 = encryption_service2
            .encrypt_data("test2".to_string())
            .await
            .unwrap();
        assert!(!encrypted2.0.is_empty());
    }

    #[tokio::test]
    async fn test_get_encryption_service_not_found() {
        shared::setup_test!();

        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        // Initialize cache
        let cache = CryptoCache::new(repo, std::path::PathBuf::from("/tmp/test-keys"));
        init_crypto_cache(&cache).await.unwrap();

        // Try to get non-existent DEK
        let result = get_encryption_service(&cache, "non-existent-dek").await;
        assert!(result.is_err());
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(err_msg.contains("not found") || err_msg.contains("Data encryption key"));
    }

    #[tokio::test]
    async fn test_get_decryption_service_not_found() {
        shared::setup_test!();

        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        // Initialize cache
        let cache = CryptoCache::new(repo, std::path::PathBuf::from("/tmp/test-keys"));
        init_crypto_cache(&cache).await.unwrap();

        // Try to get non-existent DEK
        let result = get_decryption_service(&cache, "non-existent-dek").await;
        assert!(result.is_err());
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(err_msg.contains("not found") || err_msg.contains("Data encryption key"));
    }
}
