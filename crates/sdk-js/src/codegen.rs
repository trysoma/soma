use serde::{Deserialize, Serialize};
use shared::error::CommonError;
use std::collections::HashMap;
use tera::{Context, Tera};

/// TypeScript bridge template loaded at compile time
const BRIDGE_TEMPLATE: &str = include_str!("bridge.ts.tera");

/// TypeScript agents template loaded at compile time
const AGENTS_TEMPLATE: &str = include_str!("agents.ts.tera");

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
    interface_name: String,
    sanitized_name: String,
    camel_case_name: String,
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

/// Agent data structure for code generation
#[derive(Debug, Clone)]
pub struct AgentData {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub description: String,
}

/// Serializable agent data for template
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AgentTemplateData {
    id: String,
    project_id: String,
    name: String,
    description: String,
    camel_case_id: String,
    var_name: String,
}

/// Serializable project data for template (groups agents by project)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProjectTemplateData {
    id: String,
    interface_name: String,
    camel_case_id: String,
    agents: Vec<AgentTemplateData>,
}

/// Generates TypeScript code from API data
pub fn generate_typescript_code_from_api_data(
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
                        json_schema_to_typescript(schema, 0)?
                    } else {
                        "void".to_string()
                    };

                // Get return schema
                let return_type =
                    if let Some(schema) = &func_data.function_controller.return_value_json_schema {
                        json_schema_to_typescript(schema, 0)?
                    } else {
                        "void".to_string()
                    };

                // Store provider instance ID from the first function
                if provider_instance_id.is_empty() {
                    provider_instance_id = func_data.provider_instance_id.clone();
                }

                // Generate interface names
                let function_name_pascal =
                    to_pascal_case(&sanitize_identifier(&func_data.function_controller.type_id));
                let provider_name_pascal = to_pascal_case(&sanitize_identifier(&provider_type_id));
                let params_type_name =
                    format!("{provider_name_pascal}{function_name_pascal}Params");
                let return_type_name =
                    format!("{provider_name_pascal}{function_name_pascal}Result");

                // Generate camelCase function name (stripped of provider prefix)
                let function_name_camel = strip_provider_prefix_and_camel_case(
                    &func_data.function_controller.type_id,
                    &provider_type_id,
                );

                function_data_list.push(FunctionData {
                    name: function_name_camel,
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
        let camel_case_name = to_camel_case(&provider_type_id);
        providers.push(ProviderData {
            name: provider_type_id.clone(),
            interface_name: provider_name_pascal,
            sanitized_name,
            camel_case_name,
            accounts,
        });
    }

    // Create Tera instance and render template
    let mut tera = Tera::default();
    tera.add_raw_template("bridge", BRIDGE_TEMPLATE)
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to add template: {e}")))?;

    let mut context = Context::new();
    context.insert("providers", &providers);

    let rendered = tera
        .render("bridge", &context)
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to render template: {e}")))?;

    Ok(rendered)
}

