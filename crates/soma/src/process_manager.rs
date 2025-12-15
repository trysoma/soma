use pmdaemon::{ProcessConfig as PmDaemonProcessConfig, ProcessManager, ProcessState, ProcessStatus, config::ExecMode};
use shared::error::CommonError;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::sync::RwLock;
use tokio::io::{AsyncBufReadExt, AsyncSeekExt, BufReader, SeekFrom};
use tokio::fs::OpenOptions;
use tokio::sync::mpsc;

pub struct CustomProcessManager {
    manager: ProcessManager,
    processes: Arc<RwLock<std::collections::HashMap<String, ProcessHandle>>>,
    shutdown_triggered: Arc<RwLock<bool>>,
    follow_logs: bool,
}

#[derive(Debug, Clone)]
pub enum OnStop {
    TriggerShutdown,
    Ignore,
}

pub enum ProcessHandleInner {
    ProcessStatus(ProcessStatus),
    JoinHandle(tokio::task::JoinHandle<Result<(), CommonError>>),
}

pub struct ProcessHandle {
    inner: ProcessHandleInner,
    on_stop: OnStop,
    shutdown_priority: u32,
}

#[derive(Debug)]
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

    pub on_stop: OnStop,
    
    /// Shutdown priority (higher number = higher priority, shutdown first)
    pub shutdown_priority: u32,
}

#[derive(Debug)]
pub struct ThreadConfig {
    handle: tokio::task::JoinHandle<Result<(), CommonError>>,
    health_check: Option<pmdaemon::health::HealthCheckConfig>,
    on_stop: OnStop,
    /// Shutdown priority (higher number = higher priority, shutdown first)
    pub shutdown_priority: u32,
}

fn construct_pm_process_config(config: ProcessConfig, name: &str, log_file_path: PathBuf) -> PmDaemonProcessConfig {
    PmDaemonProcessConfig {
        name: name.to_string(),
        script: config.script,
        args: config.args,
        cwd: config.cwd,
        env: config.env,
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
        health_check: config.health_check,
    }
}

impl ProcessHandle {
    pub fn new_with_process_status(inner: ProcessStatus, on_stop: OnStop, shutdown_priority: u32) -> Self {
        Self {
            inner: ProcessHandleInner::ProcessStatus(inner),
            on_stop,
            shutdown_priority,
        }
    }

    pub fn new_with_join_handle(inner: tokio::task::JoinHandle<Result<(), CommonError>>, on_stop: OnStop, shutdown_priority: u32) -> Self {
        Self {
            inner: ProcessHandleInner::JoinHandle(inner),
            on_stop,
            shutdown_priority,
        }
    }
}

