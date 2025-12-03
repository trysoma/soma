//! IdP Configuration management for OAuth/OIDC external identity providers.
//!
//! This module contains:
//! - Business logic types for working with IdP configurations in memory
//! - Stored types for serialization to/from the database (with encrypted secrets)
//! - CRUD operations for IdP configurations

use encryption::logic::crypto_services::EncryptedString;
use encryption::logic::CryptoCache;
use serde::{Deserialize, Serialize};
use shared::error::CommonError;
use shared::primitives::{PaginationRequest, WrappedChronoDateTime};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::logic::auth_client::Role;
use crate::logic::auth_config::{
    standardize_group_name, GroupToRoleMapping, ScopeToGroupMapping, ScopeToRoleMapping,
    JwtTokenMappingConfig,
};
use crate::logic::{IdpConfigCreatedInfo, OnConfigChangeEvt, OnConfigChangeTx, DEFAULT_DEK_ALIAS};
use crate::repository::{CreateIdpConfiguration, IdpConfiguration, UserRepositoryLike};

// ============================================
// Business Logic Types (used in application code)
// ============================================

/// Indicates which token type (ID token for OIDC, access token response for OAuth)
/// contains the field
#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "field")]
pub enum TokenSource<T> {
    /// Field is in the OIDC ID token
    IdToken(T),
    /// Field is in the OAuth userinfo
    Userinfo(T),
    /// Field is in the OAuth access token response 
    AccessToken(T),
}

/// OAuth2 configuration for authorization code flow
#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct OauthConfig {
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: Option<String>,
    pub client_id: String,
    pub client_secret: String,
    pub scopes: Vec<String>,

    pub mapping_template: JwtTokenMappingConfig,
    pub group_to_role_mappings: Vec<GroupToRoleMapping>,
    pub scope_to_role_mappings: Vec<ScopeToRoleMapping>,
    pub scope_to_group_mappings: Vec<ScopeToGroupMapping>,
}


/// OIDC-specific mapping configuration indicating where claims come from
#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct OidcMappingConfig {
    pub sub_field: TokenSource<String>,
    pub email_field: Option<TokenSource<String>>,
    pub groups_field: Option<TokenSource<String>>,
}

/// OIDC configuration extending OAuth2 with discovery and ID token support
#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct OidcConfig {
    pub base_config: OauthConfig,
    pub discovery_endpoint: Option<String>,
    pub jwks_uri: Option<String>,

    pub mapping_config: OidcMappingConfig,
}

/// The four supported IdP configuration types
#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum IdpConfig {
    OidcAuthorizationCodeFlow(OidcConfig),
    OauthAuthorizationCodeFlow(OauthConfig),
    OidcAuthorizationCodePkceFlow(OidcConfig),
    OauthAuthorizationCodePkceFlow(OauthConfig),
}


/// Encrypt an IdpConfig and convert to StoredIdpConfig
pub async fn encrypt_idp_config(
    config: &IdpConfig,
    crypto_cache: &CryptoCache,
) -> Result<StoredIdpConfig, CommonError> {
    let encryption_service = crypto_cache.get_encryption_service(DEFAULT_DEK_ALIAS).await?;

    match config {
        IdpConfig::OauthAuthorizationCodeFlow(oauth) => {
            let encrypted = encryption_service
                .encrypt_data(oauth.client_secret.clone())
                .await?;
            Ok(StoredIdpConfig::OauthAuthorizationCode(to_stored_oauth_config(oauth, encrypted.0)))
        }
        IdpConfig::OauthAuthorizationCodePkceFlow(oauth) => {
            let encrypted = encryption_service
                .encrypt_data(oauth.client_secret.clone())
                .await?;
            Ok(StoredIdpConfig::OauthAuthorizationCodePkce(to_stored_oauth_config(oauth, encrypted.0)))
        }
        IdpConfig::OidcAuthorizationCodeFlow(oidc) => {
            let encrypted = encryption_service
                .encrypt_data(oidc.base_config.client_secret.clone())
                .await?;
            Ok(StoredIdpConfig::OidcAuthorizationCode(to_stored_oidc_config(oidc, encrypted.0)))
        }
        IdpConfig::OidcAuthorizationCodePkceFlow(oidc) => {
            let encrypted = encryption_service
                .encrypt_data(oidc.base_config.client_secret.clone())
                .await?;
            Ok(StoredIdpConfig::OidcAuthorizationCodePkce(to_stored_oidc_config(oidc, encrypted.0)))
        }
    }
}

