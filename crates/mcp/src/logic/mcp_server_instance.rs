use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shared::error::CommonError;
use shared::identity::Identity;
use shared::primitives::{PaginatedResponse, PaginationRequest, WrappedChronoDateTime};
use shared_macros::{authn, authz_role};
use tracing::trace;
use utoipa::{IntoParams, ToSchema};

use crate::logic::{OnConfigChangeEvt, OnConfigChangeTx};
use crate::repository::{
    CreateMcpServerInstance, CreateMcpServerInstanceFunction, ProviderRepositoryLike,
    UpdateMcpServerInstanceFunction,
};

/// Represents a function mapping within an MCP server instance
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, JsonSchema)]
pub struct McpServerInstanceFunctionSerialized {
    pub mcp_server_instance_id: String,
    pub function_controller_type_id: String,
    pub provider_controller_type_id: String,
    pub provider_instance_id: String,
    pub function_name: String,
    pub function_description: Option<String>,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

/// Represents an MCP server instance
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, JsonSchema)]
pub struct McpServerInstanceSerialized {
    pub id: String,
    pub name: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

/// Represents an MCP server instance with its associated functions
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, JsonSchema)]
pub struct McpServerInstanceSerializedWithFunctions {
    pub id: String,
    pub name: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
    pub functions: Vec<McpServerInstanceFunctionSerialized>,
}

// ============================================================================
// Request/Response types for API
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct CreateMcpServerInstanceRequest {
    pub id: String,
    pub name: String,
}

pub type CreateMcpServerInstanceResponse = McpServerInstanceSerializedWithFunctions;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct UpdateMcpServerInstanceRequest {
    pub name: String,
}

pub type UpdateMcpServerInstanceResponse = McpServerInstanceSerializedWithFunctions;

pub type GetMcpServerInstanceResponse = McpServerInstanceSerializedWithFunctions;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, IntoParams)]
#[into_params(style = Form, parameter_in = Query)]
pub struct ListMcpServerInstancesParams {
    pub page_size: i64,
    pub next_page_token: Option<String>,
}

pub type ListMcpServerInstancesResponse =
    PaginatedResponse<McpServerInstanceSerializedWithFunctions>;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct AddMcpServerInstanceFunctionRequest {
    pub function_controller_type_id: String,
    pub provider_controller_type_id: String,
    pub provider_instance_id: String,
    pub function_name: String,
    pub function_description: Option<String>,
}

pub type AddMcpServerInstanceFunctionResponse = McpServerInstanceSerializedWithFunctions;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct UpdateMcpServerInstanceFunctionRequest {
    pub function_name: String,
    pub function_description: Option<String>,
}

pub type UpdateMcpServerInstanceFunctionResponse = McpServerInstanceSerializedWithFunctions;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct RemoveMcpServerInstanceFunctionRequest {
    pub function_controller_type_id: String,
    pub provider_controller_type_id: String,
    pub provider_instance_id: String,
}

pub type RemoveMcpServerInstanceFunctionResponse = McpServerInstanceSerializedWithFunctions;

// ============================================================================
// Logic functions
// ============================================================================

/// Creates a new MCP server instance (internal implementation)
pub async fn create_mcp_server_instance_internal<R: ProviderRepositoryLike>(
    on_config_change_tx: &OnConfigChangeTx,
    repository: &R,
    request: CreateMcpServerInstanceRequest,
    publish_on_change_evt: bool,
) -> Result<CreateMcpServerInstanceResponse, CommonError> {
    trace!(instance_id = %request.id, name = %request.name, "Creating MCP server instance");
    let now = WrappedChronoDateTime::now();

    let create_params = CreateMcpServerInstance {
        id: request.id.clone(),
        name: request.name,
        created_at: now,
        updated_at: now,
    };

    repository
        .create_mcp_server_instance(&create_params)
        .await?;

    // Fetch the created instance to return it
    let instance = repository
        .get_mcp_server_instance_by_id(&request.id)
        .await?
        .ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!(
                "Failed to retrieve created MCP server instance"
            ))
        })?;

    if publish_on_change_evt {
        on_config_change_tx
            .send(OnConfigChangeEvt::McpServerInstanceAdded(instance.clone()))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send config change event: {e}"))
            })?;
    }

    Ok(instance)
}

/// Creates a new MCP server instance
#[authz_role(Admin, Maintainer, permission = "mcp:write")]
#[authn]
pub async fn create_mcp_server_instance<R: ProviderRepositoryLike>(
    _identity: Identity,
    on_config_change_tx: &OnConfigChangeTx,
    repository: &R,
    request: CreateMcpServerInstanceRequest,
    publish_on_change_evt: bool,
) -> Result<CreateMcpServerInstanceResponse, CommonError> {
    let _ = &identity;
    create_mcp_server_instance_internal(
        on_config_change_tx,
        repository,
        request,
        publish_on_change_evt,
    )
    .await
}

/// Gets an MCP server instance by ID (internal implementation)
pub async fn get_mcp_server_instance_internal<R: ProviderRepositoryLike>(
    repository: &R,
    id: &str,
) -> Result<GetMcpServerInstanceResponse, CommonError> {
    trace!(instance_id = %id, "Getting MCP server instance");
    let instance = repository
        .get_mcp_server_instance_by_id(id)
        .await?
        .ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!("MCP server instance not found: {id}"))
        })?;

    Ok(instance)
}

/// Gets an MCP server instance by ID
#[authz_role(Admin, Maintainer, Agent, permission = "mcp:read")]
#[authn]
pub async fn get_mcp_server_instance<R: ProviderRepositoryLike>(
    _identity: Identity,
    repository: &R,
    id: &str,
) -> Result<GetMcpServerInstanceResponse, CommonError> {
    let _ = &identity;
    get_mcp_server_instance_internal(repository, id).await
}

