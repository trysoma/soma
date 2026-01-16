mod sqlite;

use shared::{
    error::CommonError,
    primitives::{
        PaginatedResponse, PaginationRequest, WrappedChronoDateTime, WrappedJsonValue,
        WrappedUuidV4,
    },
};

#[allow(unused_imports)]
pub use sqlite::Repository;

use crate::logic::Metadata;
use crate::logic::credential::{
    BrokerState, ResourceServerCredentialSerialized, UserCredentialSerialized,
};
use crate::logic::instance::{
    ToolInstanceSerialized, ToolInstanceSerializedWithCredentials,
    ToolGroupInstanceSerialized, ToolGroupInstanceSerializedWithCredentials,
    ToolGroupInstanceSerializedWithTools,
};
use crate::logic::mcp_server_instance::{
    McpServerInstanceToolSerialized, McpServerInstanceSerialized,
    McpServerInstanceSerializedWithFunctions,
};

// Repository parameter structs for resource server credentials
#[derive(Debug)]
pub struct CreateResourceServerCredential {
    pub id: WrappedUuidV4,
    pub type_id: String,
    pub metadata: Metadata,
    pub value: WrappedJsonValue,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
    pub next_rotation_time: Option<WrappedChronoDateTime>,
    pub dek_alias: String,
}

impl From<ResourceServerCredentialSerialized> for CreateResourceServerCredential {
    fn from(cred: ResourceServerCredentialSerialized) -> Self {
        CreateResourceServerCredential {
            id: cred.id,
            type_id: cred.type_id,
            metadata: cred.metadata,
            value: cred.value,
            created_at: cred.created_at,
            updated_at: cred.updated_at,
            next_rotation_time: cred.next_rotation_time,
            dek_alias: cred.dek_alias,
        }
    }
}

// Repository parameter structs for user credentials
#[derive(Debug)]
pub struct CreateUserCredential {
    pub id: WrappedUuidV4,
    pub type_id: String,
    pub metadata: Metadata,
    pub value: WrappedJsonValue,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
    pub next_rotation_time: Option<WrappedChronoDateTime>,
    pub dek_alias: String,
}

impl From<UserCredentialSerialized> for CreateUserCredential {
    fn from(cred: UserCredentialSerialized) -> Self {
        CreateUserCredential {
            id: cred.id,
            type_id: cred.type_id,
            metadata: cred.metadata,
            value: cred.value,
            created_at: cred.created_at,
            updated_at: cred.updated_at,
            next_rotation_time: cred.next_rotation_time,
            dek_alias: cred.dek_alias,
        }
    }
}

// Repository parameter structs for tool group instances
#[derive(Debug)]
pub struct CreateToolGroup {
    pub id: String,
    pub display_name: String,
    pub resource_server_credential_id: WrappedUuidV4,
    pub user_credential_id: Option<WrappedUuidV4>,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
    pub tool_group_deployment_type_id: String,
    pub credential_deployment_type_id: String,
    pub status: String,
    pub return_on_successful_brokering: Option<crate::logic::ReturnAddress>,
}

impl From<ToolGroupInstanceSerialized> for CreateToolGroup {
    fn from(pi: ToolGroupInstanceSerialized) -> Self {
        CreateToolGroup {
            id: pi.id,
            display_name: pi.display_name,
            resource_server_credential_id: pi.resource_server_credential_id,
            user_credential_id: pi.user_credential_id,
            created_at: pi.created_at,
            updated_at: pi.updated_at,
            tool_group_deployment_type_id: pi.tool_group_deployment_type_id,
            credential_deployment_type_id: pi.credential_deployment_type_id,
            status: pi.status,
            return_on_successful_brokering: pi.return_on_successful_brokering,
        }
    }
}

