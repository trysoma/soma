use std::path::PathBuf;

use shared::error::CommonError;

pub(crate) mod restate;
pub(crate) mod soma_agent_config;

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
