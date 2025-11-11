-- atlas:txtar

-- checks.sql --
-- The assertion below must evaluate to true, ensuring the "users" table doesn't exist.
SELECT NOT EXISTS(SELECT name FROM sqlite_master WHERE type='table' AND name='users');

-- migration.sql --
-- Executed only if the assertion above succeeds.
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_users_email ON users(email);

INSERT INTO users (id, name, email) VALUES (1, 'Alice', 'alice@example.com');

-- down.sql --
-- Used to revert the migration.
DROP INDEX IF EXISTS idx_users_email;
DROP TABLE IF EXISTS users;
