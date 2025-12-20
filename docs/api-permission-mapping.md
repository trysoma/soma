# API Permission Mapping

This document maps each API endpoint to a fine-grained permission identifier for implementing authorization checks.

## Permission Naming Convention

Permissions follow the pattern: `{resource}:{action}` where:
- `resource` - The resource type (e.g., `provider`, `mcp_server`, `user`, `dek`)
- `action` - The specific action (e.g., `read`, `write`, `delete`, `invoke`, `list`)

Common actions:
- `read` - Read a single resource
- `list` - List multiple resources
- `write` - Create or update a resource
- `delete` - Delete a resource
- `invoke` - Execute/invoke a function or action

---

## Roles

The following roles are defined in the system (see `crates/shared/src/identity.rs`):

| Role | Description | Access Level |
|------|-------------|--------------|
| `Admin` | Full system administrator | All permissions |
| `Maintainer` | Can view system configurations and resources | Read-only access to most resources |
| `Agent` | Automated agent for task execution | Task operations, function invocation, MCP access |
| `User` | Basic authenticated user | Limited access (auth operations only) |

### Role Hierarchy

```
Admin
  └── Maintainer
        └── User

Agent (separate branch - task/function focused)
```

---

## Routes and Permissions

### Bridge Routes (`/api/bridge/v1`)

#### Provider Management

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| GET | `/api/bridge/v1/available-providers` | `provider:list` | List available provider types | Admin, Maintainer |
| POST | `/api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}` | `provider_instance:write` | Create a new provider instance | Admin |
| PATCH | `/api/bridge/v1/provider/{provider_instance_id}` | `provider_instance:write` | Update an existing provider instance | Admin |
| GET | `/api/bridge/v1/provider/{provider_instance_id}` | `provider_instance:read` | Get a specific provider instance | Admin, Maintainer, Agent |
| DELETE | `/api/bridge/v1/provider/{provider_instance_id}` | `provider_instance:delete` | Delete a provider instance | Admin |
| GET | `/api/bridge/v1/provider` | `provider_instance:list` | List all provider instances | Admin, Maintainer, Agent |
| GET | `/api/bridge/v1/provider/grouped-by-function` | `provider_instance:list` | List provider instances grouped by function | Admin, Maintainer, Agent |

#### Credential Management

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| POST | `/api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/resource-server/encrypt` | `credential:encrypt` | Encrypt resource server configuration | Admin |
| POST | `/api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/user-credential/encrypt` | `credential:encrypt` | Encrypt user credential configuration | Admin |
| POST | `/api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/resource-server` | `credential:write` | Create a resource server credential | Admin |
| POST | `/api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/user-credential` | `credential:write` | Create a user credential | Admin |
| POST | `/api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/user-credential/broker` | `credential:broker` | Start user credential brokering flow | Admin |
| GET | `/api/bridge/v1/generic-oauth-callback` | - | Handle OAuth callback | *Public* |

#### Function Management

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| POST | `/api/bridge/v1/provider/{provider_instance_id}/function/{function_controller_type_id}/enable` | `mcp_function:enable` | Enable a function on a provider | Admin |
| POST | `/api/bridge/v1/provider/{provider_instance_id}/function/{function_controller_type_id}/disable` | `mcp_function:disable` | Disable a function on a provider | Admin |
| POST | `/api/bridge/v1/provider/{provider_instance_id}/function/{function_controller_type_id}/invoke` | `mcp_function:invoke` | Invoke a function | Admin, Agent |
| GET | `/api/bridge/v1/function-instances` | `mcp_function:list` | List all function instances | Admin, Maintainer, Agent |
| GET | `/api/bridge/v1/function-instances/openapi.json` | `mcp_function:read_openapi` | Get OpenAPI spec for functions | Admin, Maintainer, Agent |

#### MCP Protocol

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| GET | `/api/bridge/v1/mcp` | `mcp:connect` | Establish MCP SSE connection | Admin, Agent |
| POST | `/api/bridge/v1/mcp` | `mcp:message` | Send MCP message | Admin, Agent |

