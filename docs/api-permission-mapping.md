# API Permission Mapping

This document maps each API endpoint to a fine-grained permission identifier for implementing authorization checks.

## Permission Naming Convention

Permissions follow the pattern: `{domain}:{action}` where:
- `domain` - The resource domain (e.g., `bridge`, `task`, `secret`, `encryption`, `identity`)
- `action` - The specific action (e.g., `read`, `write`, `delete`, `invoke`)

---

## Roles

The following roles are defined in the system (see `crates/shared/src/identity.rs`):

| Role | Description | Access Level |
|------|-------------|--------------|
| `Admin` | Full system administrator | All permissions including create, update, delete |
| `Maintainer` | Can view system configurations and resources | Read-only access to all resources |
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

## Bridge Routes (`/api/bridge/v1`)

### Provider Management

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| GET | `/api/bridge/v1/available-providers` | `bridge:list_available_providers` | List available provider types | Admin, Maintainer |
| POST | `/api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}` | `bridge:create_provider_instance` | Create a new provider instance | Admin |
| PATCH | `/api/bridge/v1/provider/{provider_instance_id}` | `bridge:update_provider_instance` | Update an existing provider instance | Admin |
| GET | `/api/bridge/v1/provider/{provider_instance_id}` | `bridge:read_provider_instance` | Get a specific provider instance | Admin, Maintainer, Agent |
| DELETE | `/api/bridge/v1/provider/{provider_instance_id}` | `bridge:delete_provider_instance` | Delete a provider instance | Admin |
| GET | `/api/bridge/v1/provider` | `bridge:list_provider_instances` | List all provider instances | Admin, Maintainer, Agent |
| GET | `/api/bridge/v1/provider/grouped-by-function` | `bridge:list_provider_instances` | List provider instances grouped by function | Admin, Maintainer, Agent |

### Credential Management

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| POST | `/api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/resource-server/encrypt` | `bridge:encrypt_credential_config` | Encrypt resource server configuration | Admin |
| POST | `/api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/user-credential/encrypt` | `bridge:encrypt_credential_config` | Encrypt user credential configuration | Admin |
| POST | `/api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/resource-server` | `bridge:create_credential` | Create a resource server credential | Admin |
| POST | `/api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/user-credential` | `bridge:create_credential` | Create a user credential | Admin |
| POST | `/api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/user-credential/broker` | `bridge:broker_credential` | Start user credential brokering flow | Admin |
| GET | `/api/bridge/v1/generic-oauth-callback` | `bridge:oauth_callback` | Handle OAuth callback | *Public* |

### Function Management

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| POST | `/api/bridge/v1/provider/{provider_instance_id}/function/{function_controller_type_id}/enable` | `bridge:enable_function` | Enable a function on a provider | Admin |
| POST | `/api/bridge/v1/provider/{provider_instance_id}/function/{function_controller_type_id}/disable` | `bridge:disable_function` | Disable a function on a provider | Admin |
| POST | `/api/bridge/v1/provider/{provider_instance_id}/function/{function_controller_type_id}/invoke` | `bridge:invoke_function` | Invoke a function | Admin, Agent |
| GET | `/api/bridge/v1/function-instances` | `bridge:list_function_instances` | List all function instances | Admin, Maintainer, Agent |
| GET | `/api/bridge/v1/function-instances/openapi.json` | `bridge:read_function_openapi` | Get OpenAPI spec for functions | Admin, Maintainer, Agent |

### MCP Protocol

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| GET | `/api/bridge/v1/mcp` | `bridge:mcp_connect` | Establish MCP SSE connection | Admin, Agent |
| POST | `/api/bridge/v1/mcp` | `bridge:mcp_message` | Send MCP message | Admin, Agent |

