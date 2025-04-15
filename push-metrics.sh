#!/bin/bash
# QitOps Agent - Push Metrics to Prometheus Push Gateway
# This script collects system metrics and pushes them to the Prometheus Push Gateway

PUSH_GATEWAY_URL=${1:-"http://localhost:9091"}
JOB_NAME=${2:-"qitops_agent"}
INTERVAL_SECONDS=${3:-15}

echo "QitOps Agent - Metrics Pusher"
echo "Pushing metrics to $PUSH_GATEWAY_URL every $INTERVAL_SECONDS seconds"
echo "Press Ctrl+C to stop"
echo ""

# Function to get system metrics
get_system_metrics() {
    # Get CPU usage
    if command -v mpstat &> /dev/null; then
        CPU_USAGE=$(mpstat 1 1 | awk '/Average:/ {print 100 - $NF}')
    else
        CPU_USAGE=$(top -bn1 | grep "Cpu(s)" | sed "s/.*, *\([0-9.]*\)%* id.*/\1/" | awk '{print 100 - $1}')
    fi
    
    # Get memory usage
    MEM_TOTAL=$(free -m | awk '/Mem:/ {print $2}')
    MEM_USED=$(free -m | awk '/Mem:/ {print $3}')
    MEM_USAGE_PERCENT=$(echo "scale=2; $MEM_USED * 100 / $MEM_TOTAL" | bc)
    
    # Get disk usage
    DISK_TOTAL=$(df -m / | awk 'NR==2 {print $2}')
    DISK_USED=$(df -m / | awk 'NR==2 {print $3}')
    DISK_USAGE_PERCENT=$(echo "scale=2; $DISK_USED * 100 / $DISK_TOTAL" | bc)
    
    # Convert to GB
    MEM_TOTAL_GB=$(echo "scale=2; $MEM_TOTAL / 1024" | bc)
    MEM_USED_GB=$(echo "scale=2; $MEM_USED / 1024" | bc)
    DISK_TOTAL_GB=$(echo "scale=2; $DISK_TOTAL / 1024" | bc)
    DISK_USED_GB=$(echo "scale=2; $DISK_USED / 1024" | bc)
}

# Function to push metrics
push_metrics() {
    # Build the metrics payload
    PAYLOAD=""
    PAYLOAD+="qitops_system_cpu_usage $CPU_USAGE\n"
    PAYLOAD+="qitops_system_memory_total_gb $MEM_TOTAL_GB\n"
    PAYLOAD+="qitops_system_memory_used_gb $MEM_USED_GB\n"
    PAYLOAD+="qitops_system_memory_usage_percent $MEM_USAGE_PERCENT\n"
    PAYLOAD+="qitops_system_disk_total_gb $DISK_TOTAL_GB\n"
    PAYLOAD+="qitops_system_disk_used_gb $DISK_USED_GB\n"
    PAYLOAD+="qitops_system_disk_usage_percent $DISK_USAGE_PERCENT\n"
    PAYLOAD+="qitops_system_timestamp $(date +%s)\n"
    
    # Push metrics to the gateway
    echo -e "$PAYLOAD" | curl -s --data-binary @- "$PUSH_GATEWAY_URL/metrics/job/$JOB_NAME" > /dev/null
    
    if [ $? -eq 0 ]; then
        echo "$(date) - Metrics pushed successfully"
    else
        echo "$(date) - Failed to push metrics"
    fi
}

# Main loop
while true; do
    get_system_metrics
    push_metrics
    
    # Display metrics
    echo "CPU Usage: ${CPU_USAGE}%"
    echo "Memory Usage: ${MEM_USAGE_PERCENT}% (${MEM_USED_GB}GB / ${MEM_TOTAL_GB}GB)"
    echo "Disk Usage: ${DISK_USAGE_PERCENT}% (${DISK_USED_GB}GB / ${DISK_TOTAL_GB}GB)"
    echo ""
    
    # Wait for the next interval
    sleep $INTERVAL_SECONDS
done
