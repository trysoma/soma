use pmdaemon::{ProcessConfig as PmDaemonProcessConfig, ProcessManager, ProcessState, ProcessStatus, config::ExecMode};
use crate::error::CommonError;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::sync::RwLock;
use tokio::io::{AsyncBufReadExt, AsyncSeekExt, BufReader, SeekFrom};
use tokio::fs::OpenOptions;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use std::pin::Pin;

pub struct CustomProcessManager {
    manager: ProcessManager,
    processes: Arc<RwLock<std::collections::HashMap<String, ProcessHandle>>>,
    shutdown_triggered: Arc<RwLock<bool>>,
    shutdown_notifier: Arc<RwLock<Option<oneshot::Sender<()>>>>,
}

#[derive(Debug, Clone)]
pub enum OnTerminalStop {
    TriggerShutdown,
    Ignore,
}

#[derive(Debug, Clone)]
pub struct RestartConfig {
    pub max_restarts: u32,
    pub restart_delay: u64,
}

#[derive(Debug, Clone)]
pub enum OnStop {
    Nothing,
    Restart(RestartConfig),
}

pub enum ProcessHandleInner {
    ProcessStatus(ProcessStatus),
    JoinHandle(tokio::task::JoinHandle<Result<(), CommonError>>),
}

pub type ShutdownCallback = Box<dyn Fn() -> Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + Sync>;

pub struct ProcessHandle {
    inner: ProcessHandleInner,
    #[allow(dead_code)] // Stored for debugging/future use, used during thread startup
    on_terminal_stop: OnTerminalStop,
    #[allow(dead_code)] // Stored for debugging/future use, used during thread startup
    on_stop: Option<OnStop>, // Only used for threads, None for processes
    shutdown_priority: u32,
    on_shutdown_triggered: Option<ShutdownCallback>,
    on_shutdown_complete: Option<ShutdownCallback>,
}

pub struct ProcessConfig {

    /// Script or command to execute (required) - path to executable or command name
    pub script: String,

    /// Command line arguments passed to the script
    pub args: Vec<String>,

    /// Working directory for process execution (defaults to current directory)
    pub cwd: Option<PathBuf>,

    /// Environment variables injected into the process
    ///
    /// Note: PMDaemon automatically adds PORT, PM2_INSTANCE_ID, and NODE_APP_INSTANCE
    /// variables for clustering and port management.
    pub env: HashMap<String, String>,
    /// Health check configuration for the process
    ///
    /// Enables monitoring of process health through HTTP endpoints or custom scripts.
    /// Health checks can trigger automatic restarts when processes become unhealthy.
    pub health_check: Option<pmdaemon::health::HealthCheckConfig>,

    pub on_terminal_stop: OnTerminalStop,
    pub on_stop: OnStop, // For process restart behavior (handled by pmdaemon)
    
    /// Shutdown priority (higher number = higher priority, shutdown first)
    pub shutdown_priority: u32,
    pub follow_logs: bool,
    /// Optional callback invoked when shutdown is triggered (before stopping the process)
    /// Has a 30s timeout - if it takes longer, shutdown proceeds anyway
    pub on_shutdown_triggered: Option<ShutdownCallback>,
    /// Optional callback invoked when shutdown is complete (after stopping the process)
    /// Has a 30s timeout - if it takes longer, shutdown proceeds anyway
    pub on_shutdown_complete: Option<ShutdownCallback>,
}

pub struct ThreadConfig<F> {
    /// Callback that spawns the thread task. This allows the process manager to control restarts.
    pub spawn_fn: F,
    pub health_check: Option<pmdaemon::health::HealthCheckConfig>,
    pub on_terminal_stop: OnTerminalStop,
    pub on_stop: OnStop,
    /// Shutdown priority (higher number = higher priority, shutdown first)
    pub shutdown_priority: u32,
    pub follow_logs: bool,
    /// Optional callback invoked when shutdown is triggered (before stopping the thread)
    /// Has a 30s timeout - if it takes longer, shutdown proceeds anyway
    pub on_shutdown_triggered: Option<ShutdownCallback>,
    /// Optional callback invoked when shutdown is complete (after stopping the thread)
    /// Has a 30s timeout - if it takes longer, shutdown proceeds anyway
    pub on_shutdown_complete: Option<ShutdownCallback>,
}

