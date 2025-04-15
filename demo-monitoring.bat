@echo off
echo QitOps Agent Monitoring Demo
echo ===========================
echo.

echo Step 1: Starting monitoring stack...
call .\start-monitoring.bat
echo.

echo Step 2: Pushing test metrics...
powershell -File push-test-metrics.ps1
echo.

echo Step 3: Opening Grafana dashboard...
start http://localhost:3000/d/qitops-dashboard/qitops-dashboard
echo.

echo Demo is running!
echo.
echo - Grafana: http://localhost:3000 (admin/qitops)
echo - Prometheus: http://localhost:9090
echo - Push Gateway: http://localhost:9091
echo.
echo Press any key to stop the demo and clean up...
pause > nul

echo.
echo Stopping monitoring stack...
call .\stop-monitoring.bat
echo.

echo Demo completed!
echo.
