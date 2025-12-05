use ::encryption::logic::crypto_services::{DecryptionService, EncryptedString, EncryptionService};
use async_trait::async_trait;
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::json;
use shared::{
    error::CommonError,
    primitives::{WrappedChronoDateTime, WrappedJsonValue, WrappedSchema},
};

use crate::logic::{
    BrokerAction, BrokerActionRedirect, BrokerInput, BrokerOutcome, BrokerState,
    ConfigurationSchema, Credential, Metadata, ProviderCredentialControllerLike,
    ResourceServerCredentialLike, ResourceServerCredentialSerialized,
    RotateableControllerUserCredentialLike, RotateableCredentialLike,
    StaticCredentialConfigurationLike, StaticProviderCredentialControllerLike,
    UserCredentialBrokerLike, UserCredentialLike, UserCredentialSerialized, schemars_make_password,
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
        let typed_creds: Oauth2AuthorizationCodeFlowUserCredential =
            serde_json::from_value(user_credential.value.clone().into())?;
        let decrypted_creds: DecryptedOauthCredentials = DecryptedOauthCredentials {
            code: crypto_service.decrypt_data(typed_creds.code).await?,
            access_token: crypto_service
                .decrypt_data(typed_creds.access_token)
                .await?,
            refresh_token: crypto_service
                .decrypt_data(typed_creds.refresh_token)
                .await?,
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
    fn static_credentials(&self) -> &dyn StaticCredentialConfigurationLike {
        &self.static_credentials
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
            resource_server: WrappedSchema::new(schema_for!(
                Oauth2AuthorizationCodeFlowResourceServerCredential
            )),
            user_credential: WrappedSchema::new(schema_for!(
                Oauth2AuthorizationCodeFlowUserCredential
            )),
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
        // Construct the authorization URL with offline access to get a refresh token
        let state_id = uuid::Uuid::new_v4().to_string();
        let auth_url = format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}&access_type=offline&prompt=consent",
            self.static_credentials.auth_uri,
            urlencoding::encode(&config.client_id),
            urlencoding::encode(&config.redirect_uri),
            urlencoding::encode(&self.static_credentials.scopes.join(" ")),
            state_id
        );

        let action = BrokerAction::Redirect(BrokerActionRedirect { url: auth_url });

        // We need to wait for the callback with the authorization code
        let outcome = BrokerOutcome::Continue {
            state_metadata: config.metadata.clone(),
            state_id,
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
        let config: Oauth2AuthorizationCodeFlowResourceServerCredential =
            serde_json::from_value(resource_server_cred.value.clone().into())?;

        let client_secret = decryption_service
            .decrypt_data(config.client_secret)
            .await?;

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
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Token exchange failed: {e}")))?;

        let token_status = token_response.status();
        if !token_status.is_success() {
            let error_text = token_response.text().await.unwrap_or_default();
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "Token exchange failed with status {token_status}: {error_text}"
            )));
        }

        #[derive(Deserialize)]
        struct TokenResponse {
            access_token: String,
            refresh_token: Option<String>,
            expires_in: i64,
            #[serde(default)]
            scope: String,
        }

        let token_data: TokenResponse = token_response.json().await.map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!("Failed to parse token response: {e}"))
        })?;

        // Fetch user info to get the subject ID
        let userinfo_response = client
            .get(&self.static_credentials.userinfo_uri)
            .bearer_auth(&token_data.access_token)
            .send()
            .await
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Userinfo request failed: {e}")))?;

        #[derive(Deserialize)]
        struct UserinfoResponse {
            sub: String,
        }

        let userinfo: UserinfoResponse = userinfo_response.json().await.map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!("Failed to parse userinfo response: {e}"))
        })?;

        // Calculate expiry time
        let now = WrappedChronoDateTime::now();
        let expiry = now
            .get_inner()
            .checked_add_signed(chrono::Duration::seconds(token_data.expires_in))
            .map(WrappedChronoDateTime::new)
            .unwrap_or(now);

        // Parse scopes
        let scopes: Vec<String> = token_data
            .scope
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        // Check if we received a refresh token - this is critical for rotation
        let refresh_token = token_data.refresh_token.ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!(
                "OAuth provider did not return a refresh token. This credential cannot be rotated automatically. \
                 Ensure the OAuth authorization includes 'access_type=offline' and 'prompt=consent' parameters."
            ))
        })?;

        // Encrypt the sensitive data
        let encrypted_code = encryption_service.encrypt_data(code).await?;
        let encrypted_access_token = encryption_service
            .encrypt_data(token_data.access_token)
            .await?;
        let encrypted_refresh_token = encryption_service.encrypt_data(refresh_token).await?;

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
        decryption_service: &DecryptionService,
        encryption_service: &EncryptionService,
        _static_credentials: &dyn StaticCredentialConfigurationLike,
        resource_server_cred: &ResourceServerCredentialSerialized,
        user_cred: &UserCredentialSerialized,
    ) -> Result<UserCredentialSerialized, CommonError> {
        // Deserialize the user credential
        let current_cred: Oauth2AuthorizationCodeFlowUserCredential =
            serde_json::from_value(user_cred.value.clone().into())?;

        // Decrypt the refresh token
        let refresh_token = decryption_service
            .decrypt_data(current_cred.refresh_token.clone())
            .await?;

        // Check if refresh token is empty - if so, we cannot rotate
        if refresh_token.is_empty() {
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "Cannot rotate user credential {} - no refresh token available. This credential must be re-authorized through the OAuth flow.",
                user_cred.id
            )));
        }

        // Deserialize and decrypt the resource server credential
        let resource_server_config: Oauth2AuthorizationCodeFlowResourceServerCredential =
            serde_json::from_value(resource_server_cred.value.clone().into())?;
        let client_secret = decryption_service
            .decrypt_data(resource_server_config.client_secret)
            .await?;

        tracing::debug!(
            "Rotating user credential - client_id: {}, token_uri: {}, refresh_token_len: {}",
            resource_server_config.client_id,
            self.static_credentials.token_uri,
            refresh_token.len()
        );

        // Use the refresh token to get a new access token
        let client = reqwest::Client::new();
        let token_response = client
            .post(&self.static_credentials.token_uri)
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", &refresh_token),
                ("client_id", &resource_server_config.client_id),
                ("client_secret", &client_secret),
            ])
            .send()
            .await
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Token refresh failed: {e}")))?;

        let token_status = token_response.status();
        if !token_status.is_success() {
            let error_text = token_response.text().await.unwrap_or_default();
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "Token refresh failed with status {token_status}: {error_text}"
            )));
        }

        #[derive(Deserialize)]
        struct TokenResponse {
            access_token: String,
            refresh_token: Option<String>,
            expires_in: i64,
            #[serde(default)]
            scope: String,
        }

        let token_data: TokenResponse = token_response.json().await.map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!("Failed to parse token response: {e}"))
        })?;

        // Calculate new expiry time
        let now = WrappedChronoDateTime::now();
        let expiry = now
            .get_inner()
            .checked_add_signed(chrono::Duration::seconds(token_data.expires_in))
            .map(WrappedChronoDateTime::new)
            .unwrap_or(now);

        // Parse scopes (use new scopes if provided, otherwise keep existing ones)
        let scopes: Vec<String> = if !token_data.scope.is_empty() {
            token_data
                .scope
                .split_whitespace()
                .map(|s| s.to_string())
                .collect()
        } else {
            current_cred.scopes.clone()
        };

        // Encrypt the new tokens
        let encrypted_access_token = encryption_service
            .encrypt_data(token_data.access_token)
            .await?;
        let encrypted_refresh_token = encryption_service
            .encrypt_data(token_data.refresh_token.unwrap_or(refresh_token))
            .await?;

        // Create the updated credential
        let updated_credential = Oauth2AuthorizationCodeFlowUserCredential {
            code: current_cred.code.clone(), // Keep the original code
            access_token: encrypted_access_token,
            refresh_token: encrypted_refresh_token,
            expiry_time: expiry,
            sub: current_cred.sub.clone(),
            scopes,
            metadata: current_cred.metadata.clone(),
        };

        // Calculate next rotation time
        let next_rotation_time = updated_credential.next_rotation_time();

        // Create the serialized result
        let serialized = UserCredentialSerialized {
            id: user_cred.id.clone(),
            type_id: user_cred.type_id.clone(),
            metadata: updated_credential.metadata.clone(),
            value: WrappedJsonValue::new(json!(updated_credential)),
            created_at: user_cred.created_at,
            updated_at: now,
            next_rotation_time: Some(next_rotation_time),
            dek_alias: user_cred.dek_alias.clone(),
        };

        Ok(serialized)
    }

    async fn next_user_credential_rotation_time(
        &self,
        _static_credentials: &dyn StaticCredentialConfigurationLike,
        _resource_server_cred: &ResourceServerCredentialSerialized,
        user_cred: &UserCredentialSerialized,
        _decryption_service: &DecryptionService,
        _encryption_service: &EncryptionService,
    ) -> Result<WrappedChronoDateTime, CommonError> {
        // Deserialize to get the rotateable credential
        let config: Oauth2AuthorizationCodeFlowUserCredential =
            serde_json::from_value(user_cred.value.clone().into())?;

        Ok(config.next_rotation_time())
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
        let typed_creds: Oauth2AuthorizationCodeFlowUserCredential =
            serde_json::from_value(user_credential.value.clone().into())?;
        let decrypted_creds: DecryptedOauthCredentials = DecryptedOauthCredentials {
            code: crypto_service.decrypt_data(typed_creds.code).await?,
            access_token: crypto_service
                .decrypt_data(typed_creds.access_token)
                .await?,
            refresh_token: crypto_service
                .decrypt_data(typed_creds.refresh_token)
                .await?,
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
    fn static_credentials(&self) -> &dyn StaticCredentialConfigurationLike {
        &self.static_credentials
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
            resource_server: WrappedSchema::new(schema_for!(
                Oauth2JwtBearerAssertionFlowResourceServerCredential
            )),
            user_credential: WrappedSchema::new(schema_for!(
                Oauth2JwtBearerAssertionFlowUserCredential
            )),
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
        decryption_service: &DecryptionService,
        encryption_service: &EncryptionService,
        _static_credentials: &dyn StaticCredentialConfigurationLike,
        resource_server_cred: &ResourceServerCredentialSerialized,
        user_cred: &UserCredentialSerialized,
    ) -> Result<UserCredentialSerialized, CommonError> {
        // Deserialize the user credential
        let current_cred: Oauth2JwtBearerAssertionFlowUserCredential =
            serde_json::from_value(user_cred.value.clone().into())?;

        // Deserialize and decrypt the resource server credential
        let resource_server_config: Oauth2JwtBearerAssertionFlowResourceServerCredential =
            serde_json::from_value(resource_server_cred.value.clone().into())?;
        let private_key_pem = decryption_service
            .decrypt_data(resource_server_config.private_key)
            .await?;

        // Generate JWT assertion
        use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};

        #[derive(Serialize)]
        struct Claims {
            iss: String,
            sub: String,
            aud: String,
            exp: i64,
            iat: i64,
        }

        let now = chrono::Utc::now().timestamp();
        let claims = Claims {
            iss: resource_server_config.client_id.clone(),
            sub: current_cred.sub.clone(),
            aud: resource_server_config.token_uri.clone(),
            exp: now + 3600, // 1 hour from now
            iat: now,
        };

        let encoding_key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes()).map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!("Failed to parse private key: {e}"))
        })?;

        let header = Header::new(Algorithm::RS256);
        let jwt = encode(&header, &claims, &encoding_key)
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to generate JWT: {e}")))?;

        // Exchange JWT for access token
        let client = reqwest::Client::new();
        let token_response = client
            .post(&resource_server_config.token_uri)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", &jwt),
            ])
            .send()
            .await
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Token exchange failed: {e}")))?;

        let token_status = token_response.status();
        if !token_status.is_success() {
            let error_text = token_response.text().await.unwrap_or_default();
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "Token exchange failed with status {token_status}: {error_text}"
            )));
        }

        #[derive(Deserialize)]
        struct TokenResponse {
            access_token: String,
            expires_in: i64,
            #[serde(default)]
            scope: String,
        }

        let token_data: TokenResponse = token_response.json().await.map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!("Failed to parse token response: {e}"))
        })?;

        // Calculate new expiry time
        let now_dt = WrappedChronoDateTime::now();
        let expiry = now_dt
            .get_inner()
            .checked_add_signed(chrono::Duration::seconds(token_data.expires_in))
            .map(WrappedChronoDateTime::new)
            .unwrap_or(now_dt);

        // Parse scopes (use new scopes if provided, otherwise keep existing ones)
        let scopes: Vec<String> = if !token_data.scope.is_empty() {
            token_data
                .scope
                .split_whitespace()
                .map(|s| s.to_string())
                .collect()
        } else {
            current_cred.scopes.clone()
        };

        // Encrypt the new tokens
        let encrypted_assertion = encryption_service.encrypt_data(jwt).await?;
        let encrypted_access_token = encryption_service
            .encrypt_data(token_data.access_token)
            .await?;

        // Create the updated credential
        let updated_credential = Oauth2JwtBearerAssertionFlowUserCredential {
            assertion: encrypted_assertion,
            access_token: encrypted_access_token,
            expiry_time: expiry,
            sub: current_cred.sub.clone(),
            scopes,
            metadata: current_cred.metadata.clone(),
        };

        // Calculate next rotation time
        let next_rotation_time = updated_credential.next_rotation_time();

        // Create the serialized result
        let serialized = UserCredentialSerialized {
            id: user_cred.id.clone(),
            type_id: user_cred.type_id.clone(),
            metadata: updated_credential.metadata.clone(),
            value: WrappedJsonValue::new(json!(updated_credential)),
            created_at: user_cred.created_at,
            updated_at: now_dt,
            next_rotation_time: Some(next_rotation_time),
            dek_alias: user_cred.dek_alias.clone(),
        };

        Ok(serialized)
    }

    async fn next_user_credential_rotation_time(
        &self,
        _static_credentials: &dyn StaticCredentialConfigurationLike,
        _resource_server_cred: &ResourceServerCredentialSerialized,
        user_cred: &UserCredentialSerialized,
        _decryption_service: &DecryptionService,
        _encryption_service: &EncryptionService,
    ) -> Result<WrappedChronoDateTime, CommonError> {
        // Deserialize to get the rotateable credential
        let config: Oauth2JwtBearerAssertionFlowUserCredential =
            serde_json::from_value(user_cred.value.clone().into())?;

        Ok(config.next_rotation_time())
    }
}