### MCP Server Instance Management

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| POST | `/api/bridge/v1/mcp-instance` | `bridge:create_mcp_instance` | Create MCP server instance | Admin |
| GET | `/api/bridge/v1/mcp-instance/{mcp_server_instance_id}` | `bridge:read_mcp_instance` | Get MCP server instance | Admin, Maintainer, Agent |
| PATCH | `/api/bridge/v1/mcp-instance/{mcp_server_instance_id}` | `bridge:update_mcp_instance` | Update MCP server instance | Admin |
| DELETE | `/api/bridge/v1/mcp-instance/{mcp_server_instance_id}` | `bridge:delete_mcp_instance` | Delete MCP server instance | Admin |
| POST | `/api/bridge/v1/mcp-instance/{mcp_server_instance_id}/function` | `bridge:add_mcp_instance_function` | Add function to MCP instance | Admin |
| PATCH | `/api/bridge/v1/mcp-instance/{mcp_server_instance_id}/function/{function_id}` | `bridge:update_mcp_instance_function` | Update MCP instance function | Admin |
| DELETE | `/api/bridge/v1/mcp-instance/{mcp_server_instance_id}/function/{function_id}` | `bridge:remove_mcp_instance_function` | Remove function from MCP instance | Admin |
| * | `/api/bridge/v1/mcp-instance/{mcp_server_instance_id}/mcp` | `bridge:mcp_instance_connect` | MCP protocol handler (SSE/streaming) | Admin, Agent |

---

## Task Routes (`/api/task/v1`)

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| GET | `/api/task/v1` | `task:list` | List all tasks | Admin, Maintainer, Agent |
| GET | `/api/task/v1/context` | `task:list_contexts` | List all contexts | Admin, Maintainer, Agent |
| GET | `/api/task/v1/context/{context_id}/task` | `task:list` | List tasks by context | Admin, Maintainer, Agent |
| GET | `/api/task/v1/{task_id}` | `task:read` | Get a specific task | Admin, Maintainer, Agent |
| PUT | `/api/task/v1/{task_id}` | `task:update_status` | Update task status | Admin, Agent |
| POST | `/api/task/v1/{task_id}/message` | `task:send_message` | Send message to task | Admin, Agent |
| GET | `/api/task/v1/{task_id}/timeline` | `task:read_timeline` | Get task timeline items | Admin, Maintainer, Agent |

---

## Agent Routes (`/api/agent`)

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| GET | `/api/agent` | `agent:list` | List available agents | Admin, Maintainer, Agent |
| GET | `/api/agent/{project_id}/{agent_id}/a2a/.well-known/agent.json` | `agent:read_card` | Get agent card | Admin, Maintainer, Agent |
| POST | `/api/agent/{project_id}/{agent_id}/a2a` | `agent:execute` | Handle A2A JSON-RPC | Admin, Agent |

---

## A2A Routes (`/api/a2a/v1`)

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| GET | `/api/a2a/v1/definition` | `a2a:read_definition` | Get agent definition | Admin, Maintainer, Agent |
| POST | `/api/a2a/v1/...` | `a2a:execute` | A2A protocol execution (via a2a_rs) | Admin, Agent |

---

## Secret Routes (`/api/secret/v1`)

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| POST | `/api/secret/v1` | `secret:create` | Create a new secret | Admin |
| POST | `/api/secret/v1/import` | `secret:import` | Import a secret | Admin |
| GET | `/api/secret/v1` | `secret:list` | List all secrets | Admin, Maintainer |
| GET | `/api/secret/v1/list-decrypted` | `secret:list_decrypted` | List secrets with decrypted values | Admin |
| GET | `/api/secret/v1/{secret_id}` | `secret:read` | Get a secret by ID | Admin, Maintainer |
| GET | `/api/secret/v1/key/{key}` | `secret:read` | Get a secret by key | Admin, Maintainer |
| PUT | `/api/secret/v1/{secret_id}` | `secret:update` | Update a secret | Admin |
| DELETE | `/api/secret/v1/{secret_id}` | `secret:delete` | Delete a secret | Admin |

---

