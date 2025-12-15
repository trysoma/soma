use anyhow::Context;
use indicatif::ProgressBar;
use shared::command::run_child_process;
use shared::error::CommonError;
use shared::subsystem::SubsystemHandle;
use std::time::Duration;
use std::{collections::HashMap, fs};
use std::path::PathBuf;
use tokio::process::Command;
use tokio::sync::broadcast;
use tokio::time::sleep;
use tracing::{debug, error, info, trace};
use reqwest::Client;

use shared::port::is_port_in_use;
use soma_api_server::restate::{
    RestateServerLocalParams, RestateServerParams, RestateServerRemoteParams,
};
use std::env::var;
use crate::process_manager::{CustomProcessManager, OnStop, ProcessConfig};

/// The embedded restate-server binary for the current platform
/// This is included at compile time from the binary downloaded during build
/// We use a macro to conditionally include the binary based on platform
macro_rules! include_restate_binary {
    ($target:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/bin/",
            $target,
            "/restate-server"
        ))
    };
}

// Include the appropriate binary for the current platform
// If the file doesn't exist, we'll get a compile error which is expected
// In that case, ensure_restate_binary() will try to use the system binary
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
const RESTATE_BINARY: Option<&[u8]> = Some(include_restate_binary!("x86_64-unknown-linux-gnu"));

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
const RESTATE_BINARY: Option<&[u8]> = Some(include_restate_binary!("aarch64-unknown-linux-gnu"));

#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
const RESTATE_BINARY: Option<&[u8]> = Some(include_restate_binary!("x86_64-apple-darwin"));

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
const RESTATE_BINARY: Option<&[u8]> = Some(include_restate_binary!("aarch64-apple-darwin"));

// For unsupported platforms, we won't have an embedded binary
#[cfg(not(any(
    all(target_os = "linux", target_arch = "x86_64"),
    all(target_os = "linux", target_arch = "aarch64"),
    all(target_os = "macos", target_arch = "x86_64"),
    all(target_os = "macos", target_arch = "aarch64")
)))]
const RESTATE_BINARY: Option<&[u8]> = None;

/// Get the directory where soma stores its data
pub fn get_soma_data_dir() -> Result<PathBuf, CommonError> {
    // Try to use user's home directory first
    // if let Some(home) = dirs::home_dir() {
    //     let soma_dir = home.join(".soma");
    //     return Ok(soma_dir);
    // }

    // Fallback to /var/lib/soma if no home directory
    Ok(PathBuf::from("/var/lib/soma"))
}

/// Get the path where the restate-server binary should be installed
pub fn get_restate_binary_path() -> Result<PathBuf, CommonError> {
    let soma_dir = get_soma_data_dir()?;
    let bin_dir = soma_dir.join("bin");
    Ok(bin_dir.join("restate-server"))
}

/// Check if restate-server is available in the system PATH
pub fn is_restate_in_path() -> bool {
    std::process::Command::new("which")
        .arg("restate-server")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Install the bundled restate-server binary to the soma data directory
pub fn install_bundled_restate() -> Result<PathBuf, CommonError> {
    // Check if we have an embedded binary for this platform
    let binary_data = RESTATE_BINARY.ok_or_else(|| {
        CommonError::Unknown(anyhow::anyhow!(
            "No bundled restate-server binary available for this platform"
        ))
    })?;

    let binary_path = get_restate_binary_path()?;

    // Create parent directory if it doesn't exist
    if let Some(parent) = binary_path.parent() {
        fs::create_dir_all(parent).context("Failed to create soma bin directory")?;
    }

    // Check if binary already exists
    if binary_path.exists() {
        tracing::info!("restate-server already installed at {:?}", binary_path);
        return Ok(binary_path);
    }

    tracing::info!("Installing bundled restate-server to {:?}", binary_path);

    // Write the embedded binary to the file
    fs::write(&binary_path, binary_data).context("Failed to write restate-server binary")?;

    // Make the binary executable (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&binary_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&binary_path, perms)
            .context("Failed to set restate-server binary permissions")?;
    }

    tracing::info!("Successfully installed restate-server binary");
    Ok(binary_path)
}

/// Get the path to the restate-server binary, installing if necessary
///
/// This function checks in the following order:
/// 1. If `restate-server` is in PATH, use that
/// 2. If bundled binary is already installed in ~/.soma/bin, use that
/// 3. Install the bundled binary to ~/.soma/bin and use that
/// 4. If no bundled binary available, fall back to "restate-server" command
pub fn ensure_restate_binary() -> Result<String, CommonError> {
    // Check if restate-server exists in PATH
    if is_restate_in_path() {
        debug!("Using restate-server from system PATH");
        return Ok("restate-server".to_string());
    }
    trace!("Installing restate-server binary");
    // Check if bundled binary is already installed
    let binary_path = get_restate_binary_path()?;
    if binary_path.exists() {
        tracing::debug!("Using bundled restate-server from {:?}", binary_path);
        return Ok(binary_path.display().to_string());
    }

    // Try to install the bundled binary if available
    if RESTATE_BINARY.is_some() {
        match install_bundled_restate() {
            Ok(installed_path) => return Ok(installed_path.display().to_string()),
            Err(e) => {
                tracing::error!("Failed to install bundled restate-server");

                return Err(e)
            }
        }
    } else {
        tracing::error!("No bundled restate-server binary available for this platform to install");
        return Err(CommonError::Unknown(anyhow::anyhow!(
            "No bundled restate-server binary available for this platform to install"
        )));
    }
}

