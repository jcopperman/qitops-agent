# QitOps Agent - Push Metrics to Prometheus Push Gateway
# This script collects system metrics and pushes them to the Prometheus Push Gateway

param (
    [string]$PushGatewayUrl = "http://localhost:9091",
    [string]$JobName = "qitops_agent",
    [int]$IntervalSeconds = 15
)

function Get-SystemMetrics {
    # Get CPU usage
    $cpuUsage = (Get-Counter '\Processor(_Total)\% Processor Time').CounterSamples.CookedValue
    
    # Get memory usage
    $os = Get-CimInstance Win32_OperatingSystem
    $totalMemory = [math]::Round($os.TotalVisibleMemorySize / 1MB, 2)
    $freeMemory = [math]::Round($os.FreePhysicalMemory / 1MB, 2)
    $usedMemory = $totalMemory - $freeMemory
    $memoryUsagePercent = [math]::Round(($usedMemory / $totalMemory) * 100, 2)
    
    # Get disk usage
    $disk = Get-CimInstance Win32_LogicalDisk -Filter "DeviceID='C:'"
    $totalDiskSpace = [math]::Round($disk.Size / 1GB, 2)
    $freeDiskSpace = [math]::Round($disk.FreeSpace / 1GB, 2)
    $usedDiskSpace = $totalDiskSpace - $freeDiskSpace
    $diskUsagePercent = [math]::Round(($usedDiskSpace / $totalDiskSpace) * 100, 2)
    
    # Return metrics
    return @{
        cpu_usage = $cpuUsage
        memory_total_gb = $totalMemory
        memory_used_gb = $usedMemory
        memory_usage_percent = $memoryUsagePercent
        disk_total_gb = $totalDiskSpace
        disk_used_gb = $usedDiskSpace
        disk_usage_percent = $diskUsagePercent
    }
}

function Push-Metrics {
    param (
        [hashtable]$Metrics
    )
    
    # Build the metrics payload - using invariant culture for numbers
    $payload = ""
    
    foreach ($key in $Metrics.Keys) {
        # Format the value with a period as decimal separator (invariant culture)
        $value = $Metrics[$key].ToString([System.Globalization.CultureInfo]::InvariantCulture)
        $payload += "qitops_system_${key} $value`n"
    }
    
    # Add timestamp metric - use DateTime.Now.ToUnixTimeSeconds() instead of UFormat
    $timestamp = [DateTimeOffset]::Now.ToUnixTimeSeconds()
    $payload += "qitops_system_timestamp $timestamp`n"
    
    # Push metrics to the gateway
    try {
        $url = "$PushGatewayUrl/metrics/job/$JobName"
        Invoke-RestMethod -Uri $url -Method Post -Body $payload -ContentType "text/plain" -ErrorAction Stop
        Write-Host "$(Get-Date) - Metrics pushed successfully" -ForegroundColor Green
    }
    catch {
        Write-Host "$(Get-Date) - Failed to push metrics: $_" -ForegroundColor Red
    }
}

Write-Host "QitOps Agent - Metrics Pusher" -ForegroundColor Cyan
Write-Host "Pushing metrics to $PushGatewayUrl every $IntervalSeconds seconds" -ForegroundColor Cyan
Write-Host "Press Ctrl+C to stop" -ForegroundColor Yellow
Write-Host ""

# Main loop
while ($true) {
    $metrics = Get-SystemMetrics
    Push-Metrics -Metrics $metrics
    
    # Display metrics
    Write-Host "CPU Usage: $($metrics.cpu_usage)%" -ForegroundColor Gray
    Write-Host "Memory Usage: $($metrics.memory_usage_percent)% ($($metrics.memory_used_gb)GB / $($metrics.memory_total_gb)GB)" -ForegroundColor Gray
    Write-Host "Disk Usage: $($metrics.disk_usage_percent)% ($($metrics.disk_used_gb)GB / $($metrics.disk_total_gb)GB)" -ForegroundColor Gray
    Write-Host ""
    
    # Wait for the next interval
    Start-Sleep -Seconds $IntervalSeconds
}
