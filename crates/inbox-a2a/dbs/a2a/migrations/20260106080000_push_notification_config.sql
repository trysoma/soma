-- +goose Up
-- Push notification configuration table for A2A protocol
-- Stores webhook URLs for task update notifications per A2A spec section 3.17

CREATE TABLE IF NOT EXISTS push_notification_config (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    url TEXT NOT NULL,
    token TEXT,
    authentication JSON,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (task_id) REFERENCES task(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_push_notification_config_task_id ON push_notification_config(task_id);

-- +goose Down
DROP INDEX IF EXISTS idx_push_notification_config_task_id;
DROP TABLE IF EXISTS push_notification_config;
