use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use enum_dispatch::enum_dispatch;
use once_cell::sync::Lazy;
use reqwest::Request;
use schemars::{JsonSchema, Schema};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::json;
use shared::{
    error::CommonError,
    primitives::{
        PaginatedResponse, PaginationRequest, WrappedChronoDateTime, WrappedJsonValue,
        WrappedSchema, WrappedUuidV4,
    },
};
use std::sync::RwLock;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, ToSchema)]
#[serde(transparent)]
pub struct Metadata(pub serde_json::Map<String, serde_json::Value>);

impl Metadata {
    pub fn new() -> Self {
        Self(serde_json::Map::new())
    }
}


impl TryInto<libsql::Value> for Metadata {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    fn try_into(self) -> Result<libsql::Value, Self::Error> {
        let json_value = serde_json::Value::Object(self.0);
        let json_string = serde_json::to_string(&json_value)?;
        Ok(libsql::Value::Text(json_string))
    }
}

impl TryFrom<libsql::Value> for Metadata {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    fn try_from(value: libsql::Value) -> Result<Self, Self::Error> {
        match value {
            libsql::Value::Text(s) => {
                let json_value: serde_json::Value = serde_json::from_str(&s)?;
                match json_value {
                    serde_json::Value::Object(map) => Ok(Metadata(map)),
                    _ => Err("Expected JSON object for Metadata".into()),
                }
            }
            _ => Err("Expected Text value for Metadata".into()),
        }
    }
}

impl libsql::FromValue for Metadata {
    fn from_sql(val: libsql::Value) -> libsql::Result<Self>
    where
        Self: Sized,
    {
        match val {
            libsql::Value::Text(s) => {
                let json_value: serde_json::Value =
                    serde_json::from_str(&s).map_err(|_e| libsql::Error::InvalidColumnType)?;
                match json_value {
                    serde_json::Value::Object(map) => Ok(Metadata(map)),
                    _ => Err(libsql::Error::InvalidColumnType),
                }
            }
            libsql::Value::Null => Err(libsql::Error::NullValue),
            _ => Err(libsql::Error::InvalidColumnType),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Credential<T> {
    pub inner: T,
    pub metadata: Metadata,
    pub id: WrappedUuidV4,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

pub trait RotateableCredentialLike {
    fn next_rotation_time(&self) -> WrappedChronoDateTime;
}

// Static credential configurations

// #[derive(Serialize, Deserialize, Clone)]
// #[serde(tag = "type")]
// pub enum StaticCredentialConfigurationVariant {
//     NoAuth(NoAuthStaticCredentialConfiguration),
//     Oauth2AuthorizationCodeFlow(Oauth2AuthorizationCodeFlowStaticCredentialConfiguration),
//     Oauth2JwtBearerAssertionFlow(Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration),
//     Custom(CustomStaticCredentialConfiguration),
// }

// #[derive(Serialize, Deserialize, Clone)]
// #[serde(rename_all = "snake_case")]
// pub enum StaticCredentialConfigurationType {
//     NoAuth,
//     Oauth2AuthorizationCodeFlow,
//     Oauth2JwtBearerAssertionFlow,
//     Custom,
// }

#[derive(Serialize, Deserialize, Clone, JsonSchema, ToSchema)]
#[serde(transparent)]
pub struct EncryptedString(pub String);

pub trait StaticCredentialConfigurationLike {
    fn type_id(&self) -> &'static str;
    fn value(&self) -> WrappedJsonValue;
    fn as_rotateable_credential(&self) -> Option<&dyn RotateableCredentialLike> {
        None
    }
}

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct NoAuthStaticCredentialConfiguration {
    pub metadata: Metadata,
}


impl StaticCredentialConfigurationLike for NoAuthStaticCredentialConfiguration {
    fn type_id(&self) -> &'static str {
        "static_no_auth"
    }
    fn value(&self) -> WrappedJsonValue {
        WrappedJsonValue::new(serde_json::Value::Object(serde_json::Map::new()))
    }
}

pub type StaticCredential = Credential<Arc<dyn StaticCredentialConfigurationLike>>;

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct Oauth2AuthorizationCodeFlowStaticCredentialConfiguration {
    pub auth_uri: String,
    pub token_uri: String,
    pub userinfo_uri: String,
    pub jwks_uri: String,
    pub issuer: String,
    pub scopes: Vec<String>,
    pub metadata: Metadata,
}

impl StaticCredentialConfigurationLike for Oauth2AuthorizationCodeFlowStaticCredentialConfiguration {
    fn type_id(&self) -> &'static str {
        "static_oauth2_authorization_code_flow"
    }
    fn value(&self) -> WrappedJsonValue {
        WrappedJsonValue::new(json!(self))
    }
}

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration {
    pub auth_uri: String,
    pub token_uri: String,
    pub userinfo_uri: String,
    pub jwks_uri: String,
    pub issuer: String,
    pub scopes: Vec<String>,
    pub metadata: Metadata,
}

impl StaticCredentialConfigurationLike for Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration {
    fn type_id(&self) -> &'static str {
        "static_oauth2_jwt_bearer_assertion_flow"
    }
    fn value(&self) -> WrappedJsonValue {
        WrappedJsonValue::new(json!(self))
    }
}


// Resource server credentials

pub trait ResourceServerCredentialLike {
    fn type_id(&self) -> &'static str;
    fn value(&self) -> WrappedJsonValue;
    fn as_rotateable_credential(&self) -> Option<&dyn RotateableCredentialLike> {
        None
    }

}

pub type ResourceServerCredential = Credential<Arc<dyn ResourceServerCredentialLike>>;

#[derive(Serialize, Deserialize, Clone)]
pub struct NoAuthResourceServerCredential {
    pub metadata: Metadata,
}

impl ResourceServerCredentialLike for NoAuthResourceServerCredential {
    fn type_id(&self) -> &'static str {
        "resource_server_no_auth"
    }
    fn value(&self) -> WrappedJsonValue {
        WrappedJsonValue::new(json!(self))
    }
}

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct Oauth2AuthorizationCodeFlowResourceServerCredential {
    pub client_id: String,
    pub client_secret: EncryptedString,
    pub redirect_uri: String,
    pub metadata: Metadata,
}

