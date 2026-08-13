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
pub struct BackupEntry {
    pub id: i64,
    pub file_name: String,
    pub size: i64,
    pub created_at: String,
    pub kind: String,
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
    Err(AppError::not_found("تعذر تحديد مسار قاعدة البيانات"))
}

fn get_backup_dir(conn: &rusqlite::Connection) -> Result<std::path::PathBuf, AppError> {
    let db_path = get_db_path(conn)?;
    let db_file = Path::new(&db_path);
    let dir = db_file
        .parent()
        .ok_or_else(|| AppError::business("تعذر تحديد المجلد الأب لقاعدة البيانات"))?
        .join("backups");
    fs::create_dir_all(&dir)?;
    Ok(dir)
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

fn backup_file_name(ts: u64, kind: &str) -> String {
    format!("backup_{}_{}.db", kind, ts)
}

fn entry_from_path(path: &Path) -> Option<BackupEntry> {
    let file_name = path.file_name()?.to_string_lossy().to_string();
    let base = file_name.trim_end_matches(".db").to_string();
    let id = base
        .split('_')
        .next_back()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);
    let kind = if base.starts_with("backup_auto_") {
        "auto".to_string()
    } else {
        "manual".to_string()
    };
    let metadata = fs::metadata(path).ok()?;
    let created_at = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| format_timestamp(d.as_secs()))
        .unwrap_or_else(|| "unknown".to_string());
    Some(BackupEntry {
        id,
        file_name,
        size: metadata.len() as i64,
        created_at,
        kind,
    })
}

#[tauri::command]
pub fn backup_create(
    state: State<'_, DbState>,
    user_id: i64,
) -> Result<BackupResult, AppError> {
    let conn = state.0.lock()?;
    crate::commands::rbac::require_role(&conn, user_id, &["admin"])?;

    let dir = get_backup_dir(&conn)?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let path = dir.join(backup_file_name(ts, "manual"));
    let path_str = path.to_string_lossy().to_string();

    conn.execute("VACUUM INTO ?", [path_str.clone()])
        .map_err(|e| format!("Failed to create backup: {}", e))?;

    let (version, table_count, record_count) = get_backup_metadata(&conn)?;
    let metadata = fs::metadata(&path)?;
    let created_at = current_timestamp();

    let _ = crate::commands::rbac::log_audit(&conn, Some(user_id), None, "backup_create", "backups", None, None, Some(&path_str), None);
    Ok(BackupResult {
        success: true,
        file_path: path_str,
        file_size: metadata.len() as i64,
        created_at,
        database_version: version,
        table_count,
        record_count,
    })
}

#[tauri::command]
pub fn backup_list(state: State<'_, DbState>) -> Result<Vec<BackupEntry>, AppError> {
    let conn = state.0.lock()?;
    let dir = get_backup_dir(&conn)?;

    let mut entries: Vec<BackupEntry> = Vec::new();
    if let Ok(read_dir) = fs::read_dir(&dir) {
        for entry in read_dir.flatten() {
            let p = entry.path();
            if p.is_file() && p.extension().map(|e| e == "db").unwrap_or(false) {
                if let Some(e) = entry_from_path(&p) {
                    entries.push(e);
                }
            }
        }
    }
    entries.sort_by_key(|a| std::cmp::Reverse(a.id));
    Ok(entries)
}

