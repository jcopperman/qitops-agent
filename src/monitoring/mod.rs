// Monitoring module for QitOps Agent
//
// This module provides metrics collection and monitoring capabilities for QitOps Agent.
// It uses Prometheus for metrics collection and exposes an HTTP endpoint for scraping.

use anyhow::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::{debug, error, info};

// Re-export metrics
pub mod metrics;
pub use metrics::*;

// Re-export server
pub mod server;
pub use server::*;

// Re-export config
pub mod config;
pub use config::*;

/// Monitoring service for QitOps Agent
pub struct MonitoringService {
    /// Configuration for the monitoring service
    config: MonitoringConfig,
    /// Server handle
    server_handle: Option<JoinHandle<()>>,
    /// Start time of the service
    #[allow(dead_code)]
    start_time: Instant,
}

impl MonitoringService {
    /// Create a new monitoring service
    pub fn new(config: MonitoringConfig) -> Self {
        Self {
            config,
            server_handle: None,
            start_time: Instant::now(),
        }
    }

    /// Start the monitoring service
    pub async fn start(&mut self) -> Result<()> {
        if !self.config.enabled {
            info!("Monitoring service is disabled");
            return Ok(());
        }

        info!(
            "Starting monitoring service on {}:{}",
            self.config.host, self.config.port
        );

        // Initialize system metrics
        initialize_system_metrics();

        // Start the metrics server
        let server = MetricsServer::new(self.config.clone());
        self.server_handle = Some(tokio::spawn(async move {
            if let Err(e) = server.start().await {
                error!("Failed to start metrics server: {}", e);
            }
        }));

        // Start the system metrics collector
        let config = self.config.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(config.collection_interval_secs));
            loop {
                interval.tick().await;
                collect_system_metrics();
            }
        });

        Ok(())
    }

    /// Stop the monitoring service
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(handle) = self.server_handle.take() {
            handle.abort();
            info!("Monitoring service stopped");
        }
        Ok(())
    }

    /// Get the uptime of the service
    #[allow(dead_code)]
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
}

/// Initialize system metrics
fn initialize_system_metrics() {
    // System metrics are initialized in the metrics module
    debug!("System metrics initialized");
}

/// Collect system metrics
fn collect_system_metrics() {
    // Update system metrics using sysinfo
    use sysinfo::{System, Pid};

    // Create a new System instance
    let mut system = System::new_all();

    // Refresh system information
    system.refresh_all();

    // Memory information
    let total_memory = system.total_memory() as f64;
    let used_memory = system.used_memory() as f64;
    let free_memory = total_memory - used_memory;
    let available_memory = system.available_memory() as f64;

    SYSTEM_MEMORY_TOTAL.set(total_memory);
    SYSTEM_MEMORY_FREE.set(free_memory);
    SYSTEM_MEMORY_AVAILABLE.set(available_memory);
    SYSTEM_MEMORY_BUFFERS.set(0.0); // Not directly available in sysinfo
    SYSTEM_MEMORY_CACHED.set(0.0);  // Not directly available in sysinfo

    // CPU load - get global CPU usage
    system.refresh_cpu();
    let global_cpu_usage = system.global_cpu_info().cpu_usage();

    // Set all load averages to the same value since we don't have separate 1m, 5m, 15m values
    SYSTEM_CPU_LOAD_1M.set(global_cpu_usage as f64);
    SYSTEM_CPU_LOAD_5M.set(global_cpu_usage as f64);
    SYSTEM_CPU_LOAD_15M.set(global_cpu_usage as f64);

    // Process metrics
    let pid = std::process::id();
    system.refresh_process(Pid::from_u32(pid));

    if let Some(process) = system.process(Pid::from_u32(pid)) {
        PROCESS_CPU_USAGE.set(process.cpu_usage() as f64);
        PROCESS_MEMORY_USAGE.set(process.memory() as f64);
    } else {
        // Process not found, set default values
        PROCESS_CPU_USAGE.set(0.0);
        PROCESS_MEMORY_USAGE.set(0.0);
    }
}

