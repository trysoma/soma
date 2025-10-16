CREATE TABLE IF NOT EXISTS resource_server_credential (
    id TEXT PRIMARY KEY,
    type_id TEXT NOT NULL,
    metadata JSON NOT NULL,
    value JSON NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    next_rotation_time DATETIME
);

CREATE TABLE IF NOT EXISTS user_credential (
    id TEXT PRIMARY KEY,
    type_id TEXT NOT NULL,
    metadata JSON NOT NULL,
    value JSON NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    next_rotation_time DATETIME
);

CREATE TABLE IF NOT EXISTS provider_instance (
    id TEXT PRIMARY KEY,
    resource_server_credential_id TEXT NOT NULL,
    user_credential_id TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    provider_controller_type_id TEXT NOT NULL,
    credential_controller_type_id TEXT NOT NULL,

    FOREIGN KEY (resource_server_credential_id) REFERENCES resource_server_credential(id),
    FOREIGN KEY (user_credential_id) REFERENCES user_credential(id)
);

CREATE TABLE IF NOT EXISTS function_instance (
    id TEXT PRIMARY KEY,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    provider_instance_id TEXT NOT NULL,
    function_controller_type_id TEXT NOT NULL,

    FOREIGN KEY (provider_instance_id) REFERENCES provider_instance(id)
);

CREATE TABLE IF NOT EXISTS broker_state (
    id TEXT PRIMARY KEY,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    resource_server_cred_id TEXT NOT NULL,
    provider_controller_type_id TEXT NOT NULL,
    credential_controller_type_id TEXT NOT NULL,
    metadata JSON NOT NULL,
    action JSON NOT NULL,

    FOREIGN KEY (resource_server_cred_id) REFERENCES resource_server_credential(id)
);

CREATE TABLE IF NOT EXISTS data_encryption_key (
    id TEXT PRIMARY KEY,
    envelope_encryption_key_id JSON NOT NULL,
    encryption_key TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);