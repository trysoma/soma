// Platform-specific UnixStream implementation for client connections
// On Unix: uses tokio::net::UnixStream
// On Windows: uses uds_windows::UnixStream wrapped with SyncIoBridge

#[cfg(unix)]
mod unix_impl {

    use tokio::net::UnixStream as TokioUnixStream;

    pub type UnixStream = TokioUnixStream;

    pub async fn connect_unix_stream(path: &str) -> std::io::Result<UnixStream> {
        TokioUnixStream::connect(path).await
    }
}

#[cfg(windows)]
mod windows_impl {
    use tokio_util::io::SyncIoBridge;
    use uds_windows::UnixStream as UdsUnixStream;

    pub type UnixStream = SyncIoBridge<UdsUnixStream>;

    pub async fn connect_unix_stream(path: &str) -> std::io::Result<UnixStream> {
        let stream = tokio::task::spawn_blocking(move || UdsUnixStream::connect(path)).await??;
        Ok(SyncIoBridge::new(stream))
    }
}

#[cfg(unix)]
pub use unix_impl::*;

#[cfg(windows)]
pub use windows_impl::*;
