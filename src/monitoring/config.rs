// Configuration for the monitoring service
//
// This module provides configuration options for the monitoring service.

use serde::{Deserialize, Serialize};

/// Configuration for the monitoring service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Whether monitoring is enabled
    pub enabled: bool,
    
    /// Host to bind the metrics server to
    pub host: String,
    
    /// Port to bind the metrics server to
    pub port: u16,
    
    /// Interval in seconds for collecting system metrics
    pub collection_interval_secs: u64,
    
    /// Start time of the service (Unix timestamp)
    pub start_time: i64,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            host: "127.0.0.1".to_string(),
            port: 9090,
            collection_interval_secs: 15,
            start_time: chrono::Utc::now().timestamp(),
        }
    }
}

impl MonitoringConfig {
    /// Create a new monitoring configuration
    pub fn new(enabled: bool, host: String, port: u16, collection_interval_secs: u64) -> Self {
        Self {
            enabled,
            host,
            port,
            collection_interval_secs,
            start_time: chrono::Utc::now().timestamp(),
        }
    }
}
