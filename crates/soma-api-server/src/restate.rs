use std::path::PathBuf;

use url::Url;

use shared::error::CommonError;
use shared::restate::admin_client::AdminClient;
use shared::restate::invoke::RestateIngressClient;

#[derive(Clone)]
pub struct RestateServerLocalParams {
    pub project_dir: PathBuf,
    pub ingress_port: u16,
    pub admin_port: u16,
    pub advertised_node_port: u16,
    pub clean: bool,
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
        Ok(RestateIngressClient::new(
            self.get_ingress_address()?.to_string(),
        ))
    }

    pub async fn get_admin_client(&self) -> Result<AdminClient, CommonError> {
        AdminClient::new(self.get_admin_address()?, self.get_admin_token()).await
    }

    pub fn get_admin_address(&self) -> Result<Url, CommonError> {
        let res = match self {
            RestateServerParams::Local(params) => {
                Url::parse(&format!("http://127.0.0.1:{}", params.admin_port))?
            }
            RestateServerParams::Remote(params) => params.admin_address.clone(),
        };

        Ok(res)
    }

    pub fn get_ingress_address(&self) -> Result<Url, CommonError> {
        let res = match self {
            RestateServerParams::Local(params) => {
                Url::parse(&format!("http://127.0.0.1:{}", params.ingress_port))?
            }
            RestateServerParams::Remote(params) => params.ingress_address.clone(),
        };

        Ok(res)
    }

    pub fn get_admin_token(&self) -> Option<String> {
        match self {
            RestateServerParams::Local(_params) => None,
            RestateServerParams::Remote(params) => params.admin_token.clone(),
        }
    }

    pub fn get_private(&self) -> bool {
        match self {
            RestateServerParams::Local(_params) => false,
            RestateServerParams::Remote(_params) => false,
        }
    }

    pub fn get_insecure(&self) -> bool {
        match self {
            RestateServerParams::Local(_params) => true,
            RestateServerParams::Remote(_params) => false,
        }
    }

    pub fn get_force(&self) -> bool {
        match self {
            RestateServerParams::Local(_params) => true,
            RestateServerParams::Remote(_params) => true,
        }
    }
}
