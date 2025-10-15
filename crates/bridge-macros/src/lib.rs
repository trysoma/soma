use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token, Ident, LitStr, Token, Expr, ExprStruct,
};

// -----------------------------------------------------------------------------
// Input DSL parsing
// -----------------------------------------------------------------------------

struct DefineProviderInput {
    provider_id: Ident,
    body: ProviderBody,
}

struct ProviderBody {
    id: LitStr,
    name: LitStr,
    docs: LitStr,
    flows: Vec<FlowDef>,
    default_scopes: Vec<LitStr>,
}

struct FlowDef {
    flow_name: Ident,
    static_credentials: Option<Expr>,
}

impl Parse for DefineProviderInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let provider_id: Ident = input.parse()?;
        let content;
        braced!(content in input);
        let body = ProviderBody::parse(&content)?;
        Ok(Self { provider_id, body })
    }
}

impl Parse for ProviderBody {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Ident>()?; // id
        input.parse::<Token![:]>()?;
        let id: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;

        input.parse::<Ident>()?; // name
        input.parse::<Token![:]>()?;
        let name: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;

        input.parse::<Ident>()?; // docs
        input.parse::<Token![:]>()?;
        let docs: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;

        input.parse::<Ident>()?; // flows
        input.parse::<Token![:]>()?;
        let flow_outer;
        bracketed!(flow_outer in input);
        let flows = Punctuated::<FlowDef, Token![,]>::parse_terminated(&flow_outer)?
            .into_iter()
            .collect::<Vec<_>>();
        input.parse::<Token![,]>()?;

        input.parse::<Ident>()?; // default_scopes
        input.parse::<Token![:]>()?;
        let scopes_inner;
        bracketed!(scopes_inner in input);
        let default_scopes = Punctuated::<LitStr, Token![,]>::parse_terminated(&scopes_inner)?
            .into_iter()
            .collect::<Vec<_>>();
        input.parse::<Token![,]>().ok();

        Ok(Self {
            id,
            name,
            docs,
            flows,
            default_scopes,
        })
    }
}
impl Parse for FlowDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let flow_outer;
        braced!(flow_outer in input);

        // Parse flow name: e.g. Oauth2AuthorizationCodeFlow:
        let flow_name: Ident = flow_outer.parse()?;
        flow_outer.parse::<Token![:]>()?;

        // Inner flow body: { static_credentials: SomeType { ... } }
        let flow_body;
        braced!(flow_body in flow_outer);

        let mut static_credentials: Option<Expr> = None;

        while !flow_body.is_empty() {
            let key: Ident = flow_body.parse()?;
            flow_body.parse::<Token![:]>()?;

            match key.to_string().as_str() {
                "static_credentials" => {
                    // Parse as any expression (struct literal is an ExprStruct)
                    let expr: Expr = if flow_body.peek(Ident) && flow_body.peek2(token::Brace) {
                        // explicitly parse a struct literal expression
                        Expr::Struct(flow_body.parse::<ExprStruct>()?)
                    } else {
                        flow_body.parse()?
                    };
                    static_credentials = Some(expr);
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        key,
                        "Unknown key in flow block (expected: static_credentials: ...)",
                    ));
                }
            }

            // Optional trailing comma after key-value pair
            if flow_body.peek(Token![,]) {
                flow_body.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            flow_name,
            static_credentials,
        })
    }
}

// -----------------------------------------------------------------------------
// Macro implementation
// -----------------------------------------------------------------------------

