use std::sync::Arc;

use anyhow::anyhow;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::Deserialize;
use shared::error::CommonError;
use tracing::info;

pub struct JwtTokenTemplateValidationConfig {
    pub issuer: Option<String>,
    pub valid_audiences: Option<Vec<String>>,
    pub required_scopes: Option<Vec<String>>,
    pub required_groups: Option<Vec<String>>,
}

pub struct JwtTokenMappingConfig {
    pub issuer_field: String,
    pub audience_field: String,
    pub scopes_field: String,
    pub sub_field: String,
    pub email_field: String,
    pub groups_field: String,
}

pub struct JwtTokenTemplateConfig {
    pub validation_template: JwtTokenTemplateValidationConfig,
    pub mapping_template: JwtTokenMappingConfig,
}


#[derive(Debug, Deserialize)]
pub struct Claims {
    sub: String,
    iss: String,
    exp: usize,
    aud: String,
    hd: Option<String>,
    azp: Option<String>,
}

impl Claims {
    pub fn sub(&self) -> &String {
        &self.sub
    }

    pub fn iss(&self) -> &String {
        &self.iss
    }

    pub fn exp(&self) -> usize {
        self.exp
    }

    pub fn aud(&self) -> &String {
        &self.aud
    }

    pub fn hd(&self) -> &Option<String> {
        &self.hd
    }

    pub fn azp(&self) -> &Option<String> {
        &self.azp
    }
}

#[derive(Debug, Deserialize)]
struct OpenIdMetadata {
    issuer: String,
    jwks_uri: String,
}

#[derive(Debug, Deserialize)]
struct JwkKey {
    kid: String,
    kty: String,
    n: String,
    e: String,
    alg: String,
    use_: Option<String>, // "sig"
}

#[derive(Debug, Deserialize)]
struct Jwks {
    keys: Vec<JwkKey>,
}

type TokenValidator = Arc<dyn Fn(&Claims) -> bool + Send + Sync>;

#[derive(Clone)]
pub(crate) struct ApprovedIssuer {
    well_known_url: String,
    validate: TokenValidator,
}

pub(crate) async fn validate_jwt_against_issuer(
    token: &str,
    issuer: ApprovedIssuer,
) -> Result<Claims, CommonError> {
    // Step 1: Decode token header to get `kid`
    let header =
        decode_header(token).map_err(|e| CommonError::Unknown(anyhow!("header error: {}", e)))?;
    let kid = header
        .kid
        .ok_or_else(|| CommonError::Unknown(anyhow!("missing kid")))?;

    // Step 2: Fetch .well-known OpenID metadata

    let meta_res = reqwest::get(issuer.well_known_url).await.unwrap();

    let meta: OpenIdMetadata = meta_res.json().await.unwrap();

    // Step 3: Fetch JWKS
    info!("fetching jwks from: {}", &meta.jwks_uri);
    let jwks: Jwks = reqwest::get(&meta.jwks_uri)
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    // Step 4: Find matching key
    let key = jwks
        .keys
        .into_iter()
        .find(|k| k.kid == kid)
        .ok_or_else(|| CommonError::Unknown(anyhow!("no matching key")))?;

    // Step 5: Build decoding key and validate
    let decoding_key = DecodingKey::from_rsa_components(&key.n, &key.e).unwrap();
    let mut validation = Validation::new(Algorithm::RS256);
    // most validation must happen in the issuer callback block.
    validation.validate_aud = false;

    let data = decode::<Claims>(token, &decoding_key, &validation).unwrap();

    Ok(data.claims)
}

