use chrono::Utc;
use encryption::logic::CryptoCache;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::Serialize;
use shared::error::CommonError;
use shared::primitives::{PaginationRequest, WrappedChronoDateTime};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::logic::jwk::cache::JwksCache;
use crate::logic::user::{Role, UserType};
use crate::repository::{UpdateUser, User, UserRepositoryLike};

pub mod idp_to_soma_sync;
pub use idp_to_soma_sync::*;

// JWT signing key types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct JwtSigningKey {
    pub kid: String,
    pub encrypted_private_key: String,
    pub expires_at: WrappedChronoDateTime,
    pub public_key: String,
    pub dek_alias: String,
    pub invalidated: bool,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

/// Get a valid (non-expired, non-invalidated) signing key from the repository
async fn get_valid_signing_key<R: UserRepositoryLike>(
    repository: &R,
) -> Result<JwtSigningKey, CommonError> {
    let now = Utc::now();
    let mut next_page_token: Option<String> = None;

    loop {
        let pagination = PaginationRequest {
            page_size: 100,
            next_page_token: next_page_token.clone(),
        };

        let keys = repository.list_jwt_signing_keys(&pagination).await?;

        // Find a valid signing key in this page (not expired, not invalidated)
        if let Some(valid_key) = keys
            .items
            .into_iter()
            .find(|key| !key.invalidated && key.expires_at.get_inner() > &now)
        {
            return Ok(valid_key);
        }

        // Check if there are more pages
        match keys.next_page_token {
            Some(token) => next_page_token = Some(token),
            None => break,
        }
    }

    Err(CommonError::Unknown(anyhow::anyhow!(
        "No valid JWT signing key available. Please create one first."
    )))
}

/// Sign a JWT with the given claims and signing key
async fn sign_jwt<T: serde::Serialize>(
    claims: &T,
    signing_key: &JwtSigningKey,
    crypto_cache: &CryptoCache,
) -> Result<String, CommonError> {
    use jsonwebtoken::{EncodingKey, Header};

    // Decrypt the private key
    let decryption_service = crypto_cache
        .get_decryption_service(&signing_key.dek_alias)
        .await?;

    let private_key_pem = decryption_service
        .decrypt_data(encryption::logic::EncryptedString(
            signing_key.encrypted_private_key.clone(),
        ))
        .await?;

    // Create encoding key from PEM
    let encoding_key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes())
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to parse private key: {e}")))?;

    // Create header with kid
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(signing_key.kid.clone());

    // Sign the token
    let token = jsonwebtoken::encode(&header, claims, &encoding_key)
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to sign JWT: {e}")))?;

    Ok(token)
}

pub const ISSUER: &str = "soma-identity";
pub const AUDIENCE: &str = "soma";

