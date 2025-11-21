use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use shared::error::CommonError;
use tera::{Context, Tera};
use tracing::info;

use bridge::logic::{FunctionInstanceWithMetadata, get_function_instances};
use bridge::repository::ProviderRepositoryLike;

// Re-export Runtime for convenience
pub use crate::commands::dev::runtime::{Runtime, determine_runtime_from_dir};

/// TypeScript template loaded at compile time
const TYPESCRIPT_TEMPLATE: &str = include_str!("typescript.ts");

/// Saves TypeScript code to $project_dir/.soma/bridge.ts
pub async fn save_typescript_code(code: String, project_dir: &Path) -> Result<(), CommonError> {
    let soma_dir = project_dir.join(".soma");
    let output_path = soma_dir.join("bridge.ts");

    // Ensure .soma directory exists
    std::fs::create_dir_all(&soma_dir).map_err(|e| {
        CommonError::Unknown(anyhow::anyhow!("Failed to create .soma directory: {e}"))
    })?;

    std::fs::write(&output_path, code).map_err(|e| {
        CommonError::Unknown(anyhow::anyhow!("Failed to write bridge client file: {e}"))
    })?;

    info!("Bridge client written to: {}", output_path.display());
    Ok(())
}

/// Writes generated bridge client code to a file
#[allow(dead_code)]
pub async fn write_bridge_client_to_file(
    runtime: &Runtime,
    project_dir: &Path,
    bridge_repo: &impl ProviderRepositoryLike,
    output_path: &Path,
) -> Result<(), CommonError> {
    let code = generate_bridge_client(runtime, project_dir, bridge_repo).await?;

    std::fs::write(output_path, code).map_err(|e| {
        CommonError::Unknown(anyhow::anyhow!("Failed to write bridge client file: {e}"))
    })?;

    info!("Bridge client written to: {}", output_path.display());
    Ok(())
}

/// Regenerates and saves the bridge client for TypeScript/JavaScript runtimes
pub async fn regenerate_bridge_client(
    runtime: &Runtime,
    project_dir: &Path,
    bridge_repo: &impl ProviderRepositoryLike,
) -> Result<(), CommonError> {
    match runtime {
        Runtime::PnpmV1 => {
            let code =
                generate_typescript_code(&get_function_instances(bridge_repo).await?).await?;
            save_typescript_code(code, project_dir).await?;
            Ok(())
        }
    }
}

/// Serializable structure for provider in template
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProviderData {
    name: String,
    interface_name: String,
    accounts: Vec<AccountData>,
}

/// Serializable structure for account in template
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AccountData {
    name: String,
    provider_instance_id: String,
    functions: Vec<FunctionData>,
}

/// Serializable structure for function in template
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FunctionData {
    name: String,
    function_controller_type_id: String,
    params_type: String,
    params_type_name: String,
    return_type: String,
    return_type_name: String,
}

/// Generates TypeScript bridge client code
#[allow(dead_code)]
pub async fn generate_bridge_client(
    runtime: &Runtime,
    _project_dir: &Path,
    bridge_repo: &impl ProviderRepositoryLike,
) -> Result<String, CommonError> {
    info!("Generating bridge client for runtime: {:?}", runtime);

    // Get function instances from bridge
    let function_instances = get_function_instances(bridge_repo).await?;

    // Generate code based on runtime
    match runtime {
        Runtime::PnpmV1 => generate_typescript_code(&function_instances).await,
    }
}

