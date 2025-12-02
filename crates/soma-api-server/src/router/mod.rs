use axum::Router;
use shared::adapters::openapi::API_VERSION_TAG;
use utoipa::openapi::tag::TagBuilder;
use utoipa::openapi::{Info, OpenApi};

use crate::ApiService;
use bridge::router::create_router as create_bridge_router;
use encryption::router::create_router as create_encryption_router;
use identity::router::create_router as create_identity_router;
use shared::error::CommonError;

pub(crate) mod a2a;
pub(crate) mod environment_variable;
pub(crate) mod internal;
pub(crate) mod secret;
pub(crate) mod task;

pub fn initiaite_api_router(api_service: ApiService) -> Result<Router, CommonError> {
    let mut router = Router::new();

    // let (live_connection_changes_tx, mut live_connection_changes_rx) = tokio::sync::mpsc::channel(10);

    // agent router

    let (agent_router, _) = a2a::create_router().split_for_parts();

    let agent_router = agent_router.with_state(api_service.agent_service);
    router = router.merge(agent_router);

    // task router
    let (task_router, _) = task::create_router().split_for_parts();
    let task_router = task_router.with_state(api_service.task_service);
    router = router.merge(task_router);

    // bridge router
    let (bridge_router, _) = create_bridge_router().split_for_parts();
    let bridge_router = bridge_router.with_state(api_service.bridge_service);
    router = router.merge(bridge_router);

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

    Ok(router)
}

pub fn generate_openapi_spec() -> OpenApi {
    let (_, mut spec) = a2a::create_router().split_for_parts();
    let (_, task_spec) = task::create_router().split_for_parts();
    let (_, bridge_spec) = create_bridge_router().split_for_parts();
    let (_, internal_spec) = internal::create_router().split_for_parts();
    let (_, encryption_spec) = create_encryption_router().split_for_parts();
    let (_, secret_spec) = secret::create_router().split_for_parts();
    let (_, env_var_spec) = environment_variable::create_router().split_for_parts();
    let (_, identity_spec) = create_identity_router().split_for_parts();
    spec.merge(task_spec);
    spec.merge(bridge_spec);
    spec.merge(internal_spec);
    spec.merge(encryption_spec);
    spec.merge(secret_spec);
    spec.merge(env_var_spec);
    spec.merge(identity_spec);

    // Update OpenAPI metadata
    let mut info = Info::new("soma", "An open source AI agent runtime");
    info.version = "v1".to_string();
    spec.info = info;

    // Add tag descriptions
    spec.tags = Some(vec![
        TagBuilder::new()
            .name("task")
            .description(Some("Task management endpoints for creating, listing, and managing tasks and their messages"))
            .build(),
        TagBuilder::new()
            .name("secret")
            .description(Some("Secret management endpoints for storing and retrieving encrypted secrets"))
            .build(),
        TagBuilder::new()
            .name("environment-variable")
            .description(Some("Environment variable management endpoints for storing and retrieving environment variables"))
            .build(),
        TagBuilder::new()
            .name("encryption")
            .description(Some("Encryption key management endpoints for envelope keys, data encryption keys, and aliases"))
            .build(),
        TagBuilder::new()
            .name("bridge")
            .description(Some("Bridge endpoints for managing providers, credentials, functions, and MCP protocol communication"))
            .build(),
        TagBuilder::new()
            .name("_internal")
            .description(Some("Internal endpoints for health checks, runtime configuration, and SDK code generation"))
            .build(),
        TagBuilder::new()
            .name("a2a")
            .description(Some("Agent-to-agent communication endpoints for agent cards, definitions, and JSON-RPC requests"))
            .build(),
        TagBuilder::new()
            .name("identity")
            .description(Some("Identity management endpoints for JWKs (JSON Web Keys) and authentication"))
            .build(),
        TagBuilder::new()
            .name(API_VERSION_TAG)
            .description(Some("API version v1 endpoints"))
            .build(),
    ]);

    spec
}
