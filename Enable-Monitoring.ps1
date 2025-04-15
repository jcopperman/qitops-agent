# QitOps Agent Monitoring Environment Setup
Write-Host "Loading QitOps Agent monitoring environment variables..." -ForegroundColor Green

# Load variables from .env.monitoring file
$envFile = ".env.monitoring"
if (Test-Path $envFile) {
    Get-Content $envFile | ForEach-Object {
        $line = $_.Trim()
        if ($line -and !$line.StartsWith("#")) {
            $key, $value = $line.Split('=', 2)
            [Environment]::SetEnvironmentVariable($key, $value, "Process")
            Write-Host "Set $key = $value" -ForegroundColor Gray
        }
    }
} else {
    Write-Host "Environment file not found: $envFile" -ForegroundColor Yellow
    
    # Set default values
    [Environment]::SetEnvironmentVariable("QITOPS_MONITORING_ENABLED", "true", "Process")
    [Environment]::SetEnvironmentVariable("QITOPS_MONITORING_HOST", "127.0.0.1", "Process")
    [Environment]::SetEnvironmentVariable("QITOPS_MONITORING_PORT", "9090", "Process")
    [Environment]::SetEnvironmentVariable("QITOPS_MONITORING_INTERVAL", "15", "Process")
    
    Write-Host "Set default monitoring environment variables" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Monitoring enabled on $env:QITOPS_MONITORING_HOST`:$env:QITOPS_MONITORING_PORT" -ForegroundColor Cyan
Write-Host "Run 'qitops monitoring status' to check the monitoring status" -ForegroundColor Cyan
Write-Host ""

# Keep the PowerShell window open
Write-Host "Press Enter to exit..." -ForegroundColor Gray
Read-Host
