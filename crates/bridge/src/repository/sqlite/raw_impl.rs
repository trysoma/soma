use crate::logic::{
    BrokerAction, BrokerState, FunctionInstanceSerialized, ProviderInstanceSerialized,
    ResourceServerCredentialSerialized, UserCredentialSerialized, FunctionInstanceSerializedWithCredentials,
};
use shared::error::CommonError;

// Import generated Row types from parent module
use super::{
    Row_get_resource_server_credential_by_id,
    Row_get_user_credential_by_id,
    Row_get_provider_instance_by_id,
    Row_get_function_instance_by_id,
    Row_get_broker_state_by_id,
    Row_get_function_instance_with_credentials,
    Row_get_data_encryption_key_by_id,
};

// Implement TryFrom for query result types to domain types

impl TryFrom<Row_get_resource_server_credential_by_id> for ResourceServerCredentialSerialized {
    type Error = CommonError;
    fn try_from(row: Row_get_resource_server_credential_by_id) -> Result<Self, Self::Error> {
        Ok(ResourceServerCredentialSerialized {
            id: row.id,
            type_id: row.type_id,
            metadata: row.metadata,
            value: row.value,
            created_at: row.created_at,
            updated_at: row.updated_at,
            next_rotation_time: row.next_rotation_time,
        })
    }
}

impl TryFrom<Row_get_user_credential_by_id> for UserCredentialSerialized {
    type Error = CommonError;
    fn try_from(row: Row_get_user_credential_by_id) -> Result<Self, Self::Error> {
        Ok(UserCredentialSerialized {
            id: row.id,
            type_id: row.type_id,
            metadata: row.metadata,
            value: row.value,
            created_at: row.created_at,
            updated_at: row.updated_at,
            next_rotation_time: row.next_rotation_time,
        })
    }
}

impl TryFrom<Row_get_provider_instance_by_id> for ProviderInstanceSerialized {
    type Error = CommonError;
    fn try_from(row: Row_get_provider_instance_by_id) -> Result<Self, Self::Error> {
        Ok(ProviderInstanceSerialized {
            id: row.id,
            resource_server_credential_id: row.resource_server_credential_id,
            user_credential_id: row.user_credential_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
            provider_controller_type_id: row.provider_controller_type_id,
            credential_controller_type_id: row.credential_controller_type_id,
        })
    }
}

impl TryFrom<Row_get_function_instance_by_id> for FunctionInstanceSerialized {
    type Error = CommonError;
    fn try_from(row: Row_get_function_instance_by_id) -> Result<Self, Self::Error> {
        Ok(FunctionInstanceSerialized {
            id: row.id,
            created_at: row.created_at,
            updated_at: row.updated_at,
            provider_instance_id: row.provider_instance_id,
            function_controller_type_id: row.function_controller_type_id,
        })
    }
}

impl TryFrom<Row_get_broker_state_by_id> for BrokerState {
    type Error = CommonError;
    fn try_from(row: Row_get_broker_state_by_id) -> Result<Self, Self::Error> {
        let action: BrokerAction = serde_json::from_value(row.action.get_inner().clone())?;
        Ok(BrokerState {
            id: row.id,
            created_at: row.created_at,
            updated_at: row.updated_at,
            resource_server_cred_id: row.resource_server_cred_id,
            provider_controller_type_id: row.provider_controller_type_id,
            credential_controller_type_id: row.credential_controller_type_id,
            metadata: row.metadata,
            action,
        })
    }
}

impl TryFrom<Row_get_function_instance_with_credentials> for FunctionInstanceSerializedWithCredentials {
    type Error = CommonError;
    fn try_from(row: Row_get_function_instance_with_credentials) -> Result<Self, Self::Error> {
        Ok(FunctionInstanceSerializedWithCredentials {
            function_instance: FunctionInstanceSerialized {
                id: row.function_instance_id,
                created_at: row.function_instance_created_at,
                updated_at: row.function_instance_updated_at,
                provider_instance_id: row.function_instance_provider_instance_id.clone(),
                function_controller_type_id: row.function_controller_type_id,
            },
            provider_instance: ProviderInstanceSerialized {
                id: row.provider_instance_id,
                resource_server_credential_id: row.provider_instance_resource_server_credential_id.clone(),
                user_credential_id: row.provider_instance_user_credential_id.clone(),
                created_at: row.provider_instance_created_at,
                updated_at: row.provider_instance_updated_at,
                provider_controller_type_id: row.provider_controller_type_id,
                credential_controller_type_id: row.credential_controller_type_id,
            },
            resource_server_credential: ResourceServerCredentialSerialized {
                id: row.resource_server_credential_id,
                type_id: row.resource_server_credential_type_id,
                metadata: row.resource_server_credential_metadata,
                value: row.resource_server_credential_value,
                created_at: row.resource_server_credential_created_at,
                updated_at: row.resource_server_credential_updated_at,
                next_rotation_time: row.resource_server_credential_next_rotation_time,
            },
            user_credential: UserCredentialSerialized {
                id: row.user_credential_id,
                type_id: row.user_credential_type_id,
                metadata: row.user_credential_metadata,
                value: row.user_credential_value,
                created_at: row.user_credential_created_at,
                updated_at: row.user_credential_updated_at,
                next_rotation_time: row.user_credential_next_rotation_time,
            },
            static_credential: crate::logic::StaticCredentialSerialized {
                // Static credentials are not stored in the database, they're derived from the provider controller
                // This will need to be populated by the repository implementation
                id: "static".to_string(),
                type_id: "static_no_auth".to_string(),
                metadata: crate::logic::Metadata::new(),
                value: shared::primitives::WrappedJsonValue::new(serde_json::json!({})),
                created_at: row.function_instance_created_at,
                updated_at: row.function_instance_updated_at,
            },
        })
    }
}

impl TryFrom<Row_get_data_encryption_key_by_id> for crate::logic::DataEncryptionKey {
    type Error = CommonError;
    fn try_from(row: Row_get_data_encryption_key_by_id) -> Result<Self, Self::Error> {
        Ok(crate::logic::DataEncryptionKey {
            id: row.id,
            envelope_encryption_key_id: row.envelope_encryption_key_id,
            encryption_key: row.encryption_key,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