#[cfg(all(test, feature = "unit_test"))]
mod unit_test {
    use super::*;
    use shared::primitives::{SqlMigrationLoader, WrappedUuidV4};

    #[tokio::test]
    async fn test_oauth_authorization_code_flow_next_rotation_time() {
        shared::setup_test!();

        let now = WrappedChronoDateTime::now();
        let expiry_time = WrappedChronoDateTime::new(
            now.get_inner()
                .checked_add_signed(chrono::Duration::hours(1))
                .unwrap(),
        );

        let credential = Oauth2AuthorizationCodeFlowUserCredential {
            code: EncryptedString("encrypted_code".to_string()),
            access_token: EncryptedString("encrypted_token".to_string()),
            refresh_token: EncryptedString("encrypted_refresh".to_string()),
            expiry_time,
            sub: "test-user".to_string(),
            scopes: vec!["scope1".to_string(), "scope2".to_string()],
            metadata: Metadata::new(),
        };

        let next_rotation = credential.next_rotation_time();

        // Should be 5 minutes before expiry
        let expected = WrappedChronoDateTime::new(
            expiry_time
                .get_inner()
                .checked_sub_signed(chrono::Duration::minutes(5))
                .unwrap(),
        );

        assert_eq!(next_rotation.get_inner(), expected.get_inner());
    }

