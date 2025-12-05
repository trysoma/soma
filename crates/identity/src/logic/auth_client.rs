use http::HeaderMap;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::{Deserialize, Serialize};
use shared::error::CommonError;
use utoipa::ToSchema;

use crate::logic::api_key::cache::ApiKeyCache;
use crate::logic::api_key::hash_api_key;
use crate::logic::internal_token_issuance::{AUDIENCE, AccessTokenClaims, AccessTokenType, ISSUER};
use crate::logic::jwk::cache::JwksCache;
use crate::logic::user::Role;
use crate::router::ACCESS_TOKEN_COOKIE_NAME;

/// Header name for API key authentication
pub const API_KEY_HEADER: &str = "x-api-key";

/// Raw API key credential
pub struct ApiKey(pub String);

/// Raw Internal token credential
pub struct InternalToken(pub String);

/// Raw credentials that can be extracted from a request
pub enum RawCredentials {
    /// Machine authentication via API key
    MachineApiKey(ApiKey),
    /// Human authentication via STS token (JWT)
    HumanInternalToken(InternalToken),
    /// Machine acting on behalf of a human
    MachineOnBehalfOfHuman(ApiKey, InternalToken),
}

/// Authenticated machine identity
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Machine {
    pub sub: String,
    pub role: Role,
}

/// Authenticated human identity
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Human {
    pub sub: String,
    pub email: Option<String>,
    pub groups: Vec<String>,
    pub role: Role,
}

/// Authenticated identity
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Identity {
    /// Machine identity (API key authentication)
    Machine(Machine),
    /// Human identity (STS token authentication)
    Human(Human),
    /// Machine acting on behalf of a human
    MachineOnBehalfOfHuman { machine: Machine, human: Human },
    /// Unauthenticated request
    Unauthenticated,
}

impl Identity {
    /// Get the role of the identity
    pub fn role(&self) -> Option<&Role> {
        match self {
            Identity::Machine(m) => Some(&m.role),
            Identity::Human(h) => Some(&h.role),
            Identity::MachineOnBehalfOfHuman { machine, human: _ } => Some(&machine.role),
            Identity::Unauthenticated => None,
        }
    }

    /// Check if the identity is authenticated
    pub fn is_authenticated(&self) -> bool {
        !matches!(self, Identity::Unauthenticated)
    }

    /// Get the subject ID of the identity
    pub fn subject(&self) -> Option<&str> {
        match self {
            Identity::Machine(m) => Some(&m.sub),
            Identity::Human(h) => Some(&h.sub),
            Identity::MachineOnBehalfOfHuman { machine, human: _ } => Some(&machine.sub),
            Identity::Unauthenticated => None,
        }
    }
}

/// Authentication client for validating credentials
///
/// This struct is designed to be cloned and shared across async boundaries.
/// All internal caches use Arc and are safe to share.
/// The auth config uses ArcSwap for atomic updates that propagate to all instances.
#[derive(Clone)]
pub struct AuthClient {
    /// Cache of our JWKS for validating tokens we issued
    jwks_cache: JwksCache,
    /// Cache of API keys for authentication
    api_key_cache: ApiKeyCache,
}

impl AuthClient {
    /// Create a new AuthClient with the given caches and config
    pub fn new(jwks_cache: JwksCache, api_key_cache: ApiKeyCache) -> Self {
        Self {
            jwks_cache,
            api_key_cache,
        }
    }

    /// Get a reference to the JWKS cache
    pub fn jwks_cache(&self) -> &JwksCache {
        &self.jwks_cache
    }

    /// Get a reference to the API key cache
    pub fn api_key_cache(&self) -> &ApiKeyCache {
        &self.api_key_cache
    }

    /// Authenticate credentials and return an Identity
    pub async fn authenticate(&self, credentials: RawCredentials) -> Result<Identity, CommonError> {
        match credentials {
            RawCredentials::MachineApiKey(api_key) => self.authenticate_api_key(&api_key).await,
            RawCredentials::HumanInternalToken(internal_token) => {
                self.authenticate_internal_token(&internal_token).await
            }
            RawCredentials::MachineOnBehalfOfHuman(api_key, internal_token) => {
                self.authenticate_machine_on_behalf_of_human(&api_key, &internal_token)
                    .await
            }
        }
    }

    /// Authenticate an API key
    ///
    /// This method:
    /// 1. Hashes the incoming API key using SHA-256
    /// 2. Looks up the hash in the API key cache (with repository fallback)
    /// 3. Returns the authenticated identity if found
    async fn authenticate_api_key(&self, api_key: &ApiKey) -> Result<Identity, CommonError> {
        // Hash the incoming API key
        let hashed_value = hash_api_key(&api_key.0);

        // Look up in the cache (falls back to repository if not cached)
        let cached_api_key = self
            .api_key_cache
            .get_by_hashed_value(&hashed_value)
            .await?
            .ok_or_else(|| CommonError::Authentication {
                msg: "Invalid API key".to_string(),
                source: None,
            })?;

        Ok(Identity::Machine(Machine {
            sub: cached_api_key.user.id,
            role: cached_api_key.user.role,
        }))
    }

