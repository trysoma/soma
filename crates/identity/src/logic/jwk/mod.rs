pub mod cache;

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::Utc;
use encryption::logic::CryptoCache;
use pkcs1::EncodeRsaPrivateKey;
use pkcs8::EncodePublicKey;
use rsa::{RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};
use shared::{
    error::CommonError,
    primitives::{PaginatedResponse, PaginationRequest, WrappedChronoDateTime},
};
use uuid::Uuid;

use crate::logic::{internal_token_issuance::JwtSigningKey, jwk::cache::JwksCache};
use crate::repository::UserRepositoryLike;

use utoipa::ToSchema;

/// Default DEK alias used for JWK encryption
pub const DEFAULT_JWK_DEK_ALIAS: &str = "default";

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateJwkRequest {
    pub dek_alias: String,
    #[serde(default = "default_expires_in_days")]
    pub expires_in_days: u32,
}

fn default_expires_in_days() -> u32 {
    30
}

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct JwkResponse {
    pub kid: String,
    pub public_key: String,
    pub expires_at: WrappedChronoDateTime,
    pub created_at: WrappedChronoDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct JwksResponse {
    pub keys: Vec<Jwk>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Jwk {
    pub kty: String,
    pub kid: String,
    #[serde(rename = "use")]
    pub use_: String,
    pub alg: String,
    pub n: String,
    pub e: String,
}

// Parameter types for router endpoints
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InvalidateJwkParams {
    pub kid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListJwksParams {
    pub pagination: PaginationRequest,
}

pub type ListJwksResponse = PaginatedResponse<JwkResponse>;
pub type InvalidateJwkResponse = ();
pub type GetJwksResponse = JwksResponse;

/// Create a new JWT signing key
pub async fn create_jwk<R>(
    repository: &R,
    crypto_cache: &CryptoCache,
    jwks_cache: &JwksCache,
    request: CreateJwkRequest,
) -> Result<JwkResponse, CommonError>
where
    R: UserRepositoryLike,
{
    let encryption_service = crypto_cache
        .get_encryption_service(&request.dek_alias)
        .await?;
    // Generate RSA key pair (2048 bits)
    // Use OsRng instead of thread_rng() because thread_rng() returns a !Send type
    let private_key = RsaPrivateKey::new(&mut rand::rngs::OsRng, 2048)
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to generate RSA key: {e}")))?;

    // Get public key from private key
    let public_key = RsaPublicKey::from(&private_key);

    // Encode private key as PEM (PKCS#1 format)
    let private_key_pem = private_key
        .to_pkcs1_pem(pkcs1::LineEnding::LF)
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to encode private key: {e}")))?;

    // Encode public key as PEM (PKCS#8 format)
    let public_key_pem = public_key
        .to_public_key_pem(pkcs8::LineEnding::LF)
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to encode public key: {e}")))?;

    // Encrypt the private key
    let private_key_pem_str = private_key_pem.to_string();
    let encrypted_private_key = encryption_service.encrypt_data(private_key_pem_str).await?;

    // Generate kid (key ID)
    let kid = Uuid::new_v4().to_string();

    // Calculate expiration time
    let now = Utc::now();
    let expires_at = now + chrono::Duration::days(request.expires_in_days as i64);

    // Store in database
    let public_key_pem_str = public_key_pem.to_string();
    let create_params = JwtSigningKey {
        kid: kid.clone(),
        encrypted_private_key: encrypted_private_key.0,
        expires_at: WrappedChronoDateTime::new(expires_at),
        public_key: public_key_pem_str.clone(),
        dek_alias: request.dek_alias,
        invalidated: false,
        created_at: WrappedChronoDateTime::new(now),
        updated_at: WrappedChronoDateTime::new(now),
    };

    repository.create_jwt_signing_key(&create_params).await?;

    // Add to cache
    let jwk = pem_to_jwk(&public_key_pem_str, &kid)?;
    jwks_cache.add_jwk(jwk);

    Ok(JwkResponse {
        kid,
        public_key: public_key_pem_str,
        expires_at: create_params.expires_at,
        created_at: create_params.created_at,
    })
}

// ============================================================================
// Logic Functions
// ============================================================================

/// Invalidate a JWT signing key
pub async fn invalidate_jwk<R>(
    repository: &R,
    jwks_cache: &JwksCache,
    params: InvalidateJwkParams,
) -> Result<InvalidateJwkResponse, CommonError>
where
    R: UserRepositoryLike,
{
    repository.invalidate_jwt_signing_key(&params.kid).await?;
    jwks_cache.invalidate_jwk(&params.kid);
    Ok(())
}

/// List JWT signing keys with pagination
pub async fn list_jwks<R>(
    repository: &R,
    pagination: &PaginationRequest,
) -> Result<ListJwksResponse, CommonError>
where
    R: UserRepositoryLike,
{
    let result = repository.list_jwt_signing_keys(pagination).await?;

    let items: Vec<JwkResponse> = result
        .items
        .into_iter()
        .map(|key| JwkResponse {
            kid: key.kid,
            public_key: key.public_key,
            expires_at: key.expires_at,
            created_at: key.created_at,
        })
        .collect();

    Ok(PaginatedResponse {
        items,
        next_page_token: result.next_page_token,
    })
}

/// Get JWKS (JSON Web Key Set) for all non-expired, non-invalidated keys
/// This function checks the cache first, then falls back to repository if cache is empty
pub async fn get_jwks<R>(
    repository: &R,
    jwks_cache: &JwksCache,
) -> Result<GetJwksResponse, CommonError>
where
    R: UserRepositoryLike,
{
    // Try to get from cache first
    let cached_jwks = jwks_cache.get_cached_jwks();
    if !cached_jwks.is_empty() {
        return Ok(JwksResponse { keys: cached_jwks });
    }

    // Cache miss, load from repository
    let jwks_response = get_jwks_direct(repository).await?;

    // Refresh cache with new data
    if let Err(e) = jwks_cache.refresh_cache().await {
        tracing::warn!("Failed to refresh JWKS cache: {:?}", e);
    }

    Ok(jwks_response)
}

/// Get JWKS directly from repository without cache (used internally)
pub(crate) async fn get_jwks_direct<R>(repository: &R) -> Result<JwksResponse, CommonError>
where
    R: UserRepositoryLike,
{
    // Get all keys (we'll filter expired ones)
    let mut next_page_token: Option<String> = None;
    let mut collected_results = Vec::new();
    loop {
        let pagination = PaginationRequest {
            page_size: 1000, // Large page size to get all keys
            next_page_token,
        };
        let result = repository.list_jwt_signing_keys(&pagination).await?;
        collected_results.extend(result.items);
        if result.next_page_token.is_none() {
            break;
        }
        next_page_token = result.next_page_token;
    }
    let now = Utc::now();
    let keys: Vec<Jwk> = collected_results
        .into_iter()
        .filter(|key| {
            // Filter out invalidated keys
            !key.invalidated
        })
        .filter(|key| {
            // Filter out expired keys
            key.expires_at.get_inner() > &now
        })
        .filter_map(|key| {
            // Convert PEM to JWK format
            pem_to_jwk(&key.public_key, &key.kid).ok()
        })
        .collect();

    Ok(JwksResponse { keys })
}

// ============================================================================
// JWK Background Task and Startup Checks
// ============================================================================

/// Check if JWKs exist on startup, create one if none exist
pub async fn check_jwks_exists_on_start<R>(
    repository: &R,
    crypto_cache: &CryptoCache,
    jwks_cache: &JwksCache,
    default_dek_alias: &str,
) -> Result<(), CommonError>
where
    R: UserRepositoryLike,
{
    let pagination = PaginationRequest {
        page_size: 1,
        next_page_token: None,
    };

    let result = repository.list_jwt_signing_keys(&pagination).await?;

    if result.items.is_empty() {
        tracing::info!("No JWKs found, creating initial JWK...");
        let request = CreateJwkRequest {
            dek_alias: default_dek_alias.to_string(),
            expires_in_days: 30,
        };
        create_jwk(repository, crypto_cache, jwks_cache, request).await?;
        tracing::info!("Initial JWK created successfully");
    } else {
        tracing::info!("JWKs already exist, skipping initial creation");
        // Initialize cache with existing JWKs
        jwks_cache.refresh_cache().await?;
    }

    Ok(())
}

/// Background task that periodically checks and creates JWKs if needed
/// This function is designed to be called in its own tokio::spawn
pub async fn jwk_rotation_task<R>(
    repo: R,
    crypto_cache: CryptoCache,
    jwks_cache: JwksCache,
    default_dek_alias: String,
    mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
) where
    R: UserRepositoryLike,
{
    use tokio::time::{Duration, interval};

    let mut timer = interval(Duration::from_secs(10 * 60)); // 10 minutes

    loop {
        tokio::select! {
            _ = timer.tick() => {
                tracing::info!("Starting JWK rotation check");

                if let Err(e) = process_jwk_rotation(
                    &repo,
                    &crypto_cache,
                    &jwks_cache,
                    &default_dek_alias,
                )
                .await
                {
                    tracing::error!("Error processing JWK rotation: {:?}", e);
                }

                tracing::info!("Completed JWK rotation check");
            }
            _ = shutdown_rx.recv() => {
                tracing::info!("JWK rotation task shutdown requested");
                break;
            }
        }
    }

    tracing::info!("JWK rotation task stopped");
}

/// Process JWK rotation - check if any JWKs expire in more than 5 days, create one if not
pub async fn process_jwk_rotation<R>(
    repository: &R,
    crypto_cache: &CryptoCache,
    jwks_cache: &JwksCache,
    default_dek_alias: &str,
) -> Result<(), CommonError>
where
    R: UserRepositoryLike,
{
    // Calculate the threshold: 5 days from now
    let now = Utc::now();
    let threshold = now + chrono::Duration::days(5);

    // Get all JWKs
    let pagination = PaginationRequest {
        page_size: 1000, // Large page size to get all keys
        next_page_token: None,
    };

    let result = repository.list_jwt_signing_keys(&pagination).await?;

    // Check if any JWK expires more than 5 days from now
    let has_valid_key = result
        .items
        .iter()
        .any(|key| !key.invalidated && key.expires_at.get_inner() > &threshold);

    if !has_valid_key {
        tracing::info!("No JWKs expire in more than 5 days, creating new JWK...");
        let request = CreateJwkRequest {
            dek_alias: default_dek_alias.to_string(),
            expires_in_days: 30,
        };
        create_jwk(repository, crypto_cache, jwks_cache, request).await?;
        tracing::info!("New JWK created successfully");
    } else {
        tracing::debug!("JWKs with expiration > 5 days exist, no action needed");
    }

    // Remove expired/invalidated keys from cache
    jwks_cache.remove_expired().await?;

    Ok(())
}

/// Convert PEM public key to JWK format
fn pem_to_jwk(pem: &str, kid: &str) -> Result<Jwk, CommonError> {
    use pkcs1::DecodeRsaPublicKey;
    use pkcs8::DecodePublicKey;

    // Parse PEM public key (try PKCS#8 first, then PKCS#1)
    let public_key = RsaPublicKey::from_public_key_pem(pem)
        .or_else(|_| RsaPublicKey::from_pkcs1_pem(pem))
        .map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!("Failed to parse PEM public key: {e}"))
        })?;

    // Extract modulus (n) and exponent (e)
    // RsaPublicKey stores the key components internally
    // We need to encode them to DER/ASN.1 and extract from there, or use the public key's internal representation
    // For now, let's use the public key's encoding and parse it
    use rsa::traits::PublicKeyParts;

    let n = public_key.n();
    let e = public_key.e();

    // Convert to base64url-encoded strings
    let n_bytes = n.to_bytes_be();
    let e_bytes = e.to_bytes_be();

    let n_b64 = URL_SAFE_NO_PAD.encode(&n_bytes);
    let e_b64 = URL_SAFE_NO_PAD.encode(&e_bytes);

    Ok(Jwk {
        kty: "RSA".to_string(),
        kid: kid.to_string(),
        use_: "sig".to_string(),
        alg: "RS256".to_string(),
        n: n_b64,
        e: e_b64,
    })
}

