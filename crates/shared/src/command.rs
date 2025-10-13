use anyhow::Result;
use tracing::{error, info};
use std::process::Stdio;
use std::collections::HashMap;
use tokio::{process::Command, sync::oneshot};


pub async fn run_child_process(
    process_name: &str,
    mut process: Command,
    mut kill_signal: Option<oneshot::Receiver<()>>,
    shutdown_complete: Option<oneshot::Sender<()>>,
    extra_env: Option<HashMap<String, String>>,
) -> Result<()> {
    // Put child in its own process group so it doesn't receive SIGINT/SIGTERM directly
    // This allows the parent to handle signals and orchestrate graceful shutdown
    #[cfg(unix)]
    {
        #[allow(unused_imports)]
        use std::os::unix::process::CommandExt;
        process.process_group(0);
    }
    
    let mut child = process
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .kill_on_drop(true);

    if let Some(extra_env) = extra_env {
        for (key, value) in extra_env {
            child.env(key, value);
        }
    }

    
    let mut child =child.spawn()
        .map_err(|e| anyhow::anyhow!("{} process error: {e}", process_name))?;

    info!("🚀 Started {} (pid={:?})", process_name, child.id());

    let status_fut = async {
        let status = child
            .wait()
            .await
            .map_err(|e| anyhow::anyhow!("{} wait error: {e}", process_name))?;

        if !status.success() {
            error!("❌ {} exited with status: {:?}", process_name, status);
            Err(anyhow::anyhow!("{} exited with status: {:?}", process_name, status))
        } else {
            info!("✅ {} exited cleanly: {:?}", process_name, status);
            Ok(())
        }
    };

    // Move sender into the select! so both branches can access it by cloning Option
    let mut shutdown_sender = shutdown_complete;

    match kill_signal.as_mut() {
        Some(rx) => {
            tokio::select! {
                biased;

                _ = rx => {
                    info!("🔪 Kill signal received for {}", process_name);
                    let _ = child.kill().await;
                    let _ = child.wait().await;
                    info!("🛑 {} terminated", process_name);

                    if let Some(tx) = shutdown_sender.take() {
                        let _ = tx.send(());
                    }

                    Ok(())
                }

                result = status_fut => {
                    if let Some(tx) = shutdown_sender.take() {
                        let _ = tx.send(());
                    }

                    result
                }
            }
        }

        None => {
            let result = status_fut.await;

            if let Some(tx) = shutdown_sender.take() {
                let _ = tx.send(());
            }

            result
        }
    }
}