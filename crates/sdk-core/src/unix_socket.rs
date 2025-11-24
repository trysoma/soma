// Platform-specific Unix domain socket implementation
// On Unix: uses tokio::net::UnixListener/UnixStream
// On Windows: uses uds_windows::UnixListener/UnixStream

use anyhow::Result;
use std::path::PathBuf;

#[cfg(unix)]
mod unix_impl {
    use super::*;
    use tokio::net::UnixListener as TokioUnixListener;
    use tokio_stream::wrappers::UnixListenerStream as TokioUnixListenerStream;

    pub type UnixListener = TokioUnixListener;
    pub type UnixListenerStream = TokioUnixListenerStream;

    pub async fn bind_unix_listener(path: &PathBuf) -> Result<UnixListener> {
        // Remove stale socket file if it exists
        // This handles cases where the process crashed without cleanup
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(TokioUnixListener::bind(path)?)
    }

    pub fn create_listener_stream(listener: UnixListener) -> UnixListenerStream {
        TokioUnixListenerStream::new(listener)
    }
}

#[cfg(windows)]
mod windows_impl {
    use super::*;
    use async_stream::stream;
    use futures::Stream;
    use std::pin::Pin;
    use std::sync::Arc;
    use std::task::{Context, Poll};
    use tokio::io::{AsyncRead, AsyncWrite};
    use tokio_util::io::SyncIoBridge;
    use tonic::transport::server::Connected;
    use uds_windows::{UnixListener as UdsUnixListener, UnixStream as UdsUnixStream};

    // Wrapper to make uds_windows::UnixStream work with tokio and implement Connected
    // SyncIoBridge already implements AsyncRead and AsyncWrite, so we just wrap it
    pub struct TokioUnixStream {
        inner: SyncIoBridge<UdsUnixStream>,
    }

    impl TokioUnixStream {
        pub fn new(stream: UdsUnixStream) -> Self {
            Self {
                inner: SyncIoBridge::new(stream),
            }
        }
    }

    // Delegate AsyncRead to SyncIoBridge (which already implements it)
    impl AsyncRead for TokioUnixStream {
        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
        ) -> Poll<std::io::Result<()>> {
            // SyncIoBridge implements AsyncRead, so we can use it directly
            AsyncRead::poll_read(Pin::new(&mut self.get_mut().inner), cx, buf)
        }
    }

    // Delegate AsyncWrite to SyncIoBridge (which already implements it)
    impl AsyncWrite for TokioUnixStream {
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<std::io::Result<usize>> {
            AsyncWrite::poll_write(Pin::new(&mut self.get_mut().inner), cx, buf)
        }

        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            AsyncWrite::poll_flush(Pin::new(&mut self.get_mut().inner), cx)
        }

        fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            AsyncWrite::poll_shutdown(Pin::new(&mut self.get_mut().inner), cx)
        }
    }

    impl Connected for TokioUnixStream {
        type ConnectInfo = ();

        fn connect_info(&self) -> Self::ConnectInfo {
            ()
        }
    }

    // SyncIoBridge is Unpin if the inner type is Unpin
    // UdsUnixStream should be Unpin, so this should be safe
    impl Unpin for TokioUnixStream {}

    // Wrapper for UnixListener
    pub struct UnixListener {
        inner: Arc<UdsUnixListener>,
    }

    impl UnixListener {
        pub async fn bind(path: &PathBuf) -> Result<Self> {
            // Convert PathBuf to String to own it for the closure
            let path_str = path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid socket path"))?
                .to_string();
            // Use spawn_blocking since bind might block
            let listener =
                tokio::task::spawn_blocking(move || UdsUnixListener::bind(&path_str)).await??;
            Ok(Self {
                inner: Arc::new(listener),
            })
        }
    }

    // Stream adapter for UnixListener that works with tokio
    pub struct UnixListenerStream {
        stream: std::pin::Pin<Box<dyn Stream<Item = std::io::Result<TokioUnixStream>> + Send>>,
    }

    impl UnixListenerStream {
        pub fn new(listener: UnixListener) -> Self {
            let listener_inner = listener.inner.clone();
            let stream = stream! {
                loop {
                    let listener = listener_inner.clone();
                    let result = tokio::task::spawn_blocking(move || {
                        listener.accept()
                    }).await;

                    match result {
                        Ok(Ok((stream, _addr))) => {
                            yield Ok(TokioUnixStream::new(stream));
                        }
                        Ok(Err(e)) => {
                            yield Err(e);
                            // Continue accepting connections (same behavior as Unix)
                        }
                        Err(e) => {
                            yield Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Task join error: {}", e),
                            ));
                            break;
                        }
                    }
                }
            };
            Self {
                stream: Box::pin(stream),
            }
        }
    }

    impl Stream for UnixListenerStream {
        type Item = std::io::Result<TokioUnixStream>;

        fn poll_next(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<Self::Item>> {
            Stream::poll_next(self.stream.as_mut(), cx)
        }
    }

    pub async fn bind_unix_listener(path: &PathBuf) -> Result<UnixListener> {
        // Remove stale socket file if it exists
        // This handles cases where the process crashed without cleanup
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        UnixListener::bind(path).await
    }

    pub fn create_listener_stream(listener: UnixListener) -> UnixListenerStream {
        UnixListenerStream::new(listener)
    }
}

#[cfg(unix)]
pub use unix_impl::*;

#[cfg(windows)]
pub use windows_impl::*;