/// Decrypt a StoredIdpConfig and convert to IdpConfig
pub async fn decrypt_idp_config(
    stored: &StoredIdpConfig,
    crypto_cache: &CryptoCache,
) -> Result<IdpConfig, CommonError> {
    match stored {
        StoredIdpConfig::OauthAuthorizationCode(oauth) => {
            let decryption_service = crypto_cache
                .get_decryption_service(&oauth.dek_alias)
                .await?;
            let client_secret = decryption_service
                .decrypt_data(EncryptedString(oauth.encrypted_client_secret.clone()))
                .await?;
            Ok(IdpConfig::OauthAuthorizationCodeFlow(from_stored_oauth_config(oauth, client_secret)))
        }
        StoredIdpConfig::OauthAuthorizationCodePkce(oauth) => {
            let decryption_service = crypto_cache
                .get_decryption_service(&oauth.dek_alias)
                .await?;
            let client_secret = decryption_service
                .decrypt_data(EncryptedString(oauth.encrypted_client_secret.clone()))
                .await?;
            Ok(IdpConfig::OauthAuthorizationCodePkceFlow(from_stored_oauth_config(oauth, client_secret)))
        }
        StoredIdpConfig::OidcAuthorizationCode(oidc) => {
            let decryption_service = crypto_cache
                .get_decryption_service(&oidc.base_config.dek_alias)
                .await?;
            let client_secret = decryption_service
                .decrypt_data(EncryptedString(oidc.base_config.encrypted_client_secret.clone()))
                .await?;
            Ok(IdpConfig::OidcAuthorizationCodeFlow(from_stored_oidc_config(oidc, client_secret)))
        }
        StoredIdpConfig::OidcAuthorizationCodePkce(oidc) => {
            let decryption_service = crypto_cache
                .get_decryption_service(&oidc.base_config.dek_alias)
                .await?;
            let client_secret = decryption_service
                .decrypt_data(EncryptedString(oidc.base_config.encrypted_client_secret.clone()))
                .await?;
            Ok(IdpConfig::OidcAuthorizationCodePkceFlow(from_stored_oidc_config(oidc, client_secret)))
        }
    }
}

fn to_stored_oauth_config(oauth: &OauthConfig, encrypted_client_secret: String) -> StoredOauthConfig {
    StoredOauthConfig {
        authorization_endpoint: oauth.authorization_endpoint.clone(),
        token_endpoint: oauth.token_endpoint.clone(),
        userinfo_endpoint: oauth.userinfo_endpoint.clone(),
        client_id: oauth.client_id.clone(),
        encrypted_client_secret,
        dek_alias: DEFAULT_DEK_ALIAS.to_string(),
        scopes: oauth.scopes.clone(),
        redirect_uri: oauth.redirect_uri.clone(),
        post_login_redirect_uri: oauth.post_login_redirect_uri.clone(),
        mapping_config: to_stored_mapping_config(&oauth.mapping_template),
        group_to_role_mappings: oauth.group_to_role_mappings.iter().map(to_stored_group_mapping).collect(),
        scope_to_role_mappings: oauth.scope_to_role_mappings.iter().map(to_stored_scope_role_mapping).collect(),
        scope_to_group_mappings: oauth.scope_to_group_mappings.iter().map(to_stored_scope_group_mapping).collect(),
        default_role: oauth.default_role.as_str().to_string(),
        allowed_domains: oauth.allowed_domains.clone(),
        state_ttl_seconds: oauth.state_ttl_seconds,
    }
}

fn from_stored_oauth_config(stored: &StoredOauthConfig, client_secret: String) -> OauthConfig {
    OauthConfig {
        authorization_endpoint: stored.authorization_endpoint.clone(),
        token_endpoint: stored.token_endpoint.clone(),
        userinfo_endpoint: stored.userinfo_endpoint.clone(),
        client_id: stored.client_id.clone(),
        client_secret,
        scopes: stored.scopes.clone(),
        redirect_uri: stored.redirect_uri.clone(),
        post_login_redirect_uri: stored.post_login_redirect_uri.clone(),
        mapping_template: from_stored_mapping_config(&stored.mapping_config),
        group_to_role_mappings: stored.group_to_role_mappings.iter().map(from_stored_group_mapping).collect(),
        scope_to_role_mappings: stored.scope_to_role_mappings.iter().map(from_stored_scope_role_mapping).collect(),
        scope_to_group_mappings: stored.scope_to_group_mappings.iter().map(from_stored_scope_group_mapping).collect(),
        default_role: Role::from_str(&stored.default_role).unwrap_or(Role::User),
        allowed_domains: stored.allowed_domains.clone(),
        state_ttl_seconds: stored.state_ttl_seconds,
    }
}

