use std::net::{SocketAddr, TcpListener};

use crate::error::CommonError;

/// Finds a free port in the given range
pub fn find_free_port(start: u16, end: u16) -> std::io::Result<u16> {
    find_free_port_with_bind(start, end, TcpListener::bind)
}

/// Internal implementation that accepts a custom bind function for testing
pub fn find_free_port_with_bind<F>(start: u16, end: u16, bind_fn: F) -> std::io::Result<u16>
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

pub fn is_port_in_use(port: u16) -> Result<bool, CommonError> {
    match TcpListener::bind(("127.0.0.1", port)) {
        Ok(listener) => {
            drop(listener);
            Ok(false)
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::AddrInUse {
                Ok(true)
            } else {
                Err(CommonError::Unknown(anyhow::anyhow!(
                    "Failed to check if port is in use: {e:?}"
                )))
            }
        }
    }
}