pub struct NormalizedTokenInputFields {
    pub subject: String,
    pub email: Option<String>,
    pub groups: Vec<String>,
    pub role: Role,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessTokenType {
    Access,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefreshTokenType {
    Refresh,
}

/// Claims structure for our issued access tokens
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AccessTokenClaims {
    /// Subject (user ID)
    pub sub: String,
    /// Issuer
    pub iss: String,
    /// Audience
    pub aud: String,
    /// Expiration time (Unix timestamp)
    pub exp: i64,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// Not before (Unix timestamp)
    pub nbf: i64,
    /// JWT ID
    pub jti: String,
    /// Token type
    pub token_type: AccessTokenType,
    /// User email (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// User role
    pub role: Role,
    /// User groups
    pub groups: Vec<String>,
}

/// Claims structure for our issued refresh tokens
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RefreshTokenClaims {
    /// Subject (user ID)
    pub sub: String,
    /// Issuer
    pub iss: String,
    /// Audience
    pub aud: String,
    /// Expiration time (Unix timestamp)
    pub exp: i64,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// Not before (Unix timestamp)
    pub nbf: i64,
    /// JWT ID
    pub jti: String,
    /// Token type - always "refresh"
    pub token_type: RefreshTokenType,
}

pub struct RefreshTokenParams {
    pub refresh_token: String,
}

pub struct SignRefreshTokenParams {
    pub sub: String,
}

pub struct RefreshTokenResult {
    pub access_token: String,
    pub expires_in: i64,
}

pub struct SignAccessTokenParams {
    pub sub: String,
    pub email: Option<String>,
    pub groups: Vec<String>,
    pub role: Role,
}

pub async fn sign_access_token<R: UserRepositoryLike>(
    params: SignAccessTokenParams,
    repository: &R,
    crypto_cache: &CryptoCache,
) -> Result<String, CommonError> {
    let signing_key = get_valid_signing_key(repository).await?;
    let access_token_expires_in: i64 = 3600; // 1 hour
    let now_ts = Utc::now().timestamp();
    let access_claims = AccessTokenClaims {
        sub: params.sub,
        iss: ISSUER.to_string(),
        aud: AUDIENCE.to_string(),
        exp: now_ts + access_token_expires_in,
        iat: now_ts,
        nbf: now_ts,
        jti: Uuid::new_v4().to_string(),
        token_type: AccessTokenType::Access,
        email: params.email,
        role: params.role,
        groups: params.groups,
    };

    let access_token = sign_jwt(&access_claims, &signing_key, crypto_cache).await?;
    Ok(access_token)
}

pub async fn sign_refresh_token<R: UserRepositoryLike>(
    params: SignRefreshTokenParams,
    repository: &R,
    crypto_cache: &CryptoCache,
) -> Result<String, CommonError> {
    let signing_key = get_valid_signing_key(repository).await?;
    let refresh_token_expires_in: i64 = 86400; // 24 hours
    let now_ts = Utc::now().timestamp();
    let refresh_claims = RefreshTokenClaims {
        sub: params.sub,
        iss: ISSUER.to_string(),
        aud: AUDIENCE.to_string(),
        exp: now_ts + refresh_token_expires_in,
        iat: now_ts,
        nbf: now_ts,
        jti: Uuid::new_v4().to_string(),
        token_type: RefreshTokenType::Refresh,
    };

    let refresh_token = sign_jwt(&refresh_claims, &signing_key, crypto_cache).await?;
    Ok(refresh_token)
}

#[derive(Debug, Serialize, ToSchema)]
pub struct NormalizedTokenIssuanceResult {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

/// Create or update a user and issue tokens based on normalized STS fields.
///
/// This is the core function that handles:
/// 1. Creating or updating the user in the database
/// 2. Syncing group memberships
/// 3. Signing and issuing access + refresh tokens
///
/// This function is reused by both STS token exchange and OAuth callback flows.
pub async fn issue_tokens_for_normalized_user<R: UserRepositoryLike>(
    repository: &R,
    crypto_cache: &CryptoCache,
    normalized: NormalizedTokenInputFields,
) -> Result<NormalizedTokenIssuanceResult, CommonError> {
    // 1. Create or update user
    // For federated users, use human_$subject format
    let user_id = format!("human_{}", normalized.subject);
    let now = WrappedChronoDateTime::now();

    let existing_user = repository.get_user_by_id(&user_id).await?;

    if existing_user.is_none() {
        let user = User {
            id: user_id.clone(),
            user_type: UserType::Human,
            email: normalized.email.clone(),
            role: normalized.role.clone(),
            description: None,
            created_at: now,
            updated_at: now,
        };
        repository.create_user(&user).await?;
    } else {
        // Update user if email or role changed
        let update_user = UpdateUser {
            email: normalized.email.clone(),
            role: Some(normalized.role.clone()),
            description: None,
        };
        repository.update_user(&user_id, &update_user).await?;
    }

    // 2. Sync group memberships
    sync_user_groups(repository, &user_id, &normalized.groups).await?;

    // 3. Sign tokens
    let access_token_expires_in: i64 = 3600; // 1 hour

    let access_token = sign_access_token(
        SignAccessTokenParams {
            sub: user_id.clone(),
            email: normalized.email,
            groups: normalized.groups,
            role: normalized.role,
        },
        repository,
        crypto_cache,
    )
    .await?;

    let refresh_token = sign_refresh_token(
        SignRefreshTokenParams { sub: user_id },
        repository,
        crypto_cache,
    )
    .await?;

    Ok(NormalizedTokenIssuanceResult {
        access_token,
        refresh_token,
        expires_in: access_token_expires_in,
    })
}

/// Refresh an access token using a valid refresh token.
///
/// This function:
/// 1. Validates the refresh token signature using our JWKS
/// 2. Verifies it's a refresh token (token_type = "refresh")
/// 3. Checks the token hasn't expired
/// 4. Looks up the user to get current role and groups
/// 5. Issues a new access token with fresh expiration
pub async fn refresh_access_token<R: UserRepositoryLike>(
    repository: &R,
    crypto_cache: &CryptoCache,
    jwks_cache: &JwksCache,
    params: RefreshTokenParams,
) -> Result<RefreshTokenResult, CommonError> {
    // 1. Decode the refresh token header to get the kid
    let header = decode_header(&params.refresh_token).map_err(|e| CommonError::Authentication {
        msg: format!("Failed to decode refresh token header: {e}"),
        source: None,
    })?;

    let kid = header.kid.ok_or_else(|| CommonError::Authentication {
        msg: "Refresh token missing 'kid' in header".to_string(),
        source: None,
    })?;

    // 2. Get our JWKS and find the matching key
    let jwks = jwks_cache.get_cached_jwks();
    let jwk = jwks
        .iter()
        .find(|k| k.kid == kid)
        .ok_or_else(|| CommonError::Authentication {
            msg: format!("Signing key '{kid}' not found in JWKS"),
            source: None,
        })?;

    // 3. Create decoding key from our public key
    let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to create decoding key: {e}")))?;

    // 4. Validate and decode the refresh token
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[ISSUER]);
    validation.set_audience(&[AUDIENCE]);

    let token_data =
        decode::<RefreshTokenClaims>(&params.refresh_token, &decoding_key, &validation).map_err(
            |e| CommonError::Authentication {
                msg: format!("Refresh token validation failed: {e}"),
                source: None,
            },
        )?;

    let claims = token_data.claims;

    // 5. Verify it's a refresh token
    if !matches!(claims.token_type, RefreshTokenType::Refresh) {
        return Err(CommonError::Authentication {
            msg: "Invalid token type: expected refresh token".to_string(),
            source: None,
        });
    }

    // 6. Look up the user to get current role and groups
    let user = repository
        .get_user_by_id(&claims.sub)
        .await?
        .ok_or_else(|| CommonError::NotFound {
            msg: "User not found".to_string(),
            lookup_id: claims.sub.clone(),
            source: None,
        })?;

    // 7. Get user's current groups
    let mut next_page_token = None;
    let mut all_memberships = Vec::new();
    loop {
        let pagination = PaginationRequest {
            page_size: 1000,
            next_page_token,
        };
        let result = repository
            .list_user_groups(&claims.sub, &pagination)
            .await?;
        all_memberships.extend(result.items);
        if result.next_page_token.is_none() {
            break;
        }
        next_page_token = result.next_page_token;
    }
    let groups: Vec<String> = all_memberships
        .iter()
        .map(|ug| ug.group.id.clone())
        .collect();

    // 8. Get a valid signing key
    let signing_key = get_valid_signing_key(repository).await?;

    // 9. Create and sign a new access token
    let access_token_expires_in: i64 = 3600; // 1 hour
    let now_ts = Utc::now().timestamp();

    let access_claims = AccessTokenClaims {
        sub: claims.sub,
        iss: ISSUER.to_string(),
        aud: AUDIENCE.to_string(),
        exp: now_ts + access_token_expires_in,
        iat: now_ts,
        nbf: now_ts,
        jti: Uuid::new_v4().to_string(),
        token_type: AccessTokenType::Access,
        email: user.email,
        role: user.role.clone(),
        groups,
    };

    let access_token = sign_jwt(&access_claims, &signing_key, crypto_cache).await?;

    Ok(RefreshTokenResult {
        access_token,
        expires_in: access_token_expires_in,
    })
}
