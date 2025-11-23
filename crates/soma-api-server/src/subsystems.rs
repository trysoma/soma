use shared::subsystem::SubsystemHandle;

/// Holds handles to all running subsystems
pub struct Subsystems {
    pub restate: Option<SubsystemHandle>,
    pub file_watcher: Option<SubsystemHandle>,
    pub bridge_sync: Option<SubsystemHandle>,
    pub sdk_server: Option<SubsystemHandle>,
    pub sdk_sync: Option<SubsystemHandle>,
    pub mcp: Option<SubsystemHandle>,
    pub credential_rotation: Option<SubsystemHandle>,
    pub bridge_codegen: Option<SubsystemHandle>,
}

impl Subsystems {
    pub async fn shutdown(self) {
        tracing::info!("Shutting down all subsystems...");

        if let Some(handle) = self.restate {
            handle.wait_for_shutdown().await;
        }
        if let Some(handle) = self.file_watcher {
            handle.wait_for_shutdown().await;
        }
        if let Some(handle) = self.bridge_sync {
            handle.wait_for_shutdown().await;
        }
        if let Some(handle) = self.sdk_server {
            handle.wait_for_shutdown().await;
        }
        if let Some(handle) = self.sdk_sync {
            handle.wait_for_shutdown().await;
        }
        if let Some(handle) = self.mcp {
            handle.wait_for_shutdown().await;
        }
        if let Some(handle) = self.credential_rotation {
            handle.wait_for_shutdown().await;
        }
        if let Some(handle) = self.bridge_codegen {
            handle.wait_for_shutdown().await;
        }

        tracing::info!("All subsystems shut down successfully");
    }
}