## Environment Variable Routes (`/api/environment-variable/v1`)

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| POST | `/api/environment-variable/v1` | `env_var:create` | Create an environment variable | Admin |
| POST | `/api/environment-variable/v1/import` | `env_var:import` | Import an environment variable | Admin |
| GET | `/api/environment-variable/v1` | `env_var:list` | List environment variables | Admin, Maintainer, Agent |
| GET | `/api/environment-variable/v1/{env_var_id}` | `env_var:read` | Get an environment variable by ID | Admin, Maintainer, Agent |
| GET | `/api/environment-variable/v1/key/{key}` | `env_var:read` | Get an environment variable by key | Admin, Maintainer, Agent |
| PUT | `/api/environment-variable/v1/{env_var_id}` | `env_var:update` | Update an environment variable | Admin |
| DELETE | `/api/environment-variable/v1/{env_var_id}` | `env_var:delete` | Delete an environment variable | Admin |

---

## Encryption Routes (`/api/encryption/v1`)

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| POST | `/api/encryption/v1/envelope` | `encryption:create_envelope_key` | Create an envelope encryption key | Admin |
| GET | `/api/encryption/v1/envelope` | `encryption:list_envelope_keys` | List envelope encryption keys | Admin, Maintainer |
| POST | `/api/encryption/v1/envelope/{envelope_id}/dek` | `encryption:create_dek` | Create a data encryption key | Admin |
| POST | `/api/encryption/v1/envelope/{envelope_id}/dek/import` | `encryption:import_dek` | Import a data encryption key | Admin |
| GET | `/api/encryption/v1/envelope/{envelope_id}/dek` | `encryption:list_deks` | List data encryption keys | Admin, Maintainer |
| POST | `/api/encryption/v1/envelope/{envelope_id}/dek/{dek_id}/migrate` | `encryption:migrate_dek` | Migrate a data encryption key | Admin |
| POST | `/api/encryption/v1/envelope/{envelope_id}/migrate` | `encryption:migrate_all_deks` | Migrate all data encryption keys | Admin |
| POST | `/api/encryption/v1/dek/alias` | `encryption:create_dek_alias` | Create a DEK alias | Admin |
| GET | `/api/encryption/v1/dek/alias/{alias}` | `encryption:read_dek` | Get DEK by alias or ID | Admin, Maintainer |
| PUT | `/api/encryption/v1/dek/alias/{alias}` | `encryption:update_dek_alias` | Update a DEK alias | Admin |
| DELETE | `/api/encryption/v1/dek/alias/{alias}` | `encryption:delete_dek_alias` | Delete a DEK alias | Admin |

---

## Identity Routes (`/api/identity/v1`)

### API Key Management

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| POST | `/api/identity/v1/api-key` | `identity:create_api_key` | Create an API key | Admin |
| DELETE | `/api/identity/v1/api-key/{id}` | `identity:delete_api_key` | Delete an API key | Admin |
| GET | `/api/identity/v1/api-key` | `identity:list_api_keys` | List API keys | Admin, Maintainer |
| POST | `/api/identity/v1/api-key/import` | `identity:import_api_key` | Import an API key | Admin |

### Authentication

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| GET | `/api/identity/v1/auth/authorize/{config_id}` | `identity:start_authorization` | Start authorization flow | *Public* |
| GET | `/api/identity/v1/auth/callback` | `identity:auth_callback` | Handle auth callback | *Public* |
| POST | `/api/identity/v1/auth/refresh` | `identity:refresh_token` | Refresh access token | Admin, Maintainer, Agent, User |
| GET | `/api/identity/v1/auth/whoami` | `identity:whoami` | Get current identity | Admin, Maintainer, Agent, User |

### JWK Management

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| POST | `/api/identity/v1/jwk/{kid}/invalidate` | `identity:invalidate_jwk` | Invalidate a JWK | Admin |
| GET | `/api/identity/v1/jwk` | `identity:list_jwks` | List JWKs | Admin, Maintainer |
| GET | `/api/identity/v1/.well-known/jwks.json` | `identity:read_jwks` | Get JWKS | *Public* |

