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
    params_type_name: String,
    params_type_classes: Vec<TypedDictClass>,
    return_type_name: String,
    return_type_classes: Vec<TypedDictClass>,
}

/// Represents a TypedDict class definition
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TypedDictClass {
    name: String,
    fields: Vec<TypedDictField>,
}

/// Represents a field in a TypedDict class
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TypedDictField {
    name: String,
    type_annotation: String,
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

                // Generate TypedDict classes for params
                let params_type_classes =
                    if let Some(schema) = &func_data.function_controller.params_json_schema {
                        generate_typed_dict_classes(schema, &params_type_name, Some(schema))?
                    } else {
                        vec![TypedDictClass {
                            name: params_type_name.clone(),
                            fields: vec![],
                        }]
                    };

                // Generate TypedDict classes for return value
                let return_type_classes =
                    if let Some(schema) = &func_data.function_controller.return_value_json_schema {
                        generate_typed_dict_classes(schema, &return_type_name, Some(schema))?
                    } else {
                        vec![TypedDictClass {
                            name: return_type_name.clone(),
                            fields: vec![],
                        }]
                    };

                // Generate snake_case function name (stripped of provider prefix)
                let function_name_snake = strip_provider_prefix_and_snake_case(
                    &func_data.function_controller.type_id,
                    &provider_type_id,
                );

                function_data_list.push(FunctionData {
                    name: function_name_snake,
                    function_controller_type_id: func_data.function_controller.type_id.clone(),
                    params_type_name,
                    params_type_classes,
                    return_type_name,
                    return_type_classes,
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

/// Recursively convert JSON Schema to Python type string (convenience wrapper)
#[allow(dead_code)]
fn json_schema_to_python(value: &serde_json::Value, depth: usize) -> Result<String, CommonError> {
    json_schema_to_python_with_defs(value, depth, None)
}

/// Recursively convert JSON Schema to Python type string, with $defs support
fn json_schema_to_python_with_defs(
    value: &serde_json::Value,
    depth: usize,
    root_schema: Option<&serde_json::Value>,
) -> Result<String, CommonError> {
    // Prevent infinite recursion
    if depth > 10 {
        return Ok("Any".to_string());
    }

    match value {
        serde_json::Value::Object(map) => {
            // Handle $ref - resolve reference to $defs
            if let Some(ref_val) = map.get("$ref") {
                if let Some(ref_str) = ref_val.as_str() {
                    // Parse reference like "#/$defs/ClaimInput"
                    if ref_str.starts_with("#/$defs/") {
                        let def_name = &ref_str[8..]; // Skip "#/$defs/"
                        // Use root_schema if provided, otherwise try to use value itself
                        let schema_to_search = root_schema.unwrap_or(value);
                        if let Some(defs) = schema_to_search.get("$defs") {
                            if let Some(def_schema) = defs.get(def_name) {
                                return json_schema_to_python_with_defs(
                                    def_schema,
                                    depth + 1,
                                    root_schema.or(Some(schema_to_search)),
                                );
                            }
                        }
                    }
                    // Could not resolve reference, return Any
                    return Ok("Any".to_string());
                }
            }

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
                            let item_type = json_schema_to_python_with_defs(
                                items,
                                depth + 1,
                                root_schema.or(Some(value)),
                            )?;
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
                                    let prop_type = json_schema_to_python_with_defs(
                                        prop_schema,
                                        depth + 1,
                                        root_schema.or(Some(value)),
                                    )?;
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
                        .map(|v| {
                            json_schema_to_python_with_defs(
                                v,
                                depth + 1,
                                root_schema.or(Some(value)),
                            )
                        })
                        .collect();
                    return Ok(format!("Union[{}]", types?.join(", ")));
                }
                Ok("Any".to_string())
            } else if let Some(any_of) = map.get("anyOf") {
                // Handle anyOf (union types)
                if let Some(arr) = any_of.as_array() {
                    let types: Result<Vec<String>, CommonError> = arr
                        .iter()
                        .map(|v| {
                            json_schema_to_python_with_defs(
                                v,
                                depth + 1,
                                root_schema.or(Some(value)),
                            )
                        })
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

/// Generate TypedDict class definitions from a JSON Schema
/// Returns a list of TypedDict classes (nested classes first, main class last)
fn generate_typed_dict_classes(
    schema: &serde_json::Value,
    class_name: &str,
    root_schema: Option<&serde_json::Value>,
) -> Result<Vec<TypedDictClass>, CommonError> {
    let mut classes = Vec::new();
    generate_typed_dict_classes_recursive(schema, class_name, root_schema, &mut classes, 0)?;
    Ok(classes)
}

/// Recursive helper for generating TypedDict classes
fn generate_typed_dict_classes_recursive(
    schema: &serde_json::Value,
    class_name: &str,
    root_schema: Option<&serde_json::Value>,
    classes: &mut Vec<TypedDictClass>,
    depth: usize,
) -> Result<String, CommonError> {
    // Prevent infinite recursion
    if depth > 10 {
        return Ok("object".to_string());
    }

    match schema {
        serde_json::Value::Object(map) => {
            // Handle $ref - resolve reference to $defs
            if let Some(ref_val) = map.get("$ref") {
                if let Some(ref_str) = ref_val.as_str() {
                    if ref_str.starts_with("#/$defs/") {
                        let def_name = &ref_str[8..];
                        let schema_to_search = root_schema.unwrap_or(schema);
                        if let Some(defs) = schema_to_search.get("$defs") {
                            if let Some(def_schema) = defs.get(def_name) {
                                return generate_typed_dict_classes_recursive(
                                    def_schema,
                                    class_name,
                                    root_schema.or(Some(schema_to_search)),
                                    classes,
                                    depth + 1,
                                );
                            }
                        }
                    }
                    return Ok("object".to_string());
                }
            }

            // Check for type field
            if let Some(type_val) = map.get("type") {
                match type_val.as_str() {
                    Some("string") => {
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
                            let item_class_name = format!("{class_name}Item");
                            let item_type = generate_typed_dict_classes_recursive(
                                items,
                                &item_class_name,
                                root_schema.or(Some(schema)),
                                classes,
                                depth + 1,
                            )?;
                            Ok(format!("list[{item_type}]"))
                        } else {
                            Ok("list[object]".to_string())
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
                                    let field_name = sanitize_identifier(key);
                                    let nested_class_name =
                                        format!("_{class_name}{}", to_pascal_case(&field_name));

                                    let prop_type = generate_typed_dict_classes_recursive(
                                        prop_schema,
                                        &nested_class_name,
                                        root_schema.or(Some(schema)),
                                        classes,
                                        depth + 1,
                                    )?;

                                    let field_type = if required.contains(&key.as_str()) {
                                        prop_type
                                    } else {
                                        format!("{prop_type} | None")
                                    };

                                    fields.push(TypedDictField {
                                        name: field_name,
                                        type_annotation: field_type,
                                    });
                                }

                                // Sort fields for deterministic output
                                fields.sort_by(|a, b| a.name.cmp(&b.name));

                                classes.push(TypedDictClass {
                                    name: class_name.to_string(),
                                    fields,
                                });

                                return Ok(class_name.to_string());
                            }
                        }
                        Ok("dict[str, object]".to_string())
                    }
                    _ => Ok("object".to_string()),
                }
            } else if let Some(one_of) = map.get("oneOf") {
                if let Some(arr) = one_of.as_array() {
                    let types: Result<Vec<String>, CommonError> = arr
                        .iter()
                        .enumerate()
                        .map(|(i, v)| {
                            let variant_name = format!("{class_name}Variant{i}");
                            generate_typed_dict_classes_recursive(
                                v,
                                &variant_name,
                                root_schema.or(Some(schema)),
                                classes,
                                depth + 1,
                            )
                        })
                        .collect();
                    return Ok(types?.join(" | "));
                }
                Ok("object".to_string())
            } else if let Some(any_of) = map.get("anyOf") {
                if let Some(arr) = any_of.as_array() {
                    let types: Result<Vec<String>, CommonError> = arr
                        .iter()
                        .enumerate()
                        .map(|(i, v)| {
                            let variant_name = format!("{class_name}Variant{i}");
                            generate_typed_dict_classes_recursive(
                                v,
                                &variant_name,
                                root_schema.or(Some(schema)),
                                classes,
                                depth + 1,
                            )
                        })
                        .collect();
                    return Ok(types?.join(" | "));
                }
                Ok("object".to_string())
            } else {
                Ok("object".to_string())
            }
        }
        _ => Ok("object".to_string()),
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

    #[test]
    fn test_json_schema_to_python_with_ref() {
        // Test schema with $ref to $defs
        let schema: serde_json::Value = serde_json::json!({
            "type": "object",
            "properties": {
                "claim": {
                    "$ref": "#/$defs/ClaimInput"
                }
            },
            "required": ["claim"],
            "$defs": {
                "ClaimInput": {
                    "type": "object",
                    "properties": {
                        "date": {"type": "string"},
                        "category": {"type": "string"},
                        "amount": {"type": "number"}
                    },
                    "required": ["date", "category", "amount"]
                }
            }
        });

        let result = json_schema_to_python_with_defs(&schema, 0, Some(&schema)).unwrap();

        // Should generate TypedDict with nested TypedDict for claim
        assert!(result.contains("TypedDict"));
        assert!(result.contains("\"claim\""));
        // The inner claim should also be a TypedDict with date, category, amount
        assert!(result.contains("\"date\": str"));
        assert!(result.contains("\"category\": str"));
        assert!(result.contains("\"amount\": float"));
    }

    #[test]
    fn test_json_schema_to_python_simple_types() {
        let string_schema: serde_json::Value = serde_json::json!({"type": "string"});
        assert_eq!(json_schema_to_python(&string_schema, 0).unwrap(), "str");

        let int_schema: serde_json::Value = serde_json::json!({"type": "integer"});
        assert_eq!(json_schema_to_python(&int_schema, 0).unwrap(), "int");

        let float_schema: serde_json::Value = serde_json::json!({"type": "number"});
        assert_eq!(json_schema_to_python(&float_schema, 0).unwrap(), "float");

        let bool_schema: serde_json::Value = serde_json::json!({"type": "boolean"});
        assert_eq!(json_schema_to_python(&bool_schema, 0).unwrap(), "bool");
    }
}
