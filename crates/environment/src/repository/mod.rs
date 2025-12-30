//! Repository layer for environment crate
//! Contains trait definitions and implementations for secret and variable storage

pub mod sqlite;

use async_trait::async_trait;
use shared::{
    error::CommonError,
    primitives::{PaginatedResponse, PaginationRequest, WrappedChronoDateTime, WrappedUuidV4},
};

pub use sqlite::Repository;

use crate::logic::{secret::Secret, variable::Variable};

/// Parameters for creating a new secret
#[derive(Debug, Clone)]
pub struct CreateSecret {
    pub id: WrappedUuidV4,
    pub key: String,
    pub encrypted_secret: String,
    pub dek_alias: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

/// Parameters for updating an existing secret
#[derive(Debug, Clone)]
pub struct UpdateSecret {
    pub id: WrappedUuidV4,
    pub encrypted_secret: String,
    pub dek_alias: String,
    pub updated_at: WrappedChronoDateTime,
}

/// Parameters for creating a new variable
#[derive(Debug, Clone)]
pub struct CreateVariable {
    pub id: WrappedUuidV4,
    pub key: String,
    pub value: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

/// Parameters for updating an existing variable
#[derive(Debug, Clone)]
pub struct UpdateVariable {
    pub id: WrappedUuidV4,
    pub value: String,
    pub updated_at: WrappedChronoDateTime,
}

/// Repository trait for secret operations
#[async_trait]
pub trait SecretRepositoryLike: Send + Sync {
    /// Create a new secret
    async fn create_secret(&self, params: &CreateSecret) -> Result<(), CommonError>;

    /// Update an existing secret
    async fn update_secret(&self, params: &UpdateSecret) -> Result<(), CommonError>;

    /// Delete a secret by ID
    async fn delete_secret(&self, id: &WrappedUuidV4) -> Result<(), CommonError>;

    /// Get a secret by ID
    async fn get_secret_by_id(&self, id: &WrappedUuidV4) -> Result<Option<Secret>, CommonError>;

    /// Get a secret by key
    async fn get_secret_by_key(&self, key: &str) -> Result<Option<Secret>, CommonError>;

    /// List secrets with pagination
    async fn get_secrets(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<Secret>, CommonError>;
}

/// Repository trait for variable operations
#[async_trait]
pub trait VariableRepositoryLike: Send + Sync {
    /// Create a new variable
    async fn create_variable(&self, params: &CreateVariable) -> Result<(), CommonError>;

    /// Update an existing variable
    async fn update_variable(&self, params: &UpdateVariable) -> Result<(), CommonError>;

    /// Delete a variable by ID
    async fn delete_variable(&self, id: &WrappedUuidV4) -> Result<(), CommonError>;

    /// Get a variable by ID
    async fn get_variable_by_id(&self, id: &WrappedUuidV4)
    -> Result<Option<Variable>, CommonError>;

    /// Get a variable by key
    async fn get_variable_by_key(&self, key: &str) -> Result<Option<Variable>, CommonError>;

    /// List variables with pagination
    async fn get_variables(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<Variable>, CommonError>;
}
