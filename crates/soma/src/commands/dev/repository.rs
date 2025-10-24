use shared::error::CommonError;
use shared::libsql::{establish_db_connection, inject_auth_token_to_db_url, merge_nested_migrations};
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
      crate::repository::Repository,
      bridge::repository::Repository,
    ),
    CommonError,
  > {
  
    let migrations = merge_nested_migrations(vec![
        crate::repository::Repository::load_sql_migrations(),
      bridge::repository::Repository::load_sql_migrations(),
    ]);
    let auth_conn_string = inject_auth_token_to_db_url(conn_string, auth_token)?;
    let (db, conn) = establish_db_connection(&auth_conn_string, Some(migrations)).await?;
  
    let repo = crate::repository::Repository::new(conn.clone());
    let bridge_repo = bridge::repository::Repository::new(conn.clone());
    Ok((db, conn, repo, bridge_repo))
  }