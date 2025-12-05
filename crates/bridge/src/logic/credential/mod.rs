pub mod api_key;
pub mod no_auth;
pub mod oauth;

use std::sync::Arc;

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shared::{
    error::CommonError,
    primitives::{
        PaginationRequest, WrappedChronoDateTime, WrappedJsonValue, WrappedSchema, WrappedUuidV4,
    },
};
use tracing::info;
use utoipa::ToSchema;

use ::encryption::logic::crypto_services::{DecryptionService, EncryptionService};

use crate::{
    logic::{
        Metadata, OnConfigChangeEvt, OnConfigChangeTx, ProviderControllerLike,
        ProviderCredentialControllerLike,
        controller::{
            WithCredentialControllerTypeId, WithProviderControllerTypeId,
            get_credential_controller, get_provider_controller,
        },
        instance::{
            ProviderInstanceSerialized, ProviderInstanceSerializedWithCredentials, ReturnAddress,
        },
    },
    repository::ProviderRepositoryLike,
};

pub fn schemars_make_password(schema: &mut schemars::Schema) {
    schema.insert(
        String::from("format"),
        Value::String("password".to_string()),
    );
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Credential<T> {
    pub inner: T,
    pub metadata: Metadata,
    pub id: WrappedUuidV4,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

pub trait RotateableCredentialLike: Send + Sync {
    fn next_rotation_time(&self) -> WrappedChronoDateTime;
}

// Static credential configurations

pub trait StaticCredentialConfigurationLike: Send + Sync {
    fn type_id(&self) -> &'static str;
    fn value(&self) -> WrappedJsonValue;
    fn as_rotateable_credential(&self) -> Option<&dyn RotateableCredentialLike> {
        None
    }
}

// pub type StaticCredential = Credential<Arc<dyn StaticCredentialConfigurationLike>>;

// Resource server credentials

pub trait ResourceServerCredentialLike: Send + Sync {
    fn type_id(&self) -> &'static str;
    fn value(&self) -> WrappedJsonValue;
    fn as_rotateable_credential(&self) -> Option<&dyn RotateableCredentialLike> {
        None
    }
}

pub type ResourceServerCredential = Credential<Arc<dyn ResourceServerCredentialLike>>;

// user credentials

pub trait UserCredentialLike: Send + Sync {
    fn type_id(&self) -> &'static str;
    fn value(&self) -> WrappedJsonValue;
    fn as_rotateable_credential(&self) -> Option<&dyn RotateableCredentialLike> {
        None
    }
}

pub type UserCredential = Credential<Arc<dyn UserCredentialLike>>;

// User credentials

// Brokering user credentials

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct BrokerState {
    pub id: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
    pub provider_instance_id: String,
    pub provider_controller_type_id: String,
    pub credential_controller_type_id: String,
    pub metadata: Metadata,
    pub action: BrokerAction,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct BrokerActionRedirect {
    pub url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub enum BrokerAction {
    Redirect(BrokerActionRedirect),
    None,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BrokerInput {
    Oauth2AuthorizationCodeFlow { code: String },
    Oauth2AuthorizationCodeFlowWithPkce { code: String, code_verifier: String },
}

pub enum BrokerOutcome {
    Success {
        user_credential: Box<dyn UserCredentialLike>,
        metadata: Metadata,
    },
    Continue {
        state_id: String,
        state_metadata: Metadata,
    },
}

#[async_trait]
pub trait UserCredentialBrokerLike: Send + Sync {
    /// Called when the user (or system) initiates credential brokering.
    async fn start(
        &self,
        resource_server_cred: &Credential<Box<dyn ResourceServerCredentialLike>>,
    ) -> Result<(BrokerAction, BrokerOutcome), CommonError>;

    /// Called after an external event (callback, redirect, etc.) returns data to the platform.
    async fn resume(
        &self,
        decryption_service: &DecryptionService,
        encryption_service: &EncryptionService,
        state: &BrokerState,
        input: BrokerInput,
        resource_server_cred: &ResourceServerCredentialSerialized,
    ) -> Result<(BrokerAction, BrokerOutcome), CommonError>;
}

#[derive(Serialize, Deserialize, Clone, ToSchema, JsonSchema)]
pub struct ConfigurationSchema {
    pub resource_server: WrappedSchema,
    pub user_credential: WrappedSchema,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct StaticCredentialSerialized {
    // not UUID as some ID's will be deterministic
    pub type_id: String,
    pub metadata: Metadata,

    // this is the serialized version of the actual configuration fields
    pub value: WrappedJsonValue,
}

impl From<Credential<Arc<dyn StaticCredentialConfigurationLike>>> for StaticCredentialSerialized {
    fn from(static_cred: Credential<Arc<dyn StaticCredentialConfigurationLike>>) -> Self {
        StaticCredentialSerialized {
            type_id: static_cred.inner.type_id().to_string(),
            metadata: static_cred.metadata.clone(),
            value: static_cred.inner.value(),
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct ResourceServerCredentialSerialized {
    pub id: WrappedUuidV4,
    pub type_id: String,
    pub metadata: Metadata,
    pub value: WrappedJsonValue,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
    pub next_rotation_time: Option<WrappedChronoDateTime>,
    pub dek_alias: String,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct UserCredentialSerialized {
    pub id: WrappedUuidV4,
    pub type_id: String,
    pub metadata: Metadata,
    pub value: WrappedJsonValue,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
    pub next_rotation_time: Option<WrappedChronoDateTime>,
    pub dek_alias: String,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct CreateResourceServerCredentialParamsInner {
    // NOTE: serialized values are always already encrypted, only encrypt_provider_configuration accepts raw values
    pub resource_server_configuration: WrappedJsonValue,
    pub metadata: Option<Metadata>,
    pub dek_alias: String,
}
pub type CreateResourceServerCredentialParams = WithProviderControllerTypeId<
    WithCredentialControllerTypeId<CreateResourceServerCredentialParamsInner>,
>;
pub type CreateResourceServerCredentialResponse = ResourceServerCredentialSerialized;

pub async fn create_resource_server_credential(
    repo: &impl crate::repository::ProviderRepositoryLike,
    params: CreateResourceServerCredentialParams,
) -> Result<CreateResourceServerCredentialResponse, CommonError> {
    let provider_controller = get_provider_controller(&params.provider_controller_type_id)?;

    let credential_controller = get_credential_controller(
        &provider_controller,
        &params.inner.credential_controller_type_id,
    )?;

    let (resource_server_credential, mut core_metadata) = credential_controller
        .from_serialized_resource_server_configuration(
            params.inner.inner.resource_server_configuration,
        )?;

    if let Some(metadata) = params.inner.inner.metadata {
        core_metadata.0.extend(metadata.0);
    }

    let next_rotation_time = resource_server_credential
        .as_rotateable_credential()
        .map(|resource_server_credential| resource_server_credential.next_rotation_time());

    let now = WrappedChronoDateTime::now();
    let resource_server_credential_serialized = ResourceServerCredentialSerialized {
        id: WrappedUuidV4::new(),
        type_id: resource_server_credential.type_id().to_string(),
        metadata: core_metadata,
        value: resource_server_credential.value(),
        created_at: now,
        updated_at: now,
        next_rotation_time,
        dek_alias: params.inner.inner.dek_alias,
    };

    // Save to database
    repo.create_resource_server_credential(
        &crate::repository::CreateResourceServerCredential::from(
            resource_server_credential_serialized.clone(),
        ),
    )
    .await?;

    Ok(resource_server_credential_serialized)
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct CreateUserCredentialParamsInner {
    pub user_credential_configuration: WrappedJsonValue,
    pub metadata: Option<Metadata>,
    pub dek_alias: String,
}
pub type CreateUserCredentialParams =
    WithProviderControllerTypeId<WithCredentialControllerTypeId<CreateUserCredentialParamsInner>>;
pub type CreateUserCredentialResponse = UserCredentialSerialized;

pub async fn create_user_credential(
    repo: &impl crate::repository::ProviderRepositoryLike,
    params: CreateUserCredentialParams,
) -> Result<CreateUserCredentialResponse, CommonError> {
    let provider_controller = get_provider_controller(&params.provider_controller_type_id)?;

    let credential_controller = get_credential_controller(
        &provider_controller,
        &params.inner.credential_controller_type_id,
    )?;

    let (user_credential, mut core_metadata) = credential_controller
        .from_serialized_user_credential_configuration(
            params.inner.inner.user_credential_configuration,
        )?;

    if let Some(metadata) = params.inner.inner.metadata {
        core_metadata.0.extend(metadata.0);
    }

    let next_rotation_time = user_credential
        .as_rotateable_credential()
        .map(|user_credential| user_credential.next_rotation_time());

    let now = WrappedChronoDateTime::now();
    let user_credential_serialized = UserCredentialSerialized {
        id: WrappedUuidV4::new(),
        type_id: user_credential.type_id().to_string(),
        metadata: core_metadata,
        value: user_credential.value(),
        created_at: now,
        updated_at: now,
        next_rotation_time,
        dek_alias: params.inner.inner.dek_alias,
    };

    // Save to database
    repo.create_user_credential(&crate::repository::CreateUserCredential::from(
        user_credential_serialized.clone(),
    ))
    .await?;

    Ok(user_credential_serialized)
}

async fn process_broker_outcome(
    on_config_change_tx: &OnConfigChangeTx,
    repo: &impl crate::repository::ProviderRepositoryLike,
    provider_controller: &Arc<dyn ProviderControllerLike>,
    credential_controller: &Arc<dyn ProviderCredentialControllerLike>,
    broker_action: &BrokerAction,
    outcome: BrokerOutcome,
    provider_instance: &ProviderInstanceSerialized,
    // return_on_success: Option<ReturnAddress>,
) -> Result<UserCredentialBrokeringResponse, CommonError> {
    let response = match outcome {
        BrokerOutcome::Success {
            user_credential,
            metadata,
        } => {
            // let provider_instance = repo
            //     .get_provider_instance_by_id(&provider_instance_id)
            //     .await?
            //     .ok_or(CommonError::Unknown(anyhow::anyhow!(
            //         "Provider instance not found"
            //     )))?;

            let resource_server_cred = repo
                .get_resource_server_credential_by_id(
                    &provider_instance.resource_server_credential_id,
                )
                .await?;

            let resource_server_cred = match resource_server_cred {
                Some(resource_server_cred) => resource_server_cred,
                None => {
                    return Err(CommonError::Unknown(anyhow::anyhow!(
                        "Resource server credential not found"
                    )));
                }
            };
            let dek_alias = resource_server_cred.dek_alias;
            let user_credential = create_user_credential(
                repo,
                CreateUserCredentialParams {
                    provider_controller_type_id: provider_controller.type_id().to_string(),
                    inner: WithCredentialControllerTypeId {
                        credential_controller_type_id: credential_controller.type_id().to_string(),
                        inner: CreateUserCredentialParamsInner {
                            dek_alias,
                            user_credential_configuration: user_credential.value(),
                            metadata: Some(metadata.clone()),
                        },
                    },
                },
            )
            .await?;

            // Update the provider instance to link the user credential and set status to active
            repo.update_provider_instance_after_brokering(
                &provider_instance.id,
                &user_credential.id,
            )
            .await?;

            // Fetch the updated provider instance with credentials to send config change event
            let updated_provider_instance = repo
                .get_provider_instance_by_id(&provider_instance.id)
                .await?
                .ok_or_else(|| {
                    CommonError::Unknown(anyhow::anyhow!(
                        "Provider instance not found after update"
                    ))
                })?;

            let resource_server_cred = repo
                .get_resource_server_credential_by_id(
                    &updated_provider_instance
                        .provider_instance
                        .resource_server_credential_id,
                )
                .await?
                .ok_or_else(|| {
                    CommonError::Unknown(anyhow::anyhow!("Resource server credential not found"))
                })?;

            let provider_instance_with_creds = ProviderInstanceSerializedWithCredentials {
                provider_instance: updated_provider_instance.provider_instance,
                resource_server_credential: resource_server_cred,
                user_credential: Some(user_credential.clone()),
            };

            // Send config change event
            on_config_change_tx
                .send(OnConfigChangeEvt::ProviderInstanceAdded(
                    provider_instance_with_creds,
                ))
                .map_err(|e| {
                    CommonError::Unknown(anyhow::anyhow!("Failed to send config change event: {e}"))
                })?;

            if let Some(return_on_success) = &provider_instance.return_on_successful_brokering {
                match return_on_success {
                    ReturnAddress::Url(url) => {
                        return Ok(UserCredentialBrokeringResponse::Redirect(url.url.clone()));
                    }
                }
            }

            UserCredentialBrokeringResponse::UserCredential(user_credential)
        }
        BrokerOutcome::Continue {
            state_metadata,
            state_id,
        } => {
            let broker_state = BrokerState {
                id: state_id,
                created_at: WrappedChronoDateTime::now(),
                updated_at: WrappedChronoDateTime::now(),
                provider_instance_id: provider_instance.id.clone(),
                provider_controller_type_id: provider_controller.type_id().to_string(),
                metadata: state_metadata,
                action: broker_action.clone(),
                credential_controller_type_id: credential_controller.type_id().to_string(),
            };

            info!("Saving broker state to database: {:?}", broker_state);
            // Save broker state to database
            repo.create_broker_state(&crate::repository::CreateBrokerState::from(
                broker_state.clone(),
            ))
            .await?;

            UserCredentialBrokeringResponse::BrokerState(broker_state)
        }
    };

    Ok(response)
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct StartUserCredentialBrokeringParamsInner {
    pub provider_instance_id: String,
    // pub return_on_success: ReturnAddress
}
pub type StartUserCredentialBrokeringParams = WithProviderControllerTypeId<
    WithCredentialControllerTypeId<StartUserCredentialBrokeringParamsInner>,
>;

#[derive(Serialize, Deserialize, Clone, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UserCredentialBrokeringResponse {
    BrokerState(BrokerState),
    UserCredential(UserCredentialSerialized),
    Redirect(String),
}
pub async fn start_user_credential_brokering(
    on_config_change_tx: &OnConfigChangeTx,
    repo: &impl crate::repository::ProviderRepositoryLike,
    params: StartUserCredentialBrokeringParams,
) -> Result<UserCredentialBrokeringResponse, CommonError> {
    let provider_controller = get_provider_controller(&params.provider_controller_type_id)?;
    let credential_controller = get_credential_controller(
        &provider_controller,
        &params.inner.credential_controller_type_id,
    )?;
    let user_credential_broker = match credential_controller.as_user_credential_broker() {
        Some(broker) => broker,
        None => {
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "Provider controller does not support user credential brokering"
            )));
        }
    };

    let provider_instance = repo
        .get_provider_instance_by_id(&params.inner.inner.provider_instance_id)
        .await?
        .ok_or(CommonError::Unknown(anyhow::anyhow!(
            "Provider instance not found"
        )))?;

    // Fetch resource server credential from database
    let resource_server_cred = repo
        .get_resource_server_credential_by_id(
            &provider_instance
                .provider_instance
                .resource_server_credential_id,
        )
        .await?
        .ok_or(CommonError::Unknown(anyhow::anyhow!(
            "Resource server credential not found"
        )))?;

    let (inner, metadata) = credential_controller
        .from_serialized_resource_server_configuration(resource_server_cred.value)?;
    let resource_server_cred = Credential {
        inner,
        metadata,
        id: resource_server_cred.id,
        created_at: resource_server_cred.created_at,
        updated_at: resource_server_cred.updated_at,
    };
    let (action, outcome) = user_credential_broker.start(&resource_server_cred).await?;

    let response = process_broker_outcome(
        on_config_change_tx,
        repo,
        &provider_controller,
        &credential_controller,
        &action,
        outcome,
        &provider_instance.provider_instance,
        // Some(params.inner.inner.return_on_success),
    )
    .await?;
    Ok(response)
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct ResumeUserCredentialBrokeringParams {
    pub broker_state_id: String,
    pub input: BrokerInput,
}

pub async fn resume_user_credential_brokering<R>(
    on_config_change_tx: &OnConfigChangeTx,
    repo: &R,
    crypto_cache: &encryption::logic::crypto_services::CryptoCache,
    params: ResumeUserCredentialBrokeringParams,
) -> Result<UserCredentialBrokeringResponse, CommonError>
where
    R: crate::repository::ProviderRepositoryLike,
{
    // Fetch broker state from database
    let broker_state = repo
        .get_broker_state_by_id(&params.broker_state_id)
        .await?
        .ok_or(CommonError::Unknown(anyhow::anyhow!(
            "Broker state not found"
        )))?;

    let provider_controller = get_provider_controller(&broker_state.provider_controller_type_id)?;
    let credential_controller = get_credential_controller(
        &provider_controller,
        &broker_state.credential_controller_type_id,
    )?;

    let user_credential_broker = match credential_controller.as_user_credential_broker() {
        Some(broker) => broker,
        None => {
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "Provider controller does not support user credential brokering"
            )));
        }
    };

    let provider_instance = repo
        .get_provider_instance_by_id(&broker_state.provider_instance_id)
        .await?
        .ok_or(CommonError::Unknown(anyhow::anyhow!(
            "Provider instance not found"
        )))?;
    let resource_server_cred = repo
        .get_resource_server_credential_by_id(
            &provider_instance
                .provider_instance
                .resource_server_credential_id,
        )
        .await?
        .ok_or(CommonError::Unknown(anyhow::anyhow!(
            "Resource server credential not found for provider instance"
        )))?;

    // Get encryption/decryption services from the cache using the DEK alias
    let encryption_service = crypto_cache
        .get_encryption_service(&resource_server_cred.dek_alias)
        .await?;
    let decryption_service = crypto_cache
        .get_decryption_service(&resource_server_cred.dek_alias)
        .await?;
    let (action, outcome) = user_credential_broker
        .resume(
            &decryption_service,
            &encryption_service,
            &broker_state,
            params.input,
            &resource_server_cred,
        )
        .await?;

    let response = process_broker_outcome(
        on_config_change_tx,
        repo,
        &provider_controller,
        &credential_controller,
        &action,
        outcome,
        &provider_instance.provider_instance,
    )
    .await?;

    Ok(response)
}

// ============================================================================
// Credential Rotation Background Task
// ============================================================================

/// Background task that periodically rotates credentials
/// This function is designed to be called in its own tokio::spawn
pub async fn credential_rotation_task<R>(
    repo: R,
    crypto_cache: encryption::logic::crypto_services::CryptoCache,
    on_config_change_tx: OnConfigChangeTx,
    mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
) where
    R: ProviderRepositoryLike,
{
    use tokio::time::{Duration, interval};

    let mut timer = interval(Duration::from_secs(10 * 60)); // 10 minutes

    loop {
        tokio::select! {
            _ = timer.tick() => {
                tracing::info!("Starting credential rotation check");

                if let Err(e) = process_credential_rotations_with_window(
                    &repo,
                    &on_config_change_tx,
                    &crypto_cache,
                    20,
                )
                .await
                {
                    tracing::error!("Error processing credential rotations: {:?}", e);
                }

                tracing::info!("Completed credential rotation check");
            }
            _ = shutdown_rx.recv() => {
                tracing::info!("Credential rotation task shutdown requested");
                break;
            }
        }
    }

    tracing::info!("Credential rotation task stopped");
}

pub async fn process_credential_rotations_with_window<R>(
    repo: &R,
    on_config_change_tx: &OnConfigChangeTx,
    crypto_cache: &encryption::logic::crypto_services::CryptoCache,
    window_minutes: i64,
) -> Result<(), CommonError>
where
    R: ProviderRepositoryLike,
{
    // Calculate rotation window
    let now = WrappedChronoDateTime::now();
    let rotation_window_end: WrappedChronoDateTime = WrappedChronoDateTime::new(
        now.get_inner()
            .checked_add_signed(chrono::Duration::minutes(window_minutes))
            .ok_or_else(|| {
                CommonError::Unknown(anyhow::anyhow!("Failed to calculate rotation window"))
            })?,
    );
    let mut next_page_token: Option<String> = None;

    loop {
        let provider_instances = repo
            .list_provider_instances_with_credentials(
                &PaginationRequest {
                    page_size: 1000,
                    next_page_token,
                },
                None,
                Some(&rotation_window_end),
            )
            .await?;
        info!(
            "Provider instances with credentials: {:?}",
            provider_instances.items.len()
        );
        next_page_token = provider_instances.next_page_token.clone();

        let refresh_fut = provider_instances
            .items
            .iter()
            .map(async |pi| {
                info!(
                    "Processing credential rotation for provider instance: {:?}",
                    pi.provider_instance.id
                );
                process_credential_rotation(
                    repo,
                    on_config_change_tx,
                    crypto_cache,
                    pi,
                    &rotation_window_end,
                    true,
                )
                .await
            })
            .collect::<Vec<_>>();

        futures::future::try_join_all(refresh_fut).await?;

        if next_page_token.is_none() {
            break;
        }
    }
    Ok(())
}

pub async fn process_credential_rotation<R>(
    repo: &R,
    on_config_change_tx: &OnConfigChangeTx,
    crypto_cache: &encryption::logic::crypto_services::CryptoCache,
    pi: &ProviderInstanceSerializedWithCredentials,
    rotation_window_end: &WrappedChronoDateTime,
    publish_update: bool,
) -> Result<(), CommonError>
where
    R: ProviderRepositoryLike,
{
    let mut resource_server_rotated = false;
    let mut user_cred_rotated = false;

    // Rotate resource server credential if needed
    let resource_server_cred_rotation_result =
        match pi.resource_server_credential.next_rotation_time {
            Some(next_rotation_time) => {
                if next_rotation_time.get_inner() <= rotation_window_end.get_inner() {
                    resource_server_rotated = true;
                    rotate_resource_server_credential(
                        repo,
                        crypto_cache,
                        &pi.provider_instance,
                        &pi.resource_server_credential,
                    )
                    .await?
                } else {
                    pi.resource_server_credential.clone()
                }
            }
            None => pi.resource_server_credential.clone(),
        };

    // Rotate user credential if needed
    let user_cred_rotation_result = match &pi.user_credential {
        Some(user_cred) => match user_cred.next_rotation_time {
            Some(next_rotation_time) => {
                if next_rotation_time.get_inner() <= rotation_window_end.get_inner() {
                    user_cred_rotated = true;
                    Some(
                        rotate_user_credential(
                            repo,
                            crypto_cache,
                            &pi.provider_instance,
                            &resource_server_cred_rotation_result,
                            user_cred,
                        )
                        .await?,
                    )
                } else {
                    Some(user_cred.clone())
                }
            }
            None => Some(user_cred.clone()),
        },
        None => None,
    };

    // Only send update event if something was actually rotated
    if publish_update && (resource_server_rotated || user_cred_rotated) {
        on_config_change_tx
            .send(OnConfigChangeEvt::ProviderInstanceUpdated(
                ProviderInstanceSerializedWithCredentials {
                    provider_instance: pi.provider_instance.clone(),
                    resource_server_credential: resource_server_cred_rotation_result,
                    user_credential: user_cred_rotation_result,
                },
            ))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send config change event: {e}"))
            })?;
    }

    Ok::<(), CommonError>(())
}

async fn rotate_resource_server_credential<R>(
    repo: &R,
    crypto_cache: &encryption::logic::crypto_services::CryptoCache,
    provider_instance: &ProviderInstanceSerialized,
    resource_server_cred: &ResourceServerCredentialSerialized,
) -> Result<ResourceServerCredentialSerialized, CommonError>
where
    R: ProviderRepositoryLike,
{
    // Get encryption/decryption services from the cache using the DEK alias
    let encryption_service = crypto_cache
        .get_encryption_service(&resource_server_cred.dek_alias)
        .await?;
    let decryption_service = crypto_cache
        .get_decryption_service(&resource_server_cred.dek_alias)
        .await?;

    let provider_controller =
        get_provider_controller(&provider_instance.provider_controller_type_id)?;
    let credential_controller = get_credential_controller(
        &provider_controller,
        &provider_instance.credential_controller_type_id,
    )?;
    let rotateable_controller =
        credential_controller.as_rotateable_controller_resource_server_credential();

    let rotateable_controller = match rotateable_controller {
        Some(rotateable_controller) => rotateable_controller,
        None => {
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "Provider controller does not support resource server credential rotation"
            )));
        }
    };

    let rotated_credential = rotateable_controller
        .rotate_resource_server_credential(
            &decryption_service,
            &encryption_service,
            credential_controller.static_credentials(),
            resource_server_cred,
        )
        .await?;
    // id: &WrappedUuidV4,
    // value: Option<&WrappedJsonValue>,
    // metadata: Option<&crate::logic::Metadata>,
    // next_rotation_time: Option<&WrappedChronoDateTime>,
    // updated_at: Option<&WrappedChronoDateTime>,
    repo.update_resource_server_credential(
        &resource_server_cred.id,
        Some(&rotated_credential.value),
        Some(&rotated_credential.metadata),
        Some(&match rotated_credential.next_rotation_time {
            Some(next_rotation_time) => next_rotation_time,
            None => {
                return Err(CommonError::Unknown(anyhow::anyhow!(
                    "Resource server credential has no next rotation time"
                )));
            }
        }),
        Some(&WrappedChronoDateTime::now()),
    )
    .await?;

    Ok(rotated_credential)
}

async fn rotate_user_credential<R>(
    repo: &R,
    crypto_cache: &encryption::logic::crypto_services::CryptoCache,
    provider_instance: &ProviderInstanceSerialized,
    resource_server_cred: &ResourceServerCredentialSerialized,
    user_cred: &UserCredentialSerialized,
) -> Result<UserCredentialSerialized, CommonError>
where
    R: ProviderRepositoryLike,
{
    // Get encryption/decryption services from the cache - user and resource credentials may use different DEKs
    let encryption_service = crypto_cache
        .get_encryption_service(&user_cred.dek_alias)
        .await?;
    let decryption_service = crypto_cache
        .get_decryption_service(&user_cred.dek_alias)
        .await?;

    let provider_controller =
        get_provider_controller(&provider_instance.provider_controller_type_id)?;
    let credential_controller = get_credential_controller(
        &provider_controller,
        &provider_instance.credential_controller_type_id,
    )?;
    let rotateable_controller = credential_controller.as_rotateable_controller_user_credential();

    let rotateable_controller = match rotateable_controller {
        Some(rotateable_controller) => rotateable_controller,
        None => {
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "Provider controller does not support resource server credential rotation"
            )));
        }
    };

    let rotated_credential = rotateable_controller
        .rotate_user_credential(
            &decryption_service,
            &encryption_service,
            credential_controller.static_credentials(),
            resource_server_cred,
            user_cred,
        )
        .await?;
    // id: &WrappedUuidV4,
    // value: Option<&WrappedJsonValue>,
    // metadata: Option<&crate::logic::Metadata>,
    // next_rotation_time: Option<&WrappedChronoDateTime>,
    // updated_at: Option<&WrappedChronoDateTime>,
    repo.update_user_credential(
        &user_cred.id,
        Some(&rotated_credential.value),
        Some(&rotated_credential.metadata),
        Some(&match rotated_credential.next_rotation_time {
            Some(next_rotation_time) => next_rotation_time,
            None => {
                return Err(CommonError::Unknown(anyhow::anyhow!(
                    "User credential has no next rotation time"
                )));
            }
        }),
        Some(&WrappedChronoDateTime::now()),
    )
    .await?;

    Ok(rotated_credential)
}