#### MCP Server Instance Management

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| POST | `/api/bridge/v1/mcp-server` | `mcp_server:write` | Create MCP server instance | Admin |
| GET | `/api/bridge/v1/mcp-server/{mcp_server_instance_id}` | `mcp_server:read` | Get MCP server instance | Admin, Maintainer, Agent |
| PATCH | `/api/bridge/v1/mcp-server/{mcp_server_instance_id}` | `mcp_server:write` | Update MCP server instance | Admin |
| DELETE | `/api/bridge/v1/mcp-server/{mcp_server_instance_id}` | `mcp_server:delete` | Delete MCP server instance | Admin |
| POST | `/api/bridge/v1/mcp-server/{mcp_server_instance_id}/function` | `mcp_server_function:write` | Add function to MCP server | Admin |
| PATCH | `/api/bridge/v1/mcp-server/{mcp_server_instance_id}/function/{function_id}` | `mcp_server_function:write` | Update MCP server function | Admin |
| DELETE | `/api/bridge/v1/mcp-server/{mcp_server_instance_id}/function/{function_id}` | `mcp_server_function:delete` | Remove function from MCP server | Admin |
| * | `/api/bridge/v1/mcp-server/{mcp_server_instance_id}/mcp` | `mcp_server:connect` | MCP protocol handler (SSE/streaming) | Admin, Agent |

---

### Task Routes (`/api/task/v1`)

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| GET | `/api/task/v1` | `task:list` | List all tasks | Admin, Maintainer, Agent |
| GET | `/api/task/v1/context` | `task_context:list` | List all contexts | Admin, Maintainer, Agent |
| GET | `/api/task/v1/context/{context_id}/task` | `task:list` | List tasks by context | Admin, Maintainer, Agent |
| GET | `/api/task/v1/{task_id}` | `task:read` | Get a specific task | Admin, Maintainer, Agent |
| PUT | `/api/task/v1/{task_id}` | `task:write` | Update task status | Admin, Agent |
| POST | `/api/task/v1/{task_id}/message` | `task_message:write` | Send message to task | Admin, Agent |
| GET | `/api/task/v1/{task_id}/timeline` | `task_timeline:read` | Get task timeline items | Admin, Maintainer, Agent |

---

### Agent Routes (`/api/agent`)

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| GET | `/api/agent` | `agent:list` | List available agents | Admin, Maintainer, Agent |
| GET | `/api/agent/{project_id}/{agent_id}/a2a/.well-known/agent.json` | `agent:read` | Get agent card | Admin, Maintainer, Agent |
| POST | `/api/agent/{project_id}/{agent_id}/a2a` | `agent:execute` | Handle A2A JSON-RPC | Admin, Agent |

---

### A2A Routes (`/api/a2a/v1`)

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| GET | `/api/a2a/v1/definition` | `a2a:read` | Get agent definition | Admin, Maintainer, Agent |
| POST | `/api/a2a/v1/...` | `a2a:execute` | A2A protocol execution (via a2a_rs) | Admin, Agent |

---

### Secret Routes (`/api/secret/v1`)

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| POST | `/api/secret/v1` | `secret:write` | Create a new secret | Admin |
| POST | `/api/secret/v1/import` | `secret:import` | Import a secret | Admin |
| GET | `/api/secret/v1` | `secret:list` | List all secrets | Admin, Maintainer |
| GET | `/api/secret/v1/list-decrypted` | `secret:read_decrypted` | List secrets with decrypted values | Admin |
| GET | `/api/secret/v1/{secret_id}` | `secret:read` | Get a secret by ID | Admin, Maintainer |
| GET | `/api/secret/v1/key/{key}` | `secret:read` | Get a secret by key | Admin, Maintainer |
| PUT | `/api/secret/v1/{secret_id}` | `secret:write` | Update a secret | Admin |
| DELETE | `/api/secret/v1/{secret_id}` | `secret:delete` | Delete a secret | Admin |

---

### Environment Variable Routes (`/api/environment-variable/v1`)

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| POST | `/api/environment-variable/v1` | `env_var:write` | Create an environment variable | Admin |
| POST | `/api/environment-variable/v1/import` | `env_var:import` | Import an environment variable | Admin |
| GET | `/api/environment-variable/v1` | `env_var:list` | List environment variables | Admin, Maintainer, Agent |
| GET | `/api/environment-variable/v1/{env_var_id}` | `env_var:read` | Get an environment variable by ID | Admin, Maintainer, Agent |
| GET | `/api/environment-variable/v1/key/{key}` | `env_var:read` | Get an environment variable by key | Admin, Maintainer, Agent |
| PUT | `/api/environment-variable/v1/{env_var_id}` | `env_var:write` | Update an environment variable | Admin |
| DELETE | `/api/environment-variable/v1/{env_var_id}` | `env_var:delete` | Delete an environment variable | Admin |

---

