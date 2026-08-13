use crate::commands::rbac;
use crate::db::DbState;
use crate::error::AppError;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct InstallmentPayment {
    pub id: i64,
    pub installment_id: i64,
    pub installment_name: Option<String>,
    pub installment_number: i64,
    pub due_date: Option<String>,
    pub amount_milli: i64,
    pub paid_milli: i64,
    pub paid_date: Option<String>,
    pub penalty_milli: i64,
    pub status: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateInstallmentPaymentInput {
    pub installment_id: i64,
    pub installment_number: i64,
    pub due_date: String,
    pub amount_milli: i64,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstallmentSummary {
    pub installment_id: i64,
    pub installment_name: Option<String>,
    pub total_amount_milli: i64,
    pub total_paid_milli: i64,
    pub remaining_milli: i64,
    pub total_payments: i64,
    pub paid_payments: i64,
    pub pending_payments: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Installment {
    pub id: i64,
    pub name: Option<String>,
    pub total_amount_milli: i64,
    pub number_of_installments: i64,
    pub supplier_name: Option<String>,
    pub status: Option<String>,
}

const PAYMENT_COLUMNS: &str = "p.id, p.installment_id, i.name AS installment_name, p.installment_number, p.due_date, p.amount_milli, p.paid_milli, p.paid_date, p.penalty_milli, p.status, p.notes";

#[tauri::command]
pub fn list_installments(
    state: State<'_, DbState>,
) -> Result<Vec<Installment>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn.prepare(
        "SELECT id, name, original_milli, num_installments, source, status
         FROM installments
         ORDER BY id DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Installment {
            id: row.get(0)?,
            name: row.get(1)?,
            total_amount_milli: row.get(2)?,
            number_of_installments: row.get(3)?,
            supplier_name: row.get(4)?,
            status: row.get(5)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[tauri::command]
pub fn list_installment_payments(
    state: State<'_, DbState>,
) -> Result<Vec<InstallmentPayment>, AppError> {
    let conn = state.0.lock()?;
    let sql = format!(
        "SELECT {} FROM installment_payments p
         LEFT JOIN installments i ON i.id = p.installment_id
         ORDER BY p.installment_id, p.installment_number",
        PAYMENT_COLUMNS
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map([], |row| {
            Ok(InstallmentPayment {
                id: row.get(0)?,
                installment_id: row.get(1)?,
                installment_name: row.get(2)?,
                installment_number: row.get(3)?,
                due_date: row.get(4)?,
                amount_milli: row.get(5)?,
                paid_milli: row.get(6)?,
                paid_date: row.get(7)?,
                penalty_milli: row.get(8)?,
                status: row.get(9)?,
                notes: row.get(10)?,
            })
        })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[tauri::command]
pub fn create_installment_payment(
    state: State<'_, DbState>,
    user_id: i64,
    input: CreateInstallmentPaymentInput,
) -> Result<i64, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "manager", "accountant"])?;

    conn.execute(
        "INSERT INTO installment_payments(installment_id, installment_number, due_date, amount_milli, status, notes) VALUES(?,?,?,?, 'pending', ?)",
        params![
            input.installment_id,
            input.installment_number,
            input.due_date,
            input.amount_milli,
            input.notes,
        ],
    )?;
    let id = conn.last_insert_rowid();
    let _ = rbac::log_audit(&conn, Some(user_id), None, "create_installment_payment", "installment_payments", Some(id), None, None, None);
    Ok(id)
}

#[tauri::command]
pub fn mark_installment_paid(
    state: State<'_, DbState>,
    user_id: i64,
    id: i64,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "manager", "accountant"])?;
    conn.execute(
        "UPDATE installment_payments SET paid_milli = amount_milli, paid_date = datetime('now'), status = 'paid' WHERE id = ?",
        [id],
    )?;
    let _ = rbac::log_audit(&conn, Some(user_id), None, "mark_installment_paid", "installment_payments", Some(id), None, None, None);
    Ok("Payment marked as paid".to_string())
}

#[tauri::command]
pub fn get_installment_summary(
    state: State<'_, DbState>,
    installment_id: i64,
) -> Result<InstallmentSummary, AppError> {
    let conn = state.0.lock()?;
    Ok(conn.query_row(
        "SELECT i.id,
                i.name,
                COALESCE(SUM(p.amount_milli), 0),
                COALESCE(SUM(p.paid_milli), 0),
                COALESCE(SUM(p.amount_milli), 0) - COALESCE(SUM(p.paid_milli), 0),
                COUNT(p.id),
                SUM(CASE WHEN p.status = 'paid' THEN 1 ELSE 0 END),
                SUM(CASE WHEN p.status != 'paid' THEN 1 ELSE 0 END)
         FROM installments i
         LEFT JOIN installment_payments p ON p.installment_id = i.id
         WHERE i.id = ?1
         GROUP BY i.id",
        params![installment_id],
        |row| {
            Ok(InstallmentSummary {
                installment_id: row.get(0)?,
                installment_name: row.get(1)?,
                total_amount_milli: row.get(2)?,
                total_paid_milli: row.get(3)?,
                remaining_milli: row.get(4)?,
                total_payments: row.get(5)?,
                paid_payments: row.get(6)?,
                pending_payments: row.get(7)?,
            })
        },
    )?)
}
