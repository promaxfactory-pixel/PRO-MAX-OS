use crate::db::DbState;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupResult {
    pub success: bool,
    pub file_path: String,
    pub file_size: i64,
    pub created_at: String,
    pub database_version: i32,
    pub table_count: i32,
    pub record_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupInfo {
    pub file_path: String,
    pub file_name: String,
    pub file_size: i64,
    pub created_at: String,
    pub description: Option<String>,
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn format_timestamp(secs: u64) -> String {
    let days = (secs / 86400) as i64;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    let mut year: i64 = 1970;
    let mut remaining = days;
    loop {
        let diy = if is_leap_year(year) { 366 } else { 365 };
        if remaining < diy {
            break;
        }
        remaining -= diy;
        year += 1;
    }

    let month_days: [i64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month: u32 = 1;
    loop {
        let dim = if month == 2 && is_leap_year(year) {
            29
        } else {
            month_days[(month - 1) as usize]
        };
        if remaining < dim {
            break;
        }
        remaining -= dim;
        month += 1;
    }

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year,
        month,
        remaining + 1,
        hours,
        minutes,
        seconds
    )
}

fn current_timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format_timestamp(secs)
}

fn get_db_path(conn: &rusqlite::Connection) -> Result<String, AppError> {
    let mut stmt = conn
        .prepare("PRAGMA database_list")
        ?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name == "main" {
            let file: String = row.get(2)?;
            return Ok(file);
        }
    }
    Err(AppError::not_found("Could not determine database path"))
}

fn default_backup_dir(conn: &rusqlite::Connection) -> Result<std::path::PathBuf, AppError> {
    let db_path = get_db_path(conn)?;
    let db_file = std::path::Path::new(&db_path);
    db_file
        .parent()
        .map(|p| p.join("backups"))
        .ok_or_else(|| AppError::business("Could not determine database parent directory"))
}

fn create_backup_in(conn: &rusqlite::Connection, backup_dir: &std::path::Path) -> Result<BackupResult, AppError> {
    fs::create_dir_all(backup_dir)?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let backup_filename = format!("backup_{}.db", timestamp);
    let backup_path = backup_dir.join(&backup_filename);
    let backup_path_str = backup_path.to_string_lossy().to_string();

    conn.execute("VACUUM INTO ?", [backup_path_str.clone()])
        .map_err(|e| format!("Failed to create backup: {}", e))?;

    let (version, table_count, record_count) = get_backup_metadata(conn)?;
    let metadata = fs::metadata(&backup_path)?;
    let created_at = current_timestamp();

    Ok(BackupResult {
        success: true,
        file_path: backup_path_str,
        file_size: metadata.len() as i64,
        created_at,
        database_version: version,
        table_count,
        record_count,
    })
}

fn list_backups_in(backup_dir: &std::path::Path) -> Result<Vec<BackupInfo>, AppError> {
    let mut entries: Vec<BackupInfo> = Vec::new();

    let dir_entries = fs::read_dir(backup_dir)?;
    for entry in dir_entries {
        let entry = entry?;
        let entry_path = entry.path();

        if !entry_path.is_file() {
            continue;
        }

        let is_backup = entry_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| matches!(ext, "db" | "sqlite" | "sqlite3" | "backup" | "bak"))
            .unwrap_or(false);

        if !is_backup {
            continue;
        }

        let metadata = fs::metadata(&entry_path)?;
        let file_name = entry_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let created_at = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| format_timestamp(d.as_secs()))
            .unwrap_or_else(|| "unknown".to_string());

        entries.push(BackupInfo {
            file_path: entry_path.to_string_lossy().to_string(),
            file_name,
            file_size: metadata.len() as i64,
            created_at,
            description: None,
        });
    }

    entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(entries)
}

fn is_valid_sqlite(path: &Path) -> bool {
    let mut file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut header = [0u8; 16];
    match std::io::Read::read_exact(&mut file, &mut header) {
        Ok(()) => &header[..16] == b"SQLite format 3\0",
        Err(_) => false,
    }
}

fn get_table_names(conn: &rusqlite::Connection) -> Result<Vec<String>, AppError> {
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'")
        ?;
    let rows = stmt
        .query_map([], |row| row.get(0))
        ?;
    let mut tables = Vec::new();
    for row in rows {
        tables.push(row?);
    }
    Ok(tables)
}

fn count_table_records(conn: &rusqlite::Connection, table: &str) -> i64 {
    let safe_name = table.replace(']', "]]");
    let query = format!("SELECT COUNT(*) FROM [{}]", safe_name);
    conn.query_row(&query, [], |row| row.get(0))
        .unwrap_or(0)
}

