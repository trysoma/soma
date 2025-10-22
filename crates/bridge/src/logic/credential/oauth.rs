use async_trait::async_trait;
use http::HeaderValue;
use reqwest::Request;
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use shared::{
    error::CommonError,
    primitives::{WrappedChronoDateTime, WrappedJsonValue, WrappedSchema},
};
use std::collections::HashMap;

use crate::logic::{
    schemars_make_password, BrokerAction, BrokerInput, BrokerOutcome, BrokerState, ConfigurationSchema, Credential, DecryptionService, EncryptedString, EncryptionService, Metadata, ProviderCredentialControllerLike, ResourceServerCredentialLike, ResourceServerCredentialSerialized, RotateableControllerUserCredentialLike, RotateableCredentialLike, StaticCredentialConfigurationLike, StaticProviderCredentialControllerLike, UserCredentialBrokerLike, UserCredentialLike, UserCredentialSerialized
};

// ============================================================================
// Static Credential Configuration
// ============================================================================


#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct Oauth2AuthorizationCodeFlowStaticCredentialConfiguration {
    pub auth_uri: String,
    pub token_uri: String,
    pub userinfo_uri: String,
    pub jwks_uri: String,
    pub issuer: String,
    pub scopes: Vec<String>,
    pub metadata: Metadata,
}

impl StaticCredentialConfigurationLike
    for Oauth2AuthorizationCodeFlowStaticCredentialConfiguration
{
    fn type_id(&self) -> &'static str {
        "static_oauth2_authorization_code_flow"
    }
    fn value(&self) -> WrappedJsonValue {
        WrappedJsonValue::new(json!(self))
    }
}

// ============================================================================
// Resource Server Credentials (Client Credentials)
// ============================================================================

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct Oauth2AuthorizationCodeFlowResourceServerCredential {
    pub client_id: String,
    #[schemars(transform = schemars_make_password)]
    pub client_secret: EncryptedString,
    pub redirect_uri: String,
    #[serde(default)]
    #[schemars(skip)]
    pub metadata: Metadata,
}

impl ResourceServerCredentialLike for Oauth2AuthorizationCodeFlowResourceServerCredential {
    fn type_id(&self) -> &'static str {
        "oauth2_authorization_code_flow_resource_server"
    }

    fn value(&self) -> WrappedJsonValue {
        WrappedJsonValue::new(json!(self))
    }
}

// ============================================================================
// User Credentials (Access Token + Refresh Token)
// ============================================================================

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct Oauth2AuthorizationCodeFlowUserCredential {
    #[schemars(transform = schemars_make_password)]
    pub code: EncryptedString,
    #[schemars(transform = schemars_make_password)]
    pub access_token: EncryptedString,
    #[schemars(transform = schemars_make_password)]
    pub refresh_token: EncryptedString,
    pub expiry_time: WrappedChronoDateTime,
    pub sub: String,
    pub scopes: Vec<String>,
    pub metadata: Metadata,
}


#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct DecryptedOauthCredentials {
    #[schemars(transform = schemars_make_password)]
    pub code: String,
    #[schemars(transform = schemars_make_password)]
    pub access_token: String,
    #[schemars(transform = schemars_make_password)]
    pub refresh_token: String,
    pub expiry_time: WrappedChronoDateTime,
    pub sub: String,
    pub scopes: Vec<String>,
    pub metadata: Metadata,
}

impl UserCredentialLike for Oauth2AuthorizationCodeFlowUserCredential {
    fn type_id(&self) -> &'static str {
        "oauth2_authorization_code_flow_user"
    }

    fn value(&self) -> WrappedJsonValue {
        WrappedJsonValue::new(json!(self))
    }

    fn as_rotateable_credential(&self) -> Option<&dyn RotateableCredentialLike> {
        Some(self)
    }
}

impl RotateableCredentialLike for Oauth2AuthorizationCodeFlowUserCredential {
    fn next_rotation_time(&self) -> WrappedChronoDateTime {
        // Rotate token 5 minutes before expiry to be safe
        self.expiry_time
            .get_inner()
            .checked_sub_signed(chrono::Duration::minutes(5))
            .map(WrappedChronoDateTime::new)
            .unwrap_or(self.expiry_time)
    }
}

// ============================================================================
// OAuth Authorization Code Flow Controller
// ============================================================================

