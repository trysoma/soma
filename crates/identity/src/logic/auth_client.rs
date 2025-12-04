use std::{collections::HashMap, sync::Arc};

use arc_swap::ArcSwap;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::{Deserialize, Serialize};
use shared::error::CommonError;
use utoipa::ToSchema;

use crate::logic::api_key::{EncryptedApiKeyConfig, hash_api_key};
use crate::logic::api_key::cache::ApiKeyCache;
use crate::logic::jwk::cache::JwksCache;
use crate::logic::sts::config::StsTokenConfig;
use crate::logic::user::Role;
use crate::logic::internal_token_issuance::{ISSUER, AUDIENCE, AccessTokenClaims, AccessTokenType};

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
#[derive(Debug, Clone)]
pub struct Machine {
    pub id: String,
    pub role: Role,
}

/// Authenticated human identity
#[derive(Debug, Clone)]
pub struct Human {
    pub sub: String,
    pub email: Option<String>,
    pub groups: Vec<String>,
    pub role: Role,
}

/// Authenticated identity
#[derive(Debug, Clone)]
pub enum Identity {
    /// Machine identity (API key authentication)
    Machine(Machine),
    /// Human identity (STS token authentication)
    Human(Human),
    /// Machine acting on behalf of a human
    MachineOnBehalfOfHuman(Machine, Human),
    /// Unauthenticated request
    Unauthenticated,
}

impl Identity {
    /// Get the role of the identity
    pub fn role(&self) -> Option<&Role> {
        match self {
            Identity::Machine(m) => Some(&m.role),
            Identity::Human(h) => Some(&h.role),
            Identity::MachineOnBehalfOfHuman(_, h) => Some(&h.role),
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
            Identity::Machine(m) => Some(&m.id),
            Identity::Human(h) => Some(&h.sub),
            Identity::MachineOnBehalfOfHuman(_, h) => Some(&h.sub),
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
    pub fn new(
        jwks_cache: JwksCache,
        api_key_cache: ApiKeyCache,
    ) -> Self {
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
            id: cached_api_key.user.id,
            role: cached_api_key.user.role,
        }))
    }

    /// Authenticate an Internal token (JWT issued by us)
    async fn authenticate_internal_token(&self, internal_token: &InternalToken) -> Result<Identity, CommonError> {
        // 1. Decode the token header to get the kid
        let header =
            decode_header(&internal_token.0).map_err(|e| CommonError::Authentication {
                msg: format!("Failed to decode token header: {e}"),
                source: None,
            })?;

        let kid = header.kid.ok_or_else(|| CommonError::Authentication {
            msg: "Token missing 'kid' in header".to_string(),
            source: None,
        })?;

        // 2. Get the JWK from our cache
        let jwk = self.jwks_cache.get_jwk(&kid).ok_or_else(|| {
            CommonError::Authentication {
                msg: format!("Signing key '{}' not found", kid),
                source: None,
            }
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
                })
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
                })
            }
        };

        Ok(Identity::MachineOnBehalfOfHuman(machine, human))
    }
}
