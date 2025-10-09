use anyhow::Result;
use std::process::Stdio;
use tokio::process::Command;


pub async fn run_child_process(process_name: &str, mut process: Command) -> Result<()> {
    let mut cmd = process
        .env_clear()
        .env("PATH", "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| anyhow::anyhow!("{} process error: {e}", process_name))?;

    return match cmd.wait().await {
        Ok(status) => {
            if !status.success() {
                tracing::error!("{} exited with status: {status}", process_name);

                return Err(anyhow::anyhow!("{} exited with status: {status}", process_name));
            }
            Ok(())
        }
        Err(e) => {
            tracing::error!("{} process error: {e}", process_name);
            Err(anyhow::anyhow!("{} process error: {e}", process_name))
        },
    };
}