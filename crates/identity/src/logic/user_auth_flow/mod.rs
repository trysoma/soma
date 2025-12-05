//! User authentication flow configuration management.
//!
//! This module provides CRUD operations for OAuth/OIDC user authentication flow configurations.

pub mod config;
pub mod oauth;
pub mod oidc;

use chrono::Utc;
use encryption::logic::CryptoCache;
use serde::{Deserialize, Serialize};
use shared::error::CommonError;
use shared::primitives::{PaginationRequest, WrappedChronoDateTime};
use utoipa::ToSchema;

use crate::logic::internal_token_issuance::NormalizedTokenIssuanceResult;
use crate::logic::sts::external_jwk_cache::ExternalJwksCache;
use crate::logic::{DEFAULT_DEK_ALIAS, OnConfigChangeEvt, OnConfigChangeTx, validate_id};
use crate::repository::{UserAuthFlowConfigDb, UserRepositoryLike};

pub use config::{
    EncryptedOauthConfig, EncryptedOidcConfig, EncryptedUserAuthFlowConfig, OauthConfig,
    OidcConfig, UserAuthFlowConfig,
};

// ============================================
// Request/Response Types
// ============================================

/// Parameters for creating a user auth flow configuration
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateUserAuthFlowConfigParams {
    /// The configuration to create (unencrypted)
    pub config: UserAuthFlowConfig,
}

/// Response from creating a user auth flow configuration
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CreateUserAuthFlowConfigResponse {
    /// The ID of the created configuration
    pub id: String,
}

/// Parameters for getting a user auth flow configuration
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct GetUserAuthFlowConfigParams {
    /// The ID of the configuration to get
    pub id: String,
}

/// Response from getting a user auth flow configuration
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GetUserAuthFlowConfigResponse {
    /// The configuration (encrypted)
    pub config: EncryptedUserAuthFlowConfig,
    /// When the configuration was created
    pub created_at: WrappedChronoDateTime,
    /// When the configuration was last updated
    pub updated_at: WrappedChronoDateTime,
}

/// Parameters for deleting a user auth flow configuration
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct DeleteUserAuthFlowConfigParams {
    /// The ID of the configuration to delete
    pub id: String,
}

/// Response from deleting a user auth flow configuration
pub type DeleteUserAuthFlowConfigResponse = ();

/// Parameters for listing user auth flow configurations
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ListUserAuthFlowConfigParams {
    /// Pagination parameters
    pub pagination: PaginationRequest,
    /// Optional filter by config type
    pub config_type: Option<String>,
}

/// Response from listing user auth flow configurations
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ListUserAuthFlowConfigResponse {
    /// The configurations
    pub items: Vec<GetUserAuthFlowConfigResponse>,
    /// Token for the next page, if any
    pub next_page_token: Option<String>,
}

/// Parameters for importing a user auth flow configuration (already encrypted)
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ImportUserAuthFlowConfigParams {
    /// The encrypted configuration to import
    pub config: EncryptedUserAuthFlowConfig,
}

/// Response from importing a user auth flow configuration
pub type ImportUserAuthFlowConfigResponse = ();

// ============================================
// CRUD Operations
// ============================================

/// Create a new user auth flow configuration.
///
/// This function:
/// 1. Encrypts the configuration using the default DEK
/// 2. Stores it in the database
/// 3. Optionally broadcasts a config change event
pub async fn create_user_auth_flow_config<R: UserRepositoryLike>(
    repository: &R,
    crypto_cache: &CryptoCache,
    on_config_change_tx: &OnConfigChangeTx,
    params: CreateUserAuthFlowConfigParams,
    publish_on_change_evt: bool,
) -> Result<CreateUserAuthFlowConfigResponse, CommonError> {
    let id = params.config.id().to_string();

    // Validate the ID
    validate_id(&id, "User auth flow config")?;

    // Check if config with this ID already exists
    if repository
        .get_user_auth_flow_config_by_id(&id)
        .await?
        .is_some()
    {
        return Err(CommonError::InvalidRequest {
            msg: format!("User auth flow config with ID '{id}' already exists"),
            source: None,
        });
    }

    // Encrypt the configuration
    let encrypted_config = params
        .config
        .encrypt(crypto_cache, DEFAULT_DEK_ALIAS)
        .await?;

    let now = WrappedChronoDateTime::now();

    // Store in database
    let db_entry = UserAuthFlowConfigDb {
        id: id.clone(),
        config: encrypted_config.clone(),
        created_at: now,
        updated_at: now,
    };
    repository.create_user_auth_flow_config(&db_entry).await?;

    // Broadcast config change event
    if publish_on_change_evt {
        on_config_change_tx
            .send(OnConfigChangeEvt::UserAuthFlowConfigCreated(
                encrypted_config,
            ))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send config change event: {e}"))
            })?;
    }

    Ok(CreateUserAuthFlowConfigResponse { id })
}

