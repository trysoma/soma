use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

/// Marks a test as an integration test that requires external services.
///
/// When the `CI` environment variable is NOT set, the test runs normally.
/// When `CI` is set, the test is skipped with a message indicating it requires
/// external services.
///
/// This allows integration tests to be written alongside unit tests without
/// feature flags, while still being skippable in CI environments that don't
/// have external services configured.
///
/// # Example
///
/// ```rust,ignore
/// #[cfg(test)]
/// mod tests {
///     mod integration {
///         use super::*;
///         use shared_macros::integration_test;
///
///         #[integration_test]
///         async fn test_with_external_service() {
///             // This test will be skipped when CI env var is set
///         }
///     }
/// }
/// ```
pub fn integration_test_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_attrs = &input_fn.attrs;
    let fn_block = &input_fn.block;
    let fn_sig = &input_fn.sig;

    // Check if async
    let is_async = fn_sig.asyncness.is_some();

    let skip_check = quote! {

        if std::env::var("RUN_INTEGRATION_TESTS").is_err() && std::env::var("CI").is_ok() {
            eprintln!(
                "Skipping integration test '{}': CI environment detected. \
                Set up external services or run locally to execute this test.",
                stringify!(#fn_name)
            );
            return;
        }
    };

    let expanded = if is_async {
        quote! {
            #(#fn_attrs)*
            #[tokio::test]
            #fn_vis async fn #fn_name() {
                #skip_check
                #fn_block
            }
        }
    } else {
        quote! {
            #(#fn_attrs)*
            #[test]
            #fn_vis fn #fn_name() {
                #skip_check
                #fn_block
            }
        }
    };

    TokenStream::from(expanded)
}
