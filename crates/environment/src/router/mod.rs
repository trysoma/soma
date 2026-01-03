//! Router layer for environment crate
//! Contains HTTP endpoints for secrets and variables

pub mod secret;
pub mod variable;

use std::sync::Arc;
use utoipa::openapi::OpenApi as OpenApiDoc;
use utoipa_axum::router::OpenApiRouter;

use crate::service::EnvironmentService;

/// Create the combined environment router
pub fn create_router() -> OpenApiRouter<Arc<EnvironmentService>> {
    let secret_router = secret::create_router();
    let variable_router = variable::create_router();

    OpenApiRouter::new()
        .merge(secret_router)
        .merge(variable_router)
}

/// Get the combined OpenAPI spec for the environment crate
pub fn get_openapi_spec() -> OpenApiDoc {
    let (_, secret_spec) = secret::create_router().split_for_parts();
    let (_, variable_spec) = variable::create_router().split_for_parts();

    let mut spec = secret_spec;
    spec.merge(variable_spec);
    spec
}
