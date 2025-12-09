//! Router module for the bridge service
//!
//! This module contains all HTTP route handlers organized into submodules:
//! - `provider`: Provider-related endpoints (CRUD, credentials, functions)
//! - `mcp_server_instance`: MCP server instance management and protocol endpoints

mod mcp_server_instance;
mod provider;

use crate::logic::{OnConfigChangeTx, process_credential_rotations_with_window};
use crate::repository::Repository;
use encryption::logic::crypto_services::CryptoCache;
use rmcp::transport::sse_server::SseServerTransport;
use shared::error::CommonError;
use std::sync::Arc;
use std::time::Duration;
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
        // MCP protocol endpoints
        .routes(routes!(mcp_server_instance::mcp_sse))
        .routes(routes!(mcp_server_instance::mcp_message))
}

/// Inner state for the bridge service containing all shared dependencies
pub struct BridgeServiceInner {
    pub repository: Repository,
    pub on_config_change_tx: OnConfigChangeTx,
    pub encryption_service: CryptoCache,
    pub mcp_sessions: rmcp::transport::sse_server::TxStore,
    pub mcp_transport_tx: tokio::sync::mpsc::UnboundedSender<SseServerTransport>,
    pub mcp_sse_ping_interval: Duration,
}

impl BridgeServiceInner {
    pub fn new(
        repository: Repository,
        on_config_change_tx: OnConfigChangeTx,
        encryption_service: CryptoCache,
        mcp_transport_tx: tokio::sync::mpsc::UnboundedSender<SseServerTransport>,
        mcp_sse_ping_interval: Duration,
    ) -> Self {
        Self {
            repository,
            on_config_change_tx,
            encryption_service,
            mcp_sessions: Default::default(),
            mcp_transport_tx,
            mcp_sse_ping_interval,
        }
    }
}

/// Bridge service state shared across all routes
#[derive(Clone)]
pub struct BridgeService(pub Arc<BridgeServiceInner>);

impl BridgeService {
    pub async fn new(
        repository: Repository,
        on_config_change_tx: OnConfigChangeTx,
        encryption_service: CryptoCache,
        mcp_transport_tx: tokio::sync::mpsc::UnboundedSender<SseServerTransport>,
        mcp_sse_ping_interval: Duration,
    ) -> Result<Self, CommonError> {
        let inner = BridgeServiceInner::new(
            repository,
            on_config_change_tx,
            encryption_service,
            mcp_transport_tx,
            mcp_sse_ping_interval,
        );

        // Run initial credential rotation check for expired and soon-to-expire credentials (30 min window)
        info!("Running initial credential rotation check...");
        process_credential_rotations_with_window(
            &inner.repository,
            &inner.on_config_change_tx,
            &inner.encryption_service,
            30,
        )
        .await?;
        info!("Initial credential rotation check complete");

        Ok(Self(Arc::new(inner)))
    }

    pub fn repository(&self) -> &Repository {
        &self.0.repository
    }

    pub fn on_config_change_tx(&self) -> &OnConfigChangeTx {
        &self.0.on_config_change_tx
    }

    pub fn encryption_service(&self) -> &CryptoCache {
        &self.0.encryption_service
    }

    pub fn mcp_transport_tx(&self) -> &tokio::sync::mpsc::UnboundedSender<SseServerTransport> {
        &self.0.mcp_transport_tx
    }

    pub fn mcp_sse_ping_interval(&self) -> &Duration {
        &self.0.mcp_sse_ping_interval
    }

    pub fn mcp_sessions(&self) -> &rmcp::transport::sse_server::TxStore {
        &self.0.mcp_sessions
    }
}
