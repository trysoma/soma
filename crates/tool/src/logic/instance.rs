use std::sync::Arc;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shared::{
    error::CommonError,
    primitives::{
        PaginatedResponse, PaginationRequest, WrappedChronoDateTime, WrappedJsonValue,
        WrappedUuidV4,
    },
};
use shared_macros::{authn, authz_role};
use tracing::trace;
use utoipa::{
    IntoParams, ToSchema,
    openapi::{
        Components, Content, HttpMethod, ObjectBuilder, OpenApi, Paths, Ref, RefOr, Required,
        Response, Type, path::Operation, request_body::RequestBody, schema::SchemaType,
    },
};

use crate::{
    logic::{
        ToolLike, InvokeResult, OnConfigChangeEvt, OnConfigChangeTx,
        ToolGroupLike,
        deployment::{
            ToolDeploymentSerialized, ToolGroupDeploymentSerialized,
            ToolGroupCredentialDeploymentSerialized, WithCredentialDeploymentTypeId,
            WithToolDeploymentTypeId, WithToolGroupDeploymentTypeId,
        },
        credential::{ResourceServerCredentialSerialized, UserCredentialSerialized},
    },
    repository::ProviderRepositoryLike,
    router::{API_VERSION_1, PATH_PREFIX, SERVICE_ROUTE_KEY},
};
use encryption::logic::crypto_services::CryptoCache;

/// Sanitizes a display name to only contain alphanumeric characters and dashes.
/// This is useful for creating valid OpenAPI operation IDs and other identifiers.
fn sanitize_display_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else if c.is_whitespace() {
                '-'
            } else {
                // Skip other special characters
                '\0'
            }
        })
        .filter(|&c| c != '\0')
        .collect()
}

/// Converts a JSON Schema (from schemars) to OpenAPI 3.0 schema format.
/// This extracts the schema content and moves $defs to a separate map.
/// Returns (converted_schema, extracted_defs)
fn convert_jsonschema_to_openapi(
    json_schema: &serde_json::Value,
    schema_name_prefix: &str,
) -> Result<(serde_json::Value, Vec<(String, serde_json::Value)>), CommonError> {
    let mut schema = json_schema.clone();
    let mut extracted_defs = Vec::new();
    let mut ref_updates = Vec::new();

    // Remove the $schema field (not used in OpenAPI)
    if let Some(obj) = schema.as_object_mut() {
        obj.remove("$schema");

        // Extract and move $defs to components
        if let Some(defs) = obj.remove("$defs") {
            if let Some(defs_obj) = defs.as_object() {
                // Build a map of all definitions for reference resolution
                for (def_name, _def_schema) in defs_obj {
                    let component_name = format!("{schema_name_prefix}_{def_name}");

                    // Store reference updates to do later
                    ref_updates.push((
                        format!("#/$defs/{def_name}"),
                        format!("#/components/schemas/{component_name}"),
                    ));
                }

                // Now process each definition
                for (def_name, def_schema) in defs_obj {
                    let component_name = format!("{schema_name_prefix}_{def_name}");

                    // Clone and clean the def schema
                    let mut clean_def = def_schema.clone();
                    if let Some(def_obj) = clean_def.as_object_mut() {
                        def_obj.remove("$schema");
                        def_obj.remove("title");
                    }

                    // Update references within this definition to point to components
                    for (old_ref, new_ref) in &ref_updates {
                        update_refs_in_schema(&mut clean_def, old_ref, new_ref);
                    }

                    extracted_defs.push((component_name.clone(), clean_def));
                }
            }
        }

        // Remove title field if it's redundant
        obj.remove("title");
    }

    // Update all references in the main schema after we're done with the mutable borrow
    for (old_ref, new_ref) in &ref_updates {
        update_refs_in_schema(&mut schema, old_ref, new_ref);
    }

    Ok((schema, extracted_defs))
}

