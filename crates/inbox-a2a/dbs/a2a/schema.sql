CREATE TABLE IF NOT EXISTS task (
    id TEXT PRIMARY KEY,
    context_id TEXT NOT NULL,
    status TEXT NOT NULL,
    status_timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    metadata JSON NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT status_check CHECK (status IN ("submitted", "working", "input-required", "completed", "canceled", "failed", "rejected", "auth-required", "unknown"))
);

CREATE TABLE IF NOT EXISTS task_timeline (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    event_update_type TEXT NOT NULL,
    event_payload JSON NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT event_update_type_check CHECK (event_update_type IN ('task-status-update', 'message'))
);

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
