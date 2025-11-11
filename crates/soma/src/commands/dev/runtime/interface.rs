use std::{path::PathBuf, pin::Pin, future::Future, sync::Arc};

use shared::{error::CommonError, primitives::WrappedSchema};

use crate::commands::dev::project_file_watcher::FileChangeTx;

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
    pub socket_path: String,
    pub restate_runtime_port: u16,
    pub file_change_tx: Arc<FileChangeTx>,
    pub kill_signal_rx: tokio::sync::broadcast::Receiver<()>,
}

pub struct DevServerHandle {
    pub dev_server_fut: Pin<Box<dyn Future<Output = Result<(), CommonError>> + Send>>,
}

pub trait SdkClient {
    async fn start_dev_server(&self, ctx: ClientCtx) -> Result<(), CommonError>;
    async fn build(&self, ctx: ClientCtx) -> Result<(), CommonError>;
}