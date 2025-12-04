CREATE TABLE IF NOT EXISTS api_key (
    id TEXT NOT NULL PRIMARY KEY,
    hashed_value TEXT NOT NULL UNIQUE,
    description TEXT,
    user_id TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES user(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_api_key_hashed_value ON api_key(hashed_value);

CREATE TABLE IF NOT EXISTS user (
    id TEXT PRIMARY KEY,
    type TEXT NOT NULL,
    email TEXT,
    role TEXT NOT NULL,
    description TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT type_check CHECK (type IN ('machine', 'human'))
);

CREATE TABLE IF NOT EXISTS `group` (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS group_membership (
    group_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (group_id, user_id),
    FOREIGN KEY (group_id) REFERENCES `group`(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES user(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS jwt_signing_key (
    kid TEXT NOT NULL PRIMARY KEY,
    encrypted_private_key TEXT NOT NULL,
    expires_at DATETIME NOT NULL,
    public_key TEXT NOT NULL,
    dek_alias TEXT NOT NULL,
    invalidated BOOLEAN NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS sts_configuration (
    id TEXT NOT NULL PRIMARY KEY,
    type TEXT NOT NULL,
    value TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT type_check CHECK (type IN ('jwt_template', 'dev'))
);

CREATE TABLE IF NOT EXISTS user_auth_flow_configuration (
    id TEXT NOT NULL PRIMARY KEY,
    type TEXT NOT NULL,
    config TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT type_check CHECK (type IN ('oidc_authorization_code_flow', 'oauth_authorization_code_flow', 'oidc_authorization_code_pkce_flow', 'oauth_authorization_code_pkce_flow'))
);

CREATE TABLE IF NOT EXISTS oauth_state (
    state TEXT NOT NULL PRIMARY KEY,
    config_id TEXT NOT NULL,
    code_verifier TEXT,
    nonce TEXT,
    redirect_uri TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME NOT NULL,
    FOREIGN KEY (config_id) REFERENCES user_auth_flow_configuration(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_oauth_state_expires_at ON oauth_state(expires_at);