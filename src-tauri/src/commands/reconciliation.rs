use crate::db::DbState;
use crate::error::AppError;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationCheck {
    pub key: String,
    pub account_code: String,
    pub label: String,
    pub gl_balance_milli: i64,
    pub subledger_balance_milli: i64,
    pub timing_items_milli: i64,
    pub unposted_items_milli: i64,
    pub adjusted_subledger_milli: i64,
    pub difference_milli: i64,
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialReconciliation {
    pub checks: Vec<ReconciliationCheck>,
    pub mismatch_count: i64,
    pub review_count: i64,
    pub all_clear: bool,
}

fn gl_asset_balance(conn: &Connection, account_code: &str) -> i64 {
    conn.query_row(
        "SELECT COALESCE(SUM(debit_milli),0) - COALESCE(SUM(credit_milli),0)
         FROM journal_entry_lines WHERE account_code=?1",
        [account_code],
        |r| r.get(0),
    )
    .unwrap_or(0)
}

fn gl_liability_balance(conn: &Connection, account_code: &str) -> i64 {
    conn.query_row(
        "SELECT COALESCE(SUM(credit_milli),0) - COALESCE(SUM(debit_milli),0)
         FROM journal_entry_lines WHERE account_code=?1",
        [account_code],
        |r| r.get(0),
    )
    .unwrap_or(0)
}

fn check(
    key: &str,
    account_code: &str,
    label: &str,
    gl_balance_milli: i64,
    subledger_balance_milli: i64,
    timing_items_milli: i64,
    unposted_items_milli: i64,
    detail: String,
) -> ReconciliationCheck {
    // Timing items have already reduced an operational subledger but have not yet
    // posted to the GL. Unposted legacy items are shown for review but are not
    // silently treated as accounting balances.
    let adjusted_subledger_milli = subledger_balance_milli
        .saturating_add(timing_items_milli)
        .saturating_sub(unposted_items_milli);
    let difference_milli = gl_balance_milli.saturating_sub(adjusted_subledger_milli);
    let status = if difference_milli != 0 {
        "mismatch"
    } else if timing_items_milli != 0 || unposted_items_milli != 0 {
        "review"
    } else {
        "ok"
    };

    ReconciliationCheck {
        key: key.to_string(),
        account_code: account_code.to_string(),
        label: label.to_string(),
        gl_balance_milli,
        subledger_balance_milli,
        timing_items_milli,
        unposted_items_milli,
        adjusted_subledger_milli,
        difference_milli,
        status: status.to_string(),
        detail,
    }
}

pub(crate) fn build_financial_reconciliation(
    conn: &Connection,
) -> Result<FinancialReconciliation, AppError> {
    let mut checks = Vec::new();

    // 1200 — customer control account vs customer subledger running balances.
    let ar_subledger: i64 = conn.query_row(
        "SELECT COALESCE(SUM(balance_milli),0) FROM customers",
        [],
        |r| r.get(0),
    )?;
    checks.push(check(
        "customers_ar",
        "1200",
        "ذمم العملاء",
        gl_asset_balance(conn, "1200"),
        ar_subledger,
        0,
        0,
        "مجموع أرصدة العملاء مقابل حساب العملاء في الأستاذ العام".to_string(),
    ));

    // 2200 — supplier control account vs supplier subledger running balances.
    let ap_subledger: i64 = conn.query_row(
        "SELECT COALESCE(SUM(balance_milli),0) FROM suppliers",
        [],
        |r| r.get(0),
    )?;
    checks.push(check(
        "suppliers_ap",
        "2200",
        "ذمم الموردين",
        gl_liability_balance(conn, "2200"),
        ap_subledger,
        0,
        0,
        "مجموع أرصدة الموردين مقابل حساب الموردين في الأستاذ العام".to_string(),
    ));

    // 1110 — only petty/custody accounts mapped to this GL control account.
    let custody_subledger: i64 = conn.query_row(
        "SELECT COALESCE(SUM(balance_milli),0)
         FROM petty_cash_accounts
         WHERE active=1 AND COALESCE(NULLIF(trim(account_code),''),'1110')='1110'",
        [],
        |r| r.get(0),
    )?;
    checks.push(check(
        "custody",
        "1110",
        "العهدة والصرف النثري",
        gl_asset_balance(conn, "1110"),
        custody_subledger,
        0,
        0,
        "أرصدة العهد النشطة المرتبطة بالحساب 1110".to_string(),
    ));

    // 2250 — approved/accrued payroll still unpaid per employee.
    let payroll_subledger: i64 = conn.query_row(
        "SELECT COALESCE(SUM(MAX(0, prl.net_milli - COALESCE(prl.paid_milli,0))),0)
         FROM payroll_run_lines prl
         JOIN payroll_runs pr ON pr.id=prl.run_id
         WHERE pr.accrual_journal_id IS NOT NULL
           AND LOWER(COALESCE(pr.status,'')) NOT IN ('cancelled','void')",
        [],
        |r| r.get(0),
    )?;
    checks.push(check(
        "payroll_payable",
        "2250",
        "رواتب مستحقة",
        gl_liability_balance(conn, "2250"),
        payroll_subledger,
        0,
        0,
        "صافي الرواتب المعتمدة غير المسددة مقابل حساب الرواتب المستحقة".to_string(),
    ));

    // 2260 — approved personal employee expenses awaiting reimbursement.
    let reimbursements_subledger: i64 = conn.query_row(
        "SELECT COALESCE(SUM(amount_milli + COALESCE(vat_milli,0)),0)
         FROM expenses
         WHERE LOWER(COALESCE(paid_from_source,''))='personal'
           AND LOWER(COALESCE(approval_status,''))='approved'
           AND LOWER(COALESCE(reimbursement_status,''))='pending'",
        [],
        |r| r.get(0),
    )?;
    checks.push(check(
        "employee_reimbursements",
        "2260",
        "مستحقات رد مصروفات الموظفين",
        gl_liability_balance(conn, "2260"),
        reimbursements_subledger,
        0,
        0,
        "مصروفات شخصية معتمدة لم تُرد للموظفين بعد".to_string(),
    ));

    // 1320 — combines salary advances and operating advances. Operating spend is
    // allowed to reduce the operational balance before receipt approval; those
    // submitted linked receipts are a known timing item until their GL entry posts.
    let employee_advances_all: i64 = conn.query_row(
        "SELECT COALESCE(SUM(MAX(remaining_milli,0)),0)
         FROM employee_advances
         WHERE LOWER(COALESCE(status,'open'))!='closed'",
        [],
        |r| r.get(0),
    )?;
    let employee_advances_unposted: i64 = conn.query_row(
        "SELECT COALESCE(SUM(MAX(remaining_milli,0)),0)
         FROM employee_advances
         WHERE journal_id IS NULL
           AND LOWER(COALESCE(status,'open'))!='closed'",
        [],
        |r| r.get(0),
    )?;
    let operating_balance: i64 = conn.query_row(
        "SELECT COALESCE(SUM(MAX(balance_milli,0)),0)
         FROM operating_advances
         WHERE advance_gl_account_code='1320'
           AND disbursed_at IS NOT NULL
           AND LOWER(COALESCE(status,''))!='cancelled'",
        [],
        |r| r.get(0),
    )?;
    let operating_timing: i64 = conn.query_row(
        "SELECT COALESCE(SUM(ar.net_milli),0)
         FROM advance_receipts ar
         JOIN operating_advances oa ON oa.id=ar.advance_id
         WHERE oa.advance_gl_account_code='1320'
           AND LOWER(COALESCE(ar.status,''))='submitted'
           AND ar.transaction_id IS NOT NULL",
        [],
        |r| r.get(0),
    )?;
    checks.push(check(
        "employee_advances",
        "1320",
        "سلف الموظفين وعهد التشغيل",
        gl_asset_balance(conn, "1320"),
        employee_advances_all.saturating_add(operating_balance),
        operating_timing,
        employee_advances_unposted,
        format!(
            "سلف رواتب: {}، عهد تشغيل متبقية: {}، صرف مسجل بانتظار اعتماد مستند: {}، سلف تاريخية غير مرحلة: {}",
            employee_advances_all, operating_balance, operating_timing, employee_advances_unposted
        ),
    ));

    let mismatch_count = checks.iter().filter(|x| x.status == "mismatch").count() as i64;
    let review_count = checks.iter().filter(|x| x.status == "review").count() as i64;

    Ok(FinancialReconciliation {
        all_clear: mismatch_count == 0 && review_count == 0,
        mismatch_count,
        review_count,
        checks,
    })
}

#[tauri::command]
pub fn get_financial_reconciliation(
    state: State<'_, DbState>,
) -> Result<FinancialReconciliation, AppError> {
    let conn = state.0.lock()?;
    build_financial_reconciliation(&conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().expect("memory db");
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        conn.execute_batch(include_str!("../schema.sql")).unwrap();
        conn
    }

    #[test]
    fn empty_database_reconciles_to_zero() {
        let conn = test_conn();
        let report = build_financial_reconciliation(&conn).expect("reconciliation");
        assert_eq!(report.mismatch_count, 0);
        assert_eq!(report.review_count, 0);
        assert!(report.all_clear);
        assert_eq!(report.checks.len(), 6);
    }

    #[test]
    fn customer_subledger_without_gl_is_detected() {
        let conn = test_conn();
        conn.execute(
            "INSERT INTO customers(code,name,balance_milli) VALUES('C-TEST','Test',125000)",
            [],
        )
        .unwrap();
        let report = build_financial_reconciliation(&conn).expect("reconciliation");
        let ar = report
            .checks
            .iter()
            .find(|x| x.key == "customers_ar")
            .expect("AR check");
        assert_eq!(ar.subledger_balance_milli, 125000);
        assert_eq!(ar.gl_balance_milli, 0);
        assert_eq!(ar.difference_milli, -125000);
        assert_eq!(ar.status, "mismatch");
    }

    #[test]
    fn unposted_employee_advance_is_review_item_not_silent_balance() {
        let conn = test_conn();
        conn.execute(
            "INSERT INTO employees(code,name,active) VALUES('E-TEST','Worker',1)",
            [],
        )
        .unwrap();
        let employee_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO employee_advances(employee_id,amount_milli,date,status,remaining_milli,journal_id)
             VALUES(?1,50000,'2026-09-26','open',50000,NULL)",
            [employee_id],
        )
        .unwrap();
        let report = build_financial_reconciliation(&conn).expect("reconciliation");
        let advances = report
            .checks
            .iter()
            .find(|x| x.key == "employee_advances")
            .expect("advance check");
        assert_eq!(advances.unposted_items_milli, 50000);
        assert_eq!(advances.adjusted_subledger_milli, 0);
        assert_eq!(advances.status, "review");
    }
}
