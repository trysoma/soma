-- Credential tables
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

-- Tool group deployments are types/definitions of tool groups (e.g., "google_mail", "stripe")
-- These define what integrations are available and their configuration
-- The credential_deployments field caches the credential configuration schemas for quick access
CREATE TABLE IF NOT EXISTS tool_group_deployment (
    type_id TEXT NOT NULL,
    deployment_id TEXT NOT NULL,
    name TEXT NOT NULL,
    documentation TEXT NOT NULL,
    categories JSON NOT NULL,
    endpoint_type TEXT NOT NULL,
    endpoint_configuration JSON NOT NULL,
    credential_deployments JSON NOT NULL, -- Array of credential configuration schemas
    metadata JSON NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (type_id, deployment_id),
    CHECK (endpoint_type IN ('http'))
);

-- Tool deployments are types/definitions of tools within a tool group deployment
-- These define what capabilities/tools are available for a tool group deployment
CREATE TABLE IF NOT EXISTS tool_deployment (
    type_id TEXT NOT NULL,
    tool_group_deployment_type_id TEXT NOT NULL,
    tool_group_deployment_deployment_id TEXT NOT NULL,
    name TEXT NOT NULL,
    documentation TEXT NOT NULL,
    categories JSON NOT NULL,
    metadata JSON NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (type_id, tool_group_deployment_type_id, tool_group_deployment_deployment_id),
    FOREIGN KEY (tool_group_deployment_type_id, tool_group_deployment_deployment_id)
        REFERENCES tool_group_deployment(type_id, deployment_id) ON DELETE CASCADE
);

-- Tool group deployment aliases provide friendly names for tool group deployments
CREATE TABLE IF NOT EXISTS tool_group_deployment_alias (
    tool_group_deployment_type_id TEXT NOT NULL,
    tool_group_deployment_deployment_id TEXT NOT NULL,
    alias TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (tool_group_deployment_type_id, tool_group_deployment_deployment_id, alias),
    FOREIGN KEY (tool_group_deployment_type_id, tool_group_deployment_deployment_id)
        REFERENCES tool_group_deployment(type_id, deployment_id) ON DELETE CASCADE
);

-- Unique index on alias to ensure each alias is unique across all tool groups
CREATE UNIQUE INDEX IF NOT EXISTS idx_tool_group_deployment_alias_unique
    ON tool_group_deployment_alias(alias);

-- Tool groups are instances of tool group deployments (e.g., "my_gmail", "my_stripe")
-- These are user-configured instances with credentials
-- The alias field references tool_group_deployment_alias.alias but is optional (NULL allowed)
CREATE TABLE IF NOT EXISTS tool_group (
    id TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    alias TEXT,
    resource_server_credential_id TEXT NOT NULL,
    user_credential_id TEXT,
    tool_group_deployment_type_id TEXT NOT NULL,
    tool_group_deployment_deployment_id TEXT NOT NULL,
    credential_deployment_type_id TEXT NOT NULL,
    status TEXT NOT NULL,
    return_on_successful_brokering JSON,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (resource_server_credential_id) REFERENCES resource_server_credential(id),
    FOREIGN KEY (user_credential_id) REFERENCES user_credential(id),
    FOREIGN KEY (tool_group_deployment_type_id, tool_group_deployment_deployment_id)
        REFERENCES tool_group_deployment(type_id, deployment_id),
    CHECK (status IN ('brokering_initiated', 'active', 'disabled'))
);

-- Create index for alias lookups (when alias is not NULL)
CREATE INDEX IF NOT EXISTS idx_tool_group_alias
    ON tool_group(alias) WHERE alias IS NOT NULL;

-- Tools are enabled tool deployments within a tool group
-- These represent which capabilities are enabled for a specific tool group instance
CREATE TABLE IF NOT EXISTS tool (
    tool_deployment_type_id TEXT NOT NULL,
    tool_group_deployment_type_id TEXT NOT NULL,
    tool_group_deployment_deployment_id TEXT NOT NULL,
    tool_group_id TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (tool_deployment_type_id, tool_group_deployment_type_id, tool_group_deployment_deployment_id, tool_group_id),
    FOREIGN KEY (tool_deployment_type_id, tool_group_deployment_type_id, tool_group_deployment_deployment_id)
        REFERENCES tool_deployment(type_id, tool_group_deployment_type_id, tool_group_deployment_deployment_id) ON DELETE CASCADE,
    FOREIGN KEY (tool_group_id) REFERENCES tool_group(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS broker_state (
    id TEXT PRIMARY KEY,
    tool_group_id TEXT NOT NULL,
    tool_group_deployment_type_id TEXT NOT NULL,
    tool_group_deployment_deployment_id TEXT NOT NULL,
    credential_deployment_type_id TEXT NOT NULL,
    metadata JSON NOT NULL,
    action JSON NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP

    -- TODO: uncomment this when we have a way to delete broker states
    -- FOREIGN KEY (tool_group_id) REFERENCES tool_group(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS mcp_server_instance (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- MCP server instance tools map tools to MCP protocol functions
CREATE TABLE IF NOT EXISTS mcp_server_instance_tool (
    mcp_server_instance_id TEXT NOT NULL,
    tool_deployment_type_id TEXT NOT NULL,
    tool_group_deployment_type_id TEXT NOT NULL,
    tool_group_deployment_deployment_id TEXT NOT NULL,
    tool_group_id TEXT NOT NULL,
    tool_name TEXT NOT NULL,
    tool_description TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (mcp_server_instance_id, tool_deployment_type_id, tool_group_deployment_type_id, tool_group_deployment_deployment_id, tool_group_id),
    FOREIGN KEY (mcp_server_instance_id) REFERENCES mcp_server_instance(id) ON DELETE CASCADE,
    FOREIGN KEY (tool_deployment_type_id, tool_group_deployment_type_id, tool_group_deployment_deployment_id, tool_group_id)
        REFERENCES tool(tool_deployment_type_id, tool_group_deployment_type_id, tool_group_deployment_deployment_id, tool_group_id) ON DELETE CASCADE
);

-- Ensure tool_name is unique within each MCP server instance
CREATE UNIQUE INDEX IF NOT EXISTS idx_mcp_server_instance_tool_name
    ON mcp_server_instance_tool(mcp_server_instance_id, tool_name);