fn to_stored_oidc_config(oidc: &OidcConfig, encrypted_client_secret: String) -> StoredOidcConfig {
    StoredOidcConfig {
        base_config: to_stored_oauth_config(&oidc.base_config, encrypted_client_secret),
        discovery_endpoint: oidc.discovery_endpoint.clone(),
        jwks_uri: oidc.jwks_uri.clone(),
        mapping_config: to_stored_oidc_mapping_config(&oidc.mapping_config),
    }
}

fn from_stored_oidc_config(stored: &StoredOidcConfig, client_secret: String) -> OidcConfig {
    OidcConfig {
        base_config: from_stored_oauth_config(&stored.base_config, client_secret),
        discovery_endpoint: stored.discovery_endpoint.clone(),
        jwks_uri: stored.jwks_uri.clone(),
        mapping_config: from_stored_oidc_mapping_config(&stored.mapping_config),
    }
}

// ============================================
// API Input Types (with plaintext client_secret)
// ============================================

/// Input OAuth config (with plaintext client_secret for API input)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InputOauthConfig {
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: Option<String>,
    pub client_id: String,
    /// Client secret (will be encrypted before storage)
    pub client_secret: String,
    pub scopes: Vec<String>,
    pub redirect_uri: String,
    #[serde(default = "default_post_login_redirect")]
    pub post_login_redirect_uri: String,

    #[serde(default)]
    pub mapping_config: StoredMappingConfig,
    #[serde(default)]
    pub group_to_role_mappings: Vec<StoredGroupToRoleMapping>,
    #[serde(default)]
    pub scope_to_role_mappings: Vec<StoredScopeToRoleMapping>,
    #[serde(default)]
    pub scope_to_group_mappings: Vec<StoredScopeToGroupMapping>,
    #[serde(default = "default_role")]
    pub default_role: String,
    #[serde(default)]
    pub allowed_domains: Vec<String>,
    #[serde(default = "default_state_ttl")]
    pub state_ttl_seconds: u64,
}

/// Input OIDC config
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InputOidcConfig {
    pub base_config: InputOauthConfig,
    pub discovery_endpoint: Option<String>,
    pub jwks_uri: Option<String>,

    #[serde(default)]
    pub mapping_config: StoredOidcMappingConfig,
}

/// Input IdP configuration enum (for API)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputIdpConfig {
    OidcAuthorizationCode(InputOidcConfig),
    OauthAuthorizationCode(InputOauthConfig),
    OidcAuthorizationCodePkce(InputOidcConfig),
    OauthAuthorizationCodePkce(InputOauthConfig),
}

impl InputIdpConfig {
    /// Convert to business logic IdpConfig
    pub fn to_idp_config(self) -> IdpConfig {
        match self {
            InputIdpConfig::OidcAuthorizationCode(oidc) => {
                IdpConfig::OidcAuthorizationCodeFlow(input_to_oidc_config(oidc))
            }
            InputIdpConfig::OauthAuthorizationCode(oauth) => {
                IdpConfig::OauthAuthorizationCodeFlow(input_to_oauth_config(oauth))
            }
            InputIdpConfig::OidcAuthorizationCodePkce(oidc) => {
                IdpConfig::OidcAuthorizationCodePkceFlow(input_to_oidc_config(oidc))
            }
            InputIdpConfig::OauthAuthorizationCodePkce(oauth) => {
                IdpConfig::OauthAuthorizationCodePkceFlow(input_to_oauth_config(oauth))
            }
        }
    }
}

