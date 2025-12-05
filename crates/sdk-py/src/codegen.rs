use serde::{Deserialize, Serialize};
use shared::error::CommonError;
use std::collections::HashMap;
use tera::{Context, Tera};

/// Python template loaded at compile time
const PYTHON_TEMPLATE: &str = include_str!("python.py.tera");

/// Simplified data structures for code generation from API data
#[derive(Debug, Clone)]
pub struct FunctionInstanceData {
    pub provider_instance_id: String,
    pub provider_instance_display_name: String,
    pub provider_controller: ProviderControllerData,
    pub function_controller: FunctionControllerData,
}

#[derive(Debug, Clone)]
pub struct ProviderControllerData {
    pub type_id: String,
    pub display_name: String,
}

#[derive(Debug, Clone)]
pub struct FunctionControllerData {
    pub type_id: String,
    pub display_name: String,
    pub params_json_schema: Option<serde_json::Value>,
    pub return_value_json_schema: Option<serde_json::Value>,
}

/// Serializable structure for provider in template
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProviderData {
    name: String,
    class_name: String,
    sanitized_name: String,
    snake_case_name: String,
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

/// Generates Python code from API data
pub fn generate_python_code_from_api_data(
    function_instances: &[FunctionInstanceData],
) -> Result<String, CommonError> {
    // Group function instances by provider and account
    let mut providers_map: HashMap<String, HashMap<String, Vec<FunctionInstanceData>>> =
        HashMap::new();

    for func_data in function_instances {
        let provider_type_id = &func_data.provider_controller.type_id;
        let account_name = &func_data.provider_instance_display_name;

        providers_map
            .entry(provider_type_id.clone())
            .or_default()
            .entry(account_name.clone())
            .or_default()
            .push(func_data.clone());
    }

    // Build provider data for template
    let mut providers: Vec<ProviderData> = Vec::new();

    for (provider_type_id, accounts_map) in providers_map {
        let mut accounts: Vec<AccountData> = Vec::new();

        for (account_name, functions) in accounts_map {
            let mut function_data_list: Vec<FunctionData> = Vec::new();
            let mut provider_instance_id = String::new();

            for func_data in functions {
                // Get parameter schema
                let params_type =
                    if let Some(schema) = &func_data.function_controller.params_json_schema {
                        json_schema_to_python(schema, 0)?
                    } else {
                        "None".to_string()
                    };

                // Get return schema
                let return_type =
                    if let Some(schema) = &func_data.function_controller.return_value_json_schema {
                        json_schema_to_python(schema, 0)?
                    } else {
                        "None".to_string()
                    };

                // Store provider instance ID from the first function
                if provider_instance_id.is_empty() {
                    provider_instance_id = func_data.provider_instance_id.clone();
                }

                // Generate class names
                let function_name_pascal =
                    to_pascal_case(&sanitize_identifier(&func_data.function_controller.type_id));
                let provider_name_pascal = to_pascal_case(&sanitize_identifier(&provider_type_id));
                let params_type_name =
                    format!("{provider_name_pascal}{function_name_pascal}Params");
                let return_type_name =
                    format!("{provider_name_pascal}{function_name_pascal}Result");

                // Generate snake_case function name (stripped of provider prefix)
                let function_name_snake = strip_provider_prefix_and_snake_case(
                    &func_data.function_controller.type_id,
                    &provider_type_id,
                );

                function_data_list.push(FunctionData {
                    name: function_name_snake,
                    function_controller_type_id: func_data.function_controller.type_id.clone(),
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

        let provider_name_pascal = to_pascal_case(&sanitize_identifier(&provider_type_id));
        let sanitized_name = sanitize_identifier(&provider_type_id);
        let snake_case_name = to_snake_case(&provider_type_id);
        providers.push(ProviderData {
            name: provider_type_id.clone(),
            class_name: provider_name_pascal,
            sanitized_name,
            snake_case_name,
            accounts,
        });
    }

    // Create Tera instance and render template
    let mut tera = Tera::default();
    tera.add_raw_template("python", PYTHON_TEMPLATE)
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to add template: {e}")))?;

    let mut context = Context::new();
    context.insert("providers", &providers);

    let rendered = tera
        .render("python", &context)
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to render template: {e}")))?;

    Ok(rendered)
}

/// Recursively convert JSON Schema to Python type string
fn json_schema_to_python(value: &serde_json::Value, depth: usize) -> Result<String, CommonError> {
    // Prevent infinite recursion
    if depth > 10 {
        return Ok("Any".to_string());
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
                                return Ok(format!("Literal[{}]", variants.join(", ")));
                            }
                        }
                        Ok("str".to_string())
                    }
                    Some("number") => Ok("float".to_string()),
                    Some("integer") => Ok("int".to_string()),
                    Some("boolean") => Ok("bool".to_string()),
                    Some("null") => Ok("None".to_string()),
                    Some("array") => {
                        if let Some(items) = map.get("items") {
                            let item_type = json_schema_to_python(items, depth + 1)?;
                            Ok(format!("list[{item_type}]"))
                        } else {
                            Ok("list[Any]".to_string())
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
                                    let prop_type = json_schema_to_python(prop_schema, depth + 1)?;
                                    let field_type = if required.contains(&key.as_str()) {
                                        prop_type
                                    } else {
                                        format!("Optional[{prop_type}]")
                                    };
                                    fields.push(format!(
                                        "\"{}\": {}",
                                        sanitize_identifier(key),
                                        field_type
                                    ));
                                }
                                return Ok(format!(
                                    "TypedDict(\"_\", {{ {} }})",
                                    fields.join(", ")
                                ));
                            }
                        }
                        Ok("dict[str, Any]".to_string())
                    }
                    _ => Ok("Any".to_string()),
                }
            } else if let Some(one_of) = map.get("oneOf") {
                // Handle oneOf (union types)
                if let Some(arr) = one_of.as_array() {
                    let types: Result<Vec<String>, CommonError> = arr
                        .iter()
                        .map(|v| json_schema_to_python(v, depth + 1))
                        .collect();
                    return Ok(format!("Union[{}]", types?.join(", ")));
                }
                Ok("Any".to_string())
            } else if let Some(any_of) = map.get("anyOf") {
                // Handle anyOf (union types)
                if let Some(arr) = any_of.as_array() {
                    let types: Result<Vec<String>, CommonError> = arr
                        .iter()
                        .map(|v| json_schema_to_python(v, depth + 1))
                        .collect();
                    return Ok(format!("Union[{}]", types?.join(", ")));
                }
                Ok("Any".to_string())
            } else if let Some(all_of) = map.get("allOf") {
                // Handle allOf - in Python we'd need to merge TypedDicts, simplified to dict
                if let Some(_arr) = all_of.as_array() {
                    return Ok("dict[str, Any]".to_string());
                }
                Ok("Any".to_string())
            } else {
                // No type specified, might be a reference or empty schema
                Ok("Any".to_string())
            }
        }
        _ => Ok("Any".to_string()),
    }
}

