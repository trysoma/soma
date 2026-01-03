//! Service layer for environment crate
//! Provides the main service struct that holds all dependencies for environment operations

use encryption::logic::crypto_services::CryptoCache;

use crate::{
    logic::{secret::SecretChangeTx, variable::VariableChangeTx},
    repository::Repository,
};

/// Main service struct for environment operations
/// Holds all dependencies needed for secret and variable CRUD operations
#[derive(Clone)]
pub struct EnvironmentService {
    pub repository: Repository,
    pub crypto_cache: CryptoCache,
    pub secret_change_tx: SecretChangeTx,
    pub variable_change_tx: VariableChangeTx,
}

/// Parameters for creating an EnvironmentService
pub struct EnvironmentServiceParams {
    pub repository: Repository,
    pub crypto_cache: CryptoCache,
    pub secret_change_tx: SecretChangeTx,
    pub variable_change_tx: VariableChangeTx,
}

impl EnvironmentService {
    /// Create a new EnvironmentService instance
    pub fn new(params: EnvironmentServiceParams) -> Self {
        Self {
            repository: params.repository,
            crypto_cache: params.crypto_cache,
            secret_change_tx: params.secret_change_tx,
            variable_change_tx: params.variable_change_tx,
        }
    }
}
