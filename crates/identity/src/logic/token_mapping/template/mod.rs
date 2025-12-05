pub mod apply;
pub use self::apply::*;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::logic::user::Role;

/// Indicates which token type (ID token for OIDC, access token response for OAuth)
/// contains the field
#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "field")]
pub enum MappingSource<T> {
    /// Field is in the OIDC ID token
    IdToken(T),
    /// Field is in the OAuth userinfo
    Userinfo(T),
    /// Field is in the OAuth access token response
    AccessToken(T),
}

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct JwtTokenMappingConfig {
    pub issuer_field: MappingSource<String>,
    pub audience_field: MappingSource<String>,
    pub scopes_field: Option<MappingSource<String>>,
    pub sub_field: MappingSource<String>,
    pub email_field: Option<MappingSource<String>>,
    pub groups_field: Option<MappingSource<String>>,

    pub group_to_role_mappings: Vec<GroupToRoleMapping>,
    pub scope_to_role_mappings: Vec<ScopeToRoleMapping>,
    pub scope_to_group_mappings: Vec<ScopeToGroupMapping>,
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

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub enum TokenLocation {
    Header(String),
    Cookie(String),
}

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct JwtTokenTemplateValidationConfig {
    pub issuer: Option<String>,
    pub valid_audiences: Option<Vec<String>>,
    pub required_scopes: Option<Vec<String>>,
    pub required_groups: Option<Vec<String>>,
}

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct JwtTokenTemplateConfig {
    pub jwks_uri: String,
    pub userinfo_url: Option<String>,
    pub introspect_url: Option<String>,
    pub access_token_location: Option<TokenLocation>,
    pub id_token_location: Option<TokenLocation>,
    pub mapping_template: JwtTokenMappingConfig,
}
