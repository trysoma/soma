
pub mod api_key;
pub mod no_auth;
pub mod oauth;

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, OsRng},
};
use async_trait::async_trait;
use base64::Engine;
use enum_dispatch::enum_dispatch;
use once_cell::sync::Lazy;
use rand::RngCore;
use reqwest::Request;
use schemars::{JsonSchema, Schema};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use shared::{
    error::CommonError,
    primitives::{
        PaginatedResponse, PaginationRequest, WrappedChronoDateTime, WrappedJsonValue,
        WrappedSchema, WrappedUuidV4,
    },
};
use std::fs;
use std::path::Path;
use std::sync::RwLock;
use utoipa::ToSchema;

use crate::{
    logic::{controller::{get_credential_controller, get_provider_controller, WithCredentialControllerTypeId, WithProviderControllerTypeId}, encryption::{get_crypto_service, get_decryption_service, get_encryption_service, DecryptionService, EncryptedString, EncryptionService, EnvelopeEncryptionKeyContents}, instance::{ProviderInstanceSerialized, ProviderInstanceSerializedWithCredentials, ReturnAddress}, Metadata, OnConfigChangeEvt, OnConfigChangeTx, ProviderControllerLike, ProviderCredentialControllerLike}, providers::google_mail::GoogleMailProviderController, repository::ProviderRepositoryLike
};

pub fn schemars_make_password(schema: &mut schemars::Schema) {
    schema.insert(String::from("format"), Value::String("password".to_string()));
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
pub enum BrokerAction {
    Redirect { url: String },
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
    pub data_encryption_key_id: String,
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
    pub data_encryption_key_id: String,
}



#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct CreateResourceServerCredentialParamsInner {
    // NOTE: serialized values are always already encrypted, only encrypt_provider_configuration accepts raw values
    pub resource_server_configuration: WrappedJsonValue,
    pub metadata: Option<Metadata>,
    pub data_encryption_key_id: String,
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

    let next_rotation_time = if let Some(resource_server_credential) =
        resource_server_credential.as_rotateable_credential()
    {
        Some(resource_server_credential.next_rotation_time())
    } else {
        None
    };

    let now = WrappedChronoDateTime::now();
    let resource_server_credential_serialized = ResourceServerCredentialSerialized {
        id: WrappedUuidV4::new(),
        type_id: resource_server_credential.type_id().to_string(),
        metadata: core_metadata,
        value: resource_server_credential.value(),
        created_at: now,
        updated_at: now,
        next_rotation_time: next_rotation_time,
        data_encryption_key_id: params.inner.inner.data_encryption_key_id,
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
    pub data_encryption_key_id: String,
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

    let next_rotation_time =
        if let Some(user_credential) = user_credential.as_rotateable_credential() {
            Some(user_credential.next_rotation_time())
        } else {
            None
        };

    let now = WrappedChronoDateTime::now();
    let user_credential_serialized = UserCredentialSerialized {
        id: WrappedUuidV4::new(),
        type_id: user_credential.type_id().to_string(),
        metadata: core_metadata,
        value: user_credential.value(),
        created_at: now,
        updated_at: now,
        next_rotation_time: next_rotation_time,
        data_encryption_key_id: params.inner.inner.data_encryption_key_id,
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
                .get_resource_server_credential_by_id(&provider_instance.resource_server_credential_id)
                .await?;

            let resource_server_cred = match resource_server_cred {
                Some(resource_server_cred) => resource_server_cred,
                None => {
                    return Err(CommonError::Unknown(anyhow::anyhow!(
                        "Resource server credential not found"
                    )));
                }
            };
            let data_encryption_key_id = resource_server_cred.data_encryption_key_id;
            let user_credential = create_user_credential(
                repo,
                CreateUserCredentialParams {
                    provider_controller_type_id: provider_controller.type_id().to_string(),
                    inner: WithCredentialControllerTypeId {
                        credential_controller_type_id: credential_controller.type_id().to_string(),
                        inner: CreateUserCredentialParamsInner {
                            data_encryption_key_id: data_encryption_key_id,
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
                .ok_or_else(|| CommonError::Unknown(anyhow::anyhow!("Provider instance not found after update")))?;

            let resource_server_cred = repo
                .get_resource_server_credential_by_id(&updated_provider_instance.provider_instance.resource_server_credential_id)
                .await?
                .ok_or_else(|| CommonError::Unknown(anyhow::anyhow!("Resource server credential not found")))?;

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
                .await?;

            if let Some(return_on_success) = &provider_instance.return_on_successful_brokering {
                match return_on_success {
                    ReturnAddress::Url(url) => {
                        return Ok(UserCredentialBrokeringResponse::Redirect(url.url.clone()));
                    }
                }
            }

            UserCredentialBrokeringResponse::UserCredential(user_credential)
        }
        BrokerOutcome::Continue { state_metadata, state_id } => {
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
        .get_resource_server_credential_by_id(&provider_instance.provider_instance.resource_server_credential_id)
        .await?
        .ok_or(CommonError::Unknown(anyhow::anyhow!(
            "Resource server credential not found"
        )))?;

    let (inner, metadata) = credential_controller
        .from_serialized_resource_server_configuration(resource_server_cred.value)?;
    let resource_server_cred = Credential {
        inner: inner,
        metadata: metadata,
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

pub async fn resume_user_credential_brokering(
    on_config_change_tx: &OnConfigChangeTx,
    repo: &impl crate::repository::ProviderRepositoryLike,
    key_encryption_key_contents: &EnvelopeEncryptionKeyContents,
    params: ResumeUserCredentialBrokeringParams,
) -> Result<UserCredentialBrokeringResponse, CommonError> {
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
        .get_resource_server_credential_by_id(&provider_instance.provider_instance.resource_server_credential_id)
        .await?
        .ok_or(CommonError::Unknown(anyhow::anyhow!(
            "Resource server credential not found for provider instance"
        )))?;

    let crypto_service = get_crypto_service(
        key_encryption_key_contents,
        repo,
        &resource_server_cred.data_encryption_key_id,
    )
    .await?;
    let encryption_service = get_encryption_service(&crypto_service)?;
    let decryption_service = get_decryption_service(&crypto_service)?;
    let (action, outcome) = user_credential_broker
        .resume(&decryption_service, &encryption_service, &broker_state, params.input, &resource_server_cred)
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