/// Updates an MCP server instance name (internal implementation)
pub async fn update_mcp_server_instance_internal<R: ProviderRepositoryLike>(
    on_config_change_tx: &OnConfigChangeTx,
    repository: &R,
    id: &str,
    request: UpdateMcpServerInstanceRequest,
    publish_on_change_evt: bool,
) -> Result<UpdateMcpServerInstanceResponse, CommonError> {
    trace!(instance_id = %id, name = %request.name, "Updating MCP server instance");
    // Verify the instance exists first
    let _ = repository
        .get_mcp_server_instance_by_id(id)
        .await?
        .ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!("MCP server instance not found: {id}"))
        })?;

    repository
        .update_mcp_server_instance(id, &request.name)
        .await?;

    // Fetch the updated instance
    let instance = repository
        .get_mcp_server_instance_by_id(id)
        .await?
        .ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!(
                "Failed to retrieve updated MCP server instance"
            ))
        })?;

    if publish_on_change_evt {
        on_config_change_tx
            .send(OnConfigChangeEvt::McpServerInstanceUpdated(
                instance.clone(),
            ))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send config change event: {e}"))
            })?;
    }

    Ok(instance)
}

/// Updates an MCP server instance name
#[authz_role(Admin, Maintainer, permission = "mcp:write")]
#[authn]
#[allow(clippy::too_many_arguments)]
pub async fn update_mcp_server_instance<R: ProviderRepositoryLike>(
    _identity: Identity,
    on_config_change_tx: &OnConfigChangeTx,
    repository: &R,
    id: &str,
    request: UpdateMcpServerInstanceRequest,
    publish_on_change_evt: bool,
) -> Result<UpdateMcpServerInstanceResponse, CommonError> {
    let _ = &identity;
    update_mcp_server_instance_internal(
        on_config_change_tx,
        repository,
        id,
        request,
        publish_on_change_evt,
    )
    .await
}

/// Deletes an MCP server instance (internal implementation)
pub async fn delete_mcp_server_instance_internal<R: ProviderRepositoryLike>(
    on_config_change_tx: &OnConfigChangeTx,
    repository: &R,
    id: &str,
    publish_on_change_evt: bool,
) -> Result<(), CommonError> {
    trace!(instance_id = %id, "Deleting MCP server instance");
    // Verify the instance exists first
    let _ = repository
        .get_mcp_server_instance_by_id(id)
        .await?
        .ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!("MCP server instance not found: {id}"))
        })?;

    repository.delete_mcp_server_instance(id).await?;

    if publish_on_change_evt {
        on_config_change_tx
            .send(OnConfigChangeEvt::McpServerInstanceRemoved(id.to_string()))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send config change event: {e}"))
            })?;
    }

    Ok(())
}

/// Deletes an MCP server instance
#[authz_role(Admin, Maintainer, permission = "mcp:write")]
#[authn]
pub async fn delete_mcp_server_instance<R: ProviderRepositoryLike>(
    _identity: Identity,
    on_config_change_tx: &OnConfigChangeTx,
    repository: &R,
    id: &str,
    publish_on_change_evt: bool,
) -> Result<(), CommonError> {
    let _ = &identity;
    delete_mcp_server_instance_internal(on_config_change_tx, repository, id, publish_on_change_evt)
        .await
}

/// Lists all MCP server instances with pagination (internal implementation)
pub async fn list_mcp_server_instances_internal<R: ProviderRepositoryLike>(
    repository: &R,
    params: ListMcpServerInstancesParams,
) -> Result<ListMcpServerInstancesResponse, CommonError> {
    trace!(page_size = params.page_size, "Listing MCP server instances");
    let pagination = PaginationRequest {
        page_size: params.page_size,
        next_page_token: params.next_page_token,
    };

    let result = repository.list_mcp_server_instances(&pagination).await?;

    Ok(result)
}

/// Lists all MCP server instances with pagination
#[authz_role(Admin, Maintainer, Agent, permission = "mcp:list")]
#[authn]
pub async fn list_mcp_server_instances<R: ProviderRepositoryLike>(
    _identity: Identity,
    repository: &R,
    params: ListMcpServerInstancesParams,
) -> Result<ListMcpServerInstancesResponse, CommonError> {
    let _ = &identity;
    list_mcp_server_instances_internal(repository, params).await
}

/// Adds a function to an MCP server instance (internal implementation)
pub async fn add_mcp_server_instance_function_internal<R: ProviderRepositoryLike>(
    on_config_change_tx: &OnConfigChangeTx,
    repository: &R,
    mcp_server_instance_id: &str,
    request: AddMcpServerInstanceFunctionRequest,
    publish_on_change_evt: bool,
) -> Result<AddMcpServerInstanceFunctionResponse, CommonError> {
    trace!(
        instance_id = %mcp_server_instance_id,
        function_name = %request.function_name,
        function_type = %request.function_controller_type_id,
        "Adding function to MCP server instance"
    );
    // Verify the instance exists
    let _ = repository
        .get_mcp_server_instance_by_id(mcp_server_instance_id)
        .await?
        .ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!(
                "MCP server instance not found: {mcp_server_instance_id}"
            ))
        })?;

    // Check if function_name is already used in this instance
    let existing = repository
        .get_mcp_server_instance_function_by_name(mcp_server_instance_id, &request.function_name)
        .await?;

    if existing.is_some() {
        return Err(CommonError::InvalidRequest {
            msg: format!(
                "Function name '{}' already exists in MCP server instance '{}'",
                request.function_name, mcp_server_instance_id
            ),
            source: None,
        });
    }

    // Verify the function instance exists
    let function_instance = repository
        .get_function_instance_by_id(
            &request.function_controller_type_id,
            &request.provider_controller_type_id,
            &request.provider_instance_id,
        )
        .await?
        .ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!(
                "Function instance not found: {}/{}/{}",
                request.function_controller_type_id,
                request.provider_controller_type_id,
                request.provider_instance_id
            ))
        })?;

    let now = WrappedChronoDateTime::now();

    let create_params = CreateMcpServerInstanceFunction {
        mcp_server_instance_id: mcp_server_instance_id.to_string(),
        function_controller_type_id: function_instance.function_controller_type_id.clone(),
        provider_controller_type_id: function_instance.provider_controller_type_id.clone(),
        provider_instance_id: function_instance.provider_instance_id.clone(),
        function_name: request.function_name.clone(),
        function_description: request.function_description.clone(),
        created_at: now,
        updated_at: now,
    };

    repository
        .create_mcp_server_instance_function(&create_params)
        .await?;

    // Create the serialized function for the event
    let function_serialized = McpServerInstanceFunctionSerialized {
        mcp_server_instance_id: mcp_server_instance_id.to_string(),
        function_controller_type_id: function_instance.function_controller_type_id,
        provider_controller_type_id: function_instance.provider_controller_type_id,
        provider_instance_id: function_instance.provider_instance_id,
        function_name: request.function_name,
        function_description: request.function_description,
        created_at: now,
        updated_at: now,
    };

    if publish_on_change_evt {
        on_config_change_tx
            .send(OnConfigChangeEvt::McpServerInstanceFunctionAdded(
                function_serialized,
            ))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send config change event: {e}"))
            })?;
    }

    // Fetch the updated instance
    let instance = repository
        .get_mcp_server_instance_by_id(mcp_server_instance_id)
        .await?
        .ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!(
                "Failed to retrieve updated MCP server instance"
            ))
        })?;

    Ok(instance)
}

