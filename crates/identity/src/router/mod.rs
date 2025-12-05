mod api_key;
mod auth;
mod jwk;
mod sts_config;
mod sts_exchange;
mod user_auth_flow_config;

use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use time::Duration;
use utoipa_axum::router::OpenApiRouter;

use crate::logic::internal_token_issuance::NormalizedTokenIssuanceResult;
use crate::service::IdentityService;

pub const PATH_PREFIX: &str = "/api";
pub const API_VERSION_1: &str = "v1";
pub const SERVICE_ROUTE_KEY: &str = "identity";

// Cookie names for access and refresh tokens
pub const ACCESS_TOKEN_COOKIE_NAME: &str = "soma_access_token";
pub const REFRESH_TOKEN_COOKIE_NAME: &str = "soma_refresh_token";

/// Access token expiration in seconds (1 hour)
pub const ACCESS_TOKEN_MAX_AGE_SECONDS: i64 = 3600;
/// Refresh token expiration in seconds (7 days)
pub const REFRESH_TOKEN_MAX_AGE_SECONDS: i64 = 86400 * 7;

/// Add access and refresh token cookies to the cookie jar.
///
/// Cookie settings are permissive to allow cross-domain usage:
/// - SameSite::Lax (allows cookies on top-level navigations)
/// - Secure: false (allows non-HTTPS in development)
/// - HttpOnly: true (prevents JavaScript access for security)
pub fn add_token_cookies(jar: CookieJar, tokens: &NormalizedTokenIssuanceResult) -> CookieJar {
    add_token_cookies_with_options(jar, tokens, true)
}

/// Add access and refresh token cookies to the cookie jar, with option to include refresh token.
///
/// Cookie settings are permissive to allow cross-domain usage:
/// - SameSite::Lax (allows cookies on top-level navigations)
/// - Secure: false (allows non-HTTPS in development)
/// - HttpOnly: true (prevents JavaScript access for security)
pub fn add_token_cookies_with_options(
    jar: CookieJar,
    tokens: &NormalizedTokenIssuanceResult,
    include_refresh_token: bool,
) -> CookieJar {
    // Build access token cookie
    let access_cookie = Cookie::build((ACCESS_TOKEN_COOKIE_NAME, tokens.access_token.clone()))
        .path("/")
        .secure(false) // Allow non-HTTPS for development
        .http_only(true)
        .same_site(SameSite::Lax) // Allow cross-site top-level navigations
        .max_age(Duration::seconds(ACCESS_TOKEN_MAX_AGE_SECONDS))
        .build();

    let jar = jar.add(access_cookie);

    // Optionally add refresh token cookie
    if include_refresh_token {
        let refresh_cookie =
            Cookie::build((REFRESH_TOKEN_COOKIE_NAME, tokens.refresh_token.clone()))
                .path("/")
                .secure(false) // Allow non-HTTPS for development
                .http_only(true)
                .same_site(SameSite::Lax) // Allow cross-site top-level navigations
                .max_age(Duration::seconds(REFRESH_TOKEN_MAX_AGE_SECONDS))
                .build();

        jar.add(refresh_cookie)
    } else {
        jar
    }
}

pub fn create_router() -> OpenApiRouter<IdentityService> {
    OpenApiRouter::new()
        .merge(api_key::create_api_key_routes())
        .merge(auth::create_auth_routes())
        .merge(jwk::create_jwk_routes())
        .merge(sts_config::create_sts_config_routes())
        .merge(sts_exchange::create_sts_routes())
        .merge(user_auth_flow_config::create_user_auth_flow_config_routes())
}