/// Sanitize identifier to be valid in Python
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

/// Convert PascalCase or kebab-case to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_lower = false;

    for c in s.chars() {
        if c == '-' || c == '_' {
            result.push('_');
            prev_lower = false;
        } else if c.is_uppercase() {
            if prev_lower {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
            prev_lower = false;
        } else {
            result.push(c);
            prev_lower = true;
        }
    }

    result
}

/// Strip provider prefix from function name and convert to snake_case
/// e.g., "google_mail_send_email" with provider "google_mail" -> "send_email"
/// If function name is the same as provider, just convert to snake_case
fn strip_provider_prefix_and_snake_case(function_name: &str, provider_name: &str) -> String {
    let function_lower = function_name.to_lowercase();
    let provider_lower = provider_name.to_lowercase();

    // If function name equals provider name, just convert to snake_case
    if function_lower == provider_lower {
        return to_snake_case(function_name);
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
        to_snake_case(function_name)
    } else {
        to_snake_case(stripped)
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
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("MyFunction"), "my_function");
        assert_eq!(to_snake_case("my-function"), "my_function");
        assert_eq!(to_snake_case("myFunction"), "my_function");
        assert_eq!(to_snake_case("google_mail"), "google_mail");
        assert_eq!(to_snake_case("approve_claim"), "approve_claim");
    }

    #[test]
    fn test_strip_provider_prefix_and_snake_case() {
        assert_eq!(
            strip_provider_prefix_and_snake_case("google_mail_send_email", "google_mail"),
            "send_email"
        );
        assert_eq!(
            strip_provider_prefix_and_snake_case("approve_claim", "approve_claim"),
            "approve_claim"
        );
        assert_eq!(
            strip_provider_prefix_and_snake_case("some_other_function", "some"),
            "other_function"
        );
    }
}
