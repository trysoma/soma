#[cfg(test)]
pub mod encryption_helpers {
    use encryption::{
        CryptoCache, CryptoService, DataEncryptionKey, DecryptionService,
        EncryptionService, EnvelopeEncryptionKeyContents, CreateDekParams, CreateDekInnerParams,
    };
    use shared::primitives::WrappedChronoDateTime;

    /// Creates a temporary KEK file for local encryption tests
    pub fn create_temp_kek_file() -> (tempfile::NamedTempFile, EnvelopeEncryptionKeyContents) {
        use rand::RngCore;
        let mut kek_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut kek_bytes);

        let temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        std::fs::write(temp_file.path(), kek_bytes).expect("Failed to write KEK to temp file");

        let location = temp_file
            .path()
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("test-key")
            .to_string();

        let contents = EnvelopeEncryptionKeyContents::Local {
            location,
            key_bytes: kek_bytes.to_vec(),
        };

        (temp_file, contents)
    }

    /// Creates a test DataEncryptionKey with the given envelope encryption key contents
    pub async fn create_test_dek(
        envelope_key_contents: &EnvelopeEncryptionKeyContents,
        alias: &str,
    ) -> DataEncryptionKey {
        use rand::RngCore;

        // Generate a random 32-byte DEK
        let mut dek_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut dek_bytes);
        let plaintext_dek = String::from_utf8(dek_bytes.to_vec())
            .unwrap_or_else(|_| base64::Engine::encode(&base64::engine::general_purpose::STANDARD, dek_bytes));

        // Encrypt the DEK with the envelope key
        let encrypted_dek = encryption::encrypt_dek(envelope_key_contents, plaintext_dek)
            .await
            .expect("Failed to encrypt DEK");

        let now = WrappedChronoDateTime::now();
        DataEncryptionKey {
            id: uuid::Uuid::new_v4().to_string(),
            alias: alias.to_string(),
            envelope_encryption_key_id: encryption::EnvelopeEncryptionKey::from(envelope_key_contents.clone()),
            encrypted_data_encryption_key: encrypted_dek,
            created_at: now,
            updated_at: now,
        }
    }

    /// Creates test encryption and decryption services
    pub async fn create_test_crypto_services(
        envelope_key_contents: &EnvelopeEncryptionKeyContents,
        dek: &DataEncryptionKey,
    ) -> (EncryptionService, DecryptionService) {
        let crypto_service = CryptoService::new(envelope_key_contents.clone(), dek.clone())
            .await
            .expect("Failed to create crypto service");

        let encryption_service = EncryptionService::new(crypto_service.clone());
        let decryption_service = DecryptionService::new(crypto_service);

        (encryption_service, decryption_service)
    }

    /// Simplified helper that creates everything needed for testing with encryption
    /// Returns: (CryptoCache, DataEncryptionKey, EnvelopeEncryptionKeyContents, KEK temp file)
    pub async fn setup_test_encryption(
        alias: &str,
    ) -> (CryptoCache, DataEncryptionKey, EnvelopeEncryptionKeyContents, tempfile::NamedTempFile) {
        shared::setup_test!();

        let (temp_kek_file, kek_contents) = create_temp_kek_file();
        let dek = create_test_dek(&kek_contents, alias).await;

        // Create an in-memory encryption repository
        let db_path = format!(":memory:?alias={}", uuid::Uuid::new_v4());
        let encryption_repo = encryption::repository::sqlite::Repository::new(&db_path)
            .await
            .expect("Failed to create encryption repository");

        // Create the DEK in the repository
        encryption_repo
            .create_data_encryption_key(&encryption::WithEnvelopeEncryptionKeyId {
                envelope_encryption_key_id: dek.envelope_encryption_key_id.clone(),
                inner: CreateDekInnerParams {
                    id: Some(dek.id.clone()),
                    encrypted_dek: Some(dek.encrypted_data_encryption_key.0.clone()),
                },
            })
            .await
            .expect("Failed to create test DEK in repository");

        let cache = CryptoCache::new(encryption_repo);
        encryption::init_crypto_cache(&cache)
            .await
            .expect("Failed to initialize crypto cache");

        (cache, dek, kek_contents, temp_kek_file)
    }
}