fn csv_escape_field(field: &str) -> String {
    if field.contains(',') || field.contains('\n') || field.contains('\r') || field.contains('"') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

fn get_backup_metadata(conn: &rusqlite::Connection) -> Result<(i32, i32, i64), AppError> {
    let version: i32 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        ?;

    let tables = get_table_names(conn)?;
    let table_count = tables.len() as i32;

    let mut record_count: i64 = 0;
    for table in &tables {
        record_count += count_table_records(conn, table);
    }

    Ok((version, table_count, record_count))
}

#[tauri::command]
pub fn backup_create(state: State<'_, DbState>) -> Result<BackupResult, AppError> {
    let conn = state.0.lock()?;
    let backup_dir = default_backup_dir(&conn)?;
    create_backup_in(&conn, &backup_dir)
}

#[tauri::command]
pub fn backup_restore(
    state: State<'_, DbState>,
    backup_path: String,
) -> Result<String, AppError> {
    let path = Path::new(&backup_path);

    if !path.exists() {
        return Err(AppError::not_found(format!("Backup file not found: {}", backup_path)));
    }

    if !is_valid_sqlite(path) {
        return Err(AppError::validation("Invalid backup file: not a valid SQLite database"));
    }

    let conn = state.0.lock()?;
    let db_path = get_db_path(&conn)?;
    drop(conn);

    fs::copy(path, &db_path).map_err(|e| {
        format!(
            "Failed to restore backup to '{}': {}. The database file may be locked. \
             Close all instances of the application and try again, or manually copy '{}' to '{}'.",
            db_path, e, backup_path, db_path
        )
    })?;

    Ok(format!(
        "Backup restored successfully to '{}'. Please restart the application for changes to take effect.",
        db_path
    ))
}

#[tauri::command]
pub fn backup_list(state: State<'_, DbState>) -> Result<Vec<BackupInfo>, AppError> {
    let conn = state.0.lock()?;
    let backup_dir = default_backup_dir(&conn)?;
    list_backups_in(&backup_dir)
}

#[tauri::command]
pub fn backup_get_info(backup_path: String) -> Result<BackupInfo, AppError> {
    let path = Path::new(&backup_path);

    if !path.exists() {
        return Err(AppError::not_found(format!("Backup file not found: {}", backup_path)));
    }

    if !is_valid_sqlite(path) {
        return Err(AppError::validation("Not a valid SQLite backup file"));
    }

    let metadata = fs::metadata(path)?;
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let created_at = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| format_timestamp(d.as_secs()))
        .unwrap_or_else(|| "unknown".to_string());

    let description = if is_valid_sqlite(path) {
        match fs::File::open(path) {
            Ok(mut f) => {
                use std::io::Read;
                let mut buf = [0u8; 100];
                if f.read(&mut buf).is_ok() {
                    let header = String::from_utf8_lossy(&buf[..64]).to_string();
                    Some(format!("SQLite file. Header: {}", header.trim()))
                } else {
                    Some("SQLite file".to_string())
                }
            }
            Err(_) => Some("SQLite file".to_string()),
        }
    } else {
        None
    };

    Ok(BackupInfo {
        file_path: backup_path,
        file_name,
        file_size: metadata.len() as i64,
        created_at,
        description,
    })
}

#[tauri::command]
pub fn backup_auto(state: State<'_, DbState>) -> Result<BackupResult, AppError> {
    let conn = state.0.lock()?;

    let db_path = get_db_path(&conn)?;
    let db_file = Path::new(&db_path);

    let backup_dir = db_file
        .parent()
        .ok_or_else(|| AppError::business("Could not determine database parent directory"))?
        .join("backups");

    fs::create_dir_all(&backup_dir)?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let backup_filename = format!("backup_{}.db", timestamp);
    let backup_path = backup_dir.join(&backup_filename);
    let backup_path_str = backup_path.to_string_lossy().to_string();

    conn.execute("VACUUM INTO ?", [backup_path_str.clone()])
        .map_err(|e| format!("Failed to create auto backup: {}", e))?;

    let (version, table_count, record_count) = get_backup_metadata(&conn)?;
    let metadata = fs::metadata(&backup_path)?;
    let created_at = current_timestamp();

    Ok(BackupResult {
        success: true,
        file_path: backup_path_str,
        file_size: metadata.len() as i64,
        created_at,
        database_version: version,
        table_count,
        record_count,
    })
}

#[tauri::command]
pub fn backup_export_csv(
    state: State<'_, DbState>,
    table_name: String,
    output_path: Option<String>,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;

    let tables = get_table_names(&conn)?;
    if !tables.contains(&table_name) {
        return Err(AppError::validation(format!(
            "Invalid table name '{}'. Available tables: {}",
            table_name,
            tables.join(", ")
        )));
    }

    let out_path = match output_path {
        Some(p) if !p.trim().is_empty() => Path::new(&p).to_path_buf(),
        _ => {
            let backup_dir = default_backup_dir(&conn)?;
            backup_dir.join(format!("export_{}.csv", table_name))
        }
    };

    let out = out_path.as_path();
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }

    let safe_name = table_name.replace(']', "]]");
    let col_query = format!("PRAGMA table_info([{}])", safe_name);
    let mut col_stmt = conn.prepare(&col_query)?;
    let columns: Vec<String> = col_stmt
        .query_map([], |row| row.get::<_, String>(1))
        ?
        .filter_map(|r| r.ok())
        .collect();

    let data_query = format!("SELECT * FROM [{}]", safe_name);
    let mut data_stmt = conn.prepare(&data_query)?;
    let col_count = data_stmt.column_count() as usize;

    let mut file = fs::File::create(out)?;

    let header: Vec<String> = columns.iter().map(|c| csv_escape_field(c)).collect();
    writeln!(file, "{}", header.join(","))?;

    let mut row_count: i64 = 0;
    let rows = data_stmt
        .query_map([], |row| {
            let mut values = Vec::new();
            for i in 0..col_count {
                let val: String = row.get(i).unwrap_or_default();
                values.push(val);
            }
            Ok(values)
        })
        ?;

    for row in rows {
        let values = row?;
        let escaped: Vec<String> = values.iter().map(|v| csv_escape_field(v)).collect();
        writeln!(file, "{}", escaped.join(","))?;
        row_count += 1;
    }

    Ok(format!(
        "Exported {} rows from table '{}' to '{}'",
        row_count, table_name, out_path.display()
    ))
}