#[cfg(all(test, feature = "unit_test"))]
mod unit_test {
    use super::*;
    use crate::repository::Repository;
    use encryption::logic::crypto_services::{CryptoCache, init_crypto_cache};
    use encryption::logic::dek::{CreateDekInnerParams, CreateDekParams};
    use encryption::logic::dek_alias::{CreateAliasInnerParams, CreateAliasParams};
    use encryption::logic::envelope::get_or_create_local_envelope_encryption_key;
    use encryption::repository::{EncryptionKeyRepositoryLike, Repository as EncryptionRepository};
    use shared::primitives::{SqlMigrationLoader, WrappedChronoDateTime};
    use shared::test_utils::repository::setup_in_memory_database;
    use tokio::sync::broadcast;

    struct TestContext {
        identity_repo: Repository,
        crypto_cache: CryptoCache,
        jwks_cache: JwksCache,
        #[allow(dead_code)]
        temp_dir: tempfile::TempDir,
    }

    async fn setup_test_context() -> TestContext {
        shared::setup_test!();

        // Setup identity database
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let identity_repo = Repository::new(conn);

        // Setup encryption database
        let (_encryption_db, encryption_conn) =
            setup_in_memory_database(vec![EncryptionRepository::load_sql_migrations()])
                .await
                .unwrap();
        let encryption_repo = EncryptionRepository::new(encryption_conn);

        // Create temp dir for local keys
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
        let key_path = temp_dir.path().join("test-key");

        // Create envelope key
        let envelope_key_contents = get_or_create_local_envelope_encryption_key(&key_path).unwrap();
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
        .unwrap();

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
        .unwrap();

        // Create CryptoCache
        let crypto_cache = CryptoCache::new(encryption_repo.clone(), temp_dir.path().to_path_buf());
        init_crypto_cache(&crypto_cache).await.unwrap();

        // Create alias for the DEK
        encryption::logic::dek_alias::create_alias(
            &tx,
            &encryption_repo,
            &crypto_cache,
            CreateAliasParams {
                dek_id: dek.id.clone(),
                inner: CreateAliasInnerParams {
                    alias: "test-dek-alias".to_string(),
                },
            },
        )
        .await
        .unwrap();

        let jwks_cache = JwksCache::new(identity_repo.clone());

        TestContext {
            identity_repo,
            crypto_cache,
            jwks_cache,
            temp_dir,
        }
    }