#[proc_macro]
pub fn define_provider(input: TokenStream) -> TokenStream {
    let DefineProviderInput { provider_id, body } = syn::parse_macro_input!(input as DefineProviderInput);

    let ProviderBody {
        id,
        name,
        docs,
        flows,
        default_scopes,
    } = body;

    let controller_ident = format_ident!("{}Controller", pascal(&provider_id));
    let variant_ident = format_ident!("{}Variant", pascal(&provider_id));
    let instance_ident = format_ident!("{}Instance", pascal(&provider_id));

    let flow_names: Vec<_> = flows.iter().map(|f| &f.flow_name).collect();

    // default scopes vec
    let scope_vec = default_scopes.iter().map(|s| quote! { #s.to_string() });

    // Match arms for save_resource_server_credential and save_user_credential
    let match_arms_resource = flow_names.iter().map(|f| quote! {
        ResourceServerCredentialVariant::#f(_) => (),
    });
    let match_arms_user = flow_names.iter().map(|f| quote! {
        UserCredentialVariant::#f(_) => (),
    });

    // Match arms for get_static_credentials
    let static_match_arms = flows.iter().map(|f| {
        let flow = &f.flow_name;
        if let Some(static_expr) = &f.static_credentials {
            quote! {
                StaticCredentialConfigurationType::#flow => {
                    let creds = #static_expr;
                    return Ok(StaticCredentialConfiguration {
                        inner: StaticCredentialConfigurationVariant::#flow(creds),
                        metadata: Metadata::new(),
                    });
                }
            }
        } else {
            quote! {
                StaticCredentialConfigurationType::#flow => {
                    return Err(CommonError::InvalidRequest {
                        msg: concat!("No static credentials configured for ", stringify!(#flow)).into(),
                        source: None,
                    });
                }
            }
        }
    });

    let full_credential_idents: Vec<proc_macro2::Ident> = flow_names
        .iter()
        .map(|f| format_ident!("{}FullCredential", f))
        .collect();

    let enum_variants = flow_names.iter().zip(full_credential_idents.iter()).map(|(flow, full)| {
        quote! { #flow(#full), }
    });


    let expanded = quote! {
        use serde::{Serialize, Deserialize};
        use crate::logic::*;
        use shared::{error::CommonError, primitives::{WrappedChronoDateTime, WrappedUuidV4}};
    
        pub struct #controller_ident;
    
        impl ProviderControllerLike for #controller_ident {
            type ProviderInstance = #instance_ident;
    
            async fn save_resource_server_credential(
                input: ResourceServerCredentialVariant,
            ) -> Result<ResourceServerCredential, CommonError> {
                match input {
                    #(#match_arms_resource)*
                    _ => return Err(CommonError::InvalidRequest {
                        msg: concat!("Unsupported credential type for ", stringify!(#provider_id)).into(),
                        source: None,
                    }),
                };
                Ok(ResourceServerCredential {
                    id: WrappedUuidV4::new(),
                    created_at: WrappedChronoDateTime::now(),
                    updated_at: WrappedChronoDateTime::now(),
                    inner: input,
                    metadata: Metadata::new(),
                })
            }
    
            async fn save_user_credential(
                input: UserCredentialVariant,
            ) -> Result<UserCredential, CommonError> {
                match input {
                    #(#match_arms_user)*
                    _ => return Err(CommonError::InvalidRequest {
                        msg: concat!("Unsupported user credential type for ", stringify!(#provider_id)).into(),
                        source: None,
                    }),
                };
                Ok(UserCredential {
                    id: WrappedUuidV4::new(),
                    created_at: WrappedChronoDateTime::now(),
                    updated_at: WrappedChronoDateTime::now(),
                    inner: input,
                    metadata: Metadata::new(),
                })
            }
    
            async fn get_static_credentials(
                variant: StaticCredentialConfigurationType,
            ) -> Result<StaticCredentialConfiguration, CommonError> {
                match variant {
                    #(#static_match_arms)*
                    _ => Err(CommonError::InvalidRequest {
                        msg: concat!("No static credentials configured for ", stringify!(#provider_id)).into(),
                        source: None,
                    }),
                }
            }
    
            fn id() -> String { #id.to_string() }
            fn name() -> String { #name.to_string() }
            fn documentation_url() -> String { #docs.to_string() }
        }
    
        #[derive(Serialize, Deserialize)]
        #[serde(tag = "type", rename_all = "snake_case")]
        pub enum #variant_ident {
            #(#enum_variants)*
        }

        #[derive(Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct #instance_ident(pub #variant_ident);
    };

    TokenStream::from(expanded)
}

// -----------------------------------------------------------------------------
// helpers
// -----------------------------------------------------------------------------

fn pascal(ident: &Ident) -> String {
    let s = ident.to_string();
    let mut out = String::new();
    let mut upper = true;
    for c in s.chars() {
        if c == '_' {
            upper = true;
        } else if upper {
            out.push(c.to_ascii_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}