    #[tokio::test]
    async fn test_oauth_authorization_code_flow_rotate_user_credential() {
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
        let decryption_service = setup
            .crypto_cache
            .get_decryption_service(&setup.dek_alias)
            .await
            .unwrap();

        // Create a controller
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

        // Create encrypted resource server credential
        let client_secret = "test-secret";
        let encrypted_secret = encryption_service
            .encrypt_data(client_secret.to_string())
            .await
            .unwrap();

        let resource_server_cred = ResourceServerCredentialSerialized {
            id: WrappedUuidV4::new(),
            type_id: "oauth2_authorization_code_flow_resource_server".to_string(),
            metadata: Metadata::new(),
            value: WrappedJsonValue::new(json!({
                "client_id": "test-client-id",
                "client_secret": encrypted_secret,
                "redirect_uri": "https://example.com/callback",
                "metadata": {}
            })),
            created_at: WrappedChronoDateTime::now(),
            updated_at: WrappedChronoDateTime::now(),
            next_rotation_time: None,
            dek_alias: setup.dek_alias.clone(),
        };

        // Create encrypted user credential
        let code = "test-code";
        let access_token = "old-access-token";
        let refresh_token = "test-refresh-token";

        let encrypted_code = encryption_service
            .encrypt_data(code.to_string())
            .await
            .unwrap();
        let encrypted_access = encryption_service
            .encrypt_data(access_token.to_string())
            .await
            .unwrap();
        let encrypted_refresh = encryption_service
            .encrypt_data(refresh_token.to_string())
            .await
            .unwrap();

        let expiry = WrappedChronoDateTime::new(
            WrappedChronoDateTime::now()
                .get_inner()
                .checked_add_signed(chrono::Duration::minutes(30))
                .unwrap(),
        );

        let user_cred = UserCredentialSerialized {
            id: WrappedUuidV4::new(),
            type_id: "oauth2_authorization_code_flow_user".to_string(),
            metadata: Metadata::new(),
            value: WrappedJsonValue::new(json!({
                "code": encrypted_code,
                "access_token": encrypted_access,
                "refresh_token": encrypted_refresh,
                "expiry_time": expiry,
                "sub": "test-user",
                "scopes": ["scope1", "scope2"],
                "metadata": {}
            })),
            created_at: WrappedChronoDateTime::now(),
            updated_at: WrappedChronoDateTime::now(),
            next_rotation_time: Some(expiry),
            dek_alias: setup.dek_alias.clone(),
        };

        // Note: This test would require mocking the HTTP requests to actually test rotation
        // For now, we verify that the structure is correct
        let static_creds = controller.static_credentials();

        // Verify the controller can be used for rotation
        assert!(
            controller
                .as_rotateable_controller_user_credential()
                .is_some()
        );

        // Verify next rotation time calculation
        let next_rotation = controller
            .next_user_credential_rotation_time(
                static_creds,
                &resource_server_cred,
                &user_cred,
                &decryption_service,
                &encryption_service,
            )
            .await
            .unwrap();

        // Should be 5 minutes before expiry
        let expected = WrappedChronoDateTime::new(
            expiry
                .get_inner()
                .checked_sub_signed(chrono::Duration::minutes(5))
                .unwrap(),
        );

        assert_eq!(next_rotation.get_inner(), expected.get_inner());
    }

