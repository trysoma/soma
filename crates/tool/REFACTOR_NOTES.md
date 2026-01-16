# Tool Group Source Refactoring Notes

## Overview
The hardcoded in-memory tool group sources (GoogleMailToolGroupSource, StripeToolGroupSource) have been removed. Tool groups and their sources should now be registered via API and stored in the repository/database.

## Removed Components

### 1. Deleted Files/Directories
- `src/providers/` directory (containing google_mail and stripe implementations)

### 2. Removed Functions (from `src/logic/source.rs`)
- `get_tool_group_source(tool_group_source_type_id: &str)` - Used to fetch hardcoded sources
- `list_all_tool_group_sources()` - Returned list of hardcoded sources
- `get_credential_source(tool_group, credential_source_type_id)` - Got credential source from provider
- `get_tool_source(tool_group, tool_source_type_id)` - Got tool source from provider

### 3. Removed Trait Implementations
- `From<Arc<dyn CredentialSourceLike>> for ToolGroupCredentialSourceSerialized`
- `From<Arc<dyn ToolSourceLike>> for ToolSourceSerialized`
- `From<&dyn ToolGroupLike> for ToolGroupSourceSerialized`

### 4. Removed Constants
- `CATEGORY_EMAIL`
- `CATEGORY_PAYMENTS`

## Kept Components

### Serialization Structs (for API responses)
- `ToolGroupSourceSerialized` - Still used for API responses
- `ToolSourceSerialized` - Still used for API responses
- `ToolGroupCredentialSourceSerialized` - Still used for API responses
- `WithToolGroupSourceTypeId<T>` - Wrapper struct for params
- `WithToolSourceTypeId<T>` - Wrapper struct for params
- `WithCredentialSourceTypeId<T>` - Wrapper struct for params

## Locations Requiring Refactoring

### HIGH PRIORITY - Core Functionality Broken

#### 1. `src/logic/credential_encryption.rs`
**Functions affected:**
- `encrypt_resource_server_configuration()` - Lines 36-50
- `encrypt_user_credential_configuration()` - Lines 62-75

**Status:** ❌ Currently returns error
**Required changes:**
- Fetch tool group source metadata from repository using `tool_group_source_type_id`
- Reconstruct or fetch credential source implementation
- Use credential source to encrypt configurations

#### 2. `src/logic/credential/mod.rs`
**Functions affected:**
- `create_resource_server_credential()` - Line ~249
- `create_user_credential()` - Line ~364
- `start_user_credential_brokering()` - Line ~565
- `resume_user_credential_brokering()` - Line ~646
- `process_credential_rotations_with_window()` - Lines ~899, ~967

**Current behavior:** Will fail at runtime when trying to call removed functions
**Required changes:**
- Replace `get_tool_group_source()` calls with repository lookups
- Replace `get_credential_source()` calls with fetching from stored definitions
- May need to store credential source implementations in database or reconstruct from metadata

#### 3. `src/logic/instance.rs`
**Functions affected:**
- `list_tool_group_instances_internal()` - Line ~275, ~280
  - Used to enrich responses with controller metadata
- `get_tool_instances_internal()` - Lines ~387, ~406
  - Used to get controller and tool source for each instance
- `enrich_tool_instance()` - Line ~633
  - Used to get tool source metadata
- `create_tool_group_instance()` - Lines ~684, ~686
  - Validates tool_group_source_type_id and credential_source_type_id exist
- `get_tool_group_instance()` - Lines ~1042, ~1048
  - Gets controller metadata to enrich response
- `enable_function()` - Lines ~1107, ~1112
  - Validates function exists in tool group
- `invoke_function()` - Lines ~1207, ~1212, ~1221
  - Gets function and credential sources for invocation

**Required changes:**
- Replace source lookups with repository-based metadata fetches
- Store tool group, credential, and tool source metadata in database
- Update enrichment logic to use stored metadata instead of trait implementations

### MEDIUM PRIORITY - API Endpoints Affected

#### 4. `src/router/tool_group.rs`
**Functions affected:**
- `route_list_available_tool_groups()` - Line ~78

**Status:** ✅ Updated to return empty list with TODO
**Required changes:**
- Implement repository method to list registered tool groups
- Call repository method instead of returning empty list

### Repository Layer Changes Needed

**New repository methods required:**
1. `list_tool_group_sources()` - Fetch all registered tool group definitions
2. `get_tool_group_source_by_type_id()` - Get specific tool group definition
3. `register_tool_group_source()` - Register new tool group via API
4. `list_credential_sources_for_tool_group()` - Get credential sources for a tool group
5. `list_tool_sources_for_tool_group()` - Get tool sources for a tool group

**Database schema additions needed:**
- `tool_group_sources` table - Store tool group metadata
- `credential_sources` table - Store credential source metadata
- `tool_sources` table - Store tool source metadata (or use existing `tools` table)

## Migration Path

### Phase 1: Define Schema & Repository Methods
1. Create database tables for tool group, credential, and tool source metadata
2. Implement repository methods for CRUD operations
3. Create API endpoints for registering tool groups and sources

### Phase 2: Update Logic Layer
1. Refactor `credential_encryption.rs` to use repository lookups
2. Refactor `credential/mod.rs` to fetch sources from repository
3. Update `instance.rs` to fetch controller metadata from repository for validation and enrichment

### Phase 3: Data Migration
1. Create migration script to register existing hardcoded sources (google_mail, stripe) via API
2. Run migration to populate database with initial controllers
3. Test all affected endpoints

### Phase 4: Cleanup
1. Remove any remaining references to hardcoded sources
2. Update tests to use API-registered sources
3. Update documentation

## Notes

- The trait implementations (`ToolGroupLike`, `CredentialSourceLike`, `ToolSourceLike`) may still be useful for custom source implementations
- Consider whether sources should be fully dynamic (stored as HTTP endpoints) or if some trait-based implementation is still needed for complex logic
- The existing tool registration system in `src/router/tool.rs` may provide a good pattern to follow for tool group registration
