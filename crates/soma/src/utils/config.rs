use std::{
    fs::{self, File}, ops::Deref, path::PathBuf, sync::Arc
};

use shared::error::CommonError;
use serde::{Deserialize, Serialize};
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
        config.dev_server = Some(DevServerConfig {
            base_api_url: url,
        });
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
        None => return Err(CommonError::Unknown(anyhow::anyhow!("config directory not found"))),
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
        None => {
            Err(CommonError::Unknown(anyhow::anyhow!(
                "You are not signed in. Please sign in using 'soma auth sign-in'"
            )))
        }
    }
}

#[allow(dead_code)]
pub async fn get_cloud_api_config(
    config: &CliConfig,
) -> Result<soma_api_client::apis::configuration::Configuration, CommonError> {
    let config = config.get_config().await?;
    let user = ensure_user_is_set(&config).await?;
    let mut headers = http::HeaderMap::new();
    headers.insert(
        "authorization",
        format!("Bearer {}", user.jwt).parse().unwrap(),
    );
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;
    let client = soma_api_client::apis::configuration::Configuration {
        base_path: config.cloud.base_api_url.clone(),
        bearer_access_token: Some(user.jwt),
        client,
        ..Default::default()
    };
    info!("API config: {:?}", client);
    Ok(client)
}

#[allow(dead_code)]
pub async fn get_dev_server_api_config(config: &CliConfig) -> Result<soma_api_client::apis::configuration::Configuration, CommonError> {
    let config = config.get_config().await?;
    let dev_server = match config.dev_server.clone() {
        Some(dev_server) => dev_server,
        None => {
            return Err(CommonError::Unknown(anyhow::anyhow!("No dev server config found")));
        }
    };

    let client = reqwest::Client::builder()
        .build()?;
    let client = soma_api_client::apis::configuration::Configuration {
        base_path: dev_server.base_api_url.clone(),
        client,
        ..Default::default()
    };
    info!("API config: {:?}", client);
    Ok(client)

}

