use crate::error::CommonError;
use std::collections::HashMap;
use std::process::Stdio;
use tokio::{
    process::Command,
    sync::{broadcast, oneshot},
};
use tracing::{error, info};

pub async fn run_child_process(
    process_name: &str,
    process: Command,
    kill_signal: Option<broadcast::Receiver<()>>,
    extra_env: Option<HashMap<String, String>>,
) -> Result<(), CommonError> {
    run_child_process_with_env_options(process_name, process, kill_signal, extra_env, false).await
}

/// Run child process with an option to clear the inherited environment
/// When `clear_env` is true, only the provided `extra_env` variables will be set
/// along with essential system variables like PATH, HOME, etc.
pub async fn run_child_process_with_env_options(
    process_name: &str,
    mut process: Command,
    kill_signal: Option<broadcast::Receiver<()>>,
    extra_env: Option<HashMap<String, String>>,
    clear_env: bool,
) -> Result<(), CommonError> {
    // Put child in its own process group so it doesn't receive SIGINT/SIGTERM directly
    // This allows the parent to handle signals and orchestrate graceful shutdown
    #[cfg(unix)]
    {
        #[allow(unused_imports)]
        use std::os::unix::process::CommandExt;
        process.process_group(0);
    }

    let process = process
        .stdin(Stdio::null()) // Prevent readline/TTY errors when process is killed
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .kill_on_drop(true);

    // If clear_env is set, clear all inherited environment variables first
    // Then add back essential system variables and user-provided env vars
    let process = if clear_env {
        // Clear all inherited environment variables
        let mut process = process.env_clear();

        // Add back essential system variables from the host
        // These are needed for basic process execution
        let essential_vars = [
            "PATH",
            "HOME",
            "USER",
            "SHELL",
            "LANG",
            "LC_ALL",
            "TERM",
            "TMPDIR",
            "TMP",
            "TEMP",
            // Node.js/npm-specific variables
            "NODE_PATH",
            "NPM_CONFIG_PREFIX",
            // macOS-specific
            "DYLD_LIBRARY_PATH",
            "DYLD_FALLBACK_LIBRARY_PATH",
            // Linux-specific
            "LD_LIBRARY_PATH",
        ];

        for var in essential_vars {
            if let Ok(value) = std::env::var(var) {
                process = process.env(var, value);
            }
        }

        process
    } else {
        process
    };

    let mut child = if let Some(extra_env) = extra_env {
        let process = extra_env
            .into_iter()
            .fold(process, |proc, (key, value)| proc.env(key, value));
        process.spawn()
    } else {
        process.spawn()
    }
    .map_err(|e| anyhow::anyhow!("{process_name} process error: {e}"))?;

    info!("🚀 Started {} (pid={:?})", process_name, child.id());

    let (status_tx, status_rx) = oneshot::channel::<Result<(), CommonError>>();
    let process_name_clone = process_name.to_string();

    // Always spawn a monitoring task, whether we have a kill signal or not
    tokio::spawn(async move {
        if let Some(mut kill_signal_rx) = kill_signal {
            // Wait for kill signal
            let _ = kill_signal_rx.recv().await;
            info!("🔪 Kill signal received for {}", process_name_clone);

            // Kill the entire process group to ensure child processes are terminated
            #[cfg(unix)]
            if let Some(pid) = child.id() {
                use nix::sys::signal::{Signal, kill};
                use nix::unistd::Pid;

                // Send SIGTERM to the entire process group (negative PID)
                let pgid = Pid::from_raw(-(pid as i32));
                info!("🔪 Sending SIGTERM to process group {}", pid);
                let _ = kill(pgid, Signal::SIGTERM);

                // Wait a bit for graceful shutdown
                let wait_result =
                    tokio::time::timeout(std::time::Duration::from_secs(30), child.wait()).await;

                match wait_result {
                    Ok(Ok(status)) => {
                        info!("🛑 {} exited with {:?}", process_name_clone, status);
                        // Since we sent SIGTERM ourselves, any exit should be treated as clean
                        // The process may exit with a non-zero code when terminated by SIGTERM,
                        // but that's expected behavior when we intentionally kill it
                        info!("✅ {} terminated cleanly by SIGTERM", process_name_clone);
                    }
                    Ok(Err(err)) => {
                        error!("❌ Failed to wait for {}: {:?}", process_name_clone, err);
                        let _ = status_tx.send(Err(CommonError::Unknown(anyhow::anyhow!(
                            "{process_name_clone} exited with error: {err:?}"
                        ))));
                        return;
                    }
                    Err(_) => {
                        // Timeout expired — escalate to SIGKILL
                        info!(
                            "⏰ Timeout waiting for {}, sending SIGKILL",
                            process_name_clone
                        );
                        let _ = kill(pgid, Signal::SIGKILL);

                        match child.wait().await {
                            Ok(status) => info!("🧨 {} killed: {:?}", process_name_clone, status),
                            Err(err) => {
                                error!("❌ Failed to reap {}: {:?}", process_name_clone, err);
                                let _ = status_tx.send(Err(CommonError::Unknown(anyhow::anyhow!(
                                    "{process_name_clone} exited with error: {err:?}"
                                ))));
                                return;
                            }
                        }
                    }
                }
            }

            #[cfg(not(unix))]
            {
                let _ = child.kill().await;
                match child.wait().await {
                    Ok(status) => info!("🛑 {} terminated: {:?}", process_name_clone, status),
                    Err(err) => {
                        error!("❌ Failed to wait for {}: {:?}", process_name_clone, err);
                        let _ = status_tx.send(Err(CommonError::Unknown(anyhow::anyhow!(
                            "{process_name_clone} exited with error: {err:?}"
                        ))));
                        return;
                    }
                }
            }
        } else {
            // No kill signal - just wait for process to exit naturally
            match child.wait().await {
                Ok(status) => {
                    if status.success() {
                        info!("✅ {} exited successfully", process_name_clone);
                    } else {
                        error!("❌ {} exited with status: {:?}", process_name_clone, status);
                        let _ = status_tx.send(Err(CommonError::Unknown(anyhow::anyhow!(
                            "{process_name_clone} exited with non-zero status: {status:?}"
                        ))));
                        return;
                    }
                }
                Err(err) => {
                    error!("❌ Failed to wait for {}: {:?}", process_name_clone, err);
                    let _ = status_tx.send(Err(CommonError::Unknown(anyhow::anyhow!(
                        "{process_name_clone} wait error: {err:?}"
                    ))));
                    return;
                }
            }
        }

        let _ = status_tx.send(Ok(()));
    });

    let result = status_rx.await;

    match result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(CommonError::Unknown(anyhow::anyhow!(
            "Failed to get status for {process_name}"
        ))),
    }
}
