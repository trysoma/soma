use std::future::Future;
use std::net::SocketAddr;

use http::header::HeaderName;
use shared::error::CommonError;
use shared::port::find_free_port;
use shared::process_manager::ShutdownCallback;
use soma_api_server::ApiService;
use std::pin::Pin;
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer, ExposeHeaders};

pub struct StartAxumServerParams {
    pub host: String,
    pub port: u16,
    pub api_service: ApiService,
}

pub struct StartAxumServerResult {
    pub server_fut: Pin<Box<dyn Future<Output = Result<(), std::io::Error>> + Send>>,
    pub handle: axum_server::Handle,
    pub addr: SocketAddr,
    pub on_shutdown_triggered: ShutdownCallback,
    pub on_shutdown_complete: ShutdownCallback,
}

/// Starts the Axum server
pub async fn start_axum_server(
    params: StartAxumServerParams,
) -> Result<StartAxumServerResult, CommonError> {
    let port = find_free_port(params.port, params.port + 100)?;
    let addr: SocketAddr = format!("{}:{}", params.host, port)
        .parse()
        .map_err(|e| CommonError::AddrParseError { source: e })?;

    tracing::debug!(address = %addr, "Starting server");

    let handle = axum_server::Handle::new();

    // Build the main API router
    let api_router = soma_api_server::router::initiaite_api_router(params.api_service)?;

    // Add the frontend router (Vite dev server in debug, static files in release)
    #[cfg(debug_assertions)]
    let vite_scope_guard = {
        use soma_frontend::start_vite_dev_server;
        start_vite_dev_server()
    };

    // Merge frontend router in both debug and release modes
    use axum::Router;
    use soma_frontend::create_vite_router;

    let (vite_router, _) = create_vite_router().split_for_parts();
    let router = Router::new().merge(api_router).merge(vite_router);

    // Add CORS layer with explicit MCP session header support
    // The MCP Streamable HTTP transport requires mcp-session-id to be exposed for browser clients
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::any())
        .allow_methods(AllowMethods::any())
        .allow_headers(AllowHeaders::any())
        .expose_headers(ExposeHeaders::list([
            HeaderName::from_static("mcp-session-id"),
            HeaderName::from_static("mcp-protocol-version"),
        ]));
    let router = router.layer(cors);

    tracing::trace!("Router initiated");

    let server_fut = Box::pin(axum_server::bind(addr)
        .handle(handle.clone())
        .serve(router.into_make_service()));

    let handle_for_shutdown = handle.clone();
    
    // Create on_shutdown_triggered callback
    let on_shutdown_triggered: ShutdownCallback = Box::new(move || {
        let handle = handle_for_shutdown.clone();
        Box::pin(async move {
            tracing::debug!("Shutting down server, waiting for in-flight requests");
            // Initiate graceful shutdown (stops accepting new connections, waits for in-flight requests)
            handle.graceful_shutdown(Some(std::time::Duration::from_secs(30)));
        })
    });
    
    // Create on_shutdown_complete callback
    #[cfg(debug_assertions)]
    let on_shutdown_complete: ShutdownCallback = {
        use std::sync::{Arc, Mutex};
        let vite_guard = Arc::new(Mutex::new(Some(vite_scope_guard)));
        let vite_guard_clone = vite_guard.clone();
        Box::new(move || {
            let vite_guard = vite_guard_clone.clone();
            Box::pin(async move {
                use soma_frontend::stop_vite_dev_server;
                // First call stop_vite_dev_server to gracefully stop it
                if let Err(e) = stop_vite_dev_server().await {
                    tracing::warn!(error = ?e, "Failed to stop vite dev server gracefully");
                }
                // Then drop the guard - use catch_unwind to prevent panic from crashing the process
                let guard = vite_guard.lock().unwrap().take();
                if let Some(guard) = guard {
                    if let Err(e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        drop(guard);
                    })) {
                        tracing::warn!(error = ?e, "Vite dev server guard drop panicked, process may still be running");
                    }
                }
                tracing::debug!("Server shut down");
            })
        })
    };
    
    #[cfg(not(debug_assertions))]
    let on_shutdown_complete: ShutdownCallback = Box::new(move || {
        Box::pin(async move {
            tracing::debug!("Server shut down");
        })
    });

    tracing::trace!("Server bound");
    Ok(StartAxumServerResult {
        server_fut,
        handle,
        addr,
        on_shutdown_triggered,
        on_shutdown_complete,
    })
}

#[cfg(all(test, feature = "unit_test"))]
mod unit_test {
    use shared::port::find_free_port_with_bind;

    use super::*;
    use std::{
        io::{Error, ErrorKind},
        net::TcpListener,
    };

    #[test]
    fn test_find_free_port_success() {
        // Mock bind function that succeeds on port 3002
        let bind_fn = |addr: SocketAddr| {
            if addr.port() == 3002 {
                Ok(TcpListener::bind("127.0.0.1:0").unwrap())
            } else {
                Err(Error::new(ErrorKind::AddrInUse, "Port in use"))
            }
        };

        let port = find_free_port_with_bind(3000, 3010, bind_fn).unwrap();
        assert_eq!(port, 3002);
    }

    #[test]
    fn test_find_free_port_no_ports_available() {
        // Mock bind function that always fails
        let bind_fn = |_: SocketAddr| Err(Error::new(ErrorKind::AddrInUse, "Port in use"));

        let result = find_free_port_with_bind(3000, 3010, bind_fn);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::AddrNotAvailable);
    }

    #[test]
    fn test_find_free_port_first_port_available() {
        // Mock bind function that always succeeds
        let bind_fn = |_: SocketAddr| Ok(TcpListener::bind("127.0.0.1:0").unwrap());

        let port = find_free_port_with_bind(5000, 5100, bind_fn).unwrap();
        assert_eq!(port, 5000);
    }

    #[test]
    fn test_find_free_port_last_port_available() {
        // Mock bind function that only succeeds on the last port
        let bind_fn = |addr: SocketAddr| {
            if addr.port() == 6010 {
                Ok(TcpListener::bind("127.0.0.1:0").unwrap())
            } else {
                Err(Error::new(ErrorKind::AddrInUse, "Port in use"))
            }
        };

        let port = find_free_port_with_bind(6000, 6010, bind_fn).unwrap();
        assert_eq!(port, 6010);
    }

    #[test]
    fn test_find_free_port_integration() {
        // This is an integration test that actually binds to a port
        let port = find_free_port(50000, 50100).unwrap();
        assert!((50000..=50100).contains(&port));

        // Verify we can actually bind to the port
        let listener = TcpListener::bind(format!("127.0.0.1:{port}"));
        assert!(listener.is_ok());
    }
}