pub struct OauthAuthFlowController {
    pub static_credentials: Oauth2AuthorizationCodeFlowStaticCredentialConfiguration,
}

const STATIC_TYPE_ID_OAUTH_AUTH_FLOW: &str = "oauth_auth_flow";

impl OauthAuthFlowController {

    pub async fn decrypt_oauth_credentials(
        &self,
        crypto_service: &DecryptionService,
        user_credential: &UserCredentialSerialized,
    ) -> Result<DecryptedOauthCredentials, CommonError> {
        let typed_creds: Oauth2AuthorizationCodeFlowUserCredential = serde_json::from_value(user_credential.value.clone().into())?;
        let decrypted_creds: DecryptedOauthCredentials = DecryptedOauthCredentials {
            code: crypto_service.decrypt_data(typed_creds.code).await?,
            access_token: crypto_service.decrypt_data(typed_creds.access_token).await?,
            refresh_token: crypto_service.decrypt_data(typed_creds.refresh_token).await?,
            expiry_time: typed_creds.expiry_time,
            sub: typed_creds.sub,
            scopes: typed_creds.scopes,
            metadata: typed_creds.metadata,
        };

        Ok(decrypted_creds)
    }
}

impl StaticProviderCredentialControllerLike for OauthAuthFlowController {
    fn static_type_id() -> &'static str {
        STATIC_TYPE_ID_OAUTH_AUTH_FLOW
    }
}

#[async_trait]
impl ProviderCredentialControllerLike for OauthAuthFlowController {
    fn static_credentials(&self) -> Box<dyn StaticCredentialConfigurationLike> {
        Box::new(self.static_credentials.clone())
    }

