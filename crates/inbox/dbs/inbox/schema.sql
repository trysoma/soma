-- Inbox crate database schema
-- Contains tables for threads, messages, events, and inbox instances

-- Threads table - groups related messages
CREATE TABLE IF NOT EXISTS thread (
    id TEXT PRIMARY KEY,
    title TEXT,
    metadata JSON,
    inbox_settings JSON NOT NULL DEFAULT '{}',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_thread_created_at ON thread(created_at);

-- Messages table - UI messages following Vercel AI SDK spec
CREATE TABLE IF NOT EXISTS message (
    id TEXT PRIMARY KEY,
    thread_id TEXT NOT NULL,
    role TEXT NOT NULL,
    parts JSON NOT NULL,
    metadata JSON,
    inbox_settings JSON NOT NULL DEFAULT '{}',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT role_check CHECK (role IN ('system', 'user', 'assistant')),
    FOREIGN KEY (thread_id) REFERENCES thread(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_message_thread_id ON message(thread_id);
CREATE INDEX IF NOT EXISTS idx_message_created_at ON message(created_at);
CREATE INDEX IF NOT EXISTS idx_message_thread_created ON message(thread_id, created_at);

-- Events table - inbox events for persistence and replay
CREATE TABLE IF NOT EXISTS event (
    id TEXT PRIMARY KEY,
    kind TEXT NOT NULL,
    payload JSON NOT NULL,
    inbox_id TEXT,
    inbox_settings JSON NOT NULL DEFAULT '{}',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_event_created_at ON event(created_at);
CREATE INDEX IF NOT EXISTS idx_event_inbox_id ON event(inbox_id);
CREATE INDEX IF NOT EXISTS idx_event_kind ON event(kind);

-- Inbox instances table - configured inbox provider instances
CREATE TABLE IF NOT EXISTS inbox (
    id TEXT PRIMARY KEY,
    provider_id TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'enabled',
    configuration JSON NOT NULL,
    settings JSON NOT NULL DEFAULT '{}',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT status_check CHECK (status IN ('enabled', 'disabled'))
);

CREATE INDEX IF NOT EXISTS idx_inbox_provider_id ON inbox(provider_id);
CREATE INDEX IF NOT EXISTS idx_inbox_status ON inbox(status);