/// Generates TypeScript code from function instances
async fn generate_typescript_code(
    function_instances: &[FunctionInstanceWithMetadata],
) -> Result<String, CommonError> {
    // Group function instances by provider and account
    let mut providers_map: HashMap<String, HashMap<String, Vec<FunctionInstanceWithMetadata>>> =
        HashMap::new();

    for func_metadata in function_instances {
        let provider_type_id = &func_metadata.provider_instance.provider_controller_type_id;
        let account_name = &func_metadata.provider_instance.display_name;

        providers_map
            .entry(provider_type_id.clone())
            .or_default()
            .entry(account_name.clone())
            .or_default()
            .push(func_metadata.clone());
    }

    // Build provider data for template
    let mut providers: Vec<ProviderData> = Vec::new();

    for (provider_type_id, accounts_map) in providers_map {
        let mut accounts: Vec<AccountData> = Vec::new();

        for (account_name, functions) in accounts_map {
            let mut function_data_list: Vec<FunctionData> = Vec::new();
            let mut provider_instance_id = String::new();

            for func_metadata in functions {
                // Get parameter schema
                let params_schema = func_metadata.function_controller.parameters();
                let params_type = convert_schema_to_typescript_type(&params_schema)?;

                // Get return schema
                let return_schema = func_metadata.function_controller.output();
                let return_type = convert_schema_to_typescript_type(&return_schema)?;

                // Store provider instance ID from the first function
                if provider_instance_id.is_empty() {
                    provider_instance_id = func_metadata.provider_instance.id.clone();
                }

                // Generate interface names
                let function_name_pascal = to_pascal_case(&sanitize_identifier(
                    &func_metadata.function_controller.type_id(),
                ));
                let provider_name_pascal = to_pascal_case(&sanitize_identifier(&provider_type_id));
                let params_type_name =
                    format!("{provider_name_pascal}{function_name_pascal}Params");
                let return_type_name =
                    format!("{provider_name_pascal}{function_name_pascal}Result");

                // Generate camelCase function name (stripped of provider prefix)
                let function_name_camel = strip_provider_prefix_and_camel_case(
                    &func_metadata.function_controller.type_id(),
                    &provider_type_id,
                );

                function_data_list.push(FunctionData {
                    name: function_name_camel,
                    function_controller_type_id: func_metadata.function_controller.type_id(),
                    params_type,
                    params_type_name,
                    return_type,
                    return_type_name,
                });
            }

            accounts.push(AccountData {
                name: account_name.clone(),
                provider_instance_id,
                functions: function_data_list,
            });
        }

        providers.push(ProviderData {
            name: to_camel_case(&sanitize_identifier(&provider_type_id)),
            interface_name: format!(
                "{}Provider",
                to_pascal_case(&sanitize_identifier(&provider_type_id))
            ),
            accounts,
        });
    }

    // Create Tera instance and render template
    let mut tera = Tera::default();
    tera.add_raw_template("typescript", TYPESCRIPT_TEMPLATE)
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to add template: {e}")))?;

    let mut context = Context::new();
    context.insert("providers", &providers);

    let rendered = tera
        .render("typescript", &context)
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to render template: {e}")))?;

    Ok(rendered)
}

/// Convert a JSON schema to TypeScript type
fn convert_schema_to_typescript_type(
    schema: &shared::primitives::WrappedSchema,
) -> Result<String, CommonError> {
    let json_schema = schema.get_inner().as_value();

    // Convert the schema to a TypeScript type
    json_schema_to_typescript(json_schema, 0)
}

/// Recursively convert JSON Schema to TypeScript type string
fn json_schema_to_typescript(
    value: &serde_json::Value,
    depth: usize,
) -> Result<String, CommonError> {
    // Prevent infinite recursion
    if depth > 10 {
        return Ok("any".to_string());
    }

    match value {
        serde_json::Value::Object(map) => {
            // Check for type field
            if let Some(type_val) = map.get("type") {
                match type_val.as_str() {
                    Some("string") => {
                        // Check for enum values
                        if let Some(enum_vals) = map.get("enum") {
                            if let Some(arr) = enum_vals.as_array() {
                                let variants: Vec<String> = arr
                                    .iter()
                                    .filter_map(|v| v.as_str())
                                    .map(|s| format!("\"{s}\""))
                                    .collect();
                                return Ok(variants.join(" | "));
                            }
                        }
                        Ok("string".to_string())
                    }
                    Some("number") | Some("integer") => Ok("number".to_string()),
                    Some("boolean") => Ok("boolean".to_string()),
                    Some("null") => Ok("null".to_string()),
                    Some("array") => {
                        if let Some(items) = map.get("items") {
                            let item_type = json_schema_to_typescript(items, depth + 1)?;
                            Ok(format!("Array<{item_type}>"))
                        } else {
                            Ok("Array<any>".to_string())
                        }
                    }
                    Some("object") => {
                        if let Some(properties) = map.get("properties") {
                            if let Some(props_map) = properties.as_object() {
                                let required = map
                                    .get("required")
                                    .and_then(|r| r.as_array())
                                    .map(|arr| {
                                        arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>()
                                    })
                                    .unwrap_or_default();

                                let mut fields = Vec::new();
                                for (key, prop_schema) in props_map {
                                    let prop_type =
                                        json_schema_to_typescript(prop_schema, depth + 1)?;
                                    let optional = if required.contains(&key.as_str()) {
                                        ""
                                    } else {
                                        "?"
                                    };
                                    fields.push(format!(
                                        "{}{}: {}",
                                        sanitize_identifier(key),
                                        optional,
                                        prop_type
                                    ));
                                }
                                return Ok(format!("{{ {} }}", fields.join("; ")));
                            }
                        }
                        Ok("Record<string, any>".to_string())
                    }
                    _ => Ok("any".to_string()),
                }
            } else if let Some(one_of) = map.get("oneOf") {
                // Handle oneOf (union types)
                if let Some(arr) = one_of.as_array() {
                    let types: Result<Vec<String>, CommonError> = arr
                        .iter()
                        .map(|v| json_schema_to_typescript(v, depth + 1))
                        .collect();
                    return Ok(types?.join(" | "));
                }
                Ok("any".to_string())
            } else if let Some(any_of) = map.get("anyOf") {
                // Handle anyOf (union types)
                if let Some(arr) = any_of.as_array() {
                    let types: Result<Vec<String>, CommonError> = arr
                        .iter()
                        .map(|v| json_schema_to_typescript(v, depth + 1))
                        .collect();
                    return Ok(types?.join(" | "));
                }
                Ok("any".to_string())
            } else if let Some(all_of) = map.get("allOf") {
                // Handle allOf (intersection types)
                if let Some(arr) = all_of.as_array() {
                    let types: Result<Vec<String>, CommonError> = arr
                        .iter()
                        .map(|v| json_schema_to_typescript(v, depth + 1))
                        .collect();
                    return Ok(types?.join(" & "));
                }
                Ok("any".to_string())
            } else {
                // No type specified, might be a reference or empty schema
                Ok("any".to_string())
            }
        }
        _ => Ok("any".to_string()),
    }
}

