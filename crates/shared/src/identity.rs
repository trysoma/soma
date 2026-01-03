use crate::error::CommonError;
use http::HeaderMap;
use libsql::FromValue;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::primitives::WrappedChronoDateTime;

/// User role in the system defining permission levels.
///
/// Roles are hierarchical with Admin having the highest privileges.
#[derive(Debug, Clone, PartialEq, Eq, ToSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// Full system access including user management and configuration
    Admin,
    /// Can manage resources but not system configuration
    Maintainer,
    /// Automated service account with limited permissions
    Agent,
    /// Standard user with basic read/write access
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
    /// Returns the string representation of the role (e.g., "admin", "user").
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Admin => "admin",
            Role::Maintainer => "maintainer",
            Role::Agent => "agent",
            Role::User => "user",
        }
    }

    /// Parses a role from its string representation.
    ///
    /// Returns `None` if the string doesn't match a valid role.
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

/// Type of user identity in the system.
///
/// Distinguishes between automated services and real users.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum UserType {
    /// Automated service or API client (authenticates via API key)
    Machine,
    /// Real user (authenticates via STS/OAuth tokens)
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
    /// Returns the string representation of the user type (e.g., "machine", "human").
    pub fn as_str(&self) -> &'static str {
        match self {
            UserType::Machine => "machine",
            UserType::Human => "human",
        }
    }

    /// Parses a user type from its string representation.
    ///
    /// Returns `None` if the string doesn't match a valid user type.
    pub fn parse(s: &str) -> Option<UserType> {
        match s {
            "machine" => Some(UserType::Machine),
            "human" => Some(UserType::Human),
            _ => None,
        }
    }
}

/// A user entity in the system.
///
/// Represents both human users and machine accounts with their associated metadata.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct User {
    /// Unique identifier (UUID format)
    pub id: String,
    /// Whether this is a machine or human user
    pub user_type: UserType,
    /// Email address (required for human users, optional for machines)
    pub email: Option<String>,
    /// Permission level assigned to this user
    pub role: Role,
    /// Optional human-readable description of the user or its purpose
    pub description: Option<String>,
    /// Timestamp when the user was created (UTC)
    pub created_at: WrappedChronoDateTime,
    /// Timestamp when the user was last modified (UTC)
    pub updated_at: WrappedChronoDateTime,
}

/// A group entity for organizing users and managing access control.
///
/// Groups allow assigning permissions to multiple users at once.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct Group {
    /// Unique identifier (UUID format)
    pub id: String,
    /// Human-readable group name
    pub name: String,
    /// Timestamp when the group was created (UTC)
    pub created_at: WrappedChronoDateTime,
    /// Timestamp when the group was last modified (UTC)
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

/// Authenticated machine identity.
///
/// Represents an API client or automated service that authenticated via API key.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Machine {
    /// Subject identifier (user ID) of the machine account
    pub sub: String,
    /// Permission level of this machine
    pub role: Role,
}

/// Authenticated human identity.
///
/// Represents a real user that authenticated via STS/OAuth token.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Human {
    /// Subject identifier (user ID) of the human user
    pub sub: String,
    /// Email address of the user (if available from token)
    pub email: Option<String>,
    /// Group IDs the user belongs to
    pub groups: Vec<String>,
    /// Permission level of this user
    pub role: Role,
}

/// Authenticated identity representing the caller of a request.
///
/// This is the result of authentication and is used throughout the system
/// for authorization decisions.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Identity {
    /// Machine identity (API key authentication)
    Machine(Machine),
    /// Human identity (STS token authentication)
    Human(Human),
    /// Machine acting on behalf of a human (both credentials provided)
    MachineOnBehalfOfHuman { machine: Machine, human: Human },
    /// Unauthenticated request (no valid credentials)
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