#[cfg(all(test, feature = "unit_test"))]
mod unit_test {
    use super::*;
    use crate::logic::credential::oauth::{
        Oauth2AuthorizationCodeFlowResourceServerCredential,
        Oauth2AuthorizationCodeFlowStaticCredentialConfiguration,
        Oauth2AuthorizationCodeFlowUserCredential, OauthAuthFlowController,
    };

    use shared::primitives::SqlMigrationLoader;

    #[tokio::test]
    async fn test_create_resource_server_credential() {
        shared::setup_test!();

        let _repo = {
            let (_db, conn) = shared::test_utils::repository::setup_in_memory_database(vec![
                crate::repository::Repository::load_sql_migrations(),
            ])
            .await
            .unwrap();
            crate::repository::Repository::new(conn)
        };

        // Use the test helper to set up encryption services
        let setup = crate::test::encryption_service::setup_test_encryption("test-dek").await;
        let encryption_service = setup
            .crypto_cache
            .get_encryption_service(&setup.dek_alias)
            .await
            .unwrap();

        // Create encrypted resource server configuration
        let controller = OauthAuthFlowController {
            static_credentials: Oauth2AuthorizationCodeFlowStaticCredentialConfiguration {
                auth_uri: "https://example.com/auth".to_string(),
                token_uri: "https://example.com/token".to_string(),
                userinfo_uri: "https://example.com/userinfo".to_string(),
                jwks_uri: "https://example.com/jwks".to_string(),
                issuer: "https://example.com".to_string(),
                scopes: vec!["scope1".to_string()],
                metadata: Metadata::new(),
            },
        };

        let raw_config = WrappedJsonValue::new(serde_json::json!({
            "client_id": "test-client-id",
            "client_secret": "plain-text-secret",
            "redirect_uri": "https://example.com/callback",
            "metadata": {"key": "value"}
        }));

        let encrypted_cred = controller
            .encrypt_resource_server_configuration(&encryption_service, raw_config)
            .await
            .unwrap();

        // Note: We cannot test create_resource_server_credential directly without registering
        // a provider in the provider registry. Instead, test that the encryption works correctly.
        let config = encrypted_cred.value();
        let deserialized: Oauth2AuthorizationCodeFlowResourceServerCredential =
            serde_json::from_value(config.into()).unwrap();

        // Verify the client_id is preserved
        assert_eq!(deserialized.client_id, "test-client-id");

        // Verify the client_secret is encrypted
        assert_ne!(deserialized.client_secret.0, "plain-text-secret");

        // Verify it's base64 encoded
        assert!(
            base64::Engine::decode(
                &base64::engine::general_purpose::STANDARD,
                &deserialized.client_secret.0
            )
            .is_ok()
        );
    }

