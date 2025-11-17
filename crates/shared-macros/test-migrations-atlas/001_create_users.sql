-- +goose Up
-- create "users" table
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_users_email ON users(email);

INSERT INTO users (id, name, email) VALUES (1, 'Alice', 'alice@example.com');

-- +goose Down
-- reverse: create "users" table
DROP INDEX IF EXISTS idx_users_email;
DROP TABLE IF EXISTS users;