fn input_to_oauth_config(input: InputOauthConfig) -> OauthConfig {
    OauthConfig {
        authorization_endpoint: input.authorization_endpoint,
        token_endpoint: input.token_endpoint,
        userinfo_endpoint: input.userinfo_endpoint,
        client_id: input.client_id,
        client_secret: input.client_secret,
        scopes: input.scopes,
        redirect_uri: input.redirect_uri,
        post_login_redirect_uri: input.post_login_redirect_uri,
        mapping_template: from_stored_mapping_config(&input.mapping_config),
        group_to_role_mappings: input.group_to_role_mappings.iter().map(from_stored_group_mapping).collect(),
        scope_to_role_mappings: input.scope_to_role_mappings.iter().map(from_stored_scope_role_mapping).collect(),
        scope_to_group_mappings: input.scope_to_group_mappings.iter().map(from_stored_scope_group_mapping).collect(),
        default_role: Role::from_str(&input.default_role).unwrap_or(Role::User),
        allowed_domains: input.allowed_domains,
        state_ttl_seconds: input.state_ttl_seconds,
    }
}

fn input_to_oidc_config(input: InputOidcConfig) -> OidcConfig {
    OidcConfig {
        base_config: input_to_oauth_config(input.base_config),
        discovery_endpoint: input.discovery_endpoint,
        jwks_uri: input.jwks_uri,
        mapping_config: from_stored_oidc_mapping_config(&input.mapping_config),
    }
}

// ============================================
// API Request/Response Types
// ============================================

/// Parameters for creating an IdP configuration
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateIdpConfigParams {
    /// Optional ID (will be generated if not provided)
    pub id: Option<String>,
    /// The configuration
    pub config: InputIdpConfig,
}

/// Response from creating an IdP configuration
#[derive(Debug, Serialize, ToSchema)]
pub struct CreateIdpConfigResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub config_type: String,
}

/// Parameters for updating an IdP configuration
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateIdpConfigParams {
    /// The new configuration
    pub config: InputIdpConfig,
}

/// Response from updating an IdP configuration
#[derive(Debug, Serialize, ToSchema)]
pub struct UpdateIdpConfigResponse {
    pub success: bool,
}

/// Parameters for deleting an IdP configuration
#[derive(Debug, Deserialize, ToSchema)]
pub struct DeleteIdpConfigParams {
    pub id: String,
}

/// Response from deleting an IdP configuration
#[derive(Debug, Serialize, ToSchema)]
pub struct DeleteIdpConfigResponse {
    pub success: bool,
}

/// Parameters for listing IdP configurations
#[derive(Debug)]
pub struct ListIdpConfigParams {
    pub pagination: PaginationRequest,
    pub config_type: Option<String>,
}

/// Response item for listing IdP configurations
#[derive(Debug, Serialize, ToSchema)]
pub struct IdpConfigListItem {
    pub id: String,
    #[serde(rename = "type")]
    pub config_type: String,
    pub client_id: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

/// Response from listing IdP configurations
#[derive(Debug, Serialize, ToSchema)]
pub struct ListIdpConfigResponse {
    pub items: Vec<IdpConfigListItem>,
    pub next_page_token: Option<String>,
}

/// Parameters for getting an IdP configuration
#[derive(Debug, Deserialize, ToSchema)]
pub struct GetIdpConfigParams {
    pub id: String,
}

/// Response from getting an IdP configuration (returns stored format, no secret)
#[derive(Debug, Serialize, ToSchema)]
pub struct GetIdpConfigResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub config_type: String,
    pub config: StoredIdpConfig,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

// ============================================
// Logic Functions
// ============================================

/// Create a new IdP configuration
pub async fn create_idp_config<R: UserRepositoryLike>(
    repository: &R,
    crypto_cache: &CryptoCache,
    on_config_change_tx: &OnConfigChangeTx,
    params: CreateIdpConfigParams,
    publish_on_change_evt: bool,
) -> Result<CreateIdpConfigResponse, CommonError> {
    // Generate ID if not provided
    let id = params.id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let now = WrappedChronoDateTime::now();

    // Check if config with this ID already exists
    if repository.get_idp_configuration_by_id(&id).await?.is_some() {
        return Err(CommonError::InvalidRequest {
            msg: format!("IdP configuration with id '{}' already exists", id),
            source: None,
        });
    }

    // Convert input config to business logic type
    let idp_config = params.config.to_idp_config();

    // Encrypt and convert to stored format
    let stored_config = encrypt_idp_config(&idp_config, crypto_cache).await?;
    let config_type = stored_config.config_type().to_string();

    // Serialize stored config to JSON
    let config_json = serde_json::to_string(&stored_config).map_err(|e| {
        CommonError::InvalidRequest {
            msg: format!("Failed to serialize config: {e}"),
            source: Some(e.into()),
        }
    })?;

    // Create the IdP configuration in the repository
    let create_config = CreateIdpConfiguration {
        id: id.clone(),
        config_type: config_type.clone(),
        config: config_json.clone(),
        encrypted_client_secret: None, // Already in the serialized config
        dek_alias: None,               // Already in the serialized config
        created_at: now,
        updated_at: now,
    };
    repository.create_idp_configuration(&create_config).await?;

    // Broadcast config change event
    if publish_on_change_evt {
        on_config_change_tx
            .send(OnConfigChangeEvt::IdpConfigCreated(IdpConfigCreatedInfo {
                id: id.clone(),
                config_type: config_type.clone(),
                config: config_json,
            }))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send config change event: {e}"))
            })?;
    }

    Ok(CreateIdpConfigResponse { id, config_type })
}