/// Recursively updates $ref values in a JSON schema
fn update_refs_in_schema(value: &mut serde_json::Value, old_ref: &str, new_ref: &str) {
    match value {
        serde_json::Value::Object(map) => {
            // Update $ref if it matches
            if let Some(ref_val) = map.get_mut("$ref") {
                if ref_val.as_str() == Some(old_ref) {
                    *ref_val = serde_json::Value::String(new_ref.to_string());
                }
            }
            // Recursively process all values
            for v in map.values_mut() {
                update_refs_in_schema(v, old_ref, new_ref);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr.iter_mut() {
                update_refs_in_schema(v, old_ref, new_ref);
            }
        }
        _ => {}
    }
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct ToolGroupSerialized {
    // not UUID as some ID's will be deterministic
    pub id: String,
    pub display_name: String,
    pub resource_server_credential_id: WrappedUuidV4,
    pub user_credential_id: Option<WrappedUuidV4>,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
    pub tool_group_deployment_type_id: String,
    pub credential_deployment_type_id: String,
    pub status: String,
    pub return_on_successful_brokering: Option<ReturnAddress>,
}

#[derive(Serialize, Deserialize, Clone, ToSchema, Debug)]
pub struct ReturnAddressUrl {
    pub url: String,
}

#[derive(Serialize, Deserialize, Clone, ToSchema, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ReturnAddress {
    Url(ReturnAddressUrl),
}

// Repository layer struct - includes functions and credentials from SQL join
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct ToolGroupSerializedWithTools {
    pub tool_group_instance: ToolGroupSerialized,
    pub tools: Vec<ToolSerialized>,
    pub resource_server_credential: ResourceServerCredentialSerialized,
    pub user_credential: Option<UserCredentialSerialized>,
}

// Repository layer struct - includes credentials without functions
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct ToolGroupSerializedWithCredentials {
    pub tool_group_instance: ToolGroupSerialized,
    pub resource_server_credential: ResourceServerCredentialSerialized,
    pub user_credential: Option<UserCredentialSerialized>,
}

// List response struct - enriched with source metadata
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct ToolGroupListItem {
    #[serde(flatten)]
    pub tool_group_instance: ToolGroupSerialized,
    pub tools: Vec<ToolListItem>,
    pub source: ToolGroupDeploymentSerialized,
    pub credential_source: ToolGroupCredentialDeploymentSerialized,
}

// List response struct for function instances
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct ToolListItem {
    #[serde(flatten)]
    pub tool_instance: ToolSerialized,
    pub source: ToolDeploymentSerialized,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct ToolGroupSerializedWithEverything {
    #[serde(flatten)]
    pub instance_data: ToolGroupSerializedWithCredentials,
    pub tools: Vec<ToolListItem>,
    pub source: ToolGroupDeploymentSerialized,
    pub credential_source: ToolGroupCredentialDeploymentSerialized,
}

// we shouldn't need this besides the fact that we want to keep track of functions intentionally enabled
// by users. if all functions were enabled, always, we could drop this struct.
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct ToolSerialized {
    pub tool_deployment_type_id: String,
    pub tool_group_deployment_type_id: String,
    pub tool_group_id: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct ToolSerializedWithEverything {
    #[serde(flatten)]
    pub tool_instance: ToolSerializedWithCredentials,
    pub source: ToolDeploymentSerialized,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct ToolSerializedWithCredentials {
    pub tool_instance: ToolSerialized,
    pub tool_group_instance: ToolGroupSerialized,
    pub resource_server_credential: ResourceServerCredentialSerialized,
    pub user_credential: Option<UserCredentialSerialized>,
}

#[derive(Debug, Clone)]
pub struct ListToolGroupsParams {
    pub pagination: PaginationRequest,
    pub status: Option<String>,
    pub tool_group_deployment_type_id: Option<String>,
}

pub type ListToolGroupInstancesResponse = PaginatedResponse<ToolGroupListItem>;

/// List all provider instances with optional filtering (internal implementation)
pub async fn list_tool_groups_internal(
    _repo: &impl crate::repository::ProviderRepositoryLike,
    _params: ListToolGroupsParams,
) -> Result<ListToolGroupInstancesResponse, CommonError> {
    // TODO: Refactor to fetch tool group sources from repository instead of hardcoded sources
    // The old implementation used hardcoded in-memory sources to enrich the response with source metadata.
    // This needs to be refactored to:
    // 1. Fetch tool group source definitions from the repository
    // 2. Enrich the response with source metadata from the database
    // 3. Support listing without requiring in-memory source registry
    Err(CommonError::Unknown(anyhow::anyhow!(
        "Listing tool group instances is temporarily unavailable. \
        Tool group sources need to be registered via API before instances can be listed."
    )))
}

/// List all tool group instances with optional filtering
#[authz_role(Admin, Maintainer, permission = "tool_group:read")]
#[authn]
pub async fn list_tool_groups(
    repo: &impl crate::repository::ProviderRepositoryLike,
    params: ListToolGroupsParams,
) -> Result<ListToolGroupInstancesResponse, CommonError> {
    list_tool_group_instances_internal(repo, params).await
}

#[derive(Debug, Clone)]
pub struct ListToolsParams {
    pub pagination: PaginationRequest,
    pub tool_group_id: Option<String>,
}

pub type ListToolInstancesResponse = PaginatedResponse<ToolSerialized>;

/// List all function instances with optional filtering (internal implementation)
pub async fn list_tools_internal(
    repo: &impl crate::repository::ProviderRepositoryLike,
    params: ListToolsParams,
) -> Result<ListToolInstancesResponse, CommonError> {
    trace!(
        page_size = params.pagination.page_size,
        tool_group_instance_id = ?params.tool_group_instance_id,
        "Fetching function instances from repository"
    );
    let tool_instances = repo
        .list_tool_instances(&params.pagination, params.tool_group_instance_id.as_deref())
        .await?;
    Ok(tool_instances)
}

/// List all tool instances with optional filtering
#[authz_role(Admin, Maintainer, permission = "tool:read")]
#[authn]
pub async fn list_tools(
    repo: &impl crate::repository::ProviderRepositoryLike,
    params: ListToolsParams,
) -> Result<ListToolInstancesResponse, CommonError> {
    list_tool_instances_internal(repo, params).await
}

/// Represents a function instance with all associated metadata needed for code generation
#[derive(Clone)]
pub struct ToolWithMetadata {
    pub tool_group_instance: ToolGroupSerialized,
    pub tool_instance: ToolSerialized,
    pub tool_group: Arc<dyn ToolGroupLike>,
    pub tool_source: Arc<dyn ToolLike>,
}

/// Returns all tool instances with their associated controllers and metadata.
/// This is the core data structure that can be used for client code generation.
#[authz_role(Admin, Maintainer, permission = "tool:read")]
#[authn]
pub async fn get_tools(
    repo: &impl crate::repository::ProviderRepositoryLike,
) -> Result<Vec<ToolWithMetadata>, CommonError> {
    get_tool_instances_internal(repo).await
}

/// Internal function to get tool instances (no auth check).
/// Used by `get_tool_instances` and `get_tool_instances_openapi_spec`.
/// Also exposed for internal calls from soma-api-server codegen.
pub async fn get_tools_internal(
    _repo: &impl crate::repository::ProviderRepositoryLike,
) -> Result<Vec<ToolWithMetadata>, CommonError> {
    // TODO: Refactor to fetch tool sources from repository instead of hardcoded sources
    // The old implementation used hardcoded in-memory sources which have been removed.
    // This needs to be refactored to:
    // 1. Fetch tool group and tool source definitions from the repository
    // 2. Build ToolWithMetadata from stored definitions
    // 3. Support OpenAPI spec generation from database-stored schemas
    Err(CommonError::Unknown(anyhow::anyhow!(
        "Getting tool instances with metadata is temporarily unavailable. \
        Tool sources need to be registered via API before tool instances can be retrieved."
    )))
}

/// Returns an OpenAPI spec for all tool instances
#[authz_role(Admin, Maintainer, permission = "tool:read")]
#[authn]
pub async fn get_tools_openapi_spec(
    repo: &impl crate::repository::ProviderRepositoryLike,
) -> Result<OpenApi, CommonError> {
    fn get_openapi_path(
        tool_group_instance_id: &String,
        tool_deployment_type_id: &String,
    ) -> String {
        format!(
            "{PATH_PREFIX}/{SERVICE_ROUTE_KEY}/{API_VERSION_1}/provider/{tool_group_instance_id}/function/{tool_deployment_type_id}/invoke"
        )
    }

    // Get all function instances using the internal function (already auth'd at this point)
    let tool_instances = get_tool_instances_internal(repo).await?;

    let mut paths = Paths::new();
    let mut components = Components::builder().schema(
        "Error",
        utoipa::openapi::ObjectBuilder::new()
            .title(Some("Error"))
            .property(
                "message",
                RefOr::T(utoipa::openapi::schema::Schema::Object(
                    ObjectBuilder::new()
                        .schema_type(SchemaType::Type(Type::String))
                        .build(),
                )),
            ),
    );

    for func_metadata in tool_instances {
        let tool_group_instance = &func_metadata.tool_group_instance;
        let tool_instance = &func_metadata.tool_instance;
        let tool_source = &func_metadata.tool_source;

        // Schema names for this function
        let params_schema_name = format!(
            "{}{}Params",
            tool_group_instance.tool_group_deployment_type_id,
            tool_instance.tool_deployment_type_id
        );
        let response_schema_name = format!(
            "{}{}Response",
            tool_group_instance.tool_group_deployment_type_id,
            tool_instance.tool_deployment_type_id
        );
        // Wrapper schema name that matches InvokeToolParamsInner structure
        let wrapper_schema_name = format!("{params_schema_name}Wrapper");

        // Convert params schema: schemars::Schema -> OpenAPI schema
        let params_schema = tool_source.parameters();
        let params_json_schema = params_schema.get_inner().as_value();
        let (params_openapi_json, params_defs) =
            convert_jsonschema_to_openapi(params_json_schema, &params_schema_name)?;
        trace!(schema = %params_schema_name, "Generated params OpenAPI schema");

        // Deserialize JSON Value into utoipa Schema
        let params_utoipa_schema: utoipa::openapi::schema::Schema =
            serde_json::from_value(params_openapi_json).map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to deserialize params schema: {e}"))
            })?;
        components = components.schema(params_schema_name.clone(), params_utoipa_schema);

        // Add extracted definitions to components
        for (def_name, def_json) in params_defs {
            let def_utoipa_schema: utoipa::openapi::schema::Schema =
                serde_json::from_value(def_json).map_err(|e| {
                    CommonError::Unknown(anyhow::anyhow!(
                        "Failed to deserialize def schema {def_name}: {e}"
                    ))
                })?;
            components = components.schema(def_name, def_utoipa_schema);
        }

        // Create wrapper schema that matches InvokeToolParamsInner structure
        // This wraps the params in a "params" field as expected by the API
        let wrapper_schema = utoipa::openapi::ObjectBuilder::new()
            .title(Some(&wrapper_schema_name))
            .property(
                "params",
                RefOr::Ref(Ref::from_schema_name(&params_schema_name)),
            )
            .required("params")
            .build();
        components = components.schema(wrapper_schema_name.clone(), wrapper_schema);

        // Convert response schema: schemars::Schema -> OpenAPI schema
        let response_schema = tool_source.output();
        let response_json_schema = response_schema.get_inner().as_value();
        let (response_openapi_json, response_defs) =
            convert_jsonschema_to_openapi(response_json_schema, &response_schema_name)?;
        trace!(schema = %response_schema_name, "Generated response OpenAPI schema");

        // Deserialize JSON Value into utoipa Schema
        let response_utoipa_schema: utoipa::openapi::schema::Schema =
            serde_json::from_value(response_openapi_json).map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!(
                    "Failed to deserialize response schema: {e}"
                ))
            })?;
        components = components.schema(response_schema_name.clone(), response_utoipa_schema);

        // Add extracted definitions to components
        for (def_name, def_json) in response_defs {
            let def_utoipa_schema: utoipa::openapi::schema::Schema =
                serde_json::from_value(def_json).map_err(|e| {
                    CommonError::Unknown(anyhow::anyhow!(
                        "Failed to deserialize def schema {def_name}: {e}"
                    ))
                })?;
            components = components.schema(def_name, def_utoipa_schema);
        }

        paths.add_path_operation(
            get_openapi_path(
                &tool_instance.tool_group_instance_id,
                &tool_instance.tool_deployment_type_id,
            ),
            vec![HttpMethod::Post],
            Operation::builder()
                .description(Some(format!(
                    "Invoke function {} on provider instance {}",
                    tool_instance.tool_deployment_type_id,
                    tool_instance.tool_group_instance_id
                )))
                .operation_id(Some(format!(
                    "invoke-{}-{}",
                    sanitize_display_name(&tool_group_instance.display_name),
                    tool_instance.tool_deployment_type_id
                )))
                .request_body(Some(
                    RequestBody::builder()
                        .required(Some(Required::True))
                        .content(
                            "application/json",
                            Content::builder()
                                .schema(Some(RefOr::Ref(Ref::from_schema_name(
                                    &wrapper_schema_name,
                                ))))
                                .build(),
                        )
                        .build(),
                ))
                .response(
                    "200",
                    Response::builder()
                        .description("Invoke function")
                        .content(
                            "application/json",
                            Content::builder()
                                .schema(Some(RefOr::Ref(Ref::from_schema_name(
                                    &response_schema_name,
                                ))))
                                .build(),
                        )
                        .build(),
                )
                // TODO: map 500 to actual runtime error response
                .response(
                    "500",
                    Response::builder()
                        .description("Internal Server Error")
                        .content(
                            "application/json",
                            Content::builder()
                                .schema(Some(RefOr::Ref(Ref::from_schema_name("Error"))))
                                .build(),
                        )
                        .build(),
                )
                .build(),
        );
    }

    let openapi_spec = OpenApi::builder()
        .paths(paths)
        .components(Some(components.build()))
        .build();

    Ok(openapi_spec)
}

