use crate::logic::sts::config::{JwtTemplateModeConfig, StsTokenConfig};
use crate::logic::user_auth_flow::config::EncryptedUserAuthFlowConfig;
use crate::repository::{
    Group, GroupMemberWithUser, GroupMembership, HashedApiKey, HashedApiKeyWithUser, JwtSigningKey,
    OAuthState, StsConfigurationDb, User, UserAuthFlowConfigDb, UserGroupWithGroup,
};
use shared::error::CommonError;
use shared::primitives::WrappedJsonValue;

// Import generated Row types from parent module
use super::{
    Row_get_api_key_by_hashed_value, Row_get_api_key_by_id, Row_get_api_keys, Row_get_group_by_id,
    Row_get_group_members, Row_get_group_membership, Row_get_groups,
    Row_get_jwt_signing_key_by_kid, Row_get_jwt_signing_keys, Row_get_oauth_state_by_state,
    Row_get_sts_configuration_by_id, Row_get_sts_configurations,
    Row_get_user_auth_flow_config_by_id, Row_get_user_auth_flow_configs, Row_get_user_by_id,
    Row_get_user_groups, Row_get_users,
};

// Helper function to deserialize StsTokenConfig from database row
fn deserialize_sts_config(
    id: String,
    config_type: String,
    value: Option<shared::primitives::WrappedJsonValue>,
) -> Result<StsTokenConfig, CommonError> {
    match config_type.as_str() {
        "dev" => Ok(StsTokenConfig::DevMode(
            crate::logic::sts::config::DevModeConfig { id },
        )),
        "jwt_template" => {
            let value = value.ok_or_else(|| CommonError::Repository {
                msg: "jwt_template config requires a value".to_string(),
                source: None,
            })?;
            let config: JwtTemplateModeConfig = serde_json::from_value(value.into_inner())
                .map_err(|e| CommonError::Repository {
                    msg: format!("Failed to deserialize jwt_template config: {e}"),
                    source: Some(e.into()),
                })?;
            Ok(StsTokenConfig::JwtTemplate(config))
        }
        _ => Err(CommonError::Repository {
            msg: format!("Unknown STS config type: {config_type}"),
            source: None,
        }),
    }
}

// Conversions from generated Row types to domain types

