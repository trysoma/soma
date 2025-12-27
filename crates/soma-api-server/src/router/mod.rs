use axum::{Router, extract::OriginalUri, middleware};
use shared::adapters::openapi::API_VERSION_TAG;
use utoipa::openapi::OpenApi as OpenApiDoc;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, Http, HttpAuthScheme, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::ApiService;
use encryption::router::create_router as create_encryption_router;
use identity::router::create_router as create_identity_router;
use mcp::router::create_router as create_mcp_router;
use shared::error::CommonError;

pub(crate) mod agent;
pub(crate) mod environment_variable;
pub(crate) mod internal;
pub(crate) mod secret;
pub(crate) mod task;

/// Middleware that stores the original URI in request extensions before nest_service strips the path.
async fn store_original_uri(
    original_uri: OriginalUri,
    mut request: axum::http::Request<axum::body::Body>,
    next: middleware::Next,
) -> axum::response::Response {
    request.extensions_mut().insert(original_uri);
    next.run(request).await
}

pub fn initiaite_api_router(api_service: ApiService) -> Result<Router, CommonError> {
    let mut router = Router::new();

    // let (live_connection_changes_tx, mut live_connection_changes_rx) = tokio::sync::mpsc::channel(10);

    // agent router

    let (agent_router, _) = agent::create_router().split_for_parts();

    let agent_router = agent_router.with_state(api_service.agent_service);
    router = router.merge(agent_router);

    // task router
    let (task_router, _) = task::create_router().split_for_parts();
    let task_router = task_router.with_state(api_service.task_service);
    router = router.merge(task_router);

    // mcp router
    let (mcp_router, _) = create_mcp_router().split_for_parts();
    let mcp_router = mcp_router.with_state(api_service.mcp_service.clone());
    router = router.merge(mcp_router);

    // MCP Streamable HTTP service - nested under /api/mcp/v1/mcp-server/{mcp_server_instance_id}/mcp
    let mcp_service = api_service.mcp_service.mcp_service().clone();
    router = router.nest_service(
        "/api/mcp/v1/mcp-server/{mcp_server_instance_id}/mcp",
        mcp_service,
    );

    // internal router
    let (internal_router, _) = internal::create_router().split_for_parts();
    let internal_router = internal_router.with_state(api_service.internal_service);
    router = router.merge(internal_router);

    // encryption router
    let (encryption_router, _) = create_encryption_router().split_for_parts();
    let encryption_router = encryption_router.with_state(api_service.encryption_service.clone());
    router = router.merge(encryption_router);

    // secret router
    let (secret_router, _) = secret::create_router().split_for_parts();
    let secret_router = secret_router.with_state(api_service.secret_service);
    router = router.merge(secret_router);

    // environment variable router
    let (env_var_router, _) = environment_variable::create_router().split_for_parts();
    let env_var_router = env_var_router.with_state(api_service.environment_variable_service);
    router = router.merge(env_var_router);

    // identity router
    let (identity_router, _) = create_identity_router().split_for_parts();
    let identity_router = identity_router.with_state(api_service.identity_service);
    router = router.merge(identity_router);

    // Apply middleware to store original URI for nested services (like MCP)
    let router = router.layer(middleware::from_fn(store_original_uri));

    Ok(router)
}

pub fn generate_openapi_spec() -> OpenApiDoc {
    let mut spec = ApiDoc::openapi().clone();
    let (_, agent_spec) = agent::create_router().split_for_parts();
    let (_, task_spec) = task::create_router().split_for_parts();
    let (_, mcp_spec) = create_mcp_router().split_for_parts();
    let (_, internal_spec) = internal::create_router().split_for_parts();
    let (_, encryption_spec) = create_encryption_router().split_for_parts();
    let (_, secret_spec) = secret::create_router().split_for_parts();
    let (_, env_var_spec) = environment_variable::create_router().split_for_parts();
    let (_, identity_spec) = create_identity_router().split_for_parts();
    spec.merge(agent_spec);
    spec.merge(task_spec);
    spec.merge(mcp_spec);
    spec.merge(internal_spec);
    spec.merge(encryption_spec);
    spec.merge(secret_spec);
    spec.merge(env_var_spec);
    spec.merge(identity_spec);

    spec
}

#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
    info(
        title = "soma",
        version = "v1",
        description = "An open source AI agent runtime",
        license(identifier = "Elastic License 2.0")
    ),
    tags(
        (name = "task", description = "Task management endpoints for creating, listing, and managing tasks and their messages"),
        (name = "secret", description = "Secret management endpoints for storing and retrieving encrypted secrets"),
        (name = "environment-variable", description = "Environment variable management endpoints for storing and retrieving environment variables"),
        (name = "encryption", description = "Encryption key management endpoints for envelope keys, data encryption keys, and aliases"),
        (name = "mcp", description = "MCP endpoints for managing providers, credentials, functions, and MCP protocol communication"),
        (name = "_internal", description = "Internal endpoints for health checks, runtime configuration, and SDK code generation"),
        (name = "agent", description = "Agent management and A2A (agent-to-agent) communication endpoints"),
        (name = "identity", description = "Identity management endpoints for JWKs (JSON Web Keys) and authentication"),
        (name = API_VERSION_TAG, description = "API version v1 endpoints")
    )
)]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-API-Key"))),
            );

            components.add_security_scheme(
                "bearer_token",
                SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
            );
        }
    }
}
