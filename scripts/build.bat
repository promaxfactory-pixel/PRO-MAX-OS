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

:: Step 1: Install npm dependencies if node_modules does not exist
if not exist "node_modules" (
    echo [1/4] Installing npm dependencies...
    npm install
    if %ERRORLEVEL% neq 0 (
        echo [ERROR] npm install failed with exit code !ERRORLEVEL!
        exit /b !ERRORLEVEL!
    )
    echo [OK] npm dependencies installed successfully.
) else (
    echo [1/4] node_modules already exists. Skipping npm install.
)
echo.

:: Step 2: Run TypeScript type check
echo [2/4] Running TypeScript type check...
npm run build
if %ERRORLEVEL% neq 0 (
    echo [ERROR] TypeScript build/check failed with exit code !ERRORLEVEL!
    exit /b !ERRORLEVEL!
)
echo [OK] TypeScript check passed.
echo.

:: Step 3: Build Tauri Rust application in release mode
echo [3/4] Building Tauri Rust application (release mode)...
cd src-tauri
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Failed to change directory to src-tauri
    exit /b %ERRORLEVEL%
)
cargo build --release
if %ERRORLEVEL% neq 0 (
    echo [ERROR] cargo build --release failed with exit code !ERRORLEVEL!
    cd ..
    exit /b !ERRORLEVEL!
)
cd ..
echo [OK] Tauri release build completed successfully.
echo.

:: Step 4: Display build output directory
echo [4/4] Build output directory:
echo     D:\PRO MAX OS\src-tauri\target\release\
echo.

echo === Build Complete ===
echo Build artifacts are located in: src-tauri\target\release\
endlocal