/// Generates TypeScript agents code from agent data
pub fn generate_typescript_agents_code(agents: &[AgentData]) -> Result<String, CommonError> {
    // Group agents by project
    let mut projects_map: HashMap<String, Vec<&AgentData>> = HashMap::new();

    for agent in agents {
        projects_map
            .entry(agent.project_id.clone())
            .or_default()
            .push(agent);
    }

    // Build flat list of agents for template
    let agents_template: Vec<AgentTemplateData> = agents
        .iter()
        .map(|agent| AgentTemplateData {
            id: agent.id.clone(),
            project_id: agent.project_id.clone(),
            name: agent.name.clone(),
            description: agent.description.clone(),
            camel_case_id: to_camel_case(&agent.id),
            var_name: format!(
                "{}_{}",
                to_camel_case(&agent.project_id),
                to_camel_case(&agent.id)
            ),
        })
        .collect();

    // Build project data for template
    let projects: Vec<ProjectTemplateData> = projects_map
        .into_iter()
        .map(|(project_id, project_agents)| {
            let agents: Vec<AgentTemplateData> = project_agents
                .iter()
                .map(|agent| AgentTemplateData {
                    id: agent.id.clone(),
                    project_id: agent.project_id.clone(),
                    name: agent.name.clone(),
                    description: agent.description.clone(),
                    camel_case_id: to_camel_case(&agent.id),
                    var_name: format!(
                        "{}_{}",
                        to_camel_case(&agent.project_id),
                        to_camel_case(&agent.id)
                    ),
                })
                .collect();

            ProjectTemplateData {
                id: project_id.clone(),
                interface_name: to_pascal_case(&project_id),
                camel_case_id: to_camel_case(&project_id),
                agents,
            }
        })
        .collect();

    // Create Tera instance and render template
    let mut tera = Tera::default();
    tera.add_raw_template("agents", AGENTS_TEMPLATE)
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to add agents template: {e}")))?;

    let mut context = Context::new();
    context.insert("agents", &agents_template);
    context.insert("projects", &projects);

    let rendered = tera.render("agents", &context).map_err(|e| {
        CommonError::Unknown(anyhow::anyhow!("Failed to render agents template: {e}"))
    })?;

    Ok(rendered)
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

/// Convert snake_case, kebab-case, or camelCase to PascalCase
fn to_pascal_case(s: &str) -> String {
    // First split by underscore and hyphen
    let parts: Vec<&str> = s.split(['_', '-']).filter(|s| !s.is_empty()).collect();

    let mut result = String::new();

    for part in &parts {
        // Split each part by uppercase letters to handle camelCase
        let words = split_on_uppercase(part);

        for word in words {
            if word.is_empty() {
                continue;
            }
            // Capitalize first letter of each word
            let mut chars = word.chars();
            if let Some(first) = chars.next() {
                result.push_str(&first.to_uppercase().collect::<String>());
                result.push_str(&chars.as_str().to_lowercase());
            }
        }
    }

    result
}

/// Convert snake_case, kebab-case, or PascalCase to camelCase
/// Preserves existing camelCase strings
fn to_camel_case(s: &str) -> String {
    // First split by underscore and hyphen
    let parts: Vec<&str> = s.split(['_', '-']).filter(|s| !s.is_empty()).collect();

    if parts.is_empty() {
        return String::new();
    }

    let mut result = String::new();
    let mut is_first_word = true;

    for part in &parts {
        // Split each part by uppercase letters to handle PascalCase/camelCase
        let words = split_on_uppercase(part);

        for word in words {
            if word.is_empty() {
                continue;
            }

            if is_first_word {
                // First word should be all lowercase
                result.push_str(&word.to_lowercase());
                is_first_word = false;
            } else {
                // Subsequent words should have first letter uppercase, rest lowercase
                let mut chars = word.chars();
                if let Some(first) = chars.next() {
                    result.push_str(&first.to_uppercase().collect::<String>());
                    result.push_str(&chars.as_str().to_lowercase());
                }
            }
        }
    }

    result
}

/// Split a string on uppercase letters, preserving the uppercase letter at the start of each segment
fn split_on_uppercase(s: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current_word = String::new();

    for c in s.chars() {
        if c.is_uppercase() && !current_word.is_empty() {
            words.push(current_word);
            current_word = String::new();
        }
        current_word.push(c);
    }

    if !current_word.is_empty() {
        words.push(current_word);
    }

    words
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

#[cfg(all(test, feature = "unit_test"))]
mod unit_test {
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
        assert_eq!(to_pascal_case("claimResearchAgent"), "ClaimResearchAgent");
        assert_eq!(to_pascal_case("ClaimResearchAgent"), "ClaimResearchAgent");
        assert_eq!(to_pascal_case("acme"), "Acme");
        assert_eq!(
            to_pascal_case("my_long_function_name"),
            "MyLongFunctionName"
        );
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("my_function"), "myFunction");
        assert_eq!(to_camel_case("my-function"), "myFunction");
        assert_eq!(to_camel_case("myFunction"), "myFunction");
        assert_eq!(to_camel_case("MyFunction"), "myFunction");
        assert_eq!(to_camel_case("claimResearchAgent"), "claimResearchAgent");
        assert_eq!(to_camel_case("ClaimResearchAgent"), "claimResearchAgent");
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
