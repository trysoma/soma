use shared::error::CommonError;

use crate::logic::api_key::EncryptedApiKeyConfig;
use crate::logic::sts::config::StsTokenConfig;
use crate::logic::user_auth_flow::config::EncryptedUserAuthFlowConfig;

pub mod api_key;
pub mod auth_client;
pub mod internal_token_issuance;
pub mod jwk;
pub mod sts;
pub mod token_mapping;
pub mod user;
pub mod user_auth_flow;

/// Default DEK alias for client secret encryption
pub const DEFAULT_DEK_ALIAS: &str = "default";

/// Validate that an ID is a valid identifier.
///
/// Valid IDs must:
/// - Not be empty
/// - Only contain lowercase letters, numbers, and hyphens
/// - Start with a letter
/// - Not end with a hyphen
/// - Not contain consecutive hyphens
pub fn validate_id(id: &str, resource_type: &str) -> Result<(), CommonError> {
    if id.is_empty() {
        return Err(CommonError::InvalidRequest {
            msg: format!("{resource_type} ID cannot be empty"),
            source: None,
        });
    }

    // Check that it starts with a letter
    if !id.chars().next().unwrap().is_ascii_lowercase() {
        return Err(CommonError::InvalidRequest {
            msg: format!(
                "{resource_type} ID must start with a lowercase letter, got: '{id}'"
            ),
            source: None,
        });
    }

    // Check for valid characters
    for (i, c) in id.chars().enumerate() {
        if !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '-' {
            return Err(CommonError::InvalidRequest {
                msg: format!(
                    "{resource_type} ID can only contain lowercase letters, numbers, and hyphens. Invalid character '{c}' at position {i}"
                ),
                source: None,
            });
        }
    }

    // Check that it doesn't end with a hyphen
    if id.ends_with('-') {
        return Err(CommonError::InvalidRequest {
            msg: format!("{resource_type} ID cannot end with a hyphen: '{id}'"),
            source: None,
        });
    }

    // Check for consecutive hyphens
    if id.contains("--") {
        return Err(CommonError::InvalidRequest {
            msg: format!(
                "{resource_type} ID cannot contain consecutive hyphens: '{id}'"
            ),
            source: None,
        });
    }

    Ok(())
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
    /// A user auth flow configuration was created
    UserAuthFlowConfigCreated(EncryptedUserAuthFlowConfig),
    /// A user auth flow configuration was deleted (contains id)
    UserAuthFlowConfigDeleted(String),
}

/// Sender for config change events
pub type OnConfigChangeTx = tokio::sync::broadcast::Sender<OnConfigChangeEvt>;
/// Receiver for config change events
pub type OnConfigChangeRx = tokio::sync::broadcast::Receiver<OnConfigChangeEvt>;