// TODO: These functions need to be reimplemented to fetch tool definitions from the database
// instead of using hardcoded sources
//
// /// Enriches a function instance with its source metadata
// fn enrich_tool_instance(
//     tool_instance: ToolSerialized,
//     tool_group: &Arc<dyn ToolGroupLike>,
// ) -> Result<ToolListItem, CommonError> {
//     let tool_controller = get_tool_source(
//         tool_group,
//         &tool_instance.tool_deployment_type_id,
//     )?;
//     let tool_source_serialized: ToolDeploymentSerialized =
//         (&tool_controller).into();
//
//     Ok(ToolListItem {
//         tool_instance,
//         controller: tool_source_serialized,
//     })
// }
//
// /// Enriches multiple function instances with their source metadata
// fn enrich_tool_instances(
//     tools: Vec<ToolSerialized>,
//     tool_group: &Arc<dyn ToolGroupLike>,
// ) -> Result<Vec<ToolListItem>, CommonError> {
//     tools
//         .into_iter()
//         .map(|func| enrich_tool_instance(func, tool_group))
//         .collect()
// }

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct CreateToolGroupParamsInner {
    pub resource_server_credential_id: WrappedUuidV4,
    pub user_credential_id: Option<WrappedUuidV4>,
    pub tool_group_id: Option<String>,
    pub display_name: String,
    pub return_on_successful_brokering: Option<ReturnAddress>,
}
pub type CreateToolGroupInstanceParams =
    WithToolGroupDeploymentTypeId<WithCredentialDeploymentTypeId<CreateToolGroupParamsInner>>;
