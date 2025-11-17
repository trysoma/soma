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

#[proc_macro]
pub fn load_atlas_sql_migrations(input: TokenStream) -> TokenStream {
    let path_lit = parse_macro_input!(input as LitStr);
    let path_str = path_lit.value();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let migrations_path = Path::new(&manifest_dir).join(&path_str);

    let supported_backends = vec!["sqlite"];

    let mut backend_map = std::collections::BTreeMap::<String, Vec<(String, String)>>::new();

    for entry in fs::read_dir(&migrations_path).expect("Failed to read migrations directory") {
        let entry = entry.expect("Invalid dir entry");
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let extension = path.extension().and_then(|e| e.to_str());
        if extension != Some("sql") {
            continue;
        }

        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            let contents = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Failed to read migration file {file_name}: {e}"));

            // Parse the goose format migration (-- +goose Up / -- +goose Down)
            let (migration_sql, down_sql) = parse_goose_format(&contents, file_name);

            // Determine which backend(s) this migration applies to
            let mut matched_backend = None;
            for backend in &supported_backends {
                if file_name.contains(&format!(".{backend}.")) {
                    matched_backend = Some(*backend);
                    break;
                }
            }

            // Generate .up.sql and .down.sql filenames
            let base_name = file_name.trim_end_matches(".sql");
            let up_filename = format!("{base_name}.up.sql");
            let down_filename = format!("{base_name}.down.sql");

            if let Some(backend) = matched_backend {
                backend_map
                    .entry(backend.to_string())
                    .or_default()
                    .push((up_filename.clone(), migration_sql.clone()));
                backend_map
                    .entry(backend.to_string())
                    .or_default()
                    .push((down_filename.clone(), down_sql.clone()));
            } else {
                // No specific backend mentioned — add to ALL backends
                for backend in &supported_backends {
                    backend_map
                        .entry(backend.to_string())
                        .or_default()
                        .push((up_filename.clone(), migration_sql.clone()));
                    backend_map
                        .entry(backend.to_string())
                        .or_default()
                        .push((down_filename.clone(), down_sql.clone()));
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

/// Parse goose format migration file
/// Format:
/// ```text
/// -- +goose Up
/// CREATE TABLE ...;
///
/// -- +goose Down
/// DROP TABLE ...;
/// ```
fn parse_goose_format(contents: &str, filename: &str) -> (String, String) {
    let lines: Vec<&str> = contents.lines().collect();

    let mut up_sql = Vec::new();
    let mut down_sql = Vec::new();
    let mut current_section: Option<&str> = None;

    for line in lines.iter() {
        let trimmed = line.trim();

        // Check for goose section markers
        if trimmed == "-- +goose Up" {
            current_section = Some("up");
            continue;
        } else if trimmed == "-- +goose Down" {
            current_section = Some("down");
            continue;
        }

        // Add line to appropriate section
        match current_section {
            Some("up") => up_sql.push(*line),
            Some("down") => down_sql.push(*line),
            Some(_) => {
                // Unknown section marker, skip
            }
            None => {
                // If we haven't seen a section marker yet, skip the line
                // (allows for comments or empty lines before the first section)
            }
        }
    }

    // Validate that we found both sections
    if up_sql.is_empty() {
        panic!("Migration file '{filename}' must contain '-- +goose Up' section");
    }
    if down_sql.is_empty() {
        panic!("Migration file '{filename}' must contain '-- +goose Down' section");
    }

    let migration_sql = up_sql.join("\n").trim().to_string();
    let down_sql_content = down_sql.join("\n").trim().to_string();

    (migration_sql, down_sql_content)
}
