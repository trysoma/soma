use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shared::error::CommonError;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, JsonSchema)]
pub struct HealthCheckResponse {
    pub status: String,
}

/// Health check endpoint
pub async fn health_check() -> Result<HealthCheckResponse, CommonError> {
    Ok(HealthCheckResponse {
        status: "ok".to_string(),
    })
}