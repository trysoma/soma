
use serde::{Deserialize, Serialize};
use shared::primitives::WrappedSchema;
use utoipa::ToSchema;

use crate::logic::credential::ConfigurationSchema;

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct ToolGroupCredentialDeploymentSerialized {
    pub type_id: String,
    pub configuration_schema: ConfigurationSchema,
    pub name: String,
    pub documentation: String,
    pub requires_brokering: bool,
    pub requires_resource_server_credential_refreshing: bool,
    pub requires_user_credential_refreshing: bool,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct ToolDeploymentSerialized {
    pub type_id: String,
    pub name: String,
    pub documentation: String,
    pub parameters: WrappedSchema,
    pub output: WrappedSchema,
    pub categories: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct ToolGroupDeploymentSerialized {
    pub type_id: String,
    pub name: String,
    pub categories: Vec<String>,
    pub documentation: String,
    pub tools: Vec<ToolDeploymentSerialized>,
    pub credential_deployments: Vec<ToolGroupCredentialDeploymentSerialized>,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct WithToolDeploymentTypeId<T> {
    pub tool_deployment_type_id: String,
    pub inner: T,
}


#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct WithToolGroupDeploymentTypeId<T> {
    pub tool_group_deployment_type_id: String,
    pub inner: T,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct WithCredentialDeploymentTypeId<T> {
    pub credential_deployment_type_id: String,
    pub inner: T,
}
