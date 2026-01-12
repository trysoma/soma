//! Connection manager for A2A real-time task updates
//!
//! This module manages SSE/WebSocket connections for streaming task events.

use dashmap::DashMap;
use futures::stream::{self, StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::trace;

use shared::{error::CommonError, primitives::{WrappedChronoDateTime, WrappedUuidV4}};

/// Represents a single SSE/WebSocket connection to a task
#[derive(Debug, Clone)]
pub struct Connection {
    pub id: WrappedUuidV4,
    pub created_at: WrappedChronoDateTime,
    pub sender: Sender<crate::a2a_core::events::Event>,
}

/// Manages connections for real-time task updates
///
/// ConnectionManager tracks active SSE/WebSocket connections per task,
/// enabling broadcasting of task events to multiple subscribers.
#[derive(Debug, Clone)]
pub struct ConnectionManager {
    pub connections_by_task_id: Arc<DashMap<WrappedUuidV4, DashMap<WrappedUuidV4, Connection>>>,
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections_by_task_id: Arc::new(DashMap::new()),
        }
    }

    /// Add a new connection for a task and return the connection ID and receiver
    pub fn add_connection(
        &self,
        task_id: WrappedUuidV4,
    ) -> Result<(WrappedUuidV4, Receiver<crate::a2a_core::events::Event>), CommonError> {
        let connection_id = WrappedUuidV4::new();
        let (sender, receiver) = tokio::sync::mpsc::channel::<crate::a2a_core::events::Event>(100);
        let connections = self
            .connections_by_task_id
            .entry(task_id.clone())
            .or_default();
        connections.insert(
            connection_id.clone(),
            Connection {
                id: connection_id.clone(),
                created_at: WrappedChronoDateTime::now(),
                sender,
            },
        );
        Ok((connection_id, receiver))
    }

    /// Remove a connection from a task
    pub fn remove_connection(
        &self,
        task_id: WrappedUuidV4,
        connection_id: WrappedUuidV4,
    ) -> Result<(), CommonError> {
        let connections = match self.connections_by_task_id.get_mut(&task_id) {
            Some(connections) => connections,
            None => {
                return Err(CommonError::NotFound {
                    msg: "Connections not found".to_string(),
                    lookup_id: task_id.to_string(),
                    source: None,
                });
            }
        };
        connections.remove(&connection_id);
        Ok(())
    }

    /// Broadcast a message to all connections for a task
    pub async fn message_to_connections(
        &self,
        task_id: WrappedUuidV4,
        message: crate::a2a_core::events::Event,
    ) -> Result<(), CommonError> {
        trace!(task_id = %task_id, "Broadcasting to connections");
        let connections = match self.connections_by_task_id.get(&task_id) {
            Some(connections) => connections,
            None => return Ok(()),
        };

        // Collect all senders first (release DashMap guard)
        let senders: Vec<_> = connections
            .iter()
            .map(|entry| entry.sender.clone())
            .collect();
        let connection_count = senders.len();
        drop(connections);

        trace!(task_id = %task_id, count = connection_count, "Sending to connections");

        // Run up to 32 sends in parallel (adjust concurrency level as needed)
        stream::iter(senders)
            .for_each_concurrent(32, |sender| {
                let message = message.clone();
                async move {
                    if let Err(e) = sender.send(message).await {
                        tracing::warn!(error = %e, "Failed to send to connection");
                    }
                }
            })
            .await;

        Ok(())
    }
}
