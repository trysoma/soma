//! Common test fixtures and setup utilities for integration tests.
//!
//! This module provides helper functions for setting up test contexts including:
//! - Encryption services (DEK, KEK, CryptoCache)
//! - JWK signing keys for token issuance
//! - Repository instances
//! - JWKS cache

use encryption::logic::crypto_services::{CryptoCache, init_crypto_cache};
use encryption::logic::dek::{CreateDekInnerParams, CreateDekParams};
use encryption::logic::dek_alias::{CreateAliasInnerParams, CreateAliasParams};
use encryption::logic::envelope::get_or_create_local_envelope_encryption_key;
use encryption::repository::EncryptionKeyRepositoryLike;
use shared::primitives::{SqlMigrationLoader, WrappedChronoDateTime};
use shared::test_utils::repository::setup_in_memory_database;
use tokio::sync::broadcast;

use crate::logic::jwk::{CreateJwkRequest, cache::JwksCache, create_jwk};
use crate::logic::sts::external_jwk_cache::ExternalJwksCache;
use crate::repository::Repository;

/// Default DEK alias used in tests
pub const TEST_DEK_ALIAS: &str = "test-dek-alias";

/// Complete test context with all necessary components for integration tests.
pub struct TestContext {
    /// Identity repository for user/group/key management
    pub identity_repo: Repository,
    /// Crypto cache for encryption/decryption operations
    pub crypto_cache: CryptoCache,
    /// JWKS cache for our own signing keys
    pub jwks_cache: JwksCache,
    /// External JWKS cache for validating external IdP tokens
    pub external_jwks_cache: ExternalJwksCache,
    /// DEK alias used for encryption
    pub dek_alias: String,
    /// Temporary directory holding KEK file (must be kept alive for test duration)
    #[allow(dead_code)]
    temp_dir: tempfile::TempDir,
}

impl TestContext {
    /// Create a new test context with all components initialized.
    ///
    /// This sets up:
    /// - In-memory identity database
    /// - In-memory encryption database
    /// - Local envelope encryption key (KEK)
    /// - Data encryption key (DEK) with alias
    /// - JWKS caches (internal and external)
    pub async fn new() -> Self {
        shared::setup_test!();

        // Setup identity database
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .expect("Failed to setup identity database");
        let identity_repo = Repository::new(conn);

        // Setup encryption database
        let (_encryption_db, encryption_conn) = setup_in_memory_database(vec![
            encryption::repository::Repository::load_sql_migrations(),
        ])
        .await
        .expect("Failed to setup encryption database");
        let encryption_repo = encryption::repository::Repository::new(encryption_conn);

        // Create temp dir for local keys
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
        let key_path = temp_dir.path().join("test-key");

        // Create envelope key
        let envelope_key_contents =
            get_or_create_local_envelope_encryption_key(&key_path).expect("Failed to create KEK");
        let envelope_key =
            encryption::logic::envelope::EnvelopeEncryptionKey::from(envelope_key_contents);
        let create_params = encryption::repository::CreateEnvelopeEncryptionKey::from((
            envelope_key.clone(),
            WrappedChronoDateTime::now(),
        ));
        EncryptionKeyRepositoryLike::create_envelope_encryption_key(
            &encryption_repo,
            &create_params,
        )
        .await
        .expect("Failed to create envelope encryption key");

        // Create DEK
        let (tx, _rx) = broadcast::channel(100);
        let dek = encryption::logic::dek::create_data_encryption_key(
            &tx,
            &encryption_repo,
            CreateDekParams {
                envelope_encryption_key_id: envelope_key.id(),
                inner: CreateDekInnerParams {
                    id: Some("test-dek".to_string()),
                    encrypted_dek: None,
                },
            },
            temp_dir.path(),
            false,
        )
        .await
        .expect("Failed to create DEK");

        // Create CryptoCache
        let crypto_cache = CryptoCache::new(encryption_repo.clone(), temp_dir.path().to_path_buf());
        init_crypto_cache(&crypto_cache)
            .await
            .expect("Failed to init crypto cache");

        // Create alias for the DEK
        encryption::logic::dek_alias::create_alias(
            &tx,
            &encryption_repo,
            &crypto_cache,
            CreateAliasParams {
                dek_id: dek.id.clone(),
                inner: CreateAliasInnerParams {
                    alias: TEST_DEK_ALIAS.to_string(),
                },
            },
        )
        .await
        .expect("Failed to create DEK alias");

        // Create JWKS caches
        let jwks_cache = JwksCache::new(identity_repo.clone());
        let external_jwks_cache = ExternalJwksCache::new();

        Self {
            identity_repo,
            crypto_cache,
            jwks_cache,
            external_jwks_cache,
            dek_alias: TEST_DEK_ALIAS.to_string(),
            temp_dir,
        }
    }

    /// Create a new test context with a JWK signing key already created.
    ///
    /// This is useful for tests that need to issue tokens.
    pub async fn new_with_jwk() -> Self {
        let ctx = Self::new().await;

        // Create a JWK for signing tokens
        let request = CreateJwkRequest {
            dek_alias: ctx.dek_alias.clone(),
            expires_in_days: 30,
        };
        create_jwk(
            &ctx.identity_repo,
            &ctx.crypto_cache,
            &ctx.jwks_cache,
            request,
        )
        .await
        .expect("Failed to create JWK");

        ctx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_test_context() {
        let ctx = TestContext::new().await;

        // Verify crypto services work
        let encryption_service = ctx
            .crypto_cache
            .get_encryption_service(&ctx.dek_alias)
            .await
            .expect("Failed to get encryption service");

        let test_data = "test data".to_string();
        let encrypted = encryption_service
            .encrypt_data(test_data.clone())
            .await
            .expect("Failed to encrypt");

        let decryption_service = ctx
            .crypto_cache
            .get_decryption_service(&ctx.dek_alias)
            .await
            .expect("Failed to get decryption service");

        let decrypted = decryption_service
            .decrypt_data(encrypted)
            .await
            .expect("Failed to decrypt");

        assert_eq!(decrypted, test_data);
    }

    #[tokio::test]
    async fn test_create_test_context_with_jwk() {
        let ctx = TestContext::new_with_jwk().await;

        // Verify JWK was created
        let jwks = ctx.jwks_cache.get_cached_jwks();
        assert_eq!(jwks.len(), 1);
        assert_eq!(jwks[0].kty, "RSA");
        assert_eq!(jwks[0].alg, "RS256");
    }
}
