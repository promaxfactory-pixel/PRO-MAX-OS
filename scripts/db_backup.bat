@echo off
:: ============================================================
:: PRO MAX OS - Database Backup Script
:: Locates promax.db in the application data directory,
:: creates a timestamped backup, and logs the operation.
:: ============================================================

setlocal EnableDelayedExpansion

set "BACKUP_BASE=database\backups"
set "TIMESTAMP=%DATE:~-4%%DATE:~4,2%%DATE:~7,2%_%TIME:~0,2%%TIME:~3,2%%TIME:~6,2%"
set "TIMESTAMP=!TIMESTAMP: =0!"
set "BACKUP_DIR=!BACKUP_BASE!\!TIMESTAMP!"
set "LOG_FILE=database\backup_log.txt"

echo === PRO MAX OS - Database Backup ===
echo.

:: Step 1: Find promax.db in the application data directory
echo [1/3] Locating promax.db...

set "DB_PATH="

:: Check standard AppData locations (Tauri v2 default)
if exist "%APPDATA%\com.promaxos.app\promax.db" (
    set "DB_PATH=%APPDATA%\com.promaxos.app\promax.db"
) else if exist "%LOCALAPPDATA%\com.promaxos.app\promax.db" (
    set "DB_PATH=%LOCALAPPDATA%\com.promaxos.app\promax.db"
) else if exist "%APPDATA%\PROMAX OS\promax.db" (
    set "DB_PATH=%APPDATA%\PROMAX OS\promax.db"
) else if exist "%LOCALAPPDATA%\PROMAX OS\promax.db" (
    set "DB_PATH=%LOCALAPPDATA%\PROMAX OS\promax.db"
) else if exist ".\promax.db" (
    set "DB_PATH=.\promax.db"
)

if "!DB_PATH!"=="" (
    echo [ERROR] promax.db not found.
    echo        Searched:
    echo          %%APPDATA%%\com.promaxos.app\promax.db
    echo          %%LOCALAPPDATA%%\com.promaxos.app\promax.db
    echo          %%APPDATA%%\PROMAX OS\promax.db
    echo          %%LOCALAPPDATA%%\PROMAX OS\promax.db
    echo          .\promax.db
    exit /b 1
)

echo [OK] Found database at: !DB_PATH!
echo.

:: Step 2: Create timestamped backup
echo [2/3] Creating backup in !BACKUP_DIR!...
if not exist "!BACKUP_BASE!" mkdir "!BACKUP_BASE!"
if not exist "!BACKUP_DIR!" mkdir "!BACKUP_DIR!"

set "BACKUP_FILE=!BACKUP_DIR!\promax.db"
copy "!DB_PATH!" "!BACKUP_FILE!" >nul
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Failed to copy database to !BACKUP_FILE!
    exit /b 1
)

:: Get backup file size
for %%F in ("!BACKUP_FILE!") do set "KB_SIZE=%%~zF"
set /a "KB_SIZE=!KB_SIZE! / 1024"

echo [OK] Backup created: !BACKUP_FILE! (!KB_SIZE! KB)
echo.

:: Step 3: Log backup information
echo [3/3] Logging backup info...
if not exist "database" mkdir database

echo Backup Timestamp: !TIMESTAMP! >> "!LOG_FILE!"
echo Source: !DB_PATH! >> "!LOG_FILE!"
echo Destination: !BACKUP_FILE! >> "!LOG_FILE!"
echo Backup Size: !KB_SIZE! KB >> "!LOG_FILE!"
echo --- >> "!LOG_FILE!"

echo [OK] Backup logged to !LOG_FILE!
echo.

echo === Backup Complete ===
echo Backup saved to: !BACKUP_FILE! (!KB_SIZE! KB)
echo Log updated: !LOG_FILE!
endlocal