pub type CreateToolGroupInstanceResponse = ToolGroupSerialized;

/// Create a new tool group instance
#[authz_role(Admin, permission = "tool_group:write")]
#[authn]
pub async fn create_tool_group(
    _on_config_change_tx: &OnConfigChangeTx,
    _repo: &impl crate::repository::ProviderRepositoryLike,
    params: CreateToolGroupInstanceParams,
    _publish_on_change_evt: bool,
) -> Result<CreateToolGroupInstanceResponse, CommonError> {
    trace!(
        provider_type = %params.tool_group_deployment_type_id,
        credential_type = %params.inner.credential_deployment_type_id,
        display_name = %params.inner.inner.display_name,
        "Creating provider instance"
    );

    // TODO: Refactor to fetch tool group and credential sources from repository
    // The old implementation used hardcoded in-memory sources which have been removed.
    // This needs to be refactored to:
    // 1. Fetch the tool group source definition from the repository using tool_group_deployment_type_id
    // 2. Get or construct the credential source based on the stored definition
    // 3. Create the tool group instance without requiring in-memory source registry
    Err(CommonError::Unknown(anyhow::anyhow!(
        "Creating tool group instances is temporarily unavailable. \
        Tool group sources need to be registered via API before instances can be created."
    )))
}

pub type UpdateToolGroupInstanceParams = WithToolGroupId<UpdateToolGroupParamsInner>;

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct UpdateToolGroupParamsInner {
    pub display_name: String,
}