fn create_internal_fly_issuer() -> ApprovedIssuer {
    // example decoded fly token
    // {
    //     "app_id": "11111111",
    //     "app_name": "example-app",
    //     "aud": "https://fly.io/exmaple-org",
    //     "exp": 1712099653,
    //     "iat": 1712099053,
    //     "image": "docker-hub-mirror.fly.io/you/image:latest",
    //     "image_digest": "sha256:2c1cdaded1b3820020c9dc9fdd1d6e798d6f6ca36861bb6ae64019fad6be9ee3",
    //     "iss": "https://oidc.fly.io/exmaple-org",
    //     "jti": "93ca09e1-70e0-477b-a260-1d8fcd4ef4f4",
    //     "machine_id": "148e21ea7e46e8",
    //     "machine_name": "example-machine",
    //     "machine_version": "01HTGGC1TZ2JHK83J4AC0R3VET",
    //     "nbf": 1712099053,
    //     "org_id": "11111111",
    //     "org_name": "example-org",
    //     "region": "sea",
    //     "sub": "example-org:example-app:example-machine"
    // }
    ApprovedIssuer {
        well_known_url: format!(
            "https://oidc.fly.io/{}/.well-known/openid-configuration",
            std::env::var("VDI_FLY_ORG_SLUG").unwrap()
        )
        .to_string(),
        validate: Arc::new(|claims: &Claims| {
            let sub_arr: Vec<&str> = claims.sub.split(":").collect();

            if sub_arr.len() == 3 {
                let (org_id, _app_name, _machine_id) = (sub_arr[0], sub_arr[1], sub_arr[2]);

                if org_id == std::env::var("VDI_FLY_ORG_SLUG").unwrap() {
                    return true;
                }
            }

            false
        }),
    }
}

fn create_internal_google_workspace_issuer() -> ApprovedIssuer {
    // {
    //     "iss": "https://accounts.google.com",
    //     "azp": "93160534746-gif5q3dnso26l2g1bdlfirm5c5dnjs7i.apps.googleusercontent.com",
    //     "aud": "93160534746-gif5q3dnso26l2g1bdlfirm5c5dnjs7i.apps.googleusercontent.com",
    //     "sub": "107313083092903712383",
    //     "hd": "trysoma.ai",
    //     "email": "daniel@trysoma.ai",
    //     "email_verified": true,
    //     "at_hash": "e_Qo_0QCGT5IoH3Y43SHpw",
    //     "name": "Daniel Blignaut",
    //     "picture": "https://lh3.googleusercontent.com/a/ACg8ocJ9cfsxG8b3KJQy_T-MbPmfFXa1k8l8vtE3T4liALnEbEQTiw=s96-c",
    //     "given_name": "Daniel",
    //     "family_name": "Blignaut",
    //     "iat": 1752741133,
    //     "exp": 1752744733
    //   }
    ApprovedIssuer {
        well_known_url: "https://accounts.google.com/.well-known/openid-configuration".to_string(),
        validate: Arc::new(|claims: &Claims| {
            let hd = if let Some(hd) = claims.hd() {
                hd
            } else {
                return false;
            };

            let azp = if let Some(azp) = claims.azp() {
                azp
            } else {
                return false;
            };

            let google_workspace_client_id =
                "93160534746-gif5q3dnso26l2g1bdlfirm5c5dnjs7i.apps.googleusercontent.com";

            hd == "trysoma.ai"
                && claims.aud() == google_workspace_client_id
                && azp == google_workspace_client_id
        }),
    }
}

fn create_local_mock_issuer() -> ApprovedIssuer {
    ApprovedIssuer {
        well_known_url: "http://0.0.0.0:5556/dex/.well-known/openid-configuration".to_string(),
        validate: Arc::new(|_claims: &Claims| true),
    }
}

pub(crate) fn extract_issuer(token: &str) -> Result<ApprovedIssuer, CommonError> {
    let approved_issuers = [
        create_internal_fly_issuer(),
        create_internal_google_workspace_issuer(),
        // this must always be last as at the moment, it will always match
        // TODO: come up with strategy to make approved issuers config driven so this isn't added
        // to deployed versions.
        create_local_mock_issuer(),
    ];

    let key = DecodingKey::from_secret(&[]);
    let mut validation = Validation::new(Algorithm::HS256);
    validation.insecure_disable_signature_validation();
    validation.validate_aud = false;

    let claims: Claims = decode(token, &key, &validation)
        .map_err(|e| CommonError::Unknown(anyhow!("decode failed: {}", e)))?
        .claims;

    let maybe_issuer = approved_issuers
        .iter()
        .find(|issuer| (issuer.validate)(&claims));

    match maybe_issuer {
        Some(issuer) => Ok(issuer.clone()),
        None => Err(CommonError::Unknown(anyhow!("issuer not found"))),
    }
}