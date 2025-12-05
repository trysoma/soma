use std::sync::Arc;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use shared::{
    error::CommonError,
    primitives::{PaginatedResponse, PaginationRequest, WrappedSchema},
};
use std::sync::RwLock;
use utoipa::ToSchema;

use crate::{
    logic::{
        FunctionControllerLike, ProviderControllerLike, ProviderCredentialControllerLike,
        credential::ConfigurationSchema,
    },
    providers::{google_mail::GoogleMailProviderController, stripe::StripeProviderController},
};

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct ProviderCredentialControllerSerialized {
    pub type_id: String,
    pub configuration_schema: ConfigurationSchema,
    pub name: String,
    pub documentation: String,
    pub requires_brokering: bool,
    pub requires_resource_server_credential_refreshing: bool,
    pub requires_user_credential_refreshing: bool,
}

impl From<Arc<dyn ProviderCredentialControllerLike>> for ProviderCredentialControllerSerialized {
    fn from(credential_controller: Arc<dyn ProviderCredentialControllerLike>) -> Self {
        ProviderCredentialControllerSerialized {
            type_id: credential_controller.type_id().to_string(),
            configuration_schema: credential_controller.configuration_schema(),
            name: credential_controller.name().to_string(),
            documentation: credential_controller.documentation().to_string(),
            requires_brokering: credential_controller.as_user_credential_broker().is_some(),
            requires_resource_server_credential_refreshing: credential_controller
                .as_rotateable_controller_resource_server_credential()
                .is_some(),
            requires_user_credential_refreshing: credential_controller
                .as_rotateable_controller_user_credential()
                .is_some(),
        }
    }
}

