use std::collections::HashMap;

use jsonwebtoken::DecodingKey;
use shared::error::CommonError;

/// External JWKS cache for fetching public keys from external identity providers
#[derive(Clone)]
pub struct ExternalJwksCache {
    /// Maps JWKS URI -> (kid -> DecodingKey)
    keys: std::sync::Arc<dashmap::DashMap<String, HashMap<String, DecodingKey>>>,
}

impl Default for ExternalJwksCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ExternalJwksCache {
    pub fn new() -> Self {
        Self {
            keys: std::sync::Arc::new(dashmap::DashMap::new()),
        }
    }

    /// Fetch JWKS from the given URI and cache the keys
    pub async fn fetch_jwks(&self, jwks_uri: &str) -> Result<(), CommonError> {
        let response = reqwest::get(jwks_uri)
            .await
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to fetch JWKS: {e}")))?;

        let jwks: serde_json::Value = response
            .json()
            .await
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to parse JWKS: {e}")))?;

        let keys = jwks["keys"]
            .as_array()
            .ok_or_else(|| CommonError::Unknown(anyhow::anyhow!("JWKS missing 'keys' array")))?;

        let mut key_map = HashMap::new();
        for key in keys {
            let kid = key["kid"]
                .as_str()
                .ok_or_else(|| CommonError::Unknown(anyhow::anyhow!("JWK missing 'kid'")))?;

            let kty = key["kty"].as_str().unwrap_or("RSA");
            let decoding_key = match kty {
                "RSA" => {
                    let n = key["n"].as_str().ok_or_else(|| {
                        CommonError::Unknown(anyhow::anyhow!("RSA JWK missing 'n'"))
                    })?;
                    let e = key["e"].as_str().ok_or_else(|| {
                        CommonError::Unknown(anyhow::anyhow!("RSA JWK missing 'e'"))
                    })?;
                    DecodingKey::from_rsa_components(n, e).map_err(|e| {
                        CommonError::Unknown(anyhow::anyhow!("Failed to create RSA key: {e}"))
                    })?
                }
                _ => {
                    tracing::warn!("Unsupported key type: {}", kty);
                    continue;
                }
            };

            key_map.insert(kid.to_string(), decoding_key);
        }

        self.keys.insert(jwks_uri.to_string(), key_map);
        Ok(())
    }

    /// Get a decoding key by JWKS URI and key ID
    pub fn get_key(&self, jwks_uri: &str, kid: &str) -> Option<DecodingKey> {
        self.keys
            .get(jwks_uri)
            .and_then(|keys| keys.get(kid).cloned())
    }
}
