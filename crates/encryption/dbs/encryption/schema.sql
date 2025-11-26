CREATE TABLE IF NOT EXISTS envelope_encryption_key (
    id TEXT PRIMARY KEY,
    key_type TEXT NOT NULL CHECK (key_type IN ('local', 'aws_kms')),
    local_file_name TEXT,
    aws_arn TEXT,
    aws_region TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CHECK (
        (key_type = 'local' AND local_file_name IS NOT NULL AND aws_arn IS NULL AND aws_region IS NULL) OR
        (key_type = 'aws_kms' AND aws_arn IS NOT NULL AND aws_region IS NOT NULL AND local_file_name IS NULL)
    )
);

CREATE TABLE IF NOT EXISTS data_encryption_key (
    id TEXT PRIMARY KEY,
    envelope_encryption_key_id TEXT NOT NULL,
    encryption_key TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (envelope_encryption_key_id) REFERENCES envelope_encryption_key(id)
);

CREATE TABLE IF NOT EXISTS data_encryption_key_alias (
    alias TEXT PRIMARY KEY,
    data_encryption_key_id TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (data_encryption_key_id) REFERENCES data_encryption_key(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_dek_alias_dek_id ON data_encryption_key_alias(data_encryption_key_id);

