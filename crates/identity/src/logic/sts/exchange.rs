use encryption::logic::CryptoCache;
use http::HeaderMap;
use jsonwebtoken::{Algorithm, Validation, decode, decode_header};
use serde_json::{Map, Value};
use shared::error::CommonError;

use crate::logic::{decode_jwt_to_claims, fetch_userinfo};
use crate::logic::user::Role;
use crate::logic::internal_token_issuance::{
    NormalizedTokenInputFields, NormalizedTokenIssuanceResult, issue_tokens_for_normalized_user
};
use crate::logic::sts::config::{StsConfigId, StsTokenConfig};
use crate::logic::sts::external_jwk_cache::ExternalJwksCache;
use crate::logic::token_mapping::template::{
    apply_mapping_template, DecodedTokenSources, JwtTokenTemplateConfig,
    JwtTokenTemplateValidationConfig, TokenLocation,
};
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
fn extract_token_from_headers(headers: &HeaderMap, location: &TokenLocation) -> Result<String, CommonError> {
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
    let config = repository.get_sts_configuration_by_id(&params.sts_token_config_id).await?;

    let config = match config {
        Some(config) => config.config,
        None => return Err(CommonError::NotFound {
            msg: "STS configuration not found".to_string(),
            lookup_id: params.sts_token_config_id.clone(),
            source: None,
        }),
    };

    // 2. Apply the appropriate config to get normalized fields
    let normalized = match &config {
        StsTokenConfig::JwtTemplate(jwt_config) => {
            apply_jwt_template_config(
                &jwt_config.mapping_template,
                &jwt_config.validation_template,
                external_jwks_cache,
                &params.headers,
            ).await?
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
    let (access_token_raw, access_token_claims) = if let Some(location) = &jwt_config.access_token_location {
        let token = extract_token_from_headers(headers, location)?;
        let claims = decode_jwt_to_claims(
            &token,
            &jwt_config.jwks_uri,
            external_jwks_cache,
            validation_config,
        ).await?;
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
        ).await?;
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
    let mapping_result = apply_mapping_template(
        &sources,
        &jwt_config.mapping_template,
    )?;

    // 6. Validate required groups
    if let Some(required_groups) = &validation_config.required_groups {
        use crate::logic::token_mapping::template::standardize_group_name;

        let standardized_required: Vec<String> = required_groups
            .iter()
            .map(|g| standardize_group_name(g))
            .collect();

        let has_required = standardized_required
            .iter()
            .any(|required| mapping_result.groups.contains(required));

        if !has_required {
            return Err(CommonError::Authentication {
                msg: "User does not have required group membership".to_string(),
                source: None,
            });
        }
    }

    // 7. Validate required scopes
    if let Some(required_scopes) = &validation_config.required_scopes {
        let has_required = required_scopes.iter().all(|required| mapping_result.scopes.contains(required));
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
