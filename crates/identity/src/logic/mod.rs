use crate::logic::auth_client::Role;

pub mod api_key;
pub mod api_key_cache;
pub mod auth_client;
pub mod auth_config;
pub mod idp_config;
pub mod jwk;
pub mod jwks_cache;
pub mod oauth;
pub mod sts_config;
pub mod sts_exchange;


/// Default DEK alias for client secret encryption
pub const DEFAULT_DEK_ALIAS: &str = "default";

/// Information about a created API key (for broadcast events)
#[derive(Clone, Debug)]
pub struct ApiKeyCreatedInfo {
    /// The API key ID
    pub id: String,
    /// The encrypted hashed value of the API key
    pub encrypted_hashed_value: String,
    /// The DEK alias used for encryption
    pub dek_alias: String,
    pub role: Role,
    pub user_id: String,
}

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
    ApiKeyCreated(ApiKeyCreatedInfo),
    /// An API key was deleted (contains id)
    ApiKeyDeleted(String),
    /// An STS configuration was created
    StsConfigCreated(StsConfigCreatedInfo),
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
