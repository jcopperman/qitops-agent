#!/bin/bash
# QitOps Agent - Push Test Metrics to Prometheus Push Gateway
# This script pushes test metrics to verify the monitoring stack is working

PUSH_GATEWAY_URL="http://localhost:9091"
JOB_NAME="qitops_test"

echo "Pushing test metrics to $PUSH_GATEWAY_URL"

# Build the metrics payload with test data
read -r -d '' PAYLOAD << EOM
# HELP qitops_test_metric A test metric for QitOps
# TYPE qitops_test_metric gauge
qitops_test_metric 42

# HELP qitops_command_total Total number of QitOps commands executed
# TYPE qitops_command_total counter
qitops_command_total{command="test-gen"} 5
qitops_command_total{command="pr-analyze"} 3
qitops_command_total{command="risk"} 2
qitops_command_total{command="test-data"} 1
qitops_command_total{command="session"} 4

# HELP qitops_command_duration_seconds Time spent executing QitOps commands
# TYPE qitops_command_duration_seconds gauge
qitops_command_duration_seconds{command="test-gen"} 2.5
qitops_command_duration_seconds{command="pr-analyze"} 3.2
qitops_command_duration_seconds{command="risk"} 1.8
qitops_command_duration_seconds{command="test-data"} 0.9
qitops_command_duration_seconds{command="session"} 10.5

# HELP qitops_llm_requests_total Total number of LLM requests
# TYPE qitops_llm_requests_total counter
qitops_llm_requests_total{provider="openai"} 15
qitops_llm_requests_total{provider="anthropic"} 8
qitops_llm_requests_total{provider="ollama"} 20

# HELP qitops_system_cpu_usage System CPU usage
# TYPE qitops_system_cpu_usage gauge
qitops_system_cpu_usage 25.5

# HELP qitops_system_memory_usage_percent System memory usage percentage
# TYPE qitops_system_memory_usage_percent gauge
qitops_system_memory_usage_percent 40.2

# HELP qitops_system_disk_usage_percent System disk usage percentage
# TYPE qitops_system_disk_usage_percent gauge
qitops_system_disk_usage_percent 65.8
EOM

# Push metrics to the gateway
if curl -s --data-binary "$PAYLOAD" "$PUSH_GATEWAY_URL/metrics/job/$JOB_NAME"; then
    echo "Test metrics pushed successfully"
else
    echo "Failed to push test metrics"
fi
