//! IdP Configuration management for OAuth/OIDC external identity providers.
//!
//! This module contains:
//! - Business logic types for working with IdP configurations in memory
//! - Stored types for serialization to/from the database (with encrypted secrets)
//! - CRUD operations for IdP configurations

use encryption::logic::CryptoCache;
use encryption::logic::crypto_services::EncryptedString;
use serde::{Deserialize, Serialize};
use shared::error::CommonError;
use utoipa::ToSchema;

use crate::logic::token_mapping::TokenMapping;

// ============================================
// Business Logic Types (used in application code)
// ============================================

/// OAuth2 configuration for authorization code flow
#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct OauthConfig {
    pub id: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub client_id: String,
    pub client_secret: String,
    pub scopes: Vec<String>,
    pub jwks_endpoint: String,
    /// Token introspection endpoint URL (RFC 7662)
    /// If set, access tokens are treated as opaque and introspected via this endpoint
    pub introspect_url: Option<String>,
    pub mapping: TokenMapping,
}

/// OIDC configuration extending OAuth2 with discovery and ID token support
#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct OidcConfig {
    pub id: String,
    pub base_config: OauthConfig,
    pub discovery_endpoint: Option<String>,
    pub mapping: TokenMapping,
    pub userinfo_endpoint: Option<String>,
    /// Token introspection endpoint URL (RFC 7662)
    /// If set, access tokens are treated as opaque and introspected via this endpoint
    pub introspect_url: Option<String>,
}

/// The four supported IdP configuration types
#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum UserAuthFlowConfig {
    OidcAuthorizationCodeFlow(OidcConfig),
    OauthAuthorizationCodeFlow(OauthConfig),
    OidcAuthorizationCodePkceFlow(OidcConfig),
    OauthAuthorizationCodePkceFlow(OauthConfig),
}

impl UserAuthFlowConfig {
    /// Get the ID of the config
    pub fn id(&self) -> &str {
        match self {
            UserAuthFlowConfig::OidcAuthorizationCodeFlow(c) => &c.id,
            UserAuthFlowConfig::OauthAuthorizationCodeFlow(c) => &c.id,
            UserAuthFlowConfig::OidcAuthorizationCodePkceFlow(c) => &c.id,
            UserAuthFlowConfig::OauthAuthorizationCodePkceFlow(c) => &c.id,
        }
    }

    /// Encrypt a user auth flow config using the crypto cache and the specified DEK alias.
    pub async fn encrypt(
        &self,
        crypto_cache: &CryptoCache,
        dek_alias: &str,
    ) -> Result<EncryptedUserAuthFlowConfig, CommonError> {
        match self {
            UserAuthFlowConfig::OidcAuthorizationCodeFlow(oidc) => {
                let encrypted_oidc = encrypt_oidc_config(crypto_cache, dek_alias, oidc).await?;
                Ok(EncryptedUserAuthFlowConfig::OidcAuthorizationCodeFlow(
                    encrypted_oidc,
                ))
            }
            UserAuthFlowConfig::OauthAuthorizationCodeFlow(oauth) => {
                let encrypted_oauth = encrypt_oauth_config(crypto_cache, dek_alias, oauth).await?;
                Ok(EncryptedUserAuthFlowConfig::OauthAuthorizationCodeFlow(
                    encrypted_oauth,
                ))
            }
            UserAuthFlowConfig::OidcAuthorizationCodePkceFlow(oidc) => {
                let encrypted_oidc = encrypt_oidc_config(crypto_cache, dek_alias, oidc).await?;
                Ok(EncryptedUserAuthFlowConfig::OidcAuthorizationCodePkceFlow(
                    encrypted_oidc,
                ))
            }
            UserAuthFlowConfig::OauthAuthorizationCodePkceFlow(oauth) => {
                let encrypted_oauth = encrypt_oauth_config(crypto_cache, dek_alias, oauth).await?;
                Ok(EncryptedUserAuthFlowConfig::OauthAuthorizationCodePkceFlow(
                    encrypted_oauth,
                ))
            }
        }
    }
}

/// OAuth2 configuration for authorization code flow
#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct EncryptedOauthConfig {
    pub id: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub jwks_endpoint: String,
    pub client_id: String,
    pub encrypted_client_secret: EncryptedString,
    pub dek_alias: String,
    pub scopes: Vec<String>,
    /// Token introspection endpoint URL (RFC 7662)
    pub introspect_url: Option<String>,
    pub mapping: TokenMapping,
}

/// OIDC configuration extending OAuth2 with discovery and ID token support
#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct EncryptedOidcConfig {
    pub id: String,
    pub base_config: EncryptedOauthConfig,
    pub discovery_endpoint: Option<String>,
    pub userinfo_endpoint: Option<String>,
    /// Token introspection endpoint URL (RFC 7662)
    pub introspect_url: Option<String>,
    pub mapping: TokenMapping,
}

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EncryptedUserAuthFlowConfig {
    OidcAuthorizationCodeFlow(EncryptedOidcConfig),
    OauthAuthorizationCodeFlow(EncryptedOauthConfig),
    OidcAuthorizationCodePkceFlow(EncryptedOidcConfig),
    OauthAuthorizationCodePkceFlow(EncryptedOauthConfig),
}

