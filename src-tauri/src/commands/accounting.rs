use crate::db::DbState;
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
pub fn list_accounts(state: State<'_, DbState>) -> Result<Vec<Account>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT code, name_ar, name_en, type, parent, is_system FROM accounts ORDER BY code",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Account {
                code: row.get(0)?,
                name_ar: row.get(1)?,
                name_en: row.get(2)?,
                r#type: row.get(3)?,
                parent: row.get(4)?,
                is_system: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_account(state: State<'_, DbState>, code: String) -> Result<Account, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
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
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_account(state: State<'_, DbState>, input: CreateAccountInput) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
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
    )
    .map_err(|e| e.to_string())?;
    Ok(input.code)
}

#[tauri::command]
pub fn list_journal_entries(state: State<'_, DbState>) -> Result<Vec<JournalEntry>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, entry_no, date, memo, ref_type, ref_id, created_by, created_at FROM journal_entries ORDER BY id DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
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
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_journal_entry_lines(
    state: State<'_, DbState>,
    entry_id: i64,
) -> Result<Vec<JournalLine>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT jel.id, jel.entry_id, jel.account_code, a.name_ar, jel.debit_milli, jel.credit_milli, jel.memo FROM journal_entry_lines jel LEFT JOIN accounts a ON jel.account_code=a.code WHERE jel.entry_id=?",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([entry_id], |row| {
            Ok(JournalLine {
                id: row.get(0)?,
                entry_id: row.get(1)?,
                account_code: row.get(2)?,
                account_name: row.get(3)?,
                debit_milli: row.get(4)?,
                credit_milli: row.get(5)?,
                memo: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_journal_entry(
    state: State<'_, DbState>,
    input: CreateJournalEntryInput,
) -> Result<i64, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let year = chrono::Utc::now().format("%Y").to_string();

    let seq: i64 = conn
        .query_row(
            "SELECT COALESCE(last_number,0)+1 FROM doc_sequences WHERE doc_type='JE' AND year=?",
            [&year],
            |r| r.get(0),
        )
        .unwrap_or(1);
    if let Err(e) = conn.execute(
        "INSERT INTO doc_sequences(doc_type, year, last_number) VALUES('JE',?,?) ON CONFLICT(doc_type, year) DO UPDATE SET last_number=excluded.last_number",
        rusqlite::params![year, seq],
    ) {
        eprintln!("ERROR: Failed to increment journal entry sequence: {}", e);
    }
    let entry_no = format!("JE-{}-{:04}", year, seq);

    let total_debit: i64 = input.lines.iter().map(|l| l.debit_milli).sum();
    let total_credit: i64 = input.lines.iter().map(|l| l.credit_milli).sum();
    if total_debit != total_credit {
        return Err("Total debit must equal total credit".to_string());
    }
    if total_debit == 0 {
        return Err("At least one line is required".to_string());
    }

    conn.execute(
        "INSERT INTO journal_entries(entry_no, date, memo, ref_type, ref_id, created_by) VALUES(?,?,?,?,?,?)",
        rusqlite::params![
            entry_no,
            input.date,
            input.memo,
            input.ref_type,
            input.ref_id,
            chrono::Utc::now().to_string(),
        ],
    )
    .map_err(|e| e.to_string())?;
    let entry_id = conn.last_insert_rowid();

    for line in &input.lines {
        conn.execute(
            "INSERT INTO journal_entry_lines(entry_id, account_code, debit_milli, credit_milli, memo) VALUES(?,?,?,?,?)",
            rusqlite::params![
                entry_id,
                line.account_code,
                line.debit_milli,
                line.credit_milli,
                line.memo
            ],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(entry_id)
}

#[tauri::command]
pub fn get_trial_balance(state: State<'_, DbState>) -> Result<Vec<TrialBalanceRow>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT a.code, a.name_ar, COALESCE(SUM(jel.debit_milli),0) as total_debit, COALESCE(SUM(jel.credit_milli),0) as total_credit FROM accounts a LEFT JOIN journal_entry_lines jel ON a.code=jel.account_code GROUP BY a.code HAVING total_debit != 0 OR total_credit != 0 ORDER BY a.code",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(TrialBalanceRow {
                account_code: row.get(0)?,
                account_name: row.get(1)?,
                debit_milli: row.get(2)?,
                credit_milli: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_balance_sheet(state: State<'_, DbState>) -> Result<Vec<BalanceSheetRow>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT a.type, a.code, a.name_ar, COALESCE(SUM(jel.debit_milli),0) - COALESCE(SUM(jel.credit_milli),0) as balance FROM accounts a LEFT JOIN journal_entry_lines jel ON a.code=jel.account_code WHERE a.type IN ('Asset','Liability','Equity') GROUP BY a.code ORDER BY a.type, a.code",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(BalanceSheetRow {
                r#type: row.get(0)?,
                code: row.get(1)?,
                name: row.get(2)?,
                balance_milli: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_income_statement(state: State<'_, DbState>) -> Result<Vec<IncomeStatementRow>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT a.type, a.code, a.name_ar, COALESCE(SUM(jel.credit_milli),0) - COALESCE(SUM(jel.debit_milli),0) as balance FROM accounts a LEFT JOIN journal_entry_lines jel ON a.code=jel.account_code WHERE a.type IN ('Revenue','Expense') GROUP BY a.code ORDER BY a.type, a.code",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(IncomeStatementRow {
                r#type: row.get(0)?,
                code: row.get(1)?,
                name: row.get(2)?,
                balance_milli: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}
