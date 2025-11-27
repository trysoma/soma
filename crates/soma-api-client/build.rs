use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn main() {
    let spec = soma_api_server::router::generate_openapi_spec();

    // Get output directory for build artifacts
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    // Write OpenAPI spec to a temporary file
    let openapi_spec_path = out_dir.join("openapi.json");
    let json_str = serde_json::to_string_pretty(&spec).unwrap();
    fs::write(&openapi_spec_path, json_str).unwrap();

    // Generate Rust client using OpenAPI Generator
    let output_dir = out_dir.join("generated");

    eprintln!("Generating Rust client from OpenAPI spec...");
    eprintln!("OpenAPI spec: {}", openapi_spec_path.display());
    eprintln!("Output directory: {}", output_dir.display());

    let output = Command::new("npx")
        .args([
            "--yes",
            "@openapitools/openapi-generator-cli@latest",
            "generate",
            "-i",
            openapi_spec_path.to_str().unwrap(),
            "-g",
            "rust",
            "-o",
            output_dir.to_str().unwrap(),
            "--additional-properties=packageName=soma-api-client,packageVersion=0.0.1,avoidBoxedModels=true,bestFitInt=true,topLevelApiClient=true,useSerdePathToError=true",
        ])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap_or_else(|e| {
            panic!("Failed to execute npx: {e}. Make sure Node.js and npm are installed.");
        });

    if !output.status.success() {
        eprintln!(
            "OpenAPI Generator stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        panic!(
            "OpenAPI Generator failed with exit code: {:?}",
            output.status.code()
        );
    }

    eprintln!("Rust client generated successfully!");

    // Verify that the generated lib.rs exists
    let generated_lib = output_dir.join("src").join("lib.rs");
    if !generated_lib.exists() {
        panic!("Generated lib.rs not found at: {}", generated_lib.display());
    }

    // Strip inner attributes from generated lib.rs so it can be used with include!()
    // Inner attributes (#![...]) can only appear at the crate root, not in included files
    let lib_content = fs::read_to_string(&generated_lib).unwrap();
    let filtered_content: String = lib_content
        .lines()
        .filter(|line| !line.trim_start().starts_with("#!["))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&generated_lib, filtered_content).unwrap();

    eprintln!("Generated files available at: {}", output_dir.display());

    // Tell Cargo to rerun this build script if the OpenAPI spec changes
    println!("cargo:rerun-if-changed=build.rs");
}
