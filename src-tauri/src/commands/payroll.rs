use crate::commands::rbac;
use crate::db::{next_sequence, DbState};
use crate::error::AppError;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct PayrollRun {
    pub id: i64,
    pub run_no: Option<String>,
    pub period_start: Option<String>,
    pub period_end: Option<String>,
    pub status: Option<String>,
    pub total_gross_milli: i64,
    pub total_deductions_milli: i64,
    pub total_net_milli: i64,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PayrollRunLine {
    pub id: i64,
    pub run_id: i64,
    pub employee_id: i64,
    pub employee_name: String,
    pub basic_milli: i64,
    pub allowance_milli: i64,
    pub overtime_milli: i64,
    pub bonus_milli: i64,
    pub deduction_milli: i64,
    pub advance_deduction_milli: i64,
    pub insurance_deduction_milli: i64,
    pub tax_deduction_milli: i64,
    pub net_milli: i64,
    pub paid_milli: i64,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PayrollPayment {
    pub id: i64,
    pub run_id: i64,
    pub employee_id: Option<i64>,
    pub employee_name: Option<String>,
    pub payment_date: String,
    pub source_type: String,
    pub source_id: Option<i64>,
    pub amount_milli: i64,
    pub method: Option<String>,
    pub reference: Option<String>,
    pub notes: Option<String>,
    pub source_account_code: Option<String>,
    pub wps_status: Option<String>,
    pub wps_reference: Option<String>,
    pub journal_id: Option<i64>,
    pub paid_by: Option<String>,
    pub paid_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePayrollRunInput {
    pub period_start: String,
    pub period_end: String,
}

#[derive(Debug, Deserialize)]
pub struct RecordPayrollPaymentInput {
    pub run_id: i64,
    pub employee_id: i64,
    pub payment_date: String,
    pub amount_milli: i64,
    pub source_type: Option<String>,
    pub source_account_code: Option<String>,
    pub cashbank_id: Option<i64>,
    pub custody_id: Option<i64>,
    pub method: Option<String>,
    pub reference: Option<String>,
    pub notes: Option<String>,
    pub wps_reference: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmployeeAdvance {
    pub id: i64,
    pub employee_id: i64,
    pub employee_name: Option<String>,
    pub amount_milli: i64,
    pub date: String,
    pub reason: Option<String>,
    pub status: Option<String>,
    pub remaining_milli: i64,
    pub deduction_per_payroll_milli: i64,
    pub source_type: Option<String>,
    pub source_id: Option<i64>,
    pub journal_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAdvanceInput {
    pub employee_id: i64,
    pub amount_milli: i64,
    pub date: String,
    pub reason: Option<String>,
    pub deduction_per_payroll_milli: Option<i64>,
    pub source_type: Option<String>,
    pub cashbank_id: Option<i64>,
    pub custody_id: Option<i64>,
    pub method: Option<String>,
}

#[tauri::command]
pub fn list_payroll_runs(state: State<'_, DbState>) -> Result<Vec<PayrollRun>, AppError> {
    crate::commands::licensing::require_feature(crate::commands::licensing::FEAT_PAYROLL)?;
    let conn = state.0.lock()?;
    let mut stmt = conn.prepare(
        "SELECT id, run_no, period_start, period_end, status,
                total_gross_milli, total_deductions_milli, total_net_milli, created_at
         FROM payroll_runs ORDER BY id DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(PayrollRun {
            id: row.get(0)?,
            run_no: row.get(1)?,
            period_start: row.get(2)?,
            period_end: row.get(3)?,
            status: row.get(4)?,
            total_gross_milli: row.get(5)?,
            total_deductions_milli: row.get(6)?,
            total_net_milli: row.get(7)?,
            created_at: row.get(8)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[tauri::command]
pub fn create_payroll_run(
    state: State<'_, DbState>,
    user_id: i64,
    input: CreatePayrollRunInput,
) -> Result<i64, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "hr", "manager"])?;

    let start_ok: i64 = conn.query_row(
        "SELECT CASE WHEN date(?1) IS NOT NULL THEN 1 ELSE 0 END",
        [&input.period_start],
        |r| r.get(0),
    )?;
    let end_ok: i64 = conn.query_row(
        "SELECT CASE WHEN date(?1) IS NOT NULL THEN 1 ELSE 0 END",
        [&input.period_end],
        |r| r.get(0),
    )?;
    if start_ok == 0 || end_ok == 0 || input.period_start > input.period_end {
        return Err(AppError::validation("فترة الرواتب غير صحيحة"));
    }

    let overlap: i64 = conn.query_row(
        "SELECT COUNT(*) FROM payroll_runs
         WHERE LOWER(COALESCE(status,'')) NOT IN ('cancelled','void')
           AND NOT (period_end < ?1 OR period_start > ?2)",
        params![input.period_start, input.period_end],
        |r| r.get(0),
    )?;
    if overlap > 0 {
        return Err(AppError::validation("توجد تشغيلة رواتب متداخلة مع نفس الفترة"));
    }

    let tx = conn.transaction()?;
    let year = input.period_end.get(..4).unwrap_or("0000").to_string();
    let seq = next_sequence(&tx, "PR", &year)?;
    let run_no = format!("PR-{}-{:04}", year, seq);
    tx.execute(
        "INSERT INTO payroll_runs
         (run_no, period_start, period_end, status, total_gross_milli, total_deductions_milli,
          total_net_milli, created_by, created_at)
         VALUES(?1,?2,?3,'Draft',0,0,0,?4,datetime('now'))",
        params![run_no, input.period_start, input.period_end, user_id.to_string()],
    )?;
    let run_id = tx.last_insert_rowid();
    let _ = rbac::log_audit(
        &tx, Some(user_id), None, "create_payroll_run", "payroll_runs",
        Some(run_id), None, Some(&run_no), None
    );
    tx.commit()?;
    Ok(run_id)
}

#[tauri::command]
pub fn prepare_payroll_run(
    state: State<'_, DbState>,
    user_id: i64,
    run_id: i64,
) -> Result<String, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "hr", "manager"])?;
    let tx = conn.transaction()?;

    let (period_start, period_end, status): (String, String, String) = tx.query_row(
        "SELECT period_start, period_end, status FROM payroll_runs WHERE id=?1",
        [run_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    ).map_err(|_| AppError::not_found("تشغيلة الرواتب غير موجودة"))?;

    let status_lower = status.to_lowercase();
    if status_lower != "draft" && status_lower != "prepared" {
        return Err(AppError::validation("لا يمكن إعادة تحضير تشغيلة تم اعتمادها أو دفعها"));
    }

    tx.execute("DELETE FROM payroll_run_lines WHERE run_id=?1", [run_id])?;

    let employees: Vec<(i64, i64, i64, i64, i64, i64, i64, i64)> = {
        let mut stmt = tx.prepare(
            "SELECT id, COALESCE(salary_milli,0), COALESCE(basic_salary_milli,0),
                    COALESCE(allowances_milli,0), COALESCE(housing_allowance_milli,0),
                    COALESCE(transport_allowance_milli,0), COALESCE(food_allowance_milli,0),
                    COALESCE(other_allowances_milli,0)
             FROM employees
             WHERE active=1
             ORDER BY id",
        )?;
        let rows = stmt.query_map([], |r| Ok((
            r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?,
            r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?
        )))?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    if employees.is_empty() {
        return Err(AppError::validation("لا يوجد موظفون نشطون لتحضير الرواتب"));
    }

    let mut total_gross = 0_i64;
    let mut total_deductions = 0_i64;
    let mut total_net = 0_i64;

    for (employee_id, salary, basic_configured, allowances_fallback, housing, transport, food, other) in employees {
        // Avoid double-counting when legacy records store salary_milli as the full salary:
        // detailed allowances are used only when a separate basic salary is configured.
        let basic = if basic_configured > 0 { basic_configured } else { salary };
        let detailed_allowances = housing + transport + food + other;
        let allowances = if basic_configured > 0 {
            if detailed_allowances > 0 { detailed_allowances } else { allowances_fallback }
        } else {
            0
        };

        let overtime_count: i64 = tx.query_row(
            "SELECT COUNT(*) FROM overtime_records
             WHERE employee_id=?1 AND date BETWEEN ?2 AND ?3
               AND approved=1 AND LOWER(COALESCE(status,''))='approved'",
            params![employee_id, period_start, period_end],
            |r| r.get(0),
        ).unwrap_or(0);

        let overtime_rate: i64 = tx.query_row(
            "SELECT CAST(COALESCE(overtime_rate_milli,0) AS INTEGER) FROM employees WHERE id=?1",
            [employee_id],
            |r| r.get(0),
        ).unwrap_or(0);

        if overtime_count > 0 && overtime_rate <= 0 {
            let employee_name: String = tx.query_row(
                "SELECT name FROM employees WHERE id=?1", [employee_id], |r| r.get(0)
            ).unwrap_or_else(|_| format!("#{}", employee_id));
            return Err(AppError::validation(format!(
                "يوجد أوفر تايم معتمد للعامل '{}' لكن أجر الساعة الإضافية غير مضبوط في ملفه",
                employee_name
            )));
        }

        let overtime_milli: i64 = tx.query_row(
            "SELECT COALESCE(SUM(CAST(ROUND(hours * rate_multiplier * ?1) AS INTEGER)),0)
             FROM overtime_records
             WHERE employee_id=?2 AND date BETWEEN ?3 AND ?4
               AND approved=1 AND LOWER(COALESCE(status,''))='approved'",
            params![overtime_rate, employee_id, period_start, period_end],
            |r| r.get(0),
        ).unwrap_or(0);

        let bonus = 0_i64;
        // No deductions are invented automatically. Advances, absences and penalties
        // require an explicit documented/approved payroll adjustment workflow.
        let deduction = 0_i64;
        let advance_deduction = 0_i64;
        let insurance_deduction = 0_i64;
        let tax_deduction = 0_i64;

        let gross = basic + allowances + overtime_milli + bonus;
        let deductions = deduction + advance_deduction + insurance_deduction + tax_deduction;
        let net = gross - deductions;
        if net < 0 {
            return Err(AppError::validation("صافي راتب سالب غير مسموح"));
        }

        tx.execute(
            "INSERT INTO payroll_run_lines
             (run_id, employee_id, basic_milli, allowance_milli, overtime_milli, bonus_milli,
              deduction_milli, advance_deduction_milli, insurance_deduction_milli,
              tax_deduction_milli, net_milli, paid_milli, notes)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,0,NULL)",
            params![
                run_id, employee_id, basic, allowances, overtime_milli, bonus,
                deduction, advance_deduction, insurance_deduction, tax_deduction, net
            ],
        )?;

        total_gross += gross;
        total_deductions += deductions;
        total_net += net;
    }

    tx.execute(
        "UPDATE payroll_runs
         SET status='Prepared', total_gross_milli=?1, total_deductions_milli=?2,
             total_net_milli=?3, processed_by=?4, processed_at=datetime('now')
         WHERE id=?5",
        params![total_gross, total_deductions, total_net, user_id.to_string(), run_id],
    )?;
    let _ = rbac::log_audit(
        &tx, Some(user_id), None, "prepare_payroll_run", "payroll_runs",
        Some(run_id), None, Some(&format!("net={}", total_net)), None
    );
    tx.commit()?;
    Ok("تم تحضير مسير الرواتب من ملفات الموظفين والأوفر تايم المعتمد".to_string())
}

#[tauri::command]
pub fn list_payroll_run_lines(
    state: State<'_, DbState>,
    run_id: i64,
) -> Result<Vec<PayrollRunLine>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn.prepare(
        "SELECT prl.id, prl.run_id, prl.employee_id, e.name,
                prl.basic_milli, prl.allowance_milli, prl.overtime_milli, prl.bonus_milli,
                prl.deduction_milli, prl.advance_deduction_milli,
                prl.insurance_deduction_milli, prl.tax_deduction_milli,
                prl.net_milli, COALESCE(prl.paid_milli,0), prl.notes
         FROM payroll_run_lines prl
         JOIN employees e ON e.id=prl.employee_id
         WHERE prl.run_id=?1 ORDER BY e.name",
    )?;
    let rows = stmt.query_map([run_id], |r| {
        Ok(PayrollRunLine {
            id: r.get(0)?,
            run_id: r.get(1)?,
            employee_id: r.get(2)?,
            employee_name: r.get(3)?,
            basic_milli: r.get(4)?,
            allowance_milli: r.get(5)?,
            overtime_milli: r.get(6)?,
            bonus_milli: r.get(7)?,
            deduction_milli: r.get(8)?,
            advance_deduction_milli: r.get(9)?,
            insurance_deduction_milli: r.get(10)?,
            tax_deduction_milli: r.get(11)?,
            net_milli: r.get(12)?,
            paid_milli: r.get(13)?,
            notes: r.get(14)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[tauri::command]
pub fn approve_payroll_run(
    state: State<'_, DbState>,
    user_id: i64,
    run_id: i64,
) -> Result<String, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    let tx = conn.transaction()?;

    let (period_end, status, accrual_journal, gross, deductions, net):
        (String, String, Option<i64>, i64, i64, i64) = tx.query_row(
        "SELECT period_end, status, accrual_journal_id, total_gross_milli,
                total_deductions_milli, total_net_milli
         FROM payroll_runs WHERE id=?1",
        [run_id],
        |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?)),
    ).map_err(|_| AppError::not_found("تشغيلة الرواتب غير موجودة"))?;

    if status.to_lowercase() != "prepared" {
        return Err(AppError::validation("يجب تحضير ومراجعة المسير قبل الاعتماد"));
    }
    if accrual_journal.is_some() {
        return Err(AppError::validation("تم إثبات استحقاق هذه الرواتب مسبقًا"));
    }
    if gross <= 0 || net < 0 || gross != net + deductions {
        return Err(AppError::validation("إجماليات مسير الرواتب غير متوازنة"));
    }

    let overtime: i64 = tx.query_row(
        "SELECT COALESCE(SUM(overtime_milli),0) FROM payroll_run_lines WHERE run_id=?1",
        [run_id], |r| r.get(0)
    )?;
    let other_payroll = gross - overtime;

    let mut lines: Vec<(String, i64, i64, Option<String>)> = Vec::new();
    if other_payroll > 0 {
        lines.push(("5300".to_string(), other_payroll, 0, Some("رواتب وبدلات".to_string())));
    }
    if overtime > 0 {
        lines.push(("5310".to_string(), overtime, 0, Some("عمل إضافي".to_string())));
    }
    if deductions > 0 {
        lines.push(("2255".to_string(), 0, deductions, Some("خصومات رواتب معلقة للتسوية".to_string())));
    }
    if net > 0 {
        lines.push(("2250".to_string(), 0, net, Some("صافي رواتب مستحقة للعاملين".to_string())));
    }

    let journal_id = crate::commands::accounting::post_to_journal(
        &tx, "payroll_accrual", run_id, &period_end, "إثبات استحقاق الرواتب",
        &lines, &user_id.to_string()
    )?;

    tx.execute(
        "UPDATE payroll_runs
         SET status='Approved', approved_by=?1, approved_at=datetime('now'), accrual_journal_id=?2
         WHERE id=?3",
        params![user_id.to_string(), journal_id, run_id],
    )?;
    let _ = rbac::log_audit(
        &tx, Some(user_id), None, "approve_payroll_run", "payroll_runs",
        Some(run_id), None, Some(&format!("journal={}", journal_id)), None
    );
    tx.commit()?;
    Ok("تم اعتماد المسير وإثبات استحقاق الرواتب محاسبيًا".to_string())
}

#[tauri::command]
pub fn record_payroll_payment(
    state: State<'_, DbState>,
    user_id: i64,
    input: RecordPayrollPaymentInput,
) -> Result<i64, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    if input.amount_milli <= 0 {
        return Err(AppError::validation("مبلغ الراتب المدفوع يجب أن يكون أكبر من صفر"));
    }
    let tx = conn.transaction()?;

    let run_status: String = tx.query_row(
        "SELECT status FROM payroll_runs WHERE id=?1",
        [input.run_id], |r| r.get(0)
    ).map_err(|_| AppError::not_found("تشغيلة الرواتب غير موجودة"))?;
    let run_status_lower = run_status.to_lowercase();
    if run_status_lower != "approved" && run_status_lower != "partially paid" {
        return Err(AppError::validation("يجب اعتماد مسير الرواتب قبل تسجيل الدفع"));
    }

    let (line_id, net_milli, paid_milli): (i64, i64, i64) = tx.query_row(
        "SELECT id, net_milli, COALESCE(paid_milli,0)
         FROM payroll_run_lines WHERE run_id=?1 AND employee_id=?2",
        params![input.run_id, input.employee_id],
        |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?)),
    ).map_err(|_| AppError::not_found("العامل غير موجود في مسير الرواتب"))?;

    let remaining = net_milli - paid_milli;
    if remaining <= 0 {
        return Err(AppError::validation("راتب هذا العامل مسدد بالكامل"));
    }
    if input.amount_milli > remaining {
        return Err(AppError::validation("المبلغ أكبر من الراتب المتبقي للعامل"));
    }

    let method = input.method.clone().unwrap_or_else(|| "bank_transfer".to_string());
    let source_type = input.source_type.clone().unwrap_or_else(|| "company".to_string()).to_lowercase();
    let source_account = if let Some(code) = input.source_account_code.clone().filter(|x| !x.trim().is_empty()) {
        let exists: i64 = tx.query_row("SELECT COUNT(*) FROM accounts WHERE code=?1", [&code], |r| r.get(0)).unwrap_or(0);
        if exists == 0 {
            return Err(AppError::validation("حساب مصدر دفع الراتب غير موجود"));
        }
        code
    } else {
        match source_type.as_str() {
            "company" => crate::commands::accounting::resolve_cash_account(&tx, input.cashbank_id, &method)?,
            "custody" => {
                let pid = input.custody_id.ok_or_else(|| AppError::validation("حدد العهدة الدافعة للراتب"))?;
                tx.query_row(
                    "SELECT COALESCE(NULLIF(trim(account_code),''),'1110') FROM petty_cash_accounts WHERE id=?1 AND active=1",
                    [pid], |r| r.get::<_, String>(0)
                ).map_err(|_| AppError::not_found("حساب العهدة غير موجود"))?
            }
            "owner_saif" => "2310".to_string(),
            "owner_abu_saif" => "2320".to_string(),
            _ => return Err(AppError::validation("مصدر دفع الراتب غير معروف")),
        }
    };

    if source_type == "custody" {
        let pid = input.custody_id.ok_or_else(|| AppError::validation("حدد العهدة"))?;
        let old_balance: i64 = tx.query_row(
            "SELECT balance_milli FROM petty_cash_accounts WHERE id=?1 AND active=1",
            [pid], |r| r.get(0)
        ).map_err(|_| AppError::not_found("حساب العهدة غير موجود"))?;
        if old_balance < input.amount_milli {
            return Err(AppError::validation("رصيد العهدة غير كافٍ لدفع الراتب"));
        }
        let new_balance = old_balance - input.amount_milli;
        tx.execute(
            "UPDATE petty_cash_accounts SET balance_milli=?1 WHERE id=?2",
            params![new_balance, pid]
        )?;
        tx.execute(
            "INSERT INTO petty_cash_transactions
             (ts, petty_id, ttype, debit_milli, credit_milli, balance_milli, category,
              account_code, reference, notes, user_id)
             VALUES(?1,?2,'Payroll Payment',0,?3,?4,'راتب','2250',?5,?6,?7)",
            params![
                format!("{} 12:00:00", input.payment_date), pid, input.amount_milli,
                new_balance, input.reference, input.notes, user_id
            ],
        )?;
    }

    let lines = vec![
        ("2250".to_string(), input.amount_milli, 0, Some("تسوية راتب مستحق".to_string())),
        (source_account.clone(), 0, input.amount_milli, Some("مصدر دفع الراتب".to_string())),
    ];
    let journal_id = crate::commands::accounting::post_to_journal(
        &tx, "payroll_payment", line_id, &input.payment_date, "دفع راتب عامل",
        &lines, &user_id.to_string()
    )?;

    // Recording a bank transfer is not, by itself, a declaration of WPS compliance.
    // A WPS reference is retained as evidence and still remains subject to reconciliation.
    let wps_status = if source_type == "company"
        && method.to_lowercase().contains("bank")
        && input.wps_reference.as_deref().map(|x| !x.trim().is_empty()).unwrap_or(false)
    {
        "recorded_for_reconciliation"
    } else {
        "review_required"
    };

    let source_id = if source_type == "custody" { input.custody_id } else { input.cashbank_id };
    tx.execute(
        "INSERT INTO payroll_payments
         (run_id, payment_date, source_type, source_id, employee_id, amount_milli,
          method, reference, notes, journal_id, source_account_code, wps_status,
          wps_reference, reversed, paid_by, paid_at)
         VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,0,?14,datetime('now'))",
        params![
            input.run_id, input.payment_date, source_type, source_id, input.employee_id,
            input.amount_milli, method, input.reference, input.notes, journal_id,
            source_account, wps_status, input.wps_reference, user_id.to_string()
        ],
    )?;
    let payment_id = tx.last_insert_rowid();

    tx.execute(
        "UPDATE payroll_run_lines SET paid_milli=COALESCE(paid_milli,0)+?1 WHERE id=?2",
        params![input.amount_milli, line_id],
    )?;

    let total_paid: i64 = tx.query_row(
        "SELECT COALESCE(SUM(paid_milli),0) FROM payroll_run_lines WHERE run_id=?1",
        [input.run_id], |r| r.get(0)
    )?;
    let total_net: i64 = tx.query_row(
        "SELECT total_net_milli FROM payroll_runs WHERE id=?1",
        [input.run_id], |r| r.get(0)
    )?;
    let new_status = if total_paid >= total_net { "Paid" } else { "Partially Paid" };
    tx.execute(
        "UPDATE payroll_runs SET status=?1, paid_by=?2,
                paid_at=CASE WHEN ?1='Paid' THEN datetime('now') ELSE paid_at END
         WHERE id=?3",
        params![new_status, user_id.to_string(), input.run_id],
    )?;

    let _ = rbac::log_audit(
        &tx, Some(user_id), None, "record_payroll_payment", "payroll_payments",
        Some(payment_id), None,
        Some(&format!("employee={} source={} wps={}", input.employee_id, source_type, wps_status)), None
    );
    tx.commit()?;
    Ok(payment_id)
}

#[tauri::command]
pub fn list_payroll_payments(
    state: State<'_, DbState>,
    run_id: i64,
) -> Result<Vec<PayrollPayment>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn.prepare(
        "SELECT pp.id, pp.run_id, pp.employee_id, e.name, pp.payment_date,
                pp.source_type, pp.source_id, pp.amount_milli, pp.method, pp.reference,
                pp.notes, pp.source_account_code, pp.wps_status, pp.wps_reference,
                pp.journal_id, pp.paid_by, pp.paid_at
         FROM payroll_payments pp
         LEFT JOIN employees e ON e.id=pp.employee_id
         WHERE pp.run_id=?1 AND COALESCE(pp.reversed,0)=0
         ORDER BY pp.payment_date, pp.id",
    )?;
    let rows = stmt.query_map([run_id], |r| {
        Ok(PayrollPayment {
            id: r.get(0)?,
            run_id: r.get(1)?,
            employee_id: r.get(2)?,
            employee_name: r.get(3)?,
            payment_date: r.get(4)?,
            source_type: r.get(5)?,
            source_id: r.get(6)?,
            amount_milli: r.get(7)?,
            method: r.get(8)?,
            reference: r.get(9)?,
            notes: r.get(10)?,
            source_account_code: r.get(11)?,
            wps_status: r.get(12)?,
            wps_reference: r.get(13)?,
            journal_id: r.get(14)?,
            paid_by: r.get(15)?,
            paid_at: r.get(16)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[tauri::command]
pub fn list_employee_advances(
    state: State<'_, DbState>,
) -> Result<Vec<EmployeeAdvance>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn.prepare(
        "SELECT ea.id, ea.employee_id, e.name, ea.amount_milli, ea.date, ea.reason,
                ea.status, ea.remaining_milli, ea.deduction_per_payroll_milli,
                ea.source_type, ea.source_id, ea.journal_id
         FROM employee_advances ea
         LEFT JOIN employees e ON ea.employee_id=e.id
         ORDER BY ea.id DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(EmployeeAdvance {
            id: row.get(0)?, employee_id: row.get(1)?, employee_name: row.get(2)?,
            amount_milli: row.get(3)?, date: row.get(4)?, reason: row.get(5)?,
            status: row.get(6)?, remaining_milli: row.get(7)?, deduction_per_payroll_milli: row.get(8)?,
            source_type: row.get(9)?, source_id: row.get(10)?, journal_id: row.get(11)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[tauri::command]
pub fn create_employee_advance(
    state: State<'_, DbState>,
    user_id: i64,
    input: CreateAdvanceInput,
) -> Result<i64, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "hr", "manager"])?;
    if input.amount_milli <= 0 {
        return Err(AppError::validation("قيمة السلفة يجب أن تكون أكبر من صفر"));
    }
    let deduction = input.deduction_per_payroll_milli.unwrap_or(0);
    if deduction < 0 || deduction > input.amount_milli {
        return Err(AppError::validation("قيمة خصم السلفة غير صحيحة"));
    }

    let tx = conn.transaction()?;
    let employee_name: String = tx.query_row(
        "SELECT name FROM employees WHERE id=?1 AND active=1",
        [input.employee_id], |r| r.get(0),
    ).map_err(|_| AppError::not_found("العامل غير موجود أو غير نشط"))?;

    let source_type = input.source_type.clone().unwrap_or_else(|| "company".to_string()).to_lowercase();
    let method = input.method.clone().unwrap_or_else(|| "cash".to_string());
    let (source_account, source_id): (String, Option<i64>) = match source_type.as_str() {
        "company" => (crate::commands::accounting::resolve_cash_account(&tx, input.cashbank_id, &method)?, input.cashbank_id),
        "custody" => {
            let pid = input.custody_id.ok_or_else(|| AppError::validation("حدد العهدة التي صُرفت منها السلفة"))?;
            let account = tx.query_row(
                "SELECT COALESCE(NULLIF(trim(account_code),''),'1110') FROM petty_cash_accounts WHERE id=?1 AND active=1",
                [pid], |r| r.get::<_, String>(0),
            ).map_err(|_| AppError::not_found("حساب العهدة غير موجود"))?;
            (account, Some(pid))
        }
        "owner_saif" => ("2310".to_string(), None),
        "owner_abu_saif" => ("2320".to_string(), None),
        _ => return Err(AppError::validation("مصدر صرف السلفة غير معروف")),
    };

    tx.execute(
        "INSERT INTO employee_advances
         (employee_id, amount_milli, date, reason, status, remaining_milli,
          deduction_per_payroll_milli, journal_id, source_type, source_id, created_by, created_at)
         VALUES(?1,?2,?3,?4,'open',?5,?6,NULL,?7,?8,?9,datetime('now'))",
        params![input.employee_id, input.amount_milli, input.date, input.reason,
                input.amount_milli, deduction, source_type, source_id, user_id.to_string()],
    )?;
    let adv_id = tx.last_insert_rowid();

    let mut custody_txn_id: Option<i64> = None;
    if source_type == "custody" {
        let pid = input.custody_id.ok_or_else(|| AppError::validation("حدد العهدة"))?;
        let old_balance: i64 = tx.query_row(
            "SELECT balance_milli FROM petty_cash_accounts WHERE id=?1 AND active=1",
            [pid], |r| r.get(0),
        ).map_err(|_| AppError::not_found("حساب العهدة غير موجود"))?;
        if old_balance < input.amount_milli {
            return Err(AppError::validation("رصيد العهدة غير كافٍ لصرف السلفة"));
        }
        let new_balance = old_balance - input.amount_milli;
        tx.execute("UPDATE petty_cash_accounts SET balance_milli=?1 WHERE id=?2", params![new_balance, pid])?;
        tx.execute(
            "INSERT INTO petty_cash_transactions
             (ts, petty_id, ttype, debit_milli, credit_milli, balance_milli, category,
              account_code, reference, notes, user_id)
             VALUES(?1,?2,'Employee Advance',0,?3,?4,'سلفة موظف','1320',?5,?6,?7)",
            params![format!("{} 12:00:00", input.date), pid, input.amount_milli, new_balance,
                    format!("ADV-{}", adv_id), input.reason, user_id],
        )?;
        custody_txn_id = Some(tx.last_insert_rowid());
    }

    let lines = vec![
        ("1320".to_string(), input.amount_milli, 0, Some(format!("سلفة العامل {}", employee_name))),
        (source_account, 0, input.amount_milli, Some("مصدر صرف سلفة الموظف".to_string())),
    ];
    let journal_id = crate::commands::accounting::post_to_journal(
        &tx, "employee_advance", adv_id, &input.date,
        &format!("سلفة موظف - {}", employee_name), &lines, &user_id.to_string(),
    )?;
    tx.execute("UPDATE employee_advances SET journal_id=?1 WHERE id=?2", params![journal_id, adv_id])?;
    if let Some(txn_id) = custody_txn_id {
        tx.execute("UPDATE petty_cash_transactions SET journal_id=?1 WHERE id=?2", params![journal_id, txn_id])?;
    }

    let _ = rbac::log_audit(
        &tx, Some(user_id), None, "create_employee_advance", "employee_advances", Some(adv_id), None,
        Some(&format!("employee={} amount={} source={} journal={}", input.employee_id, input.amount_milli, source_type, journal_id)), None,
    );
    tx.commit()?;
    Ok(adv_id)
}
