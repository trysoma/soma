use serde_json::{Map, Value};
use shared::error::CommonError;

use super::{
    GroupToRoleMapping, JwtTokenMappingConfig, MappingSource, ScopeToGroupMapping,
    ScopeToRoleMapping,
};
use crate::logic::user::Role;

/// Result of applying a mapping template to extract normalized user fields
pub struct NormalizedMappingResult {
    pub subject: String,
    pub email: Option<String>,
    pub groups: Vec<String>,
    pub scopes: Vec<String>,
    pub role: Role,
}

/// Decoded token sources that can be used for field extraction
pub struct DecodedTokenSources {
    /// Decoded access token claims (if present)
    pub access_token: Option<Map<String, Value>>,
    /// Decoded ID token claims (if present)
    pub id_token: Option<Map<String, Value>>,
    /// Userinfo response (if fetched)
    pub userinfo: Option<Map<String, Value>>,
}

impl DecodedTokenSources {
    pub fn new() -> Self {
        Self {
            access_token: None,
            id_token: None,
            userinfo: None,
        }
    }

    pub fn with_access_token(mut self, claims: Map<String, Value>) -> Self {
        self.access_token = Some(claims);
        self
    }

    pub fn with_id_token(mut self, claims: Map<String, Value>) -> Self {
        self.id_token = Some(claims);
        self
    }

    pub fn with_userinfo(mut self, claims: Map<String, Value>) -> Self {
        self.userinfo = Some(claims);
        self
    }

    pub fn has_any_source(&self) -> bool {
        self.access_token.is_some() || self.id_token.is_some() || self.userinfo.is_some()
    }
}

impl Default for DecodedTokenSources {
    fn default() -> Self {
        Self::new()
    }
}

/// Standardize a group name to lowercase kebab-case with no special characters.
/// - Converts to lowercase
/// - Replaces underscores with dashes
/// - Removes all characters except alphanumeric and dashes
/// - Collapses multiple consecutive dashes into one
/// - Trims leading and trailing dashes
/// - Returns None if the result is empty (e.g., input only contains special chars)
///
/// The standardized name is used as the group ID.
pub fn standardize_group_name(name: &str) -> String {
    let mut result = String::with_capacity(name.len());

    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            result.push(c.to_ascii_lowercase());
        } else if c == '_' || c == '-' || c == ' ' {
            result.push('-');
        }
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

    collapsed.trim_matches('-').to_string()
}

/// Extract a field value from the appropriate source based on MappingSource configuration
fn extract_field_from_sources<'a>(
    sources: &'a DecodedTokenSources,
    mapping_source: &MappingSource<String>,
) -> Option<&'a Value> {
    match mapping_source {
        MappingSource::IdToken(field) => sources
            .id_token
            .as_ref()
            .and_then(|claims| claims.get(field)),
        MappingSource::Userinfo(field) => sources
            .userinfo
            .as_ref()
            .and_then(|claims| claims.get(field)),
        MappingSource::AccessToken(field) => sources
            .access_token
            .as_ref()
            .and_then(|claims| claims.get(field)),
    }
}

/// Extract a required string field from the appropriate source
fn extract_string_field(
    sources: &DecodedTokenSources,
    mapping_source: &MappingSource<String>,
    field_name: &str,
) -> Result<String, CommonError> {
    let value = extract_field_from_sources(sources, mapping_source).ok_or_else(|| {
        CommonError::Unknown(anyhow::anyhow!(
            "Missing '{field_name}' field in token/userinfo/introspection"
        ))
    })?;

    value
        .as_str()
        .ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!("Field '{field_name}' is not a string"))
        })
        .map(|s| s.to_string())
}

/// Extract an optional string field from the appropriate source
fn extract_optional_string_field(
    sources: &DecodedTokenSources,
    mapping_source: &Option<MappingSource<String>>,
) -> Option<String> {
    mapping_source.as_ref().and_then(|ms| {
        extract_field_from_sources(sources, ms)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    })
}