### STS Configuration

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| POST | `/api/identity/v1/sts-configuration` | `identity:create_sts_config` | Create STS configuration | Admin |
| GET | `/api/identity/v1/sts-configuration/{id}` | `identity:read_sts_config` | Get STS configuration | Admin, Maintainer |
| DELETE | `/api/identity/v1/sts-configuration/{id}` | `identity:delete_sts_config` | Delete STS configuration | Admin |
| GET | `/api/identity/v1/sts-configuration` | `identity:list_sts_configs` | List STS configurations | Admin, Maintainer |

### STS Token Exchange

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| POST | `/api/identity/v1/sts/{sts_config_id}` | `identity:exchange_sts_token` | Exchange STS token | Admin, Agent |

### User Auth Flow Configuration

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| POST | `/api/identity/v1/user-auth-flow-config` | `identity:create_user_auth_flow_config` | Create user auth flow configuration | Admin |
| GET | `/api/identity/v1/user-auth-flow-config/{id}` | `identity:read_user_auth_flow_config` | Get user auth flow configuration | Admin, Maintainer |
| DELETE | `/api/identity/v1/user-auth-flow-config/{id}` | `identity:delete_user_auth_flow_config` | Delete user auth flow configuration | Admin |
| GET | `/api/identity/v1/user-auth-flow-config` | `identity:list_user_auth_flow_configs` | List user auth flow configurations | Admin, Maintainer |
| POST | `/api/identity/v1/user-auth-flow-config/import` | `identity:import_user_auth_flow_config` | Import user auth flow configuration | Admin |

### User Management

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| POST | `/api/identity/v1/users` | `identity:create_user` | Create a user | Admin |
| GET | `/api/identity/v1/users/{user_id}` | `identity:read_user` | Get a user | Admin, Maintainer |
| PATCH | `/api/identity/v1/users/{user_id}` | `identity:update_user` | Update a user | Admin |
| DELETE | `/api/identity/v1/users/{user_id}` | `identity:delete_user` | Delete a user | Admin |
| GET | `/api/identity/v1/users` | `identity:list_users` | List users | Admin, Maintainer |
| GET | `/api/identity/v1/users/{user_id}/groups` | `identity:list_user_groups` | List groups for a user | Admin, Maintainer |

### Group Management

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| POST | `/api/identity/v1/groups` | `identity:create_group` | Create a group | Admin |
| GET | `/api/identity/v1/groups/{group_id}` | `identity:read_group` | Get a group | Admin, Maintainer |
| PATCH | `/api/identity/v1/groups/{group_id}` | `identity:update_group` | Update a group | Admin |
| DELETE | `/api/identity/v1/groups/{group_id}` | `identity:delete_group` | Delete a group | Admin |
| GET | `/api/identity/v1/groups` | `identity:list_groups` | List groups | Admin, Maintainer |

### Group Membership

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| POST | `/api/identity/v1/groups/{group_id}/members` | `identity:add_group_member` | Add member to group | Admin |
| DELETE | `/api/identity/v1/groups/{group_id}/members/{user_id}` | `identity:remove_group_member` | Remove member from group | Admin |
| GET | `/api/identity/v1/groups/{group_id}/members` | `identity:list_group_members` | List group members | Admin, Maintainer |

---

## SCIM 2.0 Routes (`/api/identity/scim/v2`)

SCIM (System for Cross-domain Identity Management) endpoints for enterprise identity provisioning.

