@echo off
:: ============================================================
:: PRO MAX OS - Build Script
:: Builds the application for release including:
::   - npm dependency installation
::   - TypeScript type checking
::   - Tauri Rust release build
:: ============================================================

setlocal EnableDelayedExpansion

echo === PRO MAX OS - Building ===
echo.

:: Resolve project root relative to this script
set "PROJECT_ROOT=%~dp0.."
pushd "%PROJECT_ROOT%"

:: Step 1: Install npm dependencies
echo [1/4] Installing npm dependencies...
call npm ci
if %ERRORLEVEL% neq 0 (
    echo [ERROR] npm ci failed with exit code !ERRORLEVEL!
    popd
    exit /b !ERRORLEVEL!
)
echo [OK] npm dependencies installed.
echo.

:: Step 2: Build frontend (includes TypeScript check)
echo [2/4] Building frontend...
call npm run build
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Frontend build failed with exit code !ERRORLEVEL!
    popd
    exit /b !ERRORLEVEL!
)
echo [OK] Frontend build passed.
echo.

:: Step 3: Build Tauri Rust application in release mode
echo [3/4] Building Tauri Rust application (release mode)...
pushd src-tauri
cargo build --release
if %ERRORLEVEL% neq 0 (
    echo [ERROR] cargo build --release failed with exit code !ERRORLEVEL!
    popd
    popd
    exit /b !ERRORLEVEL!
)
popd
echo [OK] Tauri release build completed.
echo.

:: Step 4: Display build output
echo [4/4] Build output:
echo     %PROJECT_ROOT%\src-tauri\target\release\
echo.
echo === Build Complete ===
echo Build artifacts are in: src-tauri\target\release\

popd
endlocal