impl<F> ThreadConfig<F>
where
    F: Fn() -> tokio::task::JoinHandle<Result<(), CommonError>> + Send + Sync + 'static,
{
    pub fn new(
        spawn_fn: F,
        health_check: Option<pmdaemon::health::HealthCheckConfig>,
        on_terminal_stop: OnTerminalStop,
        on_stop: OnStop,
        shutdown_priority: u32,
        follow_logs: bool,
    ) -> Self {
        Self {
            spawn_fn,
            health_check,
            on_terminal_stop,
            on_stop,
            shutdown_priority,
            follow_logs,
            on_shutdown_triggered: None,
            on_shutdown_complete: None,
        }
    }
}

fn construct_pm_process_config(config: &ProcessConfig, name: &str, log_file_path: PathBuf) -> PmDaemonProcessConfig {
    PmDaemonProcessConfig {
        name: name.to_string(),
        script: config.script.clone(),
        args: config.args.clone(),
        cwd: config.cwd.clone(),
        env: config.env.clone(),
        instances: 1,
        exec_mode: ExecMode::Fork,
        autorestart: true,
        max_restarts: 10,
        min_uptime: 1000 * 5,
        restart_delay: 1000,
        kill_timeout: 1600,
        max_memory_restart: None,
        out_file: Some(log_file_path.clone()),
        error_file: Some(log_file_path.clone()),
        log_file: Some(log_file_path),
        pid_file: None,
        watch: false,
        ignore_watch: vec![],
        user: None,
        group: None,
        namespace: "default".to_string(),
        port: None,
        health_check: config.health_check.clone(),
    }
}

impl ProcessHandle {
    pub fn new_with_process_status(
        inner: ProcessStatus,
        on_terminal_stop: OnTerminalStop,
        shutdown_priority: u32,
        on_shutdown_triggered: Option<ShutdownCallback>,
        on_shutdown_complete: Option<ShutdownCallback>,
    ) -> Self {
        Self {
            inner: ProcessHandleInner::ProcessStatus(inner),
            on_terminal_stop,
            on_stop: None, // Processes use pmdaemon's restart mechanism
            shutdown_priority,
            on_shutdown_triggered,
            on_shutdown_complete,
        }
    }

    pub fn new_with_join_handle(
        inner: tokio::task::JoinHandle<Result<(), CommonError>>,
        on_terminal_stop: OnTerminalStop,
        on_stop: OnStop,
        shutdown_priority: u32,
        on_shutdown_triggered: Option<ShutdownCallback>,
        on_shutdown_complete: Option<ShutdownCallback>,
    ) -> Self {
        Self {
            inner: ProcessHandleInner::JoinHandle(inner),
            on_terminal_stop,
            on_stop: Some(on_stop),
            shutdown_priority,
            on_shutdown_triggered,
            on_shutdown_complete,
        }
    }
}

impl CustomProcessManager {
    pub async fn new() -> Result<Self, CommonError> {
        Self::new_with_shutdown_notifier(None).await
    }

    pub async fn new_with_shutdown_notifier(shutdown_notifier: Option<oneshot::Sender<()>>) -> Result<Self, CommonError> {
        // Note: we dont ever want to resume processes from a previous session, so we start from scratch every time
        let uuid = Uuid::new_v4();
        let temp_pmdaemon_home = std::env::temp_dir().join(format!("pmdaemon/{uuid}"));
        unsafe {
            std::env::set_var("PMDAEMON_HOME", temp_pmdaemon_home.display().to_string());
        }
        let manager = ProcessManager::new().await
            .map_err(|e| CommonError::from(e))?;
        let processes = Arc::new(RwLock::new(std::collections::HashMap::new()));
        let shutdown_triggered = Arc::new(RwLock::new(false));
        
        Ok(Self {
            manager,
            processes,
            shutdown_triggered,
            shutdown_notifier: Arc::new(RwLock::new(shutdown_notifier)),
        })
    }