    #[tokio::test]
    async fn test_oauth_jwt_bearer_assertion_flow_next_rotation_time() {
        shared::setup_test!();

        let now = WrappedChronoDateTime::now();
        let expiry_time = WrappedChronoDateTime::new(
            now.get_inner()
                .checked_add_signed(chrono::Duration::hours(1))
                .unwrap(),
        );

        let credential = Oauth2JwtBearerAssertionFlowUserCredential {
            assertion: EncryptedString("encrypted_assertion".to_string()),
            access_token: EncryptedString("encrypted_token".to_string()),
            expiry_time,
            sub: "test-user".to_string(),
            scopes: vec!["scope1".to_string(), "scope2".to_string()],
            metadata: Metadata::new(),
        };

        let next_rotation = credential.next_rotation_time();

        // Should be 5 minutes before expiry
        let expected = WrappedChronoDateTime::new(
            expiry_time
                .get_inner()
                .checked_sub_signed(chrono::Duration::minutes(5))
                .unwrap(),
        );

        assert_eq!(next_rotation.get_inner(), expected.get_inner());
    }

    #[tokio::test]
    async fn test_encrypt_decrypt_oauth_resource_server_credentials() {
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

        // Test encrypting resource server credentials
        let raw_config = WrappedJsonValue::new(json!({
            "client_id": "test-client-id",
            "client_secret": "plain-text-secret",
            "redirect_uri": "https://example.com/callback",
            "metadata": {}
        }));

        let encrypted = controller
            .encrypt_resource_server_configuration(&encryption_service, raw_config)
            .await
            .unwrap();

        let value = encrypted.value();
        let config: Oauth2AuthorizationCodeFlowResourceServerCredential =
            serde_json::from_value(value.into()).unwrap();

        // Verify client_id is not encrypted
        assert_eq!(config.client_id, "test-client-id");

        // Verify client_secret is encrypted (should be different from plaintext)
        assert_ne!(config.client_secret.0, "plain-text-secret");

        // Verify it's base64 encoded
        assert!(
            base64::Engine::decode(
                &base64::engine::general_purpose::STANDARD,
                &config.client_secret.0
            )
            .is_ok()
        );
    }

