mod sqlite;

#[allow(unused_imports)]
pub use sqlite::Repository;

// Gateway repository trait
#[async_trait::async_trait]
pub trait GatewayRepositoryLike: Send + Sync {
    // TODO: Add repository methods for gateway-specific data
    // For example: storing request logs, usage metrics, etc.
}
