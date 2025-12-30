pub mod agent;
pub mod identity;
pub mod internal;
pub mod mcp;
pub mod on_change_pubsub;
pub mod secret_sync;
pub mod task;
pub mod variable_sync;

// Re-export MessageRole for sqlc generated code compatibility
pub use task::MessageRole;
