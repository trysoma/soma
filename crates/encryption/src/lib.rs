pub mod logic;
pub mod repository;
pub mod router;

#[cfg(all(test, feature = "unit_test"))]
mod unit_test {
    use crate::logic::envelope::{EnvelopeEncryptionKeyContents, decrypt_dek, encrypt_dek};

    // Helper function to create a temporary KEK file for local encryption tests
    fn create_temp_kek_file() -> (tempfile::NamedTempFile, EnvelopeEncryptionKeyContents) {
        use rand::RngCore;
        let mut kek_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut kek_bytes);

        let temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        std::fs::write(temp_file.path(), kek_bytes).expect("Failed to write KEK to temp file");

        let file_name = temp_file
            .path()
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("test-key")
            .to_string();

        let contents = EnvelopeEncryptionKeyContents::Local {
            file_name,
            key_bytes: kek_bytes.to_vec(),
        };

        (temp_file, contents)
    }

    #[tokio::test]
    async fn test_encrypt_dek_with_local() {
        shared::setup_test!();

        let (_temp_file, parent_key) = create_temp_kek_file();
        let test_data = "This is a test DEK for local envelope encryption";

        // Encrypt the DEK
        let result = encrypt_dek(&parent_key, test_data.to_string()).await;

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
            let encrypted = encrypt_dek(&parent_key, test_data.to_string())
                .await
                .expect("Encryption should succeed");

            // Decrypt
            let decrypted = decrypt_dek(&parent_key, &encrypted)
                .await
                .expect("Decryption should succeed");

            // Verify
            assert_eq!(
                decrypted.0,
                test_data.as_bytes(),
                "Roundtrip should preserve data for: {test_data}"
            );
        }
    }
}

#[cfg(all(test, feature = "integration_test"))]
mod integration_test {
    use crate::logic::crypto_services::{CryptoService, DecryptionService, EncryptionService};
    use crate::logic::dek::{DataEncryptionKey, EncryptedDataEncryptionKey};
    use crate::logic::envelope::{
        EnvelopeEncryptionKey, EnvelopeEncryptionKeyContents, decrypt_dek, encrypt_dek,
    };

    const TEST_KMS_KEY_ARN: &str =
        "arn:aws:kms:eu-west-2:914788356809:alias/unsafe-github-action-soma-test-key";
    const TEST_KMS_REGION: &str = "eu-west-2";

    #[tokio::test]
    async fn test_encrypt_dek_with_aws_kms() {
        shared::setup_test!();

        // Test data
        let test_data = "This is a test DEK for envelope encryption";
        let parent_key = EnvelopeEncryptionKeyContents::AwsKms {
            arn: TEST_KMS_KEY_ARN.to_string(),
            region: TEST_KMS_REGION.to_string(),
        };

        // Encrypt the DEK
        let result = encrypt_dek(&parent_key, test_data.to_string()).await;

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
    async fn test_decrypt_dek_with_aws_kms() {
        shared::setup_test!();

        // Test data
        let test_data = "This is a test DEK for envelope encryption";
        let parent_key = EnvelopeEncryptionKeyContents::AwsKms {
            arn: TEST_KMS_KEY_ARN.to_string(),
            region: TEST_KMS_REGION.to_string(),
        };

        // First, encrypt the DEK
        let encrypted_key = encrypt_dek(&parent_key, test_data.to_string())
            .await
            .expect("Encryption should succeed");

        // Now decrypt it
        let result = decrypt_dek(&parent_key, &encrypted_key).await;

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
            region: TEST_KMS_REGION.to_string(),
        };

        for test_data in test_cases {
            // Encrypt
            let encrypted = encrypt_dek(&parent_key, test_data.to_string())
                .await
                .expect("Encryption should succeed");

            // Decrypt
            let decrypted = decrypt_dek(&parent_key, &encrypted)
                .await
                .expect("Decryption should succeed");

            // Verify
            assert_eq!(
                decrypted.0,
                test_data.as_bytes(),
                "Roundtrip should preserve data for: {test_data}"
            );
        }
    }

    #[tokio::test]
    async fn test_encryption_service_aes_gcm_roundtrip() {
        shared::setup_test!();

        // Generate a 32-byte (256-bit) key using AWS KMS
        let parent_key = EnvelopeEncryptionKeyContents::AwsKms {
            arn: TEST_KMS_KEY_ARN.to_string(),
            region: TEST_KMS_REGION.to_string(),
        };

        let mut config = aws_config::load_from_env().await;
        config = config
            .to_builder()
            .region(aws_config::Region::new(TEST_KMS_REGION.to_string()))
            .build();
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

        let now = shared::primitives::WrappedChronoDateTime::now();
        let data_encryption_key = DataEncryptionKey {
            id: uuid::Uuid::new_v4().to_string(),
            envelope_encryption_key_id: EnvelopeEncryptionKey::from(parent_key.clone()),
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
                .unwrap_or_else(|_| panic!("Encryption should succeed for: {test_data}"));

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
                .unwrap_or_else(|_| panic!("Decryption should succeed for: {test_data}"));

            // Verify roundtrip
            assert_eq!(
                decrypted, test_data,
                "Decrypted data should match original plaintext"
            );
        }
    }
}
