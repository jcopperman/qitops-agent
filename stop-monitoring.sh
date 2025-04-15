#!/bin/bash
# Stop QitOps Agent Monitoring Stack

echo "Stopping QitOps Agent Monitoring Stack..."

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

# Stop the monitoring stack
docker-compose -f docker-compose.monitoring.yml down

if [ $? -ne 0 ]; then
    echo "Error: Failed to stop the monitoring stack"
    exit 1
fi

# Kill any running metrics pusher processes
pkill -f "push-metrics.sh" > /dev/null 2>&1

echo "Monitoring stack stopped successfully!"
echo
echo "Press Enter to exit..."
read
