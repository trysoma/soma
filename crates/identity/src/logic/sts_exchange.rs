use std::collections::HashMap;

use chrono::Utc;
use encryption::logic::CryptoCache;
use http::HeaderMap;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use shared::error::CommonError;
use shared::primitives::{PaginationRequest, WrappedChronoDateTime};
use uuid::Uuid;

use crate::logic::auth_client::Role;
use crate::logic::auth_config::{
    ExternalJwksCache, JwtGroupToRoleMapping, JwtTokenTemplateConfig, NormalizedStsFields,
    StsConfigId, StsTokenConfig, StsTokenConfigMap, TokenLocation, standardize_group_name,
};
use crate::logic::jwks_cache::JwksCache;
use crate::repository::{
    CreateGroup, CreateGroupMembership, CreateUser, UpdateUser, UserRepositoryLike,
};

pub const ISSUER: &str = "soma-identity";
pub const AUDIENCE: &str = "soma";

pub struct ExchangeStsTokenResult {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64,
}

pub struct ExchangeStsTokenParams {
    pub headers: HeaderMap,
    pub sts_token_config_id: StsConfigId,
}

pub struct RefreshTokenParams {
    pub refresh_token: String,
}

pub struct RefreshTokenResult {
    pub access_token: String,
    pub expires_in: i64,
}

/// Claims structure for parsing incoming JWT tokens
#[derive(Debug, serde::Deserialize)]
struct IncomingTokenClaims {
    #[serde(flatten)]
    claims: HashMap<String, serde_json::Value>,
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
    pub token_type: String,
    /// User email (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// User role
    pub role: String,
    /// User groups
    pub groups: Vec<String>,
}

/// Claims structure for our issued refresh tokens
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct RefreshTokenClaims {
    /// Subject (user ID)
    sub: String,
    /// Issuer
    iss: String,
    /// Audience
    aud: String,
    /// Expiration time (Unix timestamp)
    exp: i64,
    /// Issued at (Unix timestamp)
    iat: i64,
    /// Not before (Unix timestamp)
    nbf: i64,
    /// JWT ID
    jti: String,
    /// Token type - always "refresh"
    token_type: String,
}

/// Apply JWT template configuration to extract and validate user info from an external JWT
async fn apply_jwt_template_config(
    jwt_config: &JwtTokenTemplateConfig,
    external_jwks_cache: &ExternalJwksCache,
    headers: &HeaderMap,
) -> Result<NormalizedStsFields, CommonError> {
    // 1. Extract token from headers
    let token = extract_token(headers, &jwt_config.token_location)?;

    // 2. Decode the JWT header to get the kid
    let header = decode_header(&token)
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to decode JWT header: {e}")))?;

    let kid = header.kid.ok_or_else(|| {
        CommonError::Unknown(anyhow::anyhow!("JWT token missing 'kid' in header"))
    })?;

    // 3. Get or fetch the external JWKS
    if external_jwks_cache
        .get_key(&jwt_config.jwks_uri, &kid)
        .is_none()
    {
        external_jwks_cache.fetch_jwks(&jwt_config.jwks_uri).await?;
    }

    let decoding_key = external_jwks_cache
        .get_key(&jwt_config.jwks_uri, &kid)
        .ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!(
                "Key '{}' not found in JWKS from {}",
                kid,
                jwt_config.jwks_uri
            ))
        })?;

    // 4. Validate the token
    let mut validation = Validation::new(Algorithm::RS256);

    if let Some(ref issuer) = jwt_config.validation_template.issuer {
        validation.set_issuer(&[issuer]);
    }

    if let Some(ref audiences) = jwt_config.validation_template.valid_audiences {
        validation.set_audience(audiences);
    }

    let token_data = decode::<IncomingTokenClaims>(&token, &decoding_key, &validation)
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("JWT validation failed: {e}")))?;

    let claims = token_data.claims.claims;

    // 5. Extract user information from claims
    let subject = claims
        .get(&jwt_config.mapping_template.sub_field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!(
                "Missing '{}' claim in token",
                jwt_config.mapping_template.sub_field
            ))
        })?
        .to_string();

    let email = jwt_config
        .mapping_template
        .email_field
        .as_ref()
        .and_then(|field| claims.get(field))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Extract groups from claims
    let groups: Vec<String> = jwt_config
        .mapping_template
        .groups_field
        .as_ref()
        .and_then(|field| claims.get(field))
        .map(|v| {
            if let Some(arr) = v.as_array() {
                arr.iter()
                    .filter_map(|g| g.as_str().map(|s| standardize_group_name(s)))
                    .collect()
            } else if let Some(s) = v.as_str() {
                vec![standardize_group_name(s)]
            } else {
                vec![]
            }
        })
        .unwrap_or_default();

    // Validate required groups
    if let Some(ref required_groups) = jwt_config.validation_template.required_groups {
        let standardized_required: Vec<String> = required_groups
            .iter()
            .map(|g| standardize_group_name(g))
            .collect();

        let has_required = standardized_required
            .iter()
            .any(|required| groups.contains(required));

        if !has_required {
            return Err(CommonError::Authentication {
                msg: "User does not have required group membership".to_string(),
                source: None,
            });
        }
    }

    // 6. Determine user role from group mappings
    let role = determine_role_from_groups(&groups, &jwt_config.mapping_to_roles);

    Ok(NormalizedStsFields {
        subject,
        email,
        groups,
        role,
    })
}

