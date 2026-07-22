use crate::db::DbState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: i64,
    pub ts: String,
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub action: Option<String>,
    pub entity: Option<String>,
    pub entity_id: Option<i64>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub reason: Option<String>,
}

#[allow(dead_code)]
pub fn require_role(conn: &rusqlite::Connection, user_id: i64, allowed_roles: &[&str]) -> Result<(), String> {
    let role: String = conn
        .query_row(
            "SELECT role FROM users WHERE id = ?1",
            [user_id],
            |row| row.get(0),
        )
        .map_err(|_| "User not found".to_string())?;
    if role == "admin" || allowed_roles.iter().any(|r| *r == role) {
        Ok(())
    } else {
        Err(format!("Access denied. Required role: {:?}, your role: {}", allowed_roles, role))
    }
}

pub fn log_audit(
    conn: &rusqlite::Connection,
    user_id: Option<i64>,
    username: Option<&str>,
    action: &str,
    entity: &str,
    entity_id: Option<i64>,
    old_value: Option<&str>,
    new_value: Option<&str>,
    reason: Option<&str>,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO audit_logs(ts, user_id, username, action, entity, entity_id, old_value, new_value, reason)
         VALUES(datetime('now'), ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            user_id,
            username,
            action,
            entity,
            entity_id,
            old_value,
            new_value,
            reason,
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn list_audit_logs(
    state: State<'_, DbState>,
    entity: Option<String>,
    action: Option<String>,
    user_id: Option<i64>,
    date_from: Option<String>,
    date_to: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<AuditLogEntry>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let limit = limit.unwrap_or(200).min(1000);

    let mut sql = String::from(
        "SELECT id, ts, user_id, username, action, entity, entity_id, old_value, new_value, reason FROM audit_logs WHERE 1=1"
    );
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(ref e) = entity {
        sql.push_str(" AND entity = ?");
        params.push(Box::new(e.clone()));
    }
    if let Some(ref a) = action {
        sql.push_str(" AND action = ?");
        params.push(Box::new(a.clone()));
    }
    if let Some(uid) = user_id {
        sql.push_str(" AND user_id = ?");
        params.push(Box::new(uid));
    }
    if let Some(ref d) = date_from {
        sql.push_str(" AND ts >= ?");
        params.push(Box::new(d.clone()));
    }
    if let Some(ref d) = date_to {
        sql.push_str(" AND ts <= ?");
        params.push(Box::new(format!("{} 23:59:59", d)));
    }
    sql.push_str(" ORDER BY id DESC LIMIT ?");
    params.push(Box::new(limit));

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let rows = stmt
        .query_map(param_refs.as_slice(), |row| {
            Ok(AuditLogEntry {
                id: row.get(0)?,
                ts: row.get(1)?,
                user_id: row.get(2)?,
                username: row.get(3)?,
                action: row.get(4)?,
                entity: row.get(5)?,
                entity_id: row.get(6)?,
                old_value: row.get(7)?,
                new_value: row.get(8)?,
                reason: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut entries = Vec::new();
    for row in rows {
        entries.push(row.map_err(|e| e.to_string())?);
    }
    Ok(entries)
}