    fn type_id(&self) -> &'static str {
        STATIC_TYPE_ID_OAUTH_AUTH_FLOW
    }

    fn documentation(&self) -> &'static str {
        "OAuth 2.0 Authorization Code Flow - Standard OAuth flow for web applications"
    }

    fn name(&self) -> &'static str {
        "OAuth Authorization Code Flow"
    }

    fn configuration_schema(&self) -> ConfigurationSchema {
        ConfigurationSchema {
            resource_server: WrappedSchema::new(
                schema_for!(Oauth2AuthorizationCodeFlowResourceServerCredential).into(),
            ),
            user_credential: WrappedSchema::new(
                schema_for!(Oauth2AuthorizationCodeFlowUserCredential).into(),
            ),
        }
    }

    async fn encrypt_resource_server_configuration(
        &self,
        crypto_service: &EncryptionService,
        raw_resource_server_configuration: WrappedJsonValue,
    ) -> Result<Box<dyn ResourceServerCredentialLike>, CommonError> {
        // Parse the raw configuration
        let mut config: Oauth2AuthorizationCodeFlowResourceServerCredential =
            serde_json::from_value(raw_resource_server_configuration.into())?;

        // Encrypt the client secret
        config.client_secret =
            EncryptedString(crypto_service.encrypt_data(config.client_secret.0).await?.0);

        Ok(Box::new(config))
    }

    async fn encrypt_user_credential_configuration(
        &self,
        crypto_service: &EncryptionService,
        raw_user_credential_configuration: WrappedJsonValue,
    ) -> Result<Box<dyn UserCredentialLike>, CommonError> {
        // Parse the raw configuration
        let mut config: Oauth2AuthorizationCodeFlowUserCredential =
            serde_json::from_value(raw_user_credential_configuration.into())?;

        // Encrypt sensitive fields
        config.code = EncryptedString(crypto_service.encrypt_data(config.code.0).await?.0);
        config.access_token =
            EncryptedString(crypto_service.encrypt_data(config.access_token.0).await?.0);
        config.refresh_token =
            EncryptedString(crypto_service.encrypt_data(config.refresh_token.0).await?.0);

        Ok(Box::new(config))
    }

    fn from_serialized_resource_server_configuration(
        &self,
        raw_resource_server_configuration: WrappedJsonValue,
    ) -> Result<(Box<dyn ResourceServerCredentialLike>, Metadata), CommonError> {
        let config: Oauth2AuthorizationCodeFlowResourceServerCredential =
            serde_json::from_value(raw_resource_server_configuration.into())?;

        let metadata = config.metadata.clone();
        Ok((Box::new(config), metadata))
    }

    fn from_serialized_user_credential_configuration(
        &self,
        raw_user_credential_configuration: WrappedJsonValue,
    ) -> Result<(Box<dyn UserCredentialLike>, Metadata), CommonError> {
        let config: Oauth2AuthorizationCodeFlowUserCredential =
            serde_json::from_value(raw_user_credential_configuration.into())?;

        let metadata = config.metadata.clone();
        Ok((Box::new(config), metadata))
    }


    fn as_rotateable_controller_user_credential(
        &self,
    ) -> Option<&dyn RotateableControllerUserCredentialLike> {
        Some(self)
    }

    fn as_user_credential_broker(&self) -> Option<&dyn UserCredentialBrokerLike> {
        Some(self)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// ============================================================================
// User Credential Brokering (OAuth Flow)
// ============================================================================

#[async_trait]
impl UserCredentialBrokerLike for OauthAuthFlowController {
    async fn start(
        &self,
        resource_server_cred: &Credential<Box<dyn ResourceServerCredentialLike>>,
    ) -> Result<(BrokerAction, BrokerOutcome), CommonError> {
        // Deserialize the resource server credential
        let config: Oauth2AuthorizationCodeFlowResourceServerCredential =
            serde_json::from_value(resource_server_cred.inner.value().into())?;

        // Parse static credential to get OAuth endpoints
        // For now, we'll construct the authorization URL
        // In a real implementation, you'd get this from the static credential
        let state_id = uuid::Uuid::new_v4().to_string();
        let auth_url = format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
            self.static_credentials.auth_uri,
            urlencoding::encode(&config.client_id),
            urlencoding::encode(&config.redirect_uri),
            self.static_credentials.scopes.join(" "),
            state_id
        );

        let action = BrokerAction::Redirect { url: auth_url };

        // We need to wait for the callback with the authorization code
        let outcome = BrokerOutcome::Continue {
            state_metadata: config.metadata.clone(),
            state_id: state_id,
        };

        Ok((action, outcome))
    }

    async fn resume(
        &self,
        decryption_service: &DecryptionService,
        encryption_service: &EncryptionService,
        state: &BrokerState,
        input: BrokerInput,
        resource_server_cred: &ResourceServerCredentialSerialized,
    ) -> Result<(BrokerAction, BrokerOutcome), CommonError> {
        // Extract the authorization code from the input
        let code = match input {
            BrokerInput::Oauth2AuthorizationCodeFlow { code } => code,
            BrokerInput::Oauth2AuthorizationCodeFlowWithPkce { code, .. } => code,
        };


        // Deserialize and decrypt the resource server credential
        let config: Oauth2AuthorizationCodeFlowResourceServerCredential = serde_json::from_value(resource_server_cred.value.clone().into())?;
        
        let client_secret = decryption_service.decrypt_data(config.client_secret).await?;

        // Exchange authorization code for access token
        let client = reqwest::Client::new();
        let token_response = client
            .post(&self.static_credentials.token_uri)
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", &code),
                ("client_id", &config.client_id),
                ("client_secret", &client_secret),
                ("redirect_uri", &config.redirect_uri),
            ])
            .send()
            .await
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Token exchange failed: {}", e)))?;

        let token_status = token_response.status();
        if !token_status.is_success() {
            let error_text = token_response.text().await.unwrap_or_default();
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "Token exchange failed with status {}: {}",
                token_status,
                error_text
            )));
        }

        #[derive(Deserialize)]
        struct TokenResponse {
            access_token: String,
            refresh_token: Option<String>,
            expires_in: i64,
            #[serde(default)]
            scope: String,
            #[serde(default)]
            id_token: Option<String>,
        }

        let token_data: TokenResponse = token_response
            .json()
            .await
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to parse token response: {}", e)))?;

        // Fetch user info to get the subject ID
        let userinfo_response = client
            .get(&self.static_credentials.userinfo_uri)
            .bearer_auth(&token_data.access_token)
            .send()
            .await
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Userinfo request failed: {}", e)))?;

        #[derive(Deserialize)]
        struct UserinfoResponse {
            sub: String,
        }

        let userinfo: UserinfoResponse = userinfo_response
            .json()
            .await
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to parse userinfo response: {}", e)))?;

        // Calculate expiry time
        let now = WrappedChronoDateTime::now();
        let expiry = now
            .get_inner()
            .checked_add_signed(chrono::Duration::seconds(token_data.expires_in))
            .map(WrappedChronoDateTime::new)
            .unwrap_or(now);

        // Parse scopes
        let scopes: Vec<String> = token_data.scope
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        // Encrypt the sensitive data
        let encrypted_code = encryption_service.encrypt_data(code).await?;
        let encrypted_access_token = encryption_service.encrypt_data(token_data.access_token).await?;
        let encrypted_refresh_token = encryption_service
            .encrypt_data(token_data.refresh_token.unwrap_or_default())
            .await?;

        let user_credential = Oauth2AuthorizationCodeFlowUserCredential {
            code: encrypted_code,
            access_token: encrypted_access_token,
            refresh_token: encrypted_refresh_token,
            expiry_time: expiry,
            sub: userinfo.sub,
            scopes,
            metadata: state.metadata.clone(),
        };

        let action = BrokerAction::None;
        let outcome = BrokerOutcome::Success {
            user_credential: Box::new(user_credential),
            metadata: state.metadata.clone(),
        };

        Ok((action, outcome))
    }
}

