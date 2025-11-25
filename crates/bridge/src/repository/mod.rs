mod sqlite;

use shared::{
    error::CommonError,
    primitives::{
        PaginatedResponse, PaginationRequest, WrappedChronoDateTime, WrappedJsonValue,
        WrappedUuidV4,
    },
};

#[allow(unused_imports)]
pub use sqlite::Repository;

use crate::logic::Metadata;
use crate::logic::credential::{
    BrokerState, ResourceServerCredentialSerialized, UserCredentialSerialized,
};
use crate::logic::instance::{
    FunctionInstanceSerialized, FunctionInstanceSerializedWithCredentials,
    ProviderInstanceSerialized, ProviderInstanceSerializedWithCredentials,
    ProviderInstanceSerializedWithFunctions,
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
    pub dek_alias: String,
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
            dek_alias: cred.dek_alias,
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
    pub dek_alias: String,
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
            dek_alias: cred.dek_alias,
        }
    }
}

// Repository parameter structs for provider instances
#[derive(Debug)]
pub struct CreateProviderInstance {
    pub id: String,
    pub display_name: String,
    pub resource_server_credential_id: WrappedUuidV4,
    pub user_credential_id: Option<WrappedUuidV4>,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
    pub provider_controller_type_id: String,
    pub credential_controller_type_id: String,
    pub status: String,
    pub return_on_successful_brokering: Option<crate::logic::ReturnAddress>,
}

impl From<ProviderInstanceSerialized> for CreateProviderInstance {
    fn from(pi: ProviderInstanceSerialized) -> Self {
        CreateProviderInstance {
            id: pi.id,
            display_name: pi.display_name,
            resource_server_credential_id: pi.resource_server_credential_id,
            user_credential_id: pi.user_credential_id,
            created_at: pi.created_at,
            updated_at: pi.updated_at,
            provider_controller_type_id: pi.provider_controller_type_id,
            credential_controller_type_id: pi.credential_controller_type_id,
            status: pi.status,
            return_on_successful_brokering: pi.return_on_successful_brokering,
        }
    }
}

// Repository parameter structs for function instances
#[derive(Debug)]
pub struct CreateFunctionInstance {
    pub function_controller_type_id: String,
    pub provider_controller_type_id: String,
    pub provider_instance_id: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

impl From<FunctionInstanceSerialized> for CreateFunctionInstance {
    fn from(fi: FunctionInstanceSerialized) -> Self {
        CreateFunctionInstance {
            function_controller_type_id: fi.function_controller_type_id,
            provider_controller_type_id: fi.provider_controller_type_id,
            provider_instance_id: fi.provider_instance_id,
            created_at: fi.created_at,
            updated_at: fi.updated_at,
        }
    }
}

// Repository parameter structs for broker state
#[derive(Debug)]
pub struct CreateBrokerState {
    pub id: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
    pub provider_instance_id: String,
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
            provider_instance_id: bs.provider_instance_id,
            provider_controller_type_id: bs.provider_controller_type_id,
            credential_controller_type_id: bs.credential_controller_type_id,
            metadata: bs.metadata,
            action: WrappedJsonValue::new(action_json),
        }
    }
}

// Repository return struct for grouped provider instances
#[derive(Debug)]
pub struct ProviderInstancesGroupedByFunctionControllerTypeId {
    pub function_controller_type_id: String,
    pub provider_instances: Vec<ProviderInstanceSerializedWithCredentials>,
}

// Repository trait
#[allow(async_fn_in_trait)]
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

    async fn delete_user_credential(&self, id: &WrappedUuidV4) -> Result<(), CommonError>;

    async fn delete_resource_server_credential(
        &self,
        id: &WrappedUuidV4,
    ) -> Result<(), CommonError>;

    async fn list_user_credentials(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<UserCredentialSerialized>, CommonError>;

    async fn list_resource_server_credentials(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<ResourceServerCredentialSerialized>, CommonError>;

    async fn create_provider_instance(
        &self,
        params: &CreateProviderInstance,
    ) -> Result<(), CommonError>;

    async fn get_provider_instance_by_id(
        &self,
        id: &str,
    ) -> Result<Option<ProviderInstanceSerializedWithFunctions>, CommonError>;

    async fn update_provider_instance(
        &self,
        id: &str,
        display_name: &str,
    ) -> Result<(), CommonError>;

    async fn update_provider_instance_after_brokering(
        &self,
        id: &str,
        user_credential_id: &WrappedUuidV4,
    ) -> Result<(), CommonError>;

    async fn delete_provider_instance(&self, id: &str) -> Result<(), CommonError>;

    async fn create_function_instance(
        &self,
        params: &CreateFunctionInstance,
    ) -> Result<(), CommonError>;

    async fn get_function_instance_by_id(
        &self,
        function_controller_type_id: &str,
        provider_controller_type_id: &str,
        provider_instance_id: &str,
    ) -> Result<Option<FunctionInstanceSerialized>, CommonError>;

    async fn delete_function_instance(
        &self,
        function_controller_type_id: &str,
        provider_controller_type_id: &str,
        provider_instance_id: &str,
    ) -> Result<(), CommonError>;

    async fn get_function_instance_with_credentials(
        &self,
        function_controller_type_id: &str,
        provider_controller_type_id: &str,
        provider_instance_id: &str,
    ) -> Result<Option<FunctionInstanceSerializedWithCredentials>, CommonError>;

    async fn create_broker_state(&self, params: &CreateBrokerState) -> Result<(), CommonError>;

    async fn get_broker_state_by_id(&self, id: &str) -> Result<Option<BrokerState>, CommonError>;

    async fn delete_broker_state(&self, id: &str) -> Result<(), CommonError>;

    async fn list_provider_instances(
        &self,
        pagination: &PaginationRequest,
        status: Option<&str>,
        provider_controller_type_id: Option<&str>,
    ) -> Result<PaginatedResponse<ProviderInstanceSerializedWithFunctions>, CommonError>;

    async fn list_function_instances(
        &self,
        pagination: &PaginationRequest,
        provider_instance_id: Option<&str>,
    ) -> Result<PaginatedResponse<FunctionInstanceSerialized>, CommonError>;

    async fn get_provider_instances_grouped_by_function_controller_type_id(
        &self,
        function_controller_type_ids: &[String],
    ) -> Result<Vec<ProviderInstancesGroupedByFunctionControllerTypeId>, CommonError>;

    async fn update_resource_server_credential(
        &self,
        id: &WrappedUuidV4,
        value: Option<&WrappedJsonValue>,
        metadata: Option<&crate::logic::Metadata>,
        next_rotation_time: Option<&WrappedChronoDateTime>,
        updated_at: Option<&WrappedChronoDateTime>,
    ) -> Result<(), CommonError>;

    async fn update_user_credential(
        &self,
        id: &WrappedUuidV4,
        value: Option<&WrappedJsonValue>,
        metadata: Option<&crate::logic::Metadata>,
        next_rotation_time: Option<&WrappedChronoDateTime>,
        updated_at: Option<&WrappedChronoDateTime>,
    ) -> Result<(), CommonError>;

    async fn list_provider_instances_with_credentials(
        &self,
        pagination: &PaginationRequest,
        status: Option<&str>,
        rotation_window_end: Option<&WrappedChronoDateTime>,
    ) -> Result<PaginatedResponse<ProviderInstanceSerializedWithCredentials>, CommonError>;
}
