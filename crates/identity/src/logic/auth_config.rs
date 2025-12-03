use std::collections::HashMap;

use jsonwebtoken::DecodingKey;
use serde::{Deserialize, Serialize};
use shared::error::CommonError;
use utoipa::ToSchema;

use crate::logic::auth_client::Role;



pub struct JwtTokenTemplateValidationConfig {
    pub issuer: Option<String>,
    pub valid_audiences: Option<Vec<String>>,
    pub required_scopes: Option<Vec<String>>,
    pub required_groups: Option<Vec<String>>,
}

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct JwtTokenMappingConfig {
    pub issuer_field: String,
    pub audience_field: String,
    pub scopes_field: Option<String>,
    pub sub_field: String,
    pub email_field: Option<String>,
    pub groups_field: Option<String>,
}

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct GroupToRoleMapping {
    pub group: String,
    pub role: Role,
}

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct ScopeToRoleMapping {
    pub scope: String,
    pub role: Role,
}

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct ScopeToGroupMapping {
    pub scope: String,
    pub group: String,
}

pub enum TokenLocation {
    Header(String),
    Cookie(String),
}


pub type StsConfigId = String;
pub struct JwtTokenTemplateConfig {
    pub id: StsConfigId,
    pub jwks_uri: String,
    pub token_location: TokenLocation,
    pub validation_template: JwtTokenTemplateValidationConfig,
    pub mapping_template: JwtTokenMappingConfig,
    pub group_to_role_mappings: Vec<GroupToRoleMapping>,
    pub scope_to_role_mappings: Vec<ScopeToRoleMapping>,
    pub scope_to_group_mappings: Vec<ScopeToGroupMapping>,
}

pub type EncryptedHashedValue = String;

pub struct ApiKeyConfig {
    pub encrypted_hashed_value: EncryptedHashedValue,
    pub dek_alias: String,
    pub role: Role,
    pub api_key_id: String,
    pub user_id: String,
}

pub struct AuthConfig {
    pub api_keys: HashMap<EncryptedHashedValue, ApiKeyConfig>,
    pub sts_token_config: StsTokenConfigMap,
}


pub enum StsTokenConfig {
    JwtTemplate(JwtTokenTemplateConfig),
    DevMode,
}

pub type StsTokenConfigMap = HashMap<StsConfigId, StsTokenConfig>;

/// Standardize a group name to lowercase kebab-case with no special characters.
/// - Converts to lowercase
/// - Replaces underscores with dashes
/// - Removes all characters except alphanumeric and dashes
/// - Collapses multiple consecutive dashes into one
/// - Trims leading and trailing dashes
///
/// The standardized name is used as the group ID.
pub fn standardize_group_name(name: &str) -> String {
    let mut result = String::with_capacity(name.len());

    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            result.push(c.to_ascii_lowercase());
        } else if c == '_' || c == '-' || c == ' ' {
            // Convert underscores and spaces to dashes
            result.push('-');
        }
        // Skip other special characters
    }

    // Collapse multiple consecutive dashes into one
    let mut collapsed = String::with_capacity(result.len());
    let mut last_was_dash = false;
    for c in result.chars() {
        if c == '-' {
            if !last_was_dash {
                collapsed.push(c);
            }
            last_was_dash = true;
        } else {
            collapsed.push(c);
            last_was_dash = false;
        }
    }

    // Trim leading and trailing dashes
    collapsed.trim_matches('-').to_string()
}

/// External JWKS cache for fetching public keys from external identity providers
#[derive(Clone)]
pub struct ExternalJwksCache {
    /// Maps JWKS URI -> (kid -> DecodingKey)
    keys: std::sync::Arc<dashmap::DashMap<String, HashMap<String, DecodingKey>>>,
}

impl ExternalJwksCache {
    pub fn new() -> Self {
        Self {
            keys: std::sync::Arc::new(dashmap::DashMap::new()),
        }
    }

