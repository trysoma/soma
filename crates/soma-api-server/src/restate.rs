use std::collections::HashMap;
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
    pub soma_restate_service_port: u16,
    pub soma_restate_service_additional_headers: HashMap<String, String>,
    pub clean: bool,
}

#[derive(Clone)]
pub struct RestateServerRemoteParams {
    pub admin_address: Url,
    pub ingress_address: Url,
    pub admin_token: Option<String>,
    pub soma_restate_service_address: Url,
    pub soma_restate_service_additional_headers: HashMap<String, String>,
}

#[derive(Clone)]
#[allow(clippy::large_enum_variant)]
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

    /// Get the address where Soma's Restate service is accessible
    pub fn get_soma_restate_service_address(&self) -> Url {
        match self {
            RestateServerParams::Local(params) => Url::parse(&format!(
                "http://127.0.0.1:{}",
                params.soma_restate_service_port
            ))
            .expect("Failed to parse Soma Restate service address"),
            RestateServerParams::Remote(params) => params.soma_restate_service_address.clone(),
        }
    }

    /// Get additional headers for Soma Restate service deployment
    pub fn get_soma_restate_service_additional_headers(&self) -> HashMap<String, String> {
        match self {
            RestateServerParams::Local(params) => {
                params.soma_restate_service_additional_headers.clone()
            }
            RestateServerParams::Remote(params) => {
                params.soma_restate_service_additional_headers.clone()
            }
        }
    }
}
