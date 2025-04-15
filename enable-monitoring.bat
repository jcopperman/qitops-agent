@echo off
echo Loading QitOps Agent monitoring environment variables...

:: Load variables from .env.monitoring file
for /F "tokens=*" %%A in (.env.monitoring) do (
    set line=%%A
    if not "!line:~0,1!"=="#" (
        if not "!line!"=="" (
            set %%A
        )
    )
)

:: Set environment variables
set QITOPS_MONITORING_ENABLED=true
set QITOPS_MONITORING_HOST=127.0.0.1
set QITOPS_MONITORING_PORT=9090
set QITOPS_MONITORING_INTERVAL=15

echo Monitoring enabled on %QITOPS_MONITORING_HOST%:%QITOPS_MONITORING_PORT%
echo Run 'qitops monitoring status' to check the monitoring status
echo.

:: Start a new command prompt with the environment variables set
cmd /k