/// Singleton instance of the monitoring service
pub static MONITORING_SERVICE: once_cell::sync::Lazy<Arc<Mutex<MonitoringService>>> =
    once_cell::sync::Lazy::new(|| {
        Arc::new(Mutex::new(MonitoringService::new(MonitoringConfig::default())))
    });

/// Initialize the monitoring service
pub async fn init(config: MonitoringConfig) -> Result<()> {
    let mut service = MONITORING_SERVICE.lock().await;
    service.config = config;
    service.start().await
}

/// Stop the monitoring service
pub async fn stop() -> Result<()> {
    let mut service = MONITORING_SERVICE.lock().await;
    service.stop().await
}

/// Track the execution time of a function and record it as a metric
pub struct Timer {
    name: String,
    start: Instant,
}

impl Timer {
    /// Create a new timer
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            start: Instant::now(),
        }
    }

    /// Stop the timer and record the duration
    pub fn stop(self) {
        let duration = self.start.elapsed();
        track_duration(&self.name, duration.as_secs_f64());
    }
}

/// Track the execution time of a function and record it as a metric
pub fn track_duration(name: &str, duration: f64) {
    match name {
        "command" => COMMAND_DURATION.observe(duration),
        "llm_request" => LLM_REQUEST_DURATION.observe(duration),
        "test_gen" => TEST_GEN_DURATION.observe(duration),
        "pr_analyze" => PR_ANALYZE_DURATION.observe(duration),
        "risk" => RISK_DURATION.observe(duration),
        "test_data" => TEST_DATA_DURATION.observe(duration),
        "session" => SESSION_DURATION.observe(duration),
        _ => COMMAND_DURATION.observe(duration),
    }
}

/// Track a command execution
pub fn track_command(command: &str) {
    COMMAND_COUNTER.inc();
    match command {
        "test-gen" => TEST_GEN_COUNTER.inc(),
        "pr-analyze" => PR_ANALYZE_COUNTER.inc(),
        "risk" => RISK_COUNTER.inc(),
        "test-data" => TEST_DATA_COUNTER.inc(),
        "session" => SESSION_COUNTER.inc(),
        _ => {}
    }
}

/// Track an LLM request
pub fn track_llm_request(provider: &str) {
    LLM_REQUEST_COUNTER.inc();
    match provider {
        "openai" => LLM_OPENAI_REQUEST_COUNTER.inc(),
        "ollama" => LLM_OLLAMA_REQUEST_COUNTER.inc(),
        "anthropic" => LLM_ANTHROPIC_REQUEST_COUNTER.inc(),
        _ => {}
    }
}

/// Track LLM token usage
pub fn track_llm_token_usage(provider: &str, tokens: u64) {
    LLM_TOKEN_USAGE.inc_by(tokens as f64);
    match provider {
        "openai" => LLM_OPENAI_TOKEN_USAGE.inc_by(tokens as f64),
        "ollama" => LLM_OLLAMA_TOKEN_USAGE.inc_by(tokens as f64),
        "anthropic" => LLM_ANTHROPIC_TOKEN_USAGE.inc_by(tokens as f64),
        _ => {}
    }
}

/// Track an error
pub fn track_error(error_type: &str) {
    ERROR_COUNTER.inc();
    match error_type {
        "llm" => LLM_ERROR_COUNTER.inc(),
        "github" => GITHUB_ERROR_COUNTER.inc(),
        "agent" => AGENT_ERROR_COUNTER.inc(),
        _ => {}
    }
}

/// Track a cache hit
pub fn track_cache_hit() {
    CACHE_HIT_COUNTER.inc();
}

/// Track a cache miss
pub fn track_cache_miss() {
    CACHE_MISS_COUNTER.inc();
}

/// Track a session message
#[allow(dead_code)]
pub fn track_session_message(is_user: bool) {
    SESSION_MESSAGE_COUNTER.inc();
    if is_user {
        SESSION_USER_MESSAGE_COUNTER.inc();
    } else {
        SESSION_AGENT_MESSAGE_COUNTER.inc();
    }
}