### Encryption Routes (`/api/encryption/v1`)

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| POST | `/api/encryption/v1/envelope` | `envelope_key:write` | Create an envelope encryption key | Admin |
| GET | `/api/encryption/v1/envelope` | `envelope_key:list` | List envelope encryption keys | Admin, Maintainer |
| POST | `/api/encryption/v1/envelope/{envelope_id}/dek` | `dek:write` | Create a data encryption key | Admin |
| POST | `/api/encryption/v1/envelope/{envelope_id}/dek/import` | `dek:import` | Import a data encryption key | Admin |
| GET | `/api/encryption/v1/envelope/{envelope_id}/dek` | `dek:list` | List data encryption keys | Admin, Maintainer |
| POST | `/api/encryption/v1/envelope/{envelope_id}/dek/{dek_id}/migrate` | `dek:migrate` | Migrate a data encryption key | Admin |
| POST | `/api/encryption/v1/envelope/{envelope_id}/migrate` | `dek:migrate_all` | Migrate all data encryption keys | Admin |
| POST | `/api/encryption/v1/dek/alias` | `dek_alias:write` | Create a DEK alias | Admin |
| GET | `/api/encryption/v1/dek/alias/{alias}` | `dek_alias:read` | Get DEK by alias or ID | Admin, Maintainer |
| PUT | `/api/encryption/v1/dek/alias/{alias}` | `dek_alias:write` | Update a DEK alias | Admin |
| DELETE | `/api/encryption/v1/dek/alias/{alias}` | `dek_alias:delete` | Delete a DEK alias | Admin |

---

### Identity Routes (`/api/identity/v1`)

#### API Key Management

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| POST | `/api/identity/v1/api-key` | `api_key:write` | Create an API key | Admin |
| DELETE | `/api/identity/v1/api-key/{id}` | `api_key:delete` | Delete an API key | Admin |
| GET | `/api/identity/v1/api-key` | `api_key:list` | List API keys | Admin, Maintainer |
| POST | `/api/identity/v1/api-key/import` | `api_key:import` | Import an API key | Admin |

#### Authentication

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| GET | `/api/identity/v1/auth/authorize/{config_id}` | - | Start authorization flow | *Public* |
| GET | `/api/identity/v1/auth/callback` | - | Handle auth callback | *Public* |
| POST | `/api/identity/v1/auth/refresh` | `auth:refresh` | Refresh access token | Admin, Maintainer, Agent, User |
| GET | `/api/identity/v1/auth/whoami` | `auth:whoami` | Get current identity | Admin, Maintainer, Agent, User |

#### JWK Management

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| POST | `/api/identity/v1/jwk/{kid}/invalidate` | `jwk:invalidate` | Invalidate a JWK | Admin |
| GET | `/api/identity/v1/jwk` | `jwk:list` | List JWKs | Admin, Maintainer |
| GET | `/api/identity/v1/.well-known/jwks.json` | - | Get JWKS | *Public* |

#### STS Configuration

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| POST | `/api/identity/v1/sts-configuration` | `sts_config:write` | Create STS configuration | Admin |
| GET | `/api/identity/v1/sts-configuration/{id}` | `sts_config:read` | Get STS configuration | Admin, Maintainer |
| DELETE | `/api/identity/v1/sts-configuration/{id}` | `sts_config:delete` | Delete STS configuration | Admin |
| GET | `/api/identity/v1/sts-configuration` | `sts_config:list` | List STS configurations | Admin, Maintainer |

#### STS Token Exchange

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| POST | `/api/identity/v1/sts/{sts_config_id}` | `sts:exchange` | Exchange STS token | Admin, Agent |

#### User Auth Flow Configuration

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| POST | `/api/identity/v1/user-auth-flow-config` | `user_auth_flow_config:write` | Create user auth flow configuration | Admin |
| GET | `/api/identity/v1/user-auth-flow-config/{id}` | `user_auth_flow_config:read` | Get user auth flow configuration | Admin, Maintainer |
| DELETE | `/api/identity/v1/user-auth-flow-config/{id}` | `user_auth_flow_config:delete` | Delete user auth flow configuration | Admin |
| GET | `/api/identity/v1/user-auth-flow-config` | `user_auth_flow_config:list` | List user auth flow configurations | Admin, Maintainer |
| POST | `/api/identity/v1/user-auth-flow-config/import` | `user_auth_flow_config:import` | Import user auth flow configuration | Admin |