    #[tokio::test]
    async fn test_create_user_credential() {
        shared::setup_test!();

        let _repo = {
            let (_db, conn) = shared::test_utils::repository::setup_in_memory_database(vec![
                crate::repository::Repository::load_sql_migrations(),
            ])
            .await
            .unwrap();
            crate::repository::Repository::new(conn)
        };

        // Use the test helper to set up encryption services
        let setup = crate::test::encryption_service::setup_test_encryption("test-dek").await;
        let encryption_service = setup
            .crypto_cache
            .get_encryption_service(&setup.dek_alias)
            .await
            .unwrap();

        let controller = OauthAuthFlowController {
            static_credentials: Oauth2AuthorizationCodeFlowStaticCredentialConfiguration {
                auth_uri: "https://example.com/auth".to_string(),
                token_uri: "https://example.com/token".to_string(),
                userinfo_uri: "https://example.com/userinfo".to_string(),
                jwks_uri: "https://example.com/jwks".to_string(),
                issuer: "https://example.com".to_string(),
                scopes: vec!["scope1".to_string()],
                metadata: Metadata::new(),
            },
        };

        let expiry = WrappedChronoDateTime::new(
            WrappedChronoDateTime::now()
                .get_inner()
                .checked_add_signed(chrono::Duration::hours(1))
                .unwrap(),
        );

        let raw_config = WrappedJsonValue::new(serde_json::json!({
            "code": "plain-code",
            "access_token": "plain-access-token",
            "refresh_token": "plain-refresh-token",
            "expiry_time": expiry,
            "sub": "test-user",
            "scopes": ["scope1", "scope2"],
            "metadata": {"key": "value"}
        }));

        let encrypted_cred = controller
            .encrypt_user_credential_configuration(&encryption_service, raw_config)
            .await
            .unwrap();

        // Note: We cannot test create_user_credential directly without registering
        // a provider in the provider registry. Instead, test that the encryption and
        // rotation time calculation work correctly.
        let config = encrypted_cred.value();
        let deserialized: Oauth2AuthorizationCodeFlowUserCredential =
            serde_json::from_value(config.into()).unwrap();

        // Verify non-sensitive fields are preserved
        assert_eq!(deserialized.sub, "test-user");
        assert_eq!(deserialized.scopes, vec!["scope1", "scope2"]);

        // Verify all sensitive fields are encrypted
        assert_ne!(deserialized.code.0, "plain-code");
        assert_ne!(deserialized.access_token.0, "plain-access-token");
        assert_ne!(deserialized.refresh_token.0, "plain-refresh-token");

        // Test rotation time calculation
        let next_rotation = deserialized.next_rotation_time();
        let expected_rotation = WrappedChronoDateTime::new(
            expiry
                .get_inner()
                .checked_sub_signed(chrono::Duration::minutes(5))
                .unwrap(),
        );
        assert_eq!(next_rotation.get_inner(), expected_rotation.get_inner());
    }

