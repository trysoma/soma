use std::collections::HashMap;

use tokio::process::Command;

use super::interface::{ClientCtx, SdkClient};

pub struct Typescript {}

impl Typescript {
    pub fn new() -> Self {
        Typescript {}
    }
}

impl SdkClient for Typescript {
    async fn start_dev_server(&self, ctx: ClientCtx) -> Result<(), shared::error::CommonError> {
        // Note: ctx.file_change_tx is available but intentionally unused here.
        // Vite handles HMR automatically, so we don't need to manually handle file changes.

        let mut cmd = Command::new("pnpm");

        cmd.arg("dlx")
            .arg("vite")
            .arg("dev")
            .current_dir(ctx.project_dir.clone());

        // Set the SOMA_SERVER_SOCK environment variable and initial secrets
        let mut env_vars = HashMap::from([
            ("SOMA_SERVER_SOCK".to_string(), ctx.socket_path),
            (
                "RESTATE_RUNTIME_PORT".to_string(),
                ctx.restate_runtime_port.to_string(),
            ),
        ]);

        // Insert all initial secrets into env_vars
        for (key, value) in ctx.initial_secrets {
            env_vars.insert(key, value);
        }

        shared::command::run_child_process(
            "pnpm-dev-server",
            cmd,
            Some(ctx.kill_signal_rx),
            Some(env_vars),
        )
        .await?;

        Ok(())
    }

    async fn build(&self, ctx: ClientCtx) -> Result<(), shared::error::CommonError> {
        let mut cmd = Command::new("pnpm");
        cmd.arg("dlx")
            .arg("vite")
            .arg("build")
            .current_dir(ctx.project_dir.clone());
        shared::command::run_child_process("pnpm-build", cmd, None, None).await?;
        Ok(())
    }
}