#### User Management

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| POST | `/api/identity/v1/users` | `user:write` | Create a user | Admin |
| GET | `/api/identity/v1/users/{user_id}` | `user:read` | Get a user | Admin, Maintainer |
| PATCH | `/api/identity/v1/users/{user_id}` | `user:write` | Update a user | Admin |
| DELETE | `/api/identity/v1/users/{user_id}` | `user:delete` | Delete a user | Admin |
| GET | `/api/identity/v1/users` | `user:list` | List users | Admin, Maintainer |
| GET | `/api/identity/v1/users/{user_id}/groups` | `user_group:list` | List groups for a user | Admin, Maintainer |

#### Group Management

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| POST | `/api/identity/v1/groups` | `group:write` | Create a group | Admin |
| GET | `/api/identity/v1/groups/{group_id}` | `group:read` | Get a group | Admin, Maintainer |
| PATCH | `/api/identity/v1/groups/{group_id}` | `group:write` | Update a group | Admin |
| DELETE | `/api/identity/v1/groups/{group_id}` | `group:delete` | Delete a group | Admin |
| GET | `/api/identity/v1/groups` | `group:list` | List groups | Admin, Maintainer |

#### Group Membership

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| POST | `/api/identity/v1/groups/{group_id}/members` | `group_member:write` | Add member to group | Admin |
| DELETE | `/api/identity/v1/groups/{group_id}/members/{user_id}` | `group_member:delete` | Remove member from group | Admin |
| GET | `/api/identity/v1/groups/{group_id}/members` | `group_member:list` | List group members | Admin, Maintainer |

---

### SCIM 2.0 Routes (`/api/identity/scim/v2`)

SCIM (System for Cross-domain Identity Management) endpoints for enterprise identity provisioning.

#### SCIM Users

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| GET | `/api/identity/scim/v2/Users` | `scim_user:list` | List SCIM users | Admin |
| POST | `/api/identity/scim/v2/Users` | `scim_user:write` | Create SCIM user | Admin |
| GET | `/api/identity/scim/v2/Users/{user_id}` | `scim_user:read` | Get SCIM user | Admin |
| PUT | `/api/identity/scim/v2/Users/{user_id}` | `scim_user:write` | Replace SCIM user | Admin |
| PATCH | `/api/identity/scim/v2/Users/{user_id}` | `scim_user:write` | Patch SCIM user | Admin |
| DELETE | `/api/identity/scim/v2/Users/{user_id}` | `scim_user:delete` | Delete SCIM user | Admin |

#### SCIM Groups

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| GET | `/api/identity/scim/v2/Groups` | `scim_group:list` | List SCIM groups | Admin |
| POST | `/api/identity/scim/v2/Groups` | `scim_group:write` | Create SCIM group | Admin |
| GET | `/api/identity/scim/v2/Groups/{group_id}` | `scim_group:read` | Get SCIM group | Admin |
| PUT | `/api/identity/scim/v2/Groups/{group_id}` | `scim_group:write` | Replace SCIM group | Admin |
| PATCH | `/api/identity/scim/v2/Groups/{group_id}` | `scim_group:write` | Patch SCIM group | Admin |
| DELETE | `/api/identity/scim/v2/Groups/{group_id}` | `scim_group:delete` | Delete SCIM group | Admin |

---

### Internal Routes (`/_internal/v1`)

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| GET | `/_internal/v1/health` | - | Health check | *Public* |
| GET | `/_internal/v1/runtime_config` | `runtime_config:read` | Get runtime configuration | Admin, Maintainer |
| POST | `/_internal/v1/trigger_codegen` | `codegen:trigger` | Trigger code generation | Admin |
| POST | `/_internal/v1/resync_sdk` | `sdk:resync` | Resync SDK | Admin |

---

## Role-Permission Matrix

