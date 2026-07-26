@echo off
:: ============================================================
:: PRO MAX OS - Deployment Script
:: Builds the release version, copies artifacts to a
:: date-stamped releases folder, and creates a zip archive.
:: ============================================================

setlocal EnableDelayedExpansion

:: Generate date stamp in YYYYMMDD format
for /f "tokens=2 delims==" %%I in ('wmic os get localdatetime /value') do set "DATETIME=%%I"
set "DATE_STAMP=!DATETIME:~0,8!"
set "TIME_STAMP=!DATETIME:~8,6!"
set "RELEASE_DIR=releases\!DATE_STAMP!"
set "ZIP_NAME=promaxos-!DATE_STAMP!.zip"

echo === PRO MAX OS - Deployment ===
echo.

:: Step 1: Run release build
echo [1/3] Running release build...
cd src-tauri
cargo build --release
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Release build failed with exit code !ERRORLEVEL!
    cd ..
    exit /b !ERRORLEVEL!
)
cd ..
echo [OK] Release build completed.
echo.

:: Step 2: Copy output to releases folder with date stamp
echo [2/3] Copying release artifacts to !RELEASE_DIR!...
if not exist "releases" mkdir releases
if not exist "!RELEASE_DIR!" mkdir "!RELEASE_DIR!"

:: Copy the built application and related files
if exist "src-tauri\target\release\promaxos.exe" (
    copy "src-tauri\target\release\promaxos.exe" "!RELEASE_DIR!\" >nul
) else (
    echo [WARN] promaxos.exe not found in expected location.
)

if exist "src-tauri\target\release\build" (
    xcopy "src-tauri\target\release\build" "!RELEASE_DIR!\build\" /E /I /Y >nul
)

if exist "src-tauri\target\release\*.dll" (
    copy "src-tauri\target\release\*.dll" "!RELEASE_DIR!\" >nul
)

echo [OK] Release artifacts copied to !RELEASE_DIR!.
echo.

:: Step 3: Create zip archive of the release
echo [3/3] Creating zip archive: !ZIP_NAME!...
powershell -Command "Compress-Archive -Path '!RELEASE_DIR!\*' -DestinationPath 'releases\!ZIP_NAME%' -Force"
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Failed to create zip archive.
    exit /b 1
)
echo [OK] Zip archive created: releases\!ZIP_NAME!
echo.

echo === Deployment Complete ===
echo Release path: releases\!ZIP_NAME!
echo Artifacts directory: !RELEASE_DIR!
endlocal
