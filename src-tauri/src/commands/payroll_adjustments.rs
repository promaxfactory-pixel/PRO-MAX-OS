use crate::commands::rbac;
use crate::db::DbState;
use crate::error::AppError;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tauri::State;

const DEDUCTION_TYPES: [&str; 3] = ["absence", "disciplinary_penalty", "other_deduction"];
const ADDITION_TYPES: [&str; 2] = ["bonus", "allowance"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayrollAdjustment {
    pub id: i64,
    pub employee_id: i64,
    pub employee_name: Option<String>,
    pub adjustment_type: String,
    pub amount_milli: i64,
    pub effective_date: String,
    pub reason: Option<String>,
    pub reference: Option<String>,
    pub notes: Option<String>,
    pub status: String,
    pub approved_by: Option<String>,
    pub approved_at: Option<String>,
    pub applied_run_id: Option<i64>,
    pub applied_at: Option<String>,
    pub created_by: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePayrollAdjustmentInput {
    pub employee_id: i64,
    pub adjustment_type: String,
    pub amount_milli: i64,
    pub effective_date: String,
    pub reason: Option<String>,
    pub reference: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct EmployeePayrollAdjustments {
    pub bonus_milli: i64,
    pub deduction_milli: i64,
    pub advance_deduction_milli: i64,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct RunAdjustmentBreakdown {
    pub absence_deductions_milli: i64,
    pub withholding_deductions_milli: i64,
    pub advance_deductions_milli: i64,
}

pub(crate) fn ensure_schema(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS payroll_adjustments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            employee_id INTEGER NOT NULL REFERENCES employees(id),
            adjustment_type TEXT NOT NULL,
            amount_milli INTEGER NOT NULL CHECK(amount_milli > 0),
            effective_date TEXT NOT NULL,
            reason TEXT,
            reference TEXT,
            notes TEXT,
            status TEXT NOT NULL DEFAULT 'Pending',
            approved_by TEXT,
            approved_at TEXT,
            applied_run_id INTEGER REFERENCES payroll_runs(id),
            applied_at TEXT,
            created_by TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_payroll_adj_employee_date
            ON payroll_adjustments(employee_id, effective_date);
        CREATE INDEX IF NOT EXISTS idx_payroll_adj_status
            ON payroll_adjustments(status, applied_run_id);

        CREATE TABLE IF NOT EXISTS payroll_advance_applications (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            run_id INTEGER NOT NULL REFERENCES payroll_runs(id),
            run_line_id INTEGER NOT NULL REFERENCES payroll_run_lines(id),
            advance_id INTEGER NOT NULL REFERENCES employee_advances(id),
            employee_id INTEGER NOT NULL REFERENCES employees(id),
            amount_milli INTEGER NOT NULL CHECK(amount_milli > 0),
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(run_id, advance_id)
        );
        CREATE INDEX IF NOT EXISTS idx_payroll_adv_app_run
            ON payroll_advance_applications(run_id);
        CREATE INDEX IF NOT EXISTS idx_payroll_adv_app_advance
            ON payroll_advance_applications(advance_id);"
    )?;
    Ok(())
}

fn validate_adjustment_type(value: &str) -> Result<(), AppError> {
    if DEDUCTION_TYPES.contains(&value) || ADDITION_TYPES.contains(&value) {
        Ok(())
    } else {
        Err(AppError::validation("نوع تسوية الراتب غير معروف"))
    }
}

#[tauri::command]
pub fn list_payroll_adjustments(
    state: State<'_, DbState>,
    employee_id: Option<i64>,
) -> Result<Vec<PayrollAdjustment>, AppError> {
    let conn = state.0.lock()?;
    ensure_schema(&conn)?;
    let sql = "SELECT pa.id, pa.employee_id, e.name, pa.adjustment_type, pa.amount_milli,
                      pa.effective_date, pa.reason, pa.reference, pa.notes, pa.status,
                      pa.approved_by, pa.approved_at, pa.applied_run_id, pa.applied_at,
                      pa.created_by, pa.created_at
               FROM payroll_adjustments pa
               LEFT JOIN employees e ON e.id=pa.employee_id
               WHERE (?1 IS NULL OR pa.employee_id=?1)
               ORDER BY pa.effective_date DESC, pa.id DESC";
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(params![employee_id], |row| {
        Ok(PayrollAdjustment {
            id: row.get(0)?,
            employee_id: row.get(1)?,
            employee_name: row.get(2)?,
            adjustment_type: row.get(3)?,
            amount_milli: row.get(4)?,
            effective_date: row.get(5)?,
            reason: row.get(6)?,
            reference: row.get(7)?,
            notes: row.get(8)?,
            status: row.get(9)?,
            approved_by: row.get(10)?,
            approved_at: row.get(11)?,
            applied_run_id: row.get(12)?,
            applied_at: row.get(13)?,
            created_by: row.get(14)?,
            created_at: row.get(15)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[tauri::command]
pub fn create_payroll_adjustment(
    state: State<'_, DbState>,
    user_id: i64,
    input: CreatePayrollAdjustmentInput,
) -> Result<i64, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "hr", "manager"])?;
    ensure_schema(&conn)?;

    let adjustment_type = input.adjustment_type.trim().to_lowercase();
    validate_adjustment_type(&adjustment_type)?;
    if input.amount_milli <= 0 {
        return Err(AppError::validation("قيمة تسوية الراتب يجب أن تكون أكبر من صفر"));
    }
    let date_ok: i64 = conn.query_row(
        "SELECT CASE WHEN date(?1) IS NOT NULL THEN 1 ELSE 0 END",
        [&input.effective_date],
        |r| r.get(0),
    )?;
    if date_ok == 0 {
        return Err(AppError::validation("تاريخ تسوية الراتب غير صحيح"));
    }
    let employee_exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM employees WHERE id=?1 AND active=1",
        [input.employee_id],
        |r| r.get(0),
    )?;
    if employee_exists == 0 {
        return Err(AppError::not_found("العامل غير موجود أو غير نشط"));
    }

    conn.execute(
        "INSERT INTO payroll_adjustments
         (employee_id, adjustment_type, amount_milli, effective_date, reason,
          reference, notes, status, created_by, created_at)
         VALUES(?1,?2,?3,?4,?5,?6,?7,'Pending',?8,datetime('now'))",
        params![
            input.employee_id,
            adjustment_type,
            input.amount_milli,
            input.effective_date,
            input.reason,
            input.reference,
            input.notes,
            user_id.to_string(),
        ],
    )?;
    let id = conn.last_insert_rowid();
    let _ = rbac::log_audit(
        &conn,
        Some(user_id),
        None,
        "create_payroll_adjustment",
        "payroll_adjustments",
        Some(id),
        None,
        Some("Pending"),
        None,
    );
    Ok(id)
}

#[tauri::command]
pub fn approve_payroll_adjustment(
    state: State<'_, DbState>,
    user_id: i64,
    id: i64,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "hr", "manager"])?;
    ensure_schema(&conn)?;
    let changed = conn.execute(
        "UPDATE payroll_adjustments
         SET status='Approved', approved_by=?1, approved_at=datetime('now')
         WHERE id=?2 AND LOWER(status)='pending' AND applied_run_id IS NULL",
        params![user_id.to_string(), id],
    )?;
    if changed == 0 {
        return Err(AppError::validation("التسوية غير موجودة أو تمت معالجتها مسبقًا"));
    }
    let _ = rbac::log_audit(
        &conn,
        Some(user_id),
        None,
        "approve_payroll_adjustment",
        "payroll_adjustments",
        Some(id),
        Some("Pending"),
        Some("Approved"),
        None,
    );
    Ok("Approved".to_string())
}

#[tauri::command]
pub fn reject_payroll_adjustment(
    state: State<'_, DbState>,
    user_id: i64,
    id: i64,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "hr", "manager"])?;
    ensure_schema(&conn)?;
    let changed = conn.execute(
        "UPDATE payroll_adjustments
         SET status='Rejected'
         WHERE id=?1 AND LOWER(status)='pending' AND applied_run_id IS NULL",
        [id],
    )?;
    if changed == 0 {
        return Err(AppError::validation("التسوية غير موجودة أو تمت معالجتها مسبقًا"));
    }
    let _ = rbac::log_audit(
        &conn,
        Some(user_id),
        None,
        "reject_payroll_adjustment",
        "payroll_adjustments",
        Some(id),
        Some("Pending"),
        Some("Rejected"),
        None,
    );
    Ok("Rejected".to_string())
}