/// Update an IdP configuration
pub async fn update_idp_config<R: UserRepositoryLike>(
    repository: &R,
    crypto_cache: &CryptoCache,
    id: &str,
    params: UpdateIdpConfigParams,
) -> Result<UpdateIdpConfigResponse, CommonError> {
    // Verify the config exists
    repository
        .get_idp_configuration_by_id(id)
        .await?
        .ok_or_else(|| CommonError::NotFound {
            msg: "IdP configuration not found".to_string(),
            lookup_id: id.to_string(),
            source: None,
        })?;

    // Convert input config to business logic type
    let idp_config = params.config.to_idp_config();

    // Encrypt and convert to stored format
    let stored_config = encrypt_idp_config(&idp_config, crypto_cache).await?;
    let config_type = stored_config.config_type().to_string();

    // Serialize stored config to JSON
    let config_json = serde_json::to_string(&stored_config).map_err(|e| {
        CommonError::InvalidRequest {
            msg: format!("Failed to serialize config: {e}"),
            source: Some(e.into()),
        }
    })?;

    // Prepare update
    let update = crate::repository::UpdateIdpConfiguration {
        config_type: Some(config_type),
        config: Some(config_json),
        encrypted_client_secret: None, // Already in the serialized config
        dek_alias: None,               // Already in the serialized config
    };
    repository.update_idp_configuration(id, &update).await?;

    Ok(UpdateIdpConfigResponse { success: true })
}

/// Delete an IdP configuration
pub async fn delete_idp_config<R: UserRepositoryLike>(
    repository: &R,
    on_config_change_tx: &OnConfigChangeTx,
    params: DeleteIdpConfigParams,
    publish_on_change_evt: bool,
) -> Result<DeleteIdpConfigResponse, CommonError> {
    // Verify the config exists
    repository
        .get_idp_configuration_by_id(&params.id)
        .await?
        .ok_or_else(|| CommonError::NotFound {
            msg: "IdP configuration not found".to_string(),
            lookup_id: params.id.clone(),
            source: None,
        })?;

    // Delete the configuration
    repository.delete_idp_configuration(&params.id).await?;

    // Broadcast config change event
    if publish_on_change_evt {
        on_config_change_tx
            .send(OnConfigChangeEvt::IdpConfigDeleted(params.id.clone()))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send config change event: {e}"))
            })?;
    }

    Ok(DeleteIdpConfigResponse { success: true })
}

/// Get an IdP configuration by ID (returns stored format without decrypted secret)
pub async fn get_idp_config<R: UserRepositoryLike>(
    repository: &R,
    params: GetIdpConfigParams,
) -> Result<GetIdpConfigResponse, CommonError> {
    let db_config = repository
        .get_idp_configuration_by_id(&params.id)
        .await?
        .ok_or_else(|| CommonError::NotFound {
            msg: "IdP configuration not found".to_string(),
            lookup_id: params.id,
            source: None,
        })?;

    let stored_config: StoredIdpConfig =
        serde_json::from_str(&db_config.config).map_err(|e| CommonError::Unknown(e.into()))?;

    Ok(GetIdpConfigResponse {
        id: db_config.id,
        config_type: db_config.config_type,
        config: stored_config,
        created_at: db_config.created_at,
        updated_at: db_config.updated_at,
    })
}

