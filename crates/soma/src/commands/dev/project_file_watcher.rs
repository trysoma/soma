use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::future;
use globset::{Glob, GlobSet, GlobSetBuilder};
use notify::{EventKind, RecursiveMode};
use notify_debouncer_full::{DebounceEventResult, new_debouncer};
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, info};

use shared::error::CommonError;

#[derive(Debug, Clone)]
pub struct FileChangeEvt {
    pub changes: Vec<notify::Event>,
}

pub type FileChangeTx = broadcast::Sender<FileChangeEvt>;
pub type FileChangeRx = broadcast::Receiver<FileChangeEvt>;

fn files_to_watch_v1() -> Result<GlobSet, CommonError> {
    let mut builder = GlobSetBuilder::new();

    builder.add(Glob::new("soma.yaml")?);

    Ok(builder.build()?)
}

fn files_to_ignore_v1() -> Result<GlobSet, CommonError> {
    let builder = GlobSetBuilder::new();

    Ok(builder.build()?)
}

fn collect_paths_to_watch(
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

/// Starts a file watcher that monitors the source directory for changes
pub async fn start_dev_file_watcher(
    src_dir: PathBuf,
    file_change_tx: Arc<FileChangeTx>,
) -> Result<(), CommonError> {
    let files_to_watch = files_to_watch_v1()?;
    let files_to_ignore = files_to_ignore_v1()?;

    let (file_change_debounced_tx, mut file_change_debounced_rx) =
        mpsc::channel::<(Instant, Vec<notify::Event>)>(10);

    // Spawn a helper task that collapses bursts into one event
    tokio::spawn({
        let file_change_tx = file_change_tx.clone();
        async move {
            let mut last_trigger = Instant::now() - Duration::from_secs(10);
            let mut debounced_changes = Vec::new();
            while let Some((ts, changes)) = file_change_debounced_rx.recv().await {
                info!("🔁  file change received, {:?}", changes);
                debounced_changes.extend(changes);
                // If last trigger was within 1s, skip
                if ts.duration_since(last_trigger) < Duration::from_secs(1) {
                    continue;
                }
                last_trigger = ts;

                info!("🔁 Debounced file change fired");
                let _ = file_change_tx.send(FileChangeEvt {
                    changes: debounced_changes,
                });
                debounced_changes = Vec::new();
            }
        }
    });

    // Clone for use in the closure
    let files_to_ignore_clone = files_to_ignore.clone();
    let mut debouncer = new_debouncer(
        Duration::from_secs(1),
        None,
        move |result: DebounceEventResult| {
            match result {
                Ok(events) => {
                    // Filter out events from ignored paths
                    let mut any_relevant = vec![];

                    for event in events {
                        debug!("🔁  file change  received, {:?}", event);
                        // Filter out ignored paths
                        let ignored = event
                            .event
                            .paths
                            .iter()
                            .any(|p| files_to_ignore_clone.is_match(p));
                        if ignored {
                            continue;
                        }

                        // Only trigger on write / modify / rename / remove, not reads
                        match event.event.kind {
                            EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {
                                any_relevant.push(event.event);
                            }
                            _ => {}
                        }
                    }

                    if !any_relevant.is_empty() {
                        // Blockingly send a single event signal
                        let _ =
                            file_change_debounced_tx.blocking_send((Instant::now(), any_relevant));
                    }
                }
                Err(errors) => errors.iter().for_each(|error| println!("{error:?}")),
            }
        },
    )?;

    let paths = collect_paths_to_watch(&src_dir, &files_to_watch, &files_to_ignore);

    for path in paths {
        // For directories, watch recursively; for files, non-recursive
        let mode = if path.is_dir() {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };
        debouncer.watch(&path, mode)?;
        tracing::debug!("Watching: {:?}", path);
    }

    future::pending::<()>().await;

    Ok(())
}

/// Waits for soma.yaml configuration changes
#[allow(dead_code)]
pub async fn on_soma_config_change(file_change_rx: &mut FileChangeRx) -> Result<bool, CommonError> {
    loop {
        let event = file_change_rx.recv().await?;
        if is_soma_config_change(&event) {
            return Ok(true);
        }
    }
}

/// Checks if a file change event contains soma.yaml changes (testable)
#[allow(dead_code)]
pub fn is_soma_config_change(event: &FileChangeEvt) -> bool {
    event
        .changes
        .iter()
        .any(|change| change.paths.iter().any(|path| path.ends_with("soma.yaml")))
}


/// Starts the file watcher subsystem
// Rust doesnt allow opaque type in "type" definitions so we must disable clippy for this
#[allow(clippy::type_complexity)]
pub fn start_project_file_watcher(
    project_dir: PathBuf,
) -> Result<
    (
        Arc<FileChangeTx>,
        FileChangeRx,
        impl Future<Output = Result<(), CommonError>> + Send,
    ),
    CommonError,
> {
    // Setup file change notification
    let (file_change_tx, file_change_rx) = tokio::sync::broadcast::channel::<FileChangeEvt>(10);
    let file_change_tx = Arc::new(file_change_tx);

    let file_change_tx_clone = file_change_tx.clone();
    let file_watcher_fut = start_dev_file_watcher(project_dir, file_change_tx_clone);
    Ok((file_change_tx, file_change_rx, file_watcher_fut))
}

#[cfg(test)]
mod tests {
    use super::*;
    use notify::{Event, EventKind};
    use std::path::PathBuf;

    #[test]
    fn test_is_soma_config_change_with_soma_yaml() {
        let event = FileChangeEvt {
            changes: vec![Event {
                kind: EventKind::Modify(notify::event::ModifyKind::Data(
                    notify::event::DataChange::Any,
                )),
                paths: vec![PathBuf::from("/some/path/soma.yaml")],
                attrs: Default::default(),
            }],
        };

        assert!(is_soma_config_change(&event));
    }

    #[test]
    fn test_is_soma_config_change_without_soma_yaml() {
        let event = FileChangeEvt {
            changes: vec![Event {
                kind: EventKind::Modify(notify::event::ModifyKind::Data(
                    notify::event::DataChange::Any,
                )),
                paths: vec![PathBuf::from("/some/path/index.ts")],
                attrs: Default::default(),
            }],
        };

        assert!(!is_soma_config_change(&event));
    }

    #[test]
    fn test_is_soma_config_change_with_multiple_files() {
        let event = FileChangeEvt {
            changes: vec![
                Event {
                    kind: EventKind::Modify(notify::event::ModifyKind::Data(
                        notify::event::DataChange::Any,
                    )),
                    paths: vec![PathBuf::from("/src/index.ts")],
                    attrs: Default::default(),
                },
                Event {
                    kind: EventKind::Modify(notify::event::ModifyKind::Data(
                        notify::event::DataChange::Any,
                    )),
                    paths: vec![PathBuf::from("/soma.yaml")],
                    attrs: Default::default(),
                },
            ],
        };

        assert!(is_soma_config_change(&event));
    }

    #[test]
    fn test_is_soma_config_change_empty_event() {
        let event = FileChangeEvt { changes: vec![] };

        assert!(!is_soma_config_change(&event));
    }
}
