use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use shared::{error::CommonError, primitives::{WrappedChronoDateTime, WrappedUuidV4}};
use reqwest::Request;

#[derive(Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct Metadata(pub serde_json::Map<String, serde_json::Value>);

impl Metadata {
    pub fn new() -> Self {
        Self(serde_json::Map::new())
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DatabaseCredential<T> {
    pub inner: T,
    pub metadata: Metadata,
    pub id: WrappedUuidV4,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

// Static credential configurations

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum StaticCredentialConfigurationVariant {
    NoAuth(NoAuthStaticCredentialConfiguration),
    Oauth2AuthorizationCodeFlow(Oauth2AuthorizationCodeFlowStaticCredentialConfiguration),
    Oauth2JwtBearerAssertionFlow(Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration),
    Custom(CustomStaticCredentialConfiguration),
}


#[derive(Serialize, Deserialize, Clone  )]
#[serde(rename_all = "snake_case")]
pub enum StaticCredentialConfigurationType {
    NoAuth,
    Oauth2AuthorizationCodeFlow,
    Oauth2JwtBearerAssertionFlow,
    Custom,
}

pub struct StaticCredentialConfiguration {
    pub inner: StaticCredentialConfigurationVariant,
    pub metadata: Metadata,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NoAuthStaticCredentialConfiguration {
    pub metadata: Metadata,
}


#[derive(Serialize, Deserialize, Clone)]
pub struct Oauth2AuthorizationCodeFlowStaticCredentialConfiguration {
    pub auth_uri: String,
    pub token_uri: String,
    pub userinfo_uri: String,
    pub jwks_uri: String,
    pub issuer: String,
    pub scopes: Vec<String>,
    pub metadata: Metadata,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration {
    pub auth_uri: String,
    pub token_uri: String,
    pub userinfo_uri: String,
    pub jwks_uri: String,
    pub issuer: String,
    pub scopes: Vec<String>,
    pub metadata: Metadata,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CustomStaticCredentialConfiguration {
    pub metadata: Metadata,
}


#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ResourceServerCredentialVariant {
    // TODO: this doesnt need a struct for no auth but here for macro expansion
    NoAuth(NoAuthResourceServerCredential),
    Oauth2AuthorizationCodeFlow(Oauth2AuthorizationCodeFlowResourceServerCredential),
    Oauth2JwtBearerAssertionFlow(Oauth2JwtBearerAssertionFlowResourceServerCredential),
    Custom(CustomResourceServerCredential),
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ResourceServerCredentialType {
    NoAuth,
    Oauth2AuthorizationCodeFlow,
    Oauth2JwtBearerAssertionFlow,
    Custom,
}

pub type ResourceServerCredential = DatabaseCredential<ResourceServerCredentialVariant>;

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum UserCredentialVariant {
    // TODO: this doesnt need a struct for no auth but here for macro expansion
    NoAuth(NoAuthUserCredential),
    Oauth2AuthorizationCodeFlow(Oauth2AuthorizationCodeFlowUserCredential),
    Oauth2JwtBearerAssertionFlow(Oauth2JwtBearerAssertionFlowUserCredential),
    Custom(CustomUserCredential),
}

pub type UserCredential = DatabaseCredential<UserCredentialVariant>;

#[derive(Serialize, Deserialize, Clone)]
pub struct NoAuthFullCredential {
    pub static_cred: NoAuthStaticCredentialConfiguration,
    pub resource_server_cred: NoAuthResourceServerCredential,
    pub user_cred: NoAuthUserCredential,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Oauth2AuthorizationCodeFlowFullCredential {
    pub static_cred: Oauth2AuthorizationCodeFlowStaticCredentialConfiguration,
    pub resource_server_cred: Oauth2AuthorizationCodeFlowResourceServerCredential,
    pub user_cred: Oauth2AuthorizationCodeFlowUserCredential,
}

#[derive(Serialize, Deserialize, Clone)]    
pub struct Oauth2JwtBearerAssertionFlowFullCredential {
    pub static_cred: Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration,
    pub resource_server_cred: Oauth2JwtBearerAssertionFlowResourceServerCredential,
    pub user_cred: Oauth2JwtBearerAssertionFlowUserCredential,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CustomFullCredential {
    pub static_cred: CustomStaticCredentialConfiguration,
    pub resource_server_cred: CustomResourceServerCredential,
    pub user_cred: CustomUserCredential,
}

// Resource server credentials
#[derive(Serialize, Deserialize, Clone)]
pub struct NoAuthResourceServerCredential {
    pub metadata: Metadata,
}




#[derive(Serialize, Deserialize, Clone)]
pub struct Oauth2AuthorizationCodeFlowResourceServerCredential {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub metadata: Metadata,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Oauth2JwtBearerAssertionFlowResourceServerCredential {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub metadata: Metadata,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Oauth2AuthorizationCodeFlowUserCredential {
    pub code: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expiry_time: WrappedChronoDateTime,
    pub sub: String,
    pub metadata: Metadata,
}

// User credentials
#[derive(Serialize, Deserialize, Clone)]
pub struct NoAuthUserCredential {
    pub metadata: Metadata,
}


#[derive(Serialize, Deserialize, Clone)]
pub struct Oauth2JwtBearerAssertionFlowUserCredential {
    pub assertion: String,
    pub token: String,
    pub expiry_time: WrappedChronoDateTime,
    pub sub: String,
    pub metadata: Metadata,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CustomResourceServerCredential {
    pub metadata: Metadata,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CustomUserCredential {
    pub metadata: Metadata,
}


#[enum_dispatch]
pub trait CredentialInjectorLike {
    fn inject_credentials(&self, request: &mut Request);
}


// standard credential injectors
pub trait ProviderControllerLike {
    type ProviderInstance;
    async fn save_resource_server_credential(input: ResourceServerCredentialVariant) -> Result<ResourceServerCredential, CommonError>;
    async fn save_user_credential(input: UserCredentialVariant) -> Result<UserCredential, CommonError>;
    async fn get_static_credentials(variant: StaticCredentialConfigurationType) -> Result<StaticCredentialConfiguration, CommonError>;
    fn id() -> String;
    fn documentation_url() -> String;
    fn name() -> String;
}


// standard 