    #[tokio::test]
    async fn test_encrypt_decrypt_oauth_user_credentials() {
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
        let decryption_service = setup
            .crypto_cache
            .get_decryption_service(&setup.dek_alias)
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

        let expiry = WrappedChronoDateTime::now();

        // Test encrypting user credentials
        let raw_config = WrappedJsonValue::new(json!({
            "code": "plain-code",
            "access_token": "plain-access-token",
            "refresh_token": "plain-refresh-token",
            "expiry_time": expiry,
            "sub": "test-user",
            "scopes": ["scope1", "scope2"],
            "metadata": {}
        }));

        let encrypted = controller
            .encrypt_user_credential_configuration(&encryption_service, raw_config)
            .await
            .unwrap();

        let value = encrypted.value();
        let config: Oauth2AuthorizationCodeFlowUserCredential =
            serde_json::from_value(value.clone().into()).unwrap();

        // Verify all sensitive fields are encrypted
        assert_ne!(config.code.0, "plain-code");
        assert_ne!(config.access_token.0, "plain-access-token");
        assert_ne!(config.refresh_token.0, "plain-refresh-token");

        // Verify non-sensitive fields are preserved
        assert_eq!(config.sub, "test-user");
        assert_eq!(config.scopes, vec!["scope1", "scope2"]);

        // Test decryption
        let user_cred_serialized = UserCredentialSerialized {
            id: WrappedUuidV4::new(),
            type_id: "oauth2_authorization_code_flow_user".to_string(),
            metadata: Metadata::new(),
            value: value.clone(),
            created_at: WrappedChronoDateTime::now(),
            updated_at: WrappedChronoDateTime::now(),
            next_rotation_time: None,
            dek_alias: setup.dek_alias.clone(),
        };

        let decrypted = controller
            .decrypt_oauth_credentials(&decryption_service, &user_cred_serialized)
            .await
            .unwrap();

        assert_eq!(decrypted.code, "plain-code");
        assert_eq!(decrypted.access_token, "plain-access-token");
        assert_eq!(decrypted.refresh_token, "plain-refresh-token");
        assert_eq!(decrypted.sub, "test-user");
        assert_eq!(decrypted.scopes, vec!["scope1", "scope2"]);
    }

