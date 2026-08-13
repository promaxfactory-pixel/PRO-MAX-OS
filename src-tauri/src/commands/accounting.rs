use crate::commands::rbac;
use crate::db::{next_sequence, DbState};
use crate::error::AppError;
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

#[derive(Debug, Deserialize)]
pub struct CreateAccountInput {
    pub code: String,
    pub name_ar: Option<String>,
    pub name_en: Option<String>,
    pub r#type: String,
    pub parent: Option<String>,
    pub is_system: Option<i64>,
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
    conn.execute(
        "INSERT INTO accounts(code, name_ar, name_en, type, parent, is_system) VALUES(?,?,?,?,?,?)",
        rusqlite::params![
            input.code,
            input.name_ar,
            input.name_en,
            input.r#type,
            input.parent,
            input.is_system.unwrap_or(0),
        ],
    )?;
    let _ = rbac::log_audit(&conn, Some(user_id), None, "create_account", "accounts", None, None, Some(&input.code), None);
    Ok(input.code)
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
    let year = chrono::Utc::now().format("%Y").to_string();

    let seq = next_sequence(conn, "JE", &year)?;
    let entry_no = format!("JE-{}-{:04}", year, seq);

    let total_debit: i64 = lines.iter().map(|l| l.1).sum();
    let total_credit: i64 = lines.iter().map(|l| l.2).sum();
    if total_debit != total_credit {
        return Err(AppError::validation("يجب أن يتساوى مجموع المدين مع مجموع الدائن"));
    }
    if total_debit == 0 {
        return Err(AppError::validation("يجب إدخال بند واحد على الأقل"));
    }

    for (account_code, ..) in lines {
        let exists: i64 = conn
            .query_row("SELECT COUNT(*) FROM accounts WHERE code=?1", [&account_code], |r| r.get(0))
            .unwrap_or(0);
        if exists == 0 {
            return Err(AppError::validation(format!("الحساب المحاسبي غير موجود: {}", account_code)));
        }
    }

    conn.execute(
        "INSERT INTO journal_entries(entry_no, date, memo, ref_type, ref_id, created_by) VALUES(?,?,?,?,?,?)",
        rusqlite::params![entry_no, date, memo, ref_type, ref_id, created_by],
    )?;
    let entry_id = conn.last_insert_rowid();

    for (account_code, debit_milli, credit_milli, line_memo) in lines {
        conn.execute(
            "INSERT INTO journal_entry_lines(entry_id, account_code, debit_milli, credit_milli, memo) VALUES(?,?,?,?,?)",
            rusqlite::params![entry_id, account_code, debit_milli, credit_milli, line_memo],
        )?;
    }

    Ok(entry_id)
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
        "SELECT a.code, a.name_ar, COALESCE(SUM(jel.debit_milli),0) as total_debit, COALESCE(SUM(jel.credit_milli),0) as total_credit FROM accounts a LEFT JOIN journal_entry_lines jel ON a.code=jel.account_code GROUP BY a.code HAVING total_debit != 0 OR total_credit != 0 ORDER BY a.code",
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

#[tauri::command]
pub fn get_balance_sheet(state: State<'_, DbState>) -> Result<Vec<BalanceSheetRow>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn.prepare(
        "SELECT a.type, a.code, a.name_ar, COALESCE(SUM(jel.debit_milli),0) - COALESCE(SUM(jel.credit_milli),0) as balance FROM accounts a LEFT JOIN journal_entry_lines jel ON a.code=jel.account_code WHERE a.type IN ('Asset','Liability','Equity') GROUP BY a.code ORDER BY a.type, a.code",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(BalanceSheetRow {
            r#type: row.get(0)?,
            code: row.get(1)?,
            name: row.get(2)?,
            balance_milli: row.get(3)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

#[tauri::command]
pub fn get_income_statement(state: State<'_, DbState>) -> Result<Vec<IncomeStatementRow>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn.prepare(
        "SELECT a.type, a.code, a.name_ar, COALESCE(SUM(jel.credit_milli),0) - COALESCE(SUM(jel.debit_milli),0) as balance FROM accounts a LEFT JOIN journal_entry_lines jel ON a.code=jel.account_code WHERE a.type IN ('Revenue','Expense') GROUP BY a.code ORDER BY a.type, a.code",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(IncomeStatementRow {
            r#type: row.get(0)?,
            code: row.get(1)?,
            name: row.get(2)?,
            balance_milli: row.get(3)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}
