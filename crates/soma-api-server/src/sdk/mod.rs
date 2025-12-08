mod interface;
mod python;
pub mod sdk_provider_sync;
mod typescript;

use std::path::{Path, PathBuf};

use shared::subsystem::SubsystemHandle;
use shared::uds::DEFAULT_SOMA_SERVER_SOCK;
use tokio::sync::broadcast;
use tracing::{error, info};

use shared::error::CommonError;

use crate::logic::environment_variable_sync::fetch_all_environment_variables;
use crate::logic::secret_sync::fetch_and_decrypt_all_secrets;
use encryption::logic::crypto_services::CryptoCache;
use interface::{ClientCtx, SdkClient};
use python::Python;
use typescript::Typescript;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SdkRuntime {
    PnpmV1,
    Python,
}

/// Type alias for SDK runtime validator function
type RuntimeValidator = fn(PathBuf) -> Result<bool, CommonError>;

/// Determines which SDK runtime to use from a directory path
pub fn determine_sdk_runtime(project_dir: &Path) -> Result<Option<SdkRuntime>, CommonError> {
    let possible_runtimes: Vec<(SdkRuntime, RuntimeValidator)> = vec![
        (SdkRuntime::PnpmV1, validate_sdk_runtime_pnpm_v1),
        (SdkRuntime::Python, validate_sdk_runtime_python_v1),
    ];

    let mut matched_runtimes = vec![];

    for (runtime, validate_fn) in possible_runtimes {
        let result = validate_fn(project_dir.to_path_buf())?;
        if result {
            matched_runtimes.push(runtime);
        }
    }

    match matched_runtimes.len() {
        0 => Ok(None),
        1 => Ok(Some(matched_runtimes[0].clone())),
        _ => Err(CommonError::Unknown(anyhow::anyhow!(
            "Multiple SDK runtimes matched"
        ))),
    }
}

fn validate_sdk_runtime_pnpm_v1(project_dir: PathBuf) -> Result<bool, CommonError> {
    let files_to_check = vec!["package.json", "vite.config.ts"];
    for file in files_to_check {
        let file_path = project_dir.join(file);
        if !file_path.exists() {
            return Ok(false);
        }
    }
    Ok(true)
}

fn validate_sdk_runtime_python_v1(project_dir: PathBuf) -> Result<bool, CommonError> {
    // Check for pyproject.toml
    let pyproject_toml = project_dir.join("pyproject.toml");
    if !pyproject_toml.exists() {
        return Ok(false);
    }
    // Optionally check for functions/ or agents/ directories
    // (not required, but helps distinguish Python projects)
    Ok(true)
}

/// Check if the project uses Vite by looking for vite.config.ts
fn is_vite_project(src_dir: &Path) -> bool {
    src_dir.join("vite.config.ts").exists()
}

pub struct StartDevSdkParams {
    pub project_dir: PathBuf,
    pub sdk_runtime: SdkRuntime,
    pub restate_service_port: u16,
    pub kill_signal_rx: broadcast::Receiver<()>,
    pub repository: std::sync::Arc<crate::repository::Repository>,
    pub crypto_cache: CryptoCache,
}

/// Starts the development SDK server with hot reloading on file changes
pub async fn start_dev_sdk(params: StartDevSdkParams) -> Result<(), CommonError> {
    let sdk_runtime = params.sdk_runtime.clone();
    let StartDevSdkParams {
        project_dir,
        restate_service_port,
        kill_signal_rx,
        repository,
        crypto_cache,
        ..
    } = params;

    // Fetch all secrets from the database
    info!("Fetching initial secrets from database...");
    let decrypted_secrets = fetch_and_decrypt_all_secrets(&repository, &crypto_cache).await?;
    let initial_secrets: std::collections::HashMap<String, String> = decrypted_secrets
        .into_iter()
        .map(|s| (s.key, s.value))
        .collect();
    info!("Fetched {} initial secrets", initial_secrets.len());

    // Fetch all environment variables from the database
    info!("Fetching initial environment variables from database...");
    let env_vars = fetch_all_environment_variables(&repository).await?;
    let initial_environment_variables: std::collections::HashMap<String, String> =
        env_vars.into_iter().map(|e| (e.key, e.value)).collect();
    info!(
        "Fetched {} initial environment variables",
        initial_environment_variables.len()
    );

    let ctx = ClientCtx {
        project_dir: project_dir.clone(),
        socket_path: DEFAULT_SOMA_SERVER_SOCK.to_string(),
        restate_service_port,
        kill_signal_rx: kill_signal_rx.resubscribe(),
        initial_secrets,
        initial_environment_variables,
    };

    match sdk_runtime {
        SdkRuntime::PnpmV1 => {
            if !is_vite_project(&project_dir) {
                return Err(CommonError::Unknown(anyhow::anyhow!(
                    "Invalid runtime. Must use Vite"
                )));
            }

            info!("Detected Vite project, starting dev server...");
            let typescript_client = Typescript::new();
            typescript_client.start_dev_server(ctx).await?;
        }
        SdkRuntime::Python => {
            info!("Detected Python project, starting dev server...");
            let python_client = Python::new();
            python_client.start_dev_server(ctx).await?;
        }
    }

    Ok(())
}

pub fn start_sdk_server_subsystem(
    project_dir: PathBuf,
    sdk_runtime: SdkRuntime,
    restate_service_port: u16,
    shutdown_rx: broadcast::Receiver<()>,
    repository: crate::repository::Repository,
    crypto_cache: CryptoCache,
) -> Result<SubsystemHandle, CommonError> {
    let (handle, signal) = SubsystemHandle::new("SDK Server");
    let repository = std::sync::Arc::new(repository);

    tokio::spawn(async move {
        match start_dev_sdk(StartDevSdkParams {
            project_dir,
            sdk_runtime,
            restate_service_port,
            kill_signal_rx: shutdown_rx,
            repository,
            crypto_cache,
        })
        .await
        {
            Ok(()) => {
                signal.signal_with_message("stopped gracefully");
            }
            Err(e) => {
                error!("SDK server stopped with error: {:?}", e);
                signal.signal();
            }
        }
    });

    Ok(handle)
}

#[cfg(all(test, feature = "unit_test"))]
mod unit_test {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validate_sdk_runtime_pnpm_v1_with_valid_project() {
        let temp_dir = TempDir::new().unwrap();

        // Create required files
        fs::write(temp_dir.path().join("package.json"), r#"{"name": "test"}"#).unwrap();
        fs::write(temp_dir.path().join("vite.config.ts"), "export default {}").unwrap();

        let result = validate_sdk_runtime_pnpm_v1(temp_dir.path().to_path_buf()).unwrap();
        assert!(result, "Should validate as PnpmV1 SDK runtime");
    }

    #[test]
    fn test_validate_sdk_runtime_pnpm_v1_missing_package_json() {
        let temp_dir = TempDir::new().unwrap();

        // Only create vite.config.ts
        fs::write(temp_dir.path().join("vite.config.ts"), "export default {}").unwrap();

        let result = validate_sdk_runtime_pnpm_v1(temp_dir.path().to_path_buf()).unwrap();
        assert!(!result, "Should not validate without package.json");
    }

    #[test]
    fn test_validate_sdk_runtime_pnpm_v1_missing_vite_config() {
        let temp_dir = TempDir::new().unwrap();

        // Only create package.json
        fs::write(temp_dir.path().join("package.json"), r#"{"name": "test"}"#).unwrap();

        let result = validate_sdk_runtime_pnpm_v1(temp_dir.path().to_path_buf()).unwrap();
        assert!(!result, "Should not validate without vite.config.ts");
    }

    #[test]
    fn test_determine_sdk_runtime_pnpm_v1() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("package.json"), r#"{"name": "test"}"#).unwrap();
        fs::write(temp_dir.path().join("vite.config.ts"), "export default {}").unwrap();

        let runtime = determine_sdk_runtime(temp_dir.path()).unwrap();
        assert_eq!(runtime, Some(SdkRuntime::PnpmV1));
    }

    #[test]
    fn test_determine_sdk_runtime_no_match() {
        let temp_dir = TempDir::new().unwrap();

        // Empty directory
        let runtime = determine_sdk_runtime(temp_dir.path()).unwrap();
        assert_eq!(runtime, None);
    }

    #[test]
    fn test_validate_sdk_runtime_python_with_valid_project() {
        let temp_dir = TempDir::new().unwrap();

        // Create pyproject.toml
        fs::write(
            temp_dir.path().join("pyproject.toml"),
            r#"[project]\nname = "test""#,
        )
        .unwrap();

        let result = validate_sdk_runtime_python_v1(temp_dir.path().to_path_buf()).unwrap();
        assert!(result, "Should validate as Python SDK runtime");
    }

    #[test]
    fn test_validate_sdk_runtime_python_missing_pyproject_toml() {
        let temp_dir = TempDir::new().unwrap();

        // Empty directory
        let result = validate_sdk_runtime_python_v1(temp_dir.path().to_path_buf()).unwrap();
        assert!(!result, "Should not validate without pyproject.toml");
    }

    #[test]
    fn test_determine_sdk_runtime_python_v1() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(
            temp_dir.path().join("pyproject.toml"),
            r#"[project]\nname = "test""#,
        )
        .unwrap();

        let runtime = determine_sdk_runtime(temp_dir.path()).unwrap();
        assert_eq!(runtime, Some(SdkRuntime::Python));
    }
}
