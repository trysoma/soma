//! Authentication and authorization macros
//!
//! This module provides procedural macros for authentication and authorization:
//!
//! - `#[authn]` - Ensures the user is authenticated by injecting auth_client and credentials parameters
//! - `#[authz_role(...)]` - Role-based authorization check with optional permission name for logging

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Ident, ItemFn, ReturnType, Token, parse_macro_input};

/// Authentication macro that ensures the caller is authenticated.
///
/// This macro transforms a function to:
/// 1. Add `__auth_client: impl AuthClientLike` as the first parameter
/// 2. Add `__credentials: impl Into<RawCredentials>` as the second parameter
/// 3. Add `identity: Identity` as the first parameter of the original function signature
/// 4. Call `__auth_client.authenticate(__credentials.into()).await?` at the start
/// 5. Check authentication and pass the `Identity` to the function body
///
/// # Example
///
/// ```rust,ignore
/// #[authn]
/// pub async fn create_user(
///     identity: Identity,  // The macro adds this as the first param
///     repo: &impl UserRepositoryLike,
///     params: CreateUserParams,
/// ) -> Result<User, CommonError> {
///     // `identity` is the authenticated identity
///     // Function will return early with CommonError::Authentication if not authenticated
///     // ...
/// }
/// ```
///
/// The macro adds two hidden parameters before the function's explicit parameters:
/// - `__auth_client: impl AuthClientLike` - The auth client for authentication
/// - `__credentials: impl Into<RawCredentials>` - The credentials (HeaderMap, Identity, etc.)
///
/// Callers invoke like:
/// ```rust,ignore
/// create_user(auth_client, headers, repo, params).await
/// // or
/// create_user(auth_client, existing_identity, repo, params).await
/// ```
pub fn authn_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let _ = attr; // No attributes expected for now
    let input = parse_macro_input!(item as ItemFn);

    let vis = &input.vis;
    let sig = &input.sig;
    let block = &input.block;
    let attrs = &input.attrs;

    // Extract function components
    let fn_name = &sig.ident;
    let generics = &sig.generics;
    let where_clause = &sig.generics.where_clause;
    let output = &sig.output;
    let asyncness = &sig.asyncness;

    // Check if async
    if asyncness.is_none() {
        return syn::Error::new_spanned(sig, "#[authn] can only be applied to async functions")
            .to_compile_error()
            .into();
    }

    // Check return type is Result
    let _return_type = match output {
        ReturnType::Type(_, ty) => ty,
        ReturnType::Default => {
            return syn::Error::new_spanned(
                sig,
                "#[authn] requires a return type of Result<T, CommonError>",
            )
            .to_compile_error()
            .into();
        }
    };

    // Get the original parameters - the first one should be `identity: Identity`
    let original_params = &sig.inputs;

    // Generate the authentication check
    // Note: We use __authn_ prefix to avoid conflicts with crates named 'identity'
    // The __authn_identity is used by authz_role macro, then we bind it to `identity` for the user
    let auth_check = quote! {
        let __authn_identity = __auth_client.authenticate(__credentials.into()).await?;
        if !__authn_identity.is_authenticated() {
            return Err(::shared::error::CommonError::Authentication {
                msg: "Authentication required".to_string(),
                source: None,
            });
        }
        // Bind to `identity` for use in function body (shadowing the parameter declaration)
        let identity = __authn_identity.clone();
    };

    // Build the expanded function
    let expanded = quote! {
        #(#attrs)*
        #vis #asyncness fn #fn_name #generics(
            __auth_client: impl ::shared::identity::AuthClientLike,
            __credentials: impl Into<::shared::identity::RawCredentials>,
            #original_params
        ) #output #where_clause {
            #auth_check
            #block
        }
    };

    expanded.into()
}

/// Role authorization attribute arguments
///
/// Supports two formats:
/// - `#[authz_role(Admin, User)]` - just roles
/// - `#[authz_role(Admin, User, permission = "user:write")]` - roles with permission name
pub struct AuthzRoleArgs {
    pub roles: Vec<Ident>,
    pub permission: Option<String>,
}

