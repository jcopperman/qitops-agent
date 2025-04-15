# QitOps Agent Monitoring

This document explains how to set up and use the Docker-based monitoring system for QitOps Agent.

## Overview

QitOps Agent includes a comprehensive monitoring system that provides insights into its usage, performance, and resource consumption. The monitoring system consists of:

- **Prometheus**: For metrics collection and storage
- **Grafana**: For visualization and dashboards
- **Loki**: For log aggregation
- **Promtail**: For log collection
- **Push Gateway**: For pushing metrics from QitOps Agent

## Prerequisites

- Docker and Docker Compose installed
- Basic understanding of Docker and containerization

## Quick Start

### Starting the Monitoring Stack

#### Windows
```
start-monitoring.bat
```

#### Linux/macOS
```
chmod +x start-monitoring.sh
./start-monitoring.sh
```

This will:
1. Start the monitoring stack (Prometheus, Grafana, Loki, Promtail, Push Gateway)
2. Start a metrics pusher that collects system metrics and pushes them to Prometheus

### Stopping the Monitoring Stack

#### Windows
```
stop-monitoring.bat
```

#### Linux/macOS
```
chmod +x stop-monitoring.sh
./stop-monitoring.sh
```

## Accessing the Monitoring Tools

- **Grafana**: http://localhost:3000 (default credentials: admin/qitops)
- **Prometheus**: http://localhost:9090
- **Push Gateway**: http://localhost:9091

## Manual Setup

If you prefer to set up the monitoring stack manually, you can use the following commands:

```bash
# Start the monitoring stack
docker-compose -f docker-compose.monitoring.yml up -d

# Push metrics manually
# Windows
powershell -File push-metrics.ps1

# Linux/macOS
chmod +x push-metrics.sh
./push-metrics.sh
```

## Customizing the Monitoring Stack

### Changing Ports

If you need to change the default ports, edit the `docker-compose.monitoring.yml` file and update the port mappings.

### Adding Custom Metrics

To add custom metrics, modify the metrics pusher scripts (`push-metrics.ps1` or `push-metrics.sh`) to include your custom metrics.

### Adding Custom Dashboards

1. Create your custom dashboard in Grafana
2. Export the dashboard as JSON
3. Save the JSON file in the `monitoring/grafana/dashboards` directory
4. Update the `monitoring/grafana/provisioning/dashboards/dashboards.yml` file to include your dashboard

## Integrating with QitOps Agent

QitOps Agent can push metrics to the Prometheus Push Gateway. To enable this:

1. Start the monitoring stack
2. In your QitOps Agent code, use the `monitoring` module to push metrics to the Push Gateway

Example:
```rust
use qitops_agent::monitoring;

// Track a command execution
monitoring::track_command("test-gen");

// Track an LLM request
monitoring::track_llm_request("openai");

// Track execution time
let timer = monitoring::Timer::new("command");
// ... perform operation ...
timer.stop();
```

## Troubleshooting

### Monitoring Stack Won't Start

- Check if Docker and Docker Compose are installed and running
- Check if the required ports (3000, 9090, 9091, 3100) are available
- Check the Docker logs: `docker-compose -f docker-compose.monitoring.yml logs`

### Metrics Not Showing Up

- Check if the Push Gateway is running: `docker ps | grep pushgateway`
- Check if the metrics pusher is running
- Check the Prometheus targets: http://localhost:9090/targets

### Grafana Can't Connect to Prometheus

- Check if Prometheus is running: `docker ps | grep prometheus`
- Check the Prometheus logs: `docker-compose -f docker-compose.monitoring.yml logs prometheus`
- Check the Grafana data source configuration
