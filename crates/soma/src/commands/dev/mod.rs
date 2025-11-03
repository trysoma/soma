mod bridge_util;
mod project_file_watcher;
mod process_manager;
mod repository;
mod restate;
mod runtime;
mod server;

use std::path::PathBuf;
use std::sync::Arc;

use bridge::logic::PROVIDER_REGISTRY;
use bridge::logic::get_or_create_local_encryption_key;
use bridge::logic::register_all_bridge_providers;
use bridge::logic::{EnvelopeEncryptionKeyContents, EnvelopeEncryptionKeyId};
use clap::Args;
use clap::Parser;
use tokio::sync::{mpsc, oneshot};
use tokio_graceful_shutdown::{SubsystemBuilder, SubsystemHandle};
use tracing::{error, info, warn};
use url::Url;

use shared::error::CommonError;
use shared::libsql::{
  establish_db_connection, inject_auth_token_to_db_url, merge_nested_migrations,
};
use shared::primitives::SqlMigrationLoader;
use shared::soma_agent_definition::{SomaAgentDefinitionLike, YamlSomaAgentDefinition};

use crate::commands::dev::repository::setup_repository;
use crate::commands::dev::restate::{RestateServerLocalParams, RestateServerParams};
use crate::commands::dev::restate::RestateServerRemoteParams;
use crate::logic::ConnectionManager;
use crate::repository::Repository;
use crate::utils::config::{CliConfig, get_config_file_path};
use crate::utils::construct_src_dir_absolute;

use self::bridge_util::{start_sync_on_bridge_change_subsystem, start_bridge_background_credential_rotation_subsystem, sync_bridge_db_from_soma_definition_on_start};
use self::project_file_watcher::{FileChangeEvt, FileChangeTx, start_project_file_watcher_subsystem};
use self::process_manager::{DevReloaderSubsystemParams, start_dev_reloader_subsystem};
use self::restate::start_restate_subsystem;
use self::runtime::determine_runtime;
use self::server::find_free_port;


#[derive(Args, Debug, Clone)]
#[group(multiple = false, required = false)]
pub struct RemoteRestateParams {
  #[arg(long = "restate-admin-url", requires = "ingress_url")]
  pub admin_url: Option<Url>,
  #[arg(long = "restate-ingress-url", requires = "admin_url")]
  pub ingress_url: Option<Url>,
  #[arg(long = "restate-admin-token", requires = "admin_url", requires = "ingress_url")]
  pub admin_token: Option<String>,
}

impl TryFrom<RemoteRestateParams> for RestateServerParams {
  type Error = CommonError;
  fn try_from(params: RemoteRestateParams) -> Result<Self, Self::Error> {
    if params.admin_url.is_none() || params.ingress_url.is_none() {
      return Err(CommonError::Unknown(anyhow::anyhow!("Admin URL and ingress URL are required")));
    }
    Ok(RestateServerParams::Remote(RestateServerRemoteParams {
      admin_address: params.admin_url.unwrap(),
      ingress_address: params.ingress_url.unwrap(),
      admin_token: params.admin_token,
    }))
  }
}

#[derive(Debug, Clone, Parser)]
pub struct DevParams {
  #[arg(long, default_value = "3000")]
  pub port: u16,
  #[arg(long, default_value = "127.0.0.1")]
  pub host: String,
  #[arg(long)]
  pub src_dir: Option<PathBuf>,
  #[arg(long, default_value = "libsql://./.soma/local.db?mode=local")]
  pub db_conn_string: Url,
  #[arg(long)]
  pub db_auth_token: Option<String>,
  #[command(flatten)]
  pub remote_restate: Option<RemoteRestateParams>,


  #[arg(
    long,
    default_value = "local",
    help = "The type of key encryption key to use. Valid values are 'local' or a valid AWS KMS ARN (arn:aws:kms:region:account-id:key/key-id)."
  )]
  pub key_encryption_key: Option<String>,
}


