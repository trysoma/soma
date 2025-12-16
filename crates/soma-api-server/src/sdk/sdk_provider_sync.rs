use std::sync::Arc;

use bridge::logic::{
    PROVIDER_REGISTRY, add_provider_controller_to_registry,
    remove_provider_controller_from_registry,
};
use shared::error::CommonError;
use shared::primitives::WrappedSchema;
use tracing::{debug, error, trace, warn};

use crate::logic::bridge::providers::dynamic::{
    DynamicFunctionControllerParams, DynamicProviderController, DynamicProviderControllerParams,
};

/// Sync providers from SDK metadata to the registry.
/// Removes stale dynamic providers and registers new ones.
pub fn sync_providers_from_metadata(
    metadata: &sdk_proto::MetadataResponse,
) -> Result<(), CommonError> {
    debug!(
        count = metadata.bridge_providers.len(),
        "Syncing providers from SDK"
    );

    // Step 1: Query all existing providers and identify dynamic ones to remove
    let dynamic_provider_ids: Vec<String> = {
        let registry = PROVIDER_REGISTRY
            .read()
            .map_err(|_e| CommonError::Unknown(anyhow::anyhow!("Poison error")))?;

        registry
            .iter()
            .filter_map(|provider| {
                let metadata = provider.metadata();
                if let Some(serde_json::Value::Bool(true)) = metadata.0.get("is_dynamic") {
                    Some(provider.type_id())
                } else {
                    None
                }
            })
            .collect()
    };

    // Step 2: Remove all dynamic providers
    if !dynamic_provider_ids.is_empty() {
        trace!(
            count = dynamic_provider_ids.len(),
            "Removing stale dynamic providers"
        );
    }
    for provider_id in &dynamic_provider_ids {
        match remove_provider_controller_from_registry(provider_id) {
            Ok(_) => trace!(provider_id = %provider_id, "Removed dynamic provider"),
            Err(e) => warn!(provider_id = %provider_id, error = %e, "Failed to remove provider"),
        }
    }

    // Step 3: Add all new providers from SDK metadata
    for proto_provider in &metadata.bridge_providers {
        trace!(
            type_id = %proto_provider.type_id,
            name = %proto_provider.name,
            function_count = proto_provider.functions.len(),
            "Registering provider"
        );

        match register_provider_from_proto(proto_provider) {
            Ok(_) => {
                trace!(type_id = %proto_provider.type_id, "Provider registered");
            }
            Err(e) => {
                error!(
                    type_id = %proto_provider.type_id,
                    name = %proto_provider.name,
                    error = %e,
                    "Failed to register provider"
                );
            }
        }
    }

    debug!(
        count = metadata.bridge_providers.len(),
        "Synced SDK providers"
    );

    Ok(())
}

/// Convert proto provider to DynamicProviderController and register it
fn register_provider_from_proto(
    proto_provider: &sdk_proto::ProviderController,
) -> Result<(), CommonError> {
    let provider_type_id = proto_provider.type_id.clone();
    let functions: Result<Vec<DynamicFunctionControllerParams>, CommonError> = proto_provider
        .functions
        .iter()
        .map(|f| {
            Ok(DynamicFunctionControllerParams {
                provider_type_id: provider_type_id.clone(),
                type_id: f.name.clone(), // Use function name as type_id
                name: f.name.clone(),
                documentation: f.description.clone(),
                parameters: parse_schema_string(&f.parameters)?,
                output: parse_schema_string(&f.output)?,
                categories: proto_provider.categories.clone(), // Inherit from provider
            })
        })
        .collect();

    let provider_params = DynamicProviderControllerParams {
        type_id: proto_provider.type_id.clone(),
        name: proto_provider.name.clone(),
        documentation: proto_provider.documentation.clone(),
        categories: proto_provider.categories.clone(),
        functions: functions?,
    };

    let provider = Arc::new(DynamicProviderController::new(provider_params));
    add_provider_controller_to_registry(provider)?;

    Ok(())
}

/// Parse JSON schema string into WrappedSchema
fn parse_schema_string(schema_str: &str) -> Result<WrappedSchema, CommonError> {
    let schema_value: serde_json::Value = serde_json::from_str(schema_str)
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to parse schema: {e}")))?;

    // Convert JSON Value to schemars::Schema
    let schema: schemars::Schema = serde_json::from_value(schema_value)
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to convert to Schema: {e}")))?;

    Ok(WrappedSchema::new(schema))
}
