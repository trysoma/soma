-- +goose Up
-- Drop old api_key table and recreate with new schema
-- Note: SQLite doesn't support ALTER COLUMN, so we need to recreate the table

-- First, save any existing data
CREATE TABLE IF NOT EXISTS api_key_backup AS SELECT * FROM api_key;

-- Drop the old table
DROP TABLE IF EXISTS api_key;

-- Create new table with id as primary key and description field
CREATE TABLE IF NOT EXISTS api_key (
    id TEXT NOT NULL PRIMARY KEY,
    hashed_value TEXT NOT NULL UNIQUE,
    description TEXT,
    user_id TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES user(id) ON DELETE CASCADE
);

-- Create index on hashed_value for fast lookups during authentication
CREATE INDEX IF NOT EXISTS idx_api_key_hashed_value ON api_key(hashed_value);

-- Drop backup table
DROP TABLE IF EXISTS api_key_backup;

-- +goose Down
DROP INDEX IF EXISTS idx_api_key_hashed_value;
DROP TABLE IF EXISTS api_key;

CREATE TABLE IF NOT EXISTS api_key (
    hashed_value TEXT NOT NULL PRIMARY KEY,
    user_id TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES user(id) ON DELETE CASCADE
);
