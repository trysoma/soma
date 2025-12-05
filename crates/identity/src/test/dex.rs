//! Dex (OIDC/OAuth2 provider) test configuration and utilities.
//!
//! This module provides configuration constants for testing against a local Dex instance.
//! Dex should be running via docker-compose before running integration tests.
//!
//! # Configuration
//! The Dex instance should be configured with:
//! - Static client "trysoma.ai" with secret "example-secret"
//! - Mock connector enabled for easy authentication testing
//!
//! # Running Dex
//! Ensure Dex is running locally before running integration tests.

/// Dex server base URL (running locally via docker)
pub const DEX_BASE_URL: &str = "http://localhost:5556";

/// Authorization endpoint for OIDC flows
pub const DEX_AUTH_ENDPOINT: &str = "http://localhost:5556/dex/auth";

/// Mock connector authorization endpoint (bypasses actual login)
pub const DEX_AUTH_MOCK_ENDPOINT: &str = "http://localhost:5556/dex/auth/mock";

/// Token endpoint for exchanging authorization codes
pub const DEX_TOKEN_ENDPOINT: &str = "http://localhost:5556/dex/token";

/// Userinfo endpoint for fetching user claims (OIDC only)
pub const DEX_USERINFO_ENDPOINT: &str = "http://localhost:5556/dex/userinfo";

/// JWKS endpoint for validating token signatures
pub const DEX_JWKS_ENDPOINT: &str = "http://localhost:5556/dex/keys";

/// Issuer identifier
pub const DEX_ISSUER: &str = "http://localhost:5556/dex";

/// Test client ID configured in Dex
pub const DEX_CLIENT_ID: &str = "trysoma.ai";

/// Test client secret configured in Dex
pub const DEX_CLIENT_SECRET: &str = "example-secret";

/// OIDC scopes (includes openid for ID token)
pub const DEX_OIDC_SCOPES: &[&str] = &["openid", "email", "offline_access"];

/// OAuth2 scopes (no openid - OAuth2-only flow)
pub const DEX_OAUTH_SCOPES: &[&str] = &["email", "offline_access"];

/// Default redirect URI for tests
pub const DEX_REDIRECT_URI: &str = "http://localhost:8080/callback";

/// Mock user email returned by Dex mock connector
pub const DEX_MOCK_USER_EMAIL: &str = "kilgore@kilgore.trout";

/// Mock user ID (subject) returned by Dex mock connector
pub const DEX_MOCK_USER_ID: &str = "Cg0wLTM4NS0yODA4OS0wEgRtb2Nr";

/// Configuration for OIDC authorization code flow tests
pub struct OidcTestConfig {
    pub auth_endpoint: &'static str,
    pub token_endpoint: &'static str,
    pub userinfo_endpoint: &'static str,
    pub jwks_endpoint: &'static str,
    pub issuer: &'static str,
    pub client_id: &'static str,
    pub client_secret: &'static str,
    pub scopes: Vec<String>,
    pub redirect_uri: &'static str,
}

impl Default for OidcTestConfig {
    fn default() -> Self {
        Self {
            auth_endpoint: DEX_AUTH_ENDPOINT,
            token_endpoint: DEX_TOKEN_ENDPOINT,
            userinfo_endpoint: DEX_USERINFO_ENDPOINT,
            jwks_endpoint: DEX_JWKS_ENDPOINT,
            issuer: DEX_ISSUER,
            client_id: DEX_CLIENT_ID,
            client_secret: DEX_CLIENT_SECRET,
            scopes: DEX_OIDC_SCOPES.iter().map(|s| s.to_string()).collect(),
            redirect_uri: DEX_REDIRECT_URI,
        }
    }
}

/// Configuration for OAuth2 authorization code flow tests (no OIDC/ID token)
pub struct OauthTestConfig {
    pub auth_endpoint: &'static str,
    pub token_endpoint: &'static str,
    pub jwks_endpoint: &'static str,
    pub client_id: &'static str,
    pub client_secret: &'static str,
    pub scopes: Vec<String>,
    pub redirect_uri: &'static str,
}

impl Default for OauthTestConfig {
    fn default() -> Self {
        Self {
            auth_endpoint: DEX_AUTH_ENDPOINT,
            token_endpoint: DEX_TOKEN_ENDPOINT,
            jwks_endpoint: DEX_JWKS_ENDPOINT,
            client_id: DEX_CLIENT_ID,
            client_secret: DEX_CLIENT_SECRET,
            scopes: DEX_OAUTH_SCOPES.iter().map(|s| s.to_string()).collect(),
            redirect_uri: DEX_REDIRECT_URI,
        }
    }
}

/// Result from performing a mock OIDC login against Dex.
#[derive(Debug, Clone)]
pub struct DexOidcTokens {
    /// The access token (JWT)
    pub access_token: String,
    /// The ID token (JWT with OIDC claims)
    pub id_token: String,
    /// Optional refresh token (if offline_access scope was requested)
    pub refresh_token: Option<String>,
    /// Token expiry in seconds
    pub expires_in: i64,
}