### SCIM Users

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| GET | `/api/identity/scim/v2/Users` | `scim:list_users` | List SCIM users | Admin |
| POST | `/api/identity/scim/v2/Users` | `scim:create_user` | Create SCIM user | Admin |
| GET | `/api/identity/scim/v2/Users/{user_id}` | `scim:read_user` | Get SCIM user | Admin |
| PUT | `/api/identity/scim/v2/Users/{user_id}` | `scim:replace_user` | Replace SCIM user | Admin |
| PATCH | `/api/identity/scim/v2/Users/{user_id}` | `scim:patch_user` | Patch SCIM user | Admin |
| DELETE | `/api/identity/scim/v2/Users/{user_id}` | `scim:delete_user` | Delete SCIM user | Admin |

### SCIM Groups

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| GET | `/api/identity/scim/v2/Groups` | `scim:list_groups` | List SCIM groups | Admin |
| POST | `/api/identity/scim/v2/Groups` | `scim:create_group` | Create SCIM group | Admin |
| GET | `/api/identity/scim/v2/Groups/{group_id}` | `scim:read_group` | Get SCIM group | Admin |
| PUT | `/api/identity/scim/v2/Groups/{group_id}` | `scim:replace_group` | Replace SCIM group | Admin |
| PATCH | `/api/identity/scim/v2/Groups/{group_id}` | `scim:patch_group` | Patch SCIM group | Admin |
| DELETE | `/api/identity/scim/v2/Groups/{group_id}` | `scim:delete_group` | Delete SCIM group | Admin |

---

## Internal Routes (`/_internal/v1`)

| Method | Path | Permission | Description | Roles |
|--------|------|------------|-------------|-------|
| GET | `/_internal/v1/health` | `internal:health` | Health check | *Public* |
| GET | `/_internal/v1/runtime_config` | `internal:read_runtime_config` | Get runtime configuration | Admin, Maintainer |
| POST | `/_internal/v1/trigger_codegen` | `internal:trigger_codegen` | Trigger code generation | Admin |
| POST | `/_internal/v1/resync_sdk` | `internal:resync_sdk` | Resync SDK | Admin |

---

## Permission Summary

### All Permissions by Domain

| Domain | Permissions |
|--------|-------------|
| `bridge` | `list_available_providers`, `create_provider_instance`, `update_provider_instance`, `read_provider_instance`, `delete_provider_instance`, `list_provider_instances`, `encrypt_credential_config`, `create_credential`, `broker_credential`, `oauth_callback`, `enable_function`, `disable_function`, `invoke_function`, `list_function_instances`, `read_function_openapi`, `mcp_connect`, `mcp_message`, `create_mcp_instance`, `read_mcp_instance`, `update_mcp_instance`, `delete_mcp_instance`, `add_mcp_instance_function`, `update_mcp_instance_function`, `remove_mcp_instance_function`, `mcp_instance_connect` |
| `task` | `list`, `list_contexts`, `read`, `update_status`, `send_message`, `read_timeline` |
| `agent` | `list`, `read_card`, `execute` |
| `a2a` | `read_definition`, `execute` |
| `secret` | `create`, `import`, `list`, `list_decrypted`, `read`, `update`, `delete` |
| `env_var` | `create`, `import`, `list`, `read`, `update`, `delete` |
| `encryption` | `create_envelope_key`, `list_envelope_keys`, `create_dek`, `import_dek`, `list_deks`, `migrate_dek`, `migrate_all_deks`, `create_dek_alias`, `read_dek`, `update_dek_alias`, `delete_dek_alias` |
| `identity` | `create_api_key`, `delete_api_key`, `list_api_keys`, `import_api_key`, `start_authorization`, `auth_callback`, `refresh_token`, `whoami`, `invalidate_jwk`, `list_jwks`, `read_jwks`, `create_sts_config`, `read_sts_config`, `delete_sts_config`, `list_sts_configs`, `exchange_sts_token`, `create_user_auth_flow_config`, `read_user_auth_flow_config`, `delete_user_auth_flow_config`, `list_user_auth_flow_configs`, `import_user_auth_flow_config`, `create_user`, `read_user`, `update_user`, `delete_user`, `list_users`, `list_user_groups`, `create_group`, `read_group`, `update_group`, `delete_group`, `list_groups`, `add_group_member`, `remove_group_member`, `list_group_members` |
| `scim` | `list_users`, `create_user`, `read_user`, `replace_user`, `patch_user`, `delete_user`, `list_groups`, `create_group`, `read_group`, `replace_group`, `patch_group`, `delete_group` |
| `internal` | `health`, `read_runtime_config`, `trigger_codegen`, `resync_sdk` |

