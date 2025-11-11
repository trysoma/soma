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

Loads Atlas-format migration files that contain both up and down migrations in a single file.

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

Atlas migrations use the txtar format to combine checks, migration, and rollback SQL in a single file:

```sql
-- atlas:txtar

-- checks.sql --
-- Optional: Assertions that must pass before migration runs
SELECT NOT EXISTS(SELECT name FROM sqlite_master WHERE type='table' AND name='users');

-- migration.sql --
-- Required: The actual migration SQL
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL
);

INSERT INTO users (id, name, email) VALUES (1, 'Alice', 'alice@example.com');

-- down.sql --
-- Required: SQL to revert the migration
DROP TABLE IF EXISTS users;
```

**Requirements:**
- File must start with `-- atlas:txtar` header (panics if missing)
- Must contain `-- migration.sql --` section (panics if missing)
- Must contain `-- down.sql --` section (panics if missing)
- Optional `-- checks.sql --` section for pre-migration validation
- File naming: `001_create_users.sql` (no .up/.down suffix needed)

**Output:**

The macro automatically generates:
- `001_create_users.up.sql` → contains content from `-- migration.sql --`
- `001_create_users.down.sql` → contains content from `-- down.sql --`

These are added to the BTreeMap just like traditional migrations.

## Backend Support

Both macros support backend-specific migrations:
- Use `.sqlite.` in filename for SQLite-only: `001_create_users.sqlite.sql`
- Without backend suffix, migration applies to all supported backends

Currently supported backends:
- `sqlite`
