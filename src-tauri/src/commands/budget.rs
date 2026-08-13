use crate::commands::rbac;
use crate::db::DbState;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Budget {
    pub id: i64,
    pub budget_no: String,
    pub name: String,
    pub department: Option<String>,
    pub year: i32,
    pub period: String,
    pub status: String,
    pub total_planned_milli: i64,
    pub total_actual_milli: i64,
    pub variance_milli: i64,
    pub notes: Option<String>,
    pub created_by: Option<String>,
    pub created_at: Option<String>,
    pub approved_by: Option<String>,
    pub approved_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BudgetLine {
    pub id: i64,
    pub budget_id: i64,
    pub category: String,
    pub account_code: Option<String>,
    pub description: Option<String>,
    pub planned_milli: i64,
    pub actual_milli: i64,
    pub variance_milli: i64,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBudgetInput {
    pub name: String,
    pub department: Option<String>,
    pub year: i32,
    pub period: String,
    pub notes: Option<String>,
    pub created_by: Option<String>,
    pub lines: Option<Vec<CreateBudgetLineInput>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBudgetLineInput {
    pub category: String,
    pub account_code: Option<String>,
    pub description: Option<String>,
    pub planned_milli: i64,
    pub notes: Option<String>,
}

#[tauri::command]
pub fn list_budgets(state: State<'_, DbState>) -> Result<Vec<Budget>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn.prepare(
        "SELECT id, budget_no, name, department, year, period, status, total_planned_milli, total_actual_milli, total_planned_milli - total_actual_milli, notes, created_by, created_at, approved_by, approved_at FROM budgets ORDER BY year DESC, budget_no DESC"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Budget {
            id: row.get(0)?,
            budget_no: row.get(1)?,
            name: row.get(2)?,
            department: row.get(3)?,
            year: row.get(4)?,
            period: row.get(5)?,
            status: row.get(6)?,
            total_planned_milli: row.get(7)?,
            total_actual_milli: row.get(8)?,
            variance_milli: row.get(9)?,
            notes: row.get(10)?,
            created_by: row.get(11)?,
            created_at: row.get(12)?,
            approved_by: row.get(13)?,
            approved_at: row.get(14)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect::<Vec<_>>())
}

#[tauri::command]
pub fn get_budget(state: State<'_, DbState>, id: i64) -> Result<Budget, AppError> {
    let conn = state.0.lock()?;
    conn.query_row(
        "SELECT id, budget_no, name, department, year, period, status, total_planned_milli, total_actual_milli, total_planned_milli - total_actual_milli, notes, created_by, created_at, approved_by, approved_at FROM budgets WHERE id = ?",
        [id],
        |row| Ok(Budget {
            id: row.get(0)?,
            budget_no: row.get(1)?,
            name: row.get(2)?,
            department: row.get(3)?,
            year: row.get(4)?,
            period: row.get(5)?,
            status: row.get(6)?,
            total_planned_milli: row.get(7)?,
            total_actual_milli: row.get(8)?,
            variance_milli: row.get(9)?,
            notes: row.get(10)?,
            created_by: row.get(11)?,
            created_at: row.get(12)?,
            approved_by: row.get(13)?,
            approved_at: row.get(14)?,
        }),
    ).map_err(|_| AppError::not_found("الميزانية غير موجودة"))
}

#[tauri::command]
pub fn get_budget_lines(state: State<'_, DbState>, budget_id: i64) -> Result<Vec<BudgetLine>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn.prepare(
        "SELECT id, budget_id, category, account_code, description, planned_milli, actual_milli, planned_milli - actual_milli, notes FROM budget_lines WHERE budget_id = ? ORDER BY category"
    )?;
    let rows = stmt.query_map([budget_id], |row| {
        Ok(BudgetLine {
            id: row.get(0)?,
            budget_id: row.get(1)?,
            category: row.get(2)?,
            account_code: row.get(3)?,
            description: row.get(4)?,
            planned_milli: row.get(5)?,
            actual_milli: row.get(6)?,
            variance_milli: row.get(7)?,
            notes: row.get(8)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect::<Vec<_>>())
}

#[tauri::command]
pub fn create_budget(state: State<'_, DbState>, user_id: i64, input: CreateBudgetInput) -> Result<i64, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;

    let seq: i64 = conn.query_row("SELECT COALESCE(MAX(CAST(SUBSTR(budget_no, 5) AS INTEGER)), 0) + 1 FROM budgets", [], |r| r.get(0)).unwrap_or(1);
    let budget_no = format!("BUD-{}", seq);
    let total_planned: i64 = input.lines.as_ref().map(|l| l.iter().map(|x| x.planned_milli).sum()).unwrap_or(0);

    conn.execute(
        "INSERT INTO budgets (budget_no, name, department, year, period, status, total_planned_milli, total_actual_milli, notes, created_by, created_at) VALUES (?, ?, ?, ?, ?, 'Draft', ?, 0, ?, ?, datetime('now'))",
        rusqlite::params![budget_no, input.name, input.department, input.year, input.period, total_planned, input.notes, input.created_by],
    )?;
    let budget_id = conn.last_insert_rowid();

    if let Some(lines) = &input.lines {
        for line in lines {
            conn.execute(
                "INSERT INTO budget_lines (budget_id, category, account_code, description, planned_milli, actual_milli, notes) VALUES (?, ?, ?, ?, ?, 0, ?)",
                rusqlite::params![budget_id, line.category, line.account_code, line.description, line.planned_milli, line.notes],
            )?;
        }
    }

    let _ = rbac::log_audit(&conn, Some(user_id), None, "create_budget", "budgets", Some(budget_id), None, Some(&budget_no), None);
    Ok(budget_id)
}

#[tauri::command]
pub fn approve_budget(state: State<'_, DbState>, user_id: i64, id: i64, approved_by: String) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    conn.execute(
        "UPDATE budgets SET status = 'Approved', approved_by = ?, approved_at = datetime('now') WHERE id = ? AND status = 'Draft'",
        rusqlite::params![approved_by, id],
    )?;
    let _ = rbac::log_audit(&conn, Some(user_id), None, "approve_budget", "budgets", Some(id), None, Some("Approved"), None);
    Ok("Budget approved".to_string())
}

#[tauri::command]
pub fn update_budget_actuals(state: State<'_, DbState>, user_id: i64, budget_id: i64) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;

    let lines: Vec<(i64, i64)> = {
        let mut stmt = conn.prepare("SELECT id, planned_milli FROM budget_lines WHERE budget_id = ?")?;
        let rows = stmt.query_map([budget_id], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))?;
        rows.filter_map(|r| r.ok()).collect()
    };

    let mut total_actual: i64 = 0;
    for (line_id, _planned) in &lines {
        let actual: i64 = conn.query_row(
            "SELECT COALESCE(SUM(e.amount_milli), 0) FROM expenses e JOIN budget_lines bl ON bl.account_code = e.account_code WHERE bl.id = ? AND bl.budget_id = ? AND strftime('%Y', e.date) = (SELECT CAST(year AS TEXT) FROM budgets WHERE id = ?)",
            rusqlite::params![line_id, budget_id, budget_id],
            |r| r.get(0),
        ).unwrap_or(0);
        conn.execute("UPDATE budget_lines SET actual_milli = ? WHERE id = ?", rusqlite::params![actual, line_id]).ok();
        total_actual += actual;
    }

    conn.execute("UPDATE budgets SET total_actual_milli = ? WHERE id = ?", rusqlite::params![total_actual, budget_id])?;
    let _ = rbac::log_audit(&conn, Some(user_id), None, "update_budget_actuals", "budgets", Some(budget_id), None, Some(&format!("total={}", total_actual)), None);
    Ok(format!("Updated actuals: {} milli", total_actual))
}

#[tauri::command]
pub fn get_budget_vs_actual(state: State<'_, DbState>, budget_id: i64) -> Result<serde_json::Value, AppError> {
    let budget = get_budget(state.clone(), budget_id)?;
    let lines = get_budget_lines(state.clone(), budget_id)?;

    let line_data: Vec<serde_json::Value> = lines.iter().map(|l| {
        let pct = if l.planned_milli > 0 { l.actual_milli as f64 / l.planned_milli as f64 * 100.0 } else { 0.0 };
        serde_json::json!({
            "category": l.category,
            "description": l.description,
            "planned_milli": l.planned_milli,
            "actual_milli": l.actual_milli,
            "variance_milli": l.variance_milli,
            "utilization_pct": (pct * 10.0).round() / 10.0,
        })
    }).collect();

    let total_utilization = if budget.total_planned_milli > 0 {
        (budget.total_actual_milli as f64 / budget.total_planned_milli as f64 * 100.0 * 10.0).round() / 10.0
    } else { 0.0 };

    Ok(serde_json::json!({
        "budget": {
            "id": budget.id,
            "budget_no": budget.budget_no,
            "name": budget.name,
            "year": budget.year,
            "status": budget.status,
        },
        "summary": {
            "total_planned_milli": budget.total_planned_milli,
            "total_actual_milli": budget.total_actual_milli,
            "variance_milli": budget.variance_milli,
            "utilization_pct": total_utilization,
        },
        "lines": line_data,
    }))
}
