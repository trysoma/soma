//! Authentication and authorization macros
//!
//! This module provides procedural macros for authentication and authorization:
//!
//! - `#[authn]` - Ensures the user is authenticated by injecting auth_client and headers parameters
//! - `#[authz_role(...)]` - Role-based authorization check (stackable, at least one must pass)

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Ident, ItemFn, ReturnType, Token, parse_macro_input, punctuated::Punctuated};

/// Authentication macro that ensures the caller is authenticated.
///
/// This macro transforms a function to:
/// 1. Add `auth_client: impl AuthClientLike` as the first parameter
/// 2. Add `headers: HeaderMap` as the second parameter
/// 3. Call `auth_client.authenticate_from_headers(&headers).await?` at the start
/// 4. Make the resulting `Identity` available as `identity` in the function body
///
/// # Example
///
/// ```rust,ignore
/// #[authn]
/// pub async fn create_user(
///     repo: &impl UserRepositoryLike,
///     params: CreateUserParams,
/// ) -> Result<User, CommonError> {
///     // `identity` is available here as the authenticated identity
///     // Function will return early with CommonError::Authentication if not authenticated
///     // ...
/// }
/// ```
///
/// Expands to:
///
/// ```rust,ignore
/// pub async fn create_user(
///     auth_client: impl AuthClientLike,
///     headers: HeaderMap,
///     repo: &impl UserRepositoryLike,
///     params: CreateUserParams,
/// ) -> Result<User, CommonError> {
///     let identity = auth_client.authenticate_from_headers(&headers).await?;
///     if !identity.is_authenticated() {
///         return Err(CommonError::Authentication {
///             msg: "Authentication required".to_string(),
///             source: None,
///         });
///     }
///     // Original function body...
/// }
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

    // Build new parameters list: auth_client, headers, then original params
    let original_params = &sig.inputs;

    // Generate the authentication check
    let auth_check = quote! {
        let __identity = __auth_client.authenticate_from_headers(&__headers).await?;
        if !__identity.is_authenticated() {
            return Err(::shared::error::CommonError::Authentication {
                msg: "Authentication required".to_string(),
                source: None,
            });
        }
        let identity = __identity;
    };

    // Build the expanded function
    let expanded = quote! {
        #(#attrs)*
        #vis #asyncness fn #fn_name #generics(
            __auth_client: impl ::shared::identity::AuthClientLike,
            __headers: ::http::HeaderMap,
            #original_params
        ) #output #where_clause {
            #auth_check
            #block
        }
    };

    expanded.into()
}

/// Role authorization attribute arguments
pub struct AuthzRoleArgs {
    pub roles: Vec<Ident>,
}

impl syn::parse::Parse for AuthzRoleArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let roles: Punctuated<Ident, Token![,]> = Punctuated::parse_terminated(input)?;
        Ok(AuthzRoleArgs {
            roles: roles.into_iter().collect(),
        })
    }
}

/// Role-based authorization macro (stackable).
///
/// This macro checks if the authenticated identity has one of the specified roles.
/// Multiple `#[authz_role(...)]` attributes can be stacked - at least one must pass.
///
/// **Important**: This macro must be used after `#[authn]` as it expects `identity` to be available.
///
/// # Example
///
/// ```rust,ignore
/// #[authn]
/// #[authz_role(Admin)]
/// pub async fn admin_only_action(
///     repo: &impl Repository,
/// ) -> Result<(), CommonError> {
///     // Only Admin can access
/// }
///
/// #[authn]
/// #[authz_role(Admin, Maintainer)]
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

    let role_check = if role_checks.is_empty() {
        quote! { false }
    } else {
        quote! {
            matches!(identity.role(), #(Some(&#role_checks))|*)
        }
    };

    let role_names: Vec<String> = args.roles.iter().map(|r| r.to_string()).collect();
    let role_names_str = role_names.join(", ");

    // Generate the authorization check
    let authz_check = quote! {
        if !(#role_check) {
            return Err(::shared::error::CommonError::Authorization {
                msg: format!("Access denied. Required role(s): {}", #role_names_str),
                source: ::anyhow::anyhow!("Role authorization failed"),
            });
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