/// Adds a function to an MCP server instance
#[authz_role(Admin, Maintainer, permission = "mcp:write")]
#[authn]
#[allow(clippy::too_many_arguments)]
pub async fn add_mcp_server_instance_function<R: ProviderRepositoryLike>(
    _identity: Identity,
    on_config_change_tx: &OnConfigChangeTx,
    repository: &R,
    mcp_server_instance_id: &str,
    request: AddMcpServerInstanceFunctionRequest,
    publish_on_change_evt: bool,
) -> Result<AddMcpServerInstanceFunctionResponse, CommonError> {
    let _ = &identity;
    add_mcp_server_instance_function_internal(
        on_config_change_tx,
        repository,
        mcp_server_instance_id,
        request,
        publish_on_change_evt,
    )
    .await
}

/// Updates a function in an MCP server instance (internal implementation)
#[allow(clippy::too_many_arguments)]
pub async fn update_mcp_server_instance_function_internal<R: ProviderRepositoryLike>(
    on_config_change_tx: &OnConfigChangeTx,
    repository: &R,
    mcp_server_instance_id: &str,
    function_controller_type_id: &str,
    provider_controller_type_id: &str,
    provider_instance_id: &str,
    request: UpdateMcpServerInstanceFunctionRequest,
    publish_on_change_evt: bool,
) -> Result<UpdateMcpServerInstanceFunctionResponse, CommonError> {
    trace!(
        instance_id = %mcp_server_instance_id,
        function_type = %function_controller_type_id,
        provider_type = %provider_controller_type_id,
        provider_instance_id = %provider_instance_id,
        new_name = %request.function_name,
        "Updating MCP server instance function"
    );
    // Verify the instance exists
    let instance = repository
        .get_mcp_server_instance_by_id(mcp_server_instance_id)
        .await?
        .ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!(
                "MCP server instance not found: {mcp_server_instance_id}"
            ))
        })?;

    // Find the function in the instance
    let existing_function = instance.functions.iter().find(|f| {
        f.function_controller_type_id == function_controller_type_id
            && f.provider_controller_type_id == provider_controller_type_id
            && f.provider_instance_id == provider_instance_id
    });

    let existing_function = existing_function.ok_or_else(|| {
        CommonError::Unknown(anyhow::anyhow!(
            "Function not found in MCP server instance: {mcp_server_instance_id}/{function_controller_type_id}/{provider_controller_type_id}/{provider_instance_id}"
        ))
    })?;

    // Check if new function_name conflicts with another function in this instance
    let existing = repository
        .get_mcp_server_instance_function_by_name(mcp_server_instance_id, &request.function_name)
        .await?;

    if let Some(existing_fn) = existing {
        // Only conflict if it's a different function
        if existing_fn.function_controller_type_id != function_controller_type_id
            || existing_fn.provider_controller_type_id != provider_controller_type_id
            || existing_fn.provider_instance_id != provider_instance_id
        {
            return Err(CommonError::InvalidRequest {
                msg: format!(
                    "Function name '{}' already exists in MCP server instance '{}'",
                    request.function_name, mcp_server_instance_id
                ),
                source: None,
            });
        }
    }

    let update_params = UpdateMcpServerInstanceFunction {
        mcp_server_instance_id: mcp_server_instance_id.to_string(),
        function_controller_type_id: function_controller_type_id.to_string(),
        provider_controller_type_id: provider_controller_type_id.to_string(),
        provider_instance_id: provider_instance_id.to_string(),
        function_name: request.function_name.clone(),
        function_description: request.function_description.clone(),
    };

    repository
        .update_mcp_server_instance_function(&update_params)
        .await?;

    // Create the serialized function for the event
    let function_serialized = McpServerInstanceFunctionSerialized {
        mcp_server_instance_id: mcp_server_instance_id.to_string(),
        function_controller_type_id: function_controller_type_id.to_string(),
        provider_controller_type_id: provider_controller_type_id.to_string(),
        provider_instance_id: provider_instance_id.to_string(),
        function_name: request.function_name,
        function_description: request.function_description,
        created_at: existing_function.created_at,
        updated_at: WrappedChronoDateTime::now(),
    };

    if publish_on_change_evt {
        on_config_change_tx
            .send(OnConfigChangeEvt::McpServerInstanceFunctionUpdated(
                function_serialized,
            ))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send config change event: {e}"))
            })?;
    }

    // Fetch the updated instance
    let instance = repository
        .get_mcp_server_instance_by_id(mcp_server_instance_id)
        .await?
        .ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!(
                "Failed to retrieve updated MCP server instance"
            ))
        })?;

    Ok(instance)
}

/// Updates a function in an MCP server instance (only function_name and function_description)
#[allow(clippy::too_many_arguments)]
#[authz_role(Admin, Maintainer, permission = "mcp:write")]
#[authn]
pub async fn update_mcp_server_instance_function<R: ProviderRepositoryLike>(
    _identity: Identity,
    on_config_change_tx: &OnConfigChangeTx,
    repository: &R,
    mcp_server_instance_id: &str,
    function_controller_type_id: &str,
    provider_controller_type_id: &str,
    provider_instance_id: &str,
    request: UpdateMcpServerInstanceFunctionRequest,
    publish_on_change_evt: bool,
) -> Result<UpdateMcpServerInstanceFunctionResponse, CommonError> {
    let _ = &identity;
    update_mcp_server_instance_function_internal(
        on_config_change_tx,
        repository,
        mcp_server_instance_id,
        function_controller_type_id,
        provider_controller_type_id,
        provider_instance_id,
        request,
        publish_on_change_evt,
    )
    .await
}

