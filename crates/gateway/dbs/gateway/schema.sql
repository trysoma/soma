-- Gateway database schema
-- This will store gateway-specific data like request logs, API keys, usage metrics, etc.

-- Example: Request logs table
CREATE TABLE IF NOT EXISTS request_log (
    id TEXT PRIMARY KEY,
    model TEXT NOT NULL,
    prompt_tokens INTEGER NOT NULL,
    completion_tokens INTEGER NOT NULL,
    total_tokens INTEGER NOT NULL,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL
);

-- Example: API keys table (encrypted)
CREATE TABLE IF NOT EXISTS api_key (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    encrypted_key TEXT NOT NULL,
    data_encryption_key_id TEXT NOT NULL,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL
);
