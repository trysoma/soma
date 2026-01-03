use axum::{Router, extract::OriginalUri, middleware};
use shared::adapters::openapi::API_VERSION_TAG;
use std::sync::Arc;
use utoipa::openapi::OpenApi as OpenApiDoc;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, Http, HttpAuthScheme, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::ApiService;
use encryption::router::create_router as create_encryption_router;
use environment::router::create_router as create_environment_router;
use identity::router::create_router as create_identity_router;
use inbox_a2a::router::create_router as create_inbox_a2a_router;
use mcp::router::create_router as create_mcp_router;
use shared::error::CommonError;

pub(crate) mod agent;
pub(crate) mod internal;

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

    // A2A router (task and agent endpoints) - now from inbox-a2a crate
    let (a2a_router, _) = create_inbox_a2a_router().split_for_parts();
    let a2a_router = a2a_router.with_state(api_service.a2a_service.clone());
    router = router.merge(a2a_router);

    // Agent list route (separate from inbox-a2a)
    let agent_service = Arc::new(agent::AgentService::new(
        api_service.a2a_service.agent_cache().clone(),
    ));
    let (agent_router, _) = agent::create_router().split_for_parts();
    let agent_router = agent_router.with_state(agent_service);
    router = router.merge(agent_router);

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

    // environment router (secrets and variables)
    let (environment_router, _) = create_environment_router().split_for_parts();
    let environment_router = environment_router.with_state(api_service.environment_service.clone());
    router = router.merge(environment_router);

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
    let (_, a2a_spec) = create_inbox_a2a_router().split_for_parts();
    let (_, agent_spec) = agent::create_router().split_for_parts();
    let (_, mcp_spec) = create_mcp_router().split_for_parts();
    let (_, internal_spec) = internal::create_router().split_for_parts();
    let (_, encryption_spec) = create_encryption_router().split_for_parts();
    let (_, environment_spec) = create_environment_router().split_for_parts();
    let (_, identity_spec) = create_identity_router().split_for_parts();
    spec.merge(a2a_spec);
    spec.merge(agent_spec);
    spec.merge(mcp_spec);
    spec.merge(internal_spec);
    spec.merge(encryption_spec);
    spec.merge(environment_spec);
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
        (name = "variable", description = "Environment variable management endpoints for storing and retrieving variables"),
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