    /// Authenticate an Internal token (JWT issued by us)
    async fn authenticate_internal_token(
        &self,
        internal_token: &InternalToken,
    ) -> Result<Identity, CommonError> {
        // 1. Decode the token header to get the kid
        let header = decode_header(&internal_token.0).map_err(|e| CommonError::Authentication {
            msg: format!("Failed to decode token header: {e}"),
            source: None,
        })?;

        let kid = header.kid.ok_or_else(|| CommonError::Authentication {
            msg: "Token missing 'kid' in header".to_string(),
            source: None,
        })?;

        // 2. Get the JWK from our cache
        let jwk = self
            .jwks_cache
            .get_jwk(&kid)
            .ok_or_else(|| CommonError::Authentication {
                msg: format!("Signing key '{kid}' not found"),
                source: None,
            })?;

        // 3. Create decoding key from the JWK
        let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e).map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!("Failed to create decoding key: {e}"))
        })?;

        // 4. Validate and decode the token
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[ISSUER]);
        validation.set_audience(&[AUDIENCE]);

        let token_data = decode::<AccessTokenClaims>(&internal_token.0, &decoding_key, &validation)
            .map_err(|e| CommonError::Authentication {
                msg: format!("Token validation failed: {e}"),
                source: None,
            })?;

        let claims = token_data.claims;

        // 5. Verify it's an access token
        if !matches!(claims.token_type, AccessTokenType::Access) {
            return Err(CommonError::Authentication {
                msg: "Invalid token type: expected access token".to_string(),
                source: None,
            });
        }

        // 6. Use the role directly (it's already a Role enum)
        let role = claims.role;

        Ok(Identity::Human(Human {
            sub: claims.sub,
            email: claims.email,
            groups: claims.groups,
            role,
        }))
    }

    /// Authenticate a machine acting on behalf of a human
    async fn authenticate_machine_on_behalf_of_human(
        &self,
        api_key: &ApiKey,
        internal_token: &InternalToken,
    ) -> Result<Identity, CommonError> {
        // First authenticate the API key
        let machine_identity = self.authenticate_api_key(api_key).await?;
        let machine = match machine_identity {
            Identity::Machine(m) => m,
            _ => {
                return Err(CommonError::Authentication {
                    msg: "Expected machine identity from API key".to_string(),
                    source: None,
                });
            }
        };

        // Then authenticate the STS token
        let human_identity = self.authenticate_internal_token(internal_token).await?;
        let human = match human_identity {
            Identity::Human(h) => h,
            _ => {
                return Err(CommonError::Authentication {
                    msg: "Expected human identity from STS token".to_string(),
                    source: None,
                });
            }
        };

        Ok(Identity::MachineOnBehalfOfHuman { machine, human })
    }

    /// Authenticate from HTTP headers
    ///
    /// This method extracts credentials from HTTP headers and authenticates them.
    /// Priority order for internal token:
    /// 1. Authorization header (with or without "Bearer " prefix)
    /// 2. soma_access_token cookie
    ///
    /// API key is checked from the x-api-key header.
    ///
    /// Returns:
    /// - `Identity::Unauthenticated` if no credentials are found
    /// - The appropriate `Identity` variant if credentials are found and valid
    /// - An error if credentials are found but invalid
    pub async fn authenticate_from_headers(
        &self,
        headers: &HeaderMap,
    ) -> Result<Identity, CommonError> {
        // Extract internal token from Authorization header or cookie
        let internal_token = self.extract_internal_token(headers);

        // Extract API key from x-api-key header
        let api_key = self.extract_api_key(headers);

        // Build RawCredentials based on what we found
        let credentials = match (internal_token, api_key) {
            (Some(token), Some(key)) => {
                // Both present - machine on behalf of human
                RawCredentials::MachineOnBehalfOfHuman(ApiKey(key), InternalToken(token))
            }
            (Some(token), None) => {
                // Only internal token - human authentication
                RawCredentials::HumanInternalToken(InternalToken(token))
            }
            (None, Some(key)) => {
                // Only API key - machine authentication
                RawCredentials::MachineApiKey(ApiKey(key))
            }
            (None, None) => {
                // No credentials found
                return Ok(Identity::Unauthenticated);
            }
        };

        self.authenticate(credentials).await
    }

    /// Extract internal token from headers
    ///
    /// Checks Authorization header first (with or without "Bearer " prefix),
    /// then falls back to the soma_access_token cookie.
    fn extract_internal_token(&self, headers: &HeaderMap) -> Option<String> {
        // Check Authorization header first (takes priority)
        if let Some(auth_header) = headers.get(http::header::AUTHORIZATION) {
            if let Ok(auth_str) = auth_header.to_str() {
                let auth_str = auth_str.trim();
                // Handle both "Bearer <token>" and just "<token>"
                if let Some(token) = auth_str.strip_prefix("Bearer ") {
                    return Some(token.trim().to_string());
                } else if let Some(token) = auth_str.strip_prefix("bearer ") {
                    return Some(token.trim().to_string());
                } else if !auth_str.is_empty() {
                    // No Bearer prefix, use the whole value
                    return Some(auth_str.to_string());
                }
            }
        }

        // Fall back to cookie
        if let Some(cookie_header) = headers.get(http::header::COOKIE) {
            if let Ok(cookie_str) = cookie_header.to_str() {
                // Parse cookies manually (format: "name=value; name2=value2")
                for cookie in cookie_str.split(';') {
                    let cookie = cookie.trim();
                    if let Some((name, value)) = cookie.split_once('=') {
                        if name.trim() == ACCESS_TOKEN_COOKIE_NAME {
                            return Some(value.trim().to_string());
                        }
                    }
                }
            }
        }

        None
    }

    /// Extract API key from x-api-key header
    fn extract_api_key(&self, headers: &HeaderMap) -> Option<String> {
        headers
            .get(API_KEY_HEADER)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }
}
