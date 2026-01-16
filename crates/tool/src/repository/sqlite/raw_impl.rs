use crate::logic::credential::{
    BrokerAction, BrokerState, ResourceServerCredentialSerialized, UserCredentialSerialized,
};
use crate::logic::instance::{
    ToolInstanceSerialized, ToolInstanceSerializedWithCredentials,
    ToolGroupInstanceSerialized, ToolGroupInstanceSerializedWithCredentials,
    ToolGroupInstanceSerializedWithTools,
};
use crate::logic::mcp_server_instance::{
    McpServerInstanceToolSerialized, McpServerInstanceSerializedWithFunctions,
};
use shared::error::CommonError;

// Import generated Row types from parent module
use super::{
    Row_get_broker_state_by_id, Row_get_tool_by_id,
    Row_get_tool_with_credentials, Row_get_mcp_server_instance_by_id,
    Row_get_tool_group_by_id, Row_get_tool_groups,
    Row_get_tool_groups_grouped_by_tool_deployment_type_id,
    Row_get_resource_server_credential_by_id, Row_get_resource_server_credentials,
    Row_get_user_credential_by_id, Row_get_user_credentials, Row_list_mcp_server_instances,
};

// Helper function to deserialize functions JSON array
fn deserialize_functions(json_value: &str) -> Result<Vec<ToolInstanceSerialized>, CommonError> {
    // Handle null, empty, or "null" string cases
    if json_value.is_empty()
        || json_value == "null"
        || json_value == "[]"
        || json_value.trim().is_empty()
    {
        return Ok(Vec::new());
    }

    serde_json::from_str(json_value).map_err(|e| CommonError::Repository {
        msg: format!("Failed to deserialize functions JSON: {e}"),
        source: Some(e.into()),
    })
}

// Helper function to deserialize resource server credential JSON object
fn deserialize_resource_server_credential(
    json_value: &str,
) -> Result<ResourceServerCredentialSerialized, CommonError> {
    if json_value.is_empty() || json_value == "null" || json_value.trim().is_empty() {
        return Err(CommonError::Repository {
            msg: "Resource server credential is required but was null".to_string(),
            source: None,
        });
    }

    serde_json::from_str(json_value).map_err(|e| CommonError::Repository {
        msg: format!("Failed to deserialize resource server credential JSON: {e}"),
        source: Some(e.into()),
    })
}

// Helper function to deserialize optional user credential JSON object
fn deserialize_user_credential(
    json_value: &str,
) -> Result<Option<UserCredentialSerialized>, CommonError> {
    if json_value.is_empty() || json_value == "null" || json_value.trim().is_empty() {
        return Ok(None);
    }

    let cred: UserCredentialSerialized =
        serde_json::from_str(json_value).map_err(|e| CommonError::Repository {
            msg: format!("Failed to deserialize user credential JSON: {e}"),
            source: Some(e.into()),
        })?;
    Ok(Some(cred))
}

// Implement TryFrom for query result types to domain types

impl TryFrom<Row_get_resource_server_credential_by_id> for ResourceServerCredentialSerialized {
    type Error = CommonError;
    fn try_from(row: Row_get_resource_server_credential_by_id) -> Result<Self, Self::Error> {
        Ok(ResourceServerCredentialSerialized {
            id: row.id,
            type_id: row.type_id,
            metadata: row.metadata,
            value: row.value,
            created_at: row.created_at,
            updated_at: row.updated_at,
            next_rotation_time: row.next_rotation_time,
            dek_alias: row.dek_alias,
        })
    }
}

impl TryFrom<Row_get_user_credential_by_id> for UserCredentialSerialized {
    type Error = CommonError;
    fn try_from(row: Row_get_user_credential_by_id) -> Result<Self, Self::Error> {
        Ok(UserCredentialSerialized {
            id: row.id,
            type_id: row.type_id,
            metadata: row.metadata,
            value: row.value,
            created_at: row.created_at,
            updated_at: row.updated_at,
            next_rotation_time: row.next_rotation_time,
            dek_alias: row.dek_alias,
        })
    }
}

