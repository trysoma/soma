CREATE TABLE IF NOT EXISTS task (
    id TEXT PRIMARY KEY,
    context_id TEXT NOT NULL,
    status TEXT NOT NULL,
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