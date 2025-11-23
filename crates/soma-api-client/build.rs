use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::fs;

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
            "--additional-properties=packageName=soma-api-client,packageVersion=0.0.1",
        ])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap_or_else(|e| {
            panic!("Failed to execute npx: {}. Make sure Node.js and npm are installed.", e);
        });
    
    if !output.status.success() {
        eprintln!("OpenAPI Generator stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("OpenAPI Generator failed with exit code: {:?}", output.status.code());
    }
    
    eprintln!("Rust client generated successfully!");
    
    // Verify that the generated lib.rs exists
    let generated_lib = output_dir.join("src").join("lib.rs");
    if !generated_lib.exists() {
        panic!("Generated lib.rs not found at: {}", generated_lib.display());
    }
    
    // Copy generated src files to our src/generated directory
    // This allows mod declarations in the generated lib.rs to resolve correctly
    let generated_src = output_dir.join("src");
    let crate_root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let target_src = crate_root.join("src").join("generated");
    
    // Remove old generated src if it exists
    if target_src.exists() {
        fs::remove_dir_all(&target_src).unwrap_or_else(|e| {
            eprintln!("Warning: Failed to remove old generated src: {}", e);
        });
    }
    
    // Copy the entire src directory
    copy_dir_all(&generated_src, &target_src).unwrap_or_else(|e| {
        panic!("Failed to copy generated src files: {}", e);
    });
    
    eprintln!("Generated source files copied to: {}", target_src.display());
    
    
    // Tell Cargo to rerun this build script if the OpenAPI spec changes
    println!("cargo:rerun-if-changed=build.rs");
}

/// Recursively copy a directory
fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap();
        let dst_path = dst.join(file_name);
        
        if path.is_dir() {
            copy_dir_all(&path, &dst_path)?;
        } else {
            fs::copy(&path, &dst_path)?;
        }
    }
    Ok(())
}