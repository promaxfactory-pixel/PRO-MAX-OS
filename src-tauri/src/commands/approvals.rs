use crate::db::DbState;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApprovalRequest {
    pub id: i64,
    pub request_type: String,
    pub entity_type: String,
    pub entity_id: i64,
    pub entity_number: String,
    pub requested_by: String,
    pub requested_at: String,
    pub amount_milli: Option<i64>,
    pub description: Option<String>,
    pub status: String,
    pub approved_by: Option<String>,
    pub approved_at: Option<String>,
    pub rejection_reason: Option<String>,
    pub priority: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateApprovalInput {
    pub request_type: String,
    pub entity_type: String,
    pub entity_id: i64,
    pub entity_number: String,
    pub requested_by: String,
    pub amount_milli: Option<i64>,
    pub description: Option<String>,
    pub priority: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DecideApprovalInput {
    pub id: i64,
    pub decision: String,
    pub decided_by: String,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ApprovalListFilter {
    pub status: Option<String>,
    pub request_type: Option<String>,
    pub entity_type: Option<String>,
}

#[tauri::command]
pub fn list_approval_requests(
    state: State<'_, DbState>,
    filter: Option<ApprovalListFilter>,
) -> Result<Vec<ApprovalRequest>, AppError> {
    let conn = state.0.lock()?;
    let mut sql = String::from("SELECT id, request_type, entity_type, entity_id, entity_number, requested_by, requested_at, amount_milli, description, status, approved_by, approved_at, rejection_reason, priority FROM approval_requests WHERE 1=1");
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(f) = filter {
        if let Some(s) = &f.status {
            sql.push_str(" AND status = ?");
            params.push(Box::new(s.clone()));
        }
        if let Some(rt) = &f.request_type {
            sql.push_str(" AND request_type = ?");
            params.push(Box::new(rt.clone()));
        }
        if let Some(et) = &f.entity_type {
            sql.push_str(" AND entity_type = ?");
            params.push(Box::new(et.clone()));
        }
    }
    sql.push_str(" ORDER BY CASE priority WHEN 'urgent' THEN 0 WHEN 'high' THEN 1 WHEN 'normal' THEN 2 ELSE 3 END, requested_at DESC");

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(ApprovalRequest {
            id: row.get(0)?,
            request_type: row.get(1)?,
            entity_type: row.get(2)?,
            entity_id: row.get(3)?,
            entity_number: row.get(4)?,
            requested_by: row.get(5)?,
            requested_at: row.get(6)?,
            amount_milli: row.get(7)?,
            description: row.get(8)?,
            status: row.get(9)?,
            approved_by: row.get(10)?,
            approved_at: row.get(11)?,
            rejection_reason: row.get(12)?,
            priority: row.get(13)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect::<Vec<_>>())
}

#[tauri::command]
pub fn create_approval_request(
    state: State<'_, DbState>,
    user_id: i64,
    input: CreateApprovalInput,
) -> Result<i64, AppError> {
    let conn = state.0.lock()?;
    crate::commands::rbac::require_role(&conn, user_id, &["admin", "manager", "accountant"])?;
    let priority = input.priority.unwrap_or_else(|| "normal".to_string());
    let requested_by: String = conn
        .query_row("SELECT username FROM users WHERE id=?", [user_id], |r| r.get(0))
        .unwrap_or_else(|_| input.requested_by.clone());
    conn.execute(
        "INSERT INTO approval_requests (request_type, entity_type, entity_id, entity_number, requested_by, requested_at, amount_milli, description, status, priority) VALUES (?, ?, ?, ?, ?, datetime('now'), ?, ?, 'pending', ?)",
        rusqlite::params![input.request_type, input.entity_type, input.entity_id, input.entity_number, requested_by, input.amount_milli, input.description, priority],
    )?;
    let request_id = conn.last_insert_rowid();
    let _ = crate::commands::rbac::log_audit(&conn, Some(user_id), None, "create_approval_request", "approval_requests", Some(request_id), None, None, None);
    Ok(request_id)
}

#[tauri::command]
pub fn decide_approval(
    state: State<'_, DbState>,
    user_id: i64,
    input: DecideApprovalInput,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    crate::commands::rbac::require_role(&conn, user_id, &["admin", "manager"])?;

    let decided_by: String = conn
        .query_row("SELECT username FROM users WHERE id=?", [user_id], |r| r.get(0))
        .unwrap_or_else(|_| input.decided_by.clone());

    let current_status: String = conn.query_row(
        "SELECT status FROM approval_requests WHERE id = ?", [input.id], |r| r.get(0)
    ).map_err(|_| AppError::not_found("طلب الاعتماد غير موجود"))?;

    if current_status != "pending" {
        return Err(AppError::business(format!("Cannot decide on request with status '{}'", current_status)));
    }

    match input.decision.as_str() {
        "approve" => {
            conn.execute(
                "UPDATE approval_requests SET status = 'approved', approved_by = ?, approved_at = datetime('now') WHERE id = ?",
                rusqlite::params![decided_by, input.id],
            )?;
        }
        "reject" => {
            conn.execute(
                "UPDATE approval_requests SET status = 'rejected', approved_by = ?, approved_at = datetime('now'), rejection_reason = ? WHERE id = ?",
                rusqlite::params![decided_by, input.reason, input.id],
            )?;
        }
        _ => return Err(AppError::validation("القرار يجب أن يكون 'اعتماد' أو 'رفض'")),
    }

    crate::commands::rbac::log_audit(&conn, Some(user_id), None, "decide", "approval_requests", Some(input.id), None, Some(&input.decision), input.reason.as_deref()).ok();
    Ok("Decision recorded".to_string())
}

#[tauri::command]
pub fn get_approval_summary(state: State<'_, DbState>) -> Result<serde_json::Value, AppError> {
    let conn = state.0.lock()?;
    let pending: i64 = conn.query_row("SELECT COUNT(*) FROM approval_requests WHERE status='pending'", [], |r| r.get(0)).unwrap_or(0);
    let approved_today: i64 = conn.query_row("SELECT COUNT(*) FROM approval_requests WHERE status='approved' AND date(approved_at) = date('now')", [], |r| r.get(0)).unwrap_or(0);
    let rejected_today: i64 = conn.query_row("SELECT COUNT(*) FROM approval_requests WHERE status='rejected' AND date(approved_at) = date('now')", [], |r| r.get(0)).unwrap_or(0);
    let total_amount: i64 = conn.query_row("SELECT COALESCE(SUM(amount_milli), 0) FROM approval_requests WHERE status='pending'", [], |r| r.get(0)).unwrap_or(0);

    Ok(serde_json::json!({
        "pending_count": pending,
        "approved_today": approved_today,
        "rejected_today": rejected_today,
        "pending_amount_milli": total_amount,
    }))
}
