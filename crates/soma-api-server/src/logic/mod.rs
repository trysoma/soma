pub mod a2a;
pub mod bridge;
pub mod environment_variable;
pub mod environment_variable_sync;
pub mod identity;
pub mod internal;
pub mod on_change_pubsub;
pub mod secret;
pub mod secret_sync;
pub mod task;

// Re-export MessageRole for sqlc generated code compatibility
pub use task::MessageRole;
