use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    response::Response,
    routing::any,
};
use std::time::Duration;
use utoipa_axum::router::OpenApiRouter;
use vite_rs_axum_0_8::ViteServe;

use shared::error::CommonError;
#[cfg(debug_assertions)]
use tracing::info;

// The vite_rs::Embed proc macro embeds frontend assets at compile time
// The path is relative to CARGO_MANIFEST_DIR
#[derive(vite_rs::Embed)]
#[root = "app"]
#[dev_server_port = 21012]
pub struct Assets;

async fn ping_vite_dev_server() -> Result<(), CommonError> {
    let client = reqwest::Client::new();
    let response = client.get("http://localhost:21012").send().await?;
    if response.status().is_success() {
        Ok(())
    } else {
        Err(CommonError::Unknown(anyhow::anyhow!(
            "Failed to ping vite dev server"
        )))
    }
}

pub async fn wait_for_vite_dev_server_shutdown() -> Result<(), CommonError> {
    let mut attempts = 0;
    let max_attempts = 10;
    loop {
        if ping_vite_dev_server().await.is_err() {
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
        attempts += 1;
        if attempts >= max_attempts {
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "Failed to wait for vite dev server to shutdown"
            )));
        }
    }
    Ok(())
}

/// Starts the Vite dev server (debug builds only)
/// Returns a guard that stops the server when dropped
#[cfg(debug_assertions)]
pub fn start_vite_dev_server() -> impl Drop {
    info!("Starting vite dev server");
    // The return value is a scope guard that stops the server when dropped
    let guard = Assets::start_dev_server(false);
    guard.unwrap_or_else(|| {
        panic!("Failed to start vite dev server");
    })
}

/// Stops the Vite dev server and waits for shutdown (debug builds only)
#[cfg(debug_assertions)]
pub async fn stop_vite_dev_server() -> Result<(), CommonError> {
    info!("Stopping vite dev server");
    Assets::stop_dev_server();
    wait_for_vite_dev_server_shutdown().await?;
    Ok(())
}

#[cfg(debug_assertions)]
pub fn create_vite_router() -> OpenApiRouter<()> {
    use vite_rs_axum_0_8::ViteServe;

    let vite = ViteServe::new(Assets::boxed());

    OpenApiRouter::new()
            // "/" handled explicitly
            .route(
                "/",
                any(
                    |State(vite): State<ViteServe>, req: Request<Body>| async move {
                        vite.serve(req).await
                    },
                ),
            )
            // all other paths handled by SPA fallback
            .route("/{*path}", any(tanstack_spa_handler))
            .with_state(vite)
}

#[cfg(not(debug_assertions))]
const ROUTES_JSON: &[u8] = include_bytes!(concat!(
    env!("FRONTEND_APP_DIR"),
    "/dist/.vite-rs/routes.json"
));

#[cfg(not(debug_assertions))]
pub fn create_vite_router() -> OpenApiRouter<()> {
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;

    let vite = ViteServe::new(Assets::boxed());

    #[derive(Debug, Deserialize, Serialize, ToSchema)]
    struct RouteFile {
        paths: Vec<String>,
        assets: Vec<String>,
    }

    let routes = serde_json::from_slice::<RouteFile>(ROUTES_JSON).unwrap();

    let mut vite_router = OpenApiRouter::new()
        .route(
            "/",
            any(|State(vite): State<ViteServe>, req: Request<Body>| async move {
                vite.serve(req).await
            }),
        );

    let mut all_paths = vec![];

    all_paths.extend(routes.paths);
    all_paths.extend(routes.assets.iter().map(|asset| format!("/{}", asset)));

    for path in all_paths {
        if path == "/" {
            continue;
        }
        vite_router = vite_router.route(path.as_str(), any(tanstack_spa_handler));
    }

    // Add catch-all route for SPA routing (serves index.html for unmatched routes)
    vite_router = vite_router.route("/{*path}", any(tanstack_spa_handler));

    let vite_router = vite_router.with_state(vite);

    vite_router
}

async fn tanstack_spa_handler(State(vite): State<ViteServe>, req: Request<Body>) -> Response {
    let resp = vite.serve(req).await;

    if resp.status() == StatusCode::NOT_FOUND {
        if let Some(index_file) = vite.assets.get("index.html") {
            return Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", index_file.content_type)
                .body(Body::from(index_file.bytes))
                .unwrap();
        }
    }

    resp
}