    /// Fetch JWKS from the given URI and cache the keys
    pub async fn fetch_jwks(&self, jwks_uri: &str) -> Result<(), CommonError> {
        let response = reqwest::get(jwks_uri)
            .await
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to fetch JWKS: {e}")))?;

        let jwks: serde_json::Value = response
            .json()
            .await
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to parse JWKS: {e}")))?;

        let keys = jwks["keys"]
            .as_array()
            .ok_or_else(|| CommonError::Unknown(anyhow::anyhow!("JWKS missing 'keys' array")))?;

        let mut key_map = HashMap::new();
        for key in keys {
            let kid = key["kid"]
                .as_str()
                .ok_or_else(|| CommonError::Unknown(anyhow::anyhow!("JWK missing 'kid'")))?;

            let kty = key["kty"].as_str().unwrap_or("RSA");
            let decoding_key = match kty {
                "RSA" => {
                    let n = key["n"].as_str().ok_or_else(|| {
                        CommonError::Unknown(anyhow::anyhow!("RSA JWK missing 'n'"))
                    })?;
                    let e = key["e"].as_str().ok_or_else(|| {
                        CommonError::Unknown(anyhow::anyhow!("RSA JWK missing 'e'"))
                    })?;
                    DecodingKey::from_rsa_components(n, e).map_err(|e| {
                        CommonError::Unknown(anyhow::anyhow!("Failed to create RSA key: {e}"))
                    })?
                }
                _ => {
                    tracing::warn!("Unsupported key type: {}", kty);
                    continue;
                }
            };

            key_map.insert(kid.to_string(), decoding_key);
        }

        self.keys.insert(jwks_uri.to_string(), key_map);
        Ok(())
    }

    /// Get a decoding key by JWKS URI and key ID
    pub fn get_key(&self, jwks_uri: &str, kid: &str) -> Option<DecodingKey> {
        self.keys
            .get(jwks_uri)
            .and_then(|keys| keys.get(kid).cloned())
    }
}

impl Default for ExternalJwksCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standardize_group_name_lowercase() {
        assert_eq!(standardize_group_name("ADMIN"), "admin");
        assert_eq!(standardize_group_name("Admin"), "admin");
        assert_eq!(standardize_group_name("SUPER_ADMIN"), "super-admin");
    }

    #[test]
    fn test_standardize_group_name_underscores() {
        assert_eq!(standardize_group_name("my_group_name"), "my-group-name");
        assert_eq!(
            standardize_group_name("some__double__underscore"),
            "some-double-underscore"
        );
    }

    #[test]
    fn test_standardize_group_name_spaces() {
        assert_eq!(standardize_group_name("my group name"), "my-group-name");
        assert_eq!(
            standardize_group_name("multiple   spaces"),
            "multiple-spaces"
        );
    }

    #[test]
    fn test_standardize_group_name_special_characters() {
        assert_eq!(
            standardize_group_name("admin@company.com"),
            "admincompanycom"
        );
        assert_eq!(standardize_group_name("group#1!"), "group1");
        assert_eq!(standardize_group_name("test.group.name"), "testgroupname");
    }

    #[test]
    fn test_standardize_group_name_mixed() {
        assert_eq!(
            standardize_group_name("My_Special Group!"),
            "my-special-group"
        );
        assert_eq!(
            standardize_group_name("__leading_trailing__"),
            "leading-trailing"
        );
        assert_eq!(standardize_group_name("---dashes---"), "dashes");
    }

    #[test]
    fn test_standardize_group_name_already_valid() {
        assert_eq!(standardize_group_name("already-valid"), "already-valid");
        assert_eq!(standardize_group_name("simple"), "simple");
        assert_eq!(standardize_group_name("group123"), "group123");
    }

    #[test]
    fn test_standardize_group_name_edge_cases() {
        assert_eq!(standardize_group_name(""), "");
        assert_eq!(standardize_group_name("a"), "a");
        assert_eq!(standardize_group_name("-"), "");
        assert_eq!(standardize_group_name("_"), "");
    }

    #[test]
    fn test_role_as_str() {
        assert_eq!(Role::Admin.as_str(), "admin");
        assert_eq!(Role::Maintainer.as_str(), "maintainer");
        assert_eq!(Role::ReadOnlyMaintainer.as_str(), "read-only-maintainer");
        assert_eq!(Role::Agent.as_str(), "agent");
        assert_eq!(Role::User.as_str(), "user");
    }

    #[test]
    fn test_external_jwks_cache_new() {
        let cache = ExternalJwksCache::new();
        // Should be empty initially
        assert!(cache
            .get_key("https://example.com/.well-known/jwks.json", "test-kid")
            .is_none());
    }

    #[test]
    fn test_external_jwks_cache_default() {
        let cache = ExternalJwksCache::default();
        // Should be empty initially
        assert!(cache
            .get_key("https://example.com/.well-known/jwks.json", "test-kid")
            .is_none());
    }
}
