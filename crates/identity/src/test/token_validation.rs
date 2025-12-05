//! Utilities for validating and decoding internal Soma tokens in tests.
//!
//! This module provides helpers for decoding and validating access tokens
//! issued by Soma's internal token issuance system, verifying claims,
//! signatures, and structure.

use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use shared::error::CommonError;

use crate::logic::internal_token_issuance::{
    AUDIENCE, AccessTokenClaims, ISSUER, RefreshTokenClaims,
};
use crate::logic::jwk::cache::JwksCache;
use crate::logic::user::Role;

/// Result of validating an internal access token.
#[derive(Debug)]
pub struct ValidatedAccessToken {
    /// The decoded claims from the access token
    pub claims: AccessTokenClaims,
    /// The key ID used to sign this token
    pub kid: String,
}

/// Result of validating an internal refresh token.
#[derive(Debug)]
pub struct ValidatedRefreshToken {
    /// The decoded claims from the refresh token
    pub claims: RefreshTokenClaims,
    /// The key ID used to sign this token
    pub kid: String,
}

/// Decode and validate an internal access token issued by Soma.
///
/// This function:
/// 1. Extracts the key ID (kid) from the token header
/// 2. Looks up the signing key in the JWKS cache
/// 3. Validates the signature using RS256
/// 4. Verifies issuer and audience claims
/// 5. Returns the decoded claims
pub fn decode_and_validate_access_token(
    token: &str,
    jwks_cache: &JwksCache,
) -> Result<ValidatedAccessToken, CommonError> {
    // 1. Decode header to get kid
    let header = decode_header(token).map_err(|e| CommonError::Authentication {
        msg: format!("Failed to decode token header: {e}"),
        source: None,
    })?;

    let kid = header.kid.ok_or_else(|| CommonError::Authentication {
        msg: "Token missing 'kid' in header".to_string(),
        source: None,
    })?;

    // 2. Get the signing key from JWKS cache
    let jwks = jwks_cache.get_cached_jwks();
    let jwk = jwks
        .iter()
        .find(|k| k.kid == kid)
        .ok_or_else(|| CommonError::Authentication {
            msg: format!("Signing key '{kid}' not found in JWKS"),
            source: None,
        })?;

    // 3. Create decoding key from RSA components
    let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to create decoding key: {e}")))?;

    // 4. Set up validation
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[ISSUER]);
    validation.set_audience(&[AUDIENCE]);

    // 5. Decode and validate
    let token_data =
        decode::<AccessTokenClaims>(token, &decoding_key, &validation).map_err(|e| {
            CommonError::Authentication {
                msg: format!("Token validation failed: {e}"),
                source: None,
            }
        })?;

    Ok(ValidatedAccessToken {
        claims: token_data.claims,
        kid,
    })
}

/// Decode and validate an internal refresh token issued by Soma.
///
/// This function:
/// 1. Extracts the key ID (kid) from the token header
/// 2. Looks up the signing key in the JWKS cache
/// 3. Validates the signature using RS256
/// 4. Verifies issuer and audience claims
/// 5. Returns the decoded claims
pub fn decode_and_validate_refresh_token(
    token: &str,
    jwks_cache: &JwksCache,
) -> Result<ValidatedRefreshToken, CommonError> {
    // 1. Decode header to get kid
    let header = decode_header(token).map_err(|e| CommonError::Authentication {
        msg: format!("Failed to decode token header: {e}"),
        source: None,
    })?;

    let kid = header.kid.ok_or_else(|| CommonError::Authentication {
        msg: "Token missing 'kid' in header".to_string(),
        source: None,
    })?;

    // 2. Get the signing key from JWKS cache
    let jwks = jwks_cache.get_cached_jwks();
    let jwk = jwks
        .iter()
        .find(|k| k.kid == kid)
        .ok_or_else(|| CommonError::Authentication {
            msg: format!("Signing key '{kid}' not found in JWKS"),
            source: None,
        })?;

    // 3. Create decoding key from RSA components
    let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to create decoding key: {e}")))?;

    // 4. Set up validation
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[ISSUER]);
    validation.set_audience(&[AUDIENCE]);

    // 5. Decode and validate
    let token_data =
        decode::<RefreshTokenClaims>(token, &decoding_key, &validation).map_err(|e| {
            CommonError::Authentication {
                msg: format!("Refresh token validation failed: {e}"),
                source: None,
            }
        })?;

    Ok(ValidatedRefreshToken {
        claims: token_data.claims,
        kid,
    })
}

