use crate::error::CommonError;
use crate::libsql::merge_nested_migrations;
use crate::libsql::{Connection, Migrations};
pub async fn setup_in_memory_database<'a>(
    migrations: Vec<Migrations<'a>>,
) -> Result<(libsql::Database, Connection), CommonError> {
    let db = libsql::Builder::new_local(":memory:")
        .build()
        .await
        .unwrap();
    let conn = crate::libsql::Connection(db.connect().unwrap());

    // Enable foreign key constraints
    conn.execute("PRAGMA foreign_keys = ON", ()).await.unwrap();

    let migrations_to_run = merge_nested_migrations(migrations);
    let migrations_to_run = migrations_to_run.get("sqlite").unwrap();

    // Filter to only run .up.sql migrations (not .down.sql)
    let migrations_to_run: std::collections::BTreeMap<_, _> = migrations_to_run
        .iter()
        .filter(|(filename, _)| filename.contains(".up."))
        .map(|(k, v)| (*k, *v))
        .collect();

    for (_filename, contents) in migrations_to_run {
        conn.execute_batch(contents).await.unwrap();
    }
    Ok((db, conn))
}