#[tauri::command]
pub fn backup_restore(
    state: State<'_, DbState>,
    backup_id: i64,
    user_id: i64,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    crate::commands::rbac::require_role(&conn, user_id, &["admin"])?;

    let dir = get_backup_dir(&conn)?;
    // Resolve the backup strictly inside the backups directory (no path traversal).
    let candidates = fs::read_dir(&dir)
        .map_err(|e| AppError::business(format!("Failed to read backups directory: {}", e)))?
        .flatten()
        .filter_map(|e| entry_from_path(&e.path()))
        .filter(|e| e.id == backup_id)
        .collect::<Vec<_>>();

    let entry = candidates
        .first()
        .ok_or_else(|| AppError::not_found("النسخة الاحتياطية غير موجودة"))?;
    let backup_path = dir.join(&entry.file_name);

    if !is_valid_sqlite(&backup_path) {
        return Err(AppError::validation("ملف النسخة الاحتياطية غير صالح: ليست قاعدة بيانات SQLite"));
    }

    let _ = crate::commands::rbac::log_audit(&conn, Some(user_id), None, "backup_restore", "backups", None, None, Some(&entry.file_name), None);

    let db_path = get_db_path(&conn)?;
    drop(conn);

    fs::copy(&backup_path, &db_path).map_err(|e| {
        format!(
            "Failed to restore backup to '{}': {}. The database file may be locked. \
             Close all instances of the application and try again.",
            db_path, e
        )
    })?;

    // Drop stale WAL/SHM so the restored database starts from a clean journal.
    for suffix in ["-wal", "-shm"] {
        let stale = format!("{}{}", db_path, suffix);
        let _ = fs::remove_file(&stale);
    }

    Ok("Backup restored successfully. Please restart the application for changes to take effect.".to_string())
}

#[tauri::command]
pub fn backup_auto(state: State<'_, DbState>) -> Result<BackupResult, AppError> {
    let conn = state.0.lock()?;

    let dir = get_backup_dir(&conn)?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let path = dir.join(backup_file_name(ts, "auto"));
    let path_str = path.to_string_lossy().to_string();

    conn.execute("VACUUM INTO ?", [path_str.clone()])
        .map_err(|e| format!("Failed to create auto backup: {}", e))?;

    let (version, table_count, record_count) = get_backup_metadata(&conn)?;
    let metadata = fs::metadata(&path)?;
    let created_at = current_timestamp();

    let _ = crate::commands::rbac::log_audit(&conn, None, None, "backup_auto", "backups", None, None, Some(&path_str), None);
    Ok(BackupResult {
        success: true,
        file_path: path_str,
        file_size: metadata.len() as i64,
        created_at,
        database_version: version,
        table_count,
        record_count,
    })
}

fn export_table_name(export_type: &str) -> Option<&'static str> {
    match export_type {
        "العملاء" => Some("customers"),
        "المنتجات" => Some("products"),
        "الفواتير" => Some("sales_invoices"),
        "القيود اليومية" => Some("journal_entries"),
        "الموردون" => Some("suppliers"),
        "الموظفون" => Some("employees"),
        "المخزون" => Some("inventory_items"),
        _ => None,
    }
}

#[tauri::command]
pub fn backup_export_csv(
    state: State<'_, DbState>,
    export_type: String,
    user_id: i64,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    crate::commands::rbac::require_role(&conn, user_id, &["admin"])?;

    let table_name = export_table_name(&export_type)
        .ok_or_else(|| AppError::validation(format!("نوع التصدير غير معروف: {}", export_type)))?;

    let tables = get_table_names(&conn)?;
    if !tables.iter().any(|t| t == table_name) {
        return Err(AppError::validation(format!("Table '{}' not found in database", table_name)));
    }

    let db_path = get_db_path(&conn)?;
    let db_file = Path::new(&db_path);
    let export_dir = db_file
        .parent()
        .ok_or_else(|| AppError::business("تعذر تحديد المجلد الأب لقاعدة البيانات"))?


        .join("exports");
    fs::create_dir_all(&export_dir)?;

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let out_path = export_dir.join(format!("{}_{}.csv", table_name, ts));

    let col_query = format!("PRAGMA table_info([{}])", table_name.replace(']', "]]"));
    let mut col_stmt = conn.prepare(&col_query)?;
    let columns: Vec<String> = col_stmt
        .query_map([], |row| row.get::<_, String>(1))
        ?
        .filter_map(|r| r.ok())
        .collect();

    let data_query = format!("SELECT * FROM [{}]", table_name.replace(']', "]]"));
    let mut data_stmt = conn.prepare(&data_query)?;
    let col_count = data_stmt.column_count() as usize;

    let mut file = fs::File::create(&out_path)?;

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
        "Exported {} rows from '{}' to '{}'",
        row_count,
        table_name,
        out_path.to_string_lossy()
    ))
}
