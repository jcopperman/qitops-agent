@echo off
echo Starting QitOps Agent Monitoring Stack...

REM Check if Docker is installed
docker --version > nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo Error: Docker is not installed or not in PATH
    exit /b 1
)

REM Check if Docker Compose is installed
docker-compose --version > nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo Error: Docker Compose is not installed or not in PATH
    exit /b 1
)

REM Create necessary directories
if not exist monitoring\prometheus mkdir monitoring\prometheus
if not exist monitoring\grafana\provisioning\datasources mkdir monitoring\grafana\provisioning\datasources
if not exist monitoring\grafana\provisioning\dashboards mkdir monitoring\grafana\provisioning\dashboards
if not exist monitoring\grafana\dashboards mkdir monitoring\grafana\dashboards
if not exist monitoring\promtail mkdir monitoring\promtail
if not exist logs mkdir logs

REM Start the monitoring stack
docker-compose -f docker-compose.monitoring.yml up -d

if %ERRORLEVEL% NEQ 0 (
    echo Error: Failed to start the monitoring stack
    exit /b 1
)

echo Monitoring stack started successfully!
echo.
echo Access Grafana at: http://localhost:3000
echo Default credentials: admin/qitops
echo.
echo Access Prometheus at: http://localhost:9090
echo.
echo Starting metrics pusher...
start powershell -NoExit -File push-metrics.ps1

echo.
echo Press any key to exit...
pause > nul