// Repository parameter structs for tool instances
#[derive(Debug)]
pub struct CreateTool {
    pub tool_deployment_type_id: String,
    pub tool_group_deployment_type_id: String,
    pub tool_group_id: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

impl From<ToolInstanceSerialized> for CreateTool {
    fn from(fi: ToolInstanceSerialized) -> Self {
        CreateTool {
            tool_deployment_type_id: fi.tool_deployment_type_id,
            tool_group_deployment_type_id: fi.tool_group_deployment_type_id,
            tool_group_id: fi.tool_group_id,
            created_at: fi.created_at,
            updated_at: fi.updated_at,
        }
    }
}

// Repository parameter structs for broker state
#[derive(Debug)]
pub struct CreateBrokerState {
    pub id: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
    pub tool_group_id: String,
    pub tool_group_deployment_type_id: String,
    pub credential_deployment_type_id: String,
    pub metadata: Metadata,
    pub action: WrappedJsonValue,
}

impl From<BrokerState> for CreateBrokerState {
    fn from(bs: BrokerState) -> Self {
        let action_json = serde_json::to_value(&bs.action).unwrap_or(serde_json::json!({}));
        CreateBrokerState {
            id: bs.id,
            created_at: bs.created_at,
            updated_at: bs.updated_at,
            tool_group_id: bs.tool_group_id,
            tool_group_deployment_type_id: bs.tool_group_deployment_type_id,
            credential_deployment_type_id: bs.credential_deployment_type_id,
            metadata: bs.metadata,
            action: WrappedJsonValue::new(action_json),
        }
    }
}

// Repository return struct for grouped tool group instances
#[derive(Debug)]
pub struct ToolGroupInstancesGroupedByToolSourceTypeId {
    pub tool_deployment_type_id: String,
    pub tool_group_instances: Vec<ToolGroupInstanceSerializedWithCredentials>,
}

// Repository parameter structs for MCP server instance
#[derive(Debug)]
pub struct CreateMcpServerInstance {
    pub id: String,
    pub name: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

impl From<McpServerInstanceSerialized> for CreateMcpServerInstance {
    fn from(msi: McpServerInstanceSerialized) -> Self {
        CreateMcpServerInstance {
            id: msi.id,
            name: msi.name,
            created_at: msi.created_at,
            updated_at: msi.updated_at,
        }
    }
}

// Repository parameter structs for MCP server instance function
#[derive(Debug)]
pub struct CreateMcpServerInstanceFunction {
    pub mcp_server_instance_id: String,
    pub tool_deployment_type_id: String,
    pub tool_group_deployment_type_id: String,
    pub tool_group_id: String,
    pub tool_name: String,
    pub tool_description: Option<String>,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

impl From<McpServerInstanceToolSerialized> for CreateMcpServerInstanceFunction {
    fn from(msif: McpServerInstanceToolSerialized) -> Self {
        CreateMcpServerInstanceFunction {
            mcp_server_instance_id: msif.mcp_server_instance_id,
            tool_deployment_type_id: msif.tool_deployment_type_id,
            tool_group_deployment_type_id: msif.tool_group_deployment_type_id,
            tool_group_id: msif.tool_group_id,
            tool_name: msif.tool_name,
            tool_description: msif.tool_description,
            created_at: msif.created_at,
            updated_at: msif.updated_at,
        }
    }
}

// Repository parameter structs for updating MCP server instance function
#[derive(Debug)]
pub struct UpdateMcpServerInstanceFunction {
    pub mcp_server_instance_id: String,
    pub tool_deployment_type_id: String,
    pub tool_group_deployment_type_id: String,
    pub tool_group_id: String,
    pub tool_name: String,
    pub tool_description: Option<String>,
}

// Repository trait
#[allow(async_fn_in_trait)]
pub trait ProviderRepositoryLike {
    async fn create_resource_server_credential(
        &self,
        params: &CreateResourceServerCredential,
    ) -> Result<(), CommonError>;

    async fn get_resource_server_credential_by_id(
        &self,
        id: &WrappedUuidV4,
    ) -> Result<Option<ResourceServerCredentialSerialized>, CommonError>;

