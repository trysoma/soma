//! Router module for the mcp service
//!
//! This module contains all HTTP route handlers organized into submodules:
//! - `tool_group`: Tool group-related endpoints (CRUD, credentials, tools)
//! - `mcp_server_instance`: MCP server instance management and protocol endpoints
//! - `tool`: Tool registration and management endpoints

mod mcp_server_instance;
mod tool_group;
mod tool;

use std::sync::Arc;

use crate::logic::mcp::McpServerService;
use crate::logic::{OnConfigChangeTx, process_credential_rotations_with_window};
use crate::repository::Repository;
use encryption::logic::crypto_services::CryptoCache;
use identity::logic::auth_client::AuthClient;
use rmcp::transport::streamable_http_server::StreamableHttpService;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use shared::error::CommonError;
use tracing::{debug, trace};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub const PATH_PREFIX: &str = "/api";
pub const API_VERSION_1: &str = "v1";
pub const SERVICE_ROUTE_KEY: &str = "tool";

/// Creates the tool API router with all endpoints
pub fn create_router() -> OpenApiRouter<ToolService> {
    OpenApiRouter::new()
        // Tool group endpoints
        .routes(routes!(tool_group::route_list_available_tool_groups))
        .routes(routes!(
            tool_group::route_encrypt_resource_server_configuration
        ))
        .routes(routes!(
            tool_group::route_encrypt_user_credential_configuration
        ))
        .routes(routes!(tool_group::route_create_resource_server_credential))
        .routes(routes!(tool_group::route_create_user_credential))
        .routes(routes!(tool_group::route_start_user_credential_brokering))
        .routes(routes!(tool_group::generic_oauth_callback))
        .routes(routes!(tool_group::route_create_tool_group))
        .routes(routes!(tool_group::route_update_tool_group))
        .routes(routes!(tool_group::route_delete_tool_group))
        .routes(routes!(tool_group::route_get_tool_group))
        .routes(routes!(tool_group::route_list_tool_groups))
        .routes(routes!(
            tool_group::route_list_tool_groups_grouped_by_function
        ))
        .routes(routes!(tool_group::route_enable_tool))
        .routes(routes!(tool_group::route_disable_tool))
        .routes(routes!(tool_group::route_invoke_tool))
        .routes(routes!(tool_group::route_list_tools))
        .routes(routes!(tool_group::route_get_tools_openapi_spec))
        // MCP server instance endpoints
        .routes(routes!(
            mcp_server_instance::route_create_mcp_server_instance
        ))
        .routes(routes!(mcp_server_instance::route_get_mcp_server_instance))
        .routes(routes!(
            mcp_server_instance::route_update_mcp_server_instance
        ))
        .routes(routes!(
            mcp_server_instance::route_delete_mcp_server_instance
        ))
        .routes(routes!(
            mcp_server_instance::route_list_mcp_server_instances
        ))
        .routes(routes!(
            mcp_server_instance::route_add_mcp_server_instance_function
        ))
        .routes(routes!(
            mcp_server_instance::route_update_mcp_server_instance_tool
        ))
        .routes(routes!(
            mcp_server_instance::route_remove_mcp_server_instance_function
        ))
        // Tool registration endpoints
        .routes(routes!(tool::route_register_tool))
        .routes(routes!(tool::route_list_tools))
        .routes(routes!(tool::route_get_tool))
        .routes(routes!(tool::route_deregister_tool))
        .routes(routes!(tool::route_create_tool_alias))
        .routes(routes!(tool::route_list_tool_aliases))
        .routes(routes!(tool::route_get_tool_by_alias))
        .routes(routes!(tool::route_update_tool_alias))
        .routes(routes!(tool::route_delete_tool_alias))
}

/// Inner state for the tool service containing all shared dependencies
#[derive(Clone)]
pub struct ToolService {
    pub repository: Repository,
    pub on_config_change_tx: OnConfigChangeTx,
    pub encryption_service: CryptoCache,
    pub mcp_service: StreamableHttpService<McpServerService, LocalSessionManager>,
    pub auth_client: Arc<AuthClient>,
}

// Deprecated: Use ToolService instead
pub type McpService = ToolService;

impl ToolService {
    pub async fn new(
        repository: Repository,
        on_config_change_tx: OnConfigChangeTx,
        encryption_service: CryptoCache,
        mcp_service: StreamableHttpService<McpServerService, LocalSessionManager>,
        auth_client: Arc<AuthClient>,
    ) -> Result<Self, CommonError> {
        // Run initial credential rotation check for expired and soon-to-expire credentials (30 min window)
        debug!("Running initial credential rotation check");
        process_credential_rotations_with_window(
            &repository,
            &on_config_change_tx,
            &encryption_service,
            30,
        )
        .await?;
        trace!("Initial credential rotation check complete");

        Ok(Self {
            repository,
            on_config_change_tx,
            encryption_service,
            mcp_service,
            auth_client,
        })
    }

    pub fn repository(&self) -> &Repository {
        &self.repository
    }

    pub fn on_config_change_tx(&self) -> &OnConfigChangeTx {
        &self.on_config_change_tx
    }

    pub fn encryption_service(&self) -> &CryptoCache {
        &self.encryption_service
    }

    pub fn mcp_service(&self) -> &StreamableHttpService<McpServerService, LocalSessionManager> {
        &self.mcp_service
    }

    /// Get a reference to the auth client (cheap to clone since it only contains Arcs)
    pub fn auth_client(&self) -> &AuthClient {
        &self.auth_client
    }
}
