use std::collections::HashMap;

use a2a_rs::types::AgentCard;
use shared::soma_agent_definition::SomaAgentDefinition;



pub struct ConstructAgentCardParams {
    pub definition: SomaAgentDefinition,
    pub url: String,
}

pub fn construct_agent_card(params: ConstructAgentCardParams) -> a2a_rs::types::AgentCard {
    let definition = params.definition;
    let url = params.url;

    AgentCard {
        additional_interfaces: vec![],
        capabilities: a2a_rs::types::AgentCapabilities {
            streaming: Some(true),
            push_notifications: None,
            state_transition_history: None,
            extensions: vec![],
        },
        default_input_modes: vec![],
        default_output_modes: vec![],
        description: definition.description.clone(),
        documentation_url: None,
        icon_url: None,
        name: definition.name.clone(),
        preferred_transport: "JSONRPC".to_string(),
        protocol_version: "1.0.0".to_string(),
        provider: None,
        security: vec![],
        security_schemes: HashMap::new(),
        signatures: vec![],
        skills: vec![],
        supports_authenticated_extended_card: None,
        url: url.to_string(),
        version: definition.version.clone(),
    }
}