    #[tokio::test]
    async fn test_create_jwk() {
        let ctx = setup_test_context().await;

        let request = CreateJwkRequest {
            dek_alias: "test-dek-alias".to_string(),
            expires_in_days: 30,
        };

        let result = create_jwk(
            &ctx.identity_repo,
            &ctx.crypto_cache,
            &ctx.jwks_cache,
            request,
        )
        .await
        .unwrap();

        assert!(!result.kid.is_empty());
        assert!(result.public_key.contains("BEGIN PUBLIC KEY"));
        assert!(result.public_key.contains("END PUBLIC KEY"));
    }

    #[tokio::test]
    async fn test_delete_jwk() {
        let ctx = setup_test_context().await;

        let request = CreateJwkRequest {
            dek_alias: "test-dek-alias".to_string(),
            expires_in_days: 30,
        };

        let created = create_jwk(
            &ctx.identity_repo,
            &ctx.crypto_cache,
            &ctx.jwks_cache,
            request,
        )
        .await
        .unwrap();

        // Invalidate the key
        let params = InvalidateJwkParams {
            kid: created.kid.clone(),
        };
        invalidate_jwk(&ctx.identity_repo, &ctx.jwks_cache, params)
            .await
            .unwrap();

        // Verify it's invalidated
        let result = ctx
            .identity_repo
            .get_jwt_signing_key_by_kid(&created.kid)
            .await
            .unwrap();
        assert!(result.is_some());
        assert!(result.unwrap().invalidated);
    }

