# Tool Crate Database Schema Refactoring

## Summary

This document describes the comprehensive database schema refactoring for the tool crate to properly model the relationship between tool group sources (type definitions), tool sources (tool definitions), tool group instances, and tool instances.

## Key Changes

### 1. Table Renaming

| Old Name | New Name | Purpose |
|----------|----------|---------|
| `tool` | `tool_group_source` | Definitions/types of tool groups (e.g., "google_mail", "stripe") |
| `tool_alias` | `tool_group_alias` | Aliases for tool group sources |
| `tool_group_instance` | `tool_group` | User-configured instances of tool groups with credentials |
| `tool_instance` | `tool` | Enabled tools within a tool group |

### 2. New Tables

#### `tool_source`
New table defining tool types/definitions within a tool group source.

**Structure:**
- `type_id` (PK) - Unique identifier for the tool type
- `tool_group_source_type_id` (PK, FK) - References tool_group_source
- `tool_group_source_deployment_id` (PK, FK) - References tool_group_source
- `name` - Human-readable name
- `documentation` - Description
- `categories` - JSON array of categories
- `metadata` - Additional metadata
- Timestamps

**Composite Primary Key:** `(type_id, tool_group_source_type_id, tool_group_source_deployment_id)`

## Schema Structure

```
tool_group_source (type definitions)
  ├── type_id, deployment_id (PK)
  ├── endpoint_type, endpoint_configuration
  └── name, documentation, categories
      │
      ├─> tool_source (tool definitions)
      │     ├── type_id, tool_group_source_type_id, tool_group_source_deployment_id (PK)
      │     └── name, documentation, categories
      │
      └─> tool_group_alias (friendly names)
            └── alias (unique across all tool groups)

tool_group (user instances)
  ├── id (PK)
  ├── alias (optional, for friendly reference)
  ├── tool_group_source_type_id, tool_group_source_deployment_id (FK to tool_group_source)
  ├── credential_source_type_id
  ├── resource_server_credential_id, user_credential_id (FKs)
  └── status, display_name
      │
      └─> tool (enabled capabilities)
            ├── tool_source_type_id, tool_group_source_type_id, tool_group_source_deployment_id (FK to tool_source)
            ├── tool_group_id (FK to tool_group)
            └── (PK is composite of all four fields)
```

## Field Naming Updates

All field names have been updated to remove "controller" and "instance" wording:

### Removed "controller" wording:
- `provider_controller_type_id` → `tool_group_source_type_id`
- `credential_controller_type_id` → `credential_source_type_id`
- `function_controller_type_id` → `tool_source_type_id`

### Removed "instance" wording:
- `tool_group_instance` table → `tool_group` table
- `tool_group_instance_id` field → `tool_group_id` field
- `tool_instance` table → `tool` table

### Added deployment_id tracking:
- `tool_group_source_deployment_id` added to track which deployment of a tool group source is being used

## Updated Table Details

### `tool_group_deployment`
**Purpose:** Defines types of tool group integrations (e.g., "google_mail", "stripe")

**Key Fields:**
- `type_id`, `deployment_id` (composite PK)
- `endpoint_type`, `endpoint_configuration` - How to communicate with this integration
- `credential_deployments` (JSON) - Cached array of credential configuration schemas, each containing:
  - `type_id` - Credential type (e.g., "oauth", "api_key")
  - `configuration_schema` - JSON schema for credential configuration
  - `name`, `documentation` - Descriptive information
  - `requires_brokering` - Whether user credential brokering is required
  - `requires_resource_server_credential_refreshing` - Whether resource server credentials need refresh
  - `requires_user_credential_refreshing` - Whether user credentials need refresh
- `name`, `documentation`, `categories` - Descriptive information
- `metadata` - Additional configuration

### `tool_source`
**Purpose:** Defines tool types within a tool group source (e.g., "send_email" for "google_mail")

**Key Fields:**
- `type_id` - Unique identifier for this tool type
- `tool_group_source_type_id`, `tool_group_source_deployment_id` - Parent tool group source
- `name`, `documentation`, `categories` - Descriptive information
- `metadata` - Additional configuration

**Important:** Tool sources do NOT have `endpoint_type` and `endpoint_configuration` - they inherit these from their parent `tool_group_source`.

### `tool_group_alias`
**Purpose:** Provides friendly aliases for tool group sources

**Key Fields:**
- `tool_group_source_type_id`, `tool_group_source_deployment_id` - References tool_group_source
- `alias` - Friendly name (unique across all tool groups)

**Example:** Alias "gmail" might point to tool_group_source ("google_mail", "v1")

### `tool_group`
**Purpose:** User-configured instances of tool groups with credentials

**Key Fields:**
- `id` (PK) - Unique instance ID (e.g., "my_gmail_account")
- `alias` (optional) - Optional friendly reference
- `tool_group_source_type_id`, `tool_group_source_deployment_id` - Which tool group source this is an instance of
- `credential_source_type_id` - Type of credential (e.g., "oauth", "api_key")
- `resource_server_credential_id`, `user_credential_id` - Credential references
- `status` - Instance status ("brokering_initiated", "active", "disabled")
- `display_name` - Human-readable name

### `tool`
**Purpose:** Enabled tools within a tool group

**Key Fields:**
- `tool_source_type_id` - Which tool is enabled
- `tool_group_source_type_id`, `tool_group_source_deployment_id` - Parent tool group source
- `tool_group_id` - Which tool group instance this belongs to

**Composite PK:** All four fields

### `mcp_server_instance_tool`
**Purpose:** Maps tools to MCP protocol functions

**Key Fields:**
- `mcp_server_instance_id` - Which MCP server
- `tool_source_type_id`, `tool_group_source_type_id`, `tool_group_source_deployment_id`, `tool_group_id` - References tool
- `tool_name` - Name exposed via MCP (unique per MCP server)
- `tool_description` - Description

### `broker_state`
**Purpose:** Tracks credential brokering state

**Updated Fields:**
- `tool_group_id` (was `tool_group_instance_id`)
- `tool_group_source_type_id` (was `tool_group_source_type_id`)
- Added `tool_group_source_deployment_id`
- `credential_source_type_id` (was `credential_controller_type_id`)

## Migration Requirements

This is a **breaking schema change**. To migrate:

1. **Generate new migration** from this schema
2. **Update all SQLC queries** to use new table names and fields
3. **Update repository layer** to work with new structure
4. **Update logic layer** to work with new relationships
5. **Regenerate SQLC code**
6. **Update all references** in the codebase

## Benefits

1. **Clearer separation** between type definitions (sources) and instances
2. **Proper foreign key relationships** ensuring data integrity
3. **Deployment tracking** allows versioning of tool group sources
4. **Consistent naming** without "controller" or "instance" confusion
5. **Tool source table** properly models the relationship between tools and tool group sources
6. **Alias system** now correctly references sources, not instances

## Next Steps

1. Generate database migration
2. Update SQLC queries and regenerate code
3. Update repository trait and implementations
4. Update logic layer
5. Update API routes
6. Test thoroughly with example data
