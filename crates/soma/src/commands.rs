use std::fs;
use std::path::PathBuf;
use std::{future::Future, process::Stdio};
use std::net::SocketAddr;

use clap::Parser;
use tokio::process::Command;
use tokio_graceful_shutdown::{SubsystemBuilder, SubsystemHandle};
use tracing::{error, info};

use shared::command::run_child_process;
use crate::{router};
use shared::{error::CommonError, node::override_path_env};
use crate::vite::Assets;


#[derive(Debug, Parser)]
pub struct StartParams {
    #[arg(long, default_value = "3000")]
    pub port: u16,
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,
    #[arg(long)]
    pub src_dir: Option<PathBuf>,
}


pub async fn cmd_start(subsys: &SubsystemHandle, params: StartParams) -> Result<(), CommonError> {
    #[cfg(debug_assertions)]
    let vite_scope_guard ={
        info!("Starting vite dev server");
        Assets::start_dev_server(false)
    };

    subsys.start(SubsystemBuilder::new(
        "restate",
        move |subsys: SubsystemHandle| async move {
            tokio::select! {
                _ = subsys.on_shutdown_requested() => {
                    info!("Shutting down restate");
                },
                _ = start_restate_server() => {
                    info!("Restate server stopped");
                }
            }
            Ok::<(), CommonError>(())
        },
    ));

    subsys.start(SubsystemBuilder::new(
        "axum-server",
        move |subsys: SubsystemHandle| async move {
            let (server_fut, handle) = start_axum_server(params)?;
            tokio::select! {
                _ = subsys.on_shutdown_requested() => {
                    info!("Shutting down axum server");
                    #[cfg(debug_assertions)]
                    {
                        info!("Stopping vite dev server");
                        Assets::stop_dev_server();
                    }
                    handle.shutdown();
                    info!("Axum server shut down");
                }
                _ = server_fut => {
                    info!("Axum server started");
                    subsys.request_shutdown();
                }
            }

            Ok::<(), CommonError>(())
        },
    ));

    

    subsys.on_shutdown_requested().await;
    Ok(())
}


async fn start_restate_server() -> Result<(), CommonError> {
    run_child_process("restate-server", Command::new("restate-server")).await?;
    Ok(())
}

fn start_axum_server(params: StartParams) -> Result<(impl Future<Output = Result<(), std::io::Error>>, axum_server::Handle), CommonError> {
    let addr: SocketAddr = format!("{}:{}", params.host, params.port)
        .parse()
        .map_err(|e| CommonError::AddrParseError { source: e })?;

    info!("Starting server on {}", addr);

    let router = router::initiate_routers(&params)?;


    let handle = axum_server::Handle::new();
    let handle_clone = handle.clone();
    let server_fut = axum_server::bind(addr)
        .handle(handle)
        .serve(router.into_make_service());

    Ok((server_fut, handle_clone))
}


pub async fn cmd_codegen(_subsys: &SubsystemHandle) -> Result<(), CommonError> {
    codegen().await
}

async fn codegen()-> Result<(), CommonError> {
    info!("generating openapi spec in /openapi.json");
    let frontend_assets_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("./app");

    let rust_api_client_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("../openapi-client");

    let openapi_json_path =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let openapi_client_path = openapi_json_path.join("openapi.json");

    let spec = router::generate_openapi_spec();
    let openapi_client_json = spec.to_pretty_json().unwrap();
    fs::write(openapi_client_path.clone(), openapi_client_json).unwrap();

    info!("generating typescript client");

    let openapi_client_path_str = openapi_client_path.to_string_lossy();
    override_path_env();


    let generator_output = Command::new("npx")
        .args([
            "--yes",
            "openapi-typescript@latest",
            &openapi_client_path_str,
            "-o",
            format!("{}/src/@types/openapi.d.ts", frontend_assets_dir.display())
                .as_str(),
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
        .unwrap();

    shared::build_helpers::print_command_output(
        &generator_output.status,
        &generator_output,
    );
    if !generator_output.status.success() {
        panic!("Failed to generate openapi client");
    }

    Ok(())

}