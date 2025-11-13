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

            // Check if this is an Atlas format migration
            if !contents.starts_with("-- atlas:txtar") {
                panic!("Migration file '{file_name}' must start with '-- atlas:txtar' header");
            }

            // Parse the Atlas txtar format
            let (migration_sql, down_sql) = parse_atlas_txtar(&contents, file_name);

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

/// Parse Atlas txtar format migration file
/// Format:
/// ```text
/// -- atlas:txtar
///
/// -- checks.sql --
/// SELECT ...;
///
/// -- migration.sql --
/// CREATE TABLE ...;
///
/// -- down.sql --
/// DROP TABLE ...;
/// ```
fn parse_atlas_txtar(contents: &str, filename: &str) -> (String, String) {
    let lines: Vec<&str> = contents.lines().collect();

    let mut sections = std::collections::HashMap::new();
    let mut current_section: Option<String> = None;
    let mut current_content = Vec::new();

    for line in lines.iter().skip(1) {
        // Skip the "-- atlas:txtar" header
        // Check if this is a section header (e.g., "-- migration.sql --")
        if line.starts_with("--") && line.ends_with("--") && line.contains(".sql") {
            // Save previous section if it exists
            if let Some(section_name) = current_section.take() {
                sections.insert(section_name, current_content.join("\n"));
                current_content.clear();
            }

            // Extract section name (e.g., "migration.sql" from "-- migration.sql --")
            let section_name = line
                .trim_start_matches("--")
                .trim_end_matches("--")
                .trim()
                .to_string();
            current_section = Some(section_name);
        } else if current_section.is_some() {
            // Add line to current section
            current_content.push(*line);
        }
    }

    // Save the last section
    if let Some(section_name) = current_section {
        sections.insert(section_name, current_content.join("\n"));
    }

    // Validate required sections
    let migration_sql = sections
        .get("migration.sql")
        .unwrap_or_else(|| {
            panic!("Migration file '{filename}' must contain '-- migration.sql --' section")
        })
        .trim()
        .to_string();

    let down_sql = sections
        .get("down.sql")
        .unwrap_or_else(|| {
            panic!("Migration file '{filename}' must contain '-- down.sql --' section")
        })
        .trim()
        .to_string();

    (migration_sql, down_sql)
}
