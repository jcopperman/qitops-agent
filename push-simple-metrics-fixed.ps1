# QitOps Agent - Push Simple Test Metrics
# This script pushes very simple test metrics to verify the monitoring stack is working

$PushGatewayUrl = "http://localhost:9091"
$JobName = "qitops_simple_test"

Write-Host "Pushing simple test metrics to $PushGatewayUrl" -ForegroundColor Cyan

# Build a very simple metrics payload
$payload = @'
# HELP qitops_test_value A simple test value
# TYPE qitops_test_value gauge
qitops_test_value 42
'@

# Push metrics to the gateway
try {
    $url = "$PushGatewayUrl/metrics/job/$JobName"
    Invoke-RestMethod -Uri $url -Method Post -Body $payload -ContentType "text/plain" -ErrorAction Stop
    Write-Host "Simple test metrics pushed successfully" -ForegroundColor Green
    Write-Host "Now check Grafana at http://localhost:3000" -ForegroundColor Green
}
catch {
    Write-Host "Failed to push simple test metrics: $_" -ForegroundColor Red
}
