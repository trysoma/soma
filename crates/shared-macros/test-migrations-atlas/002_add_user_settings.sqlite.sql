-- +goose Up
-- create "user_settings" table
CREATE TABLE user_settings (
    user_id INTEGER PRIMARY KEY,
    theme TEXT NOT NULL DEFAULT 'light',
    notifications_enabled BOOLEAN DEFAULT 1,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Add settings for existing user
INSERT INTO user_settings (user_id, theme, notifications_enabled)
VALUES (1, 'dark', 1);

-- +goose Down
-- reverse: create "user_settings" table
DROP TABLE IF EXISTS user_settings;
