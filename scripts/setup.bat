@echo off
:: ============================================================
:: PRO MAX OS - Setup Script
:: Sets up a new developer environment by verifying
:: prerequisites and installing dependencies.
:: ============================================================

setlocal EnableDelayedExpansion

echo === PRO MAX OS - Environment Setup ===
echo.

:: Step 1: Check for Node.js
echo [1/6] Checking Node.js installation...
node --version >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Node.js is not installed. Please install Node.js (LTS) from https://nodejs.org
    exit /b 1
)
for /f "tokens=*" %%i in ('node --version') do set "NODE_VER=%%i"
echo [OK] Node.js found: !NODE_VER!
echo.

:: Step 2: Check for Rust
echo [2/6] Checking Rust installation...
rustc --version >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Rust is not installed. Please install Rust from https://rustup.rs
    exit /b 1
)
for /f "tokens=*" %%i in ('rustc --version') do set "RUST_VER=%%i"
echo [OK] Rust found: !RUST_VER!
echo.

:: Step 3: Check for Git
echo [3/6] Checking Git installation...
git --version >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Git is not installed. Please install Git from https://git-scm.com
    exit /b 1
)
for /f "tokens=*" %%i in ('git --version') do set "GIT_VER=%%i"
echo [OK] Git found: !GIT_VER!
echo.

:: Step 4: Create .env from .env.example if it exists
echo [4/6] Setting up environment file...
if exist ".env.example" (
    if not exist ".env" (
        copy ".env.example" ".env" >nul
        if %ERRORLEVEL% equ 0 (
            echo [OK] Created .env from .env.example
        ) else (
            echo [WARN] Failed to copy .env.example to .env
        )
    ) else (
        echo [OK] .env already exists. Skipping.
    )
) else (
    echo [SKIP] .env.example not found. Skipping environment file setup.
)
echo.

:: Step 5: Install npm dependencies
echo [5/6] Installing npm dependencies...
npm install
if %ERRORLEVEL% neq 0 (
    echo [ERROR] npm install failed with exit code !ERRORLEVEL!
    exit /b !ERRORLEVEL!
)
echo [OK] npm dependencies installed.
echo.

:: Step 6: Run tauri build
echo [6/6] Running initial Tauri build...
npm run tauri build
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Tauri build failed with exit code !ERRORLEVEL!
    exit /b !ERRORLEVEL%
)
echo [OK] Tauri build completed successfully.
echo.

echo === Setup Complete ===
echo Your development environment is ready.
endlocal