pub(crate) fn employee_adjustments_for_run(
    conn: &Connection,
    employee_id: i64,
    period_start: &str,
    period_end: &str,
) -> Result<EmployeePayrollAdjustments, AppError> {
    ensure_schema(conn)?;

    let (bonus_milli, deduction_milli): (i64, i64) = conn.query_row(
        "SELECT
            COALESCE(SUM(CASE WHEN adjustment_type IN ('bonus','allowance') THEN amount_milli ELSE 0 END),0),
            COALESCE(SUM(CASE WHEN adjustment_type IN ('absence','disciplinary_penalty','other_deduction') THEN amount_milli ELSE 0 END),0)
         FROM payroll_adjustments
         WHERE employee_id=?1
           AND effective_date BETWEEN ?2 AND ?3
           AND LOWER(status)='approved'
           AND applied_run_id IS NULL",
        params![employee_id, period_start, period_end],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;

    let advance_deduction_milli: i64 = conn.query_row(
        "SELECT COALESCE(SUM(
            CASE
                WHEN remaining_milli <= 0 OR deduction_per_payroll_milli <= 0 THEN 0
                WHEN remaining_milli < deduction_per_payroll_milli THEN remaining_milli
                ELSE deduction_per_payroll_milli
            END
         ),0)
         FROM employee_advances
         WHERE employee_id=?1
           AND date <= ?2
           AND journal_id IS NOT NULL
           AND LOWER(COALESCE(status,'open'))='open'",
        params![employee_id, period_end],
        |r| r.get(0),
    )?;

    let notes = if bonus_milli == 0 && deduction_milli == 0 && advance_deduction_milli == 0 {
        None
    } else {
        Some(format!(
            "تسويات معتمدة: إضافات={}، خصومات={}، سداد سلف={}",
            bonus_milli, deduction_milli, advance_deduction_milli
        ))
    };

    Ok(EmployeePayrollAdjustments {
        bonus_milli,
        deduction_milli,
        advance_deduction_milli,
        notes,
    })
}