### Role-Permission Matrix

| Permission | Admin | Maintainer | Agent | User |
|------------|:-----:|:----------:|:-----:|:----:|
| **Bridge - Provider** |
| `bridge:list_available_providers` | ✓ | ✓ | | |
| `bridge:create_provider_instance` | ✓ | | | |
| `bridge:update_provider_instance` | ✓ | | | |
| `bridge:read_provider_instance` | ✓ | ✓ | ✓ | |
| `bridge:delete_provider_instance` | ✓ | | | |
| `bridge:list_provider_instances` | ✓ | ✓ | ✓ | |
| **Bridge - Credential** |
| `bridge:encrypt_credential_config` | ✓ | | | |
| `bridge:create_credential` | ✓ | | | |
| `bridge:broker_credential` | ✓ | | | |
| **Bridge - Function** |
| `bridge:enable_function` | ✓ | | | |
| `bridge:disable_function` | ✓ | | | |
| `bridge:invoke_function` | ✓ | | ✓ | |
| `bridge:list_function_instances` | ✓ | ✓ | ✓ | |
| `bridge:read_function_openapi` | ✓ | ✓ | ✓ | |
| **Bridge - MCP** |
| `bridge:mcp_connect` | ✓ | | ✓ | |
| `bridge:mcp_message` | ✓ | | ✓ | |
| `bridge:create_mcp_instance` | ✓ | | | |
| `bridge:read_mcp_instance` | ✓ | ✓ | ✓ | |
| `bridge:update_mcp_instance` | ✓ | | | |
| `bridge:delete_mcp_instance` | ✓ | | | |
| `bridge:add_mcp_instance_function` | ✓ | | | |
| `bridge:update_mcp_instance_function` | ✓ | | | |
| `bridge:remove_mcp_instance_function` | ✓ | | | |
| `bridge:mcp_instance_connect` | ✓ | | ✓ | |
| **Task** |
| `task:list` | ✓ | ✓ | ✓ | |
| `task:list_contexts` | ✓ | ✓ | ✓ | |
| `task:read` | ✓ | ✓ | ✓ | |
| `task:update_status` | ✓ | | ✓ | |
| `task:send_message` | ✓ | | ✓ | |
| `task:read_timeline` | ✓ | ✓ | ✓ | |
| **Agent** |
| `agent:list` | ✓ | ✓ | ✓ | |
| `agent:read_card` | ✓ | ✓ | ✓ | |
| `agent:execute` | ✓ | | ✓ | |
| **A2A** |
| `a2a:read_definition` | ✓ | ✓ | ✓ | |
| `a2a:execute` | ✓ | | ✓ | |
| **Secret** |
| `secret:create` | ✓ | | | |
| `secret:import` | ✓ | | | |
| `secret:list` | ✓ | ✓ | | |
| `secret:list_decrypted` | ✓ | | | |
| `secret:read` | ✓ | ✓ | | |
| `secret:update` | ✓ | | | |
| `secret:delete` | ✓ | | | |
| **Environment Variable** |
| `env_var:create` | ✓ | | | |
| `env_var:import` | ✓ | | | |
| `env_var:list` | ✓ | ✓ | ✓ | |
| `env_var:read` | ✓ | ✓ | ✓ | |
| `env_var:update` | ✓ | | | |
| `env_var:delete` | ✓ | | | |
| **Encryption** |
| `encryption:create_envelope_key` | ✓ | | | |
| `encryption:list_envelope_keys` | ✓ | ✓ | | |
| `encryption:create_dek` | ✓ | | | |
| `encryption:import_dek` | ✓ | | | |
| `encryption:list_deks` | ✓ | ✓ | | |
| `encryption:migrate_dek` | ✓ | | | |
| `encryption:migrate_all_deks` | ✓ | | | |
| `encryption:create_dek_alias` | ✓ | | | |
| `encryption:read_dek` | ✓ | ✓ | | |
| `encryption:update_dek_alias` | ✓ | | | |
| `encryption:delete_dek_alias` | ✓ | | | |
| **Identity - API Key** |
| `identity:create_api_key` | ✓ | | | |
| `identity:delete_api_key` | ✓ | | | |
| `identity:list_api_keys` | ✓ | ✓ | | |
| `identity:import_api_key` | ✓ | | | |
| **Identity - Auth** |
| `identity:refresh_token` | ✓ | ✓ | ✓ | ✓ |
| `identity:whoami` | ✓ | ✓ | ✓ | ✓ |
| **Identity - JWK** |
| `identity:invalidate_jwk` | ✓ | | | |
| `identity:list_jwks` | ✓ | ✓ | | |
| **Identity - STS** |
| `identity:create_sts_config` | ✓ | | | |
| `identity:read_sts_config` | ✓ | ✓ | | |
| `identity:delete_sts_config` | ✓ | | | |
| `identity:list_sts_configs` | ✓ | ✓ | | |
| `identity:exchange_sts_token` | ✓ | | ✓ | |
| **Identity - User Auth Flow** |
| `identity:create_user_auth_flow_config` | ✓ | | | |
| `identity:read_user_auth_flow_config` | ✓ | ✓ | | |
| `identity:delete_user_auth_flow_config` | ✓ | | | |
| `identity:list_user_auth_flow_configs` | ✓ | ✓ | | |
| `identity:import_user_auth_flow_config` | ✓ | | | |
| **Identity - User** |
| `identity:create_user` | ✓ | | | |
| `identity:read_user` | ✓ | ✓ | | |
| `identity:update_user` | ✓ | | | |
| `identity:delete_user` | ✓ | | | |
| `identity:list_users` | ✓ | ✓ | | |
| `identity:list_user_groups` | ✓ | ✓ | | |
| **Identity - Group** |
| `identity:create_group` | ✓ | | | |
| `identity:read_group` | ✓ | ✓ | | |
| `identity:update_group` | ✓ | | | |
| `identity:delete_group` | ✓ | | | |
| `identity:list_groups` | ✓ | ✓ | | |
| **Identity - Group Membership** |
| `identity:add_group_member` | ✓ | | | |
| `identity:remove_group_member` | ✓ | | | |
| `identity:list_group_members` | ✓ | ✓ | | |
| **SCIM - User** |
| `scim:list_users` | ✓ | | | |
| `scim:create_user` | ✓ | | | |
| `scim:read_user` | ✓ | | | |
| `scim:replace_user` | ✓ | | | |
| `scim:patch_user` | ✓ | | | |
| `scim:delete_user` | ✓ | | | |
| **SCIM - Group** |
| `scim:list_groups` | ✓ | | | |
| `scim:create_group` | ✓ | | | |
| `scim:read_group` | ✓ | | | |
| `scim:replace_group` | ✓ | | | |
| `scim:patch_group` | ✓ | | | |
| `scim:delete_group` | ✓ | | | |
| **Internal** |
| `internal:read_runtime_config` | ✓ | ✓ | | |
| `internal:trigger_codegen` | ✓ | | | |
| `internal:resync_sdk` | ✓ | | | |

### Public Endpoints (No Auth Required)

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
- **Total Unique Permissions**: 98
- **Domains**: 10
- **Roles**: 4
- **Public Endpoints**: 5

| HTTP Method | Count |
|-------------|-------|
| GET | 45 |
| POST | 40 |
| PUT | 6 |
| PATCH | 11 |
| DELETE | 14 |
