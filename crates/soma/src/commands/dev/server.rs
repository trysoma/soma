use std::future::Future;
use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;

use tracing::info;

use shared::error::CommonError;

use crate::router;
use crate::vite::{Assets, wait_for_vite_dev_server_shutdown};


/// Finds a free port in the given range
pub fn find_free_port(start: u16, end: u16) -> std::io::Result<u16> {
    find_free_port_with_bind(start, end, TcpListener::bind)
}

/// Internal implementation that accepts a custom bind function for testing
fn find_free_port_with_bind<F>(start: u16, end: u16, bind_fn: F) -> std::io::Result<u16>
where
    F: Fn(SocketAddr) -> std::io::Result<TcpListener>,
{
    for port in start..=end {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        if bind_fn(addr).is_ok() {
            return Ok(port);
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::AddrNotAvailable,
        "No free ports found",
    ))
}

pub struct StartAxumServerParams {
    pub project_dir: PathBuf,
    pub host: String,
    pub port: u16,
    pub routers: router::Routers,
    pub system_shutdown_signal_rx: tokio::sync::broadcast::Receiver<()>,

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

    
    let router = router::initiate_routers(params.routers)?;
    info!("Router initiated");
    let handle = axum_server::Handle::new();

    #[cfg(debug_assertions)]
    use crate::commands::dev::server::{start_vite_dev_server, stop_vite_dev_server};
    #[cfg(debug_assertions)]
    let _vite_scope_guard = start_vite_dev_server();
    
    let server_fut = axum_server::bind(addr)
        .handle(handle.clone())
        .serve(router.into_make_service());

    let handle_clone = handle.clone();

    tokio::spawn(async move {
        let _ = system_shutdown_signal_rx.recv().await;

        info!("Shutting down axum server");
        #[cfg(debug_assertions)]
        {
            drop(_vite_scope_guard);
            if let Err(e) = stop_vite_dev_server().await {
                use tracing::error;

                error!("Failed to stop vite dev server: {:?}", e);
            }
        }
        handle_clone.shutdown();
        info!("Axum server shut down");
    });
    
    info!("Server bound");
    Ok((server_fut, handle, addr))
}

/// Starts the Vite dev server (debug builds only)
/// Returns a guard that stops the server when dropped
#[cfg(debug_assertions)]
pub fn start_vite_dev_server() -> impl Drop {
    use crate::vite::Assets;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error, ErrorKind};

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
        let bind_fn = |_: SocketAddr| {
            Err(Error::new(ErrorKind::AddrInUse, "Port in use"))
        };

        let result = find_free_port_with_bind(3000, 3010, bind_fn);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::AddrNotAvailable);
    }

    #[test]
    fn test_find_free_port_first_port_available() {
        // Mock bind function that always succeeds
        let bind_fn = |_: SocketAddr| {
            Ok(TcpListener::bind("127.0.0.1:0").unwrap())
        };

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
        assert!(port >= 50000 && port <= 50100);

        // Verify we can actually bind to the port
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port));
        assert!(listener.is_ok());
    }
}
