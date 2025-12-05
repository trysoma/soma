use shared::subsystem::SubsystemHandle;

/// Holds handles to all running subsystems
pub struct Subsystems {
    pub sdk_server: Option<SubsystemHandle>,
    pub mcp: Option<SubsystemHandle>,
    pub credential_rotation: Option<SubsystemHandle>,
    pub bridge_client_generation: Option<SubsystemHandle>,
    pub secret_sync: Option<SubsystemHandle>,
    pub environment_variable_sync: Option<SubsystemHandle>,
    pub jwk_init_listener: Option<SubsystemHandle>,
}

impl Subsystems {
    pub async fn shutdown(self) {
        tracing::info!("Shutting down all subsystems...");

        if let Some(handle) = self.sdk_server {
            handle.wait_for_shutdown().await;
        }
        if let Some(handle) = self.mcp {
            handle.wait_for_shutdown().await;
        }
        if let Some(handle) = self.credential_rotation {
            handle.wait_for_shutdown().await;
        }
        if let Some(handle) = self.bridge_client_generation {
            handle.wait_for_shutdown().await;
        }
        if let Some(handle) = self.secret_sync {
            handle.wait_for_shutdown().await;
        }
        if let Some(handle) = self.environment_variable_sync {
            handle.wait_for_shutdown().await;
        }
        if let Some(handle) = self.jwk_init_listener {
            handle.wait_for_shutdown().await;
        }

        tracing::info!("All subsystems shut down successfully");
    }
}
