use encryption::logic::CryptoCache;
use http::HeaderMap;
use shared::error::CommonError;

use crate::logic::internal_token_issuance::{
    NormalizedTokenInputFields, NormalizedTokenIssuanceResult, issue_tokens_for_normalized_user,
};
use crate::logic::sts::config::{StsConfigId, StsTokenConfig};
use crate::logic::sts::external_jwk_cache::ExternalJwksCache;
use crate::logic::token_mapping::template::{
    DecodedTokenSources, JwtTokenTemplateConfig, JwtTokenTemplateValidationConfig, TokenLocation,
    apply_mapping_template,
};
use crate::logic::user::Role;
use crate::logic::{decode_jwt_to_claims, fetch_userinfo};
use crate::repository::UserRepositoryLike;

/// Apply dev mode configuration - returns a default dev user
fn apply_dev_mode_config() -> Result<NormalizedTokenInputFields, CommonError> {
    Ok(NormalizedTokenInputFields {
        subject: "dev-user".to_string(),
        email: None,
        groups: vec![],
        role: Role::Admin,
    })
}

pub struct ExchangeStsTokenParams {
    pub headers: HeaderMap,
    pub sts_token_config_id: StsConfigId,
}

/// Extract token from headers based on token location configuration
fn extract_token_from_headers(
    headers: &HeaderMap,
    location: &TokenLocation,
) -> Result<String, CommonError> {
    match location {
        TokenLocation::Header(header_name) => {
            let header_value = headers
                .get(header_name)
                .ok_or_else(|| CommonError::Authentication {
                    msg: format!("Missing '{header_name}' header"),
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

            for cookie in cookie_header.split(';') {
                let cookie = cookie.trim();
                if let Some((name, value)) = cookie.split_once('=') {
                    if name.trim() == cookie_name {
                        return Ok(value.trim().to_string());
                    }
                }
            }

            Err(CommonError::Authentication {
                msg: format!("Missing '{cookie_name}' cookie"),
                source: None,
            })
        }
    }
}

/// Exchange an external STS token for an internal access token.
///
/// This function:
/// 1. Looks up the STS config by ID from the repository
/// 2. Extracts and decodes tokens from headers (access token, ID token)
/// 3. Optionally fetches userinfo from the configured endpoint
/// 4. Applies the mapping template to extract normalized user fields
/// 5. Validates required groups and scopes
/// 6. Creates or updates the user and their group memberships
/// 7. Signs and returns a new internal JWT token with refresh token
pub async fn exchange_sts_token<R: UserRepositoryLike>(
    repository: &R,
    crypto_cache: &CryptoCache,
    external_jwks_cache: &ExternalJwksCache,
    params: ExchangeStsTokenParams,
) -> Result<NormalizedTokenIssuanceResult, CommonError> {
    // 1. Look up the STS config from the repository
    let config = repository
        .get_sts_configuration_by_id(&params.sts_token_config_id)
        .await?;

    let config = match config {
        Some(config) => config.config,
        None => {
            return Err(CommonError::NotFound {
                msg: "STS configuration not found".to_string(),
                lookup_id: params.sts_token_config_id.clone(),
                source: None,
            });
        }
    };

    // 2. Apply the appropriate config to get normalized fields
    let normalized = match &config {
        StsTokenConfig::JwtTemplate(jwt_config) => {
            apply_jwt_template_config(
                &jwt_config.mapping_template,
                &jwt_config.validation_template,
                external_jwks_cache,
                &params.headers,
            )
            .await?
        }
        StsTokenConfig::DevMode(_) => apply_dev_mode_config()?,
    };

    // 3. Issue tokens using the shared function
    issue_tokens_for_normalized_user(repository, crypto_cache, normalized).await
}

/// Apply JWT template configuration to extract and validate user info from external tokens.
///
/// This function:
/// 1. Extracts access token from headers (if configured) and decodes it
/// 2. Extracts ID token from headers (if configured) and decodes it
/// 3. Fetches userinfo from endpoint (if configured) using the access token
/// 4. Applies the mapping template to extract normalized fields
/// 5. Validates required groups and scopes
async fn apply_jwt_template_config(
    jwt_config: &JwtTokenTemplateConfig,
    validation_config: &JwtTokenTemplateValidationConfig,
    external_jwks_cache: &ExternalJwksCache,
    headers: &HeaderMap,
) -> Result<NormalizedTokenInputFields, CommonError> {
    // 1. Extract and decode access token if configured
    let (access_token_raw, access_token_claims) =
        if let Some(location) = &jwt_config.access_token_location {
            let token = extract_token_from_headers(headers, location)?;
            let claims = decode_jwt_to_claims(
                &token,
                &jwt_config.jwks_uri,
                external_jwks_cache,
                validation_config,
            )
            .await?;
            (Some(token), Some(claims))
        } else {
            (None, None)
        };

    // 2. Extract and decode ID token if configured
    let id_token_claims = if let Some(location) = &jwt_config.id_token_location {
        let token = extract_token_from_headers(headers, location)?;
        let claims = decode_jwt_to_claims(
            &token,
            &jwt_config.jwks_uri,
            external_jwks_cache,
            validation_config,
        )
        .await?;
        Some(claims)
    } else {
        None
    };

    // 3. Fetch userinfo if endpoint is configured and we have an access token
    let userinfo_claims = match (&jwt_config.userinfo_url, &access_token_raw) {
        (Some(userinfo_url), Some(access_token)) => {
            Some(fetch_userinfo(userinfo_url, access_token).await?)
        }
        _ => None,
    };

    // Ensure we have at least one source of claims
    if access_token_claims.is_none() && id_token_claims.is_none() && userinfo_claims.is_none() {
        return Err(CommonError::Authentication {
            msg: "No token sources available. Configure at least one of: access_token_location, id_token_location, or userinfo_url with access_token".to_string(),
            source: None,
        });
    }

    // 4. Build decoded token sources
    let mut sources = DecodedTokenSources::new();
    if let Some(claims) = access_token_claims {
        sources = sources.with_access_token(claims);
    }
    if let Some(claims) = id_token_claims {
        sources = sources.with_id_token(claims);
    }
    if let Some(claims) = userinfo_claims {
        sources = sources.with_userinfo(claims);
    }

    // 5. Apply the mapping template
    let mapping_result = apply_mapping_template(&sources, &jwt_config.mapping_template)?;

    // 6. Validate required groups (user must have ALL required groups)
    if let Some(required_groups) = &validation_config.required_groups {
        use crate::logic::token_mapping::template::standardize_group_name;

        let standardized_required: Vec<String> = required_groups
            .iter()
            .map(|g| standardize_group_name(g))
            .collect();

        let has_required = standardized_required
            .iter()
            .all(|required| mapping_result.groups.contains(required));

        if !has_required {
            return Err(CommonError::Authentication {
                msg: "User does not have required group membership".to_string(),
                source: None,
            });
        }
    }

    // 7. Validate required scopes
    if let Some(required_scopes) = &validation_config.required_scopes {
        let has_required = required_scopes
            .iter()
            .all(|required| mapping_result.scopes.contains(required));
        if !has_required {
            return Err(CommonError::Authentication {
                msg: "User does not have required scopes".to_string(),
                source: None,
            });
        }
    }

    Ok(NormalizedTokenInputFields {
        subject: mapping_result.subject,
        email: mapping_result.email,
        groups: mapping_result.groups,
        role: mapping_result.role,
    })
}

#[cfg(all(test, feature = "integration_test"))]
mod integration_test {
    use super::*;
    use crate::logic::sts::config::{DevModeConfig, JwtTemplateModeConfig, create_sts_config};
    use crate::logic::token_mapping::template::{JwtTokenMappingConfig, MappingSource};
    use crate::test::dex::{
        DEX_JWKS_ENDPOINT, DEX_MOCK_USER_EMAIL, DEX_USERINFO_ENDPOINT, perform_dex_mock_oidc_login,
    };
    use crate::test::fixtures::TestContext;
    use crate::test::token_validation::{
        AccessTokenAssertions, decode_and_validate_access_token, decode_and_validate_refresh_token,
    };
    use tokio::sync::broadcast;

    /// Create a JWT template config that maps from Dex ID token claims.
    fn create_dex_jwt_template_config() -> JwtTemplateModeConfig {
        use crate::test::dex::DEX_CLIENT_ID;

        JwtTemplateModeConfig {
            id: "dex-jwt-template".to_string(),
            mapping_template: JwtTokenTemplateConfig {
                jwks_uri: DEX_JWKS_ENDPOINT.to_string(),
                userinfo_url: Some(DEX_USERINFO_ENDPOINT.to_string()),
                introspect_url: None,
                // Get tokens from Authorization header (access) and X-Id-Token header (id token)
                access_token_location: Some(TokenLocation::Header("authorization".to_string())),
                id_token_location: Some(TokenLocation::Header("x-id-token".to_string())),
                mapping_template: JwtTokenMappingConfig {
                    issuer_field: MappingSource::IdToken("iss".to_string()),
                    audience_field: MappingSource::IdToken("aud".to_string()),
                    scopes_field: None,
                    sub_field: MappingSource::IdToken("sub".to_string()),
                    email_field: Some(MappingSource::IdToken("email".to_string())),
                    groups_field: None,
                    group_to_role_mappings: vec![],
                    scope_to_role_mappings: vec![],
                    scope_to_group_mappings: vec![],
                },
            },
            validation_template: JwtTokenTemplateValidationConfig {
                issuer: None, // Don't validate issuer for test flexibility
                // Dex tokens have the client_id as the audience
                valid_audiences: Some(vec![DEX_CLIENT_ID.to_string()]),
                required_scopes: None,
                required_groups: None,
            },
        }
    }

    #[tokio::test]
    async fn test_exchange_sts_token_dev_mode() {
        let ctx = TestContext::new_with_jwk().await;
        let (tx, _rx) = broadcast::channel(100);

        // Create a dev mode STS config
        let dev_config = StsTokenConfig::DevMode(DevModeConfig {
            id: "dev-mode-config".to_string(),
        });

        create_sts_config(&ctx.identity_repo, &tx, dev_config, false)
            .await
            .expect("Failed to create dev mode STS config");

        // Exchange with dev mode - no headers needed
        let params = ExchangeStsTokenParams {
            headers: HeaderMap::new(),
            sts_token_config_id: "dev-mode-config".to_string(),
        };

        let result = exchange_sts_token(
            &ctx.identity_repo,
            &ctx.crypto_cache,
            &ctx.external_jwks_cache,
            params,
        )
        .await
        .expect("Dev mode STS exchange should succeed");

        // Validate the issued access token
        let validated = decode_and_validate_access_token(&result.access_token, &ctx.jwks_cache)
            .expect("Access token should be valid");

        AccessTokenAssertions::new(&validated)
            .assert_standard_claims()
            .assert_subject("human_dev-user")
            .assert_role(Role::Admin);

        // Validate refresh token
        let refresh_validated =
            decode_and_validate_refresh_token(&result.refresh_token, &ctx.jwks_cache)
                .expect("Refresh token should be valid");

        assert_eq!(refresh_validated.claims.sub, "human_dev-user");

        // Verify expires_in is reasonable (1 hour = 3600 seconds)
        assert_eq!(result.expires_in, 3600);
    }

    #[tokio::test]
    async fn test_exchange_sts_token_dev_mode_creates_user() {
        let ctx = TestContext::new_with_jwk().await;
        let (tx, _rx) = broadcast::channel(100);

        // Create a dev mode STS config
        let dev_config = StsTokenConfig::DevMode(DevModeConfig {
            id: "dev-mode-user-test".to_string(),
        });

        create_sts_config(&ctx.identity_repo, &tx, dev_config, false)
            .await
            .expect("Failed to create dev mode STS config");

        // Verify user doesn't exist yet
        let user_before = ctx
            .identity_repo
            .get_user_by_id("human_dev-user")
            .await
            .expect("Query should succeed");
        assert!(
            user_before.is_none(),
            "User should not exist before exchange"
        );

        // Exchange with dev mode
        let params = ExchangeStsTokenParams {
            headers: HeaderMap::new(),
            sts_token_config_id: "dev-mode-user-test".to_string(),
        };

        exchange_sts_token(
            &ctx.identity_repo,
            &ctx.crypto_cache,
            &ctx.external_jwks_cache,
            params,
        )
        .await
        .expect("Dev mode STS exchange should succeed");

        // Verify user was created
        let user_after = ctx
            .identity_repo
            .get_user_by_id("human_dev-user")
            .await
            .expect("Query should succeed")
            .expect("User should exist after exchange");

        assert_eq!(user_after.id, "human_dev-user");
        assert_eq!(user_after.role, Role::Admin);
    }

    #[tokio::test]
    async fn test_exchange_sts_token_jwt_template_with_dex_tokens() {
        let ctx = TestContext::new_with_jwk().await;
        let (tx, _rx) = broadcast::channel(100);

        // Get real tokens from Dex
        let dex_tokens = perform_dex_mock_oidc_login()
            .await
            .expect("Failed to get tokens from Dex");

        assert!(
            !dex_tokens.access_token.is_empty(),
            "Should have access token"
        );
        assert!(!dex_tokens.id_token.is_empty(), "Should have ID token");

        // Create JWT template STS config
        let jwt_config = StsTokenConfig::JwtTemplate(create_dex_jwt_template_config());

        create_sts_config(&ctx.identity_repo, &tx, jwt_config, false)
            .await
            .expect("Failed to create JWT template STS config");

        // Build headers with Dex tokens
        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            format!("Bearer {}", dex_tokens.access_token)
                .parse()
                .unwrap(),
        );
        headers.insert("x-id-token", dex_tokens.id_token.parse().unwrap());

        let params = ExchangeStsTokenParams {
            headers,
            sts_token_config_id: "dex-jwt-template".to_string(),
        };

        let result = exchange_sts_token(
            &ctx.identity_repo,
            &ctx.crypto_cache,
            &ctx.external_jwks_cache,
            params,
        )
        .await
        .expect("JWT template STS exchange should succeed");

        // Validate the issued access token
        let validated = decode_and_validate_access_token(&result.access_token, &ctx.jwks_cache)
            .expect("Access token should be valid");

        AccessTokenAssertions::new(&validated)
            .assert_standard_claims()
            .assert_subject_starts_with("human_")
            .assert_email(Some(DEX_MOCK_USER_EMAIL))
            .assert_role(Role::User); // Default role when no mapping

        // Validate refresh token
        let refresh_validated =
            decode_and_validate_refresh_token(&result.refresh_token, &ctx.jwks_cache)
                .expect("Refresh token should be valid");

        // Subject should match between access and refresh tokens
        assert_eq!(refresh_validated.claims.sub, validated.claims.sub);

        // Verify expires_in is reasonable
        assert_eq!(result.expires_in, 3600);
    }

    #[tokio::test]
    async fn test_exchange_sts_token_jwt_template_creates_user_from_dex() {
        let ctx = TestContext::new_with_jwk().await;
        let (tx, _rx) = broadcast::channel(100);

        // Get real tokens from Dex
        let dex_tokens = perform_dex_mock_oidc_login()
            .await
            .expect("Failed to get tokens from Dex");

        // Create JWT template STS config
        let jwt_config = StsTokenConfig::JwtTemplate(create_dex_jwt_template_config());

        create_sts_config(&ctx.identity_repo, &tx, jwt_config, false)
            .await
            .expect("Failed to create JWT template STS config");

        // Build headers with Dex tokens
        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            format!("Bearer {}", dex_tokens.access_token)
                .parse()
                .unwrap(),
        );
        headers.insert("x-id-token", dex_tokens.id_token.parse().unwrap());

        let params = ExchangeStsTokenParams {
            headers,
            sts_token_config_id: "dex-jwt-template".to_string(),
        };

        let result = exchange_sts_token(
            &ctx.identity_repo,
            &ctx.crypto_cache,
            &ctx.external_jwks_cache,
            params,
        )
        .await
        .expect("JWT template STS exchange should succeed");

        // Extract user ID from the token
        let validated = decode_and_validate_access_token(&result.access_token, &ctx.jwks_cache)
            .expect("Access token should be valid");

        let user_id = &validated.claims.sub;

        // Verify user was created in the database
        let user = ctx
            .identity_repo
            .get_user_by_id(user_id)
            .await
            .expect("Query should succeed")
            .expect("User should exist after exchange");

        assert_eq!(user.id, *user_id);
        assert_eq!(user.email.as_deref(), Some(DEX_MOCK_USER_EMAIL));
    }

    #[tokio::test]
    async fn test_exchange_sts_token_missing_config() {
        let ctx = TestContext::new_with_jwk().await;

        let params = ExchangeStsTokenParams {
            headers: HeaderMap::new(),
            sts_token_config_id: "nonexistent-config".to_string(),
        };

        let result = exchange_sts_token(
            &ctx.identity_repo,
            &ctx.crypto_cache,
            &ctx.external_jwks_cache,
            params,
        )
        .await;

        assert!(result.is_err(), "Should fail with missing config");
        let err = result.unwrap_err();
        assert!(
            matches!(err, CommonError::NotFound { .. }),
            "Should be NotFound error"
        );
    }

    #[tokio::test]
    async fn test_exchange_sts_token_jwt_template_missing_headers() {
        let ctx = TestContext::new_with_jwk().await;
        let (tx, _rx) = broadcast::channel(100);

        // Create JWT template STS config that requires authorization header
        let jwt_config = StsTokenConfig::JwtTemplate(create_dex_jwt_template_config());

        create_sts_config(&ctx.identity_repo, &tx, jwt_config, false)
            .await
            .expect("Failed to create JWT template STS config");

        // Try exchange without providing the required headers
        let params = ExchangeStsTokenParams {
            headers: HeaderMap::new(), // Empty headers
            sts_token_config_id: "dex-jwt-template".to_string(),
        };

        let result = exchange_sts_token(
            &ctx.identity_repo,
            &ctx.crypto_cache,
            &ctx.external_jwks_cache,
            params,
        )
        .await;

        assert!(result.is_err(), "Should fail with missing headers");
        let err = result.unwrap_err();
        assert!(
            matches!(err, CommonError::Authentication { .. }),
            "Should be Authentication error, got: {err:?}"
        );
    }

    #[tokio::test]
    async fn test_exchange_sts_token_jwt_template_invalid_token() {
        let ctx = TestContext::new_with_jwk().await;
        let (tx, _rx) = broadcast::channel(100);

        // Create JWT template STS config
        let jwt_config = StsTokenConfig::JwtTemplate(create_dex_jwt_template_config());

        create_sts_config(&ctx.identity_repo, &tx, jwt_config, false)
            .await
            .expect("Failed to create JWT template STS config");

        // Provide invalid tokens
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer invalid.jwt.token".parse().unwrap());
        headers.insert("x-id-token", "invalid.id.token".parse().unwrap());

        let params = ExchangeStsTokenParams {
            headers,
            sts_token_config_id: "dex-jwt-template".to_string(),
        };

        let result = exchange_sts_token(
            &ctx.identity_repo,
            &ctx.crypto_cache,
            &ctx.external_jwks_cache,
            params,
        )
        .await;

        assert!(result.is_err(), "Should fail with invalid token");
    }

    #[tokio::test]
    async fn test_extract_token_from_header_bearer_format() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer my-token-value".parse().unwrap());

        let result = extract_token_from_headers(
            &headers,
            &TokenLocation::Header("authorization".to_string()),
        )
        .expect("Should extract token");

        assert_eq!(result, "my-token-value");
    }

    #[tokio::test]
    async fn test_extract_token_from_header_raw_format() {
        let mut headers = HeaderMap::new();
        headers.insert("x-custom-token", "raw-token-value".parse().unwrap());

        let result = extract_token_from_headers(
            &headers,
            &TokenLocation::Header("x-custom-token".to_string()),
        )
        .expect("Should extract token");

        assert_eq!(result, "raw-token-value");
    }

    #[tokio::test]
    async fn test_extract_token_from_cookie() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "cookie",
            "session=abc123; access_token=my-cookie-token; other=value"
                .parse()
                .unwrap(),
        );

        let result = extract_token_from_headers(
            &headers,
            &TokenLocation::Cookie("access_token".to_string()),
        )
        .expect("Should extract token from cookie");

        assert_eq!(result, "my-cookie-token");
    }

    #[tokio::test]
    async fn test_extract_token_missing_cookie() {
        let mut headers = HeaderMap::new();
        headers.insert("cookie", "session=abc123; other=value".parse().unwrap());

        let result = extract_token_from_headers(
            &headers,
            &TokenLocation::Cookie("missing_token".to_string()),
        );

        assert!(result.is_err(), "Should fail when cookie is missing");
    }
}