/// Perform a mock OIDC login flow against Dex and return tokens.
///
/// This function performs the full OAuth2/OIDC authorization code flow:
/// 1. Initiates authorization request to Dex mock connector
/// 2. Follows redirects to get the authorization code
/// 3. Exchanges the code for tokens at the token endpoint
///
/// # Requirements
/// - Dex must be running locally with the mockCallback connector enabled
/// - The client "trysoma.ai" must be configured with the redirect URI
pub async fn perform_dex_mock_oidc_login() -> Result<DexOidcTokens, anyhow::Error> {
    perform_dex_mock_login_with_scopes(DEX_OIDC_SCOPES).await
}

/// Perform a mock OAuth2 login flow against Dex (without OIDC/ID token).
pub async fn perform_dex_mock_oauth_login() -> Result<DexOidcTokens, anyhow::Error> {
    perform_dex_mock_login_with_scopes(DEX_OAUTH_SCOPES).await
}

/// Perform mock login with custom scopes.
async fn perform_dex_mock_login_with_scopes(
    scopes: &[&str],
) -> Result<DexOidcTokens, anyhow::Error> {
    use reqwest::redirect::Policy;
    use url::Url;

    // Build the authorization URL
    let scope_str = scopes.join(" ");
    let state = uuid::Uuid::new_v4().to_string();
    let nonce = uuid::Uuid::new_v4().to_string();

    let auth_url = format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}&nonce={}",
        DEX_AUTH_MOCK_ENDPOINT,
        urlencoding::encode(DEX_CLIENT_ID),
        urlencoding::encode(DEX_REDIRECT_URI),
        urlencoding::encode(&scope_str),
        urlencoding::encode(&state),
        urlencoding::encode(&nonce),
    );

    // Create a client that doesn't follow redirects so we can capture the callback
    let client = reqwest::Client::builder()
        .redirect(Policy::none())
        .build()?;

    // Step 1: Initiate authorization - Dex mock connector redirects to internal /dex/callback first
    let response = client.get(&auth_url).send().await?;

    let internal_callback_url = if response.status().is_redirection() {
        response
            .headers()
            .get("location")
            .and_then(|l| l.to_str().ok())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("No location header in first redirect"))?
    } else {
        let status = response.status();
        let body = response.text().await?;
        return Err(anyhow::anyhow!(
            "Expected redirect from Dex mock, got HTTP {status} - {body}"
        ));
    };

    // Step 2: Follow the internal callback to get the final redirect with the code
    let internal_url = if internal_callback_url.starts_with("http") {
        internal_callback_url
    } else {
        format!("{DEX_BASE_URL}{internal_callback_url}")
    };

    let response = client.get(&internal_url).send().await?;

    let callback_url = if response.status().is_redirection() {
        response
            .headers()
            .get("location")
            .and_then(|l| l.to_str().ok())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("No location header in second redirect"))?
    } else {
        let status = response.status();
        let body = response.text().await?;
        return Err(anyhow::anyhow!(
            "Expected redirect from Dex internal callback, got HTTP {status} - {body}"
        ));
    };

    // Step 3: Extract the authorization code from the final callback URL
    let parsed_url = Url::parse(&callback_url).or_else(|_| {
        // If the URL is relative, prepend the base URL
        Url::parse(&format!("{DEX_BASE_URL}{callback_url}"))
    })?;

    let code = parsed_url
        .query_pairs()
        .find(|(k, _)| k == "code")
        .map(|(_, v)| v.to_string())
        .ok_or_else(|| anyhow::anyhow!("No code in callback URL: {callback_url}"))?;

    // Step 4: Exchange the code for tokens
    let token_response = client
        .post(DEX_TOKEN_ENDPOINT)
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("redirect_uri", DEX_REDIRECT_URI),
            ("client_id", DEX_CLIENT_ID),
            ("client_secret", DEX_CLIENT_SECRET),
        ])
        .send()
        .await?;

    if !token_response.status().is_success() {
        let status = token_response.status();
        let body = token_response.text().await?;
        return Err(anyhow::anyhow!(
            "Token exchange failed: HTTP {status} - {body}"
        ));
    }

    let token_json: serde_json::Value = token_response.json().await?;

    let access_token = token_json["access_token"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No access_token in response"))?
        .to_string();

    let id_token = token_json["id_token"].as_str().unwrap_or("").to_string();

    let refresh_token = token_json["refresh_token"].as_str().map(|s| s.to_string());

    let expires_in = token_json["expires_in"].as_i64().unwrap_or(3600);

    Ok(DexOidcTokens {
        access_token,
        id_token,
        refresh_token,
        expires_in,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oidc_config_default() {
        let config = OidcTestConfig::default();
        assert_eq!(config.client_id, DEX_CLIENT_ID);
        assert_eq!(config.client_secret, DEX_CLIENT_SECRET);
        assert!(config.scopes.contains(&"openid".to_string()));
    }

    #[test]
    fn test_oauth_config_default() {
        let config = OauthTestConfig::default();
        assert_eq!(config.client_id, DEX_CLIENT_ID);
        assert_eq!(config.client_secret, DEX_CLIENT_SECRET);
        // OAuth config should NOT have openid scope
        assert!(!config.scopes.contains(&"openid".to_string()));
    }
}
