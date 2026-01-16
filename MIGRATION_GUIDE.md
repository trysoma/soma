# soma.yaml Migration Guide

This guide explains the changes to the `soma.yaml` configuration format and how to migrate existing configurations.

## Summary of Changes

The refactoring renames key concepts to better reflect their purpose:

| **Old Term** | **New Term** | **Meaning** |
|--------------|--------------|-------------|
| `mcp:` | `tool_configuration:` | Top-level section for tool/MCP configuration |
| `providers:` | `tool_groups:` | Instances of configured integrations |
| `provider_controller_type_id` | `tool_group_deployment_type_id` | Type of integration (e.g., "google_mail") |
| `credential_controller_type_id` | `credential_deployment_type_id` | Type of credential (e.g., "oauth", "api_key") |
| `functions:` | `tools:` | List of enabled capabilities |
| `function_controller_type_id` | `tool_deployment_type_id` | Type of tool (e.g., "send_email") |
| `provider_instance_id` | `tool_group_id` | Reference to a tool group instance |

**Important**: The `id` field has been removed from all credential configurations in the YAML file. Credential IDs are now database-generated and not included in the YAML.

## Before and After Examples

### Old Format (Before Refactoring)

```yaml
version: 1.0

encryption:
  envelope_keys:
    local:
      type: local
      deks:
        default:
          encrypted_key: "..."

mcp:  # OLD: was "mcp"
  providers:  # OLD: was "providers"
    my_gmail:
      provider_controller_type_id: google_mail  # OLD
      credential_controller_type_id: oauth      # OLD
      display_name: "My Gmail"

      resource_server_credential:
        id: "abc123"  # OLD: ID field is removed in new format
        type_id: oauth
        dek_alias: default
        metadata: {...}
        value: {...}

      user_credential:
        id: "def456"  # OLD: ID field is removed in new format
        type_id: oauth
        dek_alias: default
        metadata: {...}
        value: {...}

      functions:  # OLD: was "functions"
        - send_email
        - read_email

  mcp_servers:
    my_server:
      name: "My Server"
      functions:
        - function_controller_type_id: send_email    # OLD
          provider_controller_type_id: google_mail   # OLD
          provider_instance_id: my_gmail             # OLD
          function_name: sendEmail
          function_description: "Send email"
```

### New Format (After Refactoring)

```yaml
version: 1.0

encryption:
  envelope_keys:
    local:
      type: local
      deks:
        default:
          encrypted_key: "..."

tool_configuration:  # NEW: renamed from "mcp"
  tool_groups:  # NEW: renamed from "providers"
    my_gmail:
      tool_group_deployment_type_id: google_mail  # NEW: renamed
      credential_deployment_type_id: oauth        # NEW: renamed
      display_name: "My Gmail"

      resource_server_credential:
        # NEW: "id" field removed - IDs are database-generated
        type_id: oauth
        dek_alias: default
        metadata: {...}
        value: {...}

      user_credential:
        # NEW: "id" field removed - IDs are database-generated
        type_id: oauth
        dek_alias: default
        metadata: {...}
        value: {...}

      tools:  # NEW: renamed from "functions"
        - send_email
        - read_email

  mcp_servers:
    my_server:
      name: "My Server"
      functions:
        - tool_deployment_type_id: send_email          # NEW: renamed
          tool_group_deployment_type_id: google_mail   # NEW: renamed
          tool_group_id: my_gmail                  # NEW: renamed
          function_name: sendEmail
          function_description: "Send email"
```

## Step-by-Step Migration

### 1. Rename Top-Level Section

```diff
- mcp:
+ tool_configuration:
```

### 2. Rename Providers to Tool Groups

```diff
- mcp:
-   providers:
+ tool_configuration:
+   tool_groups:
```

### 3. Update Field Names in Tool Groups

For each tool group, update these fields:

```diff
  tool_groups:
    my_integration:
-     provider_controller_type_id: google_mail
+     tool_group_deployment_type_id: google_mail

-     credential_controller_type_id: oauth
+     credential_deployment_type_id: oauth
```

### 4. Remove ID Fields from Credentials

Remove the `id` field from both `resource_server_credential` and `user_credential`:

```diff
  resource_server_credential:
-   id: "some-uuid-here"
    type_id: oauth
    dek_alias: default
```

### 5. Rename Functions to Tools

```diff
  tool_groups:
    my_integration:
-     functions:
+     tools:
        - send_email
        - read_email
```

### 6. Update MCP Server Function Mappings

For each function in your MCP servers:

```diff
  mcp_servers:
    my_server:
      functions:
-       - function_controller_type_id: send_email
+       - tool_deployment_type_id: send_email

-         provider_controller_type_id: google_mail
+         tool_group_deployment_type_id: google_mail

-         provider_instance_id: my_gmail
+         tool_group_id: my_gmail
```

## Automated Migration Script

You can use this sed script to perform most of the migration automatically:

```bash
#!/bin/bash

# Backup original file
cp soma.yaml soma.yaml.backup

# Perform replacements
sed -i '' \
  -e 's/^mcp:/tool_configuration:/g' \
  -e 's/  providers:/  tool_groups:/g' \
  -e 's/provider_controller_type_id/tool_group_deployment_type_id/g' \
  -e 's/credential_controller_type_id/credential_deployment_type_id/g' \
  -e 's/function_controller_type_id/tool_deployment_type_id/g' \
  -e 's/provider_instance_id/tool_group_id/g' \
  -e 's/      functions:/      tools:/g' \
  soma.yaml

# Remove id fields from credentials (manual verification recommended)
echo "WARNING: You must manually remove 'id:' fields from credentials!"
echo "Look for 'resource_server_credential:' and 'user_credential:' sections"
```

**Important**: After running the automated script, you MUST manually remove all `id:` fields from credential configurations.

## Validation

After migration, verify your configuration:

1. Check that all top-level sections are renamed correctly
2. Verify all field names are updated
3. Confirm all `id` fields are removed from credentials
4. Test that the configuration loads successfully with `soma dev`

## Backward Compatibility

**This is a breaking change** - there is no backward compatibility with the old format. All existing `soma.yaml` files must be migrated to the new format.

## Getting Help

If you encounter issues during migration:
- Check the example file at `soma.yaml.example` for reference
- Review this migration guide carefully
- Ensure all field names match the new naming convention exactly
- Verify JSON structure and indentation are correct