    #[tokio::test]
    async fn test_oauth_jwt_bearer_encrypt_decrypt_credentials() {
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

        let controller = Oauth2JwtBearerAssertionFlowController {
            static_credentials: Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration {
                auth_uri: "https://example.com/auth".to_string(),
                token_uri: "https://example.com/token".to_string(),
                userinfo_uri: "https://example.com/userinfo".to_string(),
                jwks_uri: "https://example.com/jwks".to_string(),
                issuer: "https://example.com".to_string(),
                scopes: vec!["scope1".to_string()],
                metadata: Metadata::new(),
            },
        };

        // Test encrypting resource server credentials
        let raw_config = WrappedJsonValue::new(json!({
            "client_id": "test-client-id",
            "private_key": "-----BEGIN PRIVATE KEY-----\ntest\n-----END PRIVATE KEY-----",
            "token_uri": "https://example.com/token",
            "metadata": {}
        }));

        let encrypted = controller
            .encrypt_resource_server_configuration(&encryption_service, raw_config)
            .await
            .unwrap();

        let value = encrypted.value();
        let config: Oauth2JwtBearerAssertionFlowResourceServerCredential =
            serde_json::from_value(value.into()).unwrap();

        // Verify private_key is encrypted
        assert_ne!(
            config.private_key.0,
            "-----BEGIN PRIVATE KEY-----\ntest\n-----END PRIVATE KEY-----"
        );
    }
}