// ============================================================================
// Token Rotation
// ============================================================================

#[async_trait]
impl RotateableControllerUserCredentialLike for OauthAuthFlowController {
    async fn rotate_user_credential(
        &self,
        _static_credentials: &Box<dyn StaticCredentialConfigurationLike>,
        _resource_server_cred: &ResourceServerCredentialSerialized,
        user_cred: &UserCredentialSerialized,
        decryption_service: &DecryptionService,
        encryption_service: &EncryptionService,
    ) -> Result<Credential<std::sync::Arc<dyn UserCredentialLike>>, CommonError> {
        // // Deserialize the user credential
        // let mut config: Oauth2AuthorizationCodeFlowUserCredential =
        //     serde_json::from_value(user_cred.inner.value().into())?;

        // // TODO: Use the refresh token to get a new access token
        // // This would involve making an HTTP request to the token endpoint
        // // For now, we'll just extend the expiry time

        // let now = WrappedChronoDateTime::now();
        // config.expiry_time = now
        //     .get_inner()
        //     .checked_add_signed(chrono::Duration::hours(1))
        //     .map(WrappedChronoDateTime::new)
        //     .unwrap_or(now);

        // // Create a new credential with the refreshed token
        // let rotated_credential = Credential {
        //     inner: std::sync::Arc::new(config) as std::sync::Arc<dyn UserCredentialLike>,
        //     metadata: user_cred.metadata.clone(),
        //     id: user_cred.id.clone(),
        //     created_at: user_cred.created_at.clone(),
        //     updated_at: now,
        // };

        // Ok(rotated_credential)

        todo!()
    }

    async fn next_user_credential_rotation_time(
        &self,
        _static_credentials: &Box<dyn StaticCredentialConfigurationLike>,
        _resource_server_cred: &ResourceServerCredentialSerialized,
        user_cred: &UserCredentialSerialized,
        decryption_service: &DecryptionService,
        encryption_service: &EncryptionService,
    ) -> WrappedChronoDateTime {
        // // Deserialize to get the rotateable credential
        // if let Ok(config) = serde_json::from_value::<Oauth2AuthorizationCodeFlowUserCredential>(
        //     user_cred.inner.value().into(),
        // ) {
        //     config.next_rotation_time()
        // } else {
        //     // Fallback: rotate in 1 hour
        //     WrappedChronoDateTime::now()
        //         .get_inner()
        //         .checked_add_signed(chrono::Duration::hours(1))
        //         .map(WrappedChronoDateTime::new)
        //         .unwrap_or(WrappedChronoDateTime::now())
        // }

        todo!()
    }
}

// ============================================================================
// OAuth 2.0 JWT Bearer Assertion Flow (for service accounts)
// ============================================================================

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration {
    pub auth_uri: String,
    pub token_uri: String,
    pub userinfo_uri: String,
    pub jwks_uri: String,
    pub issuer: String,
    pub scopes: Vec<String>,
    pub metadata: Metadata,
}

impl StaticCredentialConfigurationLike
    for Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration
{
    fn type_id(&self) -> &'static str {
        "static_oauth2_jwt_bearer_assertion_flow"
    }
    fn value(&self) -> WrappedJsonValue {
        WrappedJsonValue::new(json!(self))
    }
}

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct Oauth2JwtBearerAssertionFlowResourceServerCredential {
    pub client_id: String,
    pub private_key: EncryptedString,
    pub token_uri: String,
    #[serde(default)]
    #[schemars(skip)]
    pub metadata: Metadata,
}