    #[tokio::test]
    async fn test_list_jwks() {
        let ctx = setup_test_context().await;

        // Create multiple keys
        for _ in 0..3 {
            let request = CreateJwkRequest {
                dek_alias: "test-dek-alias".to_string(),
                expires_in_days: 30,
            };
            create_jwk(
                &ctx.identity_repo,
                &ctx.crypto_cache,
                &ctx.jwks_cache,
                request,
            )
            .await
            .unwrap();
        }

        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };

        let result = list_jwks(&ctx.identity_repo, &pagination).await.unwrap();
        assert_eq!(result.items.len(), 3);
    }

    #[tokio::test]
    async fn test_get_jwks() {
        let ctx = setup_test_context().await;

        // Create a key
        let request = CreateJwkRequest {
            dek_alias: "test-dek-alias".to_string(),
            expires_in_days: 30,
        };
        create_jwk(
            &ctx.identity_repo,
            &ctx.crypto_cache,
            &ctx.jwks_cache,
            request,
        )
        .await
        .unwrap();

        let jwks = get_jwks(&ctx.identity_repo, &ctx.jwks_cache).await.unwrap();
        assert_eq!(jwks.keys.len(), 1);
        assert_eq!(jwks.keys[0].kty, "RSA");
        assert_eq!(jwks.keys[0].use_, "sig");
        assert_eq!(jwks.keys[0].alg, "RS256");
        assert!(!jwks.keys[0].kid.is_empty());
        assert!(!jwks.keys[0].n.is_empty());
        assert!(!jwks.keys[0].e.is_empty());
    }

    #[test]
    fn test_pem_to_jwk() {
        // Test with a sample RSA public key PEM
        let pem = r#"-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA1234567890abcdefghijklmnopqrstuvwxyz
ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRST
UVWXYZ1234567890abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890abc
defghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890abcdefghijklmnopqrstuv
wxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNO
PQRSTUVWXYZ1234567890abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890
abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890abcdefghijklmnopqrstu
vwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQ
RSTUVWXYZ1234567890abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ab
cdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890abcdefghijklmnopqrstuvwx
yzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRST
UVWXYZ1234567890abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890abcdef
ghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890abcdefghijklmnopqrstuvwxyzA
QAB
-----END PUBLIC KEY-----"#;

        // This test will fail with invalid PEM, but tests the function structure
        // In real usage, valid PEM keys will be used
        let result = pem_to_jwk(pem, "test-kid");
        // We expect this to fail with invalid PEM, but the function should handle it gracefully
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_check_jwks_exists_on_start_no_keys() {
        let ctx = setup_test_context().await;

        // Initially no keys should exist
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let initial_keys = ctx
            .identity_repo
            .list_jwt_signing_keys(&pagination)
            .await
            .unwrap();
        assert_eq!(initial_keys.items.len(), 0);

        // Call check_jwks_exists_on_start
        check_jwks_exists_on_start(
            &ctx.identity_repo,
            &ctx.crypto_cache,
            &ctx.jwks_cache,
            "test-dek-alias",
        )
        .await
        .unwrap();

        // Now a key should exist
        let after_keys = ctx
            .identity_repo
            .list_jwt_signing_keys(&pagination)
            .await
            .unwrap();
        assert_eq!(after_keys.items.len(), 1);
    }

    #[tokio::test]
    async fn test_check_jwks_exists_on_start_with_keys() {
        let ctx = setup_test_context().await;

        // Create a key first
        let request = CreateJwkRequest {
            dek_alias: "test-dek-alias".to_string(),
            expires_in_days: 30,
        };
        create_jwk(
            &ctx.identity_repo,
            &ctx.crypto_cache,
            &ctx.jwks_cache,
            request,
        )
        .await
        .unwrap();

        // Count keys before
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let before_keys = ctx
            .identity_repo
            .list_jwt_signing_keys(&pagination)
            .await
            .unwrap();
        let before_count = before_keys.items.len();

        // Call check_jwks_exists_on_start - should not create another
        check_jwks_exists_on_start(
            &ctx.identity_repo,
            &ctx.crypto_cache,
            &ctx.jwks_cache,
            "test-dek-alias",
        )
        .await
        .unwrap();

        // Count should be the same
        let after_keys = ctx
            .identity_repo
            .list_jwt_signing_keys(&pagination)
            .await
            .unwrap();
        assert_eq!(after_keys.items.len(), before_count);
    }

    #[tokio::test]
    async fn test_process_jwk_rotation_no_valid_keys() {
        let ctx = setup_test_context().await;

        // Create a key that expires in 2 days (less than 5 days threshold)
        let request = CreateJwkRequest {
            dek_alias: "test-dek-alias".to_string(),
            expires_in_days: 2,
        };
        create_jwk(
            &ctx.identity_repo,
            &ctx.crypto_cache,
            &ctx.jwks_cache,
            request,
        )
        .await
        .unwrap();

        // Process rotation - should create a new key
        process_jwk_rotation(
            &ctx.identity_repo,
            &ctx.crypto_cache,
            &ctx.jwks_cache,
            "test-dek-alias",
        )
        .await
        .unwrap();

        // Should have 2 keys now
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let keys = ctx
            .identity_repo
            .list_jwt_signing_keys(&pagination)
            .await
            .unwrap();
        assert_eq!(keys.items.len(), 2);
    }

    #[tokio::test]
    async fn test_process_jwk_rotation_with_valid_keys() {
        let ctx = setup_test_context().await;

        // Create a key that expires in 10 days (more than 5 days threshold)
        let request = CreateJwkRequest {
            dek_alias: "test-dek-alias".to_string(),
            expires_in_days: 10,
        };
        create_jwk(
            &ctx.identity_repo,
            &ctx.crypto_cache,
            &ctx.jwks_cache,
            request,
        )
        .await
        .unwrap();

        // Count keys before
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let before_keys = ctx
            .identity_repo
            .list_jwt_signing_keys(&pagination)
            .await
            .unwrap();
        let before_count = before_keys.items.len();

        // Process rotation - should NOT create a new key
        process_jwk_rotation(
            &ctx.identity_repo,
            &ctx.crypto_cache,
            &ctx.jwks_cache,
            "test-dek-alias",
        )
        .await
        .unwrap();

        // Count should be the same
        let after_keys = ctx
            .identity_repo
            .list_jwt_signing_keys(&pagination)
            .await
            .unwrap();
        assert_eq!(after_keys.items.len(), before_count);
    }

    #[tokio::test]
    async fn test_process_jwk_rotation_no_keys() {
        let ctx = setup_test_context().await;

        // Process rotation with no keys - should create one
        process_jwk_rotation(
            &ctx.identity_repo,
            &ctx.crypto_cache,
            &ctx.jwks_cache,
            "test-dek-alias",
        )
        .await
        .unwrap();

        // Should have 1 key now
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let keys = ctx
            .identity_repo
            .list_jwt_signing_keys(&pagination)
            .await
            .unwrap();
        assert_eq!(keys.items.len(), 1);
    }
}