/// Sanitize identifier to be valid in TypeScript
fn sanitize_identifier(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Convert snake_case or kebab-case to PascalCase
fn to_pascal_case(s: &str) -> String {
    s.split(['_', '-'])
        .filter(|s| !s.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

/// Convert snake_case or kebab-case to camelCase
fn to_camel_case(s: &str) -> String {
    let parts: Vec<&str> = s.split(['_', '-']).filter(|s| !s.is_empty()).collect();

    if parts.is_empty() {
        return String::new();
    }

    let mut result = parts[0].to_lowercase();

    for part in &parts[1..] {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            result.push_str(&first.to_uppercase().collect::<String>());
            result.push_str(chars.as_str());
        }
    }

    result
}

/// Strip provider prefix from function name and convert to camelCase
/// e.g., "google_mail_send_email" with provider "google_mail" -> "sendEmail"
/// If function name is the same as provider, just convert to camelCase
fn strip_provider_prefix_and_camel_case(function_name: &str, provider_name: &str) -> String {
    let function_lower = function_name.to_lowercase();
    let provider_lower = provider_name.to_lowercase();

    // If function name equals provider name, just convert to camelCase
    if function_lower == provider_lower {
        return to_camel_case(function_name);
    }

    // Try to strip the provider prefix with underscore
    let stripped = if function_lower.starts_with(&format!("{provider_lower}_")) {
        &function_name[provider_lower.len() + 1..]
    } else if function_lower.starts_with(&provider_lower) {
        &function_name[provider_lower.len()..]
    } else {
        function_name
    };

    // If stripping results in empty string, use original function name
    if stripped.is_empty() {
        to_camel_case(function_name)
    } else {
        to_camel_case(stripped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_identifier() {
        assert_eq!(sanitize_identifier("my-function"), "my_function");
        assert_eq!(sanitize_identifier("my.function"), "my_function");
        assert_eq!(sanitize_identifier("my@function"), "my_function");
        assert_eq!(sanitize_identifier("myFunction"), "myFunction");
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("my_function"), "MyFunction");
        assert_eq!(to_pascal_case("my-function"), "MyFunction");
        assert_eq!(to_pascal_case("myFunction"), "MyFunction");
        assert_eq!(
            to_pascal_case("my_long_function_name"),
            "MyLongFunctionName"
        );
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("my_function"), "myFunction");
        assert_eq!(to_camel_case("my-function"), "myFunction");
        assert_eq!(to_camel_case("myFunction"), "myfunction");
        assert_eq!(to_camel_case("my_long_function_name"), "myLongFunctionName");
        assert_eq!(to_camel_case("google_mail"), "googleMail");
        assert_eq!(to_camel_case("approve_claim"), "approveClaim");
    }

    #[test]
    fn test_strip_provider_prefix_and_camel_case() {
        assert_eq!(
            strip_provider_prefix_and_camel_case("google_mail_send_email", "google_mail"),
            "sendEmail"
        );
        assert_eq!(
            strip_provider_prefix_and_camel_case("approve_claim", "approve_claim"),
            "approveClaim"
        );
        assert_eq!(
            strip_provider_prefix_and_camel_case("some_other_function", "some"),
            "otherFunction"
        );
    }
}