async fn start_restate_server_local(
    process_manager: &mut CustomProcessManager,
    params: RestateServerLocalParams,
) -> Result<(), CommonError> {
    trace!("Checking local restate server ports are free");
    if is_port_in_use(params.ingress_port)? {
        return Err(CommonError::Unknown(anyhow::anyhow!(
            "Restate ingress address is in use (127.0.0.1:{})",
            params.ingress_port
        )));
    }
    if is_port_in_use(params.admin_port)? {
        return Err(CommonError::Unknown(anyhow::anyhow!(
            "Restate admin address is in use (127.0.0.1:{})",
            params.admin_port
        )));
    }
    if is_port_in_use(params.soma_restate_service_port)? {
        return Err(CommonError::Unknown(anyhow::anyhow!(
            "Restate Soma Restate service address is in use (127.0.0.1:{})",
            params.soma_restate_service_port
        )));
    }
    trace!("Local restate server ports are free");

    // Delete Restate data directory if --clean flag is set
    if params.clean {
        debug!("Clean flag is set, deleting Restate data directory");
        let restate_data_dir = params.restate_server_data_dir.clone();
        trace!("Checking if Restate data directory exists: {:?}", restate_data_dir);
        if restate_data_dir.exists() {
            trace!(
                "Cleaning Restate data directory: {}",
                restate_data_dir.display()
            );
            std::fs::remove_dir_all(&restate_data_dir).map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!(
                    "Failed to delete Restate data directory: {e}"
                ))
            })?;
            trace!("Restate data directory deleted successfully");
        } else {
            debug!("Restate data directory does not exist, skipping clean");
        }
    }

    // Ensure restate-server binary is available (use system binary or bundled one)
    let restate_binary_path = ensure_restate_binary()?;
    debug!("Using restate-server binary: {}", restate_binary_path);

    process_manager.start_process("restate-server", ProcessConfig {
        script: restate_binary_path,
        args: vec![
            "--log-filter".to_string(),
            var("RUST_LOG").unwrap_or("info".to_string()),
            "--tracing-filter".to_string(),
            var("RUST_LOG").unwrap_or("info".to_string()),
            "--log-format".to_string(),
            "pretty".to_string(),
            "--base-dir".to_string(),
            params.restate_server_data_dir.display().to_string(),
        ],
        cwd: None,
        env: {
            let mut env = HashMap::new();
            env.insert("RESTATE__INGRESS__BIND_ADDRESS".to_string(), format!("127.0.0.1:{}", params.ingress_port));
            env.insert("RESTATE__ADMIN__BIND_ADDRESS".to_string(), format!("127.0.0.1:{}", params.admin_port));
            // env.insert("RESTATE__ADVERTISED_ADDRESS".to_string(), format!("127.0.0.1:{}", params.admin_port));
            env
        },
        health_check: Some(pmdaemon::health::HealthCheckConfig {
            check_type: pmdaemon::HealthCheckType::Http { url: (format!("http://127.0.0.1:{}", params.admin_port)) },
            timeout: Duration::from_secs(5),
            interval: Duration::from_secs(5),
            retries: 3,
            enabled: true,
        }),
        on_stop: OnStop::TriggerShutdown,
        shutdown_priority: 10,
    }).await?;
    
    Ok(())
}

async fn start_restate_server_remote(
    _params: RestateServerRemoteParams,
) -> Result<(), CommonError> {
    // TODO: should just perform a curl request to the admin address / ingress address to check health and client can connect.

    Ok(())
}

pub async fn start_restate(
    process_manager: &mut CustomProcessManager,
    restate_params: RestateServerParams,
) -> Result<(), CommonError> {
    match restate_params {
        RestateServerParams::Local(params) => {
            trace!("Starting a local restate server process");
            let bar = ProgressBar::new_spinner();
            bar.enable_steady_tick(Duration::from_millis(100));
            bar.set_message("Starting local restate server...");
            let admin_port = params.admin_port;
            start_restate_server_local(process_manager, params).await?;

            trace!("Waiting for Restate admin endpoint to be ready...");
            // Wait for Restate admin endpoint to be ready (200 status code)
            let admin_url = format!("http://127.0.0.1:{}", admin_port);
            let client = Client::new();
            let max_retries = 30;
            let retry_delay = Duration::from_secs(2);
            let mut endpoint_ready = false;
            
            for attempt in 1..=max_retries {
                trace!("Checking Restate admin endpoint (attempt {}/{}): {}", attempt, max_retries, admin_url);
                
                match client.get(&admin_url).send().await {
                    Ok(response) => {
                        if response.status().as_u16() == 200 {
                            trace!("Restate admin endpoint is ready (200 OK)");
                            endpoint_ready = true;
                            break;
                        } else {
                            trace!("Restate admin endpoint returned status: {}", response.status().as_u16());
                        }
                    }
                    Err(e) => {
                        trace!("Failed to connect to Restate admin endpoint: {}", e);
                    }
                }
                
                if attempt < max_retries {
                    trace!("Waiting {} seconds before next attempt", retry_delay.as_secs());
                    sleep(retry_delay).await;
                }
            }
            
            if !endpoint_ready {
                error!("Restate admin process started but endpoint did not become ready after {} attempts ({} seconds)", max_retries, max_retries * retry_delay.as_secs());
                return Err(CommonError::Unknown(anyhow::anyhow!(
                    "Restate admin endpoint at {} did not become ready after {} attempts ({} seconds)",
                    admin_url,
                    max_retries,
                    max_retries * retry_delay.as_secs()
                )));
            }
            
            trace!("Restate admin endpoint is ready");
            bar.finish_and_clear();
            trace!("Local restate server process started");
            
        }
        RestateServerParams::Remote(params) => {
            trace!("Restate is running remotely, checking health and client can connect...");
            start_restate_server_remote(params).await?;
            trace!("Remote restate server process started");
        }
    }

    Ok(())
}
