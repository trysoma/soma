use std::collections::HashMap;

use futures::future;
use super::interface::{ClientCtx, SdkClient};

pub struct Typescript {}

impl Typescript {
    pub fn new() -> Self {
        Typescript {}
    }
}

impl SdkClient for Typescript {
    async fn start_dev_server(&self, ctx: ClientCtx) -> Result<(), shared::error::CommonError> {
        // Note: Vite handles HMR automatically, so we don't need to manually handle file changes.

        // Set the SOMA_SERVER_SOCK environment variable and initial secrets/env vars
        let mut env_vars = HashMap::from([
            ("SOMA_SERVER_SOCK".to_string(), ctx.socket_path),
            (
                "RESTATE_SERVICE_PORT".to_string(),
                ctx.restate_service_port.to_string(),
            ),
        ]);

        // Insert all initial secrets into env_vars
        for (key, value) in ctx.initial_secrets {
            env_vars.insert(key, value);
        }

        // Insert all initial environment variables into env_vars
        for (key, value) in ctx.initial_environment_variables {
            env_vars.insert(key, value);
        }

        // Use process manager to start the Vite dev server
        ctx.process_manager.start_process("pnpm-dev-server", shared::process_manager::ProcessConfig {
            script: "pnpm".to_string(),
            args: vec!["dlx".to_string(), "vite".to_string(), "dev".to_string()],
            cwd: Some(ctx.project_dir.clone()),
            env: env_vars,
            health_check: None,
            on_terminal_stop: shared::process_manager::OnTerminalStop::TriggerShutdown,
            on_stop: shared::process_manager::OnStop::Restart(shared::process_manager::RestartConfig {
                max_restarts: 10,
                restart_delay: 2000,
            }),
            shutdown_priority: 7, // Lower than SDK server thread (8) so it shuts down first
            follow_logs: false,
            on_shutdown_triggered: None,
            on_shutdown_complete: None,
        }).await?;

        // Wait indefinitely - the process manager will handle shutdown by aborting this thread
        future::pending::<()>().await;

        Ok(())
    }

    async fn build(&self, ctx: ClientCtx) -> Result<(), shared::error::CommonError> {
        // Start with initial secrets and environment variables for consistent build environment
        let mut env_vars = HashMap::new();
        for (key, value) in ctx.initial_secrets {
            env_vars.insert(key, value);
        }
        for (key, value) in ctx.initial_environment_variables {
            env_vars.insert(key, value);
        }

        // Use process manager to run the build process
        ctx.process_manager.start_process("pnpm-build", shared::process_manager::ProcessConfig {
            script: "pnpm".to_string(),
            args: vec!["dlx".to_string(), "vite".to_string(), "build".to_string()],
            cwd: Some(ctx.project_dir.clone()),
            env: env_vars,
            health_check: None,
            on_terminal_stop: shared::process_manager::OnTerminalStop::Ignore,
            on_stop: shared::process_manager::OnStop::Nothing, // Build is one-shot, don't restart
            shutdown_priority: 1,
            follow_logs: true,
            on_shutdown_triggered: None,
            on_shutdown_complete: None,
        }).await?;

        Ok(())
    }
}
