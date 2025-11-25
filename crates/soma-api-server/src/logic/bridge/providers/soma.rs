use std::sync::Arc;

use async_trait::async_trait;
use bridge::logic::FunctionControllerLike;
use bridge::logic::InvokeError;
use bridge::logic::InvokeResult;
use bridge::logic::Metadata;
use bridge::logic::ProviderControllerLike;
use bridge::logic::ProviderCredentialControllerLike;
use bridge::logic::ResourceServerCredentialSerialized;
use bridge::logic::StaticCredentialConfigurationLike;
use bridge::logic::StaticProviderCredentialControllerLike;
use bridge::logic::UserCredentialSerialized;
use bridge::logic::no_auth::NoAuthController;
use bridge::logic::no_auth::NoAuthStaticCredentialConfiguration;
use encryption::logic::crypto_services::DecryptionService;
use schemars::schema_for;
use shared::primitives::WrappedJsonValue;
use shared::primitives::WrappedSchema;

use shared::error::CommonError;

use crate::logic::task::GetTaskTimelineItemsRequest;
use crate::logic::task::GetTaskTimelineItemsResponse;
use crate::logic::task::get_task_timeline_items;
use crate::repository::Repository;

/// Soma provider controller that provides soma-specific functions
pub struct SomaProviderController {
    repository: Repository,
}

impl SomaProviderController {
    pub fn new(repository: Repository) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl ProviderControllerLike for SomaProviderController {
    fn type_id(&self) -> String {
        "soma".to_string()
    }

    fn documentation(&self) -> String {
        "".to_string()
    }

    fn name(&self) -> String {
        "Soma".to_string()
    }

    fn categories(&self) -> Vec<String> {
        vec![]
    }

    fn functions(&self) -> Vec<Arc<dyn FunctionControllerLike>> {
        vec![Arc::new(GetTaskTimelineItemsFunctionController {
            repository: self.repository.clone(),
        })]
    }

    fn credential_controllers(&self) -> Vec<Arc<dyn ProviderCredentialControllerLike>> {
        vec![Arc::new(NoAuthController {
            static_credentials: NoAuthStaticCredentialConfiguration {
                metadata: Metadata::new(),
            },
        })]
    }

    fn metadata(&self) -> Metadata {
        Metadata::new()
    }
}

/// Function controller for getting task timeline items
struct GetTaskTimelineItemsFunctionController {
    repository: Repository,
}

impl GetTaskTimelineItemsFunctionController {
    #[allow(dead_code)]
    pub fn new(repository: Repository) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl FunctionControllerLike for GetTaskTimelineItemsFunctionController {
    fn type_id(&self) -> String {
        "soma_get_task_timeline_items".to_string()
    }
    fn name(&self) -> String {
        "Get task timeline items".to_string()
    }
    fn documentation(&self) -> String {
        "".to_string()
    }
    fn parameters(&self) -> WrappedSchema {
        WrappedSchema::new(schema_for!(GetTaskTimelineItemsRequest))
    }
    fn output(&self) -> WrappedSchema {
        WrappedSchema::new(schema_for!(GetTaskTimelineItemsResponse))
    }
    fn categories(&self) -> Vec<String> {
        vec![]
    }

    async fn invoke(
        &self,
        _crypto_service: &DecryptionService,
        credential_controller: &Arc<dyn ProviderCredentialControllerLike>,
        _static_credentials: &dyn StaticCredentialConfigurationLike,
        _resource_server_credential: &ResourceServerCredentialSerialized,
        _user_credential: &UserCredentialSerialized,
        params: WrappedJsonValue,
    ) -> Result<InvokeResult, CommonError> {
        // Parse the function parameters
        let params: GetTaskTimelineItemsRequest = serde_json::from_value(params.into())
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Invalid parameters: {e}")))?;

        // Downcast to OAuth controller and decrypt credentials
        let cred_controller_type_id = credential_controller.type_id();

        if cred_controller_type_id == NoAuthController::static_type_id() {
            let _controller = credential_controller
                .as_any()
                .downcast_ref::<NoAuthController>()
                .ok_or_else(|| {
                    CommonError::Unknown(anyhow::anyhow!("Failed to downcast to NoAuthController"))
                })?;
        } else {
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "Unsupported credential controller type: {cred_controller_type_id}"
            )));
        };

        let res = get_task_timeline_items(&self.repository, params).await;

        match res {
            Ok(res) => Ok(InvokeResult::Success(WrappedJsonValue::new(
                serde_json::json!(res),
            ))),
            Err(e) => Ok(InvokeResult::Error(InvokeError {
                message: e.to_string(),
            })),
        }
    }
}
