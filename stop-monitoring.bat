@echo off
echo Stopping QitOps Agent Monitoring Stack...

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

REM Stop the monitoring stack
docker-compose -f docker-compose.monitoring.yml down

if %ERRORLEVEL% NEQ 0 (
    echo Error: Failed to stop the monitoring stack
    exit /b 1
)

REM Kill any running metrics pusher processes
taskkill /f /im powershell.exe /fi "WINDOWTITLE eq Administrator:*push-metrics*" > nul 2>&1

echo Monitoring stack stopped successfully!
echo.
echo Press any key to exit...
pause > nul