impl TryFrom<Row_get_user_credentials> for UserCredentialSerialized {
    type Error = CommonError;
    fn try_from(row: Row_get_user_credentials) -> Result<Self, Self::Error> {
        Ok(UserCredentialSerialized {
            id: row.id,
            type_id: row.type_id,
            metadata: row.metadata,
            value: row.value,
            created_at: row.created_at,
            updated_at: row.updated_at,
            next_rotation_time: row.next_rotation_time,
            dek_alias: row.dek_alias,
        })
    }
}

impl TryFrom<Row_get_resource_server_credentials> for ResourceServerCredentialSerialized {
    type Error = CommonError;
    fn try_from(row: Row_get_resource_server_credentials) -> Result<Self, Self::Error> {
        Ok(ResourceServerCredentialSerialized {
            id: row.id,
            type_id: row.type_id,
            metadata: row.metadata,
            value: row.value,
            created_at: row.created_at,
            updated_at: row.updated_at,
            next_rotation_time: row.next_rotation_time,
            dek_alias: row.dek_alias,
        })
    }
}

impl TryFrom<Row_get_tool_group_by_id> for ToolGroupInstanceSerialized {
    type Error = CommonError;
    fn try_from(row: Row_get_tool_group_by_id) -> Result<Self, Self::Error> {
        let return_on_successful_brokering = row
            .return_on_successful_brokering
            .as_ref()
            .and_then(|v| serde_json::from_value(v.get_inner().clone()).ok());

        Ok(ToolGroupInstanceSerialized {
            id: row.id,
            display_name: row.display_name,
            resource_server_credential_id: row.resource_server_credential_id,
            user_credential_id: row.user_credential_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
            tool_group_deployment_type_id: row.tool_group_deployment_type_id,
            credential_deployment_type_id: row.credential_deployment_type_id,
            status: row.status,
            return_on_successful_brokering,
        })
    }
}

impl TryFrom<Row_get_tool_groups> for ToolGroupInstanceSerializedWithTools {
    type Error = CommonError;
    fn try_from(row: Row_get_tool_groups) -> Result<Self, Self::Error> {
        let return_on_successful_brokering = row
            .return_on_successful_brokering
            .as_ref()
            .and_then(|v| serde_json::from_value(v.get_inner().clone()).ok());

        let tool_group_instance = ToolGroupInstanceSerialized {
            id: row.id,
            display_name: row.display_name,
            resource_server_credential_id: row.resource_server_credential_id,
            user_credential_id: row.user_credential_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
            tool_group_deployment_type_id: row.tool_group_deployment_type_id,
            credential_deployment_type_id: row.credential_deployment_type_id,
            status: row.status,
            return_on_successful_brokering,
        };

        // TODO: Tools should be fetched separately via get_tool_instances query
        // The tools field was removed from the Row because we no longer store tools as JSON
        let tools = vec![];
        let resource_server_credential =
            deserialize_resource_server_credential(&row.resource_server_credential)?;
        let user_credential = deserialize_user_credential(&row.user_credential)?;

        Ok(ToolGroupInstanceSerializedWithTools {
            tool_group_instance,
            tools,
            resource_server_credential,
            user_credential,
        })
    }
}

impl TryFrom<Row_get_tool_group_by_id> for ToolGroupInstanceSerializedWithTools {
    type Error = CommonError;
    fn try_from(row: Row_get_tool_group_by_id) -> Result<Self, Self::Error> {
        let return_on_successful_brokering = row
            .return_on_successful_brokering
            .as_ref()
            .and_then(|v| serde_json::from_value(v.get_inner().clone()).ok());

        let tool_group_instance = ToolGroupInstanceSerialized {
            id: row.id,
            display_name: row.display_name,
            resource_server_credential_id: row.resource_server_credential_id,
            user_credential_id: row.user_credential_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
            tool_group_deployment_type_id: row.tool_group_deployment_type_id,
            credential_deployment_type_id: row.credential_deployment_type_id,
            status: row.status,
            return_on_successful_brokering,
        };

        // TODO: Tools should be fetched separately via get_tool_instances query
        // The tools field was removed from the Row because we no longer store tools as JSON
        let tools = vec![];
        let resource_server_credential =
            deserialize_resource_server_credential(&row.resource_server_credential)?;
        let user_credential = deserialize_user_credential(&row.user_credential)?;

        Ok(ToolGroupInstanceSerializedWithTools {
            tool_group_instance,
            tools,
            resource_server_credential,
            user_credential,
        })
    }
}