/// Assertion helper for validating access token claims.
pub struct AccessTokenAssertions<'a> {
    token: &'a ValidatedAccessToken,
}

impl<'a> AccessTokenAssertions<'a> {
    pub fn new(token: &'a ValidatedAccessToken) -> Self {
        Self { token }
    }

    /// Assert the subject matches expected value.
    pub fn assert_subject(&self, expected: &str) -> &Self {
        assert_eq!(
            self.token.claims.sub, expected,
            "Subject mismatch: expected '{}', got '{}'",
            expected, self.token.claims.sub
        );
        self
    }

    /// Assert the subject starts with expected prefix.
    pub fn assert_subject_starts_with(&self, prefix: &str) -> &Self {
        assert!(
            self.token.claims.sub.starts_with(prefix),
            "Subject should start with '{}', got '{}'",
            prefix,
            self.token.claims.sub
        );
        self
    }

    /// Assert the email matches expected value.
    pub fn assert_email(&self, expected: Option<&str>) -> &Self {
        assert_eq!(
            self.token.claims.email.as_deref(),
            expected,
            "Email mismatch"
        );
        self
    }

    /// Assert the email is present and contains a value.
    pub fn assert_email_present(&self) -> &Self {
        assert!(
            self.token.claims.email.is_some(),
            "Email should be present in token"
        );
        self
    }

    /// Assert the role matches expected value.
    pub fn assert_role(&self, expected: Role) -> &Self {
        assert_eq!(
            self.token.claims.role, expected,
            "Role mismatch: expected {:?}, got {:?}",
            expected, self.token.claims.role
        );
        self
    }

    /// Assert groups contain expected values.
    pub fn assert_groups_contain(&self, expected: &[&str]) -> &Self {
        for group in expected {
            assert!(
                self.token.claims.groups.contains(&group.to_string()),
                "Groups should contain '{}', got {:?}",
                group,
                self.token.claims.groups
            );
        }
        self
    }

    /// Assert the issuer is correct (soma-identity).
    pub fn assert_issuer(&self) -> &Self {
        assert_eq!(
            self.token.claims.iss, ISSUER,
            "Issuer mismatch: expected '{}', got '{}'",
            ISSUER, self.token.claims.iss
        );
        self
    }

    /// Assert the audience is correct (soma).
    pub fn assert_audience(&self) -> &Self {
        assert_eq!(
            self.token.claims.aud, AUDIENCE,
            "Audience mismatch: expected '{}', got '{}'",
            AUDIENCE, self.token.claims.aud
        );
        self
    }

    /// Assert token is not expired (exp > now).
    pub fn assert_not_expired(&self) -> &Self {
        let now = chrono::Utc::now().timestamp();
        assert!(
            self.token.claims.exp > now,
            "Token should not be expired: exp={}, now={}",
            self.token.claims.exp,
            now
        );
        self
    }

    /// Assert token has a valid JTI (non-empty UUID format).
    pub fn assert_valid_jti(&self) -> &Self {
        assert!(!self.token.claims.jti.is_empty(), "JTI should not be empty");
        // Try to parse as UUID
        uuid::Uuid::parse_str(&self.token.claims.jti).expect("JTI should be a valid UUID");
        self
    }

    /// Run all standard validations (issuer, audience, not expired, valid jti).
    pub fn assert_standard_claims(&self) -> &Self {
        self.assert_issuer()
            .assert_audience()
            .assert_not_expired()
            .assert_valid_jti()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_access_token_assertions_chain() {
        // This test verifies the assertion builder pattern compiles correctly
        // Actual validation tests are in the integration tests
    }
}