/// Main entry point for the start command
pub async fn cmd_dev(
  subsys: &SubsystemHandle,
  params: DevParams,
  _cli_config: &mut CliConfig,
) -> Result<(), CommonError> {


  let project_dir = construct_src_dir_absolute(params.clone().src_dir)?;

  // Determine runtime and find free port
  let runtime = determine_runtime(&params)?
    .ok_or(CommonError::Unknown(anyhow::anyhow!("No runtime matched")))?;
  let runtime_port = find_free_port(9080, 10080)?;


  // Setup encryption key
  let envelope_encryption_key_contents = setup_encryption_key(params.key_encryption_key.clone())?;

  // Load soma definition
  let soma_definition: Arc<dyn SomaAgentDefinitionLike> = load_soma_definition(&project_dir)?;

  // Setup database and repositories
  let connection_manager = ConnectionManager::new();
  let (
    _db,
    conn,
    repository,
    bridge_repo,
  ) = setup_repository(&params.db_conn_string,  &params.db_auth_token).await?;

 
  // Start Restate server subsystem
  let restate_params = match params.remote_restate {
    Some(remote_restate) => remote_restate.try_into()?,
    None => RestateServerParams::Local(RestateServerLocalParams {
      project_dir: project_dir.clone(),
      ingress_port: 8080,
      admin_port: 9070,
      advertised_node_port: 5122,
    }),
  };
  start_restate_subsystem(subsys, restate_params.clone()).await?;

  
  // Start file watcher subsystem
  let (prj_file_change_tx, prj_file_change_rx) = start_project_file_watcher_subsystem(
    subsys,
    &project_dir,
  )?;

  // Start bridge config change listener subsystem
  let on_bridge_change_evt_tx = start_sync_on_bridge_change_subsystem(subsys, &soma_definition)?;

  // Sync bridge from soma definition
  sync_bridge_db_from_soma_definition_on_start(
    &envelope_encryption_key_contents,
    &on_bridge_change_evt_tx,
    &bridge_repo,
    &repository,
    &soma_definition,
  )
  .await?;

  // Start restartable processes subsystem
  start_dev_reloader_subsystem(
    subsys,
    DevReloaderSubsystemParams {
      host: params.host,
      port: params.port,
      restate_client_params: restate_params,
      runtime,
      runtime_port,
      prj_file_change_tx,
      project_dir,
      connection_manager,
      repository,
      db_connection: conn,
      soma_definition: soma_definition.clone(),
      envelope_encryption_key_contents: envelope_encryption_key_contents.clone(),
      on_bridge_config_change_tx: on_bridge_change_evt_tx.clone(),
      bridge_repository: bridge_repo.clone(),
    },
  ).await?;


  // Start credential rotation subsystem
  start_bridge_background_credential_rotation_subsystem(
    subsys,
    bridge_repo,
    envelope_encryption_key_contents,
    on_bridge_change_evt_tx,
  );

  subsys.on_shutdown_requested().await;
  Ok(())
}

/// Sets up the envelope encryption key
fn setup_encryption_key(key_encryption_key: Option<String>) -> Result<EnvelopeEncryptionKeyContents, CommonError> {
  let local_key_path = get_config_file_path()?
    .parent()
    .ok_or(CommonError::Unknown(anyhow::anyhow!(
      "Failed to get config file path"
    )))?
    .join("local-key.bin");

  let envelope_encryption_key_contents = match key_encryption_key {
    Some(key_encryption_key) => match key_encryption_key.as_str() {
      "local" => get_or_create_local_encryption_key(&local_key_path)?,
      _ => {
        if is_valid_kms_arn(&key_encryption_key) {
          EnvelopeEncryptionKeyContents::AwsKms {
            arn: key_encryption_key,
          }
        } else {
          return Err(CommonError::Unknown(anyhow::anyhow!(
            "Invalid AWS KMS ARN: {}",
            key_encryption_key
          )));
        }
      }
    },
    None => get_or_create_local_encryption_key(&local_key_path)?,
  };

  if matches!(
    envelope_encryption_key_contents,
    EnvelopeEncryptionKeyContents::Local { .. }
  ) {
    warn!(
      "Local key encryption key is for getting started quickly. Please use a valid AWS KMS ARN for production."
    );
  }

  Ok(envelope_encryption_key_contents)
}

/// Loads the soma definition from the source directory
fn load_soma_definition(
  src_dir: &PathBuf,
) -> Result<Arc<dyn SomaAgentDefinitionLike>, CommonError> {
  let path_to_soma_definition = src_dir.join("soma.yaml");

  if !path_to_soma_definition.exists() {
    return Err(CommonError::Unknown(anyhow::anyhow!(
      "Soma definition not found at {}",
      path_to_soma_definition.display()
    )));
  }
  let soma_definition = YamlSomaAgentDefinition::load_from_file(path_to_soma_definition)?;
  Ok(Arc::new(soma_definition))
}



/// Validates if a string is a valid AWS KMS ARN (testable)
pub fn is_valid_kms_arn(arn: &str) -> bool {
  arn.starts_with("arn:aws:kms:")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_is_valid_kms_arn_with_valid_arn() {
    let arn = "arn:aws:kms:us-east-1:123456789012:key/12345678-1234-1234-1234-123456789012";
    assert!(is_valid_kms_arn(arn));
  }

  #[test]
  fn test_is_valid_kms_arn_with_invalid_arn() {
    let arn = "arn:aws:s3:::my-bucket";
    assert!(!is_valid_kms_arn(arn));
  }

  #[test]
  fn test_is_valid_kms_arn_with_random_string() {
    let arn = "not-an-arn";
    assert!(!is_valid_kms_arn(arn));
  }

  #[test]
  fn test_is_valid_kms_arn_empty_string() {
    assert!(!is_valid_kms_arn(""));
  }

  #[test]
  fn test_is_valid_kms_arn_local() {
    assert!(!is_valid_kms_arn("local"));
  }
}
