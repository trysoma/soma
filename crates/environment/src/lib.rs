//! Environment crate for managing secrets and environment variables
//!
//! This crate provides:
//! - Secret storage with encryption (using DEK aliases)
//! - Environment variable storage (plain-text)
//! - Change event publishing for real-time updates
//! - HTTP API endpoints for CRUD operations

pub mod logic;
pub mod repository;
pub mod router;
pub mod service;

#[cfg(test)]
pub mod test;

// Re-export commonly used types
pub use logic::secret::{Secret, SecretChangeEvt, SecretChangeTx, create_secret_change_channel};
pub use logic::variable::{
    Variable, VariableChangeEvt, VariableChangeTx, create_variable_change_channel,
};
pub use repository::{Repository, SecretRepositoryLike, VariableRepositoryLike};
pub use router::create_router;
pub use service::{EnvironmentService, EnvironmentServiceParams};
