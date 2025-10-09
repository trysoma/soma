extern crate proc_macro;

use std::{fs, path::Path};

use proc_macro::TokenStream;
use quote::quote;
use syn::{LitStr, parse_macro_input};

#[proc_macro]
pub fn load_sql_migrations(input: TokenStream) -> TokenStream {
    let path_lit = parse_macro_input!(input as LitStr);
    let path_str = path_lit.value();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let migrations_path = Path::new(&manifest_dir).join(&path_str);

    let supported_backends = vec!["sqlite"];

    let mut backend_map = std::collections::BTreeMap::<String, Vec<(String, String)>>::new();

    for entry in fs::read_dir(&migrations_path).expect("Failed to read migrations directory") {
        let entry = entry.expect("Invalid dir entry");
        let path = entry.path();
        if path.is_file() {
            let extension = path.extension().and_then(|e| e.to_str());
            if extension.is_none() {
                continue;
            }
            let extension = extension.unwrap();
            if extension != "sql" {
                continue;
            }
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                let contents = fs::read_to_string(&path).expect("Failed to read migration file");

                let mut matched_backend = None;
                for backend in &supported_backends {
                    if file_name.contains(&format!(".{backend}.")) {
                        matched_backend = Some(*backend);
                        break;
                    }
                }

                if let Some(backend) = matched_backend {
                    backend_map
                        .entry(backend.to_string())
                        .or_default()
                        .push((file_name.to_string(), contents));
                } else {
                    // No specific backend mentioned — add to ALL backends
                    for backend in &supported_backends {
                        backend_map
                            .entry(backend.to_string())
                            .or_default()
                            .push((file_name.to_string(), contents.clone()));
                    }
                }
            }
        }
    }

    let backend_tokens = backend_map.iter().map(|(backend, files)| {
        let file_tokens = files.iter().map(|(name, contents)| {
            quote! {
                map.insert(#name, #contents);
            }
        });

        quote! {
            {
                let mut map = ::std::collections::BTreeMap::new();
                #(#file_tokens)*
                migrations.insert(#backend, map);
            }
        }
    });

    let expanded = quote! {
        {
            let mut migrations = ::std::collections::BTreeMap::new();
            #(#backend_tokens)*
            migrations
        }
    };

    TokenStream::from(expanded)
}
