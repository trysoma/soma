use std::env;
use std::path::PathBuf;

fn main() {
    // Absolute path to this crate's dir
    let crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // Point to the app directory
    let app_dir = crate_dir.join("app");
    // Export it so the derive macro sees it
    println!("cargo:rustc-env=FRONTEND_APP_DIR={}", app_dir.display());

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