impl TryFrom<Row_get_tool_by_id> for ToolInstanceSerialized {
    type Error = CommonError;
    fn try_from(row: Row_get_tool_by_id) -> Result<Self, Self::Error> {
        Ok(ToolInstanceSerialized {
            tool_deployment_type_id: row.tool_deployment_type_id,
            tool_group_deployment_type_id: row.tool_group_deployment_type_id,
            tool_group_instance_id: row.tool_group_instance_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<Row_get_broker_state_by_id> for BrokerState {
    type Error = CommonError;
    fn try_from(row: Row_get_broker_state_by_id) -> Result<Self, Self::Error> {
        let action: BrokerAction = serde_json::from_value(row.action.get_inner().clone())?;
        Ok(BrokerState {
            id: row.id,
            created_at: row.created_at,
            updated_at: row.updated_at,
            tool_group_instance_id: row.tool_group_instance_id,
            tool_group_deployment_type_id: row.tool_group_deployment_type_id,
            credential_deployment_type_id: row.credential_deployment_type_id,
            metadata: row.metadata,
            action,
        })
    }
}

impl TryFrom<Row_get_tool_with_credentials>
    for ToolInstanceSerializedWithCredentials
{
    type Error = CommonError;
    fn try_from(row: Row_get_tool_with_credentials) -> Result<Self, Self::Error> {
        let provider_return_on_successful_brokering = row
            .tool_group_instance_return_on_successful_brokering
            .as_ref()
            .and_then(|v| serde_json::from_value(v.get_inner().clone()).ok());

        let user_credential = match row.user_credential_id {
            Some(id) => Some(UserCredentialSerialized {
                id,
                type_id: row.user_credential_type_id.clone().unwrap_or_default(),
                metadata: row.user_credential_metadata.clone().unwrap_or_default(),
                value: match row.user_credential_value.clone() {
                    Some(value) => value,
                    None => {
                        return Err(CommonError::Unknown(anyhow::anyhow!(
                            "user credential value is required when user_credential_id is present"
                        )));
                    }
                },
                created_at: match row.user_credential_created_at {
                    Some(created_at) => created_at,
                    None => {
                        return Err(CommonError::Unknown(anyhow::anyhow!(
                            "user credential created at is required when user_credential_id is present"
                        )));
                    }
                },
                updated_at: match row.user_credential_updated_at {
                    Some(updated_at) => updated_at,
                    None => {
                        return Err(CommonError::Unknown(anyhow::anyhow!(
                            "user credential updated at is required when user_credential_id is present"
                        )));
                    }
                },
                next_rotation_time: row.user_credential_next_rotation_time,
                dek_alias: match row.user_credential_dek_alias {
                    Some(dek_alias) => dek_alias,
                    None => {
                        return Err(CommonError::Unknown(anyhow::anyhow!(
                            "user credential data encryption key id is required when user_credential_id is present"
                        )));
                    }
                },
            }),
            None => None,
        };
        Ok(ToolInstanceSerializedWithCredentials {
            tool_instance: ToolInstanceSerialized {
                tool_deployment_type_id: row.tool_instance_tool_deployment_type_id,
                tool_group_deployment_type_id: row
                    .tool_instance_tool_group_deployment_type_id
                    .clone(),
                tool_group_instance_id: row.tool_instance_tool_group_instance_id.clone(),
                created_at: row.tool_instance_created_at,
                updated_at: row.tool_instance_updated_at,
            },
            tool_group_instance: ToolGroupInstanceSerialized {
                id: row.tool_group_instance_id,
                display_name: row.tool_group_instance_display_name,
                resource_server_credential_id: row
                    .tool_group_instance_resource_server_credential_id
                    .clone(),
                user_credential_id: row.tool_group_instance_user_credential_id.clone(),
                created_at: row.tool_group_instance_created_at,
                updated_at: row.tool_group_instance_updated_at,
                tool_group_deployment_type_id: row.tool_group_instance_tool_group_deployment_type_id,
                credential_deployment_type_id: row.credential_deployment_type_id,
                status: row.tool_group_instance_status,
                return_on_successful_brokering: provider_return_on_successful_brokering,
            },
            resource_server_credential: ResourceServerCredentialSerialized {
                id: row.resource_server_credential_id,
                type_id: row.resource_server_credential_type_id,
                metadata: row.resource_server_credential_metadata,
                value: row.resource_server_credential_value,
                created_at: row.resource_server_credential_created_at,
                updated_at: row.resource_server_credential_updated_at,
                next_rotation_time: row.resource_server_credential_next_rotation_time,
                dek_alias: row.resource_server_credential_dek_alias,
            },
            user_credential,
        })
    }
}

// Helper struct to deserialize provider instance from grouped query JSON
#[derive(serde::Deserialize, Debug)]
struct ProviderInstanceFromGroupedQuery {
    id: String,
    display_name: String,
    tool_group_deployment_type_id: String,
    credential_deployment_type_id: String,
    status: String,
    return_on_successful_brokering: Option<serde_json::Value>,
    created_at: String,
    updated_at: String,
    resource_server_credential: serde_json::Value,
    user_credential: serde_json::Value,
}

impl TryFrom<Row_get_tool_groups_grouped_by_tool_deployment_type_id>
    for crate::repository::ToolGroupInstancesGroupedByToolSourceTypeId
{
    type Error = CommonError;
    fn try_from(
        row: Row_get_tool_groups_grouped_by_tool_deployment_type_id,
    ) -> Result<Self, Self::Error> {
        // Parse the JSON array of provider instances
        let tool_group_instances_json: Vec<ProviderInstanceFromGroupedQuery> =
            serde_json::from_str(&row.tool_group_instances).map_err(|e| CommonError::Repository {
                msg: format!("Failed to deserialize tool_group_instances JSON: {e}"),
                source: Some(e.into()),
            })?;

        let tool_group_instances: Vec<ToolGroupInstanceSerializedWithCredentials> =
            tool_group_instances_json
                .into_iter()
                .map(|pi_json| {
                    // Parse return_on_successful_brokering
                    let return_on_successful_brokering = pi_json
                        .return_on_successful_brokering
                        .as_ref()
                        .and_then(|v| {
                            if v.is_null() {
                                None
                            } else {
                                serde_json::from_value(v.clone()).ok()
                            }
                        });

                    // Parse resource_server_credential
                    let resource_server_credential: ResourceServerCredentialSerialized =
                        serde_json::from_value(pi_json.resource_server_credential).map_err(
                            |e| CommonError::Repository {
                                msg: format!(
                                    "Failed to deserialize resource_server_credential: {e}"
                                ),
                                source: Some(e.into()),
                            },
                        )?;

                    // Parse user_credential (may be null)
                    let user_credential: Option<UserCredentialSerialized> =
                        if pi_json.user_credential.is_null() {
                            None
                        } else {
                            Some(
                                serde_json::from_value(pi_json.user_credential).map_err(|e| {
                                    CommonError::Repository {
                                        msg: format!("Failed to deserialize user_credential: {e}"),
                                        source: Some(e.into()),
                                    }
                                })?,
                            )
                        };

                    // Get resource_server_credential_id and user_credential_id from the deserialized credentials
                    let resource_server_credential_id = resource_server_credential.id.clone();
                    let user_credential_id = user_credential.as_ref().map(|uc| uc.id.clone());

                    // Parse timestamps
                    let created_at = shared::primitives::WrappedChronoDateTime::try_from(
                        pi_json.created_at.as_str(),
                    )
                    .map_err(|e| CommonError::Repository {
                        msg: format!("Failed to parse created_at: {e}"),
                        source: Some(e.into()),
                    })?;

                    let updated_at = shared::primitives::WrappedChronoDateTime::try_from(
                        pi_json.updated_at.as_str(),
                    )
                    .map_err(|e| CommonError::Repository {
                        msg: format!("Failed to parse updated_at: {e}"),
                        source: Some(e.into()),
                    })?;

                    Ok(ToolGroupInstanceSerializedWithCredentials {
                        tool_group_instance: ToolGroupInstanceSerialized {
                            id: pi_json.id,
                            display_name: pi_json.display_name,
                            resource_server_credential_id,
                            user_credential_id,
                            created_at,
                            updated_at,
                            tool_group_deployment_type_id: pi_json.tool_group_deployment_type_id,
                            credential_deployment_type_id: pi_json.credential_deployment_type_id,
                            status: pi_json.status,
                            return_on_successful_brokering,
                        },
                        resource_server_credential,
                        user_credential,
                    })
                })
                .collect::<Result<Vec<_>, CommonError>>()?;

        Ok(
            crate::repository::ToolGroupInstancesGroupedByToolSourceTypeId {
                tool_deployment_type_id: row.tool_deployment_type_id,
                tool_group_instances,
            },
        )
    }
}

impl TryFrom<super::Row_get_tool_groups_with_credentials>
    for ToolGroupInstanceSerializedWithCredentials
{
    type Error = CommonError;

    fn try_from(
        row: super::Row_get_tool_groups_with_credentials,
    ) -> Result<Self, Self::Error> {
        // Parse resource server credential from JSON string
        let resource_server_credential: ResourceServerCredentialSerialized =
            serde_json::from_str(&row.resource_server_credential).map_err(|e| {
                CommonError::Repository {
                    msg: format!("Failed to parse resource_server_credential JSON: {e}"),
                    source: Some(e.into()),
                }
            })?;

        // Parse optional user credential from JSON string
        let user_credential: Option<UserCredentialSerialized> = if row.user_credential == "null" {
            None
        } else {
            Some(serde_json::from_str(&row.user_credential).map_err(|e| {
                CommonError::Repository {
                    msg: format!("Failed to parse user_credential JSON: {e}"),
                    source: Some(e.into()),
                }
            })?)
        };

        Ok(ToolGroupInstanceSerializedWithCredentials {
            tool_group_instance: ToolGroupInstanceSerialized {
                id: row.id,
                display_name: row.display_name,
                resource_server_credential_id: resource_server_credential.id.clone(),
                user_credential_id: user_credential.as_ref().map(|uc| uc.id.clone()),
                created_at: row.created_at,
                updated_at: row.updated_at,
                tool_group_deployment_type_id: row.tool_group_deployment_type_id,
                credential_deployment_type_id: row.credential_deployment_type_id,
                status: row.status,
                return_on_successful_brokering: row
                    .return_on_successful_brokering
                    .and_then(|v| serde_json::from_value(v.into_inner()).ok()),
            },
            resource_server_credential,
            user_credential,
        })
    }
}

/// Helper function to deserialize MCP server instance functions from JSON array
fn deserialize_mcp_server_instance_functions(
    json_value: &str,
) -> Result<Vec<McpServerInstanceToolSerialized>, CommonError> {
    // Handle null, empty, or "null" string cases
    if json_value.is_empty()
        || json_value == "null"
        || json_value == "[]"
        || json_value.trim().is_empty()
    {
        return Ok(Vec::new());
    }

    serde_json::from_str(json_value).map_err(|e| CommonError::Repository {
        msg: format!("Failed to deserialize MCP server instance functions JSON: {e}"),
        source: Some(e.into()),
    })
}

impl TryFrom<Row_get_mcp_server_instance_by_id> for McpServerInstanceSerializedWithFunctions {
    type Error = CommonError;

    fn try_from(row: Row_get_mcp_server_instance_by_id) -> Result<Self, Self::Error> {
        let tools = deserialize_mcp_server_instance_functions(&row.tools)?;

        Ok(McpServerInstanceSerializedWithFunctions {
            id: row.id,
            name: row.name,
            created_at: row.created_at,
            updated_at: row.updated_at,
            tools,
        })
    }
}

impl TryFrom<Row_list_mcp_server_instances> for McpServerInstanceSerializedWithFunctions {
    type Error = CommonError;

    fn try_from(row: Row_list_mcp_server_instances) -> Result<Self, Self::Error> {
        let tools = deserialize_mcp_server_instance_functions(&row.tools)?;

        Ok(McpServerInstanceSerializedWithFunctions {
            id: row.id,
            name: row.name,
            created_at: row.created_at,
            updated_at: row.updated_at,
            tools,
        })
    }
}

impl TryFrom<super::Row_get_mcp_server_instance_tool_by_name>
    for McpServerInstanceToolSerialized
{
    type Error = CommonError;

    fn try_from(
        row: super::Row_get_mcp_server_instance_tool_by_name,
    ) -> Result<Self, Self::Error> {
        Ok(McpServerInstanceToolSerialized {
            mcp_server_instance_id: row.mcp_server_instance_id,
            tool_deployment_type_id: row.tool_deployment_type_id,
            tool_group_deployment_type_id: row.tool_group_deployment_type_id,
            tool_group_instance_id: row.tool_group_instance_id,
            tool_name: row.tool_name,
            tool_description: row.tool_description,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<super::Row_list_mcp_server_instance_tools>
    for McpServerInstanceToolSerialized
{
    type Error = CommonError;

    fn try_from(row: super::Row_list_mcp_server_instance_tools) -> Result<Self, Self::Error> {
        Ok(McpServerInstanceToolSerialized {
            mcp_server_instance_id: row.mcp_server_instance_id,
            tool_deployment_type_id: row.tool_deployment_type_id,
            tool_group_deployment_type_id: row.tool_group_deployment_type_id,
            tool_group_instance_id: row.tool_group_instance_id,
            tool_name: row.tool_name,
            tool_description: row.tool_description,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

// Tool TryFrom implementations
impl TryFrom<super::Row_get_tool_by_id> for crate::logic::ToolGroupDeploymentSerialized {
    type Error = CommonError;

    fn try_from(row: super::Row_get_tool_by_id) -> Result<Self, Self::Error> {
        let categories: Vec<String> = serde_json::from_value(row.categories.get_inner().clone())?;
        Ok(crate::logic::ToolGroupDeploymentSerialized {
            type_id: row.type_id,
            deployment_id: row.deployment_id,
            name: row.name,
            documentation: row.documentation,
            categories,
            endpoint_type: row.endpoint_type,
            endpoint_configuration: row.endpoint_configuration,
            metadata: row.metadata,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<super::Row_list_tool_group_deployments> for crate::logic::ToolGroupDeploymentSerialized {
    type Error = CommonError;

    fn try_from(row: super::Row_list_tool_group_deployments) -> Result<Self, Self::Error> {
        let categories: Vec<String> = serde_json::from_value(row.categories.get_inner().clone())?;
        Ok(crate::logic::ToolGroupDeploymentSerialized {
            type_id: row.type_id,
            deployment_id: row.deployment_id,
            name: row.name,
            documentation: row.documentation,
            categories,
            endpoint_type: row.endpoint_type,
            endpoint_configuration: row.endpoint_configuration,
            metadata: row.metadata,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<super::Row_list_tool_group_deployments_by_category> for crate::logic::ToolGroupDeploymentSerialized {
    type Error = CommonError;

    fn try_from(row: super::Row_list_tool_group_deployments_by_category) -> Result<Self, Self::Error> {
        let categories: Vec<String> = serde_json::from_value(row.categories.get_inner().clone())?;
        Ok(crate::logic::ToolGroupDeploymentSerialized {
            type_id: row.type_id,
            deployment_id: row.deployment_id,
            name: row.name,
            documentation: row.documentation,
            categories,
            endpoint_type: row.endpoint_type,
            endpoint_configuration: row.endpoint_configuration,
            metadata: row.metadata,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<super::Row_get_tool_group_deployment_by_alias> for crate::logic::ToolGroupDeploymentSerialized {
    type Error = CommonError;

    fn try_from(row: super::Row_get_tool_group_deployment_by_alias) -> Result<Self, Self::Error> {
        let categories: Vec<String> = serde_json::from_value(row.categories.get_inner().clone())?;
        Ok(crate::logic::ToolGroupDeploymentSerialized {
            type_id: row.type_id,
            deployment_id: row.deployment_id,
            name: row.name,
            documentation: row.documentation,
            categories,
            endpoint_type: row.endpoint_type,
            endpoint_configuration: row.endpoint_configuration,
            metadata: row.metadata,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<super::Row_list_tool_group_deployment_aliases> for crate::logic::ToolGroupDeploymentAliasSerialized {
    type Error = CommonError;

    fn try_from(row: super::Row_list_tool_group_deployment_aliases) -> Result<Self, Self::Error> {
        Ok(crate::logic::ToolGroupDeploymentAliasSerialized {
            tool_type_id: row.tool_type_id,
            tool_deployment_id: row.tool_deployment_id,
            alias: row.alias,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
