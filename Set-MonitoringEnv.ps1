# QitOps Agent Monitoring Environment Setup (Permanent)
# This script sets the monitoring environment variables permanently for the current user

Write-Host "Setting QitOps Agent monitoring environment variables permanently..." -ForegroundColor Green

# Set environment variables permanently for the current user
[Environment]::SetEnvironmentVariable("QITOPS_MONITORING_ENABLED", "true", "User")
[Environment]::SetEnvironmentVariable("QITOPS_MONITORING_HOST", "127.0.0.1", "User")
[Environment]::SetEnvironmentVariable("QITOPS_MONITORING_PORT", "9090", "User")
[Environment]::SetEnvironmentVariable("QITOPS_MONITORING_INTERVAL", "15", "User")

Write-Host "Environment variables set permanently for the current user" -ForegroundColor Green
Write-Host "You may need to restart your terminal or applications for the changes to take effect" -ForegroundColor Yellow
Write-Host ""
Write-Host "Monitoring will be enabled on 127.0.0.1:9090" -ForegroundColor Cyan
Write-Host "Run 'qitops monitoring status' to check the monitoring status" -ForegroundColor Cyan