| Permission | Admin | Maintainer | Agent | User |
|------------|:-----:|:----------:|:-----:|:----:|
| **Provider & Instance** |
| `provider:list` | X | X | | |
| `provider_instance:write` | X | | | |
| `provider_instance:read` | X | X | X | |
| `provider_instance:delete` | X | | | |
| `provider_instance:list` | X | X | X | |
| **Credential** |
| `credential:encrypt` | X | | | |
| `credential:write` | X | | | |
| `credential:broker` | X | | | |
| **MCP Function** |
| `mcp_function:enable` | X | | | |
| `mcp_function:disable` | X | | | |
| `mcp_function:invoke` | X | | X | |
| `mcp_function:list` | X | X | X | |
| `mcp_function:read_openapi` | X | X | X | |
| **MCP Protocol** |
| `mcp:connect` | X | | X | |
| `mcp:message` | X | | X | |
| **MCP Server** |
| `mcp_server:write` | X | | | |
| `mcp_server:read` | X | X | X | |
| `mcp_server:delete` | X | | | |
| `mcp_server:connect` | X | | X | |
| `mcp_server_function:write` | X | | | |
| `mcp_server_function:delete` | X | | | |
| **Task** |
| `task:list` | X | X | X | |
| `task:read` | X | X | X | |
| `task:write` | X | | X | |
| `task_context:list` | X | X | X | |
| `task_message:write` | X | | X | |
| `task_timeline:read` | X | X | X | |
| **Agent** |
| `agent:list` | X | X | X | |
| `agent:read` | X | X | X | |
| `agent:execute` | X | | X | |
| **A2A** |
| `a2a:read` | X | X | X | |
| `a2a:execute` | X | | X | |
| **Secret** |
| `secret:write` | X | | | |
| `secret:import` | X | | | |
| `secret:list` | X | X | | |
| `secret:read` | X | X | | |
| `secret:read_decrypted` | X | | | |
| `secret:delete` | X | | | |
| **Environment Variable** |
| `env_var:write` | X | | | |
| `env_var:import` | X | | | |
| `env_var:list` | X | X | X | |
| `env_var:read` | X | X | X | |
| `env_var:delete` | X | | | |
| **Encryption - Envelope Key** |
| `envelope_key:write` | X | | | |
| `envelope_key:list` | X | X | | |
| **Encryption - DEK** |
| `dek:write` | X | | | |
| `dek:import` | X | | | |
| `dek:list` | X | X | | |
| `dek:migrate` | X | | | |
| `dek:migrate_all` | X | | | |
| **Encryption - DEK Alias** |
| `dek_alias:write` | X | | | |
| `dek_alias:read` | X | X | | |
| `dek_alias:delete` | X | | | |
| **API Key** |
| `api_key:write` | X | | | |
| `api_key:delete` | X | | | |
| `api_key:list` | X | X | | |
| `api_key:import` | X | | | |
| **Auth** |
| `auth:refresh` | X | X | X | X |
| `auth:whoami` | X | X | X | X |
| **JWK** |
| `jwk:invalidate` | X | | | |
| `jwk:list` | X | X | | |
| **STS Config** |
| `sts_config:write` | X | | | |
| `sts_config:read` | X | X | | |
| `sts_config:delete` | X | | | |
| `sts_config:list` | X | X | | |
| **STS** |
| `sts:exchange` | X | | X | |
| **User Auth Flow Config** |
| `user_auth_flow_config:write` | X | | | |
| `user_auth_flow_config:read` | X | X | | |
| `user_auth_flow_config:delete` | X | | | |
| `user_auth_flow_config:list` | X | X | | |
| `user_auth_flow_config:import` | X | | | |
| **User** |
| `user:write` | X | | | |
| `user:read` | X | X | | |
| `user:delete` | X | | | |
| `user:list` | X | X | | |
| `user_group:list` | X | X | | |
| **Group** |
| `group:write` | X | | | |
| `group:read` | X | X | | |
| `group:delete` | X | | | |
| `group:list` | X | X | | |
| **Group Membership** |
| `group_member:write` | X | | | |
| `group_member:delete` | X | | | |
| `group_member:list` | X | X | | |
| **SCIM User** |
| `scim_user:write` | X | | | |
| `scim_user:read` | X | | | |
| `scim_user:delete` | X | | | |
| `scim_user:list` | X | | | |
| **SCIM Group** |
| `scim_group:write` | X | | | |
| `scim_group:read` | X | | | |
| `scim_group:delete` | X | | | |
| `scim_group:list` | X | | | |
| **Internal** |
| `runtime_config:read` | X | X | | |
| `codegen:trigger` | X | | | |
| `sdk:resync` | X | | | |

---

## Public Endpoints (No Auth Required)

The following endpoints are intentionally public and do not require authentication:

| Endpoint | Reason |
|----------|--------|
| `GET /_internal/v1/health` | Health check for load balancers/orchestrators |
| `GET /api/identity/v1/.well-known/jwks.json` | Standard OAuth2 JWKS endpoint |
| `GET /api/identity/v1/auth/authorize/{config_id}` | Authorization flow initiation |
| `GET /api/identity/v1/auth/callback` | OAuth callback handler |
| `GET /api/bridge/v1/generic-oauth-callback` | OAuth callback for credential brokering |

---

## Statistics

- **Total Endpoints**: 111
- **Total Unique Permissions**: 72
- **Roles**: 4
- **Public Endpoints**: 5
