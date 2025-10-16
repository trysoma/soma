CREATE TABLE IF NOT EXISTS resource_server_credential (
    id TEXT PRIMARY KEY,
    credential_type TEXT NOT NULL,
    credential_data JSON NOT NULL,
    metadata JSON NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    run_refresh_before DATETIME,
    
    CONSTRAINT credential_type_check CHECK (credential_type IN ("no_auth", "oauth2_authorization_code_flow", "oauth2_jwt_bearer_assertion_flow", "custom"))
);

CREATE TABLE IF NOT EXISTS user_credential (
    id TEXT PRIMARY KEY,
    credential_type TEXT NOT NULL,
    credential_data JSON NOT NULL,
    metadata JSON NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    run_refresh_before DATETIME,

    CONSTRAINT credential_type_check CHECK (credential_type IN ("no_auth", "oauth2_authorization_code_flow", "oauth2_jwt_bearer_assertion_flow", "custom"))
);

CREATE TABLE IF NOT EXISTS provider_instance (
    id TEXT PRIMARY KEY,
    provider_id TEXT NOT NULL,
    resource_server_credential_id TEXT NOT NULL,
    user_credential_id TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT provider_id_check CHECK (provider_id IN ("google_mail"))
);

CREATE TABLE IF NOT EXISTS function_instance (
    id TEXT PRIMARY KEY,
    function_id TEXT NOT NULL,
    provider_instance_id TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (provider_instance_id) REFERENCES provider_instance(id)
);

CREATE TABLE IF NOT EXISTS credential_exchange_state (
    id TEXT PRIMARY KEY,
    state JSON NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);