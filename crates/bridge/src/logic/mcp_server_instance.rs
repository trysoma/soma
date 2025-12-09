use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shared::error::CommonError;
use shared::primitives::{PaginatedResponse, PaginationRequest, WrappedChronoDateTime};
use utoipa::{IntoParams, ToSchema};

use crate::repository::{
    CreateMcpServerInstance, CreateMcpServerInstanceFunction, ProviderRepositoryLike,
    UpdateMcpServerInstanceFunction,
};

/// Extension data injected into MCP request context to identify the MCP server instance
#[derive(Clone, Debug)]
pub struct McpServiceInstanceExt {
    pub mcp_server_instance_id: String,
}

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

/// Creates a new MCP server instance
pub async fn create_mcp_server_instance<R: ProviderRepositoryLike>(
    repository: &R,
    request: CreateMcpServerInstanceRequest,
) -> Result<CreateMcpServerInstanceResponse, CommonError> {
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

    Ok(instance)
}

/// Gets an MCP server instance by ID
pub async fn get_mcp_server_instance<R: ProviderRepositoryLike>(
    repository: &R,
    id: &str,
) -> Result<GetMcpServerInstanceResponse, CommonError> {
    let instance = repository
        .get_mcp_server_instance_by_id(id)
        .await?
        .ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!("MCP server instance not found: {id}"))
        })?;

    Ok(instance)
}

/// Updates an MCP server instance name
pub async fn update_mcp_server_instance<R: ProviderRepositoryLike>(
    repository: &R,
    id: &str,
    request: UpdateMcpServerInstanceRequest,
) -> Result<UpdateMcpServerInstanceResponse, CommonError> {
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

    Ok(instance)
}

/// Deletes an MCP server instance
pub async fn delete_mcp_server_instance<R: ProviderRepositoryLike>(
    repository: &R,
    id: &str,
) -> Result<(), CommonError> {
    // Verify the instance exists first
    let _ = repository
        .get_mcp_server_instance_by_id(id)
        .await?
        .ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!("MCP server instance not found: {id}"))
        })?;

    repository.delete_mcp_server_instance(id).await?;

    Ok(())
}

/// Lists all MCP server instances with pagination
pub async fn list_mcp_server_instances<R: ProviderRepositoryLike>(
    repository: &R,
    params: ListMcpServerInstancesParams,
) -> Result<ListMcpServerInstancesResponse, CommonError> {
    let pagination = PaginationRequest {
        page_size: params.page_size,
        next_page_token: params.next_page_token,
    };

    let result = repository.list_mcp_server_instances(&pagination).await?;

    Ok(result)
}

/// Adds a function to an MCP server instance
pub async fn add_mcp_server_instance_function<R: ProviderRepositoryLike>(
    repository: &R,
    mcp_server_instance_id: &str,
    request: AddMcpServerInstanceFunctionRequest,
) -> Result<AddMcpServerInstanceFunctionResponse, CommonError> {
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
        function_controller_type_id: function_instance.function_controller_type_id,
        provider_controller_type_id: function_instance.provider_controller_type_id,
        provider_instance_id: function_instance.provider_instance_id,
        function_name: request.function_name,
        function_description: request.function_description,
        created_at: now,
        updated_at: now,
    };

    repository
        .create_mcp_server_instance_function(&create_params)
        .await?;

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
pub async fn update_mcp_server_instance_function<R: ProviderRepositoryLike>(
    repository: &R,
    mcp_server_instance_id: &str,
    function_controller_type_id: &str,
    provider_controller_type_id: &str,
    provider_instance_id: &str,
    request: UpdateMcpServerInstanceFunctionRequest,
) -> Result<UpdateMcpServerInstanceFunctionResponse, CommonError> {
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
        function_name: request.function_name,
        function_description: request.function_description,
    };

    repository
        .update_mcp_server_instance_function(&update_params)
        .await?;

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
pub async fn remove_mcp_server_instance_function<R: ProviderRepositoryLike>(
    repository: &R,
    mcp_server_instance_id: &str,
    function_controller_type_id: &str,
    provider_controller_type_id: &str,
    provider_instance_id: &str,
) -> Result<RemoveMcpServerInstanceFunctionResponse, CommonError> {
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
