-- atlas:txtar

-- checks.sql --
-- Ensure users table exists and settings table doesn't
SELECT EXISTS(SELECT name FROM sqlite_master WHERE type='table' AND name='users')
  AND NOT EXISTS(SELECT name FROM sqlite_master WHERE type='table' AND name='user_settings');

-- migration.sql --
-- Create user settings table
CREATE TABLE user_settings (
    user_id INTEGER PRIMARY KEY,
    theme TEXT NOT NULL DEFAULT 'light',
    notifications_enabled BOOLEAN DEFAULT 1,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Add settings for existing user
INSERT INTO user_settings (user_id, theme, notifications_enabled)
VALUES (1, 'dark', 1);

-- down.sql --
-- Remove user settings
DROP TABLE IF EXISTS user_settings;
