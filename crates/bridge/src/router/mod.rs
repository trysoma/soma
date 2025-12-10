//! Router module for the bridge service
//!
//! This module contains all HTTP route handlers organized into submodules:
//! - `provider`: Provider-related endpoints (CRUD, credentials, functions)
//! - `mcp_server_instance`: MCP server instance management and protocol endpoints

mod mcp_server_instance;
mod provider;

use crate::logic::mcp::BridgeMcpService;
use crate::logic::{OnConfigChangeTx, process_credential_rotations_with_window};
use crate::repository::Repository;
use encryption::logic::crypto_services::CryptoCache;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::StreamableHttpService;
use shared::error::CommonError;
use tracing::info;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub const PATH_PREFIX: &str = "/api";
pub const API_VERSION_1: &str = "v1";
pub const SERVICE_ROUTE_KEY: &str = "bridge";

/// Creates the bridge API router with all endpoints
pub fn create_router() -> OpenApiRouter<BridgeService> {
    OpenApiRouter::new()
        // Provider endpoints
        .routes(routes!(provider::route_list_available_providers))
        .routes(routes!(
            provider::route_encrypt_resource_server_configuration
        ))
        .routes(routes!(
            provider::route_encrypt_user_credential_configuration
        ))
        .routes(routes!(provider::route_create_resource_server_credential))
        .routes(routes!(provider::route_create_user_credential))
        .routes(routes!(provider::route_start_user_credential_brokering))
        .routes(routes!(provider::generic_oauth_callback))
        .routes(routes!(provider::route_create_provider_instance))
        .routes(routes!(provider::route_update_provider_instance))
        .routes(routes!(provider::route_delete_provider_instance))
        .routes(routes!(provider::route_get_provider_instance))
        .routes(routes!(provider::route_list_provider_instances))
        .routes(routes!(
            provider::route_list_provider_instances_grouped_by_function
        ))
        .routes(routes!(provider::route_enable_function))
        .routes(routes!(provider::route_disable_function))
        .routes(routes!(provider::route_invoke_function))
        .routes(routes!(provider::route_list_function_instances))
        .routes(routes!(provider::route_get_function_instances_openapi_spec))
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
            mcp_server_instance::route_update_mcp_server_instance_function
        ))
        .routes(routes!(
            mcp_server_instance::route_remove_mcp_server_instance_function
        ))
        
}

/// Inner state for the bridge service containing all shared dependencies
#[derive(Clone)]
pub struct BridgeService {
    pub repository: Repository,
    pub on_config_change_tx: OnConfigChangeTx,
    pub encryption_service: CryptoCache,
    pub mcp_service: StreamableHttpService<BridgeMcpService, LocalSessionManager>,
}

impl BridgeService {
    pub async fn new(
        repository: Repository,
        on_config_change_tx: OnConfigChangeTx,
        encryption_service: CryptoCache,
        mcp_service: StreamableHttpService<BridgeMcpService, LocalSessionManager>,
    ) -> Result<Self, CommonError> {
        // Run initial credential rotation check for expired and soon-to-expire credentials (30 min window)
        info!("Running initial credential rotation check...");
        process_credential_rotations_with_window(
            &repository,
            &on_config_change_tx,
            &encryption_service,
            30,
        )
        .await?;
        info!("Initial credential rotation check complete");

        Ok(Self {
            repository,
            on_config_change_tx,
            encryption_service,
            mcp_service,
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

    pub fn mcp_service(&self) -> &StreamableHttpService<BridgeMcpService, LocalSessionManager> {
        &self.mcp_service
    }
}