impl ResourceServerCredentialLike for Oauth2JwtBearerAssertionFlowResourceServerCredential {
    fn type_id(&self) -> &'static str {
        "oauth2_jwt_bearer_assertion_flow_resource_server"
    }

    fn value(&self) -> WrappedJsonValue {
        WrappedJsonValue::new(json!(self))
    }
}

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct Oauth2JwtBearerAssertionFlowUserCredential {
    pub assertion: EncryptedString,
    pub access_token: EncryptedString,
    pub expiry_time: WrappedChronoDateTime,
    pub sub: String,
    pub scopes: Vec<String>,
    pub metadata: Metadata,
}

impl UserCredentialLike for Oauth2JwtBearerAssertionFlowUserCredential {
    fn type_id(&self) -> &'static str {
        "oauth2_jwt_bearer_assertion_flow_user"
    }

    fn value(&self) -> WrappedJsonValue {
        WrappedJsonValue::new(json!(self))
    }

    fn as_rotateable_credential(&self) -> Option<&dyn RotateableCredentialLike> {
        Some(self)
    }
}

impl RotateableCredentialLike for Oauth2JwtBearerAssertionFlowUserCredential {
    fn next_rotation_time(&self) -> WrappedChronoDateTime {
        // Rotate token 5 minutes before expiry
        self.expiry_time
            .get_inner()
            .checked_sub_signed(chrono::Duration::minutes(5))
            .map(WrappedChronoDateTime::new)
            .unwrap_or(self.expiry_time)
    }
}

// ============================================================================
// OAuth JWT Bearer Assertion Flow Controller
// ============================================================================

pub struct Oauth2JwtBearerAssertionFlowController {
    pub static_credentials: Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration,
}


impl Oauth2JwtBearerAssertionFlowController {

    pub async fn decrypt_oauth_credentials(
        &self,
        crypto_service: &DecryptionService,
        user_credential: &UserCredentialSerialized,
    ) -> Result<DecryptedOauthCredentials, CommonError> {
        let typed_creds: Oauth2AuthorizationCodeFlowUserCredential = serde_json::from_value(user_credential.value.clone().into())?;
        let decrypted_creds: DecryptedOauthCredentials = DecryptedOauthCredentials {
            code: crypto_service.decrypt_data(typed_creds.code).await?,
            access_token: crypto_service.decrypt_data(typed_creds.access_token).await?,
            refresh_token: crypto_service.decrypt_data(typed_creds.refresh_token).await?,
            expiry_time: typed_creds.expiry_time,
            sub: typed_creds.sub,
            scopes: typed_creds.scopes,
            metadata: typed_creds.metadata,
        };

        Ok(decrypted_creds)
    }
}

const STATIC_TYPE_ID_OAUTH2_JWT_BEARER_ASSERTION_FLOW: &str = "oauth2_jwt_bearer_assertion_flow";

impl StaticProviderCredentialControllerLike for Oauth2JwtBearerAssertionFlowController {
    fn static_type_id() -> &'static str {
        STATIC_TYPE_ID_OAUTH2_JWT_BEARER_ASSERTION_FLOW
    }
}

#[async_trait]
impl ProviderCredentialControllerLike for Oauth2JwtBearerAssertionFlowController {
    fn static_credentials(&self) -> Box<dyn StaticCredentialConfigurationLike> {
        Box::new(self.static_credentials.clone())
    }