    pub async fn stop_process(&mut self, name: &str) -> Result<(), CommonError> {
        trace!(process = %name, "Stopping process");
        
        // Get the process handle and call on_shutdown_triggered if present
        let on_shutdown_triggered_future = {
            let processes = self.processes.read().await;
            processes.get(name).and_then(|p| p.on_shutdown_triggered.as_ref().map(|cb| cb()))
        };
        
        // Call on_shutdown_triggered with timeout
        if let Some(future) = on_shutdown_triggered_future {
            trace!(process = %name, "Calling on_shutdown_triggered callback");
            match tokio::time::timeout(Duration::from_secs(30), future).await {
                Ok(()) => trace!(process = %name, "on_shutdown_triggered callback completed"),
                Err(_) => warn!(process = %name, "on_shutdown_triggered callback timed out after 30s, proceeding with shutdown"),
            }
        }
        
        // Stop the process
        let shutdown_triggered = *self.shutdown_triggered.read().await;
        let is_pmdaemon_process = {
            let processes = self.processes.read().await;
            processes.get(name).map(|p| matches!(&p.inner, ProcessHandleInner::ProcessStatus(_)))
        };
        
        if let Some(true) = is_pmdaemon_process {
            trace!(process = %name, "Sending stop signal via ProcessStatus");
            self.manager.stop(name).await
                .inspect_err(|e| error!(process = %name, error = %e, "Failed to send stop signal"))?;
            
            // During shutdown, pmdaemon may restart the process due to autorestart: true
            // Keep stopping it until it stays stopped
            if shutdown_triggered {
                let mut consecutive_stops = 0;
                loop {
                    sleep(Duration::from_millis(1000)).await;
                    match self.manager.get_process_info(name).await {
                        Err(_) => {
                            trace!(process = %name, "Process not found, considered stopped");
                            break;
                        }
                        Ok(info) if info.state == ProcessState::Stopped => {
                            consecutive_stops += 1;
                            if consecutive_stops >= 2 {
                                trace!(process = %name, "Process stopped and stayed stopped");
                                break;
                            }
                        }
                        _ => {
                            // Process is running again, stop it
                            consecutive_stops = 0;
                            warn!(process = %name, "Process restarted during shutdown, stopping again");
                            if let Err(e) = self.manager.stop(name).await {
                                warn!(process = %name, error = %e, "Failed to stop restarted process");
                            }
                        }
                    }
                }
            } else {
                // Normal shutdown, just wait for stop
                self.wait_for_stop(name).await?;
            }
        } else if let Some(process) = self.processes.write().await.get(name) {
            match &process.inner {
                ProcessHandleInner::JoinHandle(handle) => {
                    trace!(process = %name, "Aborting thread handle");
                    handle.abort();
                    // For threads, wait a bit for abort to complete
                    sleep(Duration::from_millis(100)).await;
                }
                _ => {}
            }
        } else {
            // Process not found, consider it stopped
            trace!(process = %name, "Process not found in registry");
        }
        trace!(process = %name, "Process stopped");
        
        // Get and call on_shutdown_complete callback if present
        let on_shutdown_complete_future = {
            let processes = self.processes.read().await;
            processes.get(name).and_then(|p| p.on_shutdown_complete.as_ref().map(|cb| cb()))
        };
        
        // Remove from processes map
        self.processes.write().await.remove(name);
        
        // Call on_shutdown_complete with timeout
        if let Some(future) = on_shutdown_complete_future {
            trace!(process = %name, "Calling on_shutdown_complete callback");
            match tokio::time::timeout(Duration::from_secs(30), future).await {
                Ok(()) => trace!(process = %name, "on_shutdown_complete callback completed"),
                Err(_) => warn!(process = %name, "on_shutdown_complete callback timed out after 30s, proceeding"),
            }
        }
        
        Ok(())
    }
    
