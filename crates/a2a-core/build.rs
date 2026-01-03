use std::{env, fs, path::Path};

use typify::{TypeSpace, TypeSpaceSettings};
// A2A specification files pinned to specific commit for reproducibility
const A2A_COMMIT_HASH: &str = "d38f19b35d792bf576ba03465f1171d07a2c7bfc";

fn main() {
    println!("cargo:rerun-if-changed=proto/a2a.proto");
    println!("cargo:rerun-if-changed=types/a2a.json");
    println!("cargo:rerun-if-changed=build.rs");

    let proto_file = Path::new("proto/a2a.proto").to_path_buf();
    let _a2a_type_file = Path::new("types/a2a.json").to_path_buf();
    let out_dir_str = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir_str);
    // Use target-specific cache file to avoid cross-target cache issues
    let a2a_commit_hash_file = out_dir.join("a2a_commit_hash.txt");
    let a2a_type_file = out_dir.join("a2a_types.rs");
    let proto_service_bin_file = out_dir.join("service.bin");

    if a2a_commit_hash_file.exists() && proto_service_bin_file.exists() && a2a_type_file.exists() {
        if let Ok(cached_hash) = fs::read_to_string(&a2a_commit_hash_file) {
            if cached_hash.trim() == A2A_COMMIT_HASH {
                return;
            }
        }
    }

    // Ensure directories exist
    if let Some(proto_dir) = proto_file.parent() {
        fs::create_dir_all(proto_dir).unwrap();
    }
    // Ensure OUT_DIR exists (it should, but be safe)
    fs::create_dir_all(out_dir).unwrap();

    // Check if we have pre-fetched files from Nix (or download them)
    if let (Ok(json_path), Ok(proto_path)) = (
        env::var("A2A_JSON_SCHEMA_PATH"),
        env::var("A2A_PROTO_SPEC_PATH"),
    ) {
        // Use pre-fetched files from Nix
        let json_content = fs::read_to_string(&json_path).unwrap();
        let proto_content = fs::read_to_string(&proto_path).unwrap();

        fs::write(&a2a_type_file, json_content).unwrap();
        fs::write(&proto_file, proto_content).unwrap();
    } else {
        // Check if proto file already exists in the repo (for CI builds)
        if !proto_file.exists() {
            // Download files (for local development)
            // URLs are pinned to specific commit hash for reproducibility
            let json_url = format!(
                "https://raw.githubusercontent.com/a2aproject/A2A/{A2A_COMMIT_HASH}/specification/json/a2a.json"
            );
            let response = reqwest::blocking::get(&json_url).unwrap();
            let json_bytes = response.bytes().unwrap();

            // Parse as schema to ensure it's valid JSON
            let schema = serde_json::from_slice::<schemars::Schema>(&json_bytes).unwrap();
            fs::write(&a2a_type_file, serde_json::to_string(&schema).unwrap()).unwrap();

            let proto_url = format!(
                "https://raw.githubusercontent.com/a2aproject/A2A/{A2A_COMMIT_HASH}/specification/grpc/a2a.proto"
            );
            let proto_response = reqwest::blocking::get(&proto_url).unwrap();
            let proto_bytes = proto_response.bytes().unwrap();

            fs::write(&proto_file, proto_bytes).unwrap();
        } else {
            // Still need to download/generate the JSON schema file for types
            let json_url = format!(
                "https://raw.githubusercontent.com/a2aproject/A2A/{A2A_COMMIT_HASH}/specification/json/a2a.json"
            );
            let response = reqwest::blocking::get(&json_url).unwrap();
            let json_bytes = response.bytes().unwrap();

            // Parse as schema to ensure it's valid JSON
            let schema = serde_json::from_slice::<schemars::Schema>(&json_bytes).unwrap();
            fs::write(&a2a_type_file, serde_json::to_string(&schema).unwrap()).unwrap();
        }
    }

    // build proto
    let mut config = prost_build::Config::new();
    config.protoc_arg("--experimental_allow_proto3_optional");
    let proto_files = [proto_file.clone()];
    tonic_build::configure()
        .btree_map(["."])
        .use_arc_self(true)
        .file_descriptor_set_path(proto_service_bin_file)
        .compile_protos_with_config(
            config,
            &proto_files,
            &["proto", "proto/thirdparty/googleapis"],
        )
        .unwrap();

    // generate types
    let mut type_space = TypeSpace::new(TypeSpaceSettings::default().with_struct_builder(true));
    type_space
        .add_root_schema(
            serde_json::from_str(&fs::read_to_string(a2a_type_file.clone()).unwrap()).unwrap(),
        )
        .unwrap();

    let contents =
        prettyplease::unparse(&syn::parse2::<syn::File>(type_space.to_stream()).unwrap());

    fs::write(a2a_type_file, contents).unwrap();
    fs::write(&a2a_commit_hash_file, A2A_COMMIT_HASH).unwrap();
}