pub type UpdateToolGroupInstanceResponse = ();

/// Update an existing tool group instance
#[authz_role(Admin, permission = "tool_group:write")]
#[authn]
pub async fn update_tool_group(
    on_config_change_tx: &OnConfigChangeTx,
    repo: &impl crate::repository::ProviderRepositoryLike,
    params: UpdateToolGroupInstanceParams,
    publish_on_change_evt: bool,
) -> Result<UpdateToolGroupInstanceResponse, CommonError> {
    trace!(
        tool_group_instance_id = %params.tool_group_instance_id,
        display_name = %params.inner.display_name,
        "Updating provider instance"
    );
    repo.update_tool_group(&params.tool_group_instance_id, &params.inner.display_name)
        .await?;

    // Get the updated provider instance with credentials to send config change event
    let tool_group_instance_with_functions = repo
        .get_tool_group_instance_by_id(&params.tool_group_instance_id)
        .await?
        .ok_or_else(|| CommonError::Unknown(anyhow::anyhow!("Provider instance not found")))?;

    let resource_server_cred = repo
        .get_resource_server_credential_by_id(
            &tool_group_instance_with_functions
                .tool_group_instance
                .resource_server_credential_id,
        )
        .await?
        .ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!("Resource server credential not found"))
        })?;

    let user_cred = if let Some(user_credential_id) = &tool_group_instance_with_functions
        .tool_group_instance
        .user_credential_id
    {
        Some(
            repo.get_user_credential_by_id(user_credential_id)
                .await?
                .ok_or_else(|| {
                    CommonError::Unknown(anyhow::anyhow!("User credential not found"))
                })?,
        )
    } else {
        None
    };

    let tool_group_instance_with_creds = ToolGroupSerializedWithCredentials {
        tool_group_instance: tool_group_instance_with_functions.tool_group_instance,
        resource_server_credential: resource_server_cred,
        user_credential: user_cred,
    };

    if publish_on_change_evt {
        on_config_change_tx
            .send(OnConfigChangeEvt::ToolGroupInstanceAdded(
                tool_group_instance_with_creds,
            ))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send config change event: {e}"))
            })?;
    }

    Ok(())
}

pub type DeleteToolGroupInstanceParams = WithToolGroupId<()>;
pub type DeleteToolGroupInstanceResponse = ();

/// Delete a tool group instance
#[authz_role(Admin, permission = "tool_group:write")]
#[authn]
pub async fn delete_tool_group(
    on_config_change_tx: &OnConfigChangeTx,
    repo: &impl crate::repository::ProviderRepositoryLike,
    params: DeleteToolGroupInstanceParams,
    publish_on_change_evt: bool,
) -> Result<DeleteToolGroupInstanceResponse, CommonError> {
    trace!(tool_group_instance_id = %params.tool_group_instance_id, "Deleting tool group instance");
    repo.delete_tool_group(&params.tool_group_instance_id)
        .await?;
    if publish_on_change_evt {
        on_config_change_tx
            .send(OnConfigChangeEvt::ToolGroupInstanceRemoved(
                params.tool_group_instance_id.clone(),
            ))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send config change event: {e}"))
            })?;
    }
    Ok(())
}

// Types for list_tool_group_instances_grouped_by_function
#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct ToolConfig {
    pub tool_source: ToolDeploymentSerialized,
    pub tool_group: ToolGroupDeploymentSerialized,
    pub tool_group_instances: Vec<ToolGroupSerializedWithCredentials>,
}

#[derive(Serialize, Deserialize, Clone, ToSchema, IntoParams)]
pub struct ListToolGroupsGroupedByFunctionParams {
    pub next_page_token: Option<String>,
    pub page_size: i64,
    pub tool_group_deployment_type_id: Option<String>,
    pub function_category: Option<String>,
}
pub type ListToolGroupInstancesGroupedByFunctionResponse = PaginatedResponse<ToolConfig>;

