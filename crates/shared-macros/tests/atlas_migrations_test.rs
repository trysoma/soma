use shared_macros::load_atlas_sql_migrations;
use std::collections::BTreeMap;

#[test]
fn test_load_atlas_migrations() {
    let migrations: BTreeMap<&str, BTreeMap<&str, &str>> =
        load_atlas_sql_migrations!("test-migrations-atlas");

    // Check that sqlite backend exists
    assert!(
        migrations.contains_key("sqlite"),
        "Should have sqlite backend"
    );

    let sqlite_migrations = migrations.get("sqlite").unwrap();

    // Check that both .up.sql and .down.sql were generated for first migration
    assert!(
        sqlite_migrations.contains_key("001_create_users.up.sql"),
        "Should have generated .up.sql file"
    );
    assert!(
        sqlite_migrations.contains_key("001_create_users.down.sql"),
        "Should have generated .down.sql file"
    );

    // Verify the migration.sql content
    let up_sql = sqlite_migrations.get("001_create_users.up.sql").unwrap();
    assert!(
        up_sql.contains("CREATE TABLE users"),
        "Up migration should create users table"
    );
    assert!(
        up_sql.contains("CREATE INDEX idx_users_email"),
        "Up migration should create index"
    );
    assert!(
        up_sql.contains("INSERT INTO users"),
        "Up migration should insert Alice"
    );

    // Verify the down.sql content
    let down_sql = sqlite_migrations.get("001_create_users.down.sql").unwrap();
    assert!(
        down_sql.contains("DROP TABLE IF EXISTS users"),
        "Down migration should drop users table"
    );
    assert!(
        down_sql.contains("DROP INDEX IF EXISTS idx_users_email"),
        "Down migration should drop index"
    );

    // Verify that goose markers are NOT included in the migration content
    assert!(
        !up_sql.contains("-- +goose Up"),
        "Up migration should not contain goose marker"
    );
    assert!(
        !down_sql.contains("-- +goose Down"),
        "Down migration should not contain goose marker"
    );
}

#[test]
fn test_multiple_atlas_migrations() {
    let migrations: BTreeMap<&str, BTreeMap<&str, &str>> =
        load_atlas_sql_migrations!("test-migrations-atlas");
    let sqlite_migrations = migrations.get("sqlite").unwrap();

    // Should have migrations from both files
    assert!(
        sqlite_migrations.len() >= 4,
        "Should have at least 4 files (2 migrations × 2 files each)"
    );

    // Check second migration exists
    assert!(
        sqlite_migrations.contains_key("002_add_user_settings.sqlite.up.sql"),
        "Should have second migration up file"
    );
    assert!(
        sqlite_migrations.contains_key("002_add_user_settings.sqlite.down.sql"),
        "Should have second migration down file"
    );

    // Verify second migration content
    let up_sql = sqlite_migrations
        .get("002_add_user_settings.sqlite.up.sql")
        .unwrap();
    assert!(
        up_sql.contains("CREATE TABLE user_settings"),
        "Second migration should create user_settings table"
    );
    assert!(
        up_sql.contains("FOREIGN KEY"),
        "Second migration should have foreign key"
    );

    let down_sql = sqlite_migrations
        .get("002_add_user_settings.sqlite.down.sql")
        .unwrap();
    assert!(
        down_sql.contains("DROP TABLE IF EXISTS user_settings"),
        "Down migration should drop user_settings"
    );
}

#[test]
fn test_atlas_migration_structure() {
    let migrations: BTreeMap<&str, BTreeMap<&str, &str>> =
        load_atlas_sql_migrations!("test-migrations-atlas");
    let sqlite_migrations = migrations.get("sqlite").unwrap();

    // Ensure migrations are properly ordered
    let keys: Vec<_> = sqlite_migrations.keys().collect();
    assert!(
        keys.len() >= 2,
        "Should have at least 2 files (up and down)"
    );

    // Verify the content doesn't have the goose markers
    let up_sql = sqlite_migrations.get("001_create_users.up.sql").unwrap();
    assert!(
        !up_sql.contains("-- +goose Up"),
        "Should not contain goose Up marker"
    );
    assert!(
        !up_sql.contains("-- +goose Down"),
        "Should not contain goose Down marker"
    );
}
