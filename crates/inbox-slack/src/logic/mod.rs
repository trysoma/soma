//! Logic module for Slack inbox provider
//!
//! Contains:
//! - SlackClient for making HTTP requests to Slack API
//! - Event handler for processing inbox events and sending Slack messages

mod client;
mod event_handler;

pub use client::SlackClient;
pub use event_handler::run_event_handler;
