use std::fs;
use std::path::PathBuf;
use std::process::Stdio;

use tokio::process::Command;
use tokio_graceful_shutdown::SubsystemHandle;
use tracing::{error, info};

use crate::router;
use shared::{error::CommonError, node::override_path_env};

mod start;

pub use start::{StartParams, cmd_start};

pub async fn cmd_codegen(_subsys: &SubsystemHandle) -> Result<(), CommonError> {
    codegen().await
}

async fn codegen() -> Result<(), CommonError> {
    info!("generating openapi spec in /openapi.json");
    let frontend_assets_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?).join("./app");

    let openapi_json_path =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?).join("../../openapi.json");

    let spec = router::generate_openapi_spec();
    let openapi_client_json = spec.to_pretty_json()?;
    fs::write(openapi_json_path.clone(), openapi_client_json)?;

    info!("generating typescript client");

    let openapi_client_path_str = openapi_json_path.to_string_lossy();
    override_path_env();

    let generator_output = Command::new("npx")
        .args([
            "--yes",
            "openapi-typescript@latest",
            &openapi_client_path_str,
            "-o",
            format!("{}/src/@types/openapi.d.ts", frontend_assets_dir.display()).as_str(),
        ])
        .current_dir(frontend_assets_dir.clone())
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stdin(Stdio::null())
        .output()
        .await
        .inspect_err(|e| {
            error!("error: {:?}", e);
        })
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!(e)))?;

    shared::build_helpers::print_command_output(&generator_output.status, &generator_output);
    if !generator_output.status.success() {
        panic!("Failed to generate openapi client");
    }

    Ok(())
}
