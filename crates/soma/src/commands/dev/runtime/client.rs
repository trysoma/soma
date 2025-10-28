use std::{path::PathBuf, pin::Pin};

use shared::{error::CommonError, primitives::WrappedSchema};
use tokio::sync::oneshot;

pub struct Manifest {
    pub functions: Vec<Function>,
}

pub struct Function {
    pub path: PathBuf,
    pub input_schema: WrappedSchema,
    pub output_schema: WrappedSchema,
}

pub struct ClientCtx {
    pub project_dir: PathBuf,
}

pub struct DevServerHandle {
    pub kill_signal_tx: oneshot::Sender<()>,
    pub shutdown_complete_rx: oneshot::Receiver<()>,
    pub dev_server_fut: Pin<Box<dyn Future<Output = Result<(), CommonError>> + Send>>,
}

pub trait SdkClient {
    async fn start_dev_server(&self, ctx: ClientCtx) -> Result<DevServerHandle, CommonError>;
    async fn build(&self, ctx: ClientCtx) -> Result<(), CommonError>;
}