/// Removes a function from an MCP server instance (internal implementation)
pub async fn remove_mcp_server_instance_function_internal<R: ProviderRepositoryLike>(
    on_config_change_tx: &OnConfigChangeTx,
    repository: &R,
    mcp_server_instance_id: &str,
    function_controller_type_id: &str,
    provider_controller_type_id: &str,
    provider_instance_id: &str,
    publish_on_change_evt: bool,
) -> Result<RemoveMcpServerInstanceFunctionResponse, CommonError> {
    trace!(
        instance_id = %mcp_server_instance_id,
        function_type = %function_controller_type_id,
        provider_type = %provider_controller_type_id,
        provider_instance_id = %provider_instance_id,
        "Removing function from MCP server instance"
    );
    // Verify the instance exists
    let instance = repository
        .get_mcp_server_instance_by_id(mcp_server_instance_id)
        .await?
        .ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!(
                "MCP server instance not found: {mcp_server_instance_id}"
            ))
        })?;

    // Find the function in the instance
    let function_exists = instance.functions.iter().any(|f| {
        f.function_controller_type_id == function_controller_type_id
            && f.provider_controller_type_id == provider_controller_type_id
            && f.provider_instance_id == provider_instance_id
    });

    if !function_exists {
        return Err(CommonError::Unknown(anyhow::anyhow!(
            "Function not found in MCP server instance: {mcp_server_instance_id}/{function_controller_type_id}/{provider_controller_type_id}/{provider_instance_id}"
        )));
    }

    repository
        .delete_mcp_server_instance_function(
            mcp_server_instance_id,
            function_controller_type_id,
            provider_controller_type_id,
            provider_instance_id,
        )
        .await?;

    if publish_on_change_evt {
        on_config_change_tx
            .send(OnConfigChangeEvt::McpServerInstanceFunctionRemoved(
                mcp_server_instance_id.to_string(),
                function_controller_type_id.to_string(),
                provider_controller_type_id.to_string(),
                provider_instance_id.to_string(),
            ))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send config change event: {e}"))
            })?;
    }

    // Fetch the updated instance
    let instance = repository
        .get_mcp_server_instance_by_id(mcp_server_instance_id)
        .await?
        .ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!(
                "Failed to retrieve updated MCP server instance"
            ))
        })?;

    Ok(instance)
}

/// Removes a function from an MCP server instance
#[allow(clippy::too_many_arguments)]
#[authz_role(Admin, Maintainer, permission = "mcp:write")]
#[authn]
pub async fn remove_mcp_server_instance_function<R: ProviderRepositoryLike>(
    _identity: Identity,
    on_config_change_tx: &OnConfigChangeTx,
    repository: &R,
    mcp_server_instance_id: &str,
    function_controller_type_id: &str,
    provider_controller_type_id: &str,
    provider_instance_id: &str,
    publish_on_change_evt: bool,
) -> Result<RemoveMcpServerInstanceFunctionResponse, CommonError> {
    let _ = &identity;
    remove_mcp_server_instance_function_internal(
        on_config_change_tx,
        repository,
        mcp_server_instance_id,
        function_controller_type_id,
        provider_controller_type_id,
        provider_instance_id,
        publish_on_change_evt,
    )
    .await
}

#[cfg(test)]
mod tests {
    mod unit {
        use super::super::*;
        use crate::logic::Metadata;
        use crate::logic::credential::{
            ResourceServerCredentialSerialized, UserCredentialSerialized,
        };
        use crate::logic::instance::{FunctionInstanceSerialized, ProviderInstanceSerialized};
        use crate::repository::{
            CreateFunctionInstance, CreateProviderInstance, CreateResourceServerCredential,
            CreateUserCredential, ProviderRepositoryLike,
        };
        use shared::primitives::{SqlMigrationLoader, WrappedJsonValue, WrappedUuidV4};
        use shared::test_utils::repository::setup_in_memory_database;

        /// Helper to create a test DEK alias for tests.
        fn create_test_dek_alias() -> String {
            format!("test-dek-{}", uuid::Uuid::new_v4())
        }

        /// Helper to create a broadcast channel for tests.
        fn create_test_channel() -> OnConfigChangeTx {
            let (tx, _rx) = tokio::sync::broadcast::channel(100);
            tx
        }

        /// Helper to create the necessary provider instance and function instance
        /// for MCP server instance function tests.
        async fn setup_function_instance(
            repo: &crate::repository::Repository,
        ) -> (String, String, String) {
            let now = WrappedChronoDateTime::now();
            let dek_alias = create_test_dek_alias();

            // Create resource server credential
            let resource_server_cred = ResourceServerCredentialSerialized {
                id: WrappedUuidV4::new(),
                type_id: "resource_server_no_auth".to_string(),
                metadata: Metadata::new(),
                value: WrappedJsonValue::new(serde_json::json!({})),
                created_at: now,
                updated_at: now,
                next_rotation_time: None,
                dek_alias: dek_alias.clone(),
            };
            repo.create_resource_server_credential(&CreateResourceServerCredential::from(
                resource_server_cred.clone(),
            ))
            .await
            .unwrap();

            // Create user credential
            let user_cred = UserCredentialSerialized {
                id: WrappedUuidV4::new(),
                type_id: "no_auth".to_string(),
                metadata: Metadata::new(),
                value: WrappedJsonValue::new(serde_json::json!({})),
                created_at: now,
                updated_at: now,
                next_rotation_time: None,
                dek_alias: dek_alias.clone(),
            };
            repo.create_user_credential(&CreateUserCredential::from(user_cred.clone()))
                .await
                .unwrap();

            // Create provider instance
            let provider_instance_id = uuid::Uuid::new_v4().to_string();
            let provider_controller_type_id = "google_mail".to_string();
            let provider_instance = ProviderInstanceSerialized {
                id: provider_instance_id.clone(),
                display_name: "Test Provider".to_string(),
                resource_server_credential_id: resource_server_cred.id.clone(),
                user_credential_id: Some(user_cred.id.clone()),
                created_at: now,
                updated_at: now,
                provider_controller_type_id: provider_controller_type_id.clone(),
                credential_controller_type_id: "no_auth".to_string(),
                status: "active".to_string(),
                return_on_successful_brokering: None,
            };
            repo.create_provider_instance(&CreateProviderInstance::from(provider_instance))
                .await
                .unwrap();

            // Create function instance
            let function_controller_type_id = "send_email".to_string();
            let function_instance = FunctionInstanceSerialized {
                function_controller_type_id: function_controller_type_id.clone(),
                provider_controller_type_id: provider_controller_type_id.clone(),
                provider_instance_id: provider_instance_id.clone(),
                created_at: now,
                updated_at: now,
            };
            repo.create_function_instance(&CreateFunctionInstance::from(function_instance))
                .await
                .unwrap();

            (
                function_controller_type_id,
                provider_controller_type_id,
                provider_instance_id,
            )
        }