impl EncryptedUserAuthFlowConfig {
    /// Get the ID of the encrypted config
    pub fn id(&self) -> &str {
        match self {
            EncryptedUserAuthFlowConfig::OidcAuthorizationCodeFlow(c) => &c.id,
            EncryptedUserAuthFlowConfig::OauthAuthorizationCodeFlow(c) => &c.id,
            EncryptedUserAuthFlowConfig::OidcAuthorizationCodePkceFlow(c) => &c.id,
            EncryptedUserAuthFlowConfig::OauthAuthorizationCodePkceFlow(c) => &c.id,
        }
    }

    /// Decrypt an encrypted user auth flow config using the crypto cache.
    /// The appropriate DEK is looked up using the dek_alias stored in the encrypted config.
    pub async fn decrypt(
        &self,
        crypto_cache: &CryptoCache,
    ) -> Result<UserAuthFlowConfig, CommonError> {
        match self {
            EncryptedUserAuthFlowConfig::OidcAuthorizationCodeFlow(encrypted_oidc) => {
                let oidc = decrypt_oidc_config(crypto_cache, encrypted_oidc).await?;
                Ok(UserAuthFlowConfig::OidcAuthorizationCodeFlow(oidc))
            }
            EncryptedUserAuthFlowConfig::OauthAuthorizationCodeFlow(encrypted_oauth) => {
                let oauth = decrypt_oauth_config(crypto_cache, encrypted_oauth).await?;
                Ok(UserAuthFlowConfig::OauthAuthorizationCodeFlow(oauth))
            }
            EncryptedUserAuthFlowConfig::OidcAuthorizationCodePkceFlow(encrypted_oidc) => {
                let oidc = decrypt_oidc_config(crypto_cache, encrypted_oidc).await?;
                Ok(UserAuthFlowConfig::OidcAuthorizationCodePkceFlow(oidc))
            }
            EncryptedUserAuthFlowConfig::OauthAuthorizationCodePkceFlow(encrypted_oauth) => {
                let oauth = decrypt_oauth_config(crypto_cache, encrypted_oauth).await?;
                Ok(UserAuthFlowConfig::OauthAuthorizationCodePkceFlow(oauth))
            }
        }
    }
}

// Helper functions for encrypting/decrypting OAuth and OIDC configs

async fn decrypt_oauth_config(
    crypto_cache: &CryptoCache,
    encrypted: &EncryptedOauthConfig,
) -> Result<OauthConfig, CommonError> {
    let decryption_service = crypto_cache
        .get_decryption_service(&encrypted.dek_alias)
        .await?;
    let client_secret = decryption_service
        .decrypt_data(encrypted.encrypted_client_secret.clone())
        .await?;

    Ok(OauthConfig {
        id: encrypted.id.clone(),
        authorization_endpoint: encrypted.authorization_endpoint.clone(),
        token_endpoint: encrypted.token_endpoint.clone(),
        jwks_endpoint: encrypted.jwks_endpoint.clone(),
        client_id: encrypted.client_id.clone(),
        client_secret,
        scopes: encrypted.scopes.clone(),
        introspect_url: encrypted.introspect_url.clone(),
        mapping: encrypted.mapping.clone(),
    })
}

async fn encrypt_oauth_config(
    crypto_cache: &CryptoCache,
    dek_alias: &str,
    oauth: &OauthConfig,
) -> Result<EncryptedOauthConfig, CommonError> {
    let encryption_service = crypto_cache.get_encryption_service(dek_alias).await?;
    let encrypted_client_secret = encryption_service
        .encrypt_data(oauth.client_secret.clone())
        .await?;

    Ok(EncryptedOauthConfig {
        id: oauth.id.clone(),
        authorization_endpoint: oauth.authorization_endpoint.clone(),
        token_endpoint: oauth.token_endpoint.clone(),
        jwks_endpoint: oauth.jwks_endpoint.clone(),
        client_id: oauth.client_id.clone(),
        encrypted_client_secret,
        dek_alias: dek_alias.to_string(),
        scopes: oauth.scopes.clone(),
        introspect_url: oauth.introspect_url.clone(),
        mapping: oauth.mapping.clone(),
    })
}

async fn decrypt_oidc_config(
    crypto_cache: &CryptoCache,
    encrypted: &EncryptedOidcConfig,
) -> Result<OidcConfig, CommonError> {
    let base_config = decrypt_oauth_config(crypto_cache, &encrypted.base_config).await?;

    Ok(OidcConfig {
        id: encrypted.id.clone(),
        base_config,
        discovery_endpoint: encrypted.discovery_endpoint.clone(),
        userinfo_endpoint: encrypted.userinfo_endpoint.clone(),
        introspect_url: encrypted.introspect_url.clone(),
        mapping: encrypted.mapping.clone(),
    })
}

async fn encrypt_oidc_config(
    crypto_cache: &CryptoCache,
    dek_alias: &str,
    oidc: &OidcConfig,
) -> Result<EncryptedOidcConfig, CommonError> {
    let base_config = encrypt_oauth_config(crypto_cache, dek_alias, &oidc.base_config).await?;

    Ok(EncryptedOidcConfig {
        id: oidc.id.clone(),
        base_config,
        discovery_endpoint: oidc.discovery_endpoint.clone(),
        userinfo_endpoint: oidc.userinfo_endpoint.clone(),
        introspect_url: oidc.introspect_url.clone(),
        mapping: oidc.mapping.clone(),
    })
}