impl CustomProcessManager {
    pub async fn new() -> Result<Self, CommonError> {
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
            follow_logs: false,
        })
    }

    pub async fn stop_process(&mut self, name: &str) -> Result<(), CommonError> {
        trace!("Stopping process: {}", name);
        if let Some(process) = self.processes.write().await.get(name) {
            match &process.inner {
                ProcessHandleInner::ProcessStatus(_status) => {
                    trace!("Sending signal to stop process using ProcessStatus: {}", name);
                    self.manager.stop(name).await
                        .inspect_err(|_e| error!("Failed to send signal to stop process: {name}"))?;
                    trace!("Signal sent to stop process using ProcessStatus: {}", name);
                }
                ProcessHandleInner::JoinHandle(handle) => {
                    trace!("Aborting join handle for process: {}", name);
                    handle.abort();
                    trace!("Join handle aborted for process: {}", name);
                }
            }
        }
        trace!("Signal sent to stop process: {}", name);
        self.wait_for_stop(name).await?;
        trace!("Process stopped: {}", name);
        self.processes.write().await.remove(name);
        Ok(())
    }
    
    pub async fn start_process(&mut self, name: &str, config: ProcessConfig) -> Result<(), CommonError> {
        // Check if shutdown has been triggered
        if *self.shutdown_triggered.read().await {
            debug!("Shutdown triggered, cannot start process {name}");
            return Ok(());
        }
        
        trace!("Starting service: {}", name);
        trace!("Config for service ({name}): {:?}", config);
        
        // Stop existing service if it exists
        if let Ok(_) = self.manager.get_process_info(name).await {
            trace!("Stopping existing service: {}", name);
            self.manager.stop(name).await?;
            trace!("Waiting for service to stop: {}", name);
            self.wait_for_stop(name).await?;
            trace!("Service stopped: {}", name);
        }
        let log_process_name = format!("sys-{}-log", name);
        self.stop_process(&log_process_name).await
            .inspect_err(|e| error!("Failed to stop log process: {log_process_name}"))?;
        
        // Get PMDAEMON_HOME to construct actual log file paths
        let pmdaemon_home = std::env::var("PMDAEMON_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir().join("pmdaemon"));
        
        // PMDaemon writes to {PMDAEMON_HOME}/logs/{process-name}-out.log and {process-name}-error.log
        let logs_dir = pmdaemon_home.join("logs");
        let out_log_path = logs_dir.join(format!("{}-out.log", name));
        let error_log_path = logs_dir.join(format!("{}-error.log", name));
        
        trace!("PMDaemon log files for {}: out={}, error={}", name, out_log_path.display(), error_log_path.display());
        
        // Start log tailing task for both stdout and stderr
        if self.follow_logs {
            let name_clone = name.to_string();
            let out_log_path_clone = out_log_path.clone();
            let error_log_path_clone = error_log_path.clone();
            let handle = tokio::spawn(async move {
                // Tail both out and error logs
                let out_handle: tokio::task::JoinHandle<Result<(), CommonError>> = tokio::spawn({
                    let out_path = out_log_path_clone.clone();
                    let name = format!("{}-out", name_clone);
                    async move {
                        tail_log_file(&out_path, &name).await
                            .inspect_err(|e| error!("Failed to tail out log file for {name}: {e}"))?;
                        Ok(())
                    }
                });
                
                let error_handle: tokio::task::JoinHandle<Result<(), CommonError>> = tokio::spawn({
                    let error_path = error_log_path_clone.clone();
                    let name = format!("{}-err", name_clone);
                    async move {
                        tail_log_file(&error_path, &name).await
                            .inspect_err(|e| error!("Failed to tail error log file for {name}: {e}"))?;
                        Ok(())
                    }
                });
                
                // Wait for both to complete (they run forever, so this will never return)
                tokio::select! {
                    _ = out_handle => {},
                    _ = error_handle => {},
                }
                
                Ok(())
            });
            
            // Store the handle
            self.start_thread(&log_process_name, ThreadConfig { 
                handle, 
                health_check: None, 
                on_stop: OnStop::TriggerShutdown,
                shutdown_priority: config.shutdown_priority.saturating_sub(1), // Log processes have lower priority
            })
                .await
                .inspect_err(|_e| error!("Failed to start log tailing thread for {log_process_name}"))?;
        }
        
        // Clone on_stop and shutdown_priority before moving config
        let on_stop = config.on_stop.clone();   
        let shutdown_priority = config.shutdown_priority;
        
        // Use a dummy log file path for pmdaemon config (it will use its own paths anyway)
        let dummy_log_path = logs_dir.join(format!("{}.log", name));
        let pm_config = construct_pm_process_config(config, &name, dummy_log_path);
        
        // Start new service
        let process_id = self.manager.start(pm_config).await
            .inspect_err(|e| error!("Failed to start process: {name} via process maanger"))?;
        trace!("Service started: {} with ID: {}", name, process_id);
        
        // Health checks are configured in the ProcessConfig
        // The process manager handles health monitoring internally
        
        // Update our process tracking
        let info = self.manager.get_process_info(name).await
            .inspect_err(|e| error!("Failed to get process info for {name} via process manager"))?;
        self.processes.write().await.insert(name.to_string(), ProcessHandle::new_with_process_status(info, on_stop, shutdown_priority));
        trace!("Service info updated: {}", name);

        Ok(())
    }
    
    async fn wait_for_stop(&self, name: &str) -> Result<(), CommonError> {
        let timeout = Duration::from_secs(30);
        let start = std::time::Instant::now();
        
        while start.elapsed() < timeout {
            match self.manager.get_process_info(name).await {
                Err(_) => return Ok(()), // Process not found, it's stopped
                Ok(info) if info.state == ProcessState::Stopped => return Ok(()),
                _ => sleep(Duration::from_secs(1)).await,
            }
        }
        
        Err(CommonError::Unknown(anyhow::anyhow!("Process did not stop within timeout")))
    }

    pub async fn start_thread(
        &mut self,
        name: &str,
        config: ThreadConfig,
    ) -> Result<(), CommonError> {
        // Check if shutdown has been triggered
        if *self.shutdown_triggered.read().await {
            debug!("Shutdown triggered, immediately aborting thread {name}");
            config.handle.abort();
            return Ok(());
        }
        
        trace!("Starting process with JoinHandle: {}", name);

        let name_clone = name.to_string();
        let on_stop_clone = config.on_stop.clone();
        let original_handle = config.handle;
        
        // Create a wrapper handle that monitors the original handle
        // This wrapper awaits the original, handles logging and on_stop behavior,
        // then returns the result
        let wrapper_handle = tokio::spawn(async move {
            let result = original_handle.await;
            
            // Log the result
            match &result {
                Ok(Ok(())) => {
                    trace!("Thread {} completed successfully", name_clone);
                }
                Ok(Err(e)) => {
                    error!("Thread {} stopped unexpectedly with an error", name_clone);
                }
                Err(e) => {
                    error!("Thread {} stopped unexpectedly with an error", name_clone);
                }
            }
            
            // Handle on_stop action
            match on_stop_clone {
                OnStop::TriggerShutdown => {
                    debug!("triggering system shutdown because thread {} stopped", name_clone);
                    // Note: Actual shutdown triggering would need access to shutdown signal
                    // This is logged for now - caller can handle shutdown logic
                }
                OnStop::Ignore => {
                    debug!("thread {} stopped, ignoring as configured", name_clone);
                }
            }
            
            // Return the inner result, converting join errors to CommonError
            match result {
                Ok(inner_result) => inner_result,
                Err(e) => Err(CommonError::Unknown(anyhow::anyhow!("Join handle error: {:?}", e))),
            }
        });
        
        // Create process handle with the wrapper handle
        let process_handle = ProcessHandle::new_with_join_handle(wrapper_handle, config.on_stop.clone(), config.shutdown_priority);
        
        // Store the process handle (we can't clone it, so we store it and return a placeholder)
        self.processes.write().await.insert(name.to_string(), process_handle);

        trace!("Process started with JoinHandle: {}", name);

        Ok(())
    }

    /// Triggers graceful shutdown of all processes and threads, ordered by shutdown priority.
    /// Higher priority processes/threads are shut down first.
    /// Once called, no new processes or threads can be started.
    pub async fn trigger_shutdown(&mut self) -> Result<(), CommonError> {
        // Set shutdown flag to prevent new starts
        *self.shutdown_triggered.write().await = true;
        
        info!("Triggering graceful shutdown of all processes and threads");
        
        // Get all processes with their names and priorities
        let processes = self.processes.read().await;
        let mut process_list: Vec<(String, u32)> = processes.iter()
            .map(|(name, handle)| (name.clone(), handle.shutdown_priority))
            .collect();
        drop(processes);
        
        // Sort by shutdown priority (highest first)
        process_list.sort_by(|a, b| b.1.cmp(&a.1));
        
        trace!("Shutting down {} processes/threads in priority order", process_list.len());
        
        // Shutdown each process/thread in priority order
        for (name, priority) in process_list {
            info!("Shutting down {} (priority: {})", name, priority);
            
            // Stop the process
            if let Err(e) = self.stop_process(&name).await {
                warn!("Failed to stop process {}, continuiing to force exit, some processes may hang", name);
                // Continue with other processes even if one fails
            } else {
                debug!("Successfully shut down {}", name);
            }
        }
        
        trace!("All processes and threads have been shut down");
        Ok(())
    }

    /// Called when shutdown is complete. Can be used for cleanup or notifications.
    pub async fn on_shutdown_complete(&self) -> Result<(), CommonError> {
        debug!("Shutdown sequence completed");
        
        // Verify all processes are stopped
        let processes = self.processes.read().await;
        if !processes.is_empty() {
            warn!("Some processes are still registered after shutdown: {:#?}", processes.keys().collect::<Vec<_>>());
        } else {
            debug!("All processes have been successfully removed");
        }
        
        Ok(())
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
                        error!("Error seeking in log file: {e}");
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
                                    error!("Error reading log file: {e}");
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