/// List tool group instances grouped by tool type
#[authz_role(Admin, Maintainer, permission = "tool_group:read")]
#[authn]
pub async fn list_tool_groups_grouped_by_function(
    repo: &impl crate::repository::ProviderRepositoryLike,
    params: ListToolGroupsGroupedByFunctionParams,
) -> Result<ListToolGroupInstancesGroupedByFunctionResponse, CommonError> {
    trace!(
        page_size = params.page_size,
        provider_type = ?params.tool_group_deployment_type_id,
        function_category = ?params.function_category,
        "Listing provider instances grouped by function"
    );

    // TODO: Refactor to fetch tool group sources from repository instead of hardcoded sources
    // The old implementation used list_all_tool_group_sources() which has been removed.
    // Tool group definitions should be stored in the database and fetched from there.
    // For now, return empty result.
    //
    // Required changes:
    // 1. Store tool group and tool source metadata in database
    // 2. Fetch controllers from repository using tool_group_deployment_type_id
    // 3. Build the response from stored metadata instead of trait implementations

    Ok(PaginatedResponse {
        items: vec![],
        next_page_token: None,
    })
}

pub type GetToolGroupInstanceParams = WithToolGroupId<()>;
pub type GetToolGroupInstanceResponse = ToolGroupSerializedWithEverything;