#[cfg(all(test, feature = "integration_test"))]
mod integration_test {
    use super::*;
    use identity::test::dex::{
        DEX_AUTH_ENDPOINT, DEX_CLIENT_ID, DEX_CLIENT_SECRET, DEX_ISSUER, DEX_JWKS_ENDPOINT,
        DEX_OAUTH_SCOPES, DEX_TOKEN_ENDPOINT, DEX_USERINFO_ENDPOINT,
    };

    /// Create OAuth static credential configuration for Dex.
    fn create_dex_static_credentials() -> Oauth2AuthorizationCodeFlowStaticCredentialConfiguration {
        Oauth2AuthorizationCodeFlowStaticCredentialConfiguration {
            auth_uri: DEX_AUTH_ENDPOINT.to_string(),
            token_uri: DEX_TOKEN_ENDPOINT.to_string(),
            userinfo_uri: DEX_USERINFO_ENDPOINT.to_string(),
            jwks_uri: DEX_JWKS_ENDPOINT.to_string(),
            issuer: DEX_ISSUER.to_string(),
            scopes: DEX_OAUTH_SCOPES.iter().map(|s| s.to_string()).collect(),
            metadata: Metadata::new(),
        }
    }

    #[tokio::test]
    async fn test_dex_oauth_static_credentials_structure() {
        let static_creds = create_dex_static_credentials();

        assert_eq!(static_creds.auth_uri, DEX_AUTH_ENDPOINT);
        assert_eq!(static_creds.token_uri, DEX_TOKEN_ENDPOINT);
        assert_eq!(static_creds.jwks_uri, DEX_JWKS_ENDPOINT);
        assert_eq!(static_creds.issuer, DEX_ISSUER);
        assert!(!static_creds.scopes.is_empty());
    }

    #[tokio::test]
    async fn test_oauth_controller_type_id() {
        let controller = OauthAuthFlowController {
            static_credentials: create_dex_static_credentials(),
        };

        assert_eq!(controller.type_id(), "oauth_auth_flow");
        assert_eq!(OauthAuthFlowController::static_type_id(), "oauth_auth_flow");
    }

    #[tokio::test]
    async fn test_oauth_controller_documentation() {
        let controller = OauthAuthFlowController {
            static_credentials: create_dex_static_credentials(),
        };

        let doc = controller.documentation();
        assert!(doc.contains("OAuth"));
        assert!(doc.contains("Authorization Code Flow"));
    }