impl From<Row_get_user_by_id> for User {
    fn from(row: Row_get_user_by_id) -> Self {
        User {
            id: row.id,
            user_type: row.user_type,
            email: row.email,
            role: row.role,
            description: row.description,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<Row_get_users> for User {
    fn from(row: Row_get_users) -> Self {
        User {
            id: row.id,
            user_type: row.user_type,
            email: row.email,
            role: row.role,
            description: row.description,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<Row_get_api_keys> for HashedApiKey {
    fn from(row: Row_get_api_keys) -> Self {
        HashedApiKey {
            id: row.id,
            hashed_value: row.hashed_value,
            description: row.description,
            user_id: row.user_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<Row_get_api_key_by_hashed_value> for HashedApiKeyWithUser {
    fn from(row: Row_get_api_key_by_hashed_value) -> Self {
        HashedApiKeyWithUser {
            api_key: HashedApiKey {
                id: row.id,
                hashed_value: row.hashed_value,
                description: row.description,
                user_id: row.user_id.clone(),
                created_at: row.created_at,
                updated_at: row.updated_at,
            },
            user: User {
                id: row.user_id_fk,
                user_type: row.user_type,
                email: row.user_email,
                role: row.user_role,
                description: row.user_description,
                created_at: row.user_created_at,
                updated_at: row.user_updated_at,
            },
        }
    }
}

impl From<Row_get_api_key_by_id> for HashedApiKeyWithUser {
    fn from(row: Row_get_api_key_by_id) -> Self {
        HashedApiKeyWithUser {
            api_key: HashedApiKey {
                id: row.id,
                hashed_value: row.hashed_value,
                description: row.description,
                user_id: row.user_id.clone(),
                created_at: row.created_at,
                updated_at: row.updated_at,
            },
            user: User {
                id: row.user_id_fk,
                user_type: row.user_type,
                email: row.user_email,
                role: row.user_role,
                description: row.user_description,
                created_at: row.user_created_at,
                updated_at: row.user_updated_at,
            },
        }
    }
}

// Group conversions

impl From<Row_get_group_by_id> for Group {
    fn from(row: Row_get_group_by_id) -> Self {
        Group {
            id: row.id,
            name: row.name,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<Row_get_groups> for Group {
    fn from(row: Row_get_groups) -> Self {
        Group {
            id: row.id,
            name: row.name,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<Row_get_group_membership> for GroupMembership {
    fn from(row: Row_get_group_membership) -> Self {
        GroupMembership {
            group_id: row.group_id,
            user_id: row.user_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<Row_get_group_members> for GroupMemberWithUser {
    fn from(row: Row_get_group_members) -> Self {
        GroupMemberWithUser {
            membership: GroupMembership {
                group_id: row.group_id,
                user_id: row.user_id.clone(),
                created_at: row.membership_created_at,
                updated_at: row.membership_updated_at,
            },
            user: User {
                id: row.user_id_fk,
                user_type: row.user_type,
                email: row.user_email,
                role: row.user_role,
                description: row.user_description,
                created_at: row.user_created_at,
                updated_at: row.user_updated_at,
            },
        }
    }
}

impl From<Row_get_user_groups> for UserGroupWithGroup {
    fn from(row: Row_get_user_groups) -> Self {
        UserGroupWithGroup {
            membership: GroupMembership {
                group_id: row.group_id.clone(),
                user_id: row.user_id,
                created_at: row.membership_created_at,
                updated_at: row.membership_updated_at,
            },
            group: Group {
                id: row.group_id_fk,
                name: row.group_name,
                created_at: row.group_created_at,
                updated_at: row.group_updated_at,
            },
        }
    }
}

// JWT signing key conversions

impl From<Row_get_jwt_signing_key_by_kid> for JwtSigningKey {
    fn from(row: Row_get_jwt_signing_key_by_kid) -> Self {
        JwtSigningKey {
            kid: row.kid,
            encrypted_private_key: row.encrypted_private_key,
            expires_at: row.expires_at,
            public_key: row.public_key,
            dek_alias: row.dek_alias,
            invalidated: row.invalidated,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<Row_get_jwt_signing_keys> for JwtSigningKey {
    fn from(row: Row_get_jwt_signing_keys) -> Self {
        JwtSigningKey {
            kid: row.kid,
            encrypted_private_key: row.encrypted_private_key,
            expires_at: row.expires_at,
            public_key: row.public_key,
            dek_alias: row.dek_alias,
            invalidated: row.invalidated,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

// STS configuration conversions

impl TryFrom<Row_get_sts_configuration_by_id> for StsConfigurationDb {
    type Error = CommonError;

    fn try_from(row: Row_get_sts_configuration_by_id) -> Result<Self, Self::Error> {
        let config = deserialize_sts_config(row.id, row.config_type, row.value)?;
        Ok(StsConfigurationDb {
            config,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<Row_get_sts_configurations> for StsConfigurationDb {
    type Error = CommonError;

    fn try_from(row: Row_get_sts_configurations) -> Result<Self, Self::Error> {
        let config = deserialize_sts_config(row.id, row.config_type, row.value)?;
        Ok(StsConfigurationDb {
            config,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

// User auth flow configuration conversions

/// Helper function to deserialize EncryptedUserAuthFlowConfig from database row
fn deserialize_user_auth_flow_config(
    config_type: String,
    config_json: WrappedJsonValue,
) -> Result<EncryptedUserAuthFlowConfig, CommonError> {
    match config_type.as_str() {
        "oidc_authorization_code_flow" => {
            let config = serde_json::from_value(config_json.into_inner()).map_err(|e| {
                CommonError::Repository {
                    msg: format!("Failed to deserialize oidc_authorization_code_flow config: {e}"),
                    source: Some(e.into()),
                }
            })?;
            Ok(EncryptedUserAuthFlowConfig::OidcAuthorizationCodeFlow(
                config,
            ))
        }
        "oauth_authorization_code_flow" => {
            let config = serde_json::from_value(config_json.into_inner()).map_err(|e| {
                CommonError::Repository {
                    msg: format!("Failed to deserialize oauth_authorization_code_flow config: {e}"),
                    source: Some(e.into()),
                }
            })?;
            Ok(EncryptedUserAuthFlowConfig::OauthAuthorizationCodeFlow(
                config,
            ))
        }
        "oidc_authorization_code_pkce_flow" => {
            let config = serde_json::from_value(config_json.into_inner()).map_err(|e| {
                CommonError::Repository {
                    msg: format!(
                        "Failed to deserialize oidc_authorization_code_pkce_flow config: {e}"
                    ),
                    source: Some(e.into()),
                }
            })?;
            Ok(EncryptedUserAuthFlowConfig::OidcAuthorizationCodePkceFlow(
                config,
            ))
        }
        "oauth_authorization_code_pkce_flow" => {
            let config = serde_json::from_value(config_json.into_inner()).map_err(|e| {
                CommonError::Repository {
                    msg: format!(
                        "Failed to deserialize oauth_authorization_code_pkce_flow config: {e}"
                    ),
                    source: Some(e.into()),
                }
            })?;
            Ok(EncryptedUserAuthFlowConfig::OauthAuthorizationCodePkceFlow(
                config,
            ))
        }
        _ => Err(CommonError::Repository {
            msg: format!("Unknown user auth flow config type: {config_type}"),
            source: None,
        }),
    }
}

impl TryFrom<Row_get_user_auth_flow_config_by_id> for UserAuthFlowConfigDb {
    type Error = CommonError;

    fn try_from(row: Row_get_user_auth_flow_config_by_id) -> Result<Self, Self::Error> {
        let config = deserialize_user_auth_flow_config(row.config_type, row.config)?;
        Ok(UserAuthFlowConfigDb {
            id: row.id,
            config,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<Row_get_user_auth_flow_configs> for UserAuthFlowConfigDb {
    type Error = CommonError;

    fn try_from(row: Row_get_user_auth_flow_configs) -> Result<Self, Self::Error> {
        let config = deserialize_user_auth_flow_config(row.config_type, row.config)?;
        Ok(UserAuthFlowConfigDb {
            id: row.id,
            config,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl EncryptedUserAuthFlowConfig {
    /// Convert the encrypted config to database values (config_type, config_json)
    pub fn to_db_values(&self) -> Result<(String, WrappedJsonValue), CommonError> {
        tracing::debug!(
            "Converting EncryptedUserAuthFlowConfig variant: {:?}",
            std::mem::discriminant(self)
        );
        let (config_type, config_value) = match self {
            EncryptedUserAuthFlowConfig::OidcAuthorizationCodeFlow(config) => {
                let json = serde_json::to_value(config).map_err(|e| CommonError::Repository {
                    msg: format!("Failed to serialize oidc_authorization_code_flow config: {e}"),
                    source: Some(e.into()),
                })?;
                ("oidc_authorization_code_flow".to_string(), json)
            }
            EncryptedUserAuthFlowConfig::OauthAuthorizationCodeFlow(config) => {
                let json = serde_json::to_value(config).map_err(|e| CommonError::Repository {
                    msg: format!("Failed to serialize oauth_authorization_code_flow config: {e}"),
                    source: Some(e.into()),
                })?;
                ("oauth_authorization_code_flow".to_string(), json)
            }
            EncryptedUserAuthFlowConfig::OidcAuthorizationCodePkceFlow(config) => {
                let json = serde_json::to_value(config).map_err(|e| CommonError::Repository {
                    msg: format!(
                        "Failed to serialize oidc_authorization_code_pkce_flow config: {e}"
                    ),
                    source: Some(e.into()),
                })?;
                ("oidc_authorization_code_pkce_flow".to_string(), json)
            }
            EncryptedUserAuthFlowConfig::OauthAuthorizationCodePkceFlow(config) => {
                let json = serde_json::to_value(config).map_err(|e| CommonError::Repository {
                    msg: format!(
                        "Failed to serialize oauth_authorization_code_pkce_flow config: {e}"
                    ),
                    source: Some(e.into()),
                })?;
                ("oauth_authorization_code_pkce_flow".to_string(), json)
            }
        };
        Ok((config_type, WrappedJsonValue::new(config_value)))
    }
}

// OAuth state conversions

impl From<Row_get_oauth_state_by_state> for OAuthState {
    fn from(row: Row_get_oauth_state_by_state) -> Self {
        OAuthState {
            state: row.state,
            config_id: row.config_id,
            code_verifier: row.code_verifier,
            nonce: row.nonce,
            redirect_uri: row.redirect_uri,
            created_at: row.created_at,
            expires_at: row.expires_at,
        }
    }
}