/// Get a tool group instance by ID
#[authz_role(Admin, Maintainer, permission = "tool_group:read")]
#[authn]
pub async fn get_tool_group(
    _repo: &impl crate::repository::ProviderRepositoryLike,
    params: GetToolGroupInstanceParams,
) -> Result<GetToolGroupInstanceResponse, CommonError> {
    trace!(tool_group_instance_id = %params.tool_group_instance_id, "Getting provider instance");

    // TODO: Refactor to fetch tool group sources from repository instead of hardcoded sources
    // The old implementation used hardcoded in-memory sources to enrich the response with source metadata.
    // This needs to be refactored to:
    // 1. Fetch tool group source definitions from the repository
    // 2. Enrich the response with source metadata from the database
    // 3. Support getting tool group instance without requiring in-memory source registry
    Err(CommonError::Unknown(anyhow::anyhow!(
        "Getting tool group instance is temporarily unavailable. \
        Tool group sources need to be registered via API before instances can be retrieved."
    )))
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct WithToolGroupId<T> {
    pub tool_group_id: String,
    pub inner: T,
}

#[derive(Serialize, Deserialize, Clone, ToSchema, JsonSchema)]
pub struct EnableToolParamsInner {}
pub type EnableToolParams =
    WithToolGroupId<WithToolDeploymentTypeId<EnableToolParamsInner>>;
pub type EnableToolResponse = ToolSerialized;

/// Enable a tool on a tool group instance
#[authz_role(Admin, permission = "tool:write")]
#[authn]
pub async fn enable_tool(
    _on_config_change_tx: &OnConfigChangeTx,
    _repo: &impl crate::repository::ProviderRepositoryLike,
    params: EnableToolParams,
    _publish_on_change_evt: bool,
) -> Result<EnableToolResponse, CommonError> {
    trace!(
        tool_group_instance_id = %params.tool_group_instance_id,
        function_type = %params.inner.tool_deployment_type_id,
        "Enabling function"
    );

    // TODO: Refactor to fetch tool group and tool sources from repository
    // The old implementation used hardcoded in-memory sources which have been removed.
    // This needs to be refactored to:
    // 1. Fetch tool group source definition from the repository
    // 2. Verify tool controller exists for this tool group without in-memory registry
    // 3. Enable the tool by storing it in the database
    Err(CommonError::Unknown(anyhow::anyhow!(
        "Enabling tools is temporarily unavailable. \
        Tool group sources need to be registered via API before tools can be enabled."
    )))
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct InvokeToolParamsInner {
    pub params: WrappedJsonValue,
}
pub type InvokeToolParams =
    WithToolGroupId<WithToolId<InvokeToolParamsInner>>;
pub type InvokeToolResponse = InvokeResult;

/// Invoke a tool on a tool group instance
#[authz_role(Admin, Maintainer, Agent, User, permission = "tool:invoke")]
#[authn]
pub async fn invoke_tool(
    repo: &crate::repository::Repository,
    encryption_service: &CryptoCache,
    params: InvokeToolParams,
) -> Result<InvokeToolResponse, CommonError> {
    invoke_tool_internal(repo, encryption_service, params).await
}

/// Internal function to invoke a function (no auth check).
/// Used by `invoke_tool` and internal helpers like MCP server.
pub(crate) async fn invoke_tool_internal(
    repo: &crate::repository::Repository,
    encryption_service: &CryptoCache,
    params: InvokeToolParams,
) -> Result<InvokeToolResponse, CommonError> {
    trace!(
        tool_group_instance_id = %params.tool_group_instance_id,
        function_type = %params.inner.tool_deployment_type_id,
        "Invoking function"
    );

    // First get the tool group instance to find the tool_group_deployment_type_id
    let tool_group_instance = repo
        .get_tool_group_instance_by_id(&params.tool_group_instance_id)
        .await?
        .ok_or(CommonError::Unknown(anyhow::anyhow!(
            "Tool group instance not found"
        )))?;

    // Get tool instance with credentials to retrieve all necessary information
    let tool_instance_with_credentials = repo
        .get_tool_instance_with_credentials(
            &params.inner.tool_deployment_type_id,
            &tool_group_instance.tool_group_instance.tool_group_deployment_type_id,
            &params.tool_group_instance_id,
        )
        .await?
        .ok_or(CommonError::Unknown(anyhow::anyhow!(
            "Tool instance not found"
        )))?;

    // Get the tool definition - try by alias "latest" first, then fall back to exact type_id
    let tool = match repo.get_tool_by_alias("latest").await? {
        Some(tool) if tool.type_id == params.inner.tool_deployment_type_id => tool,
        _ => {
            // Fall back to getting tool by exact type_id with empty deployment_id as default
            repo.get_tool_by_id(&params.inner.tool_deployment_type_id, "default")
                .await?
                .ok_or(CommonError::Unknown(anyhow::anyhow!(
                    "Tool definition not found for type_id: {}",
                    params.inner.tool_deployment_type_id
                )))?
        }
    };

    // Get decryption service using the resource server credential's DEK alias
    let decryption_service = encryption_service
        .get_decryption_service(&tool_instance_with_credentials.resource_server_credential.dek_alias)
        .await?;

    // Decrypt credentials
    // The credential value is stored as encrypted JSON string
    let resource_server_cred_encrypted_string = tool_instance_with_credentials
        .resource_server_credential
        .value
        .get_inner()
        .as_str()
        .ok_or_else(|| CommonError::Unknown(anyhow::anyhow!("Resource server credential value is not a string")))?
        .to_string();

    let resource_server_cred_decrypted = decryption_service
        .decrypt_data(encryption::logic::EncryptedString(resource_server_cred_encrypted_string))
        .await?;

    // Parse decrypted JSON credential
    let resource_server_cred_json: serde_json::Value = serde_json::from_str(&resource_server_cred_decrypted)?;

    // User credential (if present)
    let user_cred_json = if let Some(ref user_cred) = tool_instance_with_credentials.user_credential {
        let user_cred_encrypted_string = user_cred
            .value
            .get_inner()
            .as_str()
            .ok_or_else(|| CommonError::Unknown(anyhow::anyhow!("User credential value is not a string")))?
            .to_string();

        let user_cred_decrypted = decryption_service
            .decrypt_data(encryption::logic::EncryptedString(user_cred_encrypted_string))
            .await?;

        Some(serde_json::from_str::<serde_json::Value>(&user_cred_decrypted)?)
    } else {
        None
    };

    // Static credentials are stored in the tool definition metadata (if any)
    let static_credentials = tool.metadata.0.get("static_credentials")
        .map(|v| WrappedJsonValue::new(v.clone()));

    // Invoke based on endpoint type
    match tool.endpoint_type {
        crate::logic::EndpointType::Http => {
            crate::logic::invoke_http_tool(
                &decryption_service,
                &tool,
                static_credentials,
                Some(WrappedJsonValue::new(resource_server_cred_json)),
                user_cred_json.map(WrappedJsonValue::new),
                params.inner.inner.params,
            )
            .await
        }
    }
}

#[derive(Serialize, Deserialize, Clone, ToSchema, JsonSchema)]
pub struct DisableToolParamsInner {}
pub type DisableToolParams =
    WithToolGroupId<WithToolDeploymentTypeId<DisableToolParamsInner>>;
pub type DisableToolResponse = ();

/// Disable a tool on a tool group instance
#[authz_role(Admin, permission = "tool:write")]
#[authn]
pub async fn disable_tool(
    on_config_change_tx: &OnConfigChangeTx,
    repo: &impl crate::repository::ProviderRepositoryLike,
    params: DisableToolParams,
    publish_on_change_evt: bool,
) -> Result<DisableToolResponse, CommonError> {
    trace!(
        tool_group_instance_id = %params.tool_group_instance_id,
        function_type = %params.inner.tool_deployment_type_id,
        "Disabling function"
    );
    // Get provider instance to retrieve tool_group_deployment_type_id
    let tool_group_instance = repo
        .get_tool_group_instance_by_id(&params.tool_group_instance_id)
        .await?
        .ok_or(CommonError::Unknown(anyhow::anyhow!(
            "Provider instance not found"
        )))?;

    // Delete from database
    repo.delete_tool(
        &params.inner.tool_deployment_type_id,
        &tool_group_instance
            .tool_group_instance
            .tool_group_deployment_type_id,
        &params.tool_group_instance_id,
    )
    .await?;

    if publish_on_change_evt {
        on_config_change_tx
            .send(OnConfigChangeEvt::ToolInstanceRemoved(
                params.inner.tool_deployment_type_id.clone(),
                tool_group_instance
                    .tool_group_instance
                    .tool_group_deployment_type_id
                    .clone(),
                params.tool_group_instance_id.clone(),
            ))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send config change event: {e}"))
            })?;
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct WithToolId<T> {
    pub tool_deployment_type_id: String,
    pub inner: T,
}

#[cfg(test)]
mod tests {
    mod unit {
        use super::super::*;
        use shared::primitives::{PaginationRequest, SqlMigrationLoader};

        #[tokio::test]
        async fn test_list_tool_group_instances_empty() {
            shared::setup_test!();

            let (_db, conn) = shared::test_utils::repository::setup_in_memory_database(vec![
                crate::repository::Repository::load_sql_migrations(),
            ])
            .await
            .unwrap();
            let repo = crate::repository::Repository::new(conn);

            let pagination = PaginationRequest {
                page_size: 10,
                next_page_token: None,
            };

            let result = list_tool_group_instances_internal(
                &repo,
                ListProviderInstancesParams {
                    pagination,
                    status: None,
                    tool_group_deployment_type_id: None,
                },
            )
            .await;
            assert!(result.is_ok());

            let response = result.unwrap();
            assert_eq!(response.items.len(), 0);
            assert!(response.next_page_token.is_none());
        }

        #[tokio::test]
        async fn test_list_tool_instances_empty() {
            shared::setup_test!();

            let repo = {
                let (_db, conn) = shared::test_utils::repository::setup_in_memory_database(vec![
                    crate::repository::Repository::load_sql_migrations(),
                ])
                .await
                .unwrap();
                crate::repository::Repository::new(conn)
            };

            let pagination = PaginationRequest {
                page_size: 10,
                next_page_token: None,
            };

            let result = list_tool_instances_internal(
                &repo,
                ListToolsParams {
                    pagination,
                    tool_group_instance_id: None,
                },
            )
            .await;
            assert!(result.is_ok());

            let response = result.unwrap();
            assert_eq!(response.items.len(), 0);
            assert!(response.next_page_token.is_none());
        }

        #[test]
        fn test_sanitize_display_name() {
            // Test basic alphanumeric and dash characters
            assert_eq!(sanitize_display_name("my-provider-123"), "my-provider-123");

            // Test whitespace replacement
            assert_eq!(sanitize_display_name("my provider"), "my-provider");
            assert_eq!(sanitize_display_name("my  provider"), "my--provider");

            // Test special character removal
            assert_eq!(sanitize_display_name("my@provider!"), "myprovider");
            assert_eq!(sanitize_display_name("provider#1"), "provider1");

            // Test mixed characters
            assert_eq!(sanitize_display_name("My Provider #1!"), "My-Provider-1");
            assert_eq!(
                sanitize_display_name("provider_name@2024"),
                "providername2024"
            );

            // Test edge cases
            assert_eq!(sanitize_display_name(""), "");
            assert_eq!(sanitize_display_name("---"), "---");
            assert_eq!(sanitize_display_name("ABC123"), "ABC123");
        }

        #[test]
        fn test_convert_jsonschema_to_openapi() {
            // Test simple schema without $defs
            let simple_schema = serde_json::json!({
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "type": "object",
                "title": "SimpleSchema",
                "properties": {
                    "name": {"type": "string"}
                }
            });

            let (converted, defs) = convert_jsonschema_to_openapi(&simple_schema, "Test").unwrap();
            assert!(defs.is_empty());
            assert_eq!(converted.get("$schema"), None); // Should be removed
            assert_eq!(converted.get("title"), None); // Should be removed
            assert_eq!(
                converted.get("type").and_then(|v| v.as_str()),
                Some("object")
            );

            // Test schema with $defs
            let schema_with_defs = serde_json::json!({
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "type": "object",
                "$defs": {
                    "Person": {
                        "type": "object",
                        "properties": {
                            "name": {"type": "string"}
                        }
                    }
                },
                "properties": {
                    "person": {"$ref": "#/$defs/Person"}
                }
            });

            let (converted, defs) =
                convert_jsonschema_to_openapi(&schema_with_defs, "Test").unwrap();
            assert_eq!(defs.len(), 1);
            assert_eq!(defs[0].0, "Test_Person");

            // Check that $ref was updated in main schema
            let person_ref = converted
                .get("properties")
                .and_then(|p| p.get("person"))
                .and_then(|p| p.get("$ref"))
                .and_then(|r| r.as_str());
            assert_eq!(person_ref, Some("#/components/schemas/Test_Person"));

            // Test schema with nested $defs (definitions referencing other definitions)
            let schema_with_nested_defs = serde_json::json!({
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "type": "object",
                "$defs": {
                    "Address": {
                        "type": "object",
                        "properties": {
                            "street": {"type": "string"}
                        }
                    },
                    "Person": {
                        "type": "object",
                        "properties": {
                            "name": {"type": "string"},
                            "address": {"$ref": "#/$defs/Address"}
                        }
                    }
                },
                "properties": {
                    "person": {"$ref": "#/$defs/Person"}
                }
            });

            let (_converted, defs) =
                convert_jsonschema_to_openapi(&schema_with_nested_defs, "Nested").unwrap();
            assert_eq!(defs.len(), 2);

            // Find the Person definition
            let person_def = defs
                .iter()
                .find(|(name, _)| name == "Nested_Person")
                .unwrap();

            // Check that the nested reference in Person was updated
            let address_ref = person_def
                .1
                .get("properties")
                .and_then(|p| p.get("address"))
                .and_then(|a| a.get("$ref"))
                .and_then(|r| r.as_str());
            assert_eq!(address_ref, Some("#/components/schemas/Nested_Address"));
        }
    }
}
