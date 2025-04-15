# QitOps Agent Monitoring

QitOps Agent includes a built-in monitoring system that provides insights into its usage, performance, and resource consumption. This document describes how to set up and use the monitoring system.

## Overview

The monitoring system consists of the following components:

- **Metrics Collection**: QitOps Agent collects metrics about its operation, such as command execution, LLM requests, and resource usage.
- **Metrics Endpoint**: QitOps Agent exposes a Prometheus-compatible metrics endpoint.
- **Grafana Dashboard**: A pre-built Grafana dashboard for visualizing the metrics.
- **Docker Compose Setup**: A Docker Compose file for setting up the complete monitoring stack.

## Metrics Collected

QitOps Agent collects the following types of metrics:

### System Metrics
- Memory usage (total, free, available, buffers, cached)
- CPU load (1m, 5m, 15m averages)
- Process CPU and memory usage

### Command Metrics
- Command execution count (total and by command type)
- Command execution duration
- Command success/failure rate

### LLM Metrics
- LLM request count (total and by provider)
- LLM request duration
- Token usage (total and by provider)
- Error rate

### Session Metrics
- Session count
- Session duration
- Message count (total, user, agent)

## Enabling Monitoring

Monitoring is disabled by default. To enable it, set the following environment variables:

```bash
# Enable monitoring
export QITOPS_MONITORING_ENABLED=true

# Configure the host and port for the metrics endpoint
export QITOPS_MONITORING_HOST=127.0.0.1  # Use 0.0.0.0 to allow external access
export QITOPS_MONITORING_PORT=9090

# Configure the metrics collection interval (in seconds)
export QITOPS_MONITORING_INTERVAL=15
```

Once enabled, QitOps Agent will expose a metrics endpoint at `http://<host>:<port>/metrics`.

## Setting Up the Monitoring Stack

QitOps Agent includes a Docker Compose file for setting up a complete monitoring stack with Prometheus, Grafana, Loki, and Promtail.

### Prerequisites

- Docker and Docker Compose installed
- QitOps Agent repository cloned

### Setup Steps

1. Create the necessary directories:

```bash
mkdir -p monitoring/prometheus monitoring/grafana/dashboards monitoring/grafana/provisioning/dashboards monitoring/grafana/provisioning/datasources monitoring/promtail
```

2. Start the monitoring stack:

```bash
docker-compose -f docker-compose-monitoring.yml up -d
```

3. Access the Grafana dashboard at `http://localhost:3000` (default credentials: admin/qitops)

## Monitoring Dashboard

The QitOps Agent dashboard in Grafana provides the following panels:

- **Command Execution Rate**: Rate of command executions over time
- **LLM Request Rate**: Rate of LLM requests over time
- **Command Distribution**: Pie chart showing the distribution of command types
- **LLM Provider Distribution**: Pie chart showing the distribution of LLM providers
- **Command Duration**: Command execution duration (p50 and p95)
- **LLM Request Duration**: LLM request duration (p50 and p95)
- **LLM Token Usage Rate**: Rate of token usage over time
- **Error Rate**: Rate of errors over time
- **Process Memory Usage**: Current memory usage of the QitOps Agent process
- **Process CPU Usage**: Current CPU usage of the QitOps Agent process
- **Total Commands**: Total number of commands executed
- **Total LLM Requests**: Total number of LLM requests

## Custom Metrics Collection

If you want to add custom metrics to your QitOps Agent plugins or extensions, you can use the monitoring module:

```rust
use qitops_agent::monitoring::{track_command, track_llm_request, track_error, Timer};

// Track a command execution
track_command("my-custom-command");

// Track an LLM request
track_llm_request("my-custom-provider");

// Track an error
track_error("my-custom-error");

// Track execution time
let timer = Timer::new("my-custom-operation");
// ... perform operation ...
timer.stop();
```

## Troubleshooting

If you encounter issues with the monitoring system, check the following:

1. **Metrics Endpoint Not Accessible**: Ensure that the host and port are correctly configured and that there are no firewall rules blocking access.

2. **No Metrics Showing in Grafana**: Check that Prometheus is correctly scraping the metrics endpoint and that the Grafana datasource is correctly configured.

3. **High Resource Usage**: If the monitoring system is consuming too many resources, consider increasing the metrics collection interval or disabling some metrics.

## Security Considerations

The metrics endpoint does not require authentication by default. If you expose it to the network, consider:

1. Using a reverse proxy with authentication
2. Restricting access using firewall rules
3. Running the monitoring stack in a private network
