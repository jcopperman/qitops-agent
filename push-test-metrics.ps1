# QitOps Agent - Push Test Metrics to Prometheus Push Gateway
# This script pushes test metrics to verify the monitoring stack is working

$PushGatewayUrl = "http://localhost:9091"
$JobName = "qitops_test"

Write-Host "Pushing test metrics to $PushGatewayUrl" -ForegroundColor Cyan

# Build the metrics payload with test data - using single quotes to avoid escaping issues
$payload = @'
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
'@

# Push metrics to the gateway
try {
    $url = "$PushGatewayUrl/metrics/job/$JobName"
    Invoke-RestMethod -Uri $url -Method Post -Body $payload -ContentType "text/plain" -ErrorAction Stop
    Write-Host "Test metrics pushed successfully" -ForegroundColor Green
}
catch {
    Write-Host "Failed to push test metrics: $_" -ForegroundColor Red
}