impl ResourceServerCredentialLike for Oauth2AuthorizationCodeFlowResourceServerCredential {
    fn type_id(&self) -> &'static str {
        "resource_server_oauth2_authorization_code_flow"
    }
    fn value(&self) -> WrappedJsonValue {
        WrappedJsonValue::new(json!(self))
    }
}

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct Oauth2JwtBearerAssertionFlowResourceServerCredential {
    pub client_id: String,
    pub client_secret: EncryptedString,
    pub redirect_uri: String,
    pub metadata: Metadata,
}


impl ResourceServerCredentialLike for Oauth2JwtBearerAssertionFlowResourceServerCredential {
    fn type_id(&self) -> &'static str {
        "resource_server_oauth2_jwt_bearer_assertion_flow"
    }
    fn value(&self) -> WrappedJsonValue {
        WrappedJsonValue::new(json!(self))
    }
}

// user credentials

pub trait UserCredentialLike {
    fn type_id(&self) -> &'static str;
    fn value(&self) -> WrappedJsonValue;
    fn as_rotateable_credential(&self) -> Option<&dyn RotateableCredentialLike> {
        None
    }
}

pub type UserCredential = Credential<Arc<dyn UserCredentialLike>>;

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct Oauth2AuthorizationCodeFlowUserCredential {
    pub code: EncryptedString,
    pub access_token: EncryptedString,
    pub refresh_token: EncryptedString,
    pub expiry_time: WrappedChronoDateTime,
    pub sub: String,
    pub metadata: Metadata,
}

impl UserCredentialLike for Oauth2AuthorizationCodeFlowUserCredential {
    fn type_id(&self) -> &'static str {
        "oauth2_authorization_code_flow"
    }
    fn value(&self) -> WrappedJsonValue {
        WrappedJsonValue::new(json!(self))
    }
}


impl RotateableCredentialLike for Oauth2AuthorizationCodeFlowUserCredential {
    fn next_rotation_time(&self) -> WrappedChronoDateTime {
        self.expiry_time
    }
}

// User credentials
#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct NoAuthUserCredential {
    pub metadata: Metadata,
}

impl UserCredentialLike for NoAuthUserCredential {
    fn type_id(&self) -> &'static str {
        "no_auth"
    }
    fn value(&self) -> WrappedJsonValue {
        WrappedJsonValue::new(json!(self))
    }
}

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct Oauth2JwtBearerAssertionFlowUserCredential {
    pub assertion: String,
    pub token: String,
    pub expiry_time: WrappedChronoDateTime,
    pub sub: String,
    pub metadata: Metadata,
}

impl UserCredentialLike for Oauth2JwtBearerAssertionFlowUserCredential {
    fn type_id(&self) -> &'static str {
        "oauth2_jwt_bearer_assertion_flow"
    }
    fn value(&self) -> WrappedJsonValue {
        WrappedJsonValue::new(json!(self))
    }
}

impl RotateableCredentialLike for Oauth2JwtBearerAssertionFlowUserCredential {
    fn next_rotation_time(&self) -> WrappedChronoDateTime {
        self.expiry_time
    }
}

