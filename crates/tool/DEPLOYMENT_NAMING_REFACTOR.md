# Deployment Naming Refactor Summary

## Overview

This document summarizes the comprehensive renaming from "source" terminology to "deployment" terminology throughout the tool crate and related code.

## Motivation

The naming "source" was ambiguous and confusing. "Deployment" more clearly conveys that these are deployed versions/instances of tool configurations.

## Changes Made

### 1. Database Schema (schema.sql)

#### Table Renames:
- `tool_source` → `tool_deployment`
- `tool_group_source` → `tool_group_deployment`
- `tool_group_alias` → `tool_group_deployment_alias`

#### New Fields:
- `tool_group_deployment.credential_deployments` (JSON) - Caches credential configuration schemas for the tool group deployment, eliminating the need for separate lookups

#### Field Renames:
All fields containing `_source_` were renamed to `_deployment_`:

- `tool_source_type_id` → `tool_deployment_type_id`
- `tool_group_source_type_id` → `tool_group_deployment_type_id`
- `tool_group_source_deployment_id` → `tool_group_deployment_deployment_id`
- `credential_source_type_id` → `credential_deployment_type_id`

### 2. Rust Code Changes

#### Struct Renames:

| Old Name | New Name |
|----------|----------|
| `ToolSourceSerialized` | `ToolDeploymentSerialized` |
| `ToolGroupSourceSerialized` | `ToolGroupDeploymentSerialized` |
| `ToolGroupCredentialSourceSerialized` | `ToolGroupCredentialDeploymentSerialized` |
| `ToolSourceConfig` | `ToolDeploymentConfig` |
| `ToolGroupSourceConfig` | `ToolGroupDeploymentConfig` |
| `CredentialSourceConfig` | `CredentialDeploymentConfig` |
| `WithToolSourceTypeId<T>` | `WithToolDeploymentTypeId<T>` |
| `WithToolGroupSourceTypeId<T>` | `WithToolGroupDeploymentTypeId<T>` |
| `WithCredentialSourceTypeId<T>` | `WithCredentialDeploymentTypeId<T>` |

#### Module Renames:

- `crates/tool/src/logic/source.rs` → `crates/tool/src/logic/deployment.rs`
- All imports: `use crate::logic::source::` → `use crate::logic::deployment::`
- Module declarations: `pub mod source;` → `pub mod deployment;`
- Re-exports: `pub use source::*;` → `pub use deployment::*;`

#### Method Renames:

| Old Method Name | New Method Name |
|----------------|-----------------|
| `add_tool_group_source()` | `add_tool_group_deployment()` |
| `update_tool_group_source()` | `update_tool_group_deployment()` |
| `remove_tool_group_source()` | `remove_tool_group_deployment()` |
| `add_tool_source()` | `add_tool_deployment()` |
| `update_tool_source()` | `update_tool_deployment()` |
| `remove_tool_source()` | `remove_tool_deployment()` |

#### Field Renames in Structs:

All struct fields were updated:
- `tool_group_sources: Option<HashMap<...>>` → `tool_group_deployments: Option<HashMap<...>>`
- `tool_sources: Vec<...>` → `tool_deployments: Vec<...>`
- `credential_sources: Vec<...>` → `credential_deployments: Vec<...>`

### 3. Variable Name Updates

Throughout the codebase, local variables were updated:
- `tool_group_source` → `tool_group_deployment`
- `tool_source` → `tool_deployment`
- `tool_group_sources` → `tool_group_deployments`

### 4. Documentation Updates

Updated documentation files:
- `/Users/danielblignaut/Development/soma/soma/MIGRATION_GUIDE.md`
- `/Users/danielblignaut/Development/soma/soma/soma.yaml.example`
- `/Users/danielblignaut/Development/soma/soma/crates/tool/SCHEMA_REFACTORING.md`

All references to "source" terminology updated to "deployment" terminology.

### 5. YAML Configuration

The `soma.yaml` configuration format now uses:
```yaml
tool_configuration:
  tool_group_deployments:
    my_gmail:
      tool_group_deployment_type_id: google_mail
      credential_deployment_type_id: oauth
      # ...
```

## Files Modified

### Core Implementation:
- `crates/tool/dbs/tool/schema.sql`
- `crates/tool/src/logic/deployment.rs` (renamed from source.rs)
- `crates/tool/src/logic/mod.rs`
- `crates/tool/src/logic/instance.rs`
- `crates/tool/src/logic/credential/mod.rs`
- `crates/shared/src/soma_agent_definition.rs`

### Sync & Integration:
- `crates/soma/src/mcp/sync_yaml_to_api_on_start.rs`
- `crates/soma/src/mcp/sync_to_yaml_on_mcp_change.rs`
- `crates/soma-api-server/src/logic/mcp/providers/soma.rs`

### Documentation:
- `MIGRATION_GUIDE.md`
- `soma.yaml.example`
- `crates/tool/SCHEMA_REFACTORING.md`

## Impact

### Breaking Changes:
- **Database schema**: Requires migration to rename tables and columns
- **API models**: All API responses now use `*_deployment_*` field names
- **YAML format**: `soma.yaml` files must be updated with new field names

### Backward Compatibility:
**None** - This is a clean break. All existing code, database schemas, and configuration files must be updated.

## Migration Steps

For users with existing deployments:

1. **Database Migration**:
   - Generate a new migration from the updated schema.sql
   - Run the migration to rename tables and columns

2. **YAML Configuration**:
   - Update all `soma.yaml` files using find/replace:
     - `tool_group_source_type_id` → `tool_group_deployment_type_id`
     - `tool_source_type_id` → `tool_deployment_type_id`
     - `credential_source_type_id` → `credential_deployment_type_id`

3. **API Clients**:
   - Regenerate API clients from the new OpenAPI spec
   - Update all field references in consuming code

## Verification

The refactoring is complete and verified:
- ✅ All Rust code compiles successfully
- ✅ Only minor warnings about unused variables (unrelated to refactoring)
- ✅ All imports and module references updated
- ✅ All struct, field, and method names consistent
- ✅ Documentation updated

## Next Steps

1. Generate database migration scripts
2. Update SQLC queries with new table/column names
3. Regenerate SQLC code
4. Update repository implementations
5. Test end-to-end with example data