    fn type_id(&self) -> &'static str {
        STATIC_TYPE_ID_OAUTH2_JWT_BEARER_ASSERTION_FLOW
    }

    fn documentation(&self) -> &'static str {
        "OAuth 2.0 JWT Bearer Assertion Flow - Service account authentication for server-to-server applications"
    }

    fn name(&self) -> &'static str {
        "OAuth JWT Bearer Assertion Flow"
    }

    fn configuration_schema(&self) -> ConfigurationSchema {
        ConfigurationSchema {
            resource_server: WrappedSchema::new(
                schema_for!(Oauth2JwtBearerAssertionFlowResourceServerCredential).into(),
            ),
            user_credential: WrappedSchema::new(
                schema_for!(Oauth2JwtBearerAssertionFlowUserCredential).into(),
            ),
        }
    }

    async fn encrypt_resource_server_configuration(
        &self,
        crypto_service: &EncryptionService,
        raw_resource_server_configuration: WrappedJsonValue,
    ) -> Result<Box<dyn ResourceServerCredentialLike>, CommonError> {
        let mut config: Oauth2JwtBearerAssertionFlowResourceServerCredential =
            serde_json::from_value(raw_resource_server_configuration.into())?;

        // Encrypt the private key
        config.private_key =
            EncryptedString(crypto_service.encrypt_data(config.private_key.0).await?.0);

        Ok(Box::new(config))
    }

    async fn encrypt_user_credential_configuration(
        &self,
        crypto_service: &EncryptionService,
        raw_user_credential_configuration: WrappedJsonValue,
    ) -> Result<Box<dyn UserCredentialLike>, CommonError> {
        let mut config: Oauth2JwtBearerAssertionFlowUserCredential =
            serde_json::from_value(raw_user_credential_configuration.into())?;

        // Encrypt sensitive fields
        config.assertion =
            EncryptedString(crypto_service.encrypt_data(config.assertion.0).await?.0);
        config.access_token =
            EncryptedString(crypto_service.encrypt_data(config.access_token.0).await?.0);

        Ok(Box::new(config))
    }

    fn from_serialized_resource_server_configuration(
        &self,
        raw_resource_server_configuration: WrappedJsonValue,
    ) -> Result<(Box<dyn ResourceServerCredentialLike>, Metadata), CommonError> {
        let config: Oauth2JwtBearerAssertionFlowResourceServerCredential =
            serde_json::from_value(raw_resource_server_configuration.into())?;

        let metadata = config.metadata.clone();
        Ok((Box::new(config), metadata))
    }

    fn from_serialized_user_credential_configuration(
        &self,
        raw_user_credential_configuration: WrappedJsonValue,
    ) -> Result<(Box<dyn UserCredentialLike>, Metadata), CommonError> {
        let config: Oauth2JwtBearerAssertionFlowUserCredential =
            serde_json::from_value(raw_user_credential_configuration.into())?;

        let metadata = config.metadata.clone();
        Ok((Box::new(config), metadata))
    }

    fn as_rotateable_controller_user_credential(
        &self,
    ) -> Option<&dyn RotateableControllerUserCredentialLike> {
        Some(self)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// ============================================================================
// JWT Bearer Assertion Token Rotation
// ============================================================================

#[async_trait]
impl RotateableControllerUserCredentialLike for Oauth2JwtBearerAssertionFlowController {
    async fn rotate_user_credential(
        &self,
        _static_credentials: &Box<dyn StaticCredentialConfigurationLike>,
        _resource_server_cred: &ResourceServerCredentialSerialized,
        user_cred: &UserCredentialSerialized,
        decryption_service: &DecryptionService,
        encryption_service: &EncryptionService,
    ) -> Result<Credential<std::sync::Arc<dyn UserCredentialLike>>, CommonError> {
        // let mut config: Oauth2JwtBearerAssertionFlowUserCredential =
        //     serde_json::from_value(user_cred.inner.value().into())?;

        // TODO: Generate a new JWT assertion and exchange it for a new access token
        // This would involve:
        // 1. Creating a new JWT assertion using the private key
        // 2. Making an HTTP request to the token endpoint
        // For now, we'll just extend the expiry time

        // let now = WrappedChronoDateTime::now();
        // config.expiry_time = now
        //     .get_inner()
        //     .checked_add_signed(chrono::Duration::hours(1))
        //     .map(WrappedChronoDateTime::new)
        //     .unwrap_or(now);

        // let rotated_credential = Credential {
        //     inner: std::sync::Arc::new(config) as std::sync::Arc<dyn UserCredentialLike>,
        //     metadata: user_cred.metadata.clone(),
        //     id: user_cred.id.clone(),
        //     created_at: user_cred.created_at.clone(),
        //     updated_at: now,
        // };

        // Ok(rotated_credential)

        todo!()
    }

    async fn next_user_credential_rotation_time(
        &self,
        _static_credentials: &Box<dyn StaticCredentialConfigurationLike>,
        _resource_server_cred: &ResourceServerCredentialSerialized,
        user_cred: &UserCredentialSerialized,
        decryption_service: &DecryptionService,
        encryption_service: &EncryptionService,
    ) -> WrappedChronoDateTime {
        todo!()
    }
}
