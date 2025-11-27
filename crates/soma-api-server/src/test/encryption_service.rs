use encryption::logic::EncryptionKeyEventSender;
use encryption::logic::crypto_services::{
    CryptoCache, CryptoService, DecryptionService, EncryptionService,
};
use encryption::logic::dek::DataEncryptionKey;
use encryption::logic::envelope::EnvelopeEncryptionKeyContents;
use encryption::repository::{DataEncryptionKeyRepositoryLike, EncryptionKeyRepositoryLike};
use shared::primitives::{SqlMigrationLoader, WrappedChronoDateTime};
use tokio::sync::broadcast;

/// Creates a temporary KEK file in the system temp directory for local encryption tests
pub fn create_temp_kek_file() -> (tempfile::TempDir, EnvelopeEncryptionKeyContents) {
    // Create temp directory in system tmp
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");

    // Generate a path for the KEK file (don't create the file yet)
    let path = temp_dir.path().join("test-kek");

    // Use get_or_create_local_envelope_encryption_key to generate and write the key
    let contents = encryption::logic::envelope::get_or_create_local_envelope_encryption_key(&path)
        .expect("Failed to create local encryption key");

    (temp_dir, contents)
}

/// Creates a test DataEncryptionKey with the given envelope encryption key contents
pub async fn create_test_dek(
    envelope_key_contents: &EnvelopeEncryptionKeyContents,
    alias: &str,
) -> (DataEncryptionKey, String) {
    use rand::RngCore;

    // Generate a random 32-byte DEK
    let mut dek_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut dek_bytes);
    // Convert raw bytes to string for encrypt_dek (will be decrypted back to bytes)
    // This is safe for tests as we're just using random bytes
    let plaintext_dek = unsafe { String::from_utf8_unchecked(dek_bytes.to_vec()) };

    // Encrypt the DEK with the envelope key
    let encrypted_dek =
        encryption::logic::envelope::encrypt_dek(envelope_key_contents, plaintext_dek)
            .await
            .expect("Failed to encrypt DEK");

    let now = WrappedChronoDateTime::now();
    let dek = DataEncryptionKey {
        id: uuid::Uuid::new_v4().to_string(),
        envelope_encryption_key_id: encryption::logic::envelope::EnvelopeEncryptionKey::from(
            envelope_key_contents.clone(),
        ),
        encrypted_data_encryption_key: encrypted_dek,
        created_at: now,
        updated_at: now,
    };
    (dek, alias.to_string())
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

/// Represents the return value from setup_test_encryption containing all necessary test components
pub struct TestEncryptionSetup {
    pub crypto_cache: CryptoCache,
    pub encryption_service: encryption::router::EncryptionService,
    pub dek_id: String,
    pub dek_alias: String,
    pub kek_contents: EnvelopeEncryptionKeyContents,
    #[allow(dead_code)]
    pub temp_dir: tempfile::TempDir,
    #[allow(dead_code)]
    pub encryption_event_tx: EncryptionKeyEventSender,
}

/// Simplified helper that creates everything needed for testing with encryption
/// Returns: TestEncryptionSetup containing crypto cache, encryption service, DEK info, and KEK info
pub async fn setup_test_encryption(alias: &str) -> TestEncryptionSetup {
    shared::setup_test!();

    let (temp_kek_file, kek_contents) = create_temp_kek_file();
    let (dek, dek_alias) = create_test_dek(&kek_contents, alias).await;

    // Create an in-memory encryption repository
    let (_enc_db, enc_conn) = shared::test_utils::repository::setup_in_memory_database(vec![
        <encryption::repository::Repository as SqlMigrationLoader>::load_sql_migrations(),
    ])
    .await
    .expect("Failed to setup encryption database");
    let encryption_repo = encryption::repository::Repository::new(enc_conn);

    // First create the envelope encryption key
    let (key_type, local_file_name, aws_arn, aws_region) = match &dek.envelope_encryption_key_id {
        encryption::logic::envelope::EnvelopeEncryptionKey::Local(local) => (
            encryption::repository::EnvelopeEncryptionKeyType::Local,
            Some(local.file_name.clone()),
            None,
            None,
        ),
        encryption::logic::envelope::EnvelopeEncryptionKey::AwsKms(aws_kms) => (
            encryption::repository::EnvelopeEncryptionKeyType::AwsKms,
            None,
            Some(aws_kms.arn.clone()),
            Some(aws_kms.region.clone()),
        ),
    };

    encryption_repo
        .create_envelope_encryption_key(&encryption::repository::CreateEnvelopeEncryptionKey {
            id: dek.envelope_encryption_key_id.id(),
            key_type,
            local_file_name,
            aws_arn,
            aws_region,
            created_at: WrappedChronoDateTime::now(),
            updated_at: WrappedChronoDateTime::now(),
        })
        .await
        .expect("Failed to create test envelope encryption key");

    // Create the DEK in the repository
    DataEncryptionKeyRepositoryLike::create_data_encryption_key(&encryption_repo, &dek)
        .await
        .expect("Failed to create test DEK in repository");

    // Create an alias for the DEK
    encryption_repo
        .create_data_encryption_key_alias(&encryption::repository::DataEncryptionKeyAlias {
            alias: dek_alias.clone(),
            data_encryption_key_id: dek.id.clone(),
            created_at: WrappedChronoDateTime::now(),
        })
        .await
        .expect("Failed to create test DEK alias");

    // Create the encryption event channel
    let (encryption_event_tx, _encryption_event_rx) = broadcast::channel(100);

    // Use the temp directory where the KEK file was created as the base path for CryptoCache
    let cache = CryptoCache::new(encryption_repo.clone(), temp_kek_file.path().to_path_buf());
    // Don't call init_crypto_cache in tests - it tries to read KEK from file which fails
    // The cache will populate on-demand when services are requested

    // Create the EncryptionService for router use
    let temp_dir = tempfile::tempdir().unwrap().path().into();
    let encryption_service = encryption::router::EncryptionService::new(
        encryption_repo,
        encryption_event_tx.clone(),
        cache.clone(),
        temp_dir,
    );

    TestEncryptionSetup {
        crypto_cache: cache,
        encryption_service,
        dek_id: dek.id,
        dek_alias,
        kek_contents,
        temp_dir: temp_kek_file,
        encryption_event_tx,
    }
}
