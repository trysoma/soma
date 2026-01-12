use std::{env, fs, path::Path};

use typify::{TypeSpace, TypeSpaceSettings};

/// A2A specification files pinned to specific commit for reproducibility
const A2A_COMMIT_HASH: &str = "d38f19b35d792bf576ba03465f1171d07a2c7bfc";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir_str = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir_str);

    // Use target-specific cache file to avoid cross-target cache issues
    let a2a_commit_hash_file = out_dir.join("a2a_commit_hash.txt");
    let a2a_type_file = out_dir.join("a2a_types.rs");

    // Check if we already have the generated types for this commit
    if a2a_commit_hash_file.exists() && a2a_type_file.exists() {
        if let Ok(cached_hash) = fs::read_to_string(&a2a_commit_hash_file) {
            if cached_hash.trim() == A2A_COMMIT_HASH {
                return;
            }
        }
    }

    // Ensure OUT_DIR exists
    fs::create_dir_all(out_dir).unwrap();

    // Fetch the JSON schema (or use pre-fetched from Nix)
    let json_content = if let Ok(json_path) = env::var("A2A_JSON_SCHEMA_PATH") {
        fs::read_to_string(&json_path).unwrap()
    } else {
        // Download the JSON schema for local development
        let json_url = format!(
            "https://raw.githubusercontent.com/a2aproject/A2A/{A2A_COMMIT_HASH}/specification/json/a2a.json"
        );
        let response = reqwest::blocking::get(&json_url).unwrap();
        let json_bytes = response.bytes().unwrap();

        // Parse as schema to ensure it's valid JSON
        let schema = serde_json::from_slice::<schemars::Schema>(&json_bytes).unwrap();
        serde_json::to_string(&schema).unwrap()
    };

    // Write the JSON schema to the output directory first
    fs::write(&a2a_type_file, &json_content).unwrap();

    // Generate types from JSON schema using typify
    let mut type_space = TypeSpace::new(TypeSpaceSettings::default().with_struct_builder(true));
    type_space
        .add_root_schema(serde_json::from_str(&json_content).unwrap())
        .unwrap();

    let contents =
        prettyplease::unparse(&syn::parse2::<syn::File>(type_space.to_stream()).unwrap());

    fs::write(&a2a_type_file, contents).unwrap();
    fs::write(&a2a_commit_hash_file, A2A_COMMIT_HASH).unwrap();
}
