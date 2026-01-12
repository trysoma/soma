use crate::error::CommonError;
use http::HeaderMap;
use libsql::FromValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::primitives::WrappedChronoDateTime;

/// User role in the system
#[derive(Debug, Clone, PartialEq, Eq, ToSchema, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Admin,
    Maintainer,
    Agent,
    User,
}

impl FromValue for Role {
    fn from_sql(val: libsql::Value) -> libsql::Result<Self> {
        match val {
            libsql::Value::Text(s) => {
                Role::parse(&s).ok_or_else(|| libsql::Error::InvalidColumnType)
            }
            _ => Err(libsql::Error::InvalidColumnType),
        }
    }
}

impl From<Role> for libsql::Value {
    fn from(val: Role) -> Self {
        libsql::Value::Text(val.as_str().to_string())
    }
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Admin => "admin",
            Role::Maintainer => "maintainer",
            Role::Agent => "agent",
            Role::User => "user",
        }
    }

    /// Parse a role from string
    pub fn parse(s: &str) -> Option<Role> {
        match s {
            "admin" => Some(Role::Admin),
            "maintainer" => Some(Role::Maintainer),
            "agent" => Some(Role::Agent),
            "user" => Some(Role::User),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum UserType {
    Machine,
    Human,
}

impl FromValue for UserType {
    fn from_sql(val: libsql::Value) -> libsql::Result<Self> {
        match val {
            libsql::Value::Text(s) => {
                UserType::parse(&s).ok_or_else(|| libsql::Error::InvalidColumnType)
            }
            _ => Err(libsql::Error::InvalidColumnType),
        }
    }
}

impl From<UserType> for libsql::Value {
    fn from(val: UserType) -> Self {
        libsql::Value::Text(val.as_str().to_string())
    }
}

impl UserType {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserType::Machine => "machine",
            UserType::Human => "human",
        }
    }

    pub fn parse(s: &str) -> Option<UserType> {
        match s {
            "machine" => Some(UserType::Machine),
            "human" => Some(UserType::Human),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct User {
    pub id: String,
    pub user_type: UserType,
    pub email: Option<String>,
    pub role: Role,
    pub description: Option<String>,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

// Group types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

/// Raw API key credential
pub struct ApiKey(pub String);

/// Raw Internal token credential
pub struct InternalToken(pub String);

/// Raw credentials that can be extracted from a request
pub enum RawCredentials {
    /// Machine authentication via API key
    MachineApiKey(ApiKey),
    /// Human authentication via STS token (JWT)
    HumanInternalToken(InternalToken),
    /// Machine acting on behalf of a human
    MachineOnBehalfOfHuman(ApiKey, InternalToken),
    /// if we havent extracted credentials from the request, use the header map
    HeaderMap(HeaderMap),
    /// if user is already authenticated, use the identity
    Identity(Identity),
}

impl From<HeaderMap> for RawCredentials {
    fn from(headers: HeaderMap) -> Self {
        RawCredentials::HeaderMap(headers)
    }
}

impl From<Identity> for RawCredentials {
    fn from(identity: Identity) -> Self {
        RawCredentials::Identity(identity)
    }
}

/// Authenticated machine identity
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct Machine {
    pub sub: String,
    pub role: Role,
}

/// Authenticated human identity
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct Human {
    pub sub: String,
    pub email: Option<String>,
    pub groups: Vec<String>,
    pub role: Role,
}

/// Authenticated identity
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Identity {
    /// Machine identity (API key authentication)
    Machine(Machine),
    /// Human identity (STS token authentication)
    Human(Human),
    /// Machine acting on behalf of a human
    MachineOnBehalfOfHuman { machine: Machine, human: Human },
    /// Unauthenticated request
    Unauthenticated,
}

impl Identity {
    /// Get the role of the identity
    pub fn role(&self) -> Option<&Role> {
        match self {
            Identity::Machine(m) => Some(&m.role),
            Identity::Human(h) => Some(&h.role),
            Identity::MachineOnBehalfOfHuman { machine, human: _ } => Some(&machine.role),
            Identity::Unauthenticated => None,
        }
    }

    /// Check if the identity is authenticated
    pub fn is_authenticated(&self) -> bool {
        !matches!(self, Identity::Unauthenticated)
    }

    /// Get the subject ID of the identity
    pub fn subject(&self) -> Option<&str> {
        match self {
            Identity::Machine(m) => Some(&m.sub),
            Identity::Human(h) => Some(&h.sub),
            Identity::MachineOnBehalfOfHuman { machine, human: _ } => Some(&machine.sub),
            Identity::Unauthenticated => None,
        }
    }
}

#[allow(async_fn_in_trait)]
pub trait AuthClientLike {
    /// Authenticate from raw credentials
    async fn authenticate(&self, credentials: RawCredentials) -> Result<Identity, CommonError>;

    /// Authenticate from HTTP headers
    ///
    /// Extracts credentials from HTTP headers and authenticates them.
    /// Returns `Identity::Unauthenticated` if no credentials are found.
    async fn authenticate_from_headers(&self, headers: &HeaderMap)
    -> Result<Identity, CommonError>;
}

/// Blanket implementation for Arc<T> where T implements AuthClientLike
///
/// This allows passing Arc<AuthClient> directly to functions expecting impl AuthClientLike
impl<T: AuthClientLike + Send + Sync> AuthClientLike for std::sync::Arc<T> {
    async fn authenticate(&self, credentials: RawCredentials) -> Result<Identity, CommonError> {
        (**self).authenticate(credentials).await
    }

    async fn authenticate_from_headers(
        &self,
        headers: &HeaderMap,
    ) -> Result<Identity, CommonError> {
        (**self).authenticate_from_headers(headers).await
    }
}
