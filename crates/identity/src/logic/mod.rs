use crate::logic::api_key::EncryptedApiKeyConfig;
use crate::logic::sts::config::StsTokenConfig;
use serde::{Deserialize, Serialize};
use shared::primitives::WrappedChronoDateTime;
use utoipa::ToSchema;

pub mod api_key;
pub mod auth_client;
pub mod user_auth_flow;
pub mod jwk;
pub mod sts;
pub mod token_mapping;
pub mod internal_token_issuance;
pub mod user;

/// Default DEK alias for client secret encryption
pub const DEFAULT_DEK_ALIAS: &str = "default";


/// Information about a created STS configuration (for broadcast events)
#[derive(Clone, Debug)]
pub struct StsConfigCreatedInfo {
    /// The STS configuration ID
    pub id: String,
    /// The configuration type (e.g., "jwt_template", "dev")
    pub config_type: String,
    /// The configuration value (JSON)
    pub value: Option<String>,
}

/// Information about a created IdP configuration (for broadcast events)
#[derive(Clone, Debug)]
pub struct IdpConfigCreatedInfo {
    /// The IdP configuration ID
    pub id: String,
    /// The configuration type (e.g., "oidc_authorization_flow", "oauth_authorization_flow")
    pub config_type: String,
    /// The configuration (JSON)
    pub config: String,
}

/// Events fired when identity configuration changes
#[derive(Clone, Debug)]
pub enum OnConfigChangeEvt {
    /// An API key was created
    ApiKeyCreated(EncryptedApiKeyConfig),
    /// An API key was deleted (contains id)
    ApiKeyDeleted(String),
    /// An STS configuration was created
    StsConfigCreated(StsTokenConfig),
    /// An STS configuration was deleted (contains id)
    StsConfigDeleted(String),
    /// An IdP configuration was created
    IdpConfigCreated(IdpConfigCreatedInfo),
    /// An IdP configuration was deleted (contains id)
    IdpConfigDeleted(String),
}

/// Sender for config change events
pub type OnConfigChangeTx = tokio::sync::broadcast::Sender<OnConfigChangeEvt>;
/// Receiver for config change events
pub type OnConfigChangeRx = tokio::sync::broadcast::Receiver<OnConfigChangeEvt>;

