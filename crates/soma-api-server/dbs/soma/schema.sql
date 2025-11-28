CREATE TABLE IF NOT EXISTS task (
    id TEXT PRIMARY KEY,
    context_id TEXT NOT NULL,
    status TEXT NOT NULL,
    status_message_id TEXT,
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

CREATE TABLE IF NOT EXISTS message (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    reference_task_ids JSON NOT NULL,
    role TEXT NOT NULL,
    metadata JSON NOT NULL,
    parts JSON NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS message_v2 (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    reference_task_ids JSON NOT NULL,
    role TEXT NOT NULL,
    metadata JSON NOT NULL,
    parts JSON NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS secret (
    id TEXT PRIMARY KEY,
    key TEXT NOT NULL UNIQUE,
    encrypted_secret TEXT NOT NULL,
    dek_alias TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS environment_variable (
    id TEXT PRIMARY KEY,
    key TEXT NOT NULL UNIQUE,
    value TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
