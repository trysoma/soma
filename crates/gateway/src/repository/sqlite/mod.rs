use libsql::Connection;

pub struct Repository {
    pub conn: Connection,
}

impl Repository {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl super::GatewayRepositoryLike for Repository {
    // TODO: Implement repository methods
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_repository_creation() {
        // TODO: Add tests
    }
}
