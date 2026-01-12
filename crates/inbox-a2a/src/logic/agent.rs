//! Agent logic for A2A protocol
//!
//! Provides agent card construction utilities.

use std::collections::HashMap;

use crate::a2a_core::types::AgentCard;
use shared::soma_agent_definition::SomaAgentDefinition;

/// Parameters for constructing an agent card
pub struct ConstructAgentCardParams {
    pub definition: SomaAgentDefinition,
    pub url: String,
}

/// Construct an A2A agent card from a definition
pub fn construct_agent_card(params: ConstructAgentCardParams) -> crate::a2a_core::types::AgentCard {
    let _definition = params.definition;
    let url = params.url;

    AgentCard {
        additional_interfaces: vec![],
        capabilities: crate::a2a_core::types::AgentCapabilities {
            streaming: Some(true),
            push_notifications: None,
            state_transition_history: None,
            extensions: vec![],
        },
        default_input_modes: vec![],
        default_output_modes: vec![],
        description: String::new(),
        documentation_url: None,
        icon_url: None,
        name: String::new(),
        preferred_transport: "JSONRPC".to_string(),
        protocol_version: "1.0.0".to_string(),
        provider: None,
        security: vec![],
        security_schemes: HashMap::new(),
        signatures: vec![],
        skills: vec![],
        supports_authenticated_extended_card: None,
        url: url.to_string(),
        version: "1.0.0".to_string(),
    }
}
