use crate::commands::rbac;
use crate::db::{next_sequence, DbState};
use crate::error::AppError;
use chrono::{Datelike, NaiveDate};
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub code: String,
    pub name_ar: Option<String>,
    pub name_en: Option<String>,
    pub r#type: String,
    pub parent: Option<String>,
    pub is_system: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: i64,
    pub entry_no: Option<String>,
    pub date: String,
    pub memo: Option<String>,
    pub ref_type: Option<String>,
    pub ref_id: Option<i64>,
    pub created_by: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JournalLine {
    pub id: i64,
    pub entry_id: i64,
    pub account_code: String,
    pub account_name: Option<String>,
    pub debit_milli: i64,
    pub credit_milli: i64,
    pub memo: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountingPeriod {
    pub id: i64,
    pub name: String,
    pub start_date: String,
    pub end_date: String,
    pub status: String,
    pub created_by: Option<i64>,
    pub created_at: String,
    pub closed_by: Option<i64>,
    pub closed_at: Option<String>,
    pub close_reason: Option<String>,
    pub reopened_by: Option<i64>,
    pub reopened_at: Option<String>,
    pub reopen_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrialBalanceRow {
    pub account_code: String,
    pub account_name: String,
    pub debit_milli: i64,
    pub credit_milli: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceSheetRow {
    pub r#type: String,
    pub code: String,
    pub name: String,
    pub balance_milli: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IncomeStatementRow {
    pub r#type: String,
    pub code: String,
    pub name: String,
    pub balance_milli: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceSheet {
    pub assets: Vec<BalanceSheetRow>,
    pub liabilities: Vec<BalanceSheetRow>,
    pub equity: Vec<BalanceSheetRow>,
    pub current_period_earnings_milli: i64,
    pub total_assets_milli: i64,
    pub total_liabilities_milli: i64,
    pub total_equity_milli: i64,
    pub total_liabilities_equity_milli: i64,
    pub out_of_balance_milli: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IncomeStatement {
    pub revenue: Vec<IncomeStatementRow>,
    pub expenses: Vec<IncomeStatementRow>,
    pub total_revenue_milli: i64,
    pub total_expenses_milli: i64,
    pub net_income_milli: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateAccountInput {
    pub code: String,
    pub name_ar: Option<String>,
    pub name_en: Option<String>,
    pub r#type: String,
    pub parent: Option<String>,
    #[serde(rename = "is_system")]
    pub _is_system: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateJournalEntryInput {
    pub date: String,
    pub memo: Option<String>,
    pub ref_type: Option<String>,
    pub ref_id: Option<i64>,
    pub lines: Vec<CreateJournalLineInput>,
}

#[derive(Debug, Deserialize)]
pub struct CreateJournalLineInput {
    pub account_code: String,
    pub debit_milli: i64,
    pub credit_milli: i64,
    pub memo: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAccountingPeriodInput {
    pub name: String,
    pub start_date: String,
    pub end_date: String,
}

fn normalized_iso_date(value: &str, field_name: &str) -> Result<String, AppError> {
    let trimmed = value.trim();
    let parsed = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d")
        .map_err(|_| AppError::validation(format!("{} يجب أن يكون بصيغة YYYY-MM-DD", field_name)))?;
    Ok(parsed.format("%Y-%m-%d").to_string())
}

fn ensure_posting_period_open(conn: &rusqlite::Connection, journal_date: &str) -> Result<(), AppError> {
    let periods_table_exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='accounting_periods'",
        [],
        |row| row.get(0),
    )?;
    if periods_table_exists == 0 {
        return Ok(());
    }

    let locked: Option<(i64, String)> = conn
        .query_row(
            "SELECT id, name FROM accounting_periods
             WHERE status='closed' AND ?1 BETWEEN start_date AND end_date
             ORDER BY start_date DESC LIMIT 1",
            [journal_date],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    if let Some((period_id, period_name)) = locked {
        return Err(AppError::business(format!(
            "لا يمكن ترحيل القيد بتاريخ {} لأن الفترة المالية مغلقة: {} (#{}). أعد فتحها بصلاحية المدير مع تسجيل السبب أولًا",
            journal_date, period_name, period_id
        )));
    }
    Ok(())
}

#[tauri::command]
pub fn list_accounting_periods(
    state: State<'_, DbState>,
    user_id: i64,
) -> Result<Vec<AccountingPeriod>, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager", "viewer"])?;
    let mut stmt = conn.prepare(
        "SELECT id, name, start_date, end_date, status, created_by, created_at,
                closed_by, closed_at, close_reason, reopened_by, reopened_at, reopen_reason
         FROM accounting_periods ORDER BY start_date DESC, id DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(AccountingPeriod {
            id: row.get(0)?,
            name: row.get(1)?,
            start_date: row.get(2)?,
            end_date: row.get(3)?,
            status: row.get(4)?,
            created_by: row.get(5)?,
            created_at: row.get(6)?,
            closed_by: row.get(7)?,
            closed_at: row.get(8)?,
            close_reason: row.get(9)?,
            reopened_by: row.get(10)?,
            reopened_at: row.get(11)?,
            reopen_reason: row.get(12)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

#[tauri::command]
pub fn create_accounting_period(
    state: State<'_, DbState>,
    user_id: i64,
    input: CreateAccountingPeriodInput,
) -> Result<i64, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant"])?;
    let name = input.name.trim();
    if name.is_empty() || name.chars().count() > 100 {
        return Err(AppError::validation("اسم الفترة المالية مطلوب وبحد أقصى 100 حرف"));
    }
    let start_date = normalized_iso_date(&input.start_date, "تاريخ بداية الفترة")?;
    let end_date = normalized_iso_date(&input.end_date, "تاريخ نهاية الفترة")?;
    if start_date > end_date {
        return Err(AppError::validation("تاريخ بداية الفترة يجب ألا يتجاوز تاريخ نهايتها"));
    }
    let overlap_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM accounting_periods
         WHERE NOT (end_date < ?1 OR start_date > ?2)",
        rusqlite::params![start_date, end_date],
        |row| row.get(0),
    )?;
    if overlap_count > 0 {
        return Err(AppError::validation("تتداخل الفترة المالية مع فترة موجودة"));
    }

    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO accounting_periods(name, start_date, end_date, created_by)
         VALUES(?1, ?2, ?3, ?4)",
        rusqlite::params![name, start_date, end_date, user_id],
    )?;
    let period_id = tx.last_insert_rowid();
    rbac::log_audit(
        &tx,
        Some(user_id),
        None,
        "create_accounting_period",
        "accounting_periods",
        Some(period_id),
        None,
        Some(&format!("{}:{}..{}", name, start_date, end_date)),
        None,
    )?;
    tx.commit()?;
    Ok(period_id)
}

#[tauri::command]
pub fn close_accounting_period(
    state: State<'_, DbState>,
    user_id: i64,
    period_id: i64,
    reason: String,
) -> Result<(), AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant"])?;
    let reason = reason.trim();
    if reason.chars().count() < 3 || reason.chars().count() > 500 {
        return Err(AppError::validation("سبب إغلاق الفترة مطلوب من 3 إلى 500 حرف"));
    }
    let period: Option<(String, String, String)> = conn
        .query_row(
            "SELECT start_date, end_date, status FROM accounting_periods WHERE id=?1",
            [period_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?;
    let (start_date, end_date, status) =
        period.ok_or_else(|| AppError::not_found("الفترة المالية غير موجودة"))?;
    if status == "closed" {
        return Err(AppError::business("الفترة المالية مغلقة بالفعل"));
    }

    let invalid_entries: i64 = conn.query_row(
        "SELECT COUNT(*) FROM (
            SELECT je.id
            FROM journal_entries je
            LEFT JOIN journal_entry_lines jel ON jel.entry_id=je.id
            WHERE substr(je.date, 1, 10) BETWEEN ?1 AND ?2
            GROUP BY je.id
            HAVING COUNT(jel.id) < 2
                OR COALESCE(SUM(jel.debit_milli), 0) != COALESCE(SUM(jel.credit_milli), 0)
                OR COALESCE(SUM(jel.debit_milli), 0) <= 0
                OR COALESCE(SUM(CASE
                    WHEN COALESCE(jel.debit_milli, 0) < 0 OR COALESCE(jel.credit_milli, 0) < 0
                      OR (COALESCE(jel.debit_milli, 0) > 0 AND COALESCE(jel.credit_milli, 0) > 0)
                      OR (COALESCE(jel.debit_milli, 0) = 0 AND COALESCE(jel.credit_milli, 0) = 0)
                    THEN 1 ELSE 0 END), 0) > 0
        )",
        rusqlite::params![start_date, end_date],
        |row| row.get(0),
    )?;
    if invalid_entries > 0 {
        return Err(AppError::business(format!(
            "لا يمكن إغلاق الفترة؛ يوجد {} قيد غير متوازن أو غير صالح داخلها",
            invalid_entries
        )));
    }

    let tx = conn.unchecked_transaction()?;
    let changed = tx.execute(
        "UPDATE accounting_periods
         SET status='closed', closed_by=?1, closed_at=datetime('now'), close_reason=?2,
             reopened_by=NULL, reopened_at=NULL, reopen_reason=NULL, updated_at=datetime('now')
         WHERE id=?3 AND status='open'",
        rusqlite::params![user_id, reason, period_id],
    )?;
    if changed != 1 {
        return Err(AppError::business("تعذر إغلاق الفترة بسبب تغير حالتها؛ أعد المحاولة"));
    }
    rbac::log_audit(
        &tx,
        Some(user_id),
        None,
        "close_accounting_period",
        "accounting_periods",
        Some(period_id),
        Some("status=open"),
        Some("status=closed"),
        Some(reason),
    )?;
    tx.commit()?;
    Ok(())
}

#[tauri::command]
pub fn reopen_accounting_period(
    state: State<'_, DbState>,
    user_id: i64,
    period_id: i64,
    reason: String,
) -> Result<(), AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin"])?;
    let reason = reason.trim();
    if reason.chars().count() < 3 || reason.chars().count() > 500 {
        return Err(AppError::validation("سبب إعادة فتح الفترة مطلوب من 3 إلى 500 حرف"));
    }
    let status: Option<String> = conn
        .query_row(
            "SELECT status FROM accounting_periods WHERE id=?1",
            [period_id],
            |row| row.get(0),
        )
        .optional()?;
    match status.as_deref() {
        None => return Err(AppError::not_found("الفترة المالية غير موجودة")),
        Some("open") => return Err(AppError::business("الفترة المالية مفتوحة بالفعل")),
        Some("closed") => {}
        _ => return Err(AppError::business("حالة الفترة المالية غير صالحة")),
    }

    let tx = conn.unchecked_transaction()?;
    let changed = tx.execute(
        "UPDATE accounting_periods
         SET status='open', reopened_by=?1, reopened_at=datetime('now'), reopen_reason=?2,
             updated_at=datetime('now')
         WHERE id=?3 AND status='closed'",
        rusqlite::params![user_id, reason, period_id],
    )?;
    if changed != 1 {
        return Err(AppError::business("تعذر إعادة فتح الفترة بسبب تغير حالتها؛ أعد المحاولة"));
    }
    rbac::log_audit(
        &tx,
        Some(user_id),
        None,
        "reopen_accounting_period",
        "accounting_periods",
        Some(period_id),
        Some("status=closed"),
        Some("status=open"),
        Some(reason),
    )?;
    tx.commit()?;
    Ok(())
}

#[tauri::command]
pub fn list_accounts(state: State<'_, DbState>) -> Result<Vec<Account>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn.prepare(
        "SELECT code, name_ar, name_en, type, parent, is_system FROM accounts ORDER BY code",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Account {
            code: row.get(0)?,
            name_ar: row.get(1)?,
            name_en: row.get(2)?,
            r#type: row.get(3)?,
            parent: row.get(4)?,
            is_system: row.get(5)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

#[tauri::command]
pub fn get_account(state: State<'_, DbState>, code: String) -> Result<Account, AppError> {
    let conn = state.0.lock()?;
    conn.query_row(
        "SELECT code, name_ar, name_en, type, parent, is_system FROM accounts WHERE code=?",
        [&code],
        |row| {
            Ok(Account {
                code: row.get(0)?,
                name_ar: row.get(1)?,
                name_en: row.get(2)?,
                r#type: row.get(3)?,
                parent: row.get(4)?,
                is_system: row.get(5)?,
            })
        },
    )
    .map_err(|_| AppError::not_found("الحساب غير موجود"))
}

#[tauri::command]
pub fn create_account(state: State<'_, DbState>, user_id: i64, input: CreateAccountInput) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant"])?;
    let code = input.code.trim().to_string();
    if code.is_empty() || code.len() > 30 || !code.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.')) {
        return Err(AppError::validation("رمز الحساب غير صالح"));
    }
    if input.name_ar.as_deref().unwrap_or_default().trim().is_empty()
        && input.name_en.as_deref().unwrap_or_default().trim().is_empty()
    {
        return Err(AppError::validation("اسم الحساب مطلوب"));
    }
    let account_type = input.r#type.trim().to_ascii_lowercase();
    if !matches!(account_type.as_str(), "asset" | "liability" | "equity" | "revenue" | "expense") {
        return Err(AppError::validation("نوع الحساب المحاسبي غير صالح"));
    }
    if let Some(parent) = input.parent.as_deref().map(str::trim).filter(|p| !p.is_empty()) {
        if parent == code.as_str() {
            return Err(AppError::validation("لا يمكن أن يكون الحساب أصلًا لنفسه"));
        }
        let parent_exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM accounts WHERE code=?1",
            [parent],
            |row| row.get(0),
        )?;
        if parent_exists == 0 {
            return Err(AppError::validation("الحساب الأب غير موجود"));
        }
    }
    conn.execute(
        "INSERT INTO accounts(code, name_ar, name_en, type, parent, is_system) VALUES(?,?,?,?,?,?)",
        rusqlite::params![
            code,
            input.name_ar,
            input.name_en,
            account_type,
            input.parent,
            0,
        ],
    )?;
    let _ = rbac::log_audit(&conn, Some(user_id), None, "create_account", "accounts", None, None, Some(&code), None);
    Ok(code)
}

#[tauri::command]
pub fn list_journal_entries(state: State<'_, DbState>) -> Result<Vec<JournalEntry>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn.prepare(
        "SELECT id, entry_no, date, memo, ref_type, ref_id, created_by, created_at FROM journal_entries ORDER BY id DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(JournalEntry {
            id: row.get(0)?,
            entry_no: row.get(1)?,
            date: row.get(2)?,
            memo: row.get(3)?,
            ref_type: row.get(4)?,
            ref_id: row.get(5)?,
            created_by: row.get(6)?,
            created_at: row.get(7)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

#[tauri::command]
pub fn get_journal_entry_lines(
    state: State<'_, DbState>,
    entry_id: i64,
) -> Result<Vec<JournalLine>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn.prepare(
        "SELECT jel.id, jel.entry_id, jel.account_code, a.name_ar, jel.debit_milli, jel.credit_milli, jel.memo FROM journal_entry_lines jel LEFT JOIN accounts a ON jel.account_code=a.code WHERE jel.entry_id=?",
    )?;
    let rows = stmt.query_map([entry_id], |row| {
        Ok(JournalLine {
            id: row.get(0)?,
            entry_id: row.get(1)?,
            account_code: row.get(2)?,
            account_name: row.get(3)?,
            debit_milli: row.get(4)?,
            credit_milli: row.get(5)?,
            memo: row.get(6)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

#[tauri::command]
pub fn create_journal_entry(
    state: State<'_, DbState>,
    user_id: i64,
    input: CreateJournalEntryInput,
) -> Result<i64, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant"])?;
    let lines: Vec<(String, i64, i64, Option<String>)> = input
        .lines
        .iter()
        .map(|l| (l.account_code.clone(), l.debit_milli, l.credit_milli, l.memo.clone()))
        .collect();
    let ref_type = input.ref_type.unwrap_or_else(|| "journal".to_string());
    let entry_id = post_to_journal(&conn, &ref_type, input.ref_id.unwrap_or(0), &input.date, &input.memo.unwrap_or_default(), &lines, "manual")?;
    let _ = rbac::log_audit(&conn, Some(user_id), None, "create_journal_entry", "journal_entries", Some(entry_id), None, Some(&format!("ref_type={}", ref_type)), None);
    Ok(entry_id)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn post_to_journal(
    conn: &rusqlite::Connection,
    ref_type: &str,
    ref_id: i64,
    date: &str,
    memo: &str,
    lines: &[(String, i64, i64, Option<String>)],
    created_by: &str,
) -> Result<i64, AppError> {
    let date_part = date
        .get(..10)
        .ok_or_else(|| AppError::validation("تاريخ القيد يجب أن يكون تاريخًا صحيحًا بصيغة YYYY-MM-DD"))?;
    let document_date = NaiveDate::parse_from_str(date_part, "%Y-%m-%d")
        .map_err(|_| AppError::validation("تاريخ القيد يجب أن يكون تاريخًا صحيحًا بصيغة YYYY-MM-DD"))?;
    let journal_date = document_date.format("%Y-%m-%d").to_string();
    let year = document_date.year().to_string();

    if ref_type.trim().is_empty() {
        return Err(AppError::validation("نوع مرجع القيد مطلوب"));
    }
    if lines.len() < 2 {
        return Err(AppError::validation("يجب أن يحتوي القيد على سطرين محاسبيين على الأقل"));
    }

    let mut total_debit = 0_i64;
    let mut total_credit = 0_i64;
    for (account_code, debit_milli, credit_milli, _) in lines {
        if account_code.trim().is_empty() {
            return Err(AppError::validation("رمز الحساب مطلوب لكل سطر قيد"));
        }
        if *debit_milli < 0 || *credit_milli < 0 {
            return Err(AppError::validation("لا يسمح بقيم سالبة في سطور القيود"));
        }
        if (*debit_milli > 0) == (*credit_milli > 0) {
            return Err(AppError::validation(
                "يجب أن يكون كل سطر مدينًا أو دائنًا فقط وبقيمة أكبر من صفر",
            ));
        }
        total_debit = total_debit
            .checked_add(*debit_milli)
            .ok_or_else(|| AppError::validation("إجمالي المدين يتجاوز الحد المسموح"))?;
        total_credit = total_credit
            .checked_add(*credit_milli)
            .ok_or_else(|| AppError::validation("إجمالي الدائن يتجاوز الحد المسموح"))?;
    }
    if total_debit != total_credit {
        return Err(AppError::validation("يجب أن يتساوى مجموع المدين مع مجموع الدائن"));
    }
    if total_debit == 0 {
        return Err(AppError::validation("يجب إدخال بند واحد على الأقل"));
    }

    ensure_posting_period_open(conn, &journal_date)?;

    for (account_code, ..) in lines {
        let exists: i64 = conn
            .query_row("SELECT COUNT(*) FROM accounts WHERE code=?1", [&account_code], |r| r.get(0))
            .unwrap_or(0);
        if exists == 0 {
            return Err(AppError::validation(format!("الحساب المحاسبي غير موجود: {}", account_code)));
        }
    }

    // SAVEPOINT works both on a bare connection and inside a caller-owned
    // transaction. Sequence allocation, header and every line therefore form
    // one atomic unit without committing the caller's wider business document.
    let savepoint = format!("journal_{}", uuid::Uuid::new_v4().simple());
    conn.execute_batch(&format!("SAVEPOINT {savepoint}"))?;
    let result = (|| -> Result<i64, AppError> {
        let seq = next_sequence(conn, "JE", &year)?;
        let entry_no = format!("JE-{}-{:04}", year, seq);
        conn.execute(
            "INSERT INTO journal_entries(entry_no, date, memo, ref_type, ref_id, created_by) VALUES(?,?,?,?,?,?)",
            rusqlite::params![entry_no, journal_date, memo, ref_type, ref_id, created_by],
        )?;
        let entry_id = conn.last_insert_rowid();

        for (account_code, debit_milli, credit_milli, line_memo) in lines {
            conn.execute(
                "INSERT INTO journal_entry_lines(entry_id, account_code, debit_milli, credit_milli, memo) VALUES(?,?,?,?,?)",
                rusqlite::params![entry_id, account_code, debit_milli, credit_milli, line_memo],
            )?;
        }

        Ok(entry_id)
    })();

    match result {
        Ok(entry_id) => {
            conn.execute_batch(&format!("RELEASE SAVEPOINT {savepoint}"))?;
            Ok(entry_id)
        }
        Err(error) => {
            let _ = conn.execute_batch(&format!(
                "ROLLBACK TO SAVEPOINT {savepoint}; RELEASE SAVEPOINT {savepoint};"
            ));
            Err(error)
        }
    }
}

pub(crate) fn resolve_cash_account(
    conn: &rusqlite::Connection,
    cashbank_id: Option<i64>,
    method: &str,
) -> Result<String, AppError> {
    if let Some(cid) = cashbank_id {
        let code: Option<String> = conn
            .query_row(
                "SELECT account_code FROM cashbank_accounts WHERE id=?1",
                [cid],
                |r| r.get(0),
            )
            .unwrap_or(None);
        if let Some(code) = code {
            let code = code.trim().to_string();
            if !code.is_empty() {
                let exists: i64 = conn
                    .query_row("SELECT COUNT(*) FROM accounts WHERE code=?1", [&code], |r| r.get(0))
                    .unwrap_or(0);
                if exists > 0 {
                    return Ok(code);
                }
            }
        }
    }
    let m = method.to_lowercase();
    if m.contains("bank") || m.contains("cheque") || m.contains("transfer") || m.contains("بنك") || m.contains("شيك") {
        Ok("1101".to_string())
    } else {
        Ok("1100".to_string())
    }
}

#[tauri::command]
pub fn get_trial_balance(state: State<'_, DbState>) -> Result<Vec<TrialBalanceRow>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn.prepare(
        "SELECT a.code, COALESCE(a.name_ar, a.name_en, a.code), COALESCE(SUM(jel.debit_milli),0) as total_debit, COALESCE(SUM(jel.credit_milli),0) as total_credit FROM accounts a LEFT JOIN journal_entry_lines jel ON a.code=jel.account_code GROUP BY a.code HAVING total_debit != 0 OR total_credit != 0 ORDER BY a.code",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(TrialBalanceRow {
            account_code: row.get(0)?,
            account_name: row.get(1)?,
            debit_milli: row.get(2)?,
            credit_milli: row.get(3)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

/// Trial balance as at the inclusive reporting date. This is read-only and
/// deliberately joins journal headers so future-dated entries cannot leak
/// into a historical report.
#[tauri::command]
pub fn get_trial_balance_as_of(
    state: State<'_, DbState>,
    date_to: String,
) -> Result<Vec<TrialBalanceRow>, AppError> {
    let date_to = normalized_iso_date(&date_to, "تاريخ التقرير")?;
    let conn = state.0.lock()?;
    let mut stmt = conn.prepare(
        "SELECT a.code, COALESCE(a.name_ar, a.name_en, a.code),
                COALESCE(SUM(CASE WHEN je.date <= ?1 THEN jel.debit_milli ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN je.date <= ?1 THEN jel.credit_milli ELSE 0 END), 0)
         FROM accounts a
         LEFT JOIN journal_entry_lines jel ON a.code = jel.account_code
         LEFT JOIN journal_entries je ON je.id = jel.entry_id
         GROUP BY a.code
         HAVING COALESCE(SUM(CASE WHEN je.date <= ?1 THEN jel.debit_milli ELSE 0 END), 0) != 0
             OR COALESCE(SUM(CASE WHEN je.date <= ?1 THEN jel.credit_milli ELSE 0 END), 0) != 0
         ORDER BY a.code",
    )?;
    let rows = stmt.query_map([date_to], |row| {
        Ok(TrialBalanceRow {
            account_code: row.get(0)?,
            account_name: row.get(1)?,
            debit_milli: row.get(2)?,
            credit_milli: row.get(3)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

pub(crate) fn build_balance_sheet(conn: &rusqlite::Connection) -> Result<BalanceSheet, AppError> {
    let mut stmt = conn.prepare(
        "SELECT CASE LOWER(a.type)
                    WHEN 'asset' THEN 'Asset'
                    WHEN 'liability' THEN 'Liability'
                    ELSE 'Equity'
                END,
                a.code, COALESCE(a.name_ar, a.name_en, a.code),
                CASE WHEN LOWER(a.type) = 'asset'
                     THEN COALESCE(SUM(jel.debit_milli),0) - COALESCE(SUM(jel.credit_milli),0)
                     ELSE COALESCE(SUM(jel.credit_milli),0) - COALESCE(SUM(jel.debit_milli),0)
                END AS balance
         FROM accounts a
         LEFT JOIN journal_entry_lines jel ON a.code=jel.account_code
         WHERE LOWER(a.type) IN ('asset','liability','equity')
         GROUP BY a.type, a.code
         ORDER BY a.type, a.code",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(BalanceSheetRow {
            r#type: row.get(0)?,
            code: row.get(1)?,
            name: row.get(2)?,
            balance_milli: row.get(3)?,
        })
    })?;
    let rows = rows.collect::<Result<Vec<_>, _>>()?;
    let mut assets = Vec::new();
    let mut liabilities = Vec::new();
    let mut equity = Vec::new();
    for row in rows {
        match row.r#type.as_str() {
            "Asset" => assets.push(row),
            "Liability" => liabilities.push(row),
            "Equity" => equity.push(row),
            _ => {}
        }
    }

    let income = build_income_statement(conn)?;
    let current_period_earnings_milli = income.net_income_milli;
    let total_assets_milli = assets.iter().map(|r| r.balance_milli).sum();
    let total_liabilities_milli = liabilities.iter().map(|r| r.balance_milli).sum();
    let total_equity_milli = equity.iter().map(|r| r.balance_milli).sum::<i64>()
        + current_period_earnings_milli;
    let total_liabilities_equity_milli = total_liabilities_milli + total_equity_milli;

    Ok(BalanceSheet {
        assets,
        liabilities,
        equity,
        current_period_earnings_milli,
        total_assets_milli,
        total_liabilities_milli,
        total_equity_milli,
        total_liabilities_equity_milli,
        out_of_balance_milli: total_assets_milli - total_liabilities_equity_milli,
    })
}

#[tauri::command]
pub fn get_balance_sheet(state: State<'_, DbState>) -> Result<BalanceSheet, AppError> {
    let conn = state.0.lock()?;
    build_balance_sheet(&conn)
}

pub(crate) fn build_income_statement(conn: &rusqlite::Connection) -> Result<IncomeStatement, AppError> {
    let mut stmt = conn.prepare(
        "SELECT CASE LOWER(a.type)
                    WHEN 'revenue' THEN 'Revenue'
                    ELSE 'Expense'
                END,
                a.code, COALESCE(a.name_ar, a.name_en, a.code),
                CASE WHEN LOWER(a.type) = 'revenue'
                     THEN COALESCE(SUM(jel.credit_milli),0) - COALESCE(SUM(jel.debit_milli),0)
                     ELSE COALESCE(SUM(jel.debit_milli),0) - COALESCE(SUM(jel.credit_milli),0)
                END AS balance
         FROM accounts a
         LEFT JOIN journal_entry_lines jel ON a.code=jel.account_code
         WHERE LOWER(a.type) IN ('revenue','expense')
         GROUP BY a.type, a.code
         ORDER BY a.type, a.code",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(IncomeStatementRow {
            r#type: row.get(0)?,
            code: row.get(1)?,
            name: row.get(2)?,
            balance_milli: row.get(3)?,
        })
    })?;
    let rows = rows.collect::<Result<Vec<_>, _>>()?;
    let mut revenue = Vec::new();
    let mut expenses = Vec::new();
    for row in rows {
        match row.r#type.as_str() {
            "Revenue" => revenue.push(row),
            "Expense" => expenses.push(row),
            _ => {}
        }
    }
    let total_revenue_milli = revenue.iter().map(|r| r.balance_milli).sum();
    let total_expenses_milli = expenses.iter().map(|r| r.balance_milli).sum();
    Ok(IncomeStatement {
        revenue,
        expenses,
        total_revenue_milli,
        total_expenses_milli,
        net_income_milli: total_revenue_milli - total_expenses_milli,
    })
}

#[tauri::command]
pub fn get_income_statement(state: State<'_, DbState>) -> Result<IncomeStatement, AppError> {
    let conn = state.0.lock()?;
    build_income_statement(&conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn accounting_connection() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             CREATE TABLE doc_sequences (
                doc_type TEXT NOT NULL,
                year INTEGER NOT NULL,
                last_number INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (doc_type, year)
             );
             CREATE TABLE accounts (
                code TEXT PRIMARY KEY,
                name_ar TEXT,
                name_en TEXT,
                type TEXT NOT NULL,
                parent TEXT,
                is_system INTEGER NOT NULL DEFAULT 0
             );
             CREATE TABLE journal_entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                entry_no TEXT,
                date TEXT NOT NULL,
                memo TEXT,
                ref_type TEXT,
                ref_id INTEGER,
                created_by TEXT,
                created_at TEXT,
                reversed_by INTEGER
             );
             CREATE TABLE journal_entry_lines (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                entry_id INTEGER NOT NULL REFERENCES journal_entries(id),
                account_code TEXT NOT NULL REFERENCES accounts(code),
                debit_milli INTEGER NOT NULL DEFAULT 0,
                credit_milli INTEGER NOT NULL DEFAULT 0,
                memo TEXT
             );
             CREATE TABLE accounting_periods (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                start_date TEXT NOT NULL,
                end_date TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'open'
             );
             CREATE TRIGGER trg_journal_period_closed_insert
             BEFORE INSERT ON journal_entries
             WHEN EXISTS (
                SELECT 1 FROM accounting_periods ap
                WHERE ap.status='closed'
                  AND substr(NEW.date, 1, 10) BETWEEN ap.start_date AND ap.end_date
             )
             BEGIN
                SELECT RAISE(ABORT, 'ACCOUNTING_PERIOD_CLOSED');
             END;
             INSERT INTO accounts(code, name_ar, name_en, type) VALUES
                ('1100', 'النقدية', 'Cash', 'Asset'),
                ('3000', 'رأس المال', 'Capital', 'Equity'),
                ('4100', 'المبيعات', 'Sales', 'Revenue'),
                ('5200', 'المصروفات', 'Expenses', 'Expense');",
        )
        .unwrap();
        conn
    }

    #[test]
    fn journal_number_uses_document_date_year() {
        let conn = accounting_connection();
        let lines = vec![
            ("1100".to_string(), 1_000, 0, None),
            ("3000".to_string(), 0, 1_000, None),
        ];
        let id = post_to_journal(
            &conn,
            "opening_balance",
            1,
            "2031-01-02",
            "Opening balance",
            &lines,
            "test",
        )
        .unwrap();
        let entry_no: String = conn
            .query_row(
                "SELECT entry_no FROM journal_entries WHERE id=?1",
                [id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(entry_no, "JE-2031-0001");
    }

    #[test]
    fn journal_header_sequence_and_lines_roll_back_together() {
        let conn = accounting_connection();
        conn.execute_batch(
            "CREATE TRIGGER fail_capital_line
             BEFORE INSERT ON journal_entry_lines
             WHEN NEW.account_code = '3000'
             BEGIN
               SELECT RAISE(ABORT, 'simulated line failure');
             END;",
        )
        .unwrap();
        let lines = vec![
            ("1100".to_string(), 1_000, 0, None),
            ("3000".to_string(), 0, 1_000, None),
        ];

        assert!(post_to_journal(
            &conn,
            "test_atomicity",
            9,
            "2029-06-01",
            "Must roll back",
            &lines,
            "test",
        )
        .is_err());

        let entries: i64 = conn
            .query_row("SELECT COUNT(*) FROM journal_entries", [], |row| row.get(0))
            .unwrap();
        let lines: i64 = conn
            .query_row("SELECT COUNT(*) FROM journal_entry_lines", [], |row| row.get(0))
            .unwrap();
        let sequences: i64 = conn
            .query_row("SELECT COUNT(*) FROM doc_sequences", [], |row| row.get(0))
            .unwrap();
        assert_eq!((entries, lines, sequences), (0, 0, 0));
    }

    #[test]
    fn journal_rejects_invalid_line_shapes() {
        let conn = accounting_connection();
        let both_sides = vec![
            ("1100".to_string(), 1_000, 1_000, None),
            ("3000".to_string(), 1_000, 1_000, None),
        ];
        assert!(post_to_journal(
            &conn,
            "manual",
            0,
            "2029-06-01",
            "Invalid",
            &both_sides,
            "test",
        )
        .is_err());
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM journal_entries", [], |row| row.get::<_, i64>(0))
                .unwrap(),
            0
        );
    }

    #[test]
    fn closed_period_rejects_normal_journal_posting_without_allocating_sequence() {
        let conn = accounting_connection();
        conn.execute(
            "INSERT INTO accounting_periods(name, start_date, end_date, status)
             VALUES('June 2029', '2029-06-01', '2029-06-30', 'closed')",
            [],
        )
        .unwrap();
        let lines = vec![
            ("1100".to_string(), 1_000, 0, None),
            ("3000".to_string(), 0, 1_000, None),
        ];

        let error = post_to_journal(
            &conn,
            "manual",
            0,
            "2029-06-15",
            "Backdated entry",
            &lines,
            "test",
        )
        .unwrap_err();

        assert!(error.to_string().contains("الفترة المالية مغلقة"));
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM journal_entries", [], |row| row.get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM doc_sequences", [], |row| row.get::<_, i64>(0))
                .unwrap(),
            0
        );
    }

    #[test]
    fn database_trigger_blocks_direct_import_into_closed_period() {
        let conn = accounting_connection();
        conn.execute(
            "INSERT INTO accounting_periods(name, start_date, end_date, status)
             VALUES('June 2029', '2029-06-01', '2029-06-30', 'closed')",
            [],
        )
        .unwrap();

        let error = conn
            .execute(
                "INSERT INTO journal_entries(entry_no, date, memo)
                 VALUES('IMP-1', '2029-06-20', 'Direct import')",
                [],
            )
            .unwrap_err();

        assert!(error.to_string().contains("ACCOUNTING_PERIOD_CLOSED"));
    }

    #[test]
    fn open_period_allows_journal_posting() {
        let conn = accounting_connection();
        conn.execute(
            "INSERT INTO accounting_periods(name, start_date, end_date, status)
             VALUES('July 2029', '2029-07-01', '2029-07-31', 'open')",
            [],
        )
        .unwrap();
        let lines = vec![
            ("1100".to_string(), 1_000, 0, None),
            ("3000".to_string(), 0, 1_000, None),
        ];

        post_to_journal(
            &conn,
            "manual",
            0,
            "2029-07-15",
            "Allowed entry",
            &lines,
            "test",
        )
        .unwrap();

        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM journal_entries", [], |row| row.get::<_, i64>(0))
                .unwrap(),
            1
        );
    }

    #[test]
    fn statements_use_normal_account_signs_and_include_current_earnings() {
        let conn = accounting_connection();
        for (ref_type, ref_id, lines) in [
            (
                "capital",
                1,
                vec![
                    ("1100".to_string(), 100_000, 0, None),
                    ("3000".to_string(), 0, 100_000, None),
                ],
            ),
            (
                "sale",
                2,
                vec![
                    ("1100".to_string(), 50_000, 0, None),
                    ("4100".to_string(), 0, 50_000, None),
                ],
            ),
            (
                "expense",
                3,
                vec![
                    ("5200".to_string(), 20_000, 0, None),
                    ("1100".to_string(), 0, 20_000, None),
                ],
            ),
        ] {
            post_to_journal(
                &conn,
                ref_type,
                ref_id,
                "2029-06-01",
                ref_type,
                &lines,
                "test",
            )
            .unwrap();
        }

        let income = build_income_statement(&conn).unwrap();
        assert_eq!(income.total_revenue_milli, 50_000);
        assert_eq!(income.total_expenses_milli, 20_000);
        assert_eq!(income.net_income_milli, 30_000);

        let balance = build_balance_sheet(&conn).unwrap();
        assert_eq!(balance.total_assets_milli, 130_000);
        assert_eq!(balance.total_liabilities_milli, 0);
        assert_eq!(balance.total_equity_milli, 130_000);
        assert_eq!(balance.total_liabilities_equity_milli, 130_000);
        assert_eq!(balance.out_of_balance_milli, 0);
    }
}
