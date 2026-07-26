use crate::db::DbState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Notification {
    pub id: i64,
    pub user_id: Option<i64>,
    pub notification_type: String,
    pub title: String,
    pub message: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<i64>,
    pub severity: String,
    pub read_status: String,
    pub action_url: Option<String>,
    pub created_at: String,
    pub read_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNotificationInput {
    pub user_id: Option<i64>,
    pub notification_type: String,
    pub title: String,
    pub message: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<i64>,
    pub severity: Option<String>,
    pub action_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NotificationFilter {
    pub user_id: Option<i64>,
    pub read: Option<bool>,
    pub notification_type: Option<String>,
    pub limit: Option<i64>,
}

#[tauri::command]
pub fn list_notifications(state: State<'_, DbState>, filter: Option<NotificationFilter>) -> Result<Vec<Notification>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut sql = String::from("SELECT id, user_id, notification_type, title, message, entity_type, entity_id, severity, read_status, action_url, created_at, read_at FROM notifications WHERE 1=1");
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(f) = filter {
        if let Some(uid) = f.user_id {
            sql.push_str(" AND (user_id = ? OR user_id IS NULL)");
            params.push(Box::new(uid));
        }
        if let Some(read) = f.read {
            if read { sql.push_str(" AND read_status = 'read'"); }
            else { sql.push_str(" AND read_status = 'unread'"); }
        }
        if let Some(nt) = &f.notification_type {
            sql.push_str(" AND notification_type = ?");
            params.push(Box::new(nt.clone()));
        }
        if let Some(lim) = f.limit {
            sql.push_str(&format!(" LIMIT {}", lim));
        }
    } else {
        sql.push_str(" LIMIT 100");
    }
    sql.push_str(" ORDER BY created_at DESC");

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(Notification {
            id: row.get(0)?, user_id: row.get(1)?, notification_type: row.get(2)?,
            title: row.get(3)?, message: row.get(4)?, entity_type: row.get(5)?,
            entity_id: row.get(6)?, severity: row.get(7)?, read_status: row.get(8)?,
            action_url: row.get(9)?, created_at: row.get(10)?, read_at: row.get(11)?,
        })
    }).map_err(|e| e.to_string())?;
    Ok(rows.filter_map(|r| r.ok()).collect::<Vec<_>>())
}

#[tauri::command]
pub fn create_notification(state: State<'_, DbState>, input: CreateNotificationInput) -> Result<i64, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let severity = input.severity.unwrap_or_else(|| "info".to_string());
    conn.execute(
        "INSERT INTO notifications (user_id, notification_type, title, message, entity_type, entity_id, severity, read_status, action_url, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, 'unread', ?, datetime('now'))",
        rusqlite::params![input.user_id, input.notification_type, input.title, input.message, input.entity_type, input.entity_id, severity, input.action_url],
    ).map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn mark_notification_read(state: State<'_, DbState>, id: i64) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("UPDATE notifications SET read_status = 'read', read_at = datetime('now') WHERE id = ? AND read_status = 'unread'", [id]).map_err(|e| e.to_string())?;
    Ok("Notification marked as read".to_string())
}

#[tauri::command]
pub fn mark_all_notifications_read(state: State<'_, DbState>, user_id: Option<i64>) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    if let Some(uid) = user_id {
        conn.execute("UPDATE notifications SET read_status = 'read', read_at = datetime('now') WHERE (user_id = ? OR user_id IS NULL) AND read_status = 'unread'", [uid]).map_err(|e| e.to_string())?;
    } else {
        conn.execute("UPDATE notifications SET read_status = 'read', read_at = datetime('now') WHERE read_status = 'unread'", []).map_err(|e| e.to_string())?;
    }
    Ok("All notifications marked as read".to_string())
}

#[tauri::command]
pub fn get_notification_count(state: State<'_, DbState>, user_id: Option<i64>) -> Result<serde_json::Value, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let total: i64 = if let Some(uid) = user_id {
        conn.query_row("SELECT COUNT(*) FROM notifications WHERE (user_id = ? OR user_id IS NULL) AND read_status = 'unread'", [uid], |r| r.get(0)).unwrap_or(0)
    } else {
        conn.query_row("SELECT COUNT(*) FROM notifications WHERE read_status = 'unread'", [], |r| r.get(0)).unwrap_or(0)
    };
    let critical: i64 = if let Some(uid) = user_id {
        conn.query_row("SELECT COUNT(*) FROM notifications WHERE (user_id = ? OR user_id IS NULL) AND read_status = 'unread' AND severity = 'critical'", [uid], |r| r.get(0)).unwrap_or(0)
    } else {
        conn.query_row("SELECT COUNT(*) FROM notifications WHERE read_status = 'unread' AND severity = 'critical'", [], |r| r.get(0)).unwrap_or(0)
    };
    Ok(serde_json::json!({
        "unread_count": total,
        "critical_count": critical,
    }))
}