/// Extract groups from the appropriate source (standardizes group names).
/// Filters out empty group names and removes duplicates while preserving order.
fn extract_groups(
    sources: &DecodedTokenSources,
    mapping_source: &Option<MappingSource<String>>,
) -> Vec<String> {
    let groups: Vec<String> = mapping_source
        .as_ref()
        .and_then(|ms| extract_field_from_sources(sources, ms))
        .map(|v| {
            if let Some(arr) = v.as_array() {
                arr.iter()
                    .filter_map(|g| g.as_str().map(standardize_group_name))
                    .filter(|g| !g.is_empty()) // Filter out empty group names
                    .collect()
            } else if let Some(s) = v.as_str() {
                let standardized = standardize_group_name(s);
                if standardized.is_empty() {
                    vec![]
                } else {
                    vec![standardized]
                }
            } else {
                vec![]
            }
        })
        .unwrap_or_default();

    // Deduplicate while preserving order
    let mut seen = std::collections::HashSet::new();
    groups
        .into_iter()
        .filter(|g| seen.insert(g.clone()))
        .collect()
}

/// Extract scopes from the appropriate source
fn extract_scopes(
    sources: &DecodedTokenSources,
    mapping_source: &Option<MappingSource<String>>,
) -> Vec<String> {
    mapping_source
        .as_ref()
        .and_then(|ms| extract_field_from_sources(sources, ms))
        .map(|v| {
            if let Some(arr) = v.as_array() {
                arr.iter()
                    .filter_map(|s| s.as_str().map(|s| s.to_string()))
                    .collect()
            } else if let Some(s) = v.as_str() {
                // Handle space-separated scopes (common in OAuth2)
                s.split_whitespace().map(|s| s.to_string()).collect()
            } else {
                vec![]
            }
        })
        .unwrap_or_default()
}

/// Determine user role from scope memberships using the configured mappings
/// Returns None if no matching scope is found
fn determine_role_from_scopes(scopes: &[String], mappings: &[ScopeToRoleMapping]) -> Option<Role> {
    for mapping in mappings {
        if scopes.contains(&mapping.scope) {
            return Some(mapping.role.clone());
        }
    }
    None
}

/// Determine user role from group memberships using the configured mappings
fn determine_role_from_groups(groups: &[String], mappings: &[GroupToRoleMapping]) -> Role {
    for mapping in mappings {
        let standardized_group = standardize_group_name(&mapping.group);
        if groups.contains(&standardized_group) {
            return mapping.role.clone();
        }
    }
    Role::User
}

/// Map scopes to additional groups using the configured mappings
fn map_scopes_to_groups(
    scopes: &[String],
    groups: &mut Vec<String>,
    mappings: &[ScopeToGroupMapping],
) {
    for mapping in mappings {
        if scopes.contains(&mapping.scope) {
            let group = standardize_group_name(&mapping.group);
            if !groups.contains(&group) {
                groups.push(group);
            }
        }
    }
}

/// Apply mapping template to extract normalized user fields from decoded token sources.
///
/// This function takes pre-decoded token sources (access token, ID token, userinfo)
/// and applies the mapping configuration to extract:
/// - subject (required)
/// - email (optional)
/// - groups (optional, standardized to kebab-case)
/// - scopes (optional)
/// - role (determined from scope-to-role or group-to-role mappings)
///
/// Note: This function does NOT perform any validation (issuer, audience, required groups/scopes).
/// Validation should be done separately before or after calling this function.
pub fn apply_mapping_template(
    sources: &DecodedTokenSources,
    mapping_config: &JwtTokenMappingConfig,
) -> Result<NormalizedMappingResult, CommonError> {
    // Extract user information from claims using the mapping template
    let subject = extract_string_field(sources, &mapping_config.sub_field, "subject")?;
    let email = extract_optional_string_field(sources, &mapping_config.email_field);
    let mut groups = extract_groups(sources, &mapping_config.groups_field);
    let scopes = extract_scopes(sources, &mapping_config.scopes_field);

    // Map scopes to additional groups
    map_scopes_to_groups(
        &scopes,
        &mut groups,
        &mapping_config.scope_to_group_mappings,
    );

    // Determine role - first check scope-to-role mappings, then group-to-role mappings
    let role = determine_role_from_scopes(&scopes, &mapping_config.scope_to_role_mappings)
        .unwrap_or_else(|| {
            determine_role_from_groups(&groups, &mapping_config.group_to_role_mappings)
        });

    Ok(NormalizedMappingResult {
        subject,
        email,
        groups,
        scopes,
        role,
    })
}
