-- +goose Up
ALTER TABLE user ADD COLUMN description TEXT;

-- +goose Down
-- SQLite doesn't support DROP COLUMN in older versions, so we recreate the table
CREATE TABLE user_backup AS SELECT id, type, email, role, created_at, updated_at FROM user;
DROP TABLE user;
CREATE TABLE user (
    id TEXT PRIMARY KEY,
    type TEXT NOT NULL,
    email TEXT,
    role TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT type_check CHECK (type IN ('service_principal', 'federated_user'))
);
INSERT INTO user SELECT * FROM user_backup;
DROP TABLE user_backup;
