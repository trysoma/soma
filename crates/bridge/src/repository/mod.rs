mod sqlite;

use shared::{
    error::CommonError,
    primitives::{WrappedChronoDateTime, WrappedJsonValue, WrappedUuidV4},
};

#[allow(unused_imports)]
pub use sqlite::Repository;

use crate::logic::{
    BrokerState, DataEncryptionKey, EncryptedDataKey, EnvelopeEncryptionKeyId, Metadata,
    ResourceServerCredentialSerialized, UserCredentialSerialized,
    ProviderInstanceSerialized, FunctionInstanceSerialized, FunctionInstanceSerializedWithCredentials,
};

// Repository parameter structs for resource server credentials
#[derive(Debug)]
pub struct CreateResourceServerCredential {
    pub id: WrappedUuidV4,
    pub type_id: String,
    pub metadata: Metadata,
    pub value: WrappedJsonValue,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
    pub next_rotation_time: Option<WrappedChronoDateTime>,
}

impl From<ResourceServerCredentialSerialized> for CreateResourceServerCredential {
    fn from(cred: ResourceServerCredentialSerialized) -> Self {
        CreateResourceServerCredential {
            id: cred.id,
            type_id: cred.type_id,
            metadata: cred.metadata,
            value: cred.value,
            created_at: cred.created_at,
            updated_at: cred.updated_at,
            next_rotation_time: cred.next_rotation_time,
        }
    }
}

// Repository parameter structs for user credentials
#[derive(Debug)]
pub struct CreateUserCredential {
    pub id: WrappedUuidV4,
    pub type_id: String,
    pub metadata: Metadata,
    pub value: WrappedJsonValue,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
    pub next_rotation_time: Option<WrappedChronoDateTime>,
}

impl From<UserCredentialSerialized> for CreateUserCredential {
    fn from(cred: UserCredentialSerialized) -> Self {
        CreateUserCredential {
            id: cred.id,
            type_id: cred.type_id,
            metadata: cred.metadata,
            value: cred.value,
            created_at: cred.created_at,
            updated_at: cred.updated_at,
            next_rotation_time: cred.next_rotation_time,
        }
    }
}

// Repository parameter structs for provider instances
#[derive(Debug)]
pub struct CreateProviderInstance {
    pub id: String,
    pub resource_server_credential_id: WrappedUuidV4,
    pub user_credential_id: WrappedUuidV4,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
    pub provider_controller_type_id: String,
    pub credential_controller_type_id: String,
}

impl From<ProviderInstanceSerialized> for CreateProviderInstance {
    fn from(pi: ProviderInstanceSerialized) -> Self {
        CreateProviderInstance {
            id: pi.id,
            resource_server_credential_id: pi.resource_server_credential_id,
            user_credential_id: pi.user_credential_id,
            created_at: pi.created_at,
            updated_at: pi.updated_at,
            provider_controller_type_id: pi.provider_controller_type_id,
            credential_controller_type_id: pi.credential_controller_type_id,
        }
    }
}

// Repository parameter structs for function instances
#[derive(Debug)]
pub struct CreateFunctionInstance {
    pub id: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
    pub provider_instance_id: String,
    pub function_controller_type_id: String,
}

impl From<FunctionInstanceSerialized> for CreateFunctionInstance {
    fn from(fi: FunctionInstanceSerialized) -> Self {
        CreateFunctionInstance {
            id: fi.id,
            created_at: fi.created_at,
            updated_at: fi.updated_at,
            provider_instance_id: fi.provider_instance_id,
            function_controller_type_id: fi.function_controller_type_id,
        }
    }
}

// Repository parameter structs for broker state
#[derive(Debug)]
pub struct CreateBrokerState {
    pub id: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
    pub resource_server_cred_id: WrappedUuidV4,
    pub provider_controller_type_id: String,
    pub credential_controller_type_id: String,
    pub metadata: Metadata,
    pub action: WrappedJsonValue,
}

impl From<BrokerState> for CreateBrokerState {
    fn from(bs: BrokerState) -> Self {
        let action_json = serde_json::to_value(&bs.action).unwrap_or(serde_json::json!({}));
        CreateBrokerState {
            id: bs.id,
            created_at: bs.created_at,
            updated_at: bs.updated_at,
            resource_server_cred_id: bs.resource_server_cred_id,
            provider_controller_type_id: bs.provider_controller_type_id,
            credential_controller_type_id: bs.credential_controller_type_id,
            metadata: bs.metadata,
            action: WrappedJsonValue::new(action_json),
        }
    }
}

// Repository parameter structs for data encryption key
#[derive(Debug)]
pub struct CreateDataEncryptionKey {
    pub id: String,
    pub envelope_encryption_key_id: EnvelopeEncryptionKeyId,
    pub encryption_key: EncryptedDataKey,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

impl From<DataEncryptionKey> for CreateDataEncryptionKey {
    fn from(dek: DataEncryptionKey) -> Self {
        CreateDataEncryptionKey {
            id: dek.id,
            envelope_encryption_key_id: dek.envelope_encryption_key_id,
            encryption_key: dek.encryption_key,
            created_at: dek.created_at,
            updated_at: dek.updated_at,
        }
    }
}

// Repository trait
pub trait ProviderRepositoryLike {
    async fn create_resource_server_credential(
        &self,
        params: &CreateResourceServerCredential,
    ) -> Result<(), CommonError>;

    async fn get_resource_server_credential_by_id(
        &self,
        id: &WrappedUuidV4,
    ) -> Result<Option<ResourceServerCredentialSerialized>, CommonError>;

    async fn create_user_credential(
        &self,
        params: &CreateUserCredential,
    ) -> Result<(), CommonError>;

    async fn get_user_credential_by_id(
        &self,
        id: &WrappedUuidV4,
    ) -> Result<Option<UserCredentialSerialized>, CommonError>;

    async fn create_provider_instance(
        &self,
        params: &CreateProviderInstance,
    ) -> Result<(), CommonError>;

    async fn get_provider_instance_by_id(
        &self,
        id: &str,
    ) -> Result<Option<ProviderInstanceSerialized>, CommonError>;

    async fn create_function_instance(
        &self,
        params: &CreateFunctionInstance,
    ) -> Result<(), CommonError>;

    async fn get_function_instance_by_id(
        &self,
        id: &str,
    ) -> Result<Option<FunctionInstanceSerialized>, CommonError>;

    async fn delete_function_instance(
        &self,
        id: &str,
    ) -> Result<(), CommonError>;

    async fn get_function_instance_with_credentials(
        &self,
        id: &str,
    ) -> Result<Option<FunctionInstanceSerializedWithCredentials>, CommonError>;

    async fn create_broker_state(
        &self,
        params: &CreateBrokerState,
    ) -> Result<(), CommonError>;

    async fn get_broker_state_by_id(
        &self,
        id: &str,
    ) -> Result<Option<BrokerState>, CommonError>;

    async fn delete_broker_state(
        &self,
        id: &str,
    ) -> Result<(), CommonError>;

    async fn create_data_encryption_key(
        &self,
        params: &CreateDataEncryptionKey,
    ) -> Result<(), CommonError>;

    async fn get_data_encryption_key_by_id(
        &self,
        id: &str,
    ) -> Result<Option<DataEncryptionKey>, CommonError>;
}
