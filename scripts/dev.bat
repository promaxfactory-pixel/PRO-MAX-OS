@echo off
:: ============================================================
:: PRO MAX OS - Dev Mode Script
:: Launches the application in development mode with live
:: reload and hot-replacement support.
:: ============================================================

setlocal EnableDelayedExpansion

echo === PRO MAX OS - Dev Mode ===
echo.

echo Starting development server...
npm run tauri dev
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Dev mode exited with error code !ERRORLEVEL!
    exit /b !ERRORLEVEL!
)

echo.
echo === Dev Mode Stopped ===
endlocal
