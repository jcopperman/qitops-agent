#!/bin/bash
# QitOps Agent Monitoring Environment Setup

echo "Loading QitOps Agent monitoring environment variables..."

# Load variables from .env.monitoring file
if [ -f .env.monitoring ]; then
    export $(grep -v '^#' .env.monitoring | xargs)
else
    echo "Environment file not found: .env.monitoring"
    
    # Set default values
    export QITOPS_MONITORING_ENABLED=true
    export QITOPS_MONITORING_HOST=127.0.0.1
    export QITOPS_MONITORING_PORT=9090
    export QITOPS_MONITORING_INTERVAL=15
    
    echo "Set default monitoring environment variables"
fi

echo ""
echo "Monitoring enabled on $QITOPS_MONITORING_HOST:$QITOPS_MONITORING_PORT"
echo "Run 'qitops monitoring status' to check the monitoring status"
echo ""

# Start a new shell with the environment variables set
exec $SHELL
