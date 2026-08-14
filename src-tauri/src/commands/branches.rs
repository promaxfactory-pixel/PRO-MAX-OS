use crate::commands::rbac;
use crate::db::DbState;
use crate::error::AppError;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct Branch {
    pub id: i64,
    pub name: String,
    pub code: Option<String>,
    pub address: Option<String>,
    pub is_head_office: bool,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OfflineQueueItem {
    pub id: i64,
    pub branch_id: Option<i64>,
    pub branch_name: Option<String>,
    pub operation: String,
    pub entity: String,
    pub entity_id: Option<i64>,
    pub payload: String,
    pub status: String,
    pub created_at: String,
    pub synced_at: Option<String>,
}

fn row_to_branch(row: &rusqlite::Row) -> rusqlite::Result<Branch> {
    Ok(Branch {
        id: row.get(0)?,
        name: row.get(1)?,
        code: row.get(2)?,
        address: row.get(3)?,
        is_head_office: row.get::<_, i32>(4)? != 0,
        is_active: row.get::<_, i32>(5)? != 0,
        created_at: row.get(6)?,
    })
}

#[tauri::command]
pub fn branches_list(
    state: State<'_, DbState>,
    user_id: i64,
) -> Result<Vec<Branch>, AppError> {
    let conn = state.0.lock()?;
    branches_list_impl(&conn, user_id)
}

pub(crate) fn branches_list_impl(
    conn: &Connection,
    user_id: i64,
) -> Result<Vec<Branch>, AppError> {
    rbac::require_role(conn, user_id, &["admin", "accountant", "manager", "viewer"])?;
    let mut stmt = conn.prepare(
        "SELECT id, name, code, address, is_head_office, is_active, created_at
         FROM branches ORDER BY is_head_office DESC, id ASC",
    )?;
    let rows = stmt.query_map([], row_to_branch)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

#[tauri::command]
pub fn branches_create(
    state: State<'_, DbState>,
    user_id: i64,
    name: String,
    code: Option<String>,
    address: Option<String>,
) -> Result<i64, AppError> {
    let conn = state.0.lock()?;
    branches_create_impl(&conn, user_id, name, code, address)
}

pub(crate) fn branches_create_impl(
    conn: &Connection,
    user_id: i64,
    name: String,
    code: Option<String>,
    address: Option<String>,
) -> Result<i64, AppError> {
    rbac::require_role(conn, user_id, &["admin"])?;
    if name.trim().is_empty() {
        return Err(AppError::business("اسم الفرع مطلوب"));
    }
    conn.execute(
        "INSERT INTO branches(name, code, address, is_head_office, is_active) VALUES(?1, ?2, ?3, 0, 1)",
        rusqlite::params![name.trim(), code, address],
    )?;
    let id = conn.last_insert_rowid();
    let _ = rbac::log_audit(conn, Some(user_id), None, "branches_create", "branches", Some(id), None, Some(&name), None);
    Ok(id)
}

#[tauri::command]
pub fn branches_update(
    state: State<'_, DbState>,
    user_id: i64,
    id: i64,
    name: String,
    code: Option<String>,
    address: Option<String>,
    is_active: bool,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    branches_update_impl(&conn, user_id, id, name, code, address, is_active)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn branches_update_impl(
    conn: &Connection,
    user_id: i64,
    id: i64,
    name: String,
    code: Option<String>,
    address: Option<String>,
    is_active: bool,
) -> Result<String, AppError> {
    rbac::require_role(conn, user_id, &["admin"])?;
    let is_head: i32 = conn
        .query_row("SELECT is_head_office FROM branches WHERE id = ?1", [id], |row| row.get(0))
        .map_err(|_| AppError::not_found("الفرع غير موجود"))?;
    if is_head != 0 && !is_active {
        return Err(AppError::business("لا يمكن تعطيل الفرع الرئيسي"));
    }
    conn.execute(
        "UPDATE branches SET name = ?1, code = ?2, address = ?3, is_active = ?4 WHERE id = ?5",
        rusqlite::params![name, code, address, is_active as i32, id],
    )?;
    let _ = rbac::log_audit(conn, Some(user_id), None, "branches_update", "branches", Some(id), None, Some(&name), None);
    Ok("تم تحديث الفرع".into())
}

#[tauri::command]
pub fn branches_delete(
    state: State<'_, DbState>,
    user_id: i64,
    id: i64,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    branches_delete_impl(&conn, user_id, id)
}

pub(crate) fn branches_delete_impl(
    conn: &Connection,
    user_id: i64,
    id: i64,
) -> Result<String, AppError> {
    rbac::require_role(conn, user_id, &["admin"])?;
    let is_head: i32 = conn
        .query_row("SELECT is_head_office FROM branches WHERE id = ?1", [id], |row| row.get(0))
        .map_err(|_| AppError::not_found("الفرع غير موجود"))?;
    if is_head != 0 {
        return Err(AppError::business("لا يمكن حذف الفرع الرئيسي"));
    }
    conn.execute("DELETE FROM branches WHERE id = ?1", [id])?;
    let _ = rbac::log_audit(conn, Some(user_id), None, "branches_delete", "branches", Some(id), None, None, None);
    Ok("تم حذف الفرع".into())
}

// ---------------------------------------------------------------------------
// Offline-first sync queue
// ---------------------------------------------------------------------------

/// Queue a mutation performed while a branch is offline. The record is
/// replayed (or re-submitted) when connectivity is restored.
#[tauri::command]
pub fn offline_queue_enqueue(
    state: State<'_, DbState>,
    user_id: i64,
    branch_id: Option<i64>,
    operation: String,
    entity: String,
    entity_id: Option<i64>,
    payload: String,
) -> Result<i64, AppError> {
    let conn = state.0.lock()?;
    offline_queue_enqueue_impl(&conn, user_id, branch_id, operation, entity, entity_id, payload)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn offline_queue_enqueue_impl(
    conn: &Connection,
    user_id: i64,
    branch_id: Option<i64>,
    operation: String,
    entity: String,
    entity_id: Option<i64>,
    payload: String,
) -> Result<i64, AppError> {
    rbac::require_role(conn, user_id, &["admin", "accountant", "manager"])?;
    if operation.trim().is_empty() || entity.trim().is_empty() {
        return Err(AppError::business("العملية والكيان مطلوبان"));
    }
    conn.execute(
        "INSERT INTO offline_sync_queue(branch_id, operation, entity, entity_id, payload, status)
         VALUES(?1, ?2, ?3, ?4, ?5, 'pending')",
        rusqlite::params![branch_id, operation, entity, entity_id, payload],
    )?;
    let id = conn.last_insert_rowid();
    let _ = rbac::log_audit(conn, Some(user_id), None, "offline_queue_enqueue", "offline_sync_queue", Some(id), None, Some(&format!("{}:{}", entity, operation)), None);
    Ok(id)
}

#[tauri::command]
pub fn offline_queue_list(
    state: State<'_, DbState>,
    user_id: i64,
    status: Option<String>,
) -> Result<Vec<OfflineQueueItem>, AppError> {
    let conn = state.0.lock()?;
    offline_queue_list_impl(&conn, user_id, status)
}

pub(crate) fn offline_queue_list_impl(
    conn: &Connection,
    user_id: i64,
    status: Option<String>,
) -> Result<Vec<OfflineQueueItem>, AppError> {
    rbac::require_role(conn, user_id, &["admin", "accountant", "manager", "viewer"])?;
    let sql = match status.as_deref() {
        Some(s) if !s.is_empty() => {
            "SELECT q.id, q.branch_id, COALESCE(b.name, ''), q.operation, q.entity, q.entity_id,
                    q.payload, q.status, q.created_at, q.synced_at
             FROM offline_sync_queue q LEFT JOIN branches b ON q.branch_id = b.id
             WHERE q.status = ?1 ORDER BY q.id ASC".to_string()
        }
        _ => {
            "SELECT q.id, q.branch_id, COALESCE(b.name, ''), q.operation, q.entity, q.entity_id,
                    q.payload, q.status, q.created_at, q.synced_at
             FROM offline_sync_queue q LEFT JOIN branches b ON q.branch_id = b.id
             ORDER BY q.id ASC".to_string()
        }
    };
    let mut stmt = conn.prepare(&sql)?;
    fn map_item(row: &rusqlite::Row) -> rusqlite::Result<OfflineQueueItem> {
        Ok(OfflineQueueItem {
            id: row.get(0)?,
            branch_id: row.get(1)?,
            branch_name: row.get(2)?,
            operation: row.get(3)?,
            entity: row.get(4)?,
            entity_id: row.get(5)?,
            payload: row.get(6)?,
            status: row.get(7)?,
            created_at: row.get(8)?,
            synced_at: row.get(9)?,
        })
    }
    let rows = match status.as_deref() {
        Some(s) if !s.is_empty() => stmt.query_map([s], map_item)?,
        _ => stmt.query_map([], map_item)?,
    };
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

/// Mark a queued mutation as synced (applied by the head office / server).
#[tauri::command]
pub fn offline_queue_mark_synced(
    state: State<'_, DbState>,
    user_id: i64,
    id: i64,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    offline_queue_mark_synced_impl(&conn, user_id, id)
}

pub(crate) fn offline_queue_mark_synced_impl(
    conn: &Connection,
    user_id: i64,
    id: i64,
) -> Result<String, AppError> {
    rbac::require_role(conn, user_id, &["admin", "accountant", "manager"])?;
    conn.execute(
        "UPDATE offline_sync_queue SET status = 'synced', synced_at = datetime('now') WHERE id = ?1",
        [id],
    )?;
    let _ = rbac::log_audit(conn, Some(user_id), None, "offline_queue_mark_synced", "offline_sync_queue", Some(id), None, None, None);
    Ok("تمت مزامنة العملية".into())
}

/// Re-queue a failed mutation for retry.
#[tauri::command]
pub fn offline_queue_retry(
    state: State<'_, DbState>,
    user_id: i64,
    id: i64,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    offline_queue_retry_impl(&conn, user_id, id)
}

pub(crate) fn offline_queue_retry_impl(
    conn: &Connection,
    user_id: i64,
    id: i64,
) -> Result<String, AppError> {
    rbac::require_role(conn, user_id, &["admin", "accountant", "manager"])?;
    conn.execute(
        "UPDATE offline_sync_queue SET status = 'pending', synced_at = NULL WHERE id = ?1",
        [id],
    )?;
    let _ = rbac::log_audit(conn, Some(user_id), None, "offline_queue_retry", "offline_sync_queue", Some(id), None, None, None);
    Ok("أُعيدت العملية إلى قائمة الانتظار".into())
}

/// Pending-sync dashboard counters for the offline banner.
#[tauri::command]
pub fn offline_queue_stats(
    state: State<'_, DbState>,
    user_id: i64,
) -> Result<serde_json::Value, AppError> {
    let conn = state.0.lock()?;
    offline_queue_stats_impl(&conn, user_id)
}

pub(crate) fn offline_queue_stats_impl(
    conn: &Connection,
    user_id: i64,
) -> Result<serde_json::Value, AppError> {
    rbac::require_role(conn, user_id, &["admin", "accountant", "manager", "viewer"])?;
    let pending: i64 = conn.query_row(
        "SELECT COUNT(*) FROM offline_sync_queue WHERE status = 'pending'",
        [],
        |row| row.get(0),
    )?;
    let synced: i64 = conn.query_row(
        "SELECT COUNT(*) FROM offline_sync_queue WHERE status = 'synced'",
        [],
        |row| row.get(0),
    )?;
    let failed: i64 = conn.query_row(
        "SELECT COUNT(*) FROM offline_sync_queue WHERE status = 'failed'",
        [],
        |row| row.get(0),
    )?;
    Ok(serde_json::json!({ "pending": pending, "synced": synced, "failed": failed }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(include_str!("../schema.sql")).unwrap();
        conn.execute(
            "INSERT INTO users(username, password_hash, salt, role) VALUES('admin', 'x', 'y', 'admin')",
            [],
        )
        .unwrap();
        conn
    }

    #[test]
    fn branches_crud_and_head_office_protection() {
        let conn = test_db();

        let list = branches_list_impl(&conn, 1).unwrap();
        assert_eq!(list.len(), 1); // seeded head office
        assert!(list[0].is_head_office);

        let id = branches_create_impl(&conn, 1, "فرع المدينة".into(), Some("MD".into()), None).unwrap();
        let list = branches_list_impl(&conn, 1).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[1].code.as_deref(), Some("MD"));

        branches_update_impl(&conn, 1, id, "فرع الرياض".into(), Some("RY".into()), None, false).unwrap();
        let err = branches_delete_impl(&conn, 1, list[0].id).unwrap_err();
        assert!(err.to_string().contains("الرئيسي"));
        branches_delete_impl(&conn, 1, id).unwrap();
        assert_eq!(branches_list_impl(&conn, 1).unwrap().len(), 1);
    }

    #[test]
    fn offline_queue_lifecycle() {
        let conn = test_db();

        let id = offline_queue_enqueue_impl(&conn, 1, Some(1), "create_invoice".into(), "sales_invoices".into(), Some(7), r#"{"total_milli":1000}"#.into()).unwrap();
        let stats = offline_queue_stats_impl(&conn, 1).unwrap();
        assert_eq!(stats["pending"], 1);
        offline_queue_mark_synced_impl(&conn, 1, id).unwrap();
        let stats = offline_queue_stats_impl(&conn, 1).unwrap();
        assert_eq!(stats["pending"], 0);
        assert_eq!(stats["synced"], 1);
        let items = offline_queue_list_impl(&conn, 1, Some("synced".into())).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].entity, "sales_invoices");
    }
}
