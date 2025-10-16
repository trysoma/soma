mod sqlite;

use shared::{
    error::CommonError,
    primitives::{WrappedChronoDateTime, WrappedJsonValue, WrappedUuidV4},
};

#[allow(unused_imports)]
pub use sqlite::Repository;

use crate::logic::{
    Metadata, ResourceServerCredential, ResourceServerCredentialType,
    ResourceServerCredentialVariant, UserCredential, UserCredentialType, UserCredentialVariant,
};

// Repository parameter structs for resource server credentials
#[derive(Debug)]
pub struct CreateResourceServerCredential {
    pub id: WrappedUuidV4,
    pub credential_type: ResourceServerCredentialType,
    pub credential_data: WrappedJsonValue,
    pub metadata: WrappedJsonValue,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
    pub run_refresh_before: Option<WrappedChronoDateTime>,
}

impl TryFrom<ResourceServerCredential> for CreateResourceServerCredential {
    type Error = CommonError;
    fn try_from(cred: ResourceServerCredential) -> Result<Self, Self::Error> {
        let credential_type = match &cred.inner {
            ResourceServerCredentialVariant::NoAuth(_) => ResourceServerCredentialType::NoAuth,
            ResourceServerCredentialVariant::Oauth2AuthorizationCodeFlow(_) => {
                ResourceServerCredentialType::Oauth2AuthorizationCodeFlow
            }
            ResourceServerCredentialVariant::Oauth2JwtBearerAssertionFlow(_) => {
                ResourceServerCredentialType::Oauth2JwtBearerAssertionFlow
            }
            ResourceServerCredentialVariant::Custom(_) => ResourceServerCredentialType::Custom,
        };
        let credential_data = WrappedJsonValue::new(serde_json::to_value(&cred.inner)?);
        let metadata = WrappedJsonValue::new(serde_json::to_value(&cred.metadata)?);
        Ok(CreateResourceServerCredential {
            id: cred.id,
            credential_type,
            credential_data,
            metadata,
            created_at: cred.created_at,
            updated_at: cred.updated_at,
            run_refresh_before: cred.run_refresh_before,
        })
    }
}

// Repository parameter structs for user credentials
#[derive(Debug)]
pub struct CreateUserCredential {
    pub id: WrappedUuidV4,
    pub credential_type: UserCredentialType,
    pub credential_data: WrappedJsonValue,
    pub metadata: WrappedJsonValue,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
    pub run_refresh_before: Option<WrappedChronoDateTime>,
}

impl TryFrom<UserCredential> for CreateUserCredential {
    type Error = CommonError;
    fn try_from(cred: UserCredential) -> Result<Self, Self::Error> {
        let credential_type = match &cred.inner {
            UserCredentialVariant::NoAuth(_) => UserCredentialType::NoAuth,
            UserCredentialVariant::Oauth2AuthorizationCodeFlow(_) => {
                UserCredentialType::Oauth2AuthorizationCodeFlow
            }
            UserCredentialVariant::Oauth2JwtBearerAssertionFlow(_) => {
                UserCredentialType::Oauth2JwtBearerAssertionFlow
            }
            UserCredentialVariant::Custom(_) => UserCredentialType::Custom,
        };
        let credential_data = WrappedJsonValue::new(serde_json::to_value(&cred.inner)?);
        let metadata = WrappedJsonValue::new(serde_json::to_value(&cred.metadata)?);
        Ok(CreateUserCredential {
            id: cred.id,
            credential_type,
            credential_data,
            metadata,
            created_at: cred.created_at,
            updated_at: cred.updated_at,
            run_refresh_before: cred.run_refresh_before,
        })
    }
}

// Repository parameter structs for provider instances
#[derive(Debug)]
pub struct CreateProviderInstance {
    pub id: String,
    pub provider_id: String,
    pub resource_server_credential_id: WrappedUuidV4,
    pub user_credential_id: WrappedUuidV4,
}

// Repository parameter structs for function instances
#[derive(Debug)]
pub struct CreateFunctionInstance {
    pub id: String,
    pub function_id: String,
    pub provider_instance_id: String,
}

// Repository parameter structs for credential exchange state
#[derive(Debug)]
pub struct CreateCredentialExchangeState {
    pub id: String,
    pub state: Metadata,
}

// Repository return struct for credential exchange state
#[derive(Debug)]
pub struct CredentialExchangeState {
    pub id: String,
    pub state: Metadata,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

// Repository trait
pub trait ProviderRepositoryLike {
    async fn create_resource_server_credential(
        &self,
        params: &CreateResourceServerCredential,
    ) -> Result<(), CommonError>;
    async fn create_user_credential(
        &self,
        params: &CreateUserCredential,
    ) -> Result<(), CommonError>;
    async fn create_provider_instance(
        &self,
        params: &CreateProviderInstance,
    ) -> Result<(), CommonError>;
    async fn create_function_instance(
        &self,
        params: &CreateFunctionInstance,
    ) -> Result<(), CommonError>;
    async fn create_credential_exchange_state(
        &self,
        params: &CreateCredentialExchangeState,
    ) -> Result<(), CommonError>;
    async fn get_credential_exchange_state_by_id(
        &self,
        id: &str,
    ) -> Result<Option<CredentialExchangeState>, CommonError>;
}
