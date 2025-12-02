use std::collections::HashMap;

use encryption::logic::DecryptionService;
use http::HeaderMap;
use shared::error::CommonError;

use crate::repository::UserRepositoryLike;

pub enum Role {
    Admin,
    Agent,
    User,
}

pub struct JwtTokenTemplateValidationConfig {
    pub issuer: Option<String>,
    pub valid_audiences: Option<Vec<String>>,
    pub required_scopes: Option<Vec<String>>,
    pub required_groups: Option<Vec<String>>,
}

pub struct JwtTokenMappingConfig {
    pub issuer_field: String,
    pub audience_field: String,
    pub scopes_field: Option<String>,
    pub sub_field: String,
    pub email_field: Option<String>,
    pub groups_field: Option<String>,
}

pub struct JwtGroupToRoleMapping {
    pub group: String,
    pub role: Role,
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
    pub mapping_to_roles: Vec<JwtGroupToRoleMapping>,
}

pub type EncryptedHashedValue = String;

pub struct ApiKeyConfig {
    pub encrypted_hashed_value: EncryptedHashedValue,
    pub dek_alias: String,
    pub role: Role,
}

pub struct AuthMiddlewareConfig {
    pub api_keys: HashMap<EncryptedHashedValue, ApiKeyConfig>,
    pub sts_token_config: HashMap<StsConfigId, StsTokenConfig>,
}

pub enum StsTokenConfig {
    JwtTemplate(JwtTokenTemplateConfig),
}

pub struct ExchangeStsTokenResult {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64,
}

pub async fn exchange_sts_token(
    _repository: &impl UserRepositoryLike,
    _crypto_service: &DecryptionService,
    _config: &AuthMiddlewareConfig,
    _sts_token_config_id: StsConfigId,
    _headers: HeaderMap,
) -> Result<ExchangeStsTokenResult, CommonError> {
    // TODO: Implement STS token exchange logic
    // This function should:
    // 1. Extract the token from headers based on token_location
    // 2. Validate the token using the JWT template config
    // 3. Map the token claims to user information
    // 4. Create or get the user
    // 5. Generate a new STS token using a JWT signing key from the repository
    // 6. Return the exchange result

    Err(CommonError::Unknown(anyhow::anyhow!("Not implemented yet")))
}
