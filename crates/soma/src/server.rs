use std::future::Future;
use std::net::SocketAddr;

use shared::error::CommonError;
use shared::port::find_free_port;
use soma_api_server::ApiService;
use tower_http::cors::CorsLayer;
use tracing::info;

pub struct StartAxumServerParams {
    pub host: String,
    pub port: u16,
    pub system_shutdown_signal_rx: tokio::sync::broadcast::Receiver<()>,
    pub api_service: ApiService,
}

/// Starts the Axum server
pub async fn start_axum_server(
    params: StartAxumServerParams,
) -> Result<
    (
        impl Future<Output = Result<(), std::io::Error>>,
        axum_server::Handle,
        SocketAddr,
    ),
    CommonError,
> {
    let mut system_shutdown_signal_rx = params.system_shutdown_signal_rx;
    let port = find_free_port(params.port, params.port + 100)?;
    let addr: SocketAddr = format!("{}:{}", params.host, port)
        .parse()
        .map_err(|e| CommonError::AddrParseError { source: e })?;

    info!("Starting server on {}", addr);

    let handle = axum_server::Handle::new();

    // Build the main API router
    let api_router = soma_api_server::router::initiaite_api_router(params.api_service)?;

    // Add the frontend router (Vite dev server in debug, static files in release)
    #[cfg(debug_assertions)]
    let _vite_scope_guard = {
        use soma_frontend::start_vite_dev_server;
        start_vite_dev_server()
    };

    // Merge frontend router in both debug and release modes
    use axum::Router;
    use soma_frontend::create_vite_router;

    let (vite_router, _) = create_vite_router().split_for_parts();
    let router = Router::new().merge(api_router).merge(vite_router);

    // Add CORS layer
    let router = router.layer(CorsLayer::permissive());

    info!("Router initiated");

    let server_fut = axum_server::bind(addr)
        .handle(handle.clone())
        .serve(router.into_make_service());

    let handle_clone = handle.clone();

    tokio::spawn(async move {
        let _ = system_shutdown_signal_rx.recv().await;

        info!("Shutting down axum server, waiting for in-flight requests to complete...");

        // Initiate graceful shutdown (stops accepting new connections, waits for in-flight requests)
        handle_clone.graceful_shutdown(Some(std::time::Duration::from_secs(30)));

        #[cfg(debug_assertions)]
        {
            use soma_frontend::stop_vite_dev_server;

            drop(_vite_scope_guard);
            if let Err(e) = stop_vite_dev_server().await {
                use tracing::error;

                error!("Failed to stop vite dev server: {:?}", e);
            }
        }

        info!("Axum server shut down gracefully");
    });

    info!("Server bound");
    Ok((server_fut, handle, addr))
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
