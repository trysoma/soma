use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

const RESTATE_VERSION: &str = "v1.5.3";

fn main() {
    // Absolute path to this crate's dir
    let crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // Point to the app directory
    let app_dir = crate_dir.join("app");
    // Export it so the derive macro sees it
    println!("cargo:rustc-env=FRONTEND_APP_DIR={}", app_dir.display());

    // Download restate-server binary for the current target
    download_restate_binary(&crate_dir);

    // if debug build, skip pnpm steps
    if cfg!(debug_assertions) {
        println!("cargo:warning=Debug build, skipping pnpm steps");
        return;
    }

    // Skip npm/pnpm commands in Nix builds (no network access)
    // Only run npm commands in non-Nix environments
    let install_result = std::process::Command::new("pnpm")
        .arg("install")
        .current_dir(app_dir.clone())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status(); // Use status() instead of spawn() to wait for completion

    if let Ok(status) = install_result {
        if !status.success() {
            println!("cargo:warning=Warning: pnpm install failed");
        }
    } else {
        println!("cargo:warning=Warning: Could not run pnpm install");
    }

    let build_result = std::process::Command::new("pnpm")
        .arg("run")
        .arg("build")
        .current_dir(app_dir.clone())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status(); // Use status() instead of spawn() to wait for completion

    if let Ok(status) = build_result {
        if !status.success() {
            println!("cargo:warning=Warning: pnpm build failed");
        }
    } else {
        println!("cargo:warning=Warning: Could not run pnpm build");
    }
}

fn download_restate_binary(crate_dir: &Path) {
    let target = env::var("TARGET").unwrap();

    // Map Rust target triples to restate binary names
    let restate_target = match target.as_str() {
        "x86_64-unknown-linux-gnu" | "x86_64-unknown-linux-musl" => "x86_64-unknown-linux-musl",
        "aarch64-unknown-linux-gnu" | "aarch64-unknown-linux-musl" => "aarch64-unknown-linux-musl",
        "x86_64-apple-darwin" => "x86_64-apple-darwin",
        "aarch64-apple-darwin" => "aarch64-apple-darwin",
        _ => {
            println!(
                "cargo:warning=Unsupported target: {target}, skipping restate-server download"
            );
            return;
        }
    };

    let binary_name = format!("restate-server-{restate_target}.tar.xz");
    let download_url = format!(
        "https://github.com/restatedev/restate/releases/download/{RESTATE_VERSION}/{binary_name}"
    );

    let bin_dir = crate_dir.join("bin");
    let target_bin_dir = bin_dir.join(&target);

    // Create bin directory if it doesn't exist
    fs::create_dir_all(&target_bin_dir).unwrap();

    let archive_path = target_bin_dir.join(&binary_name);
    let extracted_binary = target_bin_dir.join("restate-server");

    // Check if binary already exists
    if extracted_binary.exists() {
        println!("cargo:warning=restate-server binary already exists at {extracted_binary:?}");
        println!(
            "cargo:rustc-env=RESTATE_BINARY_PATH={}",
            extracted_binary.display()
        );
        return;
    }

    println!("cargo:warning=Downloading restate-server from {download_url}");

    // Download the archive
    let output = std::process::Command::new("curl")
        .arg("-L")
        .arg("-o")
        .arg(&archive_path)
        .arg(&download_url)
        .output();

    match output {
        Ok(output) if output.status.success() => {
            println!("cargo:warning=Successfully downloaded restate-server archive");
        }
        Ok(output) => {
            println!(
                "cargo:warning=Failed to download restate-server: {:?}",
                String::from_utf8_lossy(&output.stderr)
            );
            return;
        }
        Err(e) => {
            println!("cargo:warning=Failed to execute curl: {e}");
            return;
        }
    }

    // Create a temporary directory for extraction
    let temp_extract_dir = target_bin_dir.join("temp_extract");
    let _ = fs::remove_dir_all(&temp_extract_dir); // Clean up any old temp directory
    fs::create_dir_all(&temp_extract_dir).unwrap();

    // Extract the archive using tar to temp directory
    let extract_output = std::process::Command::new("tar")
        .arg("-xf")
        .arg(&archive_path)
        .arg("-C")
        .arg(&temp_extract_dir)
        .output();

    match extract_output {
        Ok(output) if output.status.success() => {
            println!("cargo:warning=Successfully extracted restate-server archive");
            // Remove the archive file
            let _ = fs::remove_file(&archive_path);

            // The tar archive contains a subdirectory like restate-server-x86_64-unknown-linux-musl/
            // We need to find the binary inside and move it to the target_bin_dir
            let archive_prefix = format!("restate-server-{restate_target}");
            let extracted_subdir = temp_extract_dir.join(&archive_prefix);
            let binary_in_subdir = extracted_subdir.join("restate-server");

            if binary_in_subdir.exists() {
                // Move the binary to the target directory
                if let Err(e) = fs::copy(&binary_in_subdir, &extracted_binary) {
                    println!("cargo:warning=Failed to copy binary: {e:?}");
                    let _ = fs::remove_dir_all(&temp_extract_dir);
                    return;
                }
                // Clean up temp directory
                let _ = fs::remove_dir_all(&temp_extract_dir);
            } else {
                println!(
                    "cargo:warning=Binary not found in expected location: {binary_in_subdir:?}"
                );
                let _ = fs::remove_dir_all(&temp_extract_dir);
                return;
            }

            // Make binary executable (Unix only)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(metadata) = fs::metadata(&extracted_binary) {
                    let mut perms = metadata.permissions();
                    perms.set_mode(0o755);
                    let _ = fs::set_permissions(&extracted_binary, perms);
                }
            }

            println!("cargo:warning=Successfully installed restate-server binary");
            println!(
                "cargo:rustc-env=RESTATE_BINARY_PATH={}",
                extracted_binary.display()
            );
        }
        Ok(output) => {
            println!(
                "cargo:warning=Failed to extract restate-server: {:?}",
                String::from_utf8_lossy(&output.stderr)
            );
            let _ = fs::remove_dir_all(&temp_extract_dir);
        }
        Err(e) => {
            println!("cargo:warning=Failed to execute tar: {e}");
            let _ = fs::remove_dir_all(&temp_extract_dir);
        }
    }
}