/// List IdP configurations
pub async fn list_idp_configs<R: UserRepositoryLike>(
    repository: &R,
    params: ListIdpConfigParams,
) -> Result<ListIdpConfigResponse, CommonError> {
    let result = repository
        .list_idp_configurations(&params.pagination, params.config_type.as_deref())
        .await?;

    let items: Vec<IdpConfigListItem> = result
        .items
        .into_iter()
        .filter_map(|config| {
            let stored: StoredIdpConfig = serde_json::from_str(&config.config).ok()?;
            Some(IdpConfigListItem {
                id: config.id,
                config_type: config.config_type,
                client_id: stored.client_id().to_string(),
                created_at: config.created_at,
                updated_at: config.updated_at,
            })
        })
        .collect();

    Ok(ListIdpConfigResponse {
        items,
        next_page_token: result.next_page_token,
    })
}

/// Load and decrypt an IdP configuration from the repository
pub async fn load_idp_config<R: UserRepositoryLike>(
    repository: &R,
    crypto_cache: &CryptoCache,
    config_id: &str,
) -> Result<(IdpConfiguration, IdpConfig), CommonError> {
    let db_config = repository
        .get_idp_configuration_by_id(config_id)
        .await?
        .ok_or_else(|| CommonError::NotFound {
            msg: "IdP configuration not found".to_string(),
            lookup_id: config_id.to_string(),
            source: None,
        })?;

    let stored_config: StoredIdpConfig =
        serde_json::from_str(&db_config.config).map_err(|e| CommonError::Unknown(e.into()))?;

    let idp_config = decrypt_idp_config(&stored_config, crypto_cache).await?;

    Ok((db_config, idp_config))
}

