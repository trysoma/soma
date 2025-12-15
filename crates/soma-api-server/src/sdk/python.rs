use std::collections::HashMap;

use tokio::process::Command;

use super::interface::{ClientCtx, SdkClient};

pub struct Python {}

impl Python {
    pub fn new() -> Self {
        Python {}
    }
}

impl SdkClient for Python {
    async fn start_dev_server(&self, ctx: ClientCtx) -> Result<(), shared::error::CommonError> {
        // First generate standalone.py, then run it with --watch mode
        // The --watch mode will start the server and watch for changes
        let mut cmd = Command::new("uv");

        cmd.arg("run")
            .arg("python")
            .arg("-m")
            .arg("trysoma_sdk.standalone")
            .arg("--watch")
            .arg(".")
            .current_dir(ctx.project_dir.clone());

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

        // Use process manager to start the Python dev server
        let mut process_manager = ctx.process_manager.lock().await;
        process_manager.start_process("python-dev-server", shared::process_manager::ProcessConfig {
            script: "uv".to_string(),
            args: vec![
                "run".to_string(),
                "python".to_string(),
                "-m".to_string(),
                "trysoma_sdk.standalone".to_string(),
                "--watch".to_string(),
                ".".to_string(),
            ],
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

        // Wait for shutdown - the process manager will handle cleanup
        // Drop the lock so other threads can access the process manager
        drop(process_manager);
        // Wait for process manager shutdown - this will return when shutdown is triggered
        ctx.process_manager.lock().await.wait_for_shutdown().await;

        Ok(())
    }

    async fn build(&self, ctx: ClientCtx) -> Result<(), shared::error::CommonError> {
        // For Python, "build" means generating the standalone.py file
        let mut cmd = Command::new("uv");
        cmd.arg("run")
            .arg("python")
            .arg("-m")
            .arg("trysoma_sdk.standalone")
            .arg("--dev")
            .arg(".")
            .current_dir(ctx.project_dir.clone());

        // Start with initial secrets and environment variables for consistent build environment
        let mut env_vars = HashMap::new();
        for (key, value) in ctx.initial_secrets {
            env_vars.insert(key, value);
        }
        for (key, value) in ctx.initial_environment_variables {
            env_vars.insert(key, value);
        }

        // Run with clear_env=true for consistent build environment
        shared::command::run_child_process_with_env_options(
            "python-build",
            cmd,
            None,
            Some(env_vars),
            true, // Clear inherited environment
        )
        .await?;

        Ok(())
    }
}
