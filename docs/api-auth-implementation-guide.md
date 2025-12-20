# API Authentication & Authorization Implementation Guide

This guide explains how to add authentication and authorization to API routes using the `#[authn]` and `#[authz_role]` macros.

## Overview

The authentication/authorization system uses two macros:

1. **`#[authn]`** - Adds authentication to a logic function
2. **`#[authz_role(...)]`** - Adds role-based authorization (must be combined with `#[authn]`)

## Macro Order (Important!)

The macros must be ordered with `#[authz_role]` **above** `#[authn]`:

```rust
#[authz_role(Admin, Maintainer, permission = "provider:list")]
#[authn]  // authn runs first (closest to fn), creates __authn_identity
pub async fn my_function(
    _identity: Identity,  // Placeholder, shadowed by macro
    other_params...
) -> Result<..., CommonError> {
    // `identity` is available here from the macro
}
```

This ordering is required because Rust applies attributes bottom-to-top. `#[authn]` must run first to create the `__authn_identity` variable that `#[authz_role]` uses.

## Step-by-Step Implementation

### 1. Add Auth Macros to Logic Function

In your logic module (e.g., `crates/bridge/src/logic/controller.rs`):

```rust
use shared::identity::Identity;
use shared_macros::{authn, authz_role};

/// List all available provider types
#[authz_role(Admin, Maintainer, permission = "provider:list")]
#[authn]
pub async fn list_available_providers(
    _identity: Identity,  // Placeholder parameter, shadowed by macro
    pagination: ListAvailableProvidersParams,
) -> Result<ListAvailableProvidersResponse, CommonError> {
    // `identity` is available from the #[authn] macro (shadows the _identity parameter)
    // You can use it like: let _ = &identity;
    let providers = PROVIDER_REGISTRY.read()...
}
```

The macros transform the function signature by adding two new parameters at the beginning:
- `__auth_client: impl AuthClientLike`
- `__credentials: impl Into<RawCredentials>` - Can accept `HeaderMap`, `Identity`, or other `RawCredentials` variants

The `identity: Identity` parameter you write is a placeholder that gets shadowed by the authenticated identity from the macro.

### 2. Update Route Handler

In your router (e.g., `crates/bridge/src/router/provider.rs`):

```rust
use http::HeaderMap;
use shared::identity::Identity;

pub async fn route_list_available_providers(
    State(ctx): State<BridgeService>,
    headers: HeaderMap,  // Extract headers from request
    Query(pagination): Query<PaginationRequest>,
) -> JsonResponse<ListAvailableProvidersResponse, CommonError> {
    // Note: identity parameter is a placeholder that gets shadowed by the #[authn] macro
    let identity_placeholder = Identity::Unauthenticated;
    let res = list_available_providers(
        ctx.auth_client().clone(),  // Clone is cheap - AuthClient only contains Arcs
        headers,                     // Credentials (impl Into<RawCredentials>)
        identity_placeholder,        // Placeholder identity (shadowed by macro)
        pagination,
    ).await;
    JsonResponse::from(res)
}
```

### 3. Permission Reference

See `docs/api-permission-mapping.md` for the complete list of permissions and which roles can access each endpoint.

Common patterns:
- **Admin only**: `#[authz_role(Admin, permission = "resource:write")]`
- **Admin + Maintainer**: `#[authz_role(Admin, Maintainer, permission = "resource:read")]`
- **Admin + Maintainer + Agent**: `#[authz_role(Admin, Maintainer, Agent, permission = "resource:list")]`
- **All authenticated users**: `#[authz_role(Admin, Maintainer, Agent, User, permission = "auth:whoami")]`

## Testing

For unit tests, use `MockAuthClient` from the shared crate:

