CREATE TABLE IF NOT EXISTS resource_server_credential (
    id TEXT PRIMARY KEY,
    type_id TEXT NOT NULL,
    metadata JSON NOT NULL,
    value JSON NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    next_rotation_time DATETIME,
    dek_alias TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS user_credential (
    id TEXT PRIMARY KEY,
    type_id TEXT NOT NULL,
    metadata JSON NOT NULL,
    value JSON NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    next_rotation_time DATETIME,
    dek_alias TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS provider_instance (
    id TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    resource_server_credential_id TEXT NOT NULL,
    user_credential_id TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    provider_controller_type_id TEXT NOT NULL,
    credential_controller_type_id TEXT NOT NULL,
    status TEXT NOT NULL,
    return_on_successful_brokering JSON,
    FOREIGN KEY (resource_server_credential_id) REFERENCES resource_server_credential(id),
    FOREIGN KEY (user_credential_id) REFERENCES user_credential(id),
    CHECK (status IN ('brokering_initiated', 'active', 'disabled'))
);

CREATE TABLE IF NOT EXISTS function_instance (
    function_controller_type_id TEXT NOT NULL,
    provider_controller_type_id TEXT NOT NULL,
    provider_instance_id TEXT NOT NULL,

    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (function_controller_type_id, provider_controller_type_id, provider_instance_id),
    FOREIGN KEY (provider_instance_id) REFERENCES provider_instance(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS broker_state (
    id TEXT PRIMARY KEY,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    provider_instance_id TEXT NOT NULL,
    provider_controller_type_id TEXT NOT NULL,
    credential_controller_type_id TEXT NOT NULL,
    metadata JSON NOT NULL,
    action JSON NOT NULL

    -- TODO: uncomment this when we have a way to delete broker states
    -- FOREIGN KEY (provider_instance_id) REFERENCES provider_instance(id) ON DELETE CASCADE
);