        #[tokio::test]
        async fn test_create_mcp_server_instance() {
            shared::setup_test!();

            let (_db, conn) =
                setup_in_memory_database(
                    vec![crate::repository::Repository::load_sql_migrations()],
                )
                .await
                .unwrap();
            let repo = crate::repository::Repository::new(conn);
            let tx = create_test_channel();

            let request = CreateMcpServerInstanceRequest {
                id: "test-mcp-instance".to_string(),
                name: "Test MCP Instance".to_string(),
            };

            let result =
                create_mcp_server_instance_internal(&tx, &repo, request.clone(), false).await;
            assert!(result.is_ok(), "Expected Ok, got {result:?}");

            let instance = result.unwrap();
            assert_eq!(instance.id, request.id);
            assert_eq!(instance.name, request.name);
            assert!(instance.functions.is_empty());
        }

        #[tokio::test]
        async fn test_get_mcp_server_instance() {
            shared::setup_test!();

            let (_db, conn) =
                setup_in_memory_database(
                    vec![crate::repository::Repository::load_sql_migrations()],
                )
                .await
                .unwrap();
            let repo = crate::repository::Repository::new(conn);
            let tx = create_test_channel();

            // Create an instance first
            let create_request = CreateMcpServerInstanceRequest {
                id: "test-get-instance".to_string(),
                name: "Test Get Instance".to_string(),
            };
            create_mcp_server_instance_internal(&tx, &repo, create_request.clone(), false)
                .await
                .unwrap();

            // Retrieve it
            let result = get_mcp_server_instance_internal(&repo, &create_request.id).await;
            assert!(result.is_ok());

            let instance = result.unwrap();
            assert_eq!(instance.id, create_request.id);
            assert_eq!(instance.name, create_request.name);
        }

