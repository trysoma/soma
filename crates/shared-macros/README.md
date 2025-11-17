# shared-macros

Procedural macros for loading SQL migrations at compile time.

## Macros

### `load_sql_migrations!`

Loads traditional `.up.sql` and `.down.sql` migration files from a directory.

**Usage:**
```rust
use shared_macros::load_sql_migrations;

impl SqlMigrationLoader for MyRepository {
    fn load_sql_migrations() -> BTreeMap<&'static str, BTreeMap<&'static str, &'static str>> {
        load_sql_migrations!("migrations")
    }
}
```

**Migration file format:**
- Files must be named with `.up.sql` or `.down.sql` suffix
- Example: `001_create_users.up.sql`, `001_create_users.down.sql`

### `load_atlas_sql_migrations!`

Loads goose-format migration files that contain both up and down migrations in a single file.

**Usage:**
```rust
use shared_macros::load_atlas_sql_migrations;

impl SqlMigrationLoader for MyRepository {
    fn load_sql_migrations() -> BTreeMap<&'static str, BTreeMap<&'static str, &'static str>> {
        load_atlas_sql_migrations!("migrations")
    }
}
```

**Migration file format:**

Goose migrations use comment markers to separate up and down migrations in a single file:

```sql
-- +goose Up
-- create "users" table
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL
);

INSERT INTO users (id, name, email) VALUES (1, 'Alice', 'alice@example.com');

-- +goose Down
-- reverse: create "users" table
DROP TABLE IF EXISTS users;
```

**Requirements:**
- Must contain `-- +goose Up` section (panics if missing)
- Must contain `-- +goose Down` section (panics if missing)
- File naming: `001_create_users.sql` (no .up/.down suffix needed)
- The macro automatically generates `.up.sql` and `.down.sql` entries from the single file

**Output:**

The macro automatically generates:
- `001_create_users.up.sql` → contains content from `-- +goose Up` section
- `001_create_users.down.sql` → contains content from `-- +goose Down` section

These are added to the BTreeMap just like traditional migrations.

## Backend Support

Both macros support backend-specific migrations:
- Use `.sqlite.` in filename for SQLite-only: `001_create_users.sqlite.sql`
- Without backend suffix, migration applies to all supported backends

Currently supported backends:
- `sqlite`
