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
    use std::sync::Arc;
    use tokio_util::io::SyncIoBridge;
    use uds_windows::{UnixListener as UdsUnixListener, UnixStream as UdsUnixStream};

    // Wrapper to make uds_windows::UnixStream work with tokio
    // We use SyncIoBridge to convert blocking I/O to async
    pub type TokioUnixStream = SyncIoBridge<UdsUnixStream>;

    // Wrapper for UnixListener
    pub struct UnixListener {
        inner: Arc<UdsUnixListener>,
    }

    impl UnixListener {
        pub async fn bind(path: &PathBuf) -> Result<Self> {
            // Convert PathBuf to &str for uds_windows
            let path_str = path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid socket path"))?;
            // Use spawn_blocking since bind might block
            let listener =
                tokio::task::spawn_blocking(move || UdsUnixListener::bind(path_str)).await??;
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
                        Ok(Ok(stream)) => {
                            yield Ok(SyncIoBridge::new(stream));
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
