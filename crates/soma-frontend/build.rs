use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Absolute path to this crate's dir
    let crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // Point to the app directory
    let app_dir = crate_dir.join("app");
    let workspace_dir = crate_dir.parent().unwrap().parent().unwrap();

    // Export it so the derive macro sees it
    println!("cargo:rustc-env=FRONTEND_APP_DIR={}", app_dir.display());

    // Step 1: Generate OpenAPI spec
    println!("cargo:warning=Generating openapi spec in /openapi.json");

    let openapi_json_path = workspace_dir.join("openapi.json");
    let typescript_client_path = app_dir.join("src/@types/openapi.d.ts");

    if cfg!(debug_assertions) || (openapi_json_path.exists() && typescript_client_path.exists()) {
        println!(
            "cargo:warning=Debug build or frontend already built, skipping openapi spec generation and client generation"
        );
    } else {
        let spec = soma_api_server::router::generate_openapi_spec();
        let openapi_client_json =
            serde_json::to_string_pretty(&spec).expect("Failed to serialize OpenAPI spec");
        fs::write(&openapi_json_path, openapi_client_json).expect("Failed to write openapi.json");

        // Step 2: Generate TypeScript client
        println!("cargo:warning=Generating typescript client");

        let openapi_client_path_str = openapi_json_path.to_string_lossy();

        let generator_output = Command::new("npx")
            .args([
                "--yes",
                "openapi-typescript@latest",
                &openapi_client_path_str,
                "-o",
                typescript_client_path.to_str().unwrap(),
            ])
            .current_dir(&app_dir)
            .output();

        match generator_output {
            Ok(output) => {
                if !output.status.success() {
                    println!(
                        "cargo:warning=npx openapi-typescript failed with exit code: {:?}",
                        output.status.code()
                    );
                    if !output.stdout.is_empty() {
                        println!(
                            "cargo:warning=stdout: {}",
                            String::from_utf8_lossy(&output.stdout)
                        );
                    }
                    if !output.stderr.is_empty() {
                        println!(
                            "cargo:warning=stderr: {}",
                            String::from_utf8_lossy(&output.stderr)
                        );
                    }
                }
            }
            Err(e) => {
                println!("cargo:warning=Could not run npx openapi-typescript: {e}");
            }
        }
    }

    // Step 3: Check if frontend is already built (routes.json exists)
    let routes_json = app_dir.join("dist/.vite-rs/routes.json");

    // if debug build, skip pnpm
    if cfg!(debug_assertions) || routes_json.exists() {
        println!(
            "cargo:warning=Debug build or frontend already built, skipping pnpm install, build"
        );
    } else {
        // Step 4: Install pnpm dependencies
        let install_result = Command::new("pnpm")
            .arg("install")
            .current_dir(workspace_dir)
            .output();

        match install_result {
            Ok(output) => {
                if !output.status.success() {
                    println!(
                        "cargo:warning=pnpm install failed with exit code: {:?}",
                        output.status.code()
                    );
                    if !output.stdout.is_empty() {
                        println!(
                            "cargo:warning=stdout: {}",
                            String::from_utf8_lossy(&output.stdout)
                        );
                    }
                    if !output.stderr.is_empty() {
                        println!(
                            "cargo:warning=stderr: {}",
                            String::from_utf8_lossy(&output.stderr)
                        );
                    }
                }
            }
            Err(e) => {
                println!("cargo:warning=Could not run pnpm install: {e}");
            }
        }

        // Step 5: Build frontend
        let build_result = Command::new("pnpm")
            .arg("run")
            .arg("build")
            .current_dir(&app_dir)
            .output();

        match build_result {
            Ok(output) => {
                if !output.status.success() {
                    println!(
                        "cargo:warning=pnpm build failed with exit code: {:?}",
                        output.status.code()
                    );
                    if !output.stdout.is_empty() {
                        println!(
                            "cargo:warning=stdout: {}",
                            String::from_utf8_lossy(&output.stdout)
                        );
                    }
                    if !output.stderr.is_empty() {
                        println!(
                            "cargo:warning=stderr: {}",
                            String::from_utf8_lossy(&output.stderr)
                        );
                    }
                }
            }
            Err(e) => {
                println!("cargo:warning=Could not run pnpm build: {e}");
            }
        }
    }

    // Step 6: fix linting errors
    let lint_fix_result = Command::new("pnpm")
        .arg("run")
        .arg("lint:fix")
        .current_dir(&app_dir)
        .output();

    match lint_fix_result {
        Ok(output) => {
            if !output.status.success() {
                println!(
                    "cargo:warning=pnpm lint:fix failed with exit code: {:?}",
                    output.status.code()
                );
                if !output.stdout.is_empty() {
                    println!(
                        "cargo:warning=stdout: {}",
                        String::from_utf8_lossy(&output.stdout)
                    );
                }
                if !output.stderr.is_empty() {
                    println!(
                        "cargo:warning=stderr: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
            }
        }
        Err(e) => {
            println!("cargo:warning=Could not run pnpm lint:fix: {e}");
        }
    }
}
