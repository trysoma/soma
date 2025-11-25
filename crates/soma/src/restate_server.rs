use anyhow::Context;
use shared::command::run_child_process;
use shared::error::CommonError;
use shared::subsystem::SubsystemHandle;
use std::fs;
use std::path::PathBuf;
use tokio::process::Command;
use tokio::sync::broadcast;
use tracing::{error, info};

use shared::port::is_port_in_use;
use soma_api_server::restate::{
    RestateServerLocalParams, RestateServerParams, RestateServerRemoteParams,
};

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
        tracing::info!("Using restate-server from system PATH");
        return Ok("restate-server".to_string());
    }

    // Check if bundled binary is already installed
    let binary_path = get_restate_binary_path()?;
    if binary_path.exists() {
        tracing::info!("Using bundled restate-server from {:?}", binary_path);
        return Ok(binary_path.display().to_string());
    }

    // Try to install the bundled binary if available
    if RESTATE_BINARY.is_some() {
        match install_bundled_restate() {
            Ok(installed_path) => return Ok(installed_path.display().to_string()),
            Err(e) => {
                tracing::warn!("Failed to install bundled restate-server: {:?}", e);
            }
        }
    } else {
        tracing::warn!("No bundled restate-server binary available for this platform");
    }

    // Fall back to expecting restate-server to be in PATH
    tracing::warn!("Falling back to 'restate-server' command from PATH");
    Ok("restate-server".to_string())
}

/// Starts the Restate server subsystem
pub async fn start_restate_server(
    params: RestateServerParams,
    kill_signal_rx: tokio::sync::broadcast::Receiver<()>,
) -> Result<(), CommonError> {
    match params {
        RestateServerParams::Local(params) => {
            info!("Starting Restate server locally");
            start_restate_server_local(params, kill_signal_rx).await
        }
        RestateServerParams::Remote(params) => {
            info!("Restate is running remotely, checking health and client can connect...");
            start_restate_server_remote(params).await
        }
    }
}

async fn start_restate_server_local(
    params: RestateServerLocalParams,
    kill_signal_rx: tokio::sync::broadcast::Receiver<()>,
) -> Result<(), CommonError> {
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
    if is_port_in_use(params.advertised_node_port)? {
        return Err(CommonError::Unknown(anyhow::anyhow!(
            "Restate advertised node address is in use (127.0.0.1:{})",
            params.advertised_node_port
        )));
    }

    // Delete Restate data directory if --clean flag is set
    if params.clean {
        let restate_data_dir = params.project_dir.join(".soma/restate-data");
        if restate_data_dir.exists() {
            info!(
                "Cleaning Restate data directory: {}",
                restate_data_dir.display()
            );
            std::fs::remove_dir_all(&restate_data_dir).map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!(
                    "Failed to delete Restate data directory: {e}"
                ))
            })?;
            info!("Restate data directory deleted successfully");
        } else {
            info!("Restate data directory does not exist, skipping clean");
        }
    }

    // Ensure restate-server binary is available (use system binary or bundled one)
    let restate_binary_path = ensure_restate_binary()?;
    info!("Using restate-server binary: {}", restate_binary_path);

    let mut cmd = Command::new(&restate_binary_path);

    cmd.arg("--log-filter")
        .arg("warn")
        .arg("--tracing-filter")
        .arg("warn")
        .arg("--base-dir")
        .arg(
            params
                .project_dir
                .join(".soma/restate-data")
                .display()
                .to_string(),
        )
        .env(
            "RESTATE__INGRESS__BIND_ADDRESS",
            format!("127.0.0.1:{}", params.ingress_port),
        )
        .env(
            "RESTATE__ADMIN__BIND_ADDRESS",
            format!("127.0.0.1:{}", params.admin_port),
        )
        .env(
            "RESTATE__ADVERTISED_ADDRESS",
            format!("127.0.0.1:{}", params.advertised_node_port),
        );
    run_child_process("restate-server", cmd, Some(kill_signal_rx), None).await?;
    Ok(())
}

async fn start_restate_server_remote(
    _params: RestateServerRemoteParams,
) -> Result<(), CommonError> {
    // TODO: should just perform a curl request to the admin address / ingress address to check health and client can connect.

    Ok(())
}

pub fn start_restate_subsystem(
    restate_params: RestateServerParams,
    shutdown_rx: broadcast::Receiver<()>,
) -> Result<SubsystemHandle, CommonError> {
    let (handle, signal) = SubsystemHandle::new("Restate");

    tokio::spawn(async move {
        match start_restate_server(restate_params, shutdown_rx).await {
            Ok(()) => {
                signal.signal_with_message("stopped gracefully");
            }
            Err(e) => {
                error!("Restate server stopped with error: {:?}", e);
                signal.signal();
            }
        }
    });

    Ok(handle)
}
