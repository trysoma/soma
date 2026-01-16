mod sqlite;

use shared::error::CommonError;

pub use sqlite::Repository;
use tracing::debug;

// Repository setup utilities
use shared::libsql::{
    establish_db_connection, inject_auth_token_to_db_url, merge_nested_migrations,
};
use shared::primitives::SqlMigrationLoader;
use url::Url;

/// Sets up the database repository and runs migrations
pub async fn setup_repository(
    conn_string: &Url,
    auth_token: &Option<String>,
) -> Result<
    (
        libsql::Database,
        shared::libsql::Connection,
        Repository,
        tool::repository::Repository,
        encryption::repository::Repository,
        environment::repository::Repository,
    ),
    CommonError,
> {
    debug!("conn_string: {}", conn_string);
    // soma-api-server repository no longer has its own migrations
    // Only load migrations from specialized crates
    let migrations = merge_nested_migrations(vec![
        tool::repository::Repository::load_sql_migrations(),
        <encryption::repository::Repository as SqlMigrationLoader>::load_sql_migrations(),
        identity::repository::Repository::load_sql_migrations(),
        environment::repository::Repository::load_sql_migrations(),
    ]);
    let auth_conn_string = inject_auth_token_to_db_url(conn_string, auth_token)?;
    let (db, conn) = establish_db_connection(&auth_conn_string, Some(migrations)).await?;

    let repo = Repository::new(conn.clone());
    let tool_repo = tool::repository::Repository::new(conn.clone());
    let encryption_repo = encryption::repository::Repository::new(conn.clone());
    let environment_repo = environment::repository::Repository::new(conn.clone());
    Ok((db, conn, repo, tool_repo, encryption_repo, environment_repo))
}