    #[tokio::test]
    async fn test_process_credential_rotations_no_credentials() {
        shared::setup_test!();

        let repo = {
            let (_db, conn) = shared::test_utils::repository::setup_in_memory_database(vec![
                crate::repository::Repository::load_sql_migrations(),
            ])
            .await
            .unwrap();
            crate::repository::Repository::new(conn)
        };
        let (tx, _rx): (crate::logic::OnConfigChangeTx, _) = tokio::sync::broadcast::channel(100);

        // Use the test helper to set up encryption services
        let setup = crate::test::encryption_service::setup_test_encryption("test-dek").await;

        // Test with no provider instances
        let result =
            process_credential_rotations_with_window(&repo, &tx, &setup.crypto_cache, 20).await;

        // Should succeed even with no credentials
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_broker_state_serialization() {
        shared::setup_test!();

        let broker_state = BrokerState {
            id: "test-id".to_string(),
            created_at: WrappedChronoDateTime::now(),
            updated_at: WrappedChronoDateTime::now(),
            provider_instance_id: "test-instance".to_string(),
            provider_controller_type_id: "google_mail".to_string(),
            credential_controller_type_id: "oauth_auth_flow".to_string(),
            metadata: Metadata::new(),
            action: BrokerAction::Redirect(BrokerActionRedirect {
                url: "https://example.com/auth".to_string(),
            }),
        };

        // Test serialization
        let json = serde_json::to_string(&broker_state).unwrap();
        let deserialized: BrokerState = serde_json::from_str(&json).unwrap();

        assert_eq!(broker_state.id, deserialized.id);
        assert_eq!(
            broker_state.provider_instance_id,
            deserialized.provider_instance_id
        );
    }

    #[tokio::test]
    async fn test_credential_rotation_time_calculation() {
        shared::setup_test!();

        let now = WrappedChronoDateTime::now();

        // Test rotation window calculation
        let rotation_window_end = WrappedChronoDateTime::new(
            now.get_inner()
                .checked_add_signed(chrono::Duration::minutes(20))
                .unwrap(),
        );

        // Verify that credentials expiring soon would be caught
        let expiry_in_10_minutes = WrappedChronoDateTime::new(
            now.get_inner()
                .checked_add_signed(chrono::Duration::minutes(10))
                .unwrap(),
        );

        let rotation_time = WrappedChronoDateTime::new(
            expiry_in_10_minutes
                .get_inner()
                .checked_sub_signed(chrono::Duration::minutes(5))
                .unwrap(),
        );

        // This credential should be rotated (rotation time is within window)
        assert!(rotation_time.get_inner() <= rotation_window_end.get_inner());

        // Test credential that doesn't need rotation yet
        let expiry_in_2_hours = WrappedChronoDateTime::new(
            now.get_inner()
                .checked_add_signed(chrono::Duration::hours(2))
                .unwrap(),
        );

        let rotation_time_later = WrappedChronoDateTime::new(
            expiry_in_2_hours
                .get_inner()
                .checked_sub_signed(chrono::Duration::minutes(5))
                .unwrap(),
        );

        // This credential should NOT be rotated yet
        assert!(rotation_time_later.get_inner() > rotation_window_end.get_inner());
    }
}
