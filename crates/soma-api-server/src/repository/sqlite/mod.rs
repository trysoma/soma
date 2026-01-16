#![allow(non_camel_case_types)]

// soma-api-server no longer needs its own database repository
// All database operations are handled by specialized crates:
// - encryption crate for secrets
// - environment crate for environment variables
// - mcp crate for MCP server instances
// - identity crate for authentication/authorization

#[derive(Clone)]
pub struct Repository {
    #[allow(dead_code)]
    conn: shared::libsql::Connection,
}

impl Repository {
    pub fn new(conn: shared::libsql::Connection) -> Self {
        Self { conn }
    }
}