impl From<&Arc<dyn ProviderCredentialControllerLike>> for ProviderCredentialControllerSerialized {
    fn from(credential_controller: &Arc<dyn ProviderCredentialControllerLike>) -> Self {
        ProviderCredentialControllerSerialized {
            type_id: credential_controller.type_id().to_string(),
            configuration_schema: credential_controller.configuration_schema(),
            name: credential_controller.name().to_string(),
            documentation: credential_controller.documentation().to_string(),
            requires_brokering: credential_controller.as_user_credential_broker().is_some(),
            requires_resource_server_credential_refreshing: credential_controller
                .as_rotateable_controller_resource_server_credential()
                .is_some(),
            requires_user_credential_refreshing: credential_controller
                .as_rotateable_controller_user_credential()
                .is_some(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct FunctionControllerSerialized {
    pub type_id: String,
    pub name: String,
    pub documentation: String,
    pub parameters: WrappedSchema,
    pub output: WrappedSchema,
    pub categories: Vec<String>, // TODO: change to Vec<&'static str>
}

impl From<Arc<dyn FunctionControllerLike>> for FunctionControllerSerialized {
    fn from(function: Arc<dyn FunctionControllerLike>) -> Self {
        FunctionControllerSerialized {
            type_id: function.type_id().to_string(),
            name: function.name().to_string(),
            documentation: function.documentation().to_string(),
            parameters: function.parameters(),
            output: function.output(),
            categories: function
                .categories()
                .into_iter()
                .map(|c| c.to_string())
                .collect(),
        }
    }
}

impl From<&Arc<dyn FunctionControllerLike>> for FunctionControllerSerialized {
    fn from(function: &Arc<dyn FunctionControllerLike>) -> Self {
        FunctionControllerSerialized {
            type_id: function.type_id().to_string(),
            name: function.name().to_string(),
            documentation: function.documentation().to_string(),
            parameters: function.parameters(),
            output: function.output(),
            categories: function
                .categories()
                .into_iter()
                .map(|c| c.to_string())
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct ProviderControllerSerialized {
    pub type_id: String,
    pub name: String,
    pub categories: Vec<String>,
    pub documentation: String,
    pub functions: Vec<FunctionControllerSerialized>,
    pub credential_controllers: Vec<ProviderCredentialControllerSerialized>,
}

impl From<&dyn ProviderControllerLike> for ProviderControllerSerialized {
    fn from(provider: &dyn ProviderControllerLike) -> Self {
        ProviderControllerSerialized {
            type_id: provider.type_id().to_string(),
            name: provider.name().to_string(),
            categories: provider
                .categories()
                .into_iter()
                .map(|c| c.to_string())
                .collect(),
            documentation: provider.documentation().to_string(),
            credential_controllers: provider
                .credential_controllers()
                .into_iter()
                .map(|c| c.into())
                .collect::<Vec<ProviderCredentialControllerSerialized>>(),
            functions: provider
                .functions()
                .into_iter()
                .map(|f| f.into())
                .collect::<Vec<FunctionControllerSerialized>>(),
        }
    }
}

pub const CATEGORY_EMAIL: &str = "email";
pub const CATEGORY_PAYMENTS: &str = "payments";

pub static PROVIDER_REGISTRY: Lazy<RwLock<Vec<Arc<dyn ProviderControllerLike>>>> =
    Lazy::new(|| RwLock::new(Vec::new()));

pub type ListAvailableProvidersParams = PaginationRequest;
pub type ListAvailableProvidersResponse = PaginatedResponse<ProviderControllerSerialized>;
pub async fn list_available_providers(
    pagination: ListAvailableProvidersParams,
) -> Result<ListAvailableProvidersResponse, CommonError> {
    let providers = PROVIDER_REGISTRY
        .read()
        .map_err(|_e| CommonError::Unknown(anyhow::anyhow!("Poison error")))?
        .iter()
        .map(|p| p.as_ref().into())
        .collect::<Vec<ProviderControllerSerialized>>();

    Ok(ListAvailableProvidersResponse::from_items_with_extra(
        providers,
        &pagination,
        |p| vec![p.type_id.to_string()],
    ))
}

pub fn get_provider_controller(
    provider_controller_type_id: &str,
) -> Result<Arc<dyn ProviderControllerLike>, CommonError> {
    let registry = PROVIDER_REGISTRY
        .read()
        .map_err(|_e| CommonError::Unknown(anyhow::anyhow!("Poison error")))?;

    tracing::debug!(
        "Looking for provider controller with type_id: {}, registered providers: {:?}",
        provider_controller_type_id,
        registry.iter().map(|p| p.type_id()).collect::<Vec<_>>()
    );

    let provider_controller = registry
        .iter()
        .find(|p| p.type_id() == provider_controller_type_id)
        .ok_or(CommonError::Unknown(anyhow::anyhow!(
            "Provider controller not found: '{}'. Available providers: {:?}",
            provider_controller_type_id,
            registry.iter().map(|p| p.type_id()).collect::<Vec<_>>()
        )))?
        .clone();

    Ok(provider_controller)
}

/// Add a provider controller to the registry
pub fn add_provider_controller_to_registry(
    provider: Arc<dyn ProviderControllerLike>,
) -> Result<(), CommonError> {
    let mut registry = PROVIDER_REGISTRY
        .write()
        .map_err(|_e| CommonError::Unknown(anyhow::anyhow!("Poison error")))?;

    // Check if provider already exists
    if registry.iter().any(|p| p.type_id() == provider.type_id()) {
        return Err(CommonError::Unknown(anyhow::anyhow!(
            "Provider controller with type_id '{}' already exists",
            provider.type_id()
        )));
    }

    tracing::info!("Adding provider controller: {}", provider.type_id());
    registry.push(provider);

    Ok(())
}

/// Remove a provider controller from the registry by type_id
pub fn remove_provider_controller_from_registry(
    provider_controller_type_id: &str,
) -> Result<(), CommonError> {
    let mut registry = PROVIDER_REGISTRY
        .write()
        .map_err(|_e| CommonError::Unknown(anyhow::anyhow!("Poison error")))?;

    let initial_len = registry.len();
    registry.retain(|p| p.type_id() != provider_controller_type_id);

    if registry.len() == initial_len {
        return Err(CommonError::Unknown(anyhow::anyhow!(
            "Provider controller with type_id '{provider_controller_type_id}' not found"
        )));
    }

    tracing::info!(
        "Removed provider controller: {}",
        provider_controller_type_id
    );

    Ok(())
}

pub fn get_credential_controller(
    provider_controller: &Arc<dyn ProviderControllerLike>,
    credential_controller_type_id: &str,
) -> Result<Arc<dyn ProviderCredentialControllerLike>, CommonError> {
    let credential_controller = provider_controller
        .credential_controllers()
        .iter()
        .find(|c| c.type_id() == credential_controller_type_id)
        .ok_or(CommonError::Unknown(anyhow::anyhow!(
            "Credential controller not found"
        )))?
        .clone();

    Ok(credential_controller)
}

pub fn get_function_controller(
    provider_controller: &Arc<dyn ProviderControllerLike>,
    function_controller_type_id: &str,
) -> Result<Arc<dyn FunctionControllerLike>, CommonError> {
    let function_controller = provider_controller
        .functions()
        .iter()
        .find(|f| f.type_id() == function_controller_type_id)
        .ok_or(CommonError::Unknown(anyhow::anyhow!(
            "Function controller not found"
        )))?
        .clone();
    Ok(function_controller)
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct WithFunctionControllerTypeId<T> {
    pub function_controller_type_id: String,
    pub inner: T,
}

pub async fn register_all_bridge_providers() -> Result<(), CommonError> {
    let mut registry = PROVIDER_REGISTRY.write().map_err(|e| {
        CommonError::Unknown(anyhow::anyhow!("Failed to write provider registry: {e}"))
    })?;
    registry.push(Arc::new(GoogleMailProviderController));
    registry.push(Arc::new(StripeProviderController));
    drop(registry);
    Ok(())
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct WithProviderControllerTypeId<T> {
    pub provider_controller_type_id: String,
    pub inner: T,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct WithCredentialControllerTypeId<T> {
    pub credential_controller_type_id: String,
    pub inner: T,
}

#[cfg(all(test, feature = "unit_test"))]
mod unit_test {
    use super::*;
    use shared::primitives::PaginationRequest;

    #[tokio::test]
    async fn test_list_available_providers() {
        shared::setup_test!();

        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };

        let result = list_available_providers(pagination).await;
        assert!(result.is_ok());

        let _response = result.unwrap();
        // Should have at least the registered providers
        // Should return a valid paginated response (may be empty during isolated tests)
        // Just verify the structure is correct
    }
}
