# QitOps Agent Monitoring Guide

This guide explains how to use the monitoring features of QitOps Agent.

## Quick Start

1. Start the monitoring stack:
   ```
   .\start-monitoring.bat  # Windows
   ./start-monitoring.sh   # Linux/macOS
   ```

2. Access the dashboards:
   - Grafana: http://localhost:3000 (username: admin, password: qitops)
   - Prometheus: http://localhost:9090

3. Run QitOps commands to generate metrics:
   ```
   qitops run test-gen --path ./src
   qitops run pr-analyze --pr 123
   ```

4. Stop the monitoring stack when done:
   ```
   .\stop-monitoring.bat  # Windows
   ./stop-monitoring.sh   # Linux/macOS
   ```

## What You Can Monitor

### 1. System Metrics

- CPU usage
- Memory usage
- Disk usage

### 2. Command Metrics

- Command execution count
- Command execution time
- Command success/failure rate

### 3. LLM Metrics

- LLM request count by provider
- Token usage
- Response time

## Troubleshooting

### No Data in Grafana

If you see "No data" in Grafana:

1. Check if Prometheus is running:
   ```
   docker ps | grep prometheus
   ```

2. Check if metrics are being pushed:
   ```
   .\push-test-metrics.ps1  # Windows
   ./push-test-metrics.sh   # Linux/macOS
   ```

3. Check Prometheus targets:
   - Go to http://localhost:9090/targets
   - Verify all targets are "UP"

### QitOps Command Errors

If QitOps commands fail:

1. For file access errors:
   - Run the command with administrator privileges
   - Check file permissions

2. For GitHub token errors:
   - Configure your GitHub token:
     ```
     qitops github config --token <your-github-token>
     ```

## Advanced Usage

### Creating Custom Dashboards

1. Log in to Grafana
2. Click "Create" → "Dashboard"
3. Add panels with Prometheus queries
4. Save your dashboard

### Adding Custom Metrics

You can add custom metrics by modifying the metrics pusher scripts:

- `push-metrics.ps1` (Windows)
- `push-metrics.sh` (Linux/macOS)

### Exporting Data

To export metrics data:

1. In Grafana, open the dashboard
2. Click the panel title → "Inspect" → "Data"
3. Click "Download CSV"
