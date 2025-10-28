mod client;
mod typescript;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use futures::{FutureExt, TryFutureExt, future};
use globset::{Glob, GlobSet, GlobSetBuilder};
use tokio::process::Command;
use tokio::sync::{broadcast, oneshot};
use tracing::info;

use shared::command::run_child_process;
use shared::error::CommonError;

use crate::commands::dev::DevParams;
use crate::utils::construct_src_dir_absolute;

use super::project_file_watcher::FileChangeRx;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Runtime {
    BunV1,
}

/// Determines which runtime to use based on the project structure
pub fn determine_runtime(params: &DevParams) -> Result<Option<Runtime>, CommonError> {
    let src_dir = construct_src_dir_absolute(params.src_dir.clone())?;
    determine_runtime_from_dir(&src_dir)
}

/// Determines runtime from a directory path (testable version)
pub fn determine_runtime_from_dir(src_dir: &Path) -> Result<Option<Runtime>, CommonError> {
    let possible_runtimes = vec![(Runtime::BunV1, validate_runtime_bun_v1)];

    let mut matched_runtimes = vec![];

    for (runtime, validate_fn) in possible_runtimes {
        let result = validate_fn(src_dir.to_path_buf())?;
        if result {
            matched_runtimes.push(runtime);
        }
    }

    match matched_runtimes.len() {
        0 => Ok(None),
        1 => Ok(Some(matched_runtimes[0].clone())),
        _ => Err(CommonError::Unknown(anyhow::anyhow!(
            "Multiple runtimes matched"
        ))),
    }
}

fn validate_runtime_bun_v1(src_dir: PathBuf) -> Result<bool, CommonError> {
    validate_runtime_bun_v1_internal(&src_dir)
}

/// Internal validation function (easier to test)
fn validate_runtime_bun_v1_internal(src_dir: &Path) -> Result<bool, CommonError> {
    let files_to_check = vec![
        "package.json",
        "index.ts",
    ];
    for file in files_to_check {
        let file_path = src_dir.join(file);
        if !file_path.exists() {
            return Ok(false);
        }
    }
    Ok(true)
}

async fn build_runtime_bun_v1(_src_dir: PathBuf) -> Result<(), CommonError> {
    Ok(())
}

async fn start_runtime_bun_v1(
    src_dir: PathBuf,
    runtime_port: u16,
    kill_signal: oneshot::Receiver<()>,
    shutdown_complete: oneshot::Sender<()>,
) -> Result<(), CommonError> {
    let mut cmd = Command::new("bun");
    cmd.arg("index.ts").current_dir(src_dir);
    run_child_process(
        "bun",
        cmd,
        Some(kill_signal),
        Some(shutdown_complete),
        Some(HashMap::from([
            ("PORT".to_string(), runtime_port.to_string()),
        ])),
    )
    .await?;

    Ok(())
}

pub fn files_to_watch_bun_v1() -> Result<GlobSet, CommonError> {
    let mut builder = GlobSetBuilder::new();

    builder.add(Glob::new("**/*.ts")?);
    builder.add(Glob::new("package.json")?);
    builder.add(Glob::new("soma.yaml")?);

    Ok(builder.build()?)
}

pub fn files_to_ignore_bun_v1() -> Result<GlobSet, CommonError> {
    let mut builder = GlobSetBuilder::new();

    // Match node_modules anywhere in the path
    builder.add(Glob::new("**/node_modules/**")?);

    Ok(builder.build()?)
}

pub fn collect_paths_to_watch(
    root: &Path,
    watch_globs: &GlobSet,
    ignore_globs: &GlobSet,
) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(path) = stack.pop() {
        // Match against path relative to root for glob patterns
        let relative_path = path.strip_prefix(root).unwrap_or(&path);

        if ignore_globs.is_match(relative_path) {
            continue;
        }

        if path.is_dir() {
            // Push subdirs for recursive traversal
            if let Ok(read_dir) = fs::read_dir(&path) {
                for entry in read_dir.flatten() {
                    stack.push(entry.path());
                }
            }
        } else if watch_globs.is_match(relative_path) {
            paths.push(path);
        }
    }

    paths
}

pub struct StartDevRuntimeParams<'a> {
    pub project_dir: PathBuf,
    pub runtime: Runtime,
    pub runtime_port: u16,
    pub file_change_signal: &'a mut FileChangeRx,
    pub kill_signal: broadcast::Receiver<()>,
    pub shutdown_complete_signal: oneshot::Sender<()>,
}