        #[tokio::test]
        async fn test_get_mcp_server_instance_not_found() {
            shared::setup_test!();

            let (_db, conn) =
                setup_in_memory_database(
                    vec![crate::repository::Repository::load_sql_migrations()],
                )
                .await
                .unwrap();
            let repo = crate::repository::Repository::new(conn);

            let result = get_mcp_server_instance_internal(&repo, "non-existent-id").await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_update_mcp_server_instance() {
            shared::setup_test!();

            let (_db, conn) =
                setup_in_memory_database(
                    vec![crate::repository::Repository::load_sql_migrations()],
                )
                .await
                .unwrap();
            let repo = crate::repository::Repository::new(conn);
            let tx = create_test_channel();

            // Create an instance
            let create_request = CreateMcpServerInstanceRequest {
                id: "test-update-instance".to_string(),
                name: "Original Name".to_string(),
            };
            create_mcp_server_instance_internal(&tx, &repo, create_request.clone(), false)
                .await
                .unwrap();

            // Update it
            let update_request = UpdateMcpServerInstanceRequest {
                name: "Updated Name".to_string(),
            };
            let result = update_mcp_server_instance_internal(
                &tx,
                &repo,
                &create_request.id,
                update_request.clone(),
                false,
            )
            .await;
            assert!(result.is_ok());

            let instance = result.unwrap();
            assert_eq!(instance.name, update_request.name);
        }

        #[tokio::test]
        async fn test_update_mcp_server_instance_not_found() {
            shared::setup_test!();

            let (_db, conn) =
                setup_in_memory_database(
                    vec![crate::repository::Repository::load_sql_migrations()],
                )
                .await
                .unwrap();
            let repo = crate::repository::Repository::new(conn);
            let tx = create_test_channel();

            let update_request = UpdateMcpServerInstanceRequest {
                name: "Updated Name".to_string(),
            };
            let result = update_mcp_server_instance_internal(
                &tx,
                &repo,
                "non-existent-id",
                update_request,
                false,
            )
            .await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_delete_mcp_server_instance() {
            shared::setup_test!();

            let (_db, conn) =
                setup_in_memory_database(
                    vec![crate::repository::Repository::load_sql_migrations()],
                )
                .await
                .unwrap();
            let repo = crate::repository::Repository::new(conn);
            let tx = create_test_channel();

            // Create an instance
            let create_request = CreateMcpServerInstanceRequest {
                id: "test-delete-instance".to_string(),
                name: "Test Delete Instance".to_string(),
            };
            create_mcp_server_instance_internal(&tx, &repo, create_request.clone(), false)
                .await
                .unwrap();

            // Delete it
            let result =
                delete_mcp_server_instance_internal(&tx, &repo, &create_request.id, false).await;
            assert!(result.is_ok());

            // Verify it's gone
            let get_result = get_mcp_server_instance_internal(&repo, &create_request.id).await;
            assert!(get_result.is_err());
        }

        #[tokio::test]
        async fn test_delete_mcp_server_instance_not_found() {
            shared::setup_test!();

            let (_db, conn) =
                setup_in_memory_database(
                    vec![crate::repository::Repository::load_sql_migrations()],
                )
                .await
                .unwrap();
            let repo = crate::repository::Repository::new(conn);
            let tx = create_test_channel();

            let result =
                delete_mcp_server_instance_internal(&tx, &repo, "non-existent-id", false).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_list_mcp_server_instances_empty() {
            shared::setup_test!();

            let (_db, conn) =
                setup_in_memory_database(
                    vec![crate::repository::Repository::load_sql_migrations()],
                )
                .await
                .unwrap();
            let repo = crate::repository::Repository::new(conn);

            let params = ListMcpServerInstancesParams {
                page_size: 10,
                next_page_token: None,
            };

            let result = list_mcp_server_instances_internal(&repo, params).await;
            assert!(result.is_ok());

            let response = result.unwrap();
            assert_eq!(response.items.len(), 0);
            assert!(response.next_page_token.is_none());
        }

        #[tokio::test]
        async fn test_list_mcp_server_instances_with_items() {
            shared::setup_test!();

            let (_db, conn) =
                setup_in_memory_database(
                    vec![crate::repository::Repository::load_sql_migrations()],
                )
                .await
                .unwrap();
            let repo = crate::repository::Repository::new(conn);
            let tx = create_test_channel();

            // Create some instances
            for i in 0..3 {
                let request = CreateMcpServerInstanceRequest {
                    id: format!("test-list-instance-{i}"),
                    name: format!("Test Instance {i}"),
                };
                create_mcp_server_instance_internal(&tx, &repo, request, false)
                    .await
                    .unwrap();
            }

            let params = ListMcpServerInstancesParams {
                page_size: 10,
                next_page_token: None,
            };

            let result = list_mcp_server_instances_internal(&repo, params).await;
            assert!(result.is_ok());

            let response = result.unwrap();
            assert_eq!(response.items.len(), 3);
        }

        #[tokio::test]
        async fn test_list_mcp_server_instances_pagination() {
            shared::setup_test!();

            let (_db, conn) =
                setup_in_memory_database(
                    vec![crate::repository::Repository::load_sql_migrations()],
                )
                .await
                .unwrap();
            let repo = crate::repository::Repository::new(conn);
            let tx = create_test_channel();

            // Create 5 instances
            for i in 0..5 {
                let request = CreateMcpServerInstanceRequest {
                    id: format!("test-pagination-instance-{i}"),
                    name: format!("Test Instance {i}"),
                };
                create_mcp_server_instance_internal(&tx, &repo, request, false)
                    .await
                    .unwrap();
            }

            // First page
            let params = ListMcpServerInstancesParams {
                page_size: 2,
                next_page_token: None,
            };

            let result = list_mcp_server_instances_internal(&repo, params).await;
            assert!(result.is_ok());

            let response = result.unwrap();
            assert_eq!(response.items.len(), 2);
            assert!(response.next_page_token.is_some());

            // Second page
            let params = ListMcpServerInstancesParams {
                page_size: 2,
                next_page_token: response.next_page_token,
            };

            let result = list_mcp_server_instances_internal(&repo, params).await;
            assert!(result.is_ok());

            let response = result.unwrap();
            assert_eq!(response.items.len(), 2);
        }

        #[tokio::test]
        async fn test_add_mcp_server_instance_function() {
            shared::setup_test!();

            let (_db, conn) =
                setup_in_memory_database(
                    vec![crate::repository::Repository::load_sql_migrations()],
                )
                .await
                .unwrap();
            let repo = crate::repository::Repository::new(conn);
            let tx = create_test_channel();

            // Setup function instance
            let (function_controller_type_id, provider_controller_type_id, provider_instance_id) =
                setup_function_instance(&repo).await;

            // Create MCP instance
            let create_request = CreateMcpServerInstanceRequest {
                id: "test-add-function-instance".to_string(),
                name: "Test Add Function Instance".to_string(),
            };
            create_mcp_server_instance_internal(&tx, &repo, create_request.clone(), false)
                .await
                .unwrap();

            // Add function
            let add_request = AddMcpServerInstanceFunctionRequest {
                function_controller_type_id: function_controller_type_id.clone(),
                provider_controller_type_id: provider_controller_type_id.clone(),
                provider_instance_id: provider_instance_id.clone(),
                function_name: "my_custom_function".to_string(),
                function_description: Some("A custom function".to_string()),
            };

            let result = add_mcp_server_instance_function_internal(
                &tx,
                &repo,
                &create_request.id,
                add_request.clone(),
                false,
            )
            .await;
            assert!(result.is_ok(), "Expected Ok, got {result:?}");

            let instance = result.unwrap();
            assert_eq!(instance.functions.len(), 1);
            assert_eq!(instance.functions[0].function_name, "my_custom_function");
            assert_eq!(
                instance.functions[0].function_description,
                Some("A custom function".to_string())
            );
        }

        #[tokio::test]
        async fn test_add_mcp_server_instance_function_duplicate_name() {
            shared::setup_test!();

            let (_db, conn) =
                setup_in_memory_database(
                    vec![crate::repository::Repository::load_sql_migrations()],
                )
                .await
                .unwrap();
            let repo = crate::repository::Repository::new(conn);
            let tx = create_test_channel();

            // Setup function instance
            let (function_controller_type_id, provider_controller_type_id, provider_instance_id) =
                setup_function_instance(&repo).await;

            // Create MCP instance
            let create_request = CreateMcpServerInstanceRequest {
                id: "test-duplicate-function-name".to_string(),
                name: "Test Duplicate Function Name".to_string(),
            };
            create_mcp_server_instance_internal(&tx, &repo, create_request.clone(), false)
                .await
                .unwrap();

            // Add first function
            let add_request = AddMcpServerInstanceFunctionRequest {
                function_controller_type_id: function_controller_type_id.clone(),
                provider_controller_type_id: provider_controller_type_id.clone(),
                provider_instance_id: provider_instance_id.clone(),
                function_name: "duplicate_name".to_string(),
                function_description: None,
            };

            add_mcp_server_instance_function_internal(
                &tx,
                &repo,
                &create_request.id,
                add_request.clone(),
                false,
            )
            .await
            .unwrap();

            // Try to add another function with the same name - should fail
            // We need another function instance for this test
            let now = WrappedChronoDateTime::now();
            let function_instance2 = FunctionInstanceSerialized {
                function_controller_type_id: "another_function".to_string(),
                provider_controller_type_id: provider_controller_type_id.clone(),
                provider_instance_id: provider_instance_id.clone(),
                created_at: now,
                updated_at: now,
            };
            repo.create_function_instance(&CreateFunctionInstance::from(function_instance2))
                .await
                .unwrap();

            let add_request2 = AddMcpServerInstanceFunctionRequest {
                function_controller_type_id: "another_function".to_string(),
                provider_controller_type_id: provider_controller_type_id.clone(),
                provider_instance_id: provider_instance_id.clone(),
                function_name: "duplicate_name".to_string(), // Same name
                function_description: None,
            };

            let result = add_mcp_server_instance_function_internal(
                &tx,
                &repo,
                &create_request.id,
                add_request2,
                false,
            )
            .await;
            assert!(result.is_err());

            // Check that it's an InvalidRequest error
            match result {
                Err(CommonError::InvalidRequest { msg, .. }) => {
                    assert!(msg.contains("already exists"));
                }
                other => panic!("Expected InvalidRequest error, got {other:?}"),
            }
        }

        #[tokio::test]
        async fn test_add_mcp_server_instance_function_instance_not_found() {
            shared::setup_test!();

            let (_db, conn) =
                setup_in_memory_database(
                    vec![crate::repository::Repository::load_sql_migrations()],
                )
                .await
                .unwrap();
            let repo = crate::repository::Repository::new(conn);
            let tx = create_test_channel();

            // Create MCP instance without setting up function instance
            let create_request = CreateMcpServerInstanceRequest {
                id: "test-function-not-found".to_string(),
                name: "Test Function Not Found".to_string(),
            };
            create_mcp_server_instance_internal(&tx, &repo, create_request.clone(), false)
                .await
                .unwrap();

            // Try to add function that doesn't exist
            let add_request = AddMcpServerInstanceFunctionRequest {
                function_controller_type_id: "nonexistent".to_string(),
                provider_controller_type_id: "nonexistent".to_string(),
                provider_instance_id: "nonexistent".to_string(),
                function_name: "my_function".to_string(),
                function_description: None,
            };

            let result = add_mcp_server_instance_function_internal(
                &tx,
                &repo,
                &create_request.id,
                add_request,
                false,
            )
            .await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_update_mcp_server_instance_function() {
            shared::setup_test!();

            let (_db, conn) =
                setup_in_memory_database(
                    vec![crate::repository::Repository::load_sql_migrations()],
                )
                .await
                .unwrap();
            let repo = crate::repository::Repository::new(conn);
            let tx = create_test_channel();

            // Setup function instance
            let (function_controller_type_id, provider_controller_type_id, provider_instance_id) =
                setup_function_instance(&repo).await;

            // Create MCP instance
            let create_request = CreateMcpServerInstanceRequest {
                id: "test-update-function".to_string(),
                name: "Test Update Function".to_string(),
            };
            create_mcp_server_instance_internal(&tx, &repo, create_request.clone(), false)
                .await
                .unwrap();

            // Add function
            let add_request = AddMcpServerInstanceFunctionRequest {
                function_controller_type_id: function_controller_type_id.clone(),
                provider_controller_type_id: provider_controller_type_id.clone(),
                provider_instance_id: provider_instance_id.clone(),
                function_name: "original_name".to_string(),
                function_description: None,
            };
            add_mcp_server_instance_function_internal(
                &tx,
                &repo,
                &create_request.id,
                add_request,
                false,
            )
            .await
            .unwrap();

            // Update function
            let update_request = UpdateMcpServerInstanceFunctionRequest {
                function_name: "updated_name".to_string(),
                function_description: Some("Updated description".to_string()),
            };

            let result = update_mcp_server_instance_function_internal(
                &tx,
                &repo,
                &create_request.id,
                &function_controller_type_id,
                &provider_controller_type_id,
                &provider_instance_id,
                update_request,
                false,
            )
            .await;
            assert!(result.is_ok());

            let instance = result.unwrap();
            assert_eq!(instance.functions[0].function_name, "updated_name");
            assert_eq!(
                instance.functions[0].function_description,
                Some("Updated description".to_string())
            );
        }

        #[tokio::test]
        async fn test_update_mcp_server_instance_function_name_conflict() {
            shared::setup_test!();

            let (_db, conn) =
                setup_in_memory_database(
                    vec![crate::repository::Repository::load_sql_migrations()],
                )
                .await
                .unwrap();
            let repo = crate::repository::Repository::new(conn);
            let tx = create_test_channel();

            // Setup function instances
            let (function_controller_type_id, provider_controller_type_id, provider_instance_id) =
                setup_function_instance(&repo).await;

            // Create another function instance
            let now = WrappedChronoDateTime::now();
            let function_instance2 = FunctionInstanceSerialized {
                function_controller_type_id: "another_function".to_string(),
                provider_controller_type_id: provider_controller_type_id.clone(),
                provider_instance_id: provider_instance_id.clone(),
                created_at: now,
                updated_at: now,
            };
            repo.create_function_instance(&CreateFunctionInstance::from(function_instance2))
                .await
                .unwrap();

            // Create MCP instance
            let create_request = CreateMcpServerInstanceRequest {
                id: "test-update-conflict".to_string(),
                name: "Test Update Conflict".to_string(),
            };
            create_mcp_server_instance_internal(&tx, &repo, create_request.clone(), false)
                .await
                .unwrap();

            // Add first function
            let add_request1 = AddMcpServerInstanceFunctionRequest {
                function_controller_type_id: function_controller_type_id.clone(),
                provider_controller_type_id: provider_controller_type_id.clone(),
                provider_instance_id: provider_instance_id.clone(),
                function_name: "first_function".to_string(),
                function_description: None,
            };
            add_mcp_server_instance_function_internal(
                &tx,
                &repo,
                &create_request.id,
                add_request1,
                false,
            )
            .await
            .unwrap();

            // Add second function
            let add_request2 = AddMcpServerInstanceFunctionRequest {
                function_controller_type_id: "another_function".to_string(),
                provider_controller_type_id: provider_controller_type_id.clone(),
                provider_instance_id: provider_instance_id.clone(),
                function_name: "second_function".to_string(),
                function_description: None,
            };
            add_mcp_server_instance_function_internal(
                &tx,
                &repo,
                &create_request.id,
                add_request2,
                false,
            )
            .await
            .unwrap();

            // Try to update second function to have the same name as first - should fail
            let update_request = UpdateMcpServerInstanceFunctionRequest {
                function_name: "first_function".to_string(), // Conflict!
                function_description: None,
            };

            let result = update_mcp_server_instance_function_internal(
                &tx,
                &repo,
                &create_request.id,
                "another_function",
                &provider_controller_type_id,
                &provider_instance_id,
                update_request,
                false,
            )
            .await;
            assert!(result.is_err());

            match result {
                Err(CommonError::InvalidRequest { msg, .. }) => {
                    assert!(msg.contains("already exists"));
                }
                other => panic!("Expected InvalidRequest error, got {other:?}"),
            }
        }

        #[tokio::test]
        async fn test_update_mcp_server_instance_function_same_name_allowed() {
            shared::setup_test!();

            let (_db, conn) =
                setup_in_memory_database(
                    vec![crate::repository::Repository::load_sql_migrations()],
                )
                .await
                .unwrap();
            let repo = crate::repository::Repository::new(conn);
            let tx = create_test_channel();

            // Setup function instance
            let (function_controller_type_id, provider_controller_type_id, provider_instance_id) =
                setup_function_instance(&repo).await;

            // Create MCP instance
            let create_request = CreateMcpServerInstanceRequest {
                id: "test-update-same-name".to_string(),
                name: "Test Update Same Name".to_string(),
            };
            create_mcp_server_instance_internal(&tx, &repo, create_request.clone(), false)
                .await
                .unwrap();

            // Add function
            let add_request = AddMcpServerInstanceFunctionRequest {
                function_controller_type_id: function_controller_type_id.clone(),
                provider_controller_type_id: provider_controller_type_id.clone(),
                provider_instance_id: provider_instance_id.clone(),
                function_name: "my_function".to_string(),
                function_description: None,
            };
            add_mcp_server_instance_function_internal(
                &tx,
                &repo,
                &create_request.id,
                add_request,
                false,
            )
            .await
            .unwrap();

            // Update function to have the same name (just updating description) - should work
            let update_request = UpdateMcpServerInstanceFunctionRequest {
                function_name: "my_function".to_string(), // Same name
                function_description: Some("New description".to_string()),
            };

            let result = update_mcp_server_instance_function_internal(
                &tx,
                &repo,
                &create_request.id,
                &function_controller_type_id,
                &provider_controller_type_id,
                &provider_instance_id,
                update_request,
                false,
            )
            .await;
            assert!(result.is_ok());

            let instance = result.unwrap();
            assert_eq!(instance.functions[0].function_name, "my_function");
            assert_eq!(
                instance.functions[0].function_description,
                Some("New description".to_string())
            );
        }

        #[tokio::test]
        async fn test_remove_mcp_server_instance_function() {
            shared::setup_test!();

            let (_db, conn) =
                setup_in_memory_database(
                    vec![crate::repository::Repository::load_sql_migrations()],
                )
                .await
                .unwrap();
            let repo = crate::repository::Repository::new(conn);
            let tx = create_test_channel();

            // Setup function instance
            let (function_controller_type_id, provider_controller_type_id, provider_instance_id) =
                setup_function_instance(&repo).await;

            // Create MCP instance
            let create_request = CreateMcpServerInstanceRequest {
                id: "test-remove-function".to_string(),
                name: "Test Remove Function".to_string(),
            };
            create_mcp_server_instance_internal(&tx, &repo, create_request.clone(), false)
                .await
                .unwrap();

            // Add function
            let add_request = AddMcpServerInstanceFunctionRequest {
                function_controller_type_id: function_controller_type_id.clone(),
                provider_controller_type_id: provider_controller_type_id.clone(),
                provider_instance_id: provider_instance_id.clone(),
                function_name: "to_be_removed".to_string(),
                function_description: None,
            };
            add_mcp_server_instance_function_internal(
                &tx,
                &repo,
                &create_request.id,
                add_request,
                false,
            )
            .await
            .unwrap();

            // Remove function
            let result = remove_mcp_server_instance_function_internal(
                &tx,
                &repo,
                &create_request.id,
                &function_controller_type_id,
                &provider_controller_type_id,
                &provider_instance_id,
                false,
            )
            .await;
            assert!(result.is_ok());

            let instance = result.unwrap();
            assert!(instance.functions.is_empty());
        }

        #[tokio::test]
        async fn test_remove_mcp_server_instance_function_not_found() {
            shared::setup_test!();

            let (_db, conn) =
                setup_in_memory_database(
                    vec![crate::repository::Repository::load_sql_migrations()],
                )
                .await
                .unwrap();
            let repo = crate::repository::Repository::new(conn);
            let tx = create_test_channel();

            // Create MCP instance without any functions
            let create_request = CreateMcpServerInstanceRequest {
                id: "test-remove-not-found".to_string(),
                name: "Test Remove Not Found".to_string(),
            };
            create_mcp_server_instance_internal(&tx, &repo, create_request.clone(), false)
                .await
                .unwrap();

            // Try to remove non-existent function
            let result = remove_mcp_server_instance_function_internal(
                &tx,
                &repo,
                &create_request.id,
                "nonexistent",
                "nonexistent",
                "nonexistent",
                false,
            )
            .await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_delete_mcp_server_instance_cascades_functions() {
            shared::setup_test!();

            let (_db, conn) =
                setup_in_memory_database(
                    vec![crate::repository::Repository::load_sql_migrations()],
                )
                .await
                .unwrap();
            let repo = crate::repository::Repository::new(conn);
            let tx = create_test_channel();

            // Setup function instance
            let (function_controller_type_id, provider_controller_type_id, provider_instance_id) =
                setup_function_instance(&repo).await;

            // Create MCP instance
            let create_request = CreateMcpServerInstanceRequest {
                id: "test-cascade-delete".to_string(),
                name: "Test Cascade Delete".to_string(),
            };
            create_mcp_server_instance_internal(&tx, &repo, create_request.clone(), false)
                .await
                .unwrap();

            // Add function
            let add_request = AddMcpServerInstanceFunctionRequest {
                function_controller_type_id,
                provider_controller_type_id,
                provider_instance_id,
                function_name: "will_be_cascaded".to_string(),
                function_description: None,
            };
            add_mcp_server_instance_function_internal(
                &tx,
                &repo,
                &create_request.id,
                add_request,
                false,
            )
            .await
            .unwrap();

            // Verify function exists
            let instance = get_mcp_server_instance_internal(&repo, &create_request.id)
                .await
                .unwrap();
            assert_eq!(instance.functions.len(), 1);

            // Delete instance - functions should be cascade deleted
            delete_mcp_server_instance_internal(&tx, &repo, &create_request.id, false)
                .await
                .unwrap();

            // Verify instance is gone
            let result = get_mcp_server_instance_internal(&repo, &create_request.id).await;
            assert!(result.is_err());
        }
    }
}
