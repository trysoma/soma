use std::collections::HashMap;

use a2a_rs::types::AgentCard;
use serde::{Deserialize, Serialize};
use url::Url;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SomaConfig {
    pub name: String,
    pub description: String,
    pub version: String,
}



impl SomaConfig {
    pub fn from_yaml(yaml_str: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml_str)
    }

    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }
}


pub struct ConstructAgentCardParams {
    pub config: SomaConfig,
    pub url: String
}

pub fn construct_agent_card(config: &SomaConfig, url: &Url) -> a2a_rs::types::AgentCard {
    AgentCard {
        additional_interfaces: vec![],
        capabilities: a2a_rs::types::AgentCapabilities {
            streaming: None,
            push_notifications: None,
            state_transition_history: None,
            extensions: vec![],
        },
        default_input_modes: vec![],
        default_output_modes: vec![],
        description: config.description.clone(),
        documentation_url: None,
        icon_url: None,
        name: config.name.clone(),
        preferred_transport: "JSONRPC".to_string(),
        protocol_version: "1.0.0".to_string(),
        provider: None,
        security: vec![],
        security_schemes: HashMap::new(),
        signatures: vec![],
        skills: vec![],
        supports_authenticated_extended_card: None,
        url: url.to_string(),
        version: config.version.clone(),
    }
}