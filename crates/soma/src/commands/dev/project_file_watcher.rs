use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::future;
use globset::GlobSet;
use notify::{EventKind, RecursiveMode};
use notify_debouncer_full::{DebounceEventResult, new_debouncer};
use tokio::sync::{broadcast, mpsc};
use tokio_graceful_shutdown::{SubsystemBuilder, SubsystemHandle};
use tracing::{debug, error, info};

use shared::error::CommonError;

use super::runtime::{Runtime, collect_paths_to_watch, files_to_ignore_bun_v1, files_to_watch_bun_v1};

#[derive(Debug, Clone)]
pub struct FileChangeEvt {
    pub changes: Vec<notify::Event>,
}

pub type FileChangeTx = broadcast::Sender<FileChangeEvt>;
pub type FileChangeRx = broadcast::Receiver<FileChangeEvt>;

/// Starts a file watcher that monitors the source directory for changes
pub async fn start_dev_file_watcher(
    src_dir: &PathBuf,
    runtime: &Runtime,
    file_change_tx: Arc<FileChangeTx>,
) -> Result<(), CommonError> {
    let (files_to_watch, files_to_ignore) = match runtime {
        Runtime::BunV1 => (files_to_watch_bun_v1()?, files_to_ignore_bun_v1()?),
    };

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
pub async fn on_soma_config_change(file_change_rx: &mut FileChangeRx) -> Result<bool, CommonError> {
    loop {
        let event = file_change_rx.recv().await?;
        if is_soma_config_change(&event) {
            return Ok(true);
        }
    }
}

/// Checks if a file change event contains soma.yaml changes (testable)
pub fn is_soma_config_change(event: &FileChangeEvt) -> bool {
    event
        .changes
        .iter()
        .any(|change| change.paths.iter().any(|path| path.ends_with("soma.yaml")))
}


/// Starts the file watcher subsystem
pub fn start_project_file_watcher_subsystem(
    subsys: &SubsystemHandle,
    project_dir: &PathBuf,
    runtime: &Runtime,
) -> Result<(Arc<FileChangeTx>, FileChangeRx), CommonError> {
    // Setup file change notification
    let (file_change_tx, file_change_rx) = tokio::sync::broadcast::channel::<FileChangeEvt>(10);
    let file_change_tx = Arc::new(file_change_tx);

    let file_change_tx_clone = file_change_tx.clone();
    let project_dir_clone = project_dir.clone();
    let runtime_clone = runtime.clone();
    subsys.start(SubsystemBuilder::new(
        "file-watcher",
        move |subsys: SubsystemHandle| async move {
            tokio::select! {
                _ = subsys.on_shutdown_requested() => {
                    info!("system shutdown requested");
                    info!("File watcher stopped");
                }
                result = start_dev_file_watcher(&project_dir_clone, &runtime_clone, file_change_tx_clone) => {
                    if let Err(e) = result {
                        error!("File watcher stopped unexpectedly: {:?}", e);
                    }
                    info!("File watcher stopped");
                    subsys.request_shutdown();
                }
            }

            Ok::<(), CommonError>(())
        },
    ));

    Ok((file_change_tx, file_change_rx))
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use notify::{Event, EventKind};

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
        let event = FileChangeEvt {
            changes: vec![],
        };

        assert!(!is_soma_config_change(&event));
    }
}
