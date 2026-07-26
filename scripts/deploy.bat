@echo off
:: ============================================================
:: PRO MAX OS - Deployment Script
:: Builds the release version, copies artifacts to a
:: date-stamped releases folder, and creates a zip archive.
:: ============================================================

setlocal EnableDelayedExpansion

:: Resolve project root relative to this script
set "PROJECT_ROOT=%~dp0.."
pushd "%PROJECT_ROOT%"

:: Generate date stamp using PowerShell (works on all Windows versions)
for /f %%I in ('powershell -Command "Get-Date -Format 'yyyyMMdd_HHmmss'"') do set "TIMESTAMP=%%I"
set "RELEASE_DIR=releases\!TIMESTAMP!"
set "ZIP_NAME=promax-os-!TIMESTAMP!.zip"

echo === PRO MAX OS - Deployment ===
echo.

:: Step 1: Run release build
echo [1/3] Running release build...
pushd src-tauri
cargo build --release
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Release build failed with exit code !ERRORLEVEL!
    popd
    popd
    exit /b !ERRORLEVEL!
)
popd
echo [OK] Release build completed.
echo.

:: Step 2: Copy output to releases folder with date stamp
echo [2/3] Copying release artifacts to !RELEASE_DIR!...
if not exist "releases" mkdir releases
if not exist "!RELEASE_DIR!" mkdir "!RELEASE_DIR!"

:: Copy the built application (correct binary name from Cargo.toml)
if exist "src-tauri\target\release\promax-os.exe" (
    copy "src-tauri\target\release\promax-os.exe" "!RELEASE_DIR!\" >nul
    echo [OK] Copied promax-os.exe
) else (
    echo [WARN] promax-os.exe not found in target\release\
)

if exist "src-tauri\target\release\promax-api.exe" (
    copy "src-tauri\target\release\promax-api.exe" "!RELEASE_DIR!\" >nul
    echo [OK] Copied promax-api.exe
)

if exist "src-tauri\target\release\promax-mcp.exe" (
    copy "src-tauri\target\release\promax-mcp.exe" "!RELEASE_DIR!\" >nul
    echo [OK] Copied promax-mcp.exe
)

if exist "src-tauri\target\release\*.dll" (
    copy "src-tauri\target\release\*.dll" "!RELEASE_DIR!\" >nul
)

echo [OK] Release artifacts copied to !RELEASE_DIR!.
echo.

:: Step 3: Create zip archive of the release
echo [3/3] Creating zip archive: !ZIP_NAME!...
powershell -Command "Compress-Archive -Path '!RELEASE_DIR!\*' -DestinationPath 'releases\!ZIP_NAME!' -Force"
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Failed to create zip archive.
    popd
    exit /b 1
)
echo [OK] Zip archive created: releases\!ZIP_NAME!
echo.

echo === Deployment Complete ===
echo Release path: releases\!ZIP_NAME!
echo Artifacts directory: !RELEASE_DIR!

popd
endlocal