/// Get a user auth flow configuration by ID.
pub async fn get_user_auth_flow_config<R: UserRepositoryLike>(
    repository: &R,
    params: GetUserAuthFlowConfigParams,
) -> Result<GetUserAuthFlowConfigResponse, CommonError> {
    let db_entry = repository
        .get_user_auth_flow_config_by_id(&params.id)
        .await?
        .ok_or_else(|| CommonError::NotFound {
            msg: "User auth flow configuration not found".to_string(),
            lookup_id: params.id.clone(),
            source: None,
        })?;

    Ok(GetUserAuthFlowConfigResponse {
        config: db_entry.config,
        created_at: db_entry.created_at,
        updated_at: db_entry.updated_at,
    })
}

/// Delete a user auth flow configuration by ID.
///
/// This function:
/// 1. Verifies the configuration exists
/// 2. Deletes it from the database
/// 3. Optionally broadcasts a config change event
pub async fn delete_user_auth_flow_config<R: UserRepositoryLike>(
    repository: &R,
    on_config_change_tx: &OnConfigChangeTx,
    params: DeleteUserAuthFlowConfigParams,
    publish_on_change_evt: bool,
) -> Result<DeleteUserAuthFlowConfigResponse, CommonError> {
    // Verify the configuration exists
    let _ = repository
        .get_user_auth_flow_config_by_id(&params.id)
        .await?
        .ok_or_else(|| CommonError::NotFound {
            msg: "User auth flow configuration not found".to_string(),
            lookup_id: params.id.clone(),
            source: None,
        })?;

    // Delete from database
    repository.delete_user_auth_flow_config(&params.id).await?;

    // Broadcast config change event
    if publish_on_change_evt {
        on_config_change_tx
            .send(OnConfigChangeEvt::UserAuthFlowConfigDeleted(params.id))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send config change event: {e}"))
            })?;
    }

    Ok(())
}

/// List user auth flow configurations.
pub async fn list_user_auth_flow_configs<R: UserRepositoryLike>(
    repository: &R,
    params: ListUserAuthFlowConfigParams,
) -> Result<ListUserAuthFlowConfigResponse, CommonError> {
    let result = repository
        .list_user_auth_flow_configs(&params.pagination, params.config_type.as_deref())
        .await?;

    let items = result
        .items
        .into_iter()
        .map(|db_entry| GetUserAuthFlowConfigResponse {
            config: db_entry.config,
            created_at: db_entry.created_at,
            updated_at: db_entry.updated_at,
        })
        .collect();

    Ok(ListUserAuthFlowConfigResponse {
        items,
        next_page_token: result.next_page_token,
    })
}

/// Import a user auth flow configuration (already encrypted).
///
/// This function imports an already encrypted configuration into the database.
/// It's idempotent - if a config with the same ID exists, it's skipped.
/// This is used for syncing configurations from soma.yaml.
pub async fn import_user_auth_flow_config<R: UserRepositoryLike>(
    repository: &R,
    params: ImportUserAuthFlowConfigParams,
) -> Result<ImportUserAuthFlowConfigResponse, CommonError> {
    let id = params.config.id().to_string();

    // Check if configuration already exists
    if repository
        .get_user_auth_flow_config_by_id(&id)
        .await?
        .is_some()
    {
        // Configuration already exists, skip import
        return Ok(());
    }

    let now = WrappedChronoDateTime::now();

    // Store in database
    let db_entry = UserAuthFlowConfigDb {
        id,
        config: params.config,
        created_at: now,
        updated_at: now,
    };
    repository.create_user_auth_flow_config(&db_entry).await?;

    Ok(())
}

/// Parameters for starting the OAuth authorization flow
#[derive(Debug)]
pub struct StartAuthorizationParams {
    /// IdP configuration ID
    pub config_id: String,
    /// Optional override for post-login redirect
    pub redirect_after_login: Option<String>,
}

/// Result of starting the authorization flow
#[derive(Debug, Serialize, ToSchema)]
pub struct StartAuthorizationResult {
    /// The URL to redirect the user to
    pub login_redirect_url: String,
}