pub(crate) fn validate_and_apply_run(
    conn: &Connection,
    run_id: i64,
    user_id: i64,
) -> Result<RunAdjustmentBreakdown, AppError> {
    ensure_schema(conn)?;
    let (period_start, period_end): (String, String) = conn
        .query_row(
            "SELECT period_start, period_end FROM payroll_runs WHERE id=?1",
            [run_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|_| AppError::not_found("تشغيلة الرواتب غير موجودة"))?;

    let mut expected_bonus = 0_i64;
    let mut expected_deduction = 0_i64;
    let mut expected_advance = 0_i64;
    let mut line_stmt = conn.prepare(
        "SELECT employee_id, bonus_milli, deduction_milli, advance_deduction_milli
         FROM payroll_run_lines WHERE run_id=?1 ORDER BY id",
    )?;
    let line_rows = line_stmt.query_map([run_id], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, i64>(1)?,
            r.get::<_, i64>(2)?,
            r.get::<_, i64>(3)?,
        ))
    })?;
    let lines = line_rows.collect::<Result<Vec<_>, _>>()?;
    for (employee_id, stored_bonus, stored_deduction, stored_advance) in &lines {
        let expected = employee_adjustments_for_run(
            conn,
            *employee_id,
            &period_start,
            &period_end,
        )?;
        if expected.bonus_milli != *stored_bonus
            || expected.deduction_milli != *stored_deduction
            || expected.advance_deduction_milli != *stored_advance
        {
            return Err(AppError::validation(
                "تغيرت تسويات أو سلف أحد العاملين بعد تحضير المسير؛ أعد تحضير المسير قبل الاعتماد",
            ));
        }
        expected_bonus = expected_bonus.saturating_add(expected.bonus_milli);
        expected_deduction = expected_deduction.saturating_add(expected.deduction_milli);
        expected_advance = expected_advance.saturating_add(expected.advance_deduction_milli);
    }

    let stored_bonus_total: i64 = conn.query_row(
        "SELECT COALESCE(SUM(bonus_milli),0) FROM payroll_run_lines WHERE run_id=?1",
        [run_id],
        |r| r.get(0),
    )?;
    let stored_deduction_total: i64 = conn.query_row(
        "SELECT COALESCE(SUM(deduction_milli),0) FROM payroll_run_lines WHERE run_id=?1",
        [run_id],
        |r| r.get(0),
    )?;
    let stored_advance_total: i64 = conn.query_row(
        "SELECT COALESCE(SUM(advance_deduction_milli),0) FROM payroll_run_lines WHERE run_id=?1",
        [run_id],
        |r| r.get(0),
    )?;
    if stored_bonus_total != expected_bonus
        || stored_deduction_total != expected_deduction
        || stored_advance_total != expected_advance
    {
        return Err(AppError::validation("إجماليات تسويات الرواتب تغيرت؛ أعد تحضير المسير"));
    }

    let (absence_deductions_milli, withholding_deductions_milli): (i64, i64) = conn.query_row(
        "SELECT
            COALESCE(SUM(CASE WHEN adjustment_type='absence' THEN amount_milli ELSE 0 END),0),
            COALESCE(SUM(CASE WHEN adjustment_type IN ('disciplinary_penalty','other_deduction') THEN amount_milli ELSE 0 END),0)
         FROM payroll_adjustments
         WHERE effective_date BETWEEN ?1 AND ?2
           AND LOWER(status)='approved'
           AND applied_run_id IS NULL",
        params![period_start, period_end],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;

    conn.execute(
        "UPDATE payroll_adjustments
         SET status='Applied', applied_run_id=?1, applied_at=datetime('now')
         WHERE effective_date BETWEEN ?2 AND ?3
           AND LOWER(status)='approved'
           AND applied_run_id IS NULL",
        params![run_id, period_start, period_end],
    )?;

    for (employee_id, _, _, stored_advance) in lines {
        if stored_advance <= 0 {
            continue;
        }
        let line_id: i64 = conn.query_row(
            "SELECT id FROM payroll_run_lines WHERE run_id=?1 AND employee_id=?2",
            params![run_id, employee_id],
            |r| r.get(0),
        )?;
        let mut remaining_to_apply = stored_advance;
        let mut adv_stmt = conn.prepare(
            "SELECT id, remaining_milli, deduction_per_payroll_milli
             FROM employee_advances
             WHERE employee_id=?1
               AND date <= ?2
               AND journal_id IS NOT NULL
               AND LOWER(COALESCE(status,'open'))='open'
               AND remaining_milli > 0
               AND deduction_per_payroll_milli > 0
             ORDER BY date, id",
        )?;
        let adv_rows = adv_stmt.query_map(params![employee_id, period_end], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?, r.get::<_, i64>(2)?))
        })?;
        let advances = adv_rows.collect::<Result<Vec<_>, _>>()?;
        for (advance_id, remaining, scheduled) in advances {
            if remaining_to_apply <= 0 {
                break;
            }
            let scheduled_amount = remaining.min(scheduled).min(remaining_to_apply);
            if scheduled_amount <= 0 {
                continue;
            }
            conn.execute(
                "INSERT INTO payroll_advance_applications
                 (run_id, run_line_id, advance_id, employee_id, amount_milli, created_at)
                 VALUES(?1,?2,?3,?4,?5,datetime('now'))",
                params![run_id, line_id, advance_id, employee_id, scheduled_amount],
            )?;
            conn.execute(
                "UPDATE employee_advances
                 SET remaining_milli=MAX(0, remaining_milli-?1),
                     status=CASE WHEN remaining_milli-?1 <= 0 THEN 'closed' ELSE 'open' END
                 WHERE id=?2",
                params![scheduled_amount, advance_id],
            )?;
            remaining_to_apply -= scheduled_amount;
        }
        if remaining_to_apply != 0 {
            return Err(AppError::validation(
                "تعذر توزيع خصم السلفة على أرصدة السلف الحالية؛ أعد تحضير المسير",
            ));
        }
    }

    let _ = rbac::log_audit(
        conn,
        Some(user_id),
        None,
        "apply_payroll_adjustments",
        "payroll_runs",
        Some(run_id),
        None,
        Some(&format!(
            "absence={} withholding={} advances={}",
            absence_deductions_milli, withholding_deductions_milli, expected_advance
        )),
        None,
    );

    Ok(RunAdjustmentBreakdown {
        absence_deductions_milli,
        withholding_deductions_milli,
        advance_deductions_milli: expected_advance,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conn() -> Connection {
        let conn = Connection::open_in_memory().expect("memory db");
        conn.execute_batch(include_str!("../schema.sql")).expect("schema");
        ensure_schema(&conn).expect("adjustment schema");
        conn
    }

    #[test]
    fn approved_adjustments_and_scheduled_advance_are_in_snapshot() {
        let conn = conn();
        conn.execute("INSERT INTO employees(code,name,active) VALUES('E1','Worker',1)", []).unwrap();
        let employee_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO payroll_adjustments(employee_id,adjustment_type,amount_milli,effective_date,status)
             VALUES(?1,'absence',5000,'2026-09-24','Approved')",
            [employee_id],
        ).unwrap();
        conn.execute(
            "INSERT INTO employee_advances(employee_id,amount_milli,date,status,remaining_milli,deduction_per_payroll_milli,journal_id)
             VALUES(?1,40000,'2026-09-01','open',40000,10000,1)",
            [employee_id],
        ).unwrap();
        let snap = employee_adjustments_for_run(&conn, employee_id, "2026-09-01", "2026-09-30").unwrap();
        assert_eq!(snap.deduction_milli, 5000);
        assert_eq!(snap.advance_deduction_milli, 10000);
    }

    #[test]
    fn pending_adjustment_is_not_applied_to_snapshot() {
        let conn = conn();
        conn.execute("INSERT INTO employees(code,name,active) VALUES('E1','Worker',1)", []).unwrap();
        let employee_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO payroll_adjustments(employee_id,adjustment_type,amount_milli,effective_date,status)
             VALUES(?1,'disciplinary_penalty',10000,'2026-09-20','Pending')",
            [employee_id],
        ).unwrap();
        let snap = employee_adjustments_for_run(&conn, employee_id, "2026-09-01", "2026-09-30").unwrap();
        assert_eq!(snap.deduction_milli, 0);
    }
}
