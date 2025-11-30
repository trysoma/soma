use std::{future::Future, path::PathBuf, pin::Pin};

use shared::{error::CommonError, primitives::WrappedSchema};
use tokio::sync::broadcast;

#[allow(dead_code)]
pub struct Manifest {
    pub functions: Vec<Function>,
}

#[allow(dead_code)]
pub struct Function {
    pub path: PathBuf,
    pub input_schema: WrappedSchema,
    pub output_schema: WrappedSchema,
}

pub struct ClientCtx {
    pub project_dir: PathBuf,
    pub socket_path: String,
    pub restate_service_port: u16,
    pub kill_signal_rx: broadcast::Receiver<()>,
    /// Pre-fetched secrets (decrypted) to inject into SDK as environment variables
    pub initial_secrets: std::collections::HashMap<String, String>,
    /// Pre-fetched environment variables to inject into SDK
    pub initial_environment_variables: std::collections::HashMap<String, String>,
}

#[allow(dead_code)]
pub struct DevServerHandle {
    pub dev_server_fut: Pin<Box<dyn Future<Output = Result<(), CommonError>> + Send>>,
}

#[allow(async_fn_in_trait)]
pub trait SdkClient {
    async fn start_dev_server(&self, ctx: ClientCtx) -> Result<(), CommonError>;
    #[allow(dead_code)]
    async fn build(&self, ctx: ClientCtx) -> Result<(), CommonError>;
}
