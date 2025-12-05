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
use tracing::info;
use utoipa::{
    IntoParams, ToSchema,
    openapi::{
        Components, Content, HttpMethod, ObjectBuilder, OpenApi, Paths, Ref, RefOr, Required,
        Response, Type, path::Operation, request_body::RequestBody, schema::SchemaType,
    },
};

use crate::{
    logic::{
        FunctionControllerLike, InvokeResult, OnConfigChangeEvt, OnConfigChangeTx,
        ProviderControllerLike,
        controller::{
            FunctionControllerSerialized, PROVIDER_REGISTRY, ProviderControllerSerialized,
            ProviderCredentialControllerSerialized, WithCredentialControllerTypeId,
            WithFunctionControllerTypeId, WithProviderControllerTypeId, get_credential_controller,
            get_function_controller, get_provider_controller,
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
pub struct ProviderInstanceSerialized {
    // not UUID as some ID's will be deterministic
    pub id: String,
    pub display_name: String,
    pub resource_server_credential_id: WrappedUuidV4,
    pub user_credential_id: Option<WrappedUuidV4>,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
    pub provider_controller_type_id: String,
    pub credential_controller_type_id: String,
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
pub struct ProviderInstanceSerializedWithFunctions {
    pub provider_instance: ProviderInstanceSerialized,
    pub functions: Vec<FunctionInstanceSerialized>,
    pub resource_server_credential: ResourceServerCredentialSerialized,
    pub user_credential: Option<UserCredentialSerialized>,
}

// Repository layer struct - includes credentials without functions
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct ProviderInstanceSerializedWithCredentials {
    pub provider_instance: ProviderInstanceSerialized,
    pub resource_server_credential: ResourceServerCredentialSerialized,
    pub user_credential: Option<UserCredentialSerialized>,
}

// List response struct - enriched with controller metadata
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct ProviderInstanceListItem {
    #[serde(flatten)]
    pub provider_instance: ProviderInstanceSerialized,
    pub functions: Vec<FunctionInstanceListItem>,
    pub controller: ProviderControllerSerialized,
    pub credential_controller: ProviderCredentialControllerSerialized,
}

// List response struct for function instances
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct FunctionInstanceListItem {
    #[serde(flatten)]
    pub function_instance: FunctionInstanceSerialized,
    pub controller: FunctionControllerSerialized,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct ProviderInstanceSerializedWithEverything {
    #[serde(flatten)]
    pub instance_data: ProviderInstanceSerializedWithCredentials,
    pub functions: Vec<FunctionInstanceListItem>,
    pub controller: ProviderControllerSerialized,
    pub credential_controller: ProviderCredentialControllerSerialized,
}

// we shouldn't need this besides the fact that we want to keep track of functions intentionally enabled
// by users. if all functions were enabled, always, we could drop this struct.
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct FunctionInstanceSerialized {
    pub function_controller_type_id: String,
    pub provider_controller_type_id: String,
    pub provider_instance_id: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct FunctionInstanceSerializedWithEverything {
    #[serde(flatten)]
    pub function_instance: FunctionInstanceSerializedWithCredentials,
    pub controller: FunctionControllerSerialized,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct FunctionInstanceSerializedWithCredentials {
    pub function_instance: FunctionInstanceSerialized,
    pub provider_instance: ProviderInstanceSerialized,
    pub resource_server_credential: ResourceServerCredentialSerialized,
    pub user_credential: UserCredentialSerialized,
}

#[derive(Debug, Clone)]
pub struct ListProviderInstancesParams {
    pub pagination: PaginationRequest,
    pub status: Option<String>,
    pub provider_controller_type_id: Option<String>,
}

pub type ListProviderInstancesResponse = PaginatedResponse<ProviderInstanceListItem>;

pub async fn list_provider_instances(
    repo: &impl crate::repository::ProviderRepositoryLike,
    params: ListProviderInstancesParams,
) -> Result<ListProviderInstancesResponse, CommonError> {
    let provider_instances_with_data = repo
        .list_provider_instances(
            &params.pagination,
            params.status.as_deref(),
            params.provider_controller_type_id.as_deref(),
        )
        .await?;

    // Enrich with PROVIDER_REGISTRY data
    let enriched_items: Result<Vec<ProviderInstanceListItem>, CommonError> =
        provider_instances_with_data
            .items
            .into_iter()
            .map(|pwf| {
                // Get provider controller from registry
                let provider_controller =
                    get_provider_controller(&pwf.provider_instance.provider_controller_type_id)?;
                let controller_serialized: ProviderControllerSerialized =
                    provider_controller.as_ref().into();

                // Get credential controller from provider
                let credential_controller = get_credential_controller(
                    &provider_controller,
                    &pwf.provider_instance.credential_controller_type_id,
                )?;
                let credential_controller_serialized: ProviderCredentialControllerSerialized =
                    (&credential_controller).into();

                // Enrich functions with their controllers using reusable helper
                let enriched_functions =
                    enrich_function_instances(pwf.functions, &provider_controller)?;

                Ok(ProviderInstanceListItem {
                    provider_instance: pwf.provider_instance,
                    functions: enriched_functions,
                    controller: controller_serialized,
                    credential_controller: credential_controller_serialized,
                })
            })
            .collect();

    Ok(PaginatedResponse {
        items: enriched_items?,
        next_page_token: provider_instances_with_data.next_page_token,
    })
}

#[derive(Debug, Clone)]
pub struct ListFunctionInstancesParams {
    pub pagination: PaginationRequest,
    pub provider_instance_id: Option<String>,
}

pub type ListFunctionInstancesResponse = PaginatedResponse<FunctionInstanceSerialized>;

pub async fn list_function_instances(
    repo: &impl crate::repository::ProviderRepositoryLike,
    params: ListFunctionInstancesParams,
) -> Result<ListFunctionInstancesResponse, CommonError> {
    let function_instances = repo
        .list_function_instances(&params.pagination, params.provider_instance_id.as_deref())
        .await?;
    Ok(function_instances)
}

/// Represents a function instance with all associated metadata needed for code generation
#[derive(Clone)]
pub struct FunctionInstanceWithMetadata {
    pub provider_instance: ProviderInstanceSerialized,
    pub function_instance: FunctionInstanceSerialized,
    pub provider_controller: Arc<dyn ProviderControllerLike>,
    pub function_controller: Arc<dyn FunctionControllerLike>,
}

/// Returns all function instances with their associated controllers and metadata.
/// This is the core data structure that can be used for client code generation.
pub async fn get_function_instances(
    repo: &impl crate::repository::ProviderRepositoryLike,
) -> Result<Vec<FunctionInstanceWithMetadata>, CommonError> {
    let mut result = Vec::new();
    let mut pagination = PaginationRequest {
        page_size: 1000,
        next_page_token: None,
    };

    loop {
        let repo_resp = repo
            .list_provider_instances(&pagination, None, None)
            .await?;
        for provider_instance in repo_resp.items {
            // Try to get provider controller, skip if not found
            let provider_controller = match get_provider_controller(
                &provider_instance
                    .provider_instance
                    .provider_controller_type_id,
            ) {
                Ok(controller) => controller,
                Err(e) => {
                    tracing::warn!(
                        "Skipping provider instance '{}' (type: {}): {}",
                        provider_instance.provider_instance.id,
                        provider_instance
                            .provider_instance
                            .provider_controller_type_id,
                        e
                    );
                    continue;
                }
            };

            for function_instance in provider_instance.functions {
                // Try to get function controller, skip if not found
                let function_controller = match get_function_controller(
                    &provider_controller,
                    &function_instance.function_controller_type_id,
                ) {
                    Ok(controller) => controller,
                    Err(e) => {
                        tracing::warn!(
                            "Skipping function '{}' for provider '{}': {}",
                            function_instance.function_controller_type_id,
                            provider_instance
                                .provider_instance
                                .provider_controller_type_id,
                            e
                        );
                        continue;
                    }
                };

                result.push(FunctionInstanceWithMetadata {
                    provider_instance: provider_instance.provider_instance.clone(),
                    function_instance,
                    provider_controller: provider_controller.clone(),
                    function_controller,
                });
            }
        }
        match repo_resp.next_page_token {
            Some(next_page_token) => {
                pagination.next_page_token = Some(next_page_token);
            }
            None => {
                break;
            }
        }
    }

    Ok(result)
}

pub async fn get_function_instances_openapi_spec(
    repo: &impl crate::repository::ProviderRepositoryLike,
) -> Result<OpenApi, CommonError> {
    fn get_openapi_path(
        provider_instance_id: &String,
        function_controller_type_id: &String,
    ) -> String {
        format!(
            "{PATH_PREFIX}/{SERVICE_ROUTE_KEY}/{API_VERSION_1}/provider/{provider_instance_id}/function/{function_controller_type_id}/invoke"
        )
    }

    // Get all function instances using the new core function
    let function_instances = get_function_instances(repo).await?;

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

    for func_metadata in function_instances {
        let provider_instance = &func_metadata.provider_instance;
        let function_instance = &func_metadata.function_instance;
        let function_controller = &func_metadata.function_controller;

        // Schema names for this function
        let params_schema_name = format!(
            "{}{}Params",
            provider_instance.provider_controller_type_id,
            function_instance.function_controller_type_id
        );
        let response_schema_name = format!(
            "{}{}Response",
            provider_instance.provider_controller_type_id,
            function_instance.function_controller_type_id
        );
        // Wrapper schema name that matches InvokeFunctionParamsInner structure
        let wrapper_schema_name = format!("{params_schema_name}Wrapper");

        // Convert params schema: schemars::Schema -> OpenAPI schema
        let params_schema = function_controller.parameters();
        let params_json_schema = params_schema.get_inner().as_value();
        let (params_openapi_json, params_defs) =
            convert_jsonschema_to_openapi(params_json_schema, &params_schema_name)?;
        info!(
            "Params OpenAPI schema for {}: {}",
            params_schema_name, params_openapi_json
        );

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

        // Create wrapper schema that matches InvokeFunctionParamsInner structure
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
        let response_schema = function_controller.output();
        let response_json_schema = response_schema.get_inner().as_value();
        let (response_openapi_json, response_defs) =
            convert_jsonschema_to_openapi(response_json_schema, &response_schema_name)?;
        info!(
            "Response OpenAPI schema for {}: {}",
            response_schema_name, response_openapi_json
        );

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
                &function_instance.provider_instance_id,
                &function_instance.function_controller_type_id,
            ),
            vec![HttpMethod::Post],
            Operation::builder()
                .description(Some(format!(
                    "Invoke function {} on provider instance {}",
                    function_instance.function_controller_type_id,
                    function_instance.provider_instance_id
                )))
                .operation_id(Some(format!(
                    "invoke-{}-{}",
                    sanitize_display_name(&provider_instance.display_name),
                    function_instance.function_controller_type_id
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

/// Enriches a function instance with its controller metadata
fn enrich_function_instance(
    function_instance: FunctionInstanceSerialized,
    provider_controller: &Arc<dyn ProviderControllerLike>,
) -> Result<FunctionInstanceListItem, CommonError> {
    let function_controller = get_function_controller(
        provider_controller,
        &function_instance.function_controller_type_id,
    )?;
    let function_controller_serialized: FunctionControllerSerialized =
        (&function_controller).into();

    Ok(FunctionInstanceListItem {
        function_instance,
        controller: function_controller_serialized,
    })
}

/// Enriches multiple function instances with their controller metadata
fn enrich_function_instances(
    functions: Vec<FunctionInstanceSerialized>,
    provider_controller: &Arc<dyn ProviderControllerLike>,
) -> Result<Vec<FunctionInstanceListItem>, CommonError> {
    functions
        .into_iter()
        .map(|func| enrich_function_instance(func, provider_controller))
        .collect()
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct CreateProviderInstanceParamsInner {
    pub resource_server_credential_id: WrappedUuidV4,
    pub user_credential_id: Option<WrappedUuidV4>,
    pub provider_instance_id: Option<String>,
    pub display_name: String,
    pub return_on_successful_brokering: Option<ReturnAddress>,
}
pub type CreateProviderInstanceParams =
    WithProviderControllerTypeId<WithCredentialControllerTypeId<CreateProviderInstanceParamsInner>>;
pub type CreateProviderInstanceResponse = ProviderInstanceSerialized;

pub async fn create_provider_instance(
    on_config_change_tx: &OnConfigChangeTx,
    repo: &impl crate::repository::ProviderRepositoryLike,
    params: CreateProviderInstanceParams,
    publish_on_change_evt: bool,
) -> Result<CreateProviderInstanceResponse, CommonError> {
    let provider_controller = get_provider_controller(&params.provider_controller_type_id)?;

    let _credential_controller = get_credential_controller(
        &provider_controller,
        &params.inner.credential_controller_type_id,
    )?;

    // Verify resource server credential exists
    let resource_server_credential = repo
        .get_resource_server_credential_by_id(&params.inner.inner.resource_server_credential_id)
        .await?
        .ok_or(CommonError::Unknown(anyhow::anyhow!(
            "Resource server credential not found"
        )))?;

    // Verify user credential exists if provided
    let user_credential = if let Some(user_credential_id) = &params.inner.inner.user_credential_id {
        Some(
            repo.get_user_credential_by_id(user_credential_id)
                .await?
                .ok_or(CommonError::Unknown(anyhow::anyhow!(
                    "User credential not found"
                )))?,
        )
    } else {
        None
    };

    let provider_instance_id = match params.inner.inner.provider_instance_id {
        Some(provider_instance_id) => provider_instance_id,
        None => uuid::Uuid::new_v4().to_string(),
    };

    // Determine status based on whether user_credential_id is set
    let status = if params.inner.inner.user_credential_id.is_some() {
        "active".to_string()
    } else {
        "brokering_initiated".to_string()
    };

    let now = WrappedChronoDateTime::now();
    let provider_instance_serialized = ProviderInstanceSerialized {
        id: provider_instance_id,
        display_name: params.inner.inner.display_name,
        resource_server_credential_id: params.inner.inner.resource_server_credential_id,
        user_credential_id: params.inner.inner.user_credential_id,
        created_at: now,
        updated_at: now,
        provider_controller_type_id: params.provider_controller_type_id,
        credential_controller_type_id: params.inner.credential_controller_type_id,
        status,
        return_on_successful_brokering: params.inner.inner.return_on_successful_brokering.clone(),
    };

    // Save to database
    repo.create_provider_instance(&crate::repository::CreateProviderInstance::from(
        provider_instance_serialized.clone(),
    ))
    .await?;

    let provider_instance_with_credentials = ProviderInstanceSerializedWithCredentials {
        provider_instance: provider_instance_serialized.clone(),
        resource_server_credential: resource_server_credential.clone(),
        user_credential: user_credential.clone(),
    };
    if publish_on_change_evt {
        on_config_change_tx
            .send(OnConfigChangeEvt::ProviderInstanceAdded(
                provider_instance_with_credentials,
            ))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send config change event: {e}"))
            })?;
    }

    Ok(provider_instance_serialized)
}

pub type UpdateProviderInstanceParams = WithProviderInstanceId<UpdateProviderInstanceParamsInner>;

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct UpdateProviderInstanceParamsInner {
    pub display_name: String,
}

pub type UpdateProviderInstanceResponse = ();

pub async fn update_provider_instance(
    on_config_change_tx: &OnConfigChangeTx,
    repo: &impl crate::repository::ProviderRepositoryLike,
    params: UpdateProviderInstanceParams,
    publish_on_change_evt: bool,
) -> Result<UpdateProviderInstanceResponse, CommonError> {
    repo.update_provider_instance(&params.provider_instance_id, &params.inner.display_name)
        .await?;

    // Get the updated provider instance with credentials to send config change event
    let provider_instance_with_functions = repo
        .get_provider_instance_by_id(&params.provider_instance_id)
        .await?
        .ok_or_else(|| CommonError::Unknown(anyhow::anyhow!("Provider instance not found")))?;

    let resource_server_cred = repo
        .get_resource_server_credential_by_id(
            &provider_instance_with_functions
                .provider_instance
                .resource_server_credential_id,
        )
        .await?
        .ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!("Resource server credential not found"))
        })?;

    let user_cred = if let Some(user_credential_id) = &provider_instance_with_functions
        .provider_instance
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

    let provider_instance_with_creds = ProviderInstanceSerializedWithCredentials {
        provider_instance: provider_instance_with_functions.provider_instance,
        resource_server_credential: resource_server_cred,
        user_credential: user_cred,
    };

    if publish_on_change_evt {
        on_config_change_tx
            .send(OnConfigChangeEvt::ProviderInstanceAdded(
                provider_instance_with_creds,
            ))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send config change event: {e}"))
            })?;
    }

    Ok(())
}

pub type DeleteProviderInstanceParams = WithProviderInstanceId<()>;
pub type DeleteProviderInstanceResponse = ();

pub async fn delete_provider_instance(
    on_config_change_tx: &OnConfigChangeTx,
    repo: &impl crate::repository::ProviderRepositoryLike,
    params: DeleteProviderInstanceParams,
    publish_on_change_evt: bool,
) -> Result<DeleteProviderInstanceResponse, CommonError> {
    repo.delete_provider_instance(&params.provider_instance_id)
        .await?;
    if publish_on_change_evt {
        on_config_change_tx
            .send(OnConfigChangeEvt::ProviderInstanceRemoved(
                params.provider_instance_id.clone(),
            ))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send config change event: {e}"))
            })?;
    }
    Ok(())
}

// Types for list_provider_instances_grouped_by_function
#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct FunctionInstanceConfig {
    pub function_controller: FunctionControllerSerialized,
    pub provider_controller: ProviderControllerSerialized,
    pub provider_instances: Vec<ProviderInstanceSerializedWithCredentials>,
}

#[derive(Serialize, Deserialize, Clone, ToSchema, IntoParams)]
pub struct ListProviderInstancesGroupedByFunctionParams {
    pub next_page_token: Option<String>,
    pub page_size: i64,
    pub provider_controller_type_id: Option<String>,
    pub function_category: Option<String>,
}
pub type ListProviderInstancesGroupedByFunctionResponse = PaginatedResponse<FunctionInstanceConfig>;

pub async fn list_provider_instances_grouped_by_function(
    repo: &impl crate::repository::ProviderRepositoryLike,
    params: ListProviderInstancesGroupedByFunctionParams,
) -> Result<ListProviderInstancesGroupedByFunctionResponse, CommonError> {
    // Get all providers from registry
    let providers = PROVIDER_REGISTRY
        .read()
        .map_err(|_e| CommonError::Unknown(anyhow::anyhow!("Poison error")))?
        .clone();

    // Create a vector of (provider_controller, function_controller) tuples, sorted by display names
    let mut function_configs: Vec<(
        Arc<dyn ProviderControllerLike>,
        Arc<dyn FunctionControllerLike>,
    )> = Vec::new();

    for provider in providers.iter() {
        // Apply provider_controller_type_id filter if provided
        if let Some(ref filter_provider_type) = params.provider_controller_type_id {
            if provider.type_id() != *filter_provider_type {
                continue;
            }
        }

        for function in provider.functions() {
            // Apply function_category filter if provided
            if let Some(ref filter_category) = params.function_category {
                if !function.categories().contains(filter_category) {
                    continue;
                }
            }

            function_configs.push((provider.clone(), function));
        }
    }

    // Sort by provider name (ascending), then by function name (ascending)
    function_configs.sort_by(|a, b| {
        let provider_cmp = a.0.name().cmp(&b.0.name());
        if provider_cmp == std::cmp::Ordering::Equal {
            a.1.name().cmp(&b.1.name())
        } else {
            provider_cmp
        }
    });

    // Apply pagination - decode the next_page_token as an offset
    let offset = if let Some(token) = &params.next_page_token {
        let decoded_parts = shared::primitives::decode_pagination_token(token).map_err(|e| {
            CommonError::Repository {
                msg: format!("Invalid pagination token: {e}"),
                source: Some(e.into()),
            }
        })?;
        if decoded_parts.is_empty() {
            0
        } else {
            decoded_parts[0]
                .parse::<usize>()
                .map_err(|e| CommonError::Repository {
                    msg: format!("Invalid offset in pagination token: {e}"),
                    source: Some(e.into()),
                })?
        }
    } else {
        0
    };

    // Get the paginated slice
    let page_size = params.page_size as usize;
    let end_offset = std::cmp::min(offset + page_size, function_configs.len());
    let paginated_configs = &function_configs[offset..end_offset];

    // Extract function_controller_type_ids from the paginated slice
    let function_controller_type_ids: Vec<String> = paginated_configs
        .iter()
        .map(|(_, function)| function.type_id().to_string())
        .collect();

    // If no functions match the filter criteria, return empty result immediately
    // This avoids passing an empty array to the SQL IN clause which would be invalid SQL
    let grouped_results = if function_controller_type_ids.is_empty() {
        Vec::new()
    } else {
        // Call the repository method to get provider instances grouped by function controller type id
        repo.get_provider_instances_grouped_by_function_controller_type_id(
            &function_controller_type_ids,
        )
        .await?
    };

    // Create a HashMap for quick lookup of provider instances by function_controller_type_id
    let provider_instances_map: std::collections::HashMap<
        String,
        Vec<ProviderInstanceSerializedWithCredentials>,
    > = grouped_results
        .into_iter()
        .map(|group| (group.function_controller_type_id, group.provider_instances))
        .collect();

    // Construct the final result maintaining the sorted order from paginated_configs
    let items: Vec<FunctionInstanceConfig> = paginated_configs
        .iter()
        .map(|(provider, function)| {
            let provider_instances = provider_instances_map
                .get(&function.type_id())
                .cloned()
                .unwrap_or_else(Vec::new);

            FunctionInstanceConfig {
                function_controller: function.into(),
                provider_controller: provider.as_ref().into(),
                provider_instances,
            }
        })
        .collect();

    // Calculate next_page_token
    let next_page_token = if end_offset < function_configs.len() {
        let token_value = end_offset.to_string();
        Some(base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            token_value.as_bytes(),
        ))
    } else {
        None
    };

    Ok(PaginatedResponse {
        items,
        next_page_token,
    })
}

pub type GetProviderInstanceParams = WithProviderInstanceId<()>;
pub type GetProviderInstanceResponse = ProviderInstanceSerializedWithEverything;

pub async fn get_provider_instance(
    repo: &impl crate::repository::ProviderRepositoryLike,
    params: GetProviderInstanceParams,
) -> Result<GetProviderInstanceResponse, CommonError> {
    let provider_instance = repo
        .get_provider_instance_by_id(&params.provider_instance_id)
        .await?
        .ok_or(CommonError::Unknown(anyhow::anyhow!(
            "Provider instance not found"
        )))?;

    let provider_controller = get_provider_controller(
        &provider_instance
            .provider_instance
            .provider_controller_type_id,
    )?;

    let credential_controller = get_credential_controller(
        &provider_controller,
        &provider_instance
            .provider_instance
            .credential_controller_type_id,
    )?;
    let credential_controller_serialized: ProviderCredentialControllerSerialized =
        (&credential_controller).into();

    let functions = enrich_function_instances(provider_instance.functions, &provider_controller)?;

    let provider_instance_with_everything = ProviderInstanceSerializedWithEverything {
        functions,
        controller: provider_controller.as_ref().into(),
        credential_controller: credential_controller_serialized,
        instance_data: ProviderInstanceSerializedWithCredentials {
            provider_instance: provider_instance.provider_instance,
            resource_server_credential: provider_instance.resource_server_credential,
            user_credential: provider_instance.user_credential,
        },
    };
    Ok(provider_instance_with_everything)
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct WithProviderInstanceId<T> {
    pub provider_instance_id: String,
    pub inner: T,
}

#[derive(Serialize, Deserialize, Clone, ToSchema, JsonSchema)]
pub struct EnableFunctionParamsInner {}
pub type EnableFunctionParams =
    WithProviderInstanceId<WithFunctionControllerTypeId<EnableFunctionParamsInner>>;
pub type EnableFunctionResponse = FunctionInstanceSerialized;

pub async fn enable_function(
    on_config_change_tx: &OnConfigChangeTx,
    repo: &impl crate::repository::ProviderRepositoryLike,
    params: EnableFunctionParams,
    publish_on_change_evt: bool,
) -> Result<EnableFunctionResponse, CommonError> {
    // Verify provider instance exists
    let provider_instance = repo
        .get_provider_instance_by_id(&params.provider_instance_id)
        .await?
        .ok_or(CommonError::Unknown(anyhow::anyhow!(
            "Provider instance not found"
        )))?;

    // // Verify function exists in provider controller
    let provider_controller = get_provider_controller(
        &provider_instance
            .provider_instance
            .provider_controller_type_id,
    )?;
    let _function_controller = get_function_controller(
        &provider_controller,
        &params.inner.function_controller_type_id,
    )?;

    let now = WrappedChronoDateTime::now();
    let function_instance_serialized = FunctionInstanceSerialized {
        function_controller_type_id: params.inner.function_controller_type_id.clone(),
        provider_controller_type_id: provider_instance
            .provider_instance
            .provider_controller_type_id
            .clone(),
        provider_instance_id: params.provider_instance_id.clone(),
        created_at: now,
        updated_at: now,
    };

    // Save to database
    let create_params =
        crate::repository::CreateFunctionInstance::from(function_instance_serialized.clone());
    repo.create_function_instance(&create_params).await?;

    if publish_on_change_evt {
        on_config_change_tx
            .send(OnConfigChangeEvt::FunctionInstanceAdded(
                function_instance_serialized.clone(),
            ))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send config change event: {e}"))
            })?;
    }

    Ok(function_instance_serialized)
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct InvokeFunctionParamsInner {
    pub params: WrappedJsonValue,
}
pub type InvokeFunctionParams =
    WithProviderInstanceId<WithFunctionInstanceId<InvokeFunctionParamsInner>>;
pub type InvokeFunctionResponse = InvokeResult;

pub async fn invoke_function(
    repo: &crate::repository::Repository,
    encryption_service: &CryptoCache,
    params: InvokeFunctionParams,
) -> Result<InvokeFunctionResponse, CommonError> {
    // Get provider instance to retrieve provider_controller_type_id
    let provider_instance = repo
        .get_provider_instance_by_id(&params.provider_instance_id)
        .await?
        .ok_or(CommonError::Unknown(anyhow::anyhow!(
            "Provider instance not found"
        )))?;

    let function_instance_with_credentials = repo
        .get_function_instance_with_credentials(
            &params.inner.function_controller_type_id,
            &provider_instance
                .provider_instance
                .provider_controller_type_id,
            &params.provider_instance_id,
        )
        .await?
        .ok_or(CommonError::Unknown(anyhow::anyhow!(
            "Function instance not found"
        )))?;

    // Get decryption service from the encryption service cache using the DEK alias
    let decryption_service = encryption_service
        .get_decryption_service(
            &function_instance_with_credentials
                .resource_server_credential
                .dek_alias,
        )
        .await?;
    let provder_controller = get_provider_controller(
        &function_instance_with_credentials
            .provider_instance
            .provider_controller_type_id,
    )?;
    let function_controller = get_function_controller(
        &provder_controller,
        &function_instance_with_credentials
            .function_instance
            .function_controller_type_id,
    )?;

    // TODO: I think the credential controller should manage decrypting the resource server credential and user credential and static credentials
    // and pass a single return type to the function invocation that implements a DecryptedFullCredentialLike trait?
    let credential_controller = get_credential_controller(
        &provder_controller,
        &function_instance_with_credentials
            .provider_instance
            .credential_controller_type_id,
    )?;
    let static_credentials = credential_controller.static_credentials();

    let response = function_controller
        .invoke(
            &decryption_service,
            &credential_controller,
            static_credentials,
            &function_instance_with_credentials.resource_server_credential,
            &function_instance_with_credentials.user_credential,
            params.inner.inner.params,
        )
        .await?;
    Ok(response)
}

#[derive(Serialize, Deserialize, Clone, ToSchema, JsonSchema)]
pub struct DisableFunctionParamsInner {}
pub type DisableFunctionParams =
    WithProviderInstanceId<WithFunctionControllerTypeId<DisableFunctionParamsInner>>;
pub type DisableFunctionResponse = ();

pub async fn disable_function(
    on_config_change_tx: &OnConfigChangeTx,
    repo: &impl crate::repository::ProviderRepositoryLike,
    params: DisableFunctionParams,
    publish_on_change_evt: bool,
) -> Result<DisableFunctionResponse, CommonError> {
    // Get provider instance to retrieve provider_controller_type_id
    let provider_instance = repo
        .get_provider_instance_by_id(&params.provider_instance_id)
        .await?
        .ok_or(CommonError::Unknown(anyhow::anyhow!(
            "Provider instance not found"
        )))?;

    // Delete from database
    repo.delete_function_instance(
        &params.inner.function_controller_type_id,
        &provider_instance
            .provider_instance
            .provider_controller_type_id,
        &params.provider_instance_id,
    )
    .await?;

    if publish_on_change_evt {
        on_config_change_tx
            .send(OnConfigChangeEvt::FunctionInstanceRemoved(
                params.inner.function_controller_type_id.clone(),
                provider_instance
                    .provider_instance
                    .provider_controller_type_id
                    .clone(),
                params.provider_instance_id.clone(),
            ))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send config change event: {e}"))
            })?;
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct WithFunctionInstanceId<T> {
    pub function_controller_type_id: String,
    pub inner: T,
}

#[cfg(all(test, feature = "unit_test"))]
mod unit_test {
    use super::*;
    use shared::primitives::{PaginationRequest, SqlMigrationLoader};

    #[tokio::test]
    async fn test_list_provider_instances_empty() {
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

        let result = list_provider_instances(
            &repo,
            ListProviderInstancesParams {
                pagination,
                status: None,
                provider_controller_type_id: None,
            },
        )
        .await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.items.len(), 0);
        assert!(response.next_page_token.is_none());
    }

    #[tokio::test]
    async fn test_list_function_instances_empty() {
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

        let result = list_function_instances(
            &repo,
            ListFunctionInstancesParams {
                pagination,
                provider_instance_id: None,
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

        let (converted, defs) = convert_jsonschema_to_openapi(&schema_with_defs, "Test").unwrap();
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
