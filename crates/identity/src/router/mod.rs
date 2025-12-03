mod api_key;
mod jwk;
mod sts_config;
mod sts_exchange;

use utoipa_axum::router::OpenApiRouter;

use crate::service::IdentityService;

pub const PATH_PREFIX: &str = "/api";
pub const API_VERSION_1: &str = "v1";
pub const SERVICE_ROUTE_KEY: &str = "identity";

// Cookie names for access and refresh tokens
pub const ACCESS_TOKEN_COOKIE_NAME: &str = "soma_access_token";
pub const REFRESH_TOKEN_COOKIE_NAME: &str = "soma_refresh_token";

pub fn create_router() -> OpenApiRouter<IdentityService> {
    OpenApiRouter::new()
        .merge(api_key::create_api_key_routes())
        .merge(jwk::create_jwk_routes())
        .merge(sts_config::create_sts_config_routes())
        .merge(sts_exchange::create_sts_routes())
}
