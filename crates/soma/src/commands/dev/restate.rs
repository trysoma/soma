use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::process::Command;
use tokio::sync::{broadcast, oneshot};
use tracing::{error, info};
use url::Url;

use shared::command::run_child_process;
use shared::error::CommonError;
use shared::soma_agent_definition::SomaAgentDefinitionLike;

use crate::utils::restate::admin_client::AdminClient;
use crate::utils::restate::invoke::RestateIngressClient;
use crate::utils::{is_port_in_use, restate};
use crate::utils::restate::deploy::DeploymentRegistrationConfig;

use super::DevParams;

#[derive(Clone)]
pub struct RestateServerLocalParams {
    pub project_dir: PathBuf,
    pub ingress_port: u16,
    pub admin_port: u16,
    pub advertised_node_port: u16,
}

#[derive(Clone)]
pub struct RestateServerRemoteParams {
    pub admin_address: Url,
    pub ingress_address: Url,
    pub admin_token: Option<String>,
}

#[derive(Clone)]
pub enum RestateServerParams {
    Local(RestateServerLocalParams),
    Remote(RestateServerRemoteParams),
}

impl RestateServerParams {

    pub fn get_ingress_client(&self) -> Result<RestateIngressClient, CommonError> {
        Ok(RestateIngressClient::new(self.get_ingress_address()?.to_string()))
    }

    pub async fn get_admin_client(&self) -> Result<AdminClient, CommonError> {
        AdminClient::new(self.get_admin_address()?, self.get_admin_token()).await
    }

    pub fn get_admin_address(&self) -> Result<Url, CommonError> {
        let res = match self {
            RestateServerParams::Local(params) => Url::parse(&format!("http://127.0.0.1:{}", params.admin_port))?,
            RestateServerParams::Remote(params) => params.admin_address.clone(),
        };

        Ok(res)
    }

    pub fn get_ingress_address(&self) -> Result<Url, CommonError> {
        let res = match self {
            RestateServerParams::Local(params) => Url::parse(&format!("http://127.0.0.1:{}", params.ingress_port))?,
            RestateServerParams::Remote(params) => params.ingress_address.clone(),
        };

        Ok(res)
    }

    pub fn get_admin_token(&self) -> Option<String> {
        match self {
            RestateServerParams::Local(params) => None,
            RestateServerParams::Remote(params) => params.admin_token.clone(),
        }
    }

    pub fn get_private(&self) -> bool {
        match self {
            RestateServerParams::Local(params) => false,
            RestateServerParams::Remote(params) => false,
        }
    }

    pub fn get_insecure(&self) -> bool {
        match self {
            RestateServerParams::Local(params) => true,
            RestateServerParams::Remote(params) => false,
        }
    }

    pub fn get_force(&self) -> bool {
        match self {
            RestateServerParams::Local(params) => true,
            RestateServerParams::Remote(params) => true,
        }
    }
}


/// Registers the deployment with Restate
/// TODO: this should be moved to the deployment subsystem and run on metadata response to register all agents in the metadata response
pub async fn start_restate_deployment(
    params: &RestateServerParams,
    deployment_type: restate::deploy::DeploymentType,
    service_path: String,
) -> Result<(), CommonError> {
    info!("Starting Restate deployment registration");

    // The HTTP service URI should point to the local Axum server, not the Restate admin
    // let service_uri = format!("http://127.0.0.1:{runtime_port}");


    // let admin_address = params.get_admin_address()?;
    // let admin_token = params.get_admin_token();
    // let private = params.get_private();
    // let insecure = params.get_insecure();
    // let force = params.get_force();
    // info!(
    //     "Registering service {} with target {} with Restate admin at {}",
    //     service_path, deployment_type, admin_address
    // );
    // // let definition = soma_definition.get_definition().await?;
    // restate::deploy::register_deployment(DeploymentRegistrationConfig {
    //     admin_url: admin_address.to_string(),
    //     // TODO: this should be the service path from the soma.yaml file
    //     service_path: service_path.clone(),
    //     deployment_type: deployment_type.clone(),
    //     bearer_token: admin_token.clone(),
    //     private,
    //     insecure,
    //     force,
    // })
    // .await?;

    // info!("Restate deployment registration complete");
    Ok(())
}

/// Starts the Restate server subsystem
pub async fn start_restate_server(params: RestateServerParams, kill_signal_rx: tokio::sync::broadcast::Receiver<()>) -> Result<(), CommonError> {

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


async fn start_restate_server_local(params: RestateServerLocalParams, kill_signal_rx: tokio::sync::broadcast::Receiver<()>) -> Result<(), CommonError> {

    if is_port_in_use(params.ingress_port).await? {
        return Err(CommonError::Unknown(anyhow::anyhow!("Restate ingress address is in use (127.0.0.1:{})", params.ingress_port)));
    }
    if is_port_in_use(params.admin_port).await? {
        return Err(CommonError::Unknown(anyhow::anyhow!("Restate admin address is in use (127.0.0.1:{})", params.admin_port)));
    }
    if is_port_in_use(params.advertised_node_port).await? {
        return Err(CommonError::Unknown(anyhow::anyhow!("Restate advertised node address is in use (127.0.0.1:{})", params.advertised_node_port)));
    }




    let mut cmd = Command::new("restate-server");
    
    cmd.arg("--log-filter")
        .arg("warn")
        .arg("--tracing-filter")
        .arg("warn")
        .arg("--base-dir")
        .arg(params.project_dir.join(".soma/restate-data").display().to_string())
        .env("RESTATE__INGRESS__BIND_ADDRESS", format!("127.0.0.1:{}", params.ingress_port))
        .env("RESTATE__ADMIN__BIND_ADDRESS", format!("127.0.0.1:{}", params.admin_port))
        .env("RESTATE__ADVERTISED_ADDRESS", format!("127.0.0.1:{}", params.advertised_node_port));
    let result =run_child_process(
        "restate-server",
        cmd,
        Some(kill_signal_rx),
        None,
    )
    .await?;
    Ok(())
}

async fn start_restate_server_remote(params: RestateServerRemoteParams) -> Result<(), CommonError> {
    // TODO: should just perform a curl request to the admin address / ingress address to check health and client can connect.
    
    Ok(())
}