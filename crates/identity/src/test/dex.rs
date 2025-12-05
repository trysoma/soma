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