/// Import an IdP configuration (idempotent - creates if not exists, skips if exists)
/// Used for syncing configurations from soma.yaml
pub async fn import_idp_config<R: UserRepositoryLike>(
    repository: &R,
    crypto_cache: &CryptoCache,
    params: CreateIdpConfigParams,
) -> Result<CreateIdpConfigResponse, CommonError> {
    let id = params.id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());

    // Check if config already exists
    if let Some(existing) = repository.get_idp_configuration_by_id(&id).await? {
        // Config already exists - return existing info
        return Ok(CreateIdpConfigResponse {
            id: existing.id,
            config_type: existing.config_type,
        });
    }

    let now = WrappedChronoDateTime::now();

    // Convert input config to business logic type
    let idp_config = params.config.to_idp_config();

    // Encrypt and convert to stored format
    let stored_config = encrypt_idp_config(&idp_config, crypto_cache).await?;
    let config_type = stored_config.config_type().to_string();

    // Serialize stored config to JSON
    let config_json = serde_json::to_string(&stored_config).map_err(|e| {
        CommonError::InvalidRequest {
            msg: format!("Failed to serialize config: {e}"),
            source: Some(e.into()),
        }
    })?;

    // Create the IdP configuration in the repository
    let create_config = CreateIdpConfiguration {
        id: id.clone(),
        config_type: config_type.clone(),
        config: config_json,
        encrypted_client_secret: None, // Already in the serialized config
        dek_alias: None,               // Already in the serialized config
        created_at: now,
        updated_at: now,
    };
    repository.create_idp_configuration(&create_config).await?;

    Ok(CreateIdpConfigResponse { id, config_type })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idp_config_type() {
        let oauth = IdpConfig::OauthAuthorizationCodeFlow(OauthConfig {
            authorization_endpoint: "https://example.com/authorize".to_string(),
            token_endpoint: "https://example.com/token".to_string(),
            userinfo_endpoint: None,
            client_id: "test".to_string(),
            client_secret: "secret".to_string(),
            scopes: vec!["openid".to_string()],
            redirect_uri: "http://localhost/callback".to_string(),
            post_login_redirect_uri: "/".to_string(),
            mapping_template: JwtTokenMappingConfig {
                issuer_field: "iss".to_string(),
                audience_field: "aud".to_string(),
                scopes_field: None,
                sub_field: "sub".to_string(),
                email_field: Some("email".to_string()),
                groups_field: None,
            },
            group_to_role_mappings: vec![],
            scope_to_role_mappings: vec![],
            scope_to_group_mappings: vec![],
            default_role: Role::User,
            allowed_domains: vec![],
            state_ttl_seconds: 300,
        });

        assert_eq!(oauth.config_type(), "oauth_authorization_code");
        assert!(!oauth.uses_pkce());
        assert!(!oauth.is_oidc());
    }

    #[test]
    fn test_idp_config_pkce() {
        let oauth = IdpConfig::OauthAuthorizationCodePkceFlow(OauthConfig {
            authorization_endpoint: "https://example.com/authorize".to_string(),
            token_endpoint: "https://example.com/token".to_string(),
            userinfo_endpoint: None,
            client_id: "test".to_string(),
            client_secret: "secret".to_string(),
            scopes: vec!["openid".to_string()],
            redirect_uri: "http://localhost/callback".to_string(),
            post_login_redirect_uri: "/".to_string(),
            mapping_template: JwtTokenMappingConfig {
                issuer_field: "iss".to_string(),
                audience_field: "aud".to_string(),
                scopes_field: None,
                sub_field: "sub".to_string(),
                email_field: Some("email".to_string()),
                groups_field: None,
            },
            group_to_role_mappings: vec![],
            scope_to_role_mappings: vec![],
            scope_to_group_mappings: vec![],
            default_role: Role::User,
            allowed_domains: vec![],
            state_ttl_seconds: 300,
        });

        assert_eq!(oauth.config_type(), "oauth_authorization_code_pkce");
        assert!(oauth.uses_pkce());
        assert!(!oauth.is_oidc());
    }

    #[test]
    fn test_stored_config_serialization() {
        let stored = StoredIdpConfig::OauthAuthorizationCode(StoredOauthConfig {
            authorization_endpoint: "https://example.com/authorize".to_string(),
            token_endpoint: "https://example.com/token".to_string(),
            userinfo_endpoint: None,
            client_id: "test-client".to_string(),
            encrypted_client_secret: "encrypted_secret".to_string(),
            dek_alias: "default".to_string(),
            scopes: vec!["openid".to_string()],
            redirect_uri: "http://localhost/callback".to_string(),
            post_login_redirect_uri: "/".to_string(),
            mapping_config: StoredMappingConfig::default(),
            group_to_role_mappings: vec![],
            scope_to_role_mappings: vec![],
            scope_to_group_mappings: vec![],
            default_role: "user".to_string(),
            allowed_domains: vec![],
            state_ttl_seconds: 300,
        });

        let json = serde_json::to_string(&stored).expect("Should serialize");
        assert!(json.contains("oauth_authorization_code"));
        assert!(json.contains("encrypted_client_secret"));
        assert!(json.contains("dek_alias"));
        assert!(!json.contains("client_secret\":\"secret")); // No plaintext secret

        let deserialized: StoredIdpConfig = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.config_type(), "oauth_authorization_code");
        assert_eq!(deserialized.client_id(), "test-client");
    }

    #[test]
    fn test_token_source_serialization() {
        let id_token = StoredTokenSource::IdToken("sub".to_string());
        let json = serde_json::to_string(&id_token).expect("Should serialize");
        assert!(json.contains("id_token"));

        let userinfo = StoredTokenSource::Userinfo("email".to_string());
        let json = serde_json::to_string(&userinfo).expect("Should serialize");
        assert!(json.contains("userinfo"));
    }

    #[test]
    fn test_determine_role_from_groups() {
        let config = IdpConfig::OauthAuthorizationCodeFlow(OauthConfig {
            authorization_endpoint: "https://example.com/authorize".to_string(),
            token_endpoint: "https://example.com/token".to_string(),
            userinfo_endpoint: None,
            client_id: "test".to_string(),
            client_secret: "secret".to_string(),
            scopes: vec![],
            redirect_uri: "http://localhost/callback".to_string(),
            post_login_redirect_uri: "/".to_string(),
            mapping_template: JwtTokenMappingConfig {
                issuer_field: "iss".to_string(),
                audience_field: "aud".to_string(),
                scopes_field: None,
                sub_field: "sub".to_string(),
                email_field: None,
                groups_field: None,
            },
            group_to_role_mappings: vec![JwtGroupToRoleMapping {
                group: "Admins".to_string(),
                role: Role::Admin,
            }],
            scope_to_role_mappings: vec![],
            scope_to_group_mappings: vec![],
            default_role: Role::User,
            allowed_domains: vec![],
            state_ttl_seconds: 300,
        });

        // Match (case-insensitive via standardize_group_name)
        let role = config.determine_role_from_groups(&["admins".to_string()]);
        assert_eq!(role, Role::Admin);

        // No match - returns default
        let role = config.determine_role_from_groups(&["other".to_string()]);
        assert_eq!(role, Role::User);
    }
}
