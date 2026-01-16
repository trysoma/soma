//! Dex (OIDC/OAuth2 provider) test configuration constants.
//!
//! These constants are duplicated from identity::test::dex for use in mcp crate tests,
//! since cross-crate test module dependencies don't work with #[cfg(test)].

/// Authorization endpoint for OIDC flows
pub const DEX_AUTH_ENDPOINT: &str = "http://localhost:5556/dex/auth";

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

/// OAuth2 scopes (no openid - OAuth2-only flow)
pub const DEX_OAUTH_SCOPES: &[&str] = &["email", "offline_access"];
