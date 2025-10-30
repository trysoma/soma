use std::path::PathBuf;
use std::collections::HashMap;

use tokio::process::Command;
use tokio::sync::oneshot;

use crate::commands::dev::runtime::client::{ClientCtx, DevServerHandle, SdkClient};

pub struct Typescript {}

impl Typescript {
    pub fn new() -> Self {
        Typescript {}
    }
}

impl SdkClient for Typescript {
    async fn start_dev_server(&self, ctx: ClientCtx) -> Result<DevServerHandle, shared::error::CommonError> {

        let mut cmd = Command::new("pnpm");

        cmd
            .arg("dlx")
            .arg("vite")
            .arg("dev")
            .current_dir(ctx.project_dir.clone());

        let (kill_signal_tx, kill_signal_rx) = tokio::sync::oneshot::channel::<()>();
        let (shutdown_complete_tx, shutdown_complete_rx) = tokio::sync::oneshot::channel::<()>();

        // Set the SOMA_SERVER_SOCK environment variable
        let env_vars = HashMap::from([
            ("SOMA_SERVER_SOCK".to_string(), ctx.socket_path),
        ]);

        let dev_server_fut = shared::command::run_child_process("pnpm-dev-server", cmd, Some(kill_signal_rx), Some(shutdown_complete_tx), Some(env_vars));

        Ok(DevServerHandle {
            kill_signal_tx,
            shutdown_complete_rx,
            dev_server_fut: Box::pin(dev_server_fut),
        })
    }

    async fn build(&self, ctx: ClientCtx) -> Result<(), shared::error::CommonError> {
        let mut cmd = Command::new("pnpm");
        cmd.arg("dlx")
            .arg("vite")
            .arg("build")
            .current_dir(ctx.project_dir.clone());
        shared::command::run_child_process("pnpm-build", cmd, None, None, None).await?;
        Ok(())
    }

}
