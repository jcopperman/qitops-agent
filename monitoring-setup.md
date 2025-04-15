# QitOps Agent Monitoring Setup

This document explains how to set up and configure monitoring for QitOps Agent.

## Environment Variables

QitOps Agent uses the following environment variables for monitoring configuration:

- `QITOPS_MONITORING_ENABLED`: Enable or disable monitoring (true/false)
- `QITOPS_MONITORING_HOST`: Host to bind the monitoring server to
- `QITOPS_MONITORING_PORT`: Port to bind the monitoring server to
- `QITOPS_MONITORING_INTERVAL`: Metrics collection interval in seconds

## Setup Options

### Option 1: Using the Environment File

1. Edit the `.env.monitoring` file to configure your monitoring settings
2. Use one of the provided scripts to load the environment variables:

#### Windows (Command Prompt)
```
enable-monitoring.bat
```

#### Windows (PowerShell)
```
.\Enable-Monitoring.ps1
```

#### Linux/macOS
```
source ./enable-monitoring.sh
```

### Option 2: Setting Environment Variables Permanently (Windows)

Run the PowerShell script to set the environment variables permanently for the current user:

```
.\Set-MonitoringEnv.ps1
```

You'll need to restart your terminal or applications for the changes to take effect.

### Option 3: Setting Environment Variables Manually

#### Windows (Command Prompt)
```
set QITOPS_MONITORING_ENABLED=true
set QITOPS_MONITORING_HOST=127.0.0.1
set QITOPS_MONITORING_PORT=9090
set QITOPS_MONITORING_INTERVAL=15
```

#### Windows (PowerShell)
```
$env:QITOPS_MONITORING_ENABLED = "true"
$env:QITOPS_MONITORING_HOST = "127.0.0.1"
$env:QITOPS_MONITORING_PORT = "9090"
$env:QITOPS_MONITORING_INTERVAL = "15"
```

#### Linux/macOS
```
export QITOPS_MONITORING_ENABLED=true
export QITOPS_MONITORING_HOST=127.0.0.1
export QITOPS_MONITORING_PORT=9090
export QITOPS_MONITORING_INTERVAL=15
```

## Verifying the Setup

After setting up the environment variables, you can verify that monitoring is enabled:

```
qitops monitoring status
```

## Starting the Monitoring Server

Start the monitoring server:

```
qitops monitoring start
```

Or start with the Docker monitoring stack:

```
qitops monitoring start --docker
```

## Accessing Metrics

- Metrics endpoint: http://127.0.0.1:9090/metrics
- Grafana dashboard (if using Docker): http://localhost:3000 (default credentials: admin/qitops)

## Stopping the Monitoring Server

Stop the monitoring server:

```
qitops monitoring stop
```

Or stop with the Docker monitoring stack:

```
qitops monitoring stop --docker
```