    #[tokio::test]
    async fn test_oauth_controller_provides_broker() {
        let controller = OauthAuthFlowController {
            static_credentials: create_dex_static_credentials(),
        };

        // OAuth controller should provide user credential broker
        assert!(
            controller.as_user_credential_broker().is_some(),
            "OAuth controller should provide user credential broker"
        );

        // OAuth controller should provide rotateable credential support
        assert!(
            controller
                .as_rotateable_controller_user_credential()
                .is_some(),
            "OAuth controller should provide rotateable credential support"
        );
    }

    #[tokio::test]
    async fn test_dex_token_endpoint_reachable() {
        // Token endpoint should return an error for invalid requests (not 404)
        let client = reqwest::Client::new();
        let response = client
            .post(DEX_TOKEN_ENDPOINT)
            .form(&[("grant_type", "authorization_code")])
            .send()
            .await
            .expect("Failed to reach Dex token endpoint");

        // Should return 400 (bad request) not 404
        assert_ne!(
            response.status().as_u16(),
            404,
            "Token endpoint should exist"
        );
    }

    #[tokio::test]
    async fn test_token_refresh_invalid_token() {
        // Attempt to refresh with an invalid token
        let client = reqwest::Client::new();
        let response = client
            .post(DEX_TOKEN_ENDPOINT)
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", "invalid_refresh_token"),
                ("client_id", DEX_CLIENT_ID),
                ("client_secret", DEX_CLIENT_SECRET),
            ])
            .send()
            .await
            .expect("Failed to reach Dex token endpoint");

        // Should return 400 (invalid grant) for invalid refresh token
        assert!(
            response.status().is_client_error(),
            "Should reject invalid refresh token"
        );
    }

    #[tokio::test]
    async fn test_oauth_user_credential_rotation_time_calculation() {
        let now = WrappedChronoDateTime::now();
        let expiry_time = WrappedChronoDateTime::new(
            now.get_inner()
                .checked_add_signed(chrono::Duration::hours(2))
                .unwrap(),
        );

        let credential = Oauth2AuthorizationCodeFlowUserCredential {
            code: EncryptedString("encrypted_code".to_string()),
            access_token: EncryptedString("encrypted_token".to_string()),
            refresh_token: EncryptedString("encrypted_refresh".to_string()),
            expiry_time,
            sub: "test-user".to_string(),
            scopes: vec!["email".to_string(), "offline_access".to_string()],
            metadata: Metadata::new(),
        };

        let next_rotation = credential.next_rotation_time();

        // Should be 5 minutes before expiry
        let expected = WrappedChronoDateTime::new(
            expiry_time
                .get_inner()
                .checked_sub_signed(chrono::Duration::minutes(5))
                .unwrap(),
        );

        assert_eq!(next_rotation.get_inner(), expected.get_inner());

        // Verify rotation time is before expiry
        assert!(
            next_rotation.get_inner() < expiry_time.get_inner(),
            "Rotation time should be before expiry"
        );
    }

    #[tokio::test]
    async fn test_oauth_credential_types() {
        let user_cred = Oauth2AuthorizationCodeFlowUserCredential {
            code: EncryptedString("code".to_string()),
            access_token: EncryptedString("access".to_string()),
            refresh_token: EncryptedString("refresh".to_string()),
            expiry_time: WrappedChronoDateTime::now(),
            sub: "sub".to_string(),
            scopes: vec!["scope".to_string()],
            metadata: Metadata::new(),
        };

        assert_eq!(user_cred.type_id(), "oauth2_authorization_code_flow_user");

        // Should be rotateable
        assert!(user_cred.as_rotateable_credential().is_some());
    }

    #[tokio::test]
    async fn test_resource_server_credential_type() {
        let resource_cred = Oauth2AuthorizationCodeFlowResourceServerCredential {
            client_id: DEX_CLIENT_ID.to_string(),
            client_secret: EncryptedString(DEX_CLIENT_SECRET.to_string()),
            redirect_uri: "http://localhost:8080/callback".to_string(),
            metadata: Metadata::new(),
        };

        assert_eq!(
            resource_cred.type_id(),
            "oauth2_authorization_code_flow_resource_server"
        );
    }
}
