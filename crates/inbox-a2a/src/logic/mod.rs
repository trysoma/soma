//! Business logic for the A2A inbox provider
//!
//! This module contains the core logic for handling A2A protocol interactions,
//! separated from the HTTP routing layer.

pub mod agent;
pub mod connection_manager;
pub mod push_notification;
pub mod task;

pub use agent::{construct_agent_card, ConstructAgentCardParams};
pub use connection_manager::ConnectionManager;
pub use push_notification::{
    CreatePushNotificationConfig, PushNotificationConfigModel, UpdatePushNotificationConfig,
};