pub async fn start_authorization_handshake<R: UserRepositoryLike>(
    repository: &R,
    crypto_cache: &CryptoCache,
    base_redirect_uri: &str,
    params: StartAuthorizationParams,
) -> Result<StartAuthorizationResult, CommonError> {
    let config = repository
        .get_user_auth_flow_config_by_id(&params.config_id)
        .await?;
    let config = match config {
        Some(config) => config,
        None => {
            return Err(CommonError::NotFound {
                msg: "User auth flow configuration not found".to_string(),
                lookup_id: params.config_id.clone(),
                source: None,
            });
        }
    };

    let config = config.config.decrypt(crypto_cache).await?;

    match config {
        UserAuthFlowConfig::OidcAuthorizationCodeFlow(_) => {
            return self::oidc::start_authorization_handshake(
                repository,
                crypto_cache,
                base_redirect_uri,
                params,
            )
            .await;
        }
        UserAuthFlowConfig::OauthAuthorizationCodeFlow(_) => {
            return self::oauth::start_authorization_handshake(
                repository,
                crypto_cache,
                base_redirect_uri,
                params,
            )
            .await;
        }
        UserAuthFlowConfig::OidcAuthorizationCodePkceFlow(_) => {
            return self::oidc::start_authorization_handshake(
                repository,
                crypto_cache,
                base_redirect_uri,
                params,
            )
            .await;
        }
        UserAuthFlowConfig::OauthAuthorizationCodePkceFlow(_) => {
            return self::oauth::start_authorization_handshake(
                repository,
                crypto_cache,
                base_redirect_uri,
                params,
            )
            .await;
        }
    }
}

/// Parameters for handling the OAuth callback
#[derive(Debug, Deserialize)]
pub struct OAuthCallbackParams {
    /// Authorization code from the IdP
    pub code: String,
    /// State parameter (for CSRF validation)
    pub state: String,
    /// Error from the IdP (if any)
    pub error: Option<String>,
    /// Error description from the IdP
    pub error_description: Option<String>,
}

/// Result of handling the OAuth callback
#[derive(Debug, Serialize, ToSchema)]
pub struct OAuthCallbackResult {
    pub issued_tokens: NormalizedTokenIssuanceResult,
    /// Optional redirect URI after login
    pub redirect_uri: Option<String>,
}

/// Handle the OAuth2 callback.
///
/// This function:
/// 1. Validates state parameter
/// 2. Exchanges authorization code for tokens
/// 3. Fetches userinfo
/// 4. Applies the mapping template to extract normalized fields
/// 5. Issues internal access/refresh tokens
pub async fn handle_authorization_handshake_callback<R: UserRepositoryLike>(
    repository: &R,
    crypto_cache: &CryptoCache,
    external_jwks_cache: &ExternalJwksCache,
    params: OAuthCallbackParams,
    base_redirect_uri: &str,
) -> Result<OAuthCallbackResult, CommonError> {
    // Check for error response from IdP
    if let Some(error) = &params.error {
        return Err(CommonError::InvalidRequest {
            msg: format!(
                "OAuth error from IdP: {} - {}",
                error,
                params
                    .error_description
                    .as_deref()
                    .unwrap_or("No description")
            ),
            source: None,
        });
    }

    // Validate state and get stored data
    let oauth_state = repository
        .get_oauth_state_by_state(&params.state)
        .await?
        .ok_or_else(|| CommonError::InvalidRequest {
            msg: "Invalid or expired state parameter".to_string(),
            source: None,
        })?;

    // Check if state has expired
    if oauth_state.expires_at.get_inner() < &Utc::now() {
        repository.delete_oauth_state(&params.state).await?;
        return Err(CommonError::InvalidRequest {
            msg: "State parameter has expired".to_string(),
            source: None,
        });
    }

    // Delete state (one-time use)
    repository.delete_oauth_state(&params.state).await?;

    // Load and decrypt config
    let config_db = repository
        .get_user_auth_flow_config_by_id(&oauth_state.config_id)
        .await?
        .ok_or_else(|| CommonError::InvalidRequest {
            msg: "Configuration not found".to_string(),
            source: None,
        })?;

    let config = config_db.config.decrypt(crypto_cache).await?;

    match config {
        UserAuthFlowConfig::OidcAuthorizationCodeFlow(config) => {
            return self::oidc::handle_authorization_handshake_callback(
                repository,
                crypto_cache,
                external_jwks_cache,
                base_redirect_uri,
                params,
                &config,
                &oauth_state,
            )
            .await;
        }
        UserAuthFlowConfig::OauthAuthorizationCodeFlow(config) => {
            return self::oauth::handle_authorization_handshake_callback(
                repository,
                crypto_cache,
                external_jwks_cache,
                base_redirect_uri,
                params,
                &config,
                &oauth_state,
            )
            .await;
        }
        UserAuthFlowConfig::OidcAuthorizationCodePkceFlow(config) => {
            return self::oidc::handle_authorization_handshake_callback(
                repository,
                crypto_cache,
                external_jwks_cache,
                base_redirect_uri,
                params,
                &config,
                &oauth_state,
            )
            .await;
        }
        UserAuthFlowConfig::OauthAuthorizationCodePkceFlow(config) => {
            return self::oauth::handle_authorization_handshake_callback(
                repository,
                crypto_cache,
                external_jwks_cache,
                base_redirect_uri,
                params,
                &config,
                &oauth_state,
            )
            .await;
        }
    }
}
