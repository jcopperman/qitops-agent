#!/bin/bash
# Start QitOps Agent Monitoring Stack

echo "Starting QitOps Agent Monitoring Stack..."

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "Error: Docker is not installed or not in PATH"
    exit 1
fi

# Check if Docker Compose is installed
if ! command -v docker-compose &> /dev/null; then
    echo "Error: Docker Compose is not installed or not in PATH"
    exit 1
fi

# Create necessary directories
mkdir -p monitoring/prometheus
mkdir -p monitoring/grafana/provisioning/datasources
mkdir -p monitoring/grafana/provisioning/dashboards
mkdir -p monitoring/grafana/dashboards
mkdir -p monitoring/promtail
mkdir -p logs

# Start the monitoring stack
docker-compose -f docker-compose.monitoring.yml up -d

if [ $? -ne 0 ]; then
    echo "Error: Failed to start the monitoring stack"
    exit 1
fi

echo "Monitoring stack started successfully!"
echo
echo "Access Grafana at: http://localhost:3000"
echo "Default credentials: admin/qitops"
echo
echo "Access Prometheus at: http://localhost:9090"
echo
echo "Starting metrics pusher..."

# Make the metrics pusher executable
chmod +x push-metrics.sh

# Start the metrics pusher in a new terminal
if command -v gnome-terminal &> /dev/null; then
    gnome-terminal -- ./push-metrics.sh
elif command -v xterm &> /dev/null; then
    xterm -e "./push-metrics.sh" &
elif command -v konsole &> /dev/null; then
    konsole -e "./push-metrics.sh" &
else
    # Start in background if no terminal emulator is available
    ./push-metrics.sh &
fi

echo
echo "Press Enter to exit..."
read