/// Starts the development runtime with hot reloading on file changes
pub async fn start_dev_runtime<'a>(
    params: StartDevRuntimeParams<'a>,
) -> Result<(), CommonError> {
    let StartDevRuntimeParams {
        project_dir,
        runtime,
        runtime_port,
        file_change_signal,
        mut kill_signal,
        shutdown_complete_signal,
    } = params;
    loop {
        let (dev_kill_signal_tx, dev_kill_signal_rx) = oneshot::channel::<()>();
        let (dev_shutdown_complete_tx, dev_shutdown_complete_rx) = oneshot::channel::<()>();

        let serve_fut = match runtime {
            Runtime::BunV1 => build_runtime_bun_v1(project_dir.clone()).and_then(|_| {
                start_runtime_bun_v1(
                    project_dir.clone(),
                    runtime_port,
                    dev_kill_signal_rx,
                    dev_shutdown_complete_tx,
                )
            }),
        };

        let serve_fut = serve_fut.then(async |_| {
            info!("Runtime stopped, awaiting file change to restart or complete shutdown (CTRL+C)");
            future::pending::<()>().await;
            Ok::<(), CommonError>(())
        });

        tokio::select! {
            _ = file_change_signal.recv() => {
                info!("File change detected");
                let _ = dev_kill_signal_tx.send(());
                // Ignore channel errors during restart - process may have already exited
                let _ = dev_shutdown_complete_rx.await;
                continue;
            }
            _ = serve_fut => {}
            _ = kill_signal.recv() => {
                info!("System kill signal received");
                let _ = dev_kill_signal_tx.send(());
                // Ignore channel errors during shutdown - process may have already exited
                let _ = dev_shutdown_complete_rx.await;
                let _ = shutdown_complete_signal.send(());
                break;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validate_runtime_bun_v1_with_valid_project() {
        let temp_dir = TempDir::new().unwrap();

        // Create required files
        fs::write(temp_dir.path().join("package.json"), r#"{"name": "test"}"#).unwrap();
        fs::write(temp_dir.path().join("index.ts"), "console.log('test');").unwrap();

        let result = validate_runtime_bun_v1_internal(temp_dir.path()).unwrap();
        assert!(result, "Should validate as BunV1 runtime");
    }

    #[test]
    fn test_validate_runtime_bun_v1_missing_package_json() {
        let temp_dir = TempDir::new().unwrap();

        // Only create index.ts
        fs::write(temp_dir.path().join("index.ts"), "console.log('test');").unwrap();

        let result = validate_runtime_bun_v1_internal(temp_dir.path()).unwrap();
        assert!(!result, "Should not validate without package.json");
    }

    #[test]
    fn test_validate_runtime_bun_v1_missing_index_ts() {
        let temp_dir = TempDir::new().unwrap();

        // Only create package.json
        fs::write(temp_dir.path().join("package.json"), r#"{"name": "test"}"#).unwrap();

        let result = validate_runtime_bun_v1_internal(temp_dir.path()).unwrap();
        assert!(!result, "Should not validate without index.ts");
    }

    #[test]
    fn test_determine_runtime_from_dir_bun_v1() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("package.json"), r#"{"name": "test"}"#).unwrap();
        fs::write(temp_dir.path().join("index.ts"), "console.log('test');").unwrap();

        let runtime = determine_runtime_from_dir(temp_dir.path()).unwrap();
        assert_eq!(runtime, Some(Runtime::BunV1));
    }

    #[test]
    fn test_determine_runtime_from_dir_no_match() {
        let temp_dir = TempDir::new().unwrap();

        // Empty directory
        let runtime = determine_runtime_from_dir(temp_dir.path()).unwrap();
        assert_eq!(runtime, None);
    }

    #[test]
    fn test_files_to_watch_bun_v1() {
        let globs = files_to_watch_bun_v1().unwrap();

        assert!(globs.is_match("src/index.ts"));
        assert!(globs.is_match("package.json"));
        assert!(globs.is_match("soma.yaml"));
        assert!(globs.is_match("foo/bar/baz.ts"));
        assert!(!globs.is_match("README.md"));
    }

    #[test]
    fn test_files_to_ignore_bun_v1() {
        let globs = files_to_ignore_bun_v1().unwrap();

        assert!(globs.is_match("node_modules/foo/bar.js"));
        assert!(globs.is_match("src/node_modules/test.ts"));
        assert!(!globs.is_match("src/index.ts"));
    }

    #[test]
    fn test_collect_paths_to_watch() {
        let temp_dir = TempDir::new().unwrap();

        // Create a directory structure
        fs::create_dir(temp_dir.path().join("src")).unwrap();
        fs::create_dir(temp_dir.path().join("node_modules")).unwrap();

        fs::write(temp_dir.path().join("package.json"), "{}").unwrap();
        fs::write(temp_dir.path().join("src/index.ts"), "").unwrap();
        fs::write(temp_dir.path().join("src/app.ts"), "").unwrap();
        fs::write(temp_dir.path().join("node_modules/pkg.js"), "").unwrap();
        fs::write(temp_dir.path().join("README.md"), "").unwrap();

        let watch_globs = files_to_watch_bun_v1().unwrap();
        let ignore_globs = files_to_ignore_bun_v1().unwrap();

        let paths = collect_paths_to_watch(temp_dir.path(), &watch_globs, &ignore_globs);

        // Should include package.json and .ts files, but not node_modules or README.md
        assert!(paths.iter().any(|p| p.ends_with("package.json")));
        assert!(paths.iter().any(|p| p.ends_with("index.ts")));
        assert!(paths.iter().any(|p| p.ends_with("app.ts")));
        assert!(!paths.iter().any(|p| p.to_string_lossy().contains("node_modules")));
        assert!(!paths.iter().any(|p| p.ends_with("README.md")));
    }
}
