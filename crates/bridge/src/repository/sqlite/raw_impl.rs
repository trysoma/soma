use crate::logic::credential::{
    BrokerAction, BrokerState, ResourceServerCredentialSerialized, UserCredentialSerialized,
};
use crate::logic::instance::{
    FunctionInstanceSerialized, FunctionInstanceSerializedWithCredentials,
    ProviderInstanceSerialized, ProviderInstanceSerializedWithCredentials,
    ProviderInstanceSerializedWithFunctions,
};
use shared::error::CommonError;

// Import generated Row types from parent module
use super::{
    Row_get_broker_state_by_id, Row_get_function_instance_by_id,
    Row_get_function_instance_with_credentials, Row_get_provider_instance_by_id,
    Row_get_provider_instances, Row_get_provider_instances_grouped_by_function_controller_type_id,
    Row_get_resource_server_credential_by_id, Row_get_resource_server_credentials,
    Row_get_user_credential_by_id, Row_get_user_credentials,
};

// Helper function to deserialize functions JSON array
fn deserialize_functions(json_value: &str) -> Result<Vec<FunctionInstanceSerialized>, CommonError> {
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

impl TryFrom<Row_get_provider_instance_by_id> for ProviderInstanceSerialized {
    type Error = CommonError;
    fn try_from(row: Row_get_provider_instance_by_id) -> Result<Self, Self::Error> {
        let return_on_successful_brokering = row
            .return_on_successful_brokering
            .as_ref()
            .and_then(|v| serde_json::from_value(v.get_inner().clone()).ok());

        Ok(ProviderInstanceSerialized {
            id: row.id,
            display_name: row.display_name,
            resource_server_credential_id: row.resource_server_credential_id,
            user_credential_id: row.user_credential_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
            provider_controller_type_id: row.provider_controller_type_id,
            credential_controller_type_id: row.credential_controller_type_id,
            status: row.status,
            return_on_successful_brokering,
        })
    }
}

impl TryFrom<Row_get_provider_instances> for ProviderInstanceSerializedWithFunctions {
    type Error = CommonError;
    fn try_from(row: Row_get_provider_instances) -> Result<Self, Self::Error> {
        let return_on_successful_brokering = row
            .return_on_successful_brokering
            .as_ref()
            .and_then(|v| serde_json::from_value(v.get_inner().clone()).ok());

        let provider_instance = ProviderInstanceSerialized {
            id: row.id,
            display_name: row.display_name,
            resource_server_credential_id: row.resource_server_credential_id,
            user_credential_id: row.user_credential_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
            provider_controller_type_id: row.provider_controller_type_id,
            credential_controller_type_id: row.credential_controller_type_id,
            status: row.status,
            return_on_successful_brokering,
        };

        let functions = deserialize_functions(&row.functions)?;
        let resource_server_credential =
            deserialize_resource_server_credential(&row.resource_server_credential)?;
        let user_credential = deserialize_user_credential(&row.user_credential)?;

        Ok(ProviderInstanceSerializedWithFunctions {
            provider_instance,
            functions,
            resource_server_credential,
            user_credential,
        })
    }
}

impl TryFrom<Row_get_provider_instance_by_id> for ProviderInstanceSerializedWithFunctions {
    type Error = CommonError;
    fn try_from(row: Row_get_provider_instance_by_id) -> Result<Self, Self::Error> {
        let return_on_successful_brokering = row
            .return_on_successful_brokering
            .as_ref()
            .and_then(|v| serde_json::from_value(v.get_inner().clone()).ok());

        let provider_instance = ProviderInstanceSerialized {
            id: row.id,
            display_name: row.display_name,
            resource_server_credential_id: row.resource_server_credential_id,
            user_credential_id: row.user_credential_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
            provider_controller_type_id: row.provider_controller_type_id,
            credential_controller_type_id: row.credential_controller_type_id,
            status: row.status,
            return_on_successful_brokering,
        };

        let functions = deserialize_functions(&row.functions)?;
        let resource_server_credential =
            deserialize_resource_server_credential(&row.resource_server_credential)?;
        let user_credential = deserialize_user_credential(&row.user_credential)?;

        Ok(ProviderInstanceSerializedWithFunctions {
            provider_instance,
            functions,
            resource_server_credential,
            user_credential,
        })
    }
}

impl TryFrom<Row_get_function_instance_by_id> for FunctionInstanceSerialized {
    type Error = CommonError;
    fn try_from(row: Row_get_function_instance_by_id) -> Result<Self, Self::Error> {
        Ok(FunctionInstanceSerialized {
            function_controller_type_id: row.function_controller_type_id,
            provider_controller_type_id: row.provider_controller_type_id,
            provider_instance_id: row.provider_instance_id,
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
            provider_instance_id: row.provider_instance_id,
            provider_controller_type_id: row.provider_controller_type_id,
            credential_controller_type_id: row.credential_controller_type_id,
            metadata: row.metadata,
            action,
        })
    }
}

impl TryFrom<Row_get_function_instance_with_credentials>
    for FunctionInstanceSerializedWithCredentials
{
    type Error = CommonError;
    fn try_from(row: Row_get_function_instance_with_credentials) -> Result<Self, Self::Error> {
        let provider_return_on_successful_brokering = row
            .provider_instance_return_on_successful_brokering
            .as_ref()
            .and_then(|v| serde_json::from_value(v.get_inner().clone()).ok());

        let user_credential = match row.user_credential_id {
            Some(id) => UserCredentialSerialized {
                id,
                type_id: row.user_credential_type_id.clone().unwrap_or_default(),
                metadata: row.user_credential_metadata.clone().unwrap_or_default(),
                value: match row.user_credential_value.clone() {
                    Some(value) => value,
                    None => {
                        return Err(CommonError::Unknown(anyhow::anyhow!(
                            "user credential value is required"
                        )));
                    }
                },
                created_at: match row.user_credential_created_at {
                    Some(created_at) => created_at,
                    None => {
                        return Err(CommonError::Unknown(anyhow::anyhow!(
                            "user credential created at is required"
                        )));
                    }
                },
                updated_at: match row.user_credential_updated_at {
                    Some(updated_at) => updated_at,
                    None => {
                        return Err(CommonError::Unknown(anyhow::anyhow!(
                            "user credential updated at is required"
                        )));
                    }
                },
                next_rotation_time: row.user_credential_next_rotation_time,
                dek_alias: match row.user_credential_dek_alias {
                    Some(dek_alias) => dek_alias,
                    None => {
                        return Err(CommonError::Unknown(anyhow::anyhow!(
                            "user credential data encryption key id is required"
                        )));
                    }
                },
            },
            None => {
                return Err(CommonError::Unknown(anyhow::anyhow!(
                    "user credential id is required"
                )));
            }
        };
        Ok(FunctionInstanceSerializedWithCredentials {
            function_instance: FunctionInstanceSerialized {
                function_controller_type_id: row.function_instance_function_controller_type_id,
                provider_controller_type_id: row
                    .function_instance_provider_controller_type_id
                    .clone(),
                provider_instance_id: row.function_instance_provider_instance_id.clone(),
                created_at: row.function_instance_created_at,
                updated_at: row.function_instance_updated_at,
            },
            provider_instance: ProviderInstanceSerialized {
                id: row.provider_instance_id,
                display_name: row.provider_instance_display_name,
                resource_server_credential_id: row
                    .provider_instance_resource_server_credential_id
                    .clone(),
                user_credential_id: row.provider_instance_user_credential_id.clone(),
                created_at: row.provider_instance_created_at,
                updated_at: row.provider_instance_updated_at,
                provider_controller_type_id: row.provider_instance_provider_controller_type_id,
                credential_controller_type_id: row.credential_controller_type_id,
                status: row.provider_instance_status,
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
    provider_controller_type_id: String,
    credential_controller_type_id: String,
    status: String,
    return_on_successful_brokering: Option<serde_json::Value>,
    created_at: String,
    updated_at: String,
    resource_server_credential: serde_json::Value,
    user_credential: serde_json::Value,
}

impl TryFrom<Row_get_provider_instances_grouped_by_function_controller_type_id>
    for crate::repository::ProviderInstancesGroupedByFunctionControllerTypeId
{
    type Error = CommonError;
    fn try_from(
        row: Row_get_provider_instances_grouped_by_function_controller_type_id,
    ) -> Result<Self, Self::Error> {
        // Parse the JSON array of provider instances
        let provider_instances_json: Vec<ProviderInstanceFromGroupedQuery> =
            serde_json::from_str(&row.provider_instances).map_err(|e| CommonError::Repository {
                msg: format!("Failed to deserialize provider_instances JSON: {e}"),
                source: Some(e.into()),
            })?;

        let provider_instances: Vec<ProviderInstanceSerializedWithCredentials> =
            provider_instances_json
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

                    Ok(ProviderInstanceSerializedWithCredentials {
                        provider_instance: ProviderInstanceSerialized {
                            id: pi_json.id,
                            display_name: pi_json.display_name,
                            resource_server_credential_id,
                            user_credential_id,
                            created_at,
                            updated_at,
                            provider_controller_type_id: pi_json.provider_controller_type_id,
                            credential_controller_type_id: pi_json.credential_controller_type_id,
                            status: pi_json.status,
                            return_on_successful_brokering,
                        },
                        resource_server_credential,
                        user_credential,
                    })
                })
                .collect::<Result<Vec<_>, CommonError>>()?;

        Ok(
            crate::repository::ProviderInstancesGroupedByFunctionControllerTypeId {
                function_controller_type_id: row.function_controller_type_id,
                provider_instances,
            },
        )
    }
}

impl TryFrom<super::Row_get_provider_instances_with_credentials>
    for ProviderInstanceSerializedWithCredentials
{
    type Error = CommonError;

    fn try_from(
        row: super::Row_get_provider_instances_with_credentials,
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

        Ok(ProviderInstanceSerializedWithCredentials {
            provider_instance: ProviderInstanceSerialized {
                id: row.id,
                display_name: row.display_name,
                resource_server_credential_id: resource_server_credential.id.clone(),
                user_credential_id: user_credential.as_ref().map(|uc| uc.id.clone()),
                created_at: row.created_at,
                updated_at: row.updated_at,
                provider_controller_type_id: row.provider_controller_type_id,
                credential_controller_type_id: row.credential_controller_type_id,
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