    async fn create_user_credential(
        &self,
        params: &CreateUserCredential,
    ) -> Result<(), CommonError>;

    async fn get_user_credential_by_id(
        &self,
        id: &WrappedUuidV4,
    ) -> Result<Option<UserCredentialSerialized>, CommonError>;

    async fn delete_user_credential(&self, id: &WrappedUuidV4) -> Result<(), CommonError>;

    async fn delete_resource_server_credential(
        &self,
        id: &WrappedUuidV4,
    ) -> Result<(), CommonError>;

    async fn list_user_credentials(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<UserCredentialSerialized>, CommonError>;

    async fn list_resource_server_credentials(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<ResourceServerCredentialSerialized>, CommonError>;

    async fn create_tool_group(
        &self,
        params: &CreateToolGroup,
    ) -> Result<(), CommonError>;

    async fn get_tool_group_by_id(
        &self,
        id: &str,
    ) -> Result<Option<ToolGroupInstanceSerializedWithTools>, CommonError>;

    async fn update_tool_group(
        &self,
        id: &str,
        display_name: &str,
    ) -> Result<(), CommonError>;

    async fn update_tool_group_after_brokering(
        &self,
        id: &str,
        user_credential_id: &WrappedUuidV4,
    ) -> Result<(), CommonError>;

    async fn delete_tool_group(&self, id: &str) -> Result<(), CommonError>;

    async fn create_tool(
        &self,
        params: &CreateTool,
    ) -> Result<(), CommonError>;

    async fn get_tool_by_id(
        &self,
        tool_deployment_type_id: &str,
        tool_group_deployment_type_id: &str,
        tool_group_id: &str,
    ) -> Result<Option<ToolInstanceSerialized>, CommonError>;

    async fn delete_tool(
        &self,
        tool_deployment_type_id: &str,
        tool_group_deployment_type_id: &str,
        tool_group_id: &str,
    ) -> Result<(), CommonError>;

    async fn get_tool_with_credentials(
        &self,
        tool_deployment_type_id: &str,
        tool_group_deployment_type_id: &str,
        tool_group_id: &str,
    ) -> Result<Option<ToolInstanceSerializedWithCredentials>, CommonError>;

    async fn create_broker_state(&self, params: &CreateBrokerState) -> Result<(), CommonError>;

    async fn get_broker_state_by_id(&self, id: &str) -> Result<Option<BrokerState>, CommonError>;

    async fn delete_broker_state(&self, id: &str) -> Result<(), CommonError>;

    async fn list_tool_groups(
        &self,
        pagination: &PaginationRequest,
        status: Option<&str>,
        tool_group_deployment_type_id: Option<&str>,
    ) -> Result<PaginatedResponse<ToolGroupInstanceSerializedWithTools>, CommonError>;

    async fn list_tool_group_deployments(
        &self,
        pagination: &PaginationRequest,
        tool_group_id: Option<&str>,
    ) -> Result<PaginatedResponse<ToolInstanceSerialized>, CommonError>;

    async fn get_tool_groups_grouped_by_tool_deployment_type_id(
        &self,
        tool_deployment_type_ids: &[String],
    ) -> Result<Vec<ToolGroupInstancesGroupedByToolSourceTypeId>, CommonError>;

    async fn update_resource_server_credential(
        &self,
        id: &WrappedUuidV4,
        value: Option<&WrappedJsonValue>,
        metadata: Option<&crate::logic::Metadata>,
        next_rotation_time: Option<&WrappedChronoDateTime>,
        updated_at: Option<&WrappedChronoDateTime>,
    ) -> Result<(), CommonError>;

    async fn update_user_credential(
        &self,
        id: &WrappedUuidV4,
        value: Option<&WrappedJsonValue>,
        metadata: Option<&crate::logic::Metadata>,
        next_rotation_time: Option<&WrappedChronoDateTime>,
        updated_at: Option<&WrappedChronoDateTime>,
    ) -> Result<(), CommonError>;

    async fn list_tool_groups_with_credentials(
        &self,
        pagination: &PaginationRequest,
        status: Option<&str>,
        rotation_window_end: Option<&WrappedChronoDateTime>,
    ) -> Result<PaginatedResponse<ToolGroupInstanceSerializedWithCredentials>, CommonError>;

    // MCP server instance methods
    async fn create_mcp_server_instance(
        &self,
        params: &CreateMcpServerInstance,
    ) -> Result<(), CommonError>;

    async fn get_mcp_server_instance_by_id(
        &self,
        id: &str,
    ) -> Result<Option<McpServerInstanceSerializedWithFunctions>, CommonError>;

    async fn update_mcp_server_instance(&self, id: &str, name: &str) -> Result<(), CommonError>;

    async fn delete_mcp_server_instance(&self, id: &str) -> Result<(), CommonError>;

    async fn list_mcp_server_instances(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<McpServerInstanceSerializedWithFunctions>, CommonError>;

    async fn create_mcp_server_instance_tool(
        &self,
        params: &CreateMcpServerInstanceFunction,
    ) -> Result<(), CommonError>;

    async fn delete_mcp_server_instance_tool(
        &self,
        mcp_server_instance_id: &str,
        tool_deployment_type_id: &str,
        tool_group_deployment_type_id: &str,
        tool_group_id: &str,
    ) -> Result<(), CommonError>;

    async fn delete_all_mcp_server_instance_tools(
        &self,
        mcp_server_instance_id: &str,
    ) -> Result<(), CommonError>;

    async fn update_mcp_server_instance_tool(
        &self,
        params: &UpdateMcpServerInstanceFunction,
    ) -> Result<(), CommonError>;

    async fn get_mcp_server_instance_tool_by_name(
        &self,
        mcp_server_instance_id: &str,
        tool_name: &str,
    ) -> Result<Option<McpServerInstanceToolSerialized>, CommonError>;

    /// List all functions for a specific MCP server instance with pagination
    async fn list_mcp_server_instance_tools(
        &self,
        mcp_server_instance_id: &str,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<McpServerInstanceToolSerialized>, CommonError>;

    // Tool registry methods
    async fn create_tool(&self, params: &CreateTool) -> Result<(), CommonError>;

    async fn get_tool_by_id(
        &self,
        type_id: &str,
        deployment_id: &str,
    ) -> Result<Option<ToolGroupDeploymentSerialized>, CommonError>;

    async fn delete_tool(&self, type_id: &str, deployment_id: &str) -> Result<(), CommonError>;

    async fn list_tool_group_deployments(
        &self,
        pagination: &PaginationRequest,
        endpoint_type: Option<&str>,
    ) -> Result<PaginatedResponse<ToolGroupDeploymentSerialized>, CommonError>;

    async fn list_tool_group_deployments_by_category(
        &self,
        category: &str,
        pagination: &PaginationRequest,
        endpoint_type: Option<&str>,
    ) -> Result<PaginatedResponse<ToolGroupDeploymentSerialized>, CommonError>;

    async fn create_tool_group_deployment_alias(&self, params: &CreateToolGroupDeploymentAlias) -> Result<(), CommonError>;

    async fn get_tool_group_deployment_by_alias(&self, alias: &str) -> Result<Option<ToolGroupDeploymentSerialized>, CommonError>;

    async fn delete_tool_group_deployment_alias(&self, alias: &str) -> Result<(), CommonError>;

    async fn list_tool_group_deployment_aliases(
        &self,
        pagination: &PaginationRequest,
        tool_type_id: Option<&str>,
        tool_deployment_id: Option<&str>,
    ) -> Result<PaginatedResponse<ToolGroupDeploymentAliasSerialized>, CommonError>;

    async fn update_tool_group_deployment_alias(
        &self,
        tool_type_id: &str,
        alias: &str,
        new_deployment_id: &str,
    ) -> Result<(), CommonError>;
}

// Repository parameter structs for tool registration

use crate::logic::{EndpointType, ToolGroupDeploymentAliasSerialized, ToolGroupDeploymentSerialized};

#[derive(Debug)]
pub struct CreateTool {
    pub type_id: String,
    pub deployment_id: String,
    pub name: String,
    pub documentation: String,
    pub categories: WrappedJsonValue,
    pub endpoint_type: EndpointType,
    pub endpoint_configuration: WrappedJsonValue,
    pub metadata: Metadata,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

impl From<ToolGroupDeploymentSerialized> for CreateTool {
    fn from(tool: ToolGroupDeploymentSerialized) -> Self {
        let categories_json = serde_json::to_value(&tool.categories).unwrap_or(serde_json::json!([]));
        CreateTool {
            type_id: tool.type_id,
            deployment_id: tool.deployment_id,
            name: tool.name,
            documentation: tool.documentation,
            categories: WrappedJsonValue::new(categories_json),
            endpoint_type: tool.endpoint_type,
            endpoint_configuration: tool.endpoint_configuration,
            metadata: tool.metadata,
            created_at: tool.created_at,
            updated_at: tool.updated_at,
        }
    }
}

#[derive(Debug)]
pub struct CreateToolGroupDeploymentAlias {
    pub tool_type_id: String,
    pub tool_deployment_id: String,
    pub alias: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

impl From<ToolGroupDeploymentAliasSerialized> for CreateToolGroupDeploymentAlias {
    fn from(alias: ToolGroupDeploymentAliasSerialized) -> Self {
        CreateToolGroupDeploymentAlias {
            tool_type_id: alias.tool_type_id,
            tool_deployment_id: alias.tool_deployment_id,
            alias: alias.alias,
            created_at: alias.created_at,
            updated_at: alias.updated_at,
        }
    }
}

/// Extended repository trait for tool registration and management
#[allow(async_fn_in_trait)]
pub trait ToolRepositoryLike: ProviderRepositoryLike {
    // Tool CRUD operations
    async fn create_tool(&self, params: &CreateTool) -> Result<(), CommonError>;

    async fn get_tool_by_id(
        &self,
        type_id: &str,
        deployment_id: &str,
    ) -> Result<Option<ToolGroupDeploymentSerialized>, CommonError>;

    async fn delete_tool(&self, type_id: &str, deployment_id: &str) -> Result<(), CommonError>;

    async fn list_tool_group_deployments(
        &self,
        pagination: &PaginationRequest,
        endpoint_type: Option<&str>,
    ) -> Result<PaginatedResponse<ToolGroupDeploymentSerialized>, CommonError>;

    async fn list_tool_group_deployments_by_category(
        &self,
        pagination: &PaginationRequest,
        category: &str,
        endpoint_type: Option<&str>,
    ) -> Result<PaginatedResponse<ToolGroupDeploymentSerialized>, CommonError>;

    // Tool alias operations
    async fn create_tool_group_deployment_alias(&self, params: &CreateToolGroupDeploymentAlias) -> Result<(), CommonError>;

    async fn get_tool_group_deployment_by_alias(&self, alias: &str) -> Result<Option<ToolGroupDeploymentSerialized>, CommonError>;

    async fn delete_tool_group_deployment_alias(&self, alias: &str) -> Result<(), CommonError>;

    async fn list_tool_group_deployment_aliases(
        &self,
        pagination: &PaginationRequest,
        tool_type_id: Option<&str>,
        tool_deployment_id: Option<&str>,
    ) -> Result<PaginatedResponse<ToolGroupDeploymentAliasSerialized>, CommonError>;

    async fn get_aliases_for_tool(
        &self,
        type_id: &str,
        deployment_id: &str,
    ) -> Result<Vec<String>, CommonError>;

    async fn update_tool_group_deployment_alias(
        &self,
        tool_type_id: &str,
        alias: &str,
        new_deployment_id: &str,
    ) -> Result<(), CommonError>;
}
