# QitOps Monitoring Visual Guide

This guide provides a visual walkthrough of setting up and using monitoring with QitOps Agent.

## Step 1: Start the Monitoring Stack

Run the start-monitoring script:

```
.\start-monitoring.bat  # Windows
./start-monitoring.sh   # Linux/macOS
```

You should see output indicating that the Docker containers are starting:

```
Starting QitOps Agent Monitoring Stack...
[+] Running 9/9
 ✔ Network qitops-agent_qitops-monitoring    Created
 ✔ Volume "qitops-agent_prometheus_data"     Created
 ✔ Volume "qitops-agent_grafana_data"        Created
 ✔ Volume "qitops-agent_loki_data"           Created
 ✔ Container qitops-agent-qitops-exporter-1  Started
 ✔ Container qitops-agent-prometheus-1       Started
 ✔ Container qitops-agent-loki-1             Started
 ✔ Container qitops-agent-grafana-1          Started
 ✔ Container qitops-agent-promtail-1         Started
Monitoring stack started successfully!
```

## Step 2: Push Test Metrics

Run the test metrics script:

```
.\push-simple-metrics.ps1  # Windows
./push-simple-metrics.sh   # Linux/macOS
```

You should see:

```
Pushing simple test metrics to http://localhost:9091
Simple test metrics pushed successfully
Now check Grafana at http://localhost:3000
```

## Step 3: Access Grafana

Open your browser and go to http://localhost:3000

Log in with:
- Username: `admin`
- Password: `qitops`

![Grafana Login](https://grafana.com/static/img/docs/grafana/login_v10.png)

## Step 4: Create a Simple Dashboard

1. Click on "Dashboards" in the left sidebar
2. Click "New" and then "New Dashboard"
3. Click "Add visualization"
4. Select "Prometheus" as the data source
5. In the query field, enter: `qitops_test_value`
6. Click "Run queries" to see the data
7. Click "Apply" to add the panel to your dashboard
8. Click "Save" to save your dashboard

## Step 5: Run QitOps Commands

Run some QitOps commands to generate real metrics:

```
qitops run test-gen --path ./src
qitops run pr-analyze --pr 123
```

## Step 6: View Command Metrics

1. Go back to your Grafana dashboard
2. Add a new panel
3. Use these queries:
   - `qitops_command_total` (Command execution count)
   - `qitops_command_duration_seconds` (Command execution time)

## Step 7: Stop the Monitoring Stack

When you're done, stop the monitoring stack:

```
.\stop-monitoring.bat  # Windows
./stop-monitoring.sh   # Linux/macOS
```

## Common Issues and Solutions

### No Data in Grafana

If you see "No data" in Grafana:

1. Verify the Push Gateway is running:
   ```
   docker ps | grep qitops-exporter
   ```

2. Try pushing test metrics again:
   ```
   .\push-simple-metrics.ps1
   ```

3. Check Prometheus targets:
   - Go to http://localhost:9090/targets
   - All targets should show as "UP"

### QitOps Command Errors

If QitOps commands fail:

1. For file access errors:
   - Run as administrator
   - Check file permissions

2. For GitHub token errors:
   - Configure your GitHub token:
     ```
     qitops github config --token <your-github-token>
     ```

## Next Steps

- Create custom dashboards for specific metrics
- Set up alerts for important thresholds
- Integrate with your CI/CD pipeline
