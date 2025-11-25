use tokio::sync::{broadcast, oneshot};
use tracing::{error, info};

/// A helper for managing subsystem lifecycle with graceful shutdown
pub struct SubsystemHandle {
    name: String,
    shutdown_complete_rx: oneshot::Receiver<()>,
}

impl SubsystemHandle {
    /// Create a new subsystem handle
    pub fn new(name: impl Into<String>) -> (Self, SubsystemShutdownSignal) {
        let (shutdown_complete_tx, shutdown_complete_rx) = oneshot::channel();
        let name = name.into();

        (
            Self {
                name: name.clone(),
                shutdown_complete_rx,
            },
            SubsystemShutdownSignal {
                name,
                shutdown_complete_tx,
            },
        )
    }

    /// Wait for the subsystem to complete shutdown
    pub async fn wait_for_shutdown(self) {
        match self.shutdown_complete_rx.await {
            Ok(()) => info!("{} subsystem stopped gracefully", self.name),
            Err(_) => error!(
                "{} subsystem stopped without signaling completion",
                self.name
            ),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Signal to be sent by subsystem when it completes shutdown
pub struct SubsystemShutdownSignal {
    name: String,
    shutdown_complete_tx: oneshot::Sender<()>,
}

impl SubsystemShutdownSignal {
    /// Signal that the subsystem has completed shutdown
    pub fn signal(self) {
        let _ = self.shutdown_complete_tx.send(());
    }

    /// Signal with a custom message
    pub fn signal_with_message(self, message: &str) {
        info!("{}: {}", self.name, message);
        let _ = self.shutdown_complete_tx.send(());
    }
}

/// Spawn a subsystem task with automatic shutdown handling
pub fn spawn_subsystem<F>(
    name: impl Into<String>,
    _system_shutdown_rx: broadcast::Receiver<()>,
    task: F,
) -> SubsystemHandle
where
    F: futures::Future<Output = Result<(), crate::error::CommonError>> + Send + 'static,
{
    let (handle, signal) = SubsystemHandle::new(name);
    let subsystem_name = handle.name().to_string();

    tokio::spawn(async move {
        match task.await {
            Ok(()) => {
                signal.signal_with_message("stopped gracefully");
            }
            Err(e) => {
                error!("{} stopped with error: {:?}", subsystem_name, e);
                signal.signal();
            }
        }
    });

    handle
}

/// Spawn a subsystem task that can be manually controlled
pub fn spawn_subsystem_manual<F>(name: impl Into<String>, task: F) -> SubsystemHandle
where
    F: futures::Future<Output = ()> + Send + 'static,
{
    let (handle, signal) = SubsystemHandle::new(name);

    tokio::spawn(async move {
        task.await;
        signal.signal();
    });

    handle
}