    pub async fn start_process(&mut self, name: &str, config: ProcessConfig) -> Result<(), CommonError> {
        // Check if shutdown has been triggered
        if *self.shutdown_triggered.read().await {
            debug!(process = %name, "Shutdown triggered, skipping process start");
            return Ok(());
        }
        
        trace!(process = %name, "Starting process");
        
        // Stop existing service if it exists
        if let Ok(_) = self.manager.get_process_info(name).await {
            trace!(process = %name, "Stopping existing process");
            self.manager.stop(name).await?;
            self.wait_for_stop(name).await?;
        }
        let log_process_name = format!("sys-{}-log", name);
        self.stop_process(&log_process_name).await
            .inspect_err(|e| error!(process = %log_process_name, error = %e, "Failed to stop log tailing process"))?;
        
        // Get PMDAEMON_HOME to construct actual log file paths
        let pmdaemon_home = std::env::var("PMDAEMON_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir().join("pmdaemon"));
        
        // PMDaemon writes to {PMDAEMON_HOME}/logs/{process-name}-out.log and {process-name}-error.log
        let logs_dir = pmdaemon_home.join("logs");
        let out_log_path = logs_dir.join(format!("{}-out.log", name));
        let error_log_path = logs_dir.join(format!("{}-error.log", name));
        
        trace!(process = %name, out_log = %out_log_path.display(), error_log = %error_log_path.display(), "Resolved log file paths");
        
        // Start log tailing task for both stdout and stderr
        if config.follow_logs {
            // Store the handle - capture the spawn logic in a closure
            let out_log_path_final = out_log_path.clone();
            let error_log_path_final = error_log_path.clone();
            let name_for_log = name.to_string();
            self.start_thread(&log_process_name, ThreadConfig {
                spawn_fn: move || {
                    let out_path = out_log_path_final.clone();
                    let error_path = error_log_path_final.clone();
                    let name = format!("{}-log", name_for_log);
                    tokio::spawn(async move {
                        // Tail both out and error logs
                        let out_handle: tokio::task::JoinHandle<Result<(), CommonError>> = tokio::spawn({
                            let out_path = out_path.clone();
                            let name = format!("{}-out", name);
                            async move {
                                tail_log_file(&out_path, &name).await
                                    .inspect_err(|e| error!(log_file = %out_path.display(), error = %e, "Failed to tail stdout log"))?;
                                Ok(())
                            }
                        });
                        
                        let error_handle: tokio::task::JoinHandle<Result<(), CommonError>> = tokio::spawn({
                            let error_path = error_path.clone();
                            let name = format!("{}-err", name);
                            async move {
                                tail_log_file(&error_path, &name).await
                                    .inspect_err(|e| error!(log_file = %error_path.display(), error = %e, "Failed to tail stderr log"))?;
                                Ok(())
                            }
                        });
                        
                        // Wait for both to complete (they run forever, so this will never return)
                        tokio::select! {
                            _ = out_handle => {},
                            _ = error_handle => {},
                        }
                        
                        Ok(())
                    })
                },
                health_check: None, 
                on_terminal_stop: OnTerminalStop::TriggerShutdown,
                on_stop: OnStop::Restart(RestartConfig {
                    max_restarts: 10,
                    restart_delay: 1000,
                }),
                shutdown_priority: config.shutdown_priority.saturating_sub(1), // Log processes have lower priority
                follow_logs: false,
                on_shutdown_triggered: None,
                on_shutdown_complete: None,
            })
                .await
                .inspect_err(|e| error!(process = %log_process_name, error = %e, "Failed to start log tailing thread"))?;
        }
        
        // Clone on_terminal_stop and shutdown_priority before moving config
        let on_terminal_stop = config.on_terminal_stop.clone();   
        let shutdown_priority = config.shutdown_priority;
        
        // Use a dummy log file path for pmdaemon config (it will use its own paths anyway)
        let dummy_log_path = logs_dir.join(format!("{}.log", name));
        let pm_config = construct_pm_process_config(&config, &name, dummy_log_path);
        
        // Extract shutdown callbacks after using config (construct_pm_process_config doesn't use them)
        let on_shutdown_triggered = config.on_shutdown_triggered;
        let on_shutdown_complete = config.on_shutdown_complete;
        
        // Start new service
        let process_id = self.manager.start(pm_config).await
            .inspect_err(|e| error!(process = %name, error = %e, "Failed to start process"))?;
        info!(process = %name, process_id = %process_id, "Process started");
        
        // Health checks are configured in the ProcessConfig
        // The process manager handles health monitoring internally
        
        // Update our process tracking
        let info = self.manager.get_process_info(name).await
            .inspect_err(|e| error!(process = %name, error = %e, "Failed to get process info"))?;
        self.processes.write().await.insert(name.to_string(), ProcessHandle::new_with_process_status(info, on_terminal_stop, shutdown_priority, on_shutdown_triggered, on_shutdown_complete));
        trace!(process = %name, "Process registered");

        Ok(())
    }
    
    async fn wait_for_stop(&self, name: &str) -> Result<(), CommonError> {
        let timeout = Duration::from_secs(30);
        let start = std::time::Instant::now();
        
        while start.elapsed() < timeout {
            match self.manager.get_process_info(name).await {
                Err(_) => {
                    trace!(process = %name, "Process not found, considered stopped");
                    return Ok(());
                }
                Ok(info) if info.state == ProcessState::Stopped => {
                    trace!(process = %name, "Process stopped");
                    return Ok(());
                }
                _ => {
                    trace!(process = %name, elapsed_ms = start.elapsed().as_millis(), "Waiting for process to stop");
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }
        
        Err(CommonError::Unknown(anyhow::anyhow!("Process did not stop within timeout")))
    }

    pub async fn start_thread<F>(
        &mut self,
        name: &str,
        config: ThreadConfig<F>,
    ) -> Result<(), CommonError>
    where
        F: Fn() -> tokio::task::JoinHandle<Result<(), CommonError>> + Send + Sync + 'static,
    {
        // Check if shutdown has been triggered
        if *self.shutdown_triggered.read().await {
            debug!(thread = %name, "Shutdown triggered, skipping thread start");
            return Ok(());
        }
        
        trace!(thread = %name, "Starting thread");

        let name_clone = name.to_string();
        let on_terminal_stop_clone = config.on_terminal_stop.clone();
        let on_stop_clone = config.on_stop.clone();
        let shutdown_priority = config.shutdown_priority;
        let spawn_fn = config.spawn_fn;
        
        // Get a reference to shutdown_triggered for the restart loop
        let shutdown_triggered = self.shutdown_triggered.clone();
        let shutdown_notifier = self.shutdown_notifier.clone();
        
        // Create a wrapper handle that manages the thread lifecycle, including restarts
        let wrapper_handle = tokio::spawn(async move {
            let mut restart_count = 0u32;
            let max_restarts = match &on_stop_clone {
                OnStop::Restart(restart_config) => restart_config.max_restarts,
                OnStop::Nothing => 0,
            };
            
            let mut last_result: Result<Result<(), CommonError>, tokio::task::JoinError> = Ok(Ok(()));
            
            loop {
                // Check if shutdown was triggered
                if *shutdown_triggered.read().await {
                    trace!(thread = %name_clone, "Shutdown triggered, stopping thread");
                    break;
                }
                
                // Spawn the thread using the callback
                let handle = (spawn_fn)();
                last_result = handle.await;
                
                // Log the result
                match &last_result {
                    Ok(Ok(())) => {
                        trace!(thread = %name_clone, "Thread completed successfully");
                    }
                    Ok(Err(e)) => {
                        error!(thread = %name_clone, error = %e, "Thread stopped with error");
                    }
                    Err(e) => {
                        error!(thread = %name_clone, error = ?e, "Thread join handle error");
                    }
                }
                
                // Handle on_stop action
                match &on_stop_clone {
                    OnStop::Restart(restart_config) => {
                        restart_count += 1;
                        if restart_count > max_restarts {
                            error!(thread = %name_clone, max_restarts, "Thread exceeded max restarts");
                            // Handle terminal stop
                            match on_terminal_stop_clone {
                                OnTerminalStop::TriggerShutdown => {
                                    warn!(thread = %name_clone, "Thread exceeded max restarts, triggering shutdown");
                                    if let Some(notifier) = shutdown_notifier.write().await.take() {
                                        let _ = notifier.send(());
                                    }
                                }
                                OnTerminalStop::Ignore => {
                                    debug!(thread = %name_clone, "Thread exceeded max restarts, ignoring as configured");
                                }
                            }
                            break;
                        }
                        
                        let delay_ms = restart_config.restart_delay;
                        trace!(thread = %name_clone, delay_ms, attempt = restart_count, max_attempts = max_restarts, "Restarting thread");
                        sleep(Duration::from_millis(delay_ms)).await;
                        // Continue loop to restart
                    }
                    OnStop::Nothing => {
                        // Handle terminal stop
                        match on_terminal_stop_clone {
                            OnTerminalStop::TriggerShutdown => {
                                warn!(thread = %name_clone, "Thread stopped, triggering shutdown");
                                if let Some(notifier) = shutdown_notifier.write().await.take() {
                                    let _ = notifier.send(());
                                }
                            }
                            OnTerminalStop::Ignore => {
                                debug!(thread = %name_clone, "Thread stopped, ignoring as configured");
                            }
                        }
                        break;
                    }
                }
            }
            
            // Return the last result, converting join errors to CommonError
            match last_result {
                Ok(inner_result) => inner_result,
                Err(e) => Err(CommonError::Unknown(anyhow::anyhow!("Join handle error: {:?}", e))),
            }
        });
        
        // Create process handle with the wrapper handle
        let process_handle = ProcessHandle::new_with_join_handle(
            wrapper_handle,
            config.on_terminal_stop,
            config.on_stop,
            shutdown_priority,
            config.on_shutdown_triggered,
            config.on_shutdown_complete,
        );
        
        // Store the process handle
        self.processes.write().await.insert(name.to_string(), process_handle);

        trace!(thread = %name, "Thread registered");

        Ok(())
    }

    /// Triggers graceful shutdown of all processes and threads, ordered by shutdown priority.
    /// Higher priority processes/threads are shut down first.
    /// Once called, no new processes or threads can be started.
    pub async fn trigger_shutdown(&mut self) -> Result<(), CommonError> {
        // Set shutdown flag to prevent new starts
        *self.shutdown_triggered.write().await = true;
        
        info!("Graceful shutdown initiated");
        
        // Get all processes with their names and priorities
        let processes = self.processes.read().await;
        let mut process_list: Vec<(String, u32)> = processes.iter()
            .map(|(name, handle)| (name.clone(), handle.shutdown_priority))
            .collect();
        drop(processes);
        
        // Sort by shutdown priority (highest first)
        process_list.sort_by(|a, b| b.1.cmp(&a.1));
        
        trace!(count = process_list.len(), "Shutting down processes in priority order");
        
        // Shutdown each process/thread in priority order
        for (name, priority) in process_list {
            trace!(process = %name, priority, "Stopping process");
            
            // Stop the process
            if let Err(e) = self.stop_process(&name).await {
                warn!(process = %name, error = %e, "Failed to stop process, continuing shutdown");
                // Continue with other processes even if one fails
            }
        }
        
        trace!("Shutdown sequence completed");
        Ok(())
    }

    /// Called when shutdown is complete. Can be used for cleanup or notifications.
    pub async fn on_shutdown_complete(&self) -> Result<(), CommonError> {
        // Verify all processes are stopped
        let processes = self.processes.read().await;
        if !processes.is_empty() {
            let remaining: Vec<&String> = processes.keys().collect();
            warn!(remaining = ?remaining, "Processes still registered after shutdown");
        } else {
            trace!("All processes removed");
        }
        
        Ok(())
    }
    
    /// Returns a future that completes when shutdown is triggered
    pub fn wait_for_shutdown(&self) -> impl std::future::Future<Output = ()> + Send {
        let shutdown_triggered = self.shutdown_triggered.clone();
        async move {
            loop {
                if *shutdown_triggered.read().await {
                    break;
                }
                sleep(Duration::from_millis(100)).await;
            }
        }
    }
}

async fn tail_log_file(log_file_path: &PathBuf, process_name: &str) -> Result<(), CommonError> {
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    
    // Spawn task to read log file and send lines through channel
    let log_file_path_clone = log_file_path.clone();
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        let mut last_position = 0u64;
        
        loop {
            match OpenOptions::new()
                .read(true)
                .open(&log_file_path_clone)
                .await
            {
                Ok(mut file) => {
                    // Get current file size
                    let metadata = match file.metadata().await {
                        Ok(m) => m,
                        Err(_) => {
                            sleep(Duration::from_millis(100)).await;
                            continue;
                        }
                    };
                    
                    let current_size = metadata.len();
                    
                    // If file was truncated or we haven't started reading yet
                    if current_size < last_position {
                        last_position = 0;
                    }
                    
                    // Seek to last position
                    if let Err(e) = file.seek(SeekFrom::Start(last_position)).await {
                        error!(log_file = %log_file_path_clone.display(), position = last_position, error = %e, "Failed to seek in log file");
                        sleep(Duration::from_millis(1000)).await;
                        continue;
                    }
                    
                    // Read new content
                    if current_size > last_position {
                        let mut reader = BufReader::new(file);
                        let mut buffer = String::new();
                        
                        loop {
                            buffer.clear();
                            match reader.read_line(&mut buffer).await {
                                Ok(0) => {
                                    // No more data
                                    break;
                                }
                                Ok(n) => {
                                    last_position += n as u64;
                                    let line = buffer.trim_end_matches('\n').trim_end_matches('\r');
                                    if !line.is_empty() {
                                        let _ = tx_clone.send(line.to_string());
                                    }
                                }
                                Err(e) => {
                                    error!(log_file = %log_file_path_clone.display(), error = %e, "Failed to read log file");
                                    break;
                                }
                            }
                        }
                    }
                    
                    sleep(Duration::from_millis(100)).await;
                }
                Err(_) => {
                    // File doesn't exist yet, wait a bit
                    sleep(Duration::from_millis(100)).await;
                }
            }
        }
    });
    
    // Spawn task to receive log lines and print them
    let process_name_clone = process_name.to_string();
    tokio::spawn(async move {
        while let Some(line) = rx.recv().await {
            println!("[{}] {}", process_name_clone, line);
        }
    });
    
    // Keep the function running (this task will run until aborted)
    loop {
        sleep(Duration::from_secs(1)).await;
    }
}

