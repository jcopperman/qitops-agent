// Metrics server for QitOps Agent
//
// This module provides an HTTP server for exposing Prometheus metrics.

use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use prometheus::{Encoder, TextEncoder};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info};

use super::config::MonitoringConfig;

/// Metrics server for QitOps Agent
pub struct MetricsServer {
    /// Configuration for the metrics server
    config: MonitoringConfig,
}

impl MetricsServer {
    /// Create a new metrics server
    pub fn new(config: MonitoringConfig) -> Self {
        Self { config }
    }

    /// Start the metrics server
    pub async fn start(&self) -> Result<()> {
        if !self.config.enabled {
            info!("Metrics server is disabled");
            return Ok(());
        }

        let addr = format!("{}:{}", self.config.host, self.config.port);
        let addr: SocketAddr = addr.parse()?;

        // Create the router
        let app = Router::new()
            .route("/", get(index_handler))
            .route("/metrics", get(metrics_handler))
            .route("/health", get(health_handler))
            .with_state(Arc::new(self.config.clone()));

        // Start the server
        info!("Starting metrics server on {}", addr);
        let listener = TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

/// Handler for the index route
async fn index_handler() -> impl IntoResponse {
    Html(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>QitOps Agent Metrics</title>
            <style>
                body {
                    font-family: Arial, sans-serif;
                    margin: 0;
                    padding: 20px;
                    line-height: 1.6;
                }
                h1 {
                    color: #333;
                }
                a {
                    color: #0366d6;
                    text-decoration: none;
                }
                a:hover {
                    text-decoration: underline;
                }
                .container {
                    max-width: 800px;
                    margin: 0 auto;
                }
                .card {
                    background-color: #f6f8fa;
                    border-radius: 6px;
                    padding: 20px;
                    margin-bottom: 20px;
                }
            </style>
        </head>
        <body>
            <div class="container">
                <h1>QitOps Agent Metrics</h1>
                <div class="card">
                    <p>This is the metrics endpoint for QitOps Agent.</p>
                    <p>Available endpoints:</p>
                    <ul>
                        <li><a href="/metrics">/metrics</a> - Prometheus metrics</li>
                        <li><a href="/health">/health</a> - Health check</li>
                    </ul>
                </div>
                <div class="card">
                    <p>To view these metrics in Grafana, configure a Prometheus data source with this URL.</p>
                </div>
            </div>
        </body>
        </html>
        "#,
    )
}

/// Handler for the metrics route
async fn metrics_handler() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    
    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        error!("Failed to encode metrics: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to encode metrics").into_response();
    }
    
    match String::from_utf8(buffer) {
        Ok(metrics) => (StatusCode::OK, metrics).into_response(),
        Err(e) => {
            error!("Failed to convert metrics to UTF-8: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to convert metrics to UTF-8").into_response()
        }
    }
}

/// Handler for the health route
async fn health_handler(State(config): State<Arc<MonitoringConfig>>) -> impl IntoResponse {
    let uptime = chrono::Utc::now().timestamp() - config.start_time;
    let response = serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_seconds": uptime,
    });
    
    (StatusCode::OK, serde_json::to_string(&response).unwrap()).into_response()
}
