@echo off
:: ============================================================
:: PRO MAX OS - Test Script
:: Runs the Rust library unit tests via cargo test.
:: ============================================================

setlocal EnableDelayedExpansion

echo === PRO MAX OS - Running Tests ===
echo.

echo [1/1] Running Rust library tests...
cd src-tauri
cargo test --lib
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Tests failed with exit code !ERRORLEVEL!
    cd ..
    exit /b !ERRORLEVEL!
)
cd ..
echo.

echo === All Tests Passed ===
echo Test results summary: All Rust library tests passed successfully.
endlocal
