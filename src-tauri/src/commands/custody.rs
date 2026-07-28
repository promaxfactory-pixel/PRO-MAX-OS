use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::rbac;
use crate::db::DbState;
use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct CustodyAccount {
    pub id: i64,
    pub code: Option<String>,
    pub name: String,
    pub responsible: Option<String>,
    pub employee_id: Option<i64>,
    pub spending_limit_milli: i64,
    pub balance_milli: i64,
    pub active: i64,
    pub notes: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustodyTransaction {
    pub id: i64,
    pub ts: String,
    pub petty_id: i64,
    pub ttype: Option<String>,
    pub debit_milli: i64,
    pub credit_milli: i64,
    pub balance_milli: i64,
    pub category: Option<String>,
    pub reference: Option<String>,
    pub notes: Option<String>,
    pub journal_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFundInput {
    pub name: String,
    pub responsible: Option<String>,
    pub employee_id: Option<i64>,
    pub spending_limit_milli: Option<i64>,
    pub opening_balance_milli: Option<i64>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSpendInput {
    pub petty_id: i64,
    pub amount_milli: i64,
    pub category: Option<String>,
    pub reference: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTransferInput {
    pub from_petty_id: i64,
    pub to_petty_id: i64,
    pub amount_milli: i64,
    pub notes: Option<String>,
}

#[tauri::command]
pub fn list_custody_accounts(state: State<'_, DbState>) -> Result<Vec<CustodyAccount>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, code, name, responsible, employee_id, spending_limit_milli, balance_milli, active, notes, created_at
             FROM petty_cash_accounts WHERE active = 1 ORDER BY name",
        )?;

    let rows = stmt
        .query_map([], |row| {
            Ok(CustodyAccount {
                id: row.get(0)?,
                code: row.get(1)?,
                name: row.get(2)?,
                responsible: row.get(3)?,
                employee_id: row.get(4)?,
                spending_limit_milli: row.get(5)?,
                balance_milli: row.get(6)?,
                active: row.get(7)?,
                notes: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?;

    let mut accounts = Vec::new();
    for row in rows {
        accounts.push(row?);
    }
    Ok(accounts)
}

#[tauri::command]
pub fn get_custody_account(
    state: State<'_, DbState>,
    id: i64,
) -> Result<CustodyAccount, AppError> {
    let conn = state.0.lock()?;
    Ok(conn.query_row(
        "SELECT id, code, name, responsible, employee_id, spending_limit_milli, balance_milli, active, notes, created_at
         FROM petty_cash_accounts WHERE id = ?1",
        params![id],
        |row| {
            Ok(CustodyAccount {
                id: row.get(0)?,
                code: row.get(1)?,
                name: row.get(2)?,
                responsible: row.get(3)?,
                employee_id: row.get(4)?,
                spending_limit_milli: row.get(5)?,
                balance_milli: row.get(6)?,
                active: row.get(7)?,
                notes: row.get(8)?,
                created_at: row.get(9)?,
            })
        },
    )?)
}

#[tauri::command]
pub fn create_custody_fund(
    state: State<'_, DbState>,
    input: CreateFundInput,
) -> Result<CustodyAccount, AppError> {
    let mut conn = state.0.lock()?;

    let seq: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(id), 0) + 1 FROM petty_cash_accounts",
            [],
            |row| row.get(0),
        )?;
    let code = format!("PC-{:04}", seq);

    let opening = input.opening_balance_milli.unwrap_or(0);

    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO petty_cash_accounts (code, name, responsible, employee_id, spending_limit_milli, balance_milli, notes, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'))",
        params![
            code,
            input.name,
            input.responsible,
            input.employee_id,
            input.spending_limit_milli.unwrap_or(0),
            opening,
            input.notes,
        ],
    )?;

    let id = tx.last_insert_rowid();

    if opening > 0 {
        tx.execute(
            "INSERT INTO petty_cash_transactions (ts, petty_id, ttype, debit_milli, credit_milli, balance_milli, notes)
             VALUES (datetime('now'), ?1, 'Fund', ?2, 0, ?2, 'Opening balance')",
            params![id, opening],
        )?;
    }

    tx.commit()?;
    let _ = rbac::log_audit(&conn, None, None, "create_custody_fund", "petty_cash_accounts", Some(id), None, Some(&input.name), None);

    Ok(conn.query_row(
        "SELECT id, code, name, responsible, employee_id, spending_limit_milli, balance_milli, active, notes, created_at FROM petty_cash_accounts WHERE id=?1",
        params![id],
        |row| Ok(CustodyAccount { id: row.get(0)?, code: row.get(1)?, name: row.get(2)?, responsible: row.get(3)?, employee_id: row.get(4)?, spending_limit_milli: row.get(5)?, balance_milli: row.get(6)?, active: row.get(7)?, notes: row.get(8)?, created_at: row.get(9)? }),
    )?)
}

#[tauri::command]
pub fn create_custody_spend(
    state: State<'_, DbState>,
    input: CreateSpendInput,
) -> Result<CustodyAccount, AppError> {
    let mut conn = state.0.lock()?;
    let tx = conn.transaction()?;

    let current_balance: i64 = tx
        .query_row(
            "SELECT balance_milli FROM petty_cash_accounts WHERE id = ?1",
            params![input.petty_id],
            |row| row.get(0),
        )?;

    if current_balance < input.amount_milli {
        return Err(AppError::validation("Insufficient balance"));
    }

    let new_balance = current_balance - input.amount_milli;

    tx.execute(
        "UPDATE petty_cash_accounts SET balance_milli = ?1 WHERE id = ?2",
        params![new_balance, input.petty_id],
    )?;

    tx.execute(
        "INSERT INTO petty_cash_transactions (ts, petty_id, ttype, debit_milli, credit_milli, balance_milli, category, reference, notes)
         VALUES (datetime('now'), ?1, 'Spend', 0, ?2, ?3, ?4, ?5, ?6)",
        params![
            input.petty_id,
            input.amount_milli,
            new_balance,
            input.category,
            input.reference,
            input.notes,
        ],
    )?;

    tx.commit()?;
    let _ = rbac::log_audit(&conn, None, None, "create_custody_spend", "petty_cash_accounts", Some(input.petty_id), None, None, None);

    Ok(conn.query_row(
        "SELECT id, code, name, responsible, employee_id, spending_limit_milli, balance_milli, active, notes, created_at FROM petty_cash_accounts WHERE id=?1",
        params![input.petty_id],
        |row| Ok(CustodyAccount { id: row.get(0)?, code: row.get(1)?, name: row.get(2)?, responsible: row.get(3)?, employee_id: row.get(4)?, spending_limit_milli: row.get(5)?, balance_milli: row.get(6)?, active: row.get(7)?, notes: row.get(8)?, created_at: row.get(9)? }),
    )?)
}

#[tauri::command]
pub fn create_custody_transfer(
    state: State<'_, DbState>,
    input: CreateTransferInput,
) -> Result<Vec<CustodyAccount>, AppError> {
    let mut conn = state.0.lock()?;
    let tx = conn.transaction()?;

    let from_balance: i64 = tx
        .query_row(
            "SELECT balance_milli FROM petty_cash_accounts WHERE id = ?1",
            params![input.from_petty_id],
            |row| row.get(0),
        )?;

    let to_balance: i64 = tx
        .query_row(
            "SELECT balance_milli FROM petty_cash_accounts WHERE id = ?1",
            params![input.to_petty_id],
            |row| row.get(0),
        )?;

    if from_balance < input.amount_milli {
        return Err(AppError::validation("Insufficient balance in source account"));
    }

    let new_from = from_balance - input.amount_milli;
    let new_to = to_balance + input.amount_milli;

    tx.execute(
        "UPDATE petty_cash_accounts SET balance_milli = ?1 WHERE id = ?2",
        params![new_from, input.from_petty_id],
    )?;

    tx.execute(
        "UPDATE petty_cash_accounts SET balance_milli = ?1 WHERE id = ?2",
        params![new_to, input.to_petty_id],
    )?;

    tx.execute(
        "INSERT INTO petty_cash_transactions (ts, petty_id, ttype, debit_milli, credit_milli, balance_milli, counter_petty_id, notes)
         VALUES (datetime('now'), ?1, 'Transfer Out', 0, ?2, ?3, ?4, ?5)",
        params![
            input.from_petty_id,
            input.amount_milli,
            new_from,
            input.to_petty_id,
            input.notes,
        ],
    )?;

    tx.execute(
        "INSERT INTO petty_cash_transactions (ts, petty_id, ttype, debit_milli, credit_milli, balance_milli, counter_petty_id, notes)
         VALUES (datetime('now'), ?1, 'Transfer In', ?2, 0, ?3, ?4, ?5)",
        params![
            input.to_petty_id,
            input.amount_milli,
            new_to,
            input.from_petty_id,
            input.notes,
        ],
    )?;

    tx.commit()?;
    let _ = rbac::log_audit(&conn, None, None, "create_custody_transfer", "petty_cash_accounts", Some(input.from_petty_id), None, Some(&format!("->{} amt:{}", input.to_petty_id, input.amount_milli)), None);
    let from = conn.query_row(
        "SELECT id, code, name, responsible, employee_id, spending_limit_milli, balance_milli, active, notes, created_at FROM petty_cash_accounts WHERE id=?1",
        params![input.from_petty_id],
        |row| Ok(CustodyAccount { id: row.get(0)?, code: row.get(1)?, name: row.get(2)?, responsible: row.get(3)?, employee_id: row.get(4)?, spending_limit_milli: row.get(5)?, balance_milli: row.get(6)?, active: row.get(7)?, notes: row.get(8)?, created_at: row.get(9)? }),
    )?;
    let to = conn.query_row(
        "SELECT id, code, name, responsible, employee_id, spending_limit_milli, balance_milli, active, notes, created_at FROM petty_cash_accounts WHERE id=?1",
        params![input.to_petty_id],
        |row| Ok(CustodyAccount { id: row.get(0)?, code: row.get(1)?, name: row.get(2)?, responsible: row.get(3)?, employee_id: row.get(4)?, spending_limit_milli: row.get(5)?, balance_milli: row.get(6)?, active: row.get(7)?, notes: row.get(8)?, created_at: row.get(9)? }),
    )?;
    Ok(vec![from, to])
}

#[tauri::command]
pub fn get_custody_statement(
    state: State<'_, DbState>,
    petty_id: i64,
) -> Result<Vec<CustodyTransaction>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, ts, petty_id, ttype, debit_milli, credit_milli, balance_milli, category, reference, notes, journal_id
             FROM petty_cash_transactions WHERE petty_id = ?1 ORDER BY id DESC",
        )?;

    let rows = stmt
        .query_map(params![petty_id], |row| {
            Ok(CustodyTransaction {
                id: row.get(0)?,
                ts: row.get(1)?,
                petty_id: row.get(2)?,
                ttype: row.get(3)?,
                debit_milli: row.get(4)?,
                credit_milli: row.get(5)?,
                balance_milli: row.get(6)?,
                category: row.get(7)?,
                reference: row.get(8)?,
                notes: row.get(9)?,
                journal_id: row.get(10)?,
            })
        })?;

    let mut txns = Vec::new();
    for row in rows {
        txns.push(row?);
    }
    Ok(txns)
}