```rust
#[cfg(all(test, feature = "unit_test"))]
mod unit_test {
    use super::*;
    use http::HeaderMap;
    use shared::identity::Identity;
    use shared::test_utils::helpers::MockAuthClient;

    #[tokio::test]
    async fn test_list_available_providers() {
        shared::setup_test!();

        let auth_client = MockAuthClient::admin();  // Returns admin identity
        let headers = HeaderMap::new();
        // Note: identity parameter is a placeholder that gets shadowed by the #[authn] macro
        let identity_placeholder = Identity::Unauthenticated;
        let pagination = PaginationRequest { page_size: 10, next_page_token: None };

        let result = list_available_providers(
            auth_client,
            headers,
            identity_placeholder,
            pagination,
        ).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unauthorized_access() {
        shared::setup_test!();

        let auth_client = MockAuthClient::user();  // User role
        let headers = HeaderMap::new();
        let identity_placeholder = Identity::Unauthenticated;
        let pagination = PaginationRequest { page_size: 10, next_page_token: None };

        // Should fail because User role doesn't have Admin/Maintainer permission
        let result = list_available_providers(
            auth_client,
            headers,
            identity_placeholder,
            pagination,
        ).await;
        assert!(result.is_err());
    }
}
```

Available MockAuthClient methods:
- `MockAuthClient::admin()` - Admin role
- `MockAuthClient::maintainer()` - Maintainer role
- `MockAuthClient::agent()` - Agent role
- `MockAuthClient::user()` - User role
- `MockAuthClient::unauthenticated()` - Unauthenticated
- `MockAuthClient::new(identity)` - Custom identity

## Passing Already-Authenticated Identity

The `#[authn]` macro accepts `impl Into<RawCredentials>`, so you can pass:
- `HeaderMap` - Authentication will be performed from headers
- `Identity` - Skip authentication, use this identity directly (useful for internal calls)

```rust
// From external request - authenticate from headers
list_available_providers(auth_client, headers, identity_placeholder, pagination).await;

// Internal call with known identity - skip re-authentication
let known_identity = Identity::Machine(Machine { sub: "...", role: Role::Admin });
list_available_providers(auth_client, known_identity, identity_placeholder, pagination).await;
```

## Public Endpoints (No Auth Required)

Some endpoints don't require authentication. For these, don't add the macros:

```rust
// No macros - public endpoint
pub async fn route_health_check(...) -> ... {
    // Anyone can access
}
```

Public endpoints per `docs/api-permission-mapping.md`:
- `GET /_internal/v1/health`
- `GET /api/identity/v1/.well-known/jwks.json`
- `GET /api/identity/v1/auth/authorize/{config_id}`
- `GET /api/identity/v1/auth/callback`
- `GET /api/bridge/v1/generic-oauth-callback`

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Route Handler                          │
│  - Extracts HeaderMap from request                          │
│  - Gets auth_client from state                              │
│  - Calls logic function with:                               │
│    (auth_client, credentials, identity_placeholder, ...)    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      Logic Function                          │
│  #[authz_role(Admin, permission = "foo:bar")]               │
│  #[authn]                                                    │
│  async fn do_something(                                      │
│      _identity: Identity,   // Placeholder                   │
│      original_params...                                      │
│  ) -> Result<...>                                            │
│                                                              │
│  Macro expansion adds:                                       │
│  1. __auth_client and __credentials parameters              │
│  2. Authentication check at start                            │
│  3. Authorization check (role validation)                    │
│  4. `identity` variable available in body (shadows param)   │
└─────────────────────────────────────────────────────────────┘
```

## Checklist for Each Route

- [ ] Identify the permission from `docs/api-permission-mapping.md`
- [ ] Add imports: `use shared::identity::Identity;` and `use shared_macros::{authn, authz_role};`
- [ ] Add `#[authz_role(...)]` macro with correct roles and permission
- [ ] Add `#[authn]` macro below authz_role
- [ ] Add `_identity: Identity` as the first parameter (placeholder, gets shadowed)
- [ ] Update route handler to:
  - Extract `HeaderMap` from request
  - Create `identity_placeholder = Identity::Unauthenticated`
  - Pass all params to logic function
- [ ] Update any tests that call the logic function directly
- [ ] Run tests to verify: `cargo test --package <crate> --features unit_test`