impl syn::parse::Parse for AuthzRoleArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut roles = Vec::new();
        let mut permission = None;

        while !input.is_empty() {
            // Check if this is the permission = "..." part
            if input.peek(Ident) {
                let ident: Ident = input.parse()?;
                if ident == "permission" {
                    input.parse::<Token![=]>()?;
                    let lit: syn::LitStr = input.parse()?;
                    permission = Some(lit.value());
                } else {
                    roles.push(ident);
                }
            }

            // Consume optional comma
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(AuthzRoleArgs { roles, permission })
    }
}

/// Role-based authorization macro with optional permission name.
///
/// This macro checks if the authenticated identity has one of the specified roles.
/// An optional `permission` parameter can be provided for logging and error messages.
///
/// **Important**: This macro must be placed ABOVE `#[authn]` so that `#[authn]` runs first
/// (attributes are applied bottom-to-top) and `__authn_identity` is available.
///
/// # Example
///
/// ```rust,ignore
/// #[authz_role(Admin, permission = "user:write")]
/// #[authn]  // authn runs first (closest to fn), creates __authn_identity
/// pub async fn admin_only_action(
///     repo: &impl Repository,
/// ) -> Result<(), CommonError> {
///     // Only Admin can access
///     // __authn_identity is available from #[authn]
/// }
///
/// #[authz_role(Admin, Maintainer, permission = "config:read")]
/// #[authn]
/// pub async fn admin_or_maintainer(
///     repo: &impl Repository,
/// ) -> Result<(), CommonError> {
///     // Admin or Maintainer can access
/// }
/// ```
pub fn authz_role_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as AuthzRoleArgs);
    let input = parse_macro_input!(item as ItemFn);

    let vis = &input.vis;
    let sig = &input.sig;
    let block = &input.block;
    let attrs = &input.attrs;

    // Extract function components
    let fn_name = &sig.ident;
    let generics = &sig.generics;
    let where_clause = &sig.generics.where_clause;
    let output = &sig.output;
    let asyncness = &sig.asyncness;
    let inputs = &sig.inputs;

    // Build role checks
    let role_checks: Vec<TokenStream2> = args
        .roles
        .iter()
        .map(|role| {
            quote! {
                ::shared::identity::Role::#role
            }
        })
        .collect();

    // Note: We expect a variable named `__authn_identity` from the #[authn] macro
    let role_check = if role_checks.is_empty() {
        quote! { false }
    } else {
        quote! {
            matches!(__authn_identity.role(), #(Some(&#role_checks))|*)
        }
    };

    let role_names: Vec<String> = args.roles.iter().map(|r| r.to_string()).collect();
    let role_names_str = role_names.join(", ");
    let fn_name_str = fn_name.to_string();

    // Use permission name if provided, otherwise use function name
    let permission_str = args.permission.unwrap_or_else(|| fn_name_str.clone());

    // Generate the authorization check with logging
    // Uses __authn_identity variable from the #[authn] macro
    let authz_check = quote! {
        {
            let __authz_role = __authn_identity.role().map(|r| r.as_str()).unwrap_or("none");
            let __authz_sub = __authn_identity.subject().unwrap_or("unknown");

            ::tracing::trace!(
                permission = #permission_str,
                role = __authz_role,
                subject = __authz_sub,
                required_roles = #role_names_str,
                "Checking role authorization"
            );

            if !(#role_check) {
                ::tracing::debug!(
                    permission = #permission_str,
                    role = __authz_role,
                    subject = __authz_sub,
                    required_roles = #role_names_str,
                    "Authorization denied: insufficient role"
                );
                return Err(::shared::error::CommonError::Authorization {
                    msg: format!(
                        "Permission '{}' denied. Required role(s): {}. Your role: {}",
                        #permission_str,
                        #role_names_str,
                        __authz_role
                    ),
                    source: ::anyhow::anyhow!("Role authorization failed for permission '{}'", #permission_str),
                });
            }

            ::tracing::debug!(
                permission = #permission_str,
                role = __authz_role,
                subject = __authz_sub,
                "Authorization granted"
            );
        }
    };

    // Build the expanded function
    let expanded = quote! {
        #(#attrs)*
        #vis #asyncness fn #fn_name #generics(#inputs) #output #where_clause {
            #authz_check
            #block
        }
    };

    expanded.into()
}
