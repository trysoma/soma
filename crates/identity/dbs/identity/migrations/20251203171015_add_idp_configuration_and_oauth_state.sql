-- +goose Up
-- IdP configuration for OAuth/OIDC authorization flows
CREATE TABLE IF NOT EXISTS idp_configuration (
    id TEXT NOT NULL PRIMARY KEY,
    -- Type: 'oidc_authorization_flow' or 'oauth_authorization_flow'
    type TEXT NOT NULL,
    -- JSON configuration object containing provider settings
    config TEXT NOT NULL,
    -- Encrypted client secret (optional, for confidential clients)
    encrypted_client_secret TEXT,
    -- DEK alias for decrypting client secret
    dek_alias TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT type_check CHECK (type IN ('oidc_authorization_flow', 'oauth_authorization_flow'))
);

-- OAuth state table for CSRF protection and PKCE verifier storage
CREATE TABLE IF NOT EXISTS oauth_state (
    state TEXT NOT NULL PRIMARY KEY,
    -- Reference to the IdP configuration
    config_id TEXT NOT NULL,
    -- PKCE code verifier (stored only for PKCE flows)
    code_verifier TEXT,
    -- OIDC nonce (stored only for OIDC flows)
    nonce TEXT,
    -- Where to redirect user after successful authentication
    redirect_uri TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME NOT NULL,
    FOREIGN KEY (config_id) REFERENCES idp_configuration(id) ON DELETE CASCADE
);

-- Index for cleanup of expired states
CREATE INDEX IF NOT EXISTS idx_oauth_state_expires_at ON oauth_state(expires_at);

-- +goose Down
DROP INDEX IF EXISTS idx_oauth_state_expires_at;
DROP TABLE IF EXISTS oauth_state;
DROP TABLE IF EXISTS idp_configuration;
