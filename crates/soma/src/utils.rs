use std::{
    fs::{self, File},
    ops::Deref,
    path::PathBuf,
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use shared::error::CommonError;
use soma_api_client::apis::configuration::Configuration as ApiClientConfiguration;
use tokio::sync::{Mutex, MutexGuard};
use tracing::info;

#[derive(Deserialize, Serialize, Clone)]
pub struct CliUser {
    pub email: String,
    pub jwt: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct CloudConfig {
    pub base_api_url: String,
    pub user: Option<CliUser>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct CliConfigInner {
    pub cloud: CloudConfig,
    pub dev_server: Option<DevServerConfig>,
}

#[derive(Clone)]
pub struct CliConfig(Arc<Mutex<CliConfigInner>>);

impl CliConfig {
    pub fn new(config: CliConfigInner) -> Self {
        Self(Arc::new(Mutex::new(config)))
    }
}

impl CliConfig {
    #[allow(dead_code)]
    pub async fn update_dev_server_url(&self, url: String) -> Result<&Self, CommonError> {
        let mut config = self.0.lock().await;
        config.dev_server = Some(DevServerConfig { base_api_url: url });
        self.save(&config).await?;
        Ok(self)
    }

    pub async fn save(&self, guard: &MutexGuard<'_, CliConfigInner>) -> Result<(), CommonError> {
        let config_file_path = get_config_file_path()?;
        let config_file = File::create(config_file_path)?;
        serde_json::to_writer_pretty(config_file, guard.deref())?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn get_config(&self) -> Result<CliConfigInner, CommonError> {
        let config = self.0.lock().await;
        Ok(config.clone())
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct DevServerConfig {
    pub base_api_url: String,
}

const CONFIG_FILE_PATH: &str = "soma/config.json";
const BASE_CLOUD_API_URL: &str = "https://console.trysoma.ai";

pub fn get_config_file_path() -> Result<PathBuf, CommonError> {
    let config_dir = match dirs::config_dir() {
        Some(home_dir) => home_dir,
        None => {
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "config directory not found"
            )));
        }
    };
    let config_file_path = config_dir.join(CONFIG_FILE_PATH);
    Ok(config_file_path)
}

pub async fn get_or_init_cli_config() -> Result<CliConfig, CommonError> {
    let config_file_path = get_config_file_path()?;
    info!("Config file path: {:?}", config_file_path);
    let config = match config_file_path.exists() {
        true => {
            let config_file = File::open(config_file_path)?;
            let config = serde_json::from_reader(config_file)?;
            CliConfig::new(config)
        }
        false => {
            fs::create_dir_all(config_file_path.parent().unwrap())?;
            let config = CliConfigInner {
                cloud: CloudConfig {
                    base_api_url: BASE_CLOUD_API_URL.to_string(),
                    user: None,
                },
                dev_server: Some(DevServerConfig {
                    base_api_url: "http://localhost:3000".to_string(),
                }),
            };
            let config = CliConfig::new(config);
            // TODO: strange syntax to force a guard
            config.save(&config.0.lock().await).await?;
            config
        }
    };
    Ok(config)
}

#[allow(dead_code)]
pub async fn ensure_user_is_set(config: &CliConfigInner) -> Result<CliUser, CommonError> {
    match config.cloud.user.clone() {
        Some(user) => Ok(user),
        None => Err(CommonError::Unknown(anyhow::anyhow!(
            "You are not signed in. Please sign in using 'soma auth sign-in'"
        ))),
    }
}

pub fn construct_cwd_absolute(cwd: Option<PathBuf>) -> Result<PathBuf, CommonError> {
    let current_dir = std::env::current_dir()?;
    let mut cwd = match cwd {
        Some(cwd) => cwd,
        None => current_dir.clone(),
    };
    if !cwd.is_absolute() {
        cwd = current_dir.join(cwd);
    }

    Ok(cwd)
}

/// Creates an API client configuration for the given base URL
///
/// # Arguments
/// * `base_url` - The base URL of the API server (e.g., "http://localhost:3000")
///
/// # Returns
/// * API client configuration ready to use with soma_api_client functions
pub fn create_api_client_config(base_url: &str) -> ApiClientConfiguration {
    ApiClientConfiguration {
        base_path: base_url.to_string(),
        user_agent: Some("soma-cli".to_string()),
        client: reqwest::Client::new(),
        basic_auth: None,
        oauth_access_token: None,
        bearer_access_token: None,
        api_key: None,
    }
}

/// Creates an API client configuration and waits for the API server to be ready
///
/// # Arguments
/// * `api_url` - The base URL of the API server (e.g., "http://localhost:3000")
/// * `timeout_secs` - Maximum time to wait for the API server to be ready (in seconds)
///
/// # Returns
/// * `Ok(ApiClientConfiguration)` if the API server is ready
/// * `Err(CommonError)` if the timeout is reached or an error occurs
pub async fn create_and_wait_for_api_client(
    api_url: &str,
    timeout_secs: u64,
) -> Result<ApiClientConfiguration, CommonError> {
    use soma_api_client::apis::a2a_api;
    use std::time::Duration;

    // Create HTTP client
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to create HTTP client: {e}")))?;

    // Create API config for health check
    let api_config = ApiClientConfiguration {
        base_path: api_url.to_string(),
        user_agent: Some("soma-cli".to_string()),
        client: client.clone(),
        basic_auth: None,
        oauth_access_token: None,
        bearer_access_token: None,
        api_key: None,
    };

    // Wait for API to be ready
    info!("Waiting for Soma API server at {} to be ready...", api_url);

    let max_retries = timeout_secs / 2; // Check every 2 seconds
    let mut connected = false;

    for attempt in 1..=max_retries {
        match a2a_api::get_agent_definition(&api_config).await {
            Ok(_) => {
                info!("Connected to Soma API server successfully");
                connected = true;
                break;
            }
            Err(e) => {
                if attempt == max_retries {
                    return Err(CommonError::Unknown(anyhow::anyhow!(
                        "Failed to connect to Soma API server after {max_retries} attempts: {e:?}. Please ensure 'soma dev' is running."
                    )));
                }
                if attempt == 1 {
                    info!(
                        "Waiting for server... (attempt {}/{})",
                        attempt, max_retries
                    );
                }
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }

    if !connected {
        return Err(CommonError::Unknown(anyhow::anyhow!(
            "Failed to connect to Soma API server. Please ensure 'soma dev' is running at {api_url}"
        )));
    }

    Ok(api_config)
}

/// Polls the health endpoint until it returns a successful response
///
/// # Arguments
/// * `api_config` - The API client configuration
/// * `timeout_secs` - Maximum time to wait for health check (in seconds)
/// * `max_retries` - Maximum number of retries
///
/// # Returns
/// * `Ok(())` if the health endpoint responds successfully
/// * `Err(CommonError)` if the timeout is reached or an error occurs
pub async fn wait_for_soma_api_health_check(
    api_config: &ApiClientConfiguration,
    timeout_secs: u64,
    max_retries: u64,
) -> Result<(), CommonError> {
    let health_url = format!("{}/_internal/v1/health", api_config.base_path);
    let client = &api_config.client;
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(timeout_secs);

    for _ in 0..max_retries {
        if start.elapsed() >= timeout {
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "Health check timeout after {timeout_secs} seconds"
            )));
        }

        match client.get(&health_url).send().await {
            Ok(response) if response.status().is_success() => {
                info!("Health check successful at {}", health_url);
                return Ok(());
            }
            Ok(response) => {
                info!(
                    "Health check returned status {}, retrying...",
                    response.status()
                );
            }
            Err(e) => {
                info!("Health check failed: {}, retrying...", e);
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    Err(CommonError::Unknown(anyhow::anyhow!(
        "Health check failed after {max_retries} retries"
    )))
}
