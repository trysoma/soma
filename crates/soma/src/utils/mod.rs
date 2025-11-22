pub mod config;
pub(crate) mod restate_binary;

use std::path::PathBuf;

use shared::error::CommonError;
use tokio::net::TcpListener;

pub fn construct_src_dir_absolute(src_dir: Option<PathBuf>) -> Result<PathBuf, CommonError> {
    let cwd = std::env::current_dir()?;
    let mut src_dir = match src_dir {
        Some(src_dir) => src_dir,
        None => cwd.clone(),
    };
    if !src_dir.is_absolute() {
        src_dir = cwd.join(src_dir);
    }

    Ok(src_dir)
}

pub async fn is_port_in_use(port: u16) -> Result<bool, CommonError> {
    match TcpListener::bind(("127.0.0.1", port)).await {
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
