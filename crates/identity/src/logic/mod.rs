use crate::logic::auth_client::Role;

pub mod api_key;
pub mod api_key_cache;
pub mod auth_client;
pub mod auth_config;
pub mod jwk;
pub mod jwks_cache;
pub mod sts_exchange;

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

/// Events fired when identity configuration changes
#[derive(Clone, Debug)]
pub enum OnConfigChangeEvt {
    /// An API key was created
    ApiKeyCreated(ApiKeyCreatedInfo),
    /// An API key was deleted (contains id)
    ApiKeyDeleted(String),
}

/// Sender for config change events
pub type OnConfigChangeTx = tokio::sync::broadcast::Sender<OnConfigChangeEvt>;
/// Receiver for config change events
pub type OnConfigChangeRx = tokio::sync::broadcast::Receiver<OnConfigChangeEvt>;