/// Apply dev mode configuration - returns a default dev user
fn apply_dev_mode_config() -> Result<NormalizedStsFields, CommonError> {
    Ok(NormalizedStsFields {
        subject: "dev-user".to_string(),
        email: None,
        groups: vec![],
        role: Role::Admin,
    })
}

/// Exchange an external STS token for an internal access token.
///
/// This function:
/// 1. Looks up the STS config by ID
/// 2. Applies the appropriate config (JWT template or dev mode)
/// 3. Creates or updates the user and their group memberships
/// 4. Signs and returns a new internal JWT token with refresh token
pub async fn exchange_sts_token<R: UserRepositoryLike>(
    repository: &R,
    crypto_cache: &CryptoCache,
    _jwks_cache: &JwksCache,
    external_jwks_cache: &ExternalJwksCache,
    sts_token_config: &StsTokenConfigMap,
    params: ExchangeStsTokenParams,
) -> Result<ExchangeStsTokenResult, CommonError> {
    // 1. Look up the STS config
    let config = sts_token_config
        .get(&params.sts_token_config_id)
        .ok_or_else(|| CommonError::NotFound {
            msg: "STS config not found".to_string(),
            lookup_id: params.sts_token_config_id.clone(),
            source: None,
        })?;

    // 2. Apply the appropriate config to get normalized fields
    let normalized = match config {
        StsTokenConfig::JwtTemplate(jwt_config) => {
            apply_jwt_template_config(jwt_config, external_jwks_cache, &params.headers).await?
        }
        StsTokenConfig::DevMode => {
            apply_dev_mode_config( )?
        }
    };

    // 3. Create or update user
    let user_id = format!("sts:{}", normalized.subject);
    let now = WrappedChronoDateTime::now();

    let existing_user = repository.get_user_by_id(&user_id).await?;

    if existing_user.is_none() {
        let create_user = CreateUser {
            id: user_id.clone(),
            user_type: "federated_user".to_string(),
            email: normalized.email.clone(),
            role: normalized.role.as_str().to_string(),
            description: None,
            created_at: now,
            updated_at: now,
        };
        repository.create_user(&create_user).await?;
    } else {
        // Update user if email or role changed
        let update_user = UpdateUser {
            email: normalized.email.clone(),
            role: Some(normalized.role.as_str().to_string()),
            description: None,
        };
        repository.update_user(&user_id, &update_user).await?;
    }

    // 4. Sync group memberships
    sync_user_groups(repository, &user_id, &normalized.groups).await?;

    // 5. Get a valid signing key
    let signing_key = get_valid_signing_key(repository, crypto_cache).await?;

    // 6. Create and sign the access token
    let access_token_expires_in: i64 = 3600; // 1 hour
    let refresh_token_expires_in: i64 = 86400 * 7; // 7 days
    let now_ts = Utc::now().timestamp();

    let access_claims = AccessTokenClaims {
        sub: user_id.clone(),
        iss: ISSUER.to_string(),
        aud: AUDIENCE.to_string(),
        exp: now_ts + access_token_expires_in,
        iat: now_ts,
        nbf: now_ts,
        jti: Uuid::new_v4().to_string(),
        token_type: "access".to_string(),
        email: normalized.email,
        role: normalized.role.as_str().to_string(),
        groups: normalized.groups,
    };

    let access_token = sign_jwt(&access_claims, &signing_key, crypto_cache).await?;

    // 7. Create and sign the refresh token
    let refresh_claims = RefreshTokenClaims {
        sub: user_id,
        iss: ISSUER.to_string(),
        aud: AUDIENCE.to_string(),
        exp: now_ts + refresh_token_expires_in,
        iat: now_ts,
        nbf: now_ts,
        jti: Uuid::new_v4().to_string(),
        token_type: "refresh".to_string(),
    };

    let refresh_token = sign_jwt(&refresh_claims, &signing_key, crypto_cache).await?;

    Ok(ExchangeStsTokenResult {
        access_token,
        refresh_token: Some(refresh_token),
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
    let jwk = jwks.iter().find(|k| k.kid == kid).ok_or_else(|| {
        CommonError::Authentication {
            msg: format!("Signing key '{}' not found in JWKS", kid),
            source: None,
        }
    })?;

    // 3. Create decoding key from our public key
    let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e).map_err(|e| {
        CommonError::Unknown(anyhow::anyhow!("Failed to create decoding key: {e}"))
    })?;

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
    if claims.token_type != "refresh" {
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
    let pagination = PaginationRequest {
        page_size: 1000,
        next_page_token: None,
    };
    let user_groups = repository
        .list_user_groups(&claims.sub, &pagination)
        .await?;
    let groups: Vec<String> = user_groups
        .items
        .iter()
        .map(|ug| ug.group.id.clone())
        .collect();

    // 8. Get a valid signing key
    let signing_key = get_valid_signing_key(repository, crypto_cache).await?;

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
        token_type: "access".to_string(),
        email: user.email,
        role: user.role,
        groups,
    };

    let access_token = sign_jwt(&access_claims, &signing_key, crypto_cache).await?;

    Ok(RefreshTokenResult {
        access_token,
        expires_in: access_token_expires_in,
    })
}

/// Extract token from headers based on token location configuration
fn extract_token(headers: &HeaderMap, location: &TokenLocation) -> Result<String, CommonError> {
    match location {
        TokenLocation::Header(header_name) => {
            let header_value = headers
                .get(header_name)
                .ok_or_else(|| CommonError::Authentication {
                    msg: format!("Missing '{}' header", header_name),
                    source: None,
                })?
                .to_str()
                .map_err(|_| CommonError::Authentication {
                    msg: "Invalid header value".to_string(),
                    source: None,
                })?;

            // Handle "Bearer <token>" format
            if header_value.to_lowercase().starts_with("bearer ") {
                Ok(header_value[7..].to_string())
            } else {
                Ok(header_value.to_string())
            }
        }
        TokenLocation::Cookie(cookie_name) => {
            let cookie_header = headers
                .get("cookie")
                .ok_or_else(|| CommonError::Authentication {
                    msg: "Missing cookie header".to_string(),
                    source: None,
                })?
                .to_str()
                .map_err(|_| CommonError::Authentication {
                    msg: "Invalid cookie header".to_string(),
                    source: None,
                })?;

            // Parse cookies and find the one we need
            for cookie in cookie_header.split(';') {
                let cookie = cookie.trim();
                if let Some((name, value)) = cookie.split_once('=') {
                    if name.trim() == cookie_name {
                        return Ok(value.trim().to_string());
                    }
                }
            }

            Err(CommonError::Authentication {
                msg: format!("Missing '{}' cookie", cookie_name),
                source: None,
            })
        }
    }
}

/// Determine user role from group memberships using the configured mappings
fn determine_role_from_groups(groups: &[String], mappings: &[JwtGroupToRoleMapping]) -> Role {
    // Check mappings in order - first match wins
    for mapping in mappings {
        let standardized_group = standardize_group_name(&mapping.group);
        if groups.contains(&standardized_group) {
            return mapping.role.clone();
        }
    }

    // Default to User role
    Role::User
}

/// Sync user's group memberships - add new groups, remove old ones
async fn sync_user_groups<R: UserRepositoryLike>(
    repository: &R,
    user_id: &str,
    groups: &[String],
) -> Result<(), CommonError> {
    let now = WrappedChronoDateTime::now();

    // Get current group memberships
    let pagination = PaginationRequest {
        page_size: 1000,
        next_page_token: None,
    };
    let current_memberships = repository.list_user_groups(user_id, &pagination).await?;
    let current_group_ids: std::collections::HashSet<String> = current_memberships
        .items
        .iter()
        .map(|m| m.group.id.clone())
        .collect();

    let desired_group_ids: std::collections::HashSet<String> = groups.iter().cloned().collect();

    // Add memberships to new groups
    for group_id in desired_group_ids.difference(&current_group_ids) {
        // Ensure group exists (using standardized name as both ID and name)
        if repository.get_group_by_id(group_id).await?.is_none() {
            let create_group = CreateGroup {
                id: group_id.clone(),
                name: group_id.clone(), // Use standardized name
                created_at: now,
                updated_at: now,
            };
            repository.create_group(&create_group).await?;
        }

        // Create membership
        let create_membership = CreateGroupMembership {
            group_id: group_id.clone(),
            user_id: user_id.to_string(),
            created_at: now,
            updated_at: now,
        };
        repository.create_group_membership(&create_membership).await?;
    }

    // Remove memberships from groups no longer in the token
    for group_id in current_group_ids.difference(&desired_group_ids) {
        repository
            .delete_group_membership(group_id, user_id)
            .await?;
    }

    Ok(())
}

/// Get a valid (non-expired, non-invalidated) signing key from the repository
async fn get_valid_signing_key<R: UserRepositoryLike>(
    repository: &R,
    _crypto_cache: &CryptoCache,
) -> Result<crate::repository::JwtSigningKey, CommonError> {
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
    signing_key: &crate::repository::JwtSigningKey,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_token_from_header_bearer() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer my-test-token".parse().unwrap());

        let location = TokenLocation::Header("authorization".to_string());
        let result = extract_token(&headers, &location).unwrap();
        assert_eq!(result, "my-test-token");
    }

    #[test]
    fn test_extract_token_from_header_bearer_case_insensitive() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "BEARER my-test-token".parse().unwrap());

        let location = TokenLocation::Header("authorization".to_string());
        let result = extract_token(&headers, &location).unwrap();
        assert_eq!(result, "my-test-token");
    }

    #[test]
    fn test_extract_token_from_header_no_bearer() {
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", "raw-token-value".parse().unwrap());

        let location = TokenLocation::Header("x-api-key".to_string());
        let result = extract_token(&headers, &location).unwrap();
        assert_eq!(result, "raw-token-value");
    }

    #[test]
    fn test_extract_token_from_header_missing() {
        let headers = HeaderMap::new();

        let location = TokenLocation::Header("authorization".to_string());
        let result = extract_token(&headers, &location);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_token_from_cookie() {
        let mut headers = HeaderMap::new();
        headers.insert("cookie", "session=abc123; other=value".parse().unwrap());

        let location = TokenLocation::Cookie("session".to_string());
        let result = extract_token(&headers, &location).unwrap();
        assert_eq!(result, "abc123");
    }

    #[test]
    fn test_extract_token_from_cookie_multiple() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "cookie",
            "first=1; target=my-token; last=3".parse().unwrap(),
        );

        let location = TokenLocation::Cookie("target".to_string());
        let result = extract_token(&headers, &location).unwrap();
        assert_eq!(result, "my-token");
    }

    #[test]
    fn test_extract_token_from_cookie_missing() {
        let mut headers = HeaderMap::new();
        headers.insert("cookie", "other=value".parse().unwrap());

        let location = TokenLocation::Cookie("session".to_string());
        let result = extract_token(&headers, &location);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_token_from_cookie_no_cookie_header() {
        let headers = HeaderMap::new();

        let location = TokenLocation::Cookie("session".to_string());
        let result = extract_token(&headers, &location);
        assert!(result.is_err());
    }

    #[test]
    fn test_determine_role_from_groups_admin() {
        let groups = vec!["admin".to_string(), "users".to_string()];
        let mappings = vec![
            JwtGroupToRoleMapping {
                group: "admin".to_string(),
                role: Role::Admin,
            },
            JwtGroupToRoleMapping {
                group: "users".to_string(),
                role: Role::User,
            },
        ];

        let role = determine_role_from_groups(&groups, &mappings);
        assert_eq!(role.as_str(), "admin");
    }

    #[test]
    fn test_determine_role_from_groups_first_match_wins() {
        let groups = vec!["maintainer".to_string(), "admin".to_string()];
        let mappings = vec![
            JwtGroupToRoleMapping {
                group: "maintainer".to_string(),
                role: Role::Maintainer,
            },
            JwtGroupToRoleMapping {
                group: "admin".to_string(),
                role: Role::Admin,
            },
        ];

        // Mappings are checked in order, so maintainer should win
        let role = determine_role_from_groups(&groups, &mappings);
        assert_eq!(role.as_str(), "maintainer");
    }

    #[test]
    fn test_determine_role_from_groups_standardizes_mapping_group() {
        let groups = vec!["super-admin".to_string()];
        let mappings = vec![JwtGroupToRoleMapping {
            group: "SUPER_ADMIN".to_string(), // Different format
            role: Role::Admin,
        }];

        let role = determine_role_from_groups(&groups, &mappings);
        assert_eq!(role.as_str(), "admin");
    }

    #[test]
    fn test_determine_role_from_groups_no_match_defaults_to_user() {
        let groups = vec!["random-group".to_string()];
        let mappings = vec![JwtGroupToRoleMapping {
            group: "admin".to_string(),
            role: Role::Admin,
        }];

        let role = determine_role_from_groups(&groups, &mappings);
        assert_eq!(role.as_str(), "user");
    }

    #[test]
    fn test_determine_role_from_groups_empty_groups() {
        let groups: Vec<String> = vec![];
        let mappings = vec![JwtGroupToRoleMapping {
            group: "admin".to_string(),
            role: Role::Admin,
        }];

        let role = determine_role_from_groups(&groups, &mappings);
        assert_eq!(role.as_str(), "user");
    }

    #[test]
    fn test_determine_role_from_groups_empty_mappings() {
        let groups = vec!["admin".to_string()];
        let mappings: Vec<JwtGroupToRoleMapping> = vec![];

        let role = determine_role_from_groups(&groups, &mappings);
        assert_eq!(role.as_str(), "user");
    }
}