// Brokering user credentials

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct BrokerState {
    pub id: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
    pub resource_server_cred_id: WrappedUuidV4,
    pub provider_controller_type_id: String,
    pub credential_controller_type_id: String,
    pub metadata: Metadata,
    pub action: BrokerAction,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub enum BrokerAction {
    Redirect { url: String },
    None,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BrokerInput {
    Oauth2AuthorizationCodeFlow { code: String },
    Oauth2AuthorizationCodeFlowWithPkce { code: String, code_verifier: String },
}

pub enum BrokerOutcome {
    Success {
        user_credential: Box<dyn UserCredentialLike>,
        metadata: Metadata,
    },
    Continue {
        metadata: Metadata,
    },
}


#[async_trait]
pub trait UserCredentialBrokerLike: Send + Sync {
    /// Called when the user (or system) initiates credential brokering.
    async fn start(
        &self,
        resource_server_cred: &Credential<Box<dyn ResourceServerCredentialLike>>,
    ) -> Result<(BrokerAction, BrokerOutcome), CommonError>;

    /// Called after an external event (callback, redirect, etc.) returns data to the platform.
    async fn resume(
        &self,
        state: &BrokerState,
        input: BrokerInput,
    ) -> Result<(BrokerAction, BrokerOutcome), CommonError>;
}



#[derive(Serialize, Deserialize, Clone, ToSchema, JsonSchema)]
pub struct ConfigurationSchemaItem {
    pub resource_server: WrappedSchema,
    pub user_credential: WrappedSchema,
}

#[derive(Serialize, Deserialize, Clone, ToSchema, JsonSchema)]
#[serde(transparent)]
pub struct ConfigurationSchema(pub HashMap<String, ConfigurationSchemaItem>);

#[async_trait]
pub trait ProviderCredentialControllerLike: Send + Sync {
    fn type_id(&self) -> &'static str;
    fn documentation(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn configuration_schema(&self) -> ConfigurationSchema;
    fn as_rotateable_controller_resource_server_credential(&self) -> Option<&dyn RotateableControllerResourceServerCredentialLike> {
        None
    }
    fn as_rotateable_controller_user_credential(&self) -> Option<&dyn RotateableControllerUserCredentialLike> {
        None
    }
    fn as_user_credential_broker(&self) -> Option<&dyn UserCredentialBrokerLike> {
        None
    }
     // TODO: need to pass in the encryption provider here to do the actual encryption 
     async fn encrypt_resource_server_configuration(&self, raw_resource_server_configuration: WrappedJsonValue) -> Result<Box<dyn ResourceServerCredentialLike>, CommonError>;
     async fn encrypt_user_credential_configuration(&self, raw_user_credential_configuration: WrappedJsonValue) -> Result<Box<dyn UserCredentialLike>, CommonError>;
 
     // NOTE: serialized values are always already encrypted
     fn from_serialized_resource_server_configuration(&self, raw_resource_server_configuration: WrappedJsonValue) -> Result<(Box<dyn ResourceServerCredentialLike>, Metadata), CommonError>;
     fn from_serialized_user_credential_configuration(&self, raw_user_credential_configuration: WrappedJsonValue) -> Result<(Box<dyn UserCredentialLike>, Metadata), CommonError>;
}

#[async_trait]
pub trait ProviderControllerLike: Send + Sync {
    // async fn save_resource_server_credential(
    //     &self,
    //     input: Arc<dyn ResourceServerCredentialLike>,
    // ) -> Result<ResourceServerCredential, CommonError>;
    // async fn save_user_credential(
    //     &self,
    //     input: UserCredentialVariant,
    // ) -> Result<UserCredential, CommonError>;
    // async fn get_static_credentials(
    //     &self,
    //     variant: StaticCredentialConfigurationType,
    // ) -> Result<StaticCredentialConfiguration, CommonError>;
    fn type_id(&self) -> &'static str;
    fn documentation(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn functions(&self) -> Vec<Arc<dyn FunctionControllerLike>>;
    fn credential_controllers(&self) -> Vec<Arc<dyn ProviderCredentialControllerLike>>;
 
    async fn function_call(
        &self,
        instance: Arc<dyn ProviderInstanceLike>,
        function_type_id: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, CommonError>;
}


pub trait ProviderInstanceLike {
    fn provider_controller_type_id(&self) -> &'static str;
    fn type_id(&self) -> &'static str;
    fn credential_controller_type_id(&self) -> &'static str;

    fn static_credential_value(&self) -> WrappedJsonValue;
    fn resource_server_credential_value(&self) -> WrappedJsonValue;
    fn user_credential_value(&self) -> WrappedJsonValue;
}


#[async_trait]
pub trait RotateableControllerResourceServerCredentialLike {
    async fn rotate_resource_server_credential(&self, resource_server_cred: &Credential<Arc<dyn ResourceServerCredentialLike>>) -> Result<Credential<Arc<dyn ResourceServerCredentialLike>>, CommonError>;
    fn next_resource_server_credential_rotation_time(&self, resource_server_cred: &Credential<Arc<dyn ResourceServerCredentialLike>>) -> WrappedChronoDateTime;
}

#[async_trait]
pub trait RotateableControllerUserCredentialLike {
    async fn rotate_user_credential(&self, resource_server_cred: &ResourceServerCredential, user_cred: &Credential<Arc<dyn UserCredentialLike>>) -> Result<Credential<Arc<dyn UserCredentialLike>>, CommonError>;
    async fn next_user_credential_rotation_time(&self, resource_server_cred: &ResourceServerCredential, user_cred: &Credential<Arc<dyn UserCredentialLike>>) -> WrappedChronoDateTime;
}

pub trait FunctionControllerLike {
    // type ProviderInstance;
    // type Params;
    // type Output;
    // async fn handle(provider_instance: Self::ProviderInstance, params: Self::Params) -> Result<Self::Output, CommonError>;
    fn type_id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn documentation(&self) -> &'static str;
    fn parameters(&self) -> WrappedSchema;
    fn output(&self) -> WrappedSchema;
}


// serialized versions of the above types

#[derive(Serialize, Deserialize, ToSchema)]
pub struct StaticCredentialSerialized {
    // not UUID as some ID's will be deterministic
    pub id: String,
    pub type_id: String,
    pub metadata: Metadata,

    // this is the serialized version of the actual configuration fields
    pub value: WrappedJsonValue,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

impl From<Credential<Arc<dyn StaticCredentialConfigurationLike>>> for StaticCredentialSerialized {
    fn from(static_cred: Credential<Arc<dyn StaticCredentialConfigurationLike>>) -> Self {
        StaticCredentialSerialized {
            type_id: static_cred.inner.type_id().to_string(),
            metadata: static_cred.metadata.clone(),
            id: static_cred.id.to_string(),
            created_at: static_cred.created_at,
            updated_at: static_cred.updated_at,
            value: static_cred.inner.value(),
        }
    }
}


#[derive(Serialize, Deserialize, ToSchema)]
pub struct ResourceServerCredentialSerialized {
    pub id: WrappedUuidV4,
    pub type_id: String,
    pub metadata: Metadata,
    pub value: WrappedJsonValue,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
    pub next_rotation_time: Option<WrappedChronoDateTime>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UserCredentialSerialized {
    pub id: WrappedUuidV4,
    pub type_id: String,
    pub metadata: Metadata,
    pub value: WrappedJsonValue,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
    pub next_rotation_time: Option<WrappedChronoDateTime>,
}


#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct ProviderCredentialControllerSerialized {
    pub type_id: String,
    pub configuration_schema: ConfigurationSchema,
    pub name: String,
    pub documentation: String,
    pub requires_brokering: bool,
    pub requires_resource_server_credential_refreshing: bool,
    pub requires_user_credential_refreshing: bool,
}

impl From<Arc<dyn ProviderCredentialControllerLike>> for ProviderCredentialControllerSerialized {
    fn from(credential_controller: Arc<dyn ProviderCredentialControllerLike>) -> Self {
        ProviderCredentialControllerSerialized {
            type_id: credential_controller.type_id().to_string(),
            configuration_schema: credential_controller.configuration_schema(),
            name: credential_controller.name().to_string(),
            documentation: credential_controller.documentation().to_string(),
            requires_brokering: credential_controller.as_user_credential_broker().is_some(),
            requires_resource_server_credential_refreshing: credential_controller.as_rotateable_controller_resource_server_credential().is_some(),
            requires_user_credential_refreshing: credential_controller.as_rotateable_controller_user_credential().is_some(),
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ProviderInstanceSerialized {
    // not UUID as some ID's will be deterministic
    pub id: String,
    pub resource_server_credential_id: WrappedUuidV4,
    pub user_credential_id: WrappedUuidV4,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
    pub provider_controller_type_id: String,
    pub credential_controller_type_id: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ProviderInstanceSerializedWithCredentials {
    pub provider_instance: ProviderInstanceSerialized,
    pub static_credential: StaticCredentialSerialized,
    pub resource_server_credential: ResourceServerCredentialSerialized,
    pub user_credential: UserCredentialSerialized,
}

// we shouldn't need this besides the fact that we want to keep track of functions intentionally enabled
// by users. if all functions were enabled, always, we could drop this struct.
pub struct FunctionInstanceSerialized {
    pub id: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
    pub provider_instance_id: String,
    pub function_controller_type_id: String,
}

// api method modelling

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct FunctionControllerSerialized {
    pub type_id: String,
    pub name: String,
    pub documentation: String,
    pub parameters: WrappedSchema,
    pub output: WrappedSchema,
}

impl From<Arc<dyn FunctionControllerLike>> for FunctionControllerSerialized {
    fn from(function: Arc<dyn FunctionControllerLike>) -> Self {
        FunctionControllerSerialized {
            type_id: function.type_id().to_string(),
            name: function.name().to_string(),
            documentation: function.documentation().to_string(),
            parameters: function.parameters(),
            output: function.output(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct ProviderControllerSerialized {
    pub type_id: String,
    pub name: String,
    pub documentation: String,
    pub functions: Vec<FunctionControllerSerialized>,
    pub credential_controllers: Vec<ProviderCredentialControllerSerialized>,
}

impl From<&dyn ProviderControllerLike> for ProviderControllerSerialized {
    fn from(provider: &dyn ProviderControllerLike) -> Self {
        ProviderControllerSerialized {
            type_id: provider.type_id().to_string(),
            name: provider.name().to_string(),
            documentation: provider.documentation().to_string(),
            credential_controllers: provider.credential_controllers().into_iter().map(|c| c.into()).collect::<Vec<ProviderCredentialControllerSerialized>>(),
            functions: provider.functions().into_iter().map(|f| f.into()).collect::<Vec<FunctionControllerSerialized>>(),
        }
    }
}

pub const MAIL_CATEGORY: &str = "mail";

pub static PROVIDER_REGISTRY: Lazy<
    RwLock<Vec<Arc<dyn ProviderControllerLike>>>,
> = Lazy::new(|| RwLock::new(Vec::new()));



pub type ListAvailableProvidersParams = PaginationRequest;
pub type ListAvailableProvidersResponse = PaginatedResponse<ProviderControllerSerialized>;
pub async fn list_available_providers(
    pagination: ListAvailableProvidersParams,
) -> Result<ListAvailableProvidersResponse, CommonError> {
    let providers = PROVIDER_REGISTRY
        .read()
        .map_err(|_e| CommonError::Unknown(anyhow::anyhow!("Poison error")))?
        .iter()
        .map(|p| p.as_ref().into())
        .collect::<Vec<ProviderControllerSerialized>>();
    
    Ok(ListAvailableProvidersResponse::from_items_with_extra(providers, &pagination, |p| vec![p.type_id.to_string()]))
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct WithProviderControllerTypeId<T> {
    pub provider_controller_type_id: String,
    pub inner: T
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct WithCredentialControllerTypeId<T> {
    pub credential_controller_type_id: String,
    pub inner: T
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct EncryptedProviderConfigurationParamsInner {
    pub resource_server_configuration: WrappedJsonValue,
    pub user_credential_configuration: WrappedJsonValue,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct EncryptedProviderConfigurationResponse {
    pub resource_server_configuration: WrappedJsonValue,
    pub user_credential_configuration: WrappedJsonValue,
}

pub type EncryptedProviderConfigurationParams = WithProviderControllerTypeId<WithCredentialControllerTypeId<EncryptedProviderConfigurationParamsInner>>;

pub async fn encrypt_provider_configuration(
    params: EncryptedProviderConfigurationParams,
) -> Result<EncryptedProviderConfigurationResponse, CommonError> {
    let provider_controller = PROVIDER_REGISTRY
        .read()
        .map_err(|_e| CommonError::Unknown(anyhow::anyhow!("Poison error")))?
        .iter()
        .find(|p| p.type_id() == params.provider_controller_type_id)
        .ok_or(CommonError::Unknown(anyhow::anyhow!("Provider controller not found")))?
        .clone();
    

    let credential_controller = provider_controller.credential_controllers().iter().find(|c| c.type_id() == params.inner.credential_controller_type_id)
        .ok_or(CommonError::Unknown(anyhow::anyhow!("Credential controller not found")))?
        .clone();
    let resource_server_configuration = params.inner.inner.resource_server_configuration;
    let user_credential_configuration = params.inner.inner.user_credential_configuration;

    let encrypted_resource_server_configuration = credential_controller.encrypt_resource_server_configuration(resource_server_configuration).await?;
    let encrypted_user_credential_configuration = credential_controller.encrypt_user_credential_configuration(user_credential_configuration).await?;
    
    Ok(EncryptedProviderConfigurationResponse {
        resource_server_configuration: encrypted_resource_server_configuration.value(),
        user_credential_configuration: encrypted_user_credential_configuration.value(),
    })
}


#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct CreateResourceServerCredentialParamsInner {
    // NOTE: serialized values are always already encrypted, only encrypt_provider_configuration accepts raw values
    pub resource_server_configuration: WrappedJsonValue,
}
pub type CreateResourceServerCredentialParams = WithProviderControllerTypeId<WithCredentialControllerTypeId<CreateResourceServerCredentialParamsInner>>;
pub type CreateResourceServerCredentialResponse = ResourceServerCredentialSerialized;

pub async fn create_resource_server_credential(
    params: CreateResourceServerCredentialParams,
) -> Result<CreateResourceServerCredentialResponse, CommonError> {
    let provider_controller = PROVIDER_REGISTRY
        .read()
        .map_err(|_e| CommonError::Unknown(anyhow::anyhow!("Poison error")))?
        .iter()
        .find(|p| p.type_id() == params.provider_controller_type_id)
        .ok_or(CommonError::Unknown(anyhow::anyhow!("Provider controller not found")))?
        .clone();

    let credential_controller = provider_controller.credential_controllers().iter().find(|c| c.type_id() == params.inner.credential_controller_type_id)
        .ok_or(CommonError::Unknown(anyhow::anyhow!("Credential controller not found")))?
        .clone();

    let (resource_server_credential, metadata) = credential_controller.from_serialized_resource_server_configuration(params.inner.inner.resource_server_configuration)?;

    let next_rotation_time = if let Some(resource_server_credential) = resource_server_credential.as_rotateable_credential() {
        Some(resource_server_credential.next_rotation_time())
    } else {
        None
    };
    
    let now = WrappedChronoDateTime::now();
    let resource_server_credential_serialized = ResourceServerCredentialSerialized {
        id: WrappedUuidV4::new(),
        type_id: resource_server_credential.type_id().to_string(),
        metadata: metadata,
        value: resource_server_credential.value(),
        created_at: now,
        updated_at: now,
        next_rotation_time: next_rotation_time,
    };

    // TODO: save to database
    Ok(resource_server_credential_serialized)
}

pub struct CreateUserCredentialParamsInner {
    pub user_credential_configuration: WrappedJsonValue,
    pub metadata: Option<Metadata>,
}
pub type CreateUserCredentialParams = WithProviderControllerTypeId<WithCredentialControllerTypeId<CreateUserCredentialParamsInner>>;
pub type CreateUserCredentialResponse = UserCredentialSerialized;

pub async fn create_user_credential(
    params: CreateUserCredentialParams,
) -> Result<CreateUserCredentialResponse, CommonError> {
    let provider_controller = PROVIDER_REGISTRY
        .read()
        .map_err(|_e| CommonError::Unknown(anyhow::anyhow!("Poison error")))?
        .iter()
        .find(|p| p.type_id() == params.provider_controller_type_id)
        .ok_or(CommonError::Unknown(anyhow::anyhow!("Provider controller not found")))?
        .clone();

    let credential_controller = provider_controller.credential_controllers().iter().find(|c| c.type_id() == params.inner.credential_controller_type_id)
        .ok_or(CommonError::Unknown(anyhow::anyhow!("Credential controller not found")))?
        .clone();

    let (user_credential, mut core_metadata) = credential_controller.from_serialized_user_credential_configuration(params.inner.inner.user_credential_configuration)?;

    if let Some(metadata) = params.inner.inner.metadata {
        core_metadata.0.extend(metadata.0);
    }

    let next_rotation_time = if let Some(user_credential) = user_credential.as_rotateable_credential() {
        Some(user_credential.next_rotation_time())
    } else {
        None
    };

    let now = WrappedChronoDateTime::now();
    let user_credential_serialized = UserCredentialSerialized {
        id: WrappedUuidV4::new(),
        type_id: user_credential.type_id().to_string(),
        metadata: core_metadata,
        value: user_credential.value(),
        created_at: now,
        updated_at: now,
        next_rotation_time: next_rotation_time,
    };

    // TODO: save to database
    Ok(user_credential_serialized)
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct CreateProviderInstanceParamsInner {
    pub resource_server_credential_id: WrappedUuidV4,
    pub user_credential_id: WrappedUuidV4,
    pub provider_instance_id: Option<String>,
}
pub type CreateProviderInstanceParams = WithProviderControllerTypeId<WithCredentialControllerTypeId<CreateProviderInstanceParamsInner>>;
pub type CreateProviderInstanceResponse = ProviderInstanceSerialized;

pub async fn create_provider_instance(
    params: CreateProviderInstanceParams,
) -> Result<CreateProviderInstanceResponse, CommonError> {

    let provider_controller = PROVIDER_REGISTRY
        .read()
        .map_err(|_e| CommonError::Unknown(anyhow::anyhow!("Poison error")))?
        .iter()
        .find(|p| p.type_id() == params.provider_controller_type_id)
        .ok_or(CommonError::Unknown(anyhow::anyhow!("Provider controller not found")))?
        .clone();

    let credential_controller = provider_controller.credential_controllers().iter().find(|c| c.type_id() == params.inner.credential_controller_type_id)
        .ok_or(CommonError::Unknown(anyhow::anyhow!("Credential controller not found")))?
        .clone();

    // let resource_server_credential = provider_controller.get_resource_server_credential(params.resource_server_credential_id)?;
    // let user_credential = provider_controller.get_user_credential(params.user_credential_id)?;

    // TODO: fetch resource and user credentials from database to ensure they exist

    let provider_instance_id = match params.inner.inner.provider_instance_id {
        Some(provider_instance_id) => provider_instance_id,
        None => uuid::Uuid::new_v4().to_string(),
    };
    let now = WrappedChronoDateTime::now();
    let provider_instance_serialized = ProviderInstanceSerialized {
        id: provider_instance_id,
        resource_server_credential_id: params.inner.inner.resource_server_credential_id,
        user_credential_id: params.inner.inner.user_credential_id,
        created_at: now,
        updated_at: now,
        provider_controller_type_id: params.provider_controller_type_id,
        credential_controller_type_id: params.inner.credential_controller_type_id,
    };

    // TODO: save to database
    Ok(provider_instance_serialized)
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct WithProviderInstanceId<T> {
    pub provider_instance_id: String,
    pub inner: T
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct EnableFunctionParamsInner {
    pub function_controller_type_id: String,
    pub function_instance_id: Option<String>,
}
pub type EnableFunctionParams = WithProviderInstanceId<EnableFunctionParamsInner>;
pub type EnableFunctionResponse = FunctionInstanceSerialized;

pub async fn enable_function(
    params: EnableFunctionParams,
) -> Result<EnableFunctionResponse, CommonError> {
    let provider_instance = PROVIDER_REGISTRY
        .read()
        .map_err(|_e| CommonError::Unknown(anyhow::anyhow!("Poison error")))?
        .iter()
        .find(|p| p.type_id() == params.provider_instance_id)
        .ok_or(CommonError::Unknown(anyhow::anyhow!("Provider instance not found")))?
        .clone();
    let function = provider_instance.functions().iter().find(|f| f.type_id() == params.inner.function_controller_type_id)
        .ok_or(CommonError::Unknown(anyhow::anyhow!("Function not found")))?
        .clone();
    let function_instance_id = match params.inner.function_instance_id {
        Some(function_instance_id) => function_instance_id,
        None => uuid::Uuid::new_v4().to_string(),
    };
    let now = WrappedChronoDateTime::now();
    let function_instance_serialized = FunctionInstanceSerialized {
        id: function_instance_id,
        created_at: now,
        updated_at: now,
        provider_instance_id: params.provider_instance_id,
        function_controller_type_id: params.inner.function_controller_type_id,
    };

    // TODO: save to database
    Ok(function_instance_serialized)
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct DisableFunctionParamsInner {
    pub function_instance_id: String,
}
pub type DisableFunctionParams = WithProviderInstanceId<DisableFunctionParamsInner>;
pub type DisableFunctionResponse = ();

pub async fn disable_function(
    params: DisableFunctionParams,
) -> Result<DisableFunctionResponse, CommonError> {
    // TODO: delete from database
    Ok(())
}

// TODO: list functions potentially? but assuming a clean database on startup for now.

async fn process_broker_outcome(provider_controller: &Arc<dyn ProviderControllerLike>, credential_controller: &Arc<dyn ProviderCredentialControllerLike>, resource_server_cred_id: WrappedUuidV4, broker_action: &BrokerAction, outcome: BrokerOutcome) -> Result<UserCredentialBrokeringResponse, CommonError> {
    let response = match outcome {
        BrokerOutcome::Success { user_credential, metadata } => {
            let user_credential = create_user_credential(CreateUserCredentialParams {
                provider_controller_type_id: provider_controller.type_id().to_string(),
                inner: WithCredentialControllerTypeId {
                    credential_controller_type_id: credential_controller.type_id().to_string(),
                    inner: CreateUserCredentialParamsInner {
                        user_credential_configuration: user_credential.value(),
                        metadata: Some(metadata),
                    },
                },
            }).await?;

            UserCredentialBrokeringResponse::UserCredential(user_credential)
        },
        BrokerOutcome::Continue { metadata } => { 
            let broker_state = BrokerState {
                id: uuid::Uuid::new_v4().to_string(),
                created_at: WrappedChronoDateTime::now(),
                updated_at: WrappedChronoDateTime::now(),
                resource_server_cred_id: resource_server_cred_id,
                provider_controller_type_id: provider_controller.type_id().to_string(),
                metadata: metadata,
                action: broker_action.clone(),
                credential_controller_type_id: credential_controller.type_id().to_string(),
            };

            UserCredentialBrokeringResponse::BrokerState(broker_state)
        },
    };

    Ok(response)
}



#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct StartUserCredentialBrokeringParamsInner {
    pub resource_server_cred_id: WrappedUuidV4,
}
pub type StartUserCredentialBrokeringParams = WithProviderControllerTypeId<WithCredentialControllerTypeId<StartUserCredentialBrokeringParamsInner>>;
pub enum UserCredentialBrokeringResponse {
    BrokerState(BrokerState),
    UserCredential(UserCredentialSerialized),
}
pub async fn start_user_credential_brokering(
    params: StartUserCredentialBrokeringParams,
) -> Result<UserCredentialBrokeringResponse, CommonError> {
    let provider_controller = PROVIDER_REGISTRY
        .read()
        .map_err(|_e| CommonError::Unknown(anyhow::anyhow!("Poison error")))?
        .iter()
        .find(|p| p.type_id() == params.provider_controller_type_id)
        .ok_or(CommonError::Unknown(anyhow::anyhow!("Provider controller not found")))?
        .clone();
    let credential_controller = provider_controller.credential_controllers().iter().find(|c| c.type_id() == params.inner.credential_controller_type_id)
        .ok_or(CommonError::Unknown(anyhow::anyhow!("Credential controller not found")))?
        .clone();
    let  user_credential_broker = match credential_controller.as_user_credential_broker() {
        Some(broker) => broker,
        None => return Err(CommonError::Unknown(anyhow::anyhow!("Provider controller does not support user credential brokering"))),
    };


    // TODO: fetch resource server credential from database to ensure it exists
    let resource_server_cred = ResourceServerCredentialSerialized {
        id: WrappedUuidV4::new(),
        type_id: provider_controller.type_id().to_string(),
        metadata: Metadata::new(),
        value: WrappedJsonValue::new(serde_json::json!({})),
        created_at: WrappedChronoDateTime::now(),
        updated_at: WrappedChronoDateTime::now(),
        next_rotation_time: None,
    };

    let (inner, metadata) = credential_controller.from_serialized_resource_server_configuration(resource_server_cred.value)?;
    let resource_server_cred = Credential {
        inner: inner,
        metadata: metadata,
        id: resource_server_cred.id,
        created_at: resource_server_cred.created_at,
        updated_at: resource_server_cred.updated_at,
    };
    let (action, outcome) = user_credential_broker.start(&resource_server_cred).await?;

    
    let response = process_broker_outcome(&provider_controller, &credential_controller, params.inner.inner.resource_server_cred_id, &action,outcome).await?;
    Ok(response)
}

pub struct  ResumeUserCredentialBrokeringParams {
    pub broker_state_id: String,
    pub input: BrokerInput
}

pub async fn resume_user_credential_brokering(
    params: ResumeUserCredentialBrokeringParams,
) -> Result<UserCredentialBrokeringResponse, CommonError> {
    // TODO: fetch broker state from database to ensure it exists
    let broker_state = BrokerState {
        id: params.broker_state_id,
        created_at: WrappedChronoDateTime::now(),
        updated_at: WrappedChronoDateTime::now(),
        resource_server_cred_id: WrappedUuidV4::new(),
        provider_controller_type_id: "".to_string(),
        metadata: Metadata::new(),
        credential_controller_type_id: "".to_string(),
        action: BrokerAction::None,
    };

    let provider_controller = PROVIDER_REGISTRY
        .read()
        .map_err(|_e| CommonError::Unknown(anyhow::anyhow!("Poison error")))?
        .iter()
        .find(|p| p.type_id() == broker_state.provider_controller_type_id)
        .ok_or(CommonError::Unknown(anyhow::anyhow!("Provider controller not found")))?
        .clone();
    let credential_controller = provider_controller.credential_controllers().iter().find(|c| c.type_id() == broker_state.credential_controller_type_id)
        .ok_or(CommonError::Unknown(anyhow::anyhow!("Credential controller not found")))?
        .clone();

    let user_credential_broker = match credential_controller.as_user_credential_broker() {
        Some(broker) => broker,
        None => return Err(CommonError::Unknown(anyhow::anyhow!("Provider controller does not support user credential brokering"))),
    };


    let (action, outcome) = user_credential_broker.resume(&broker_state, params.input).await?;

    let response = process_broker_outcome(&provider_controller, &credential_controller, broker_state.resource_server_cred_id, &action,outcome).await?;
    Ok(response)
}