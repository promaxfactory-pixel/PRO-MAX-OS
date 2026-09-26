use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::rbac;
use crate::db::{next_sequence, DbState};
use crate::error::AppError;

#[derive(Debug, Deserialize)]
pub struct UpdateSpendInput {
    pub txn_id: i64,
    pub amount_milli: Option<i64>,
    pub category: Option<String>,
    pub reference: Option<String>,
    pub notes: Option<String>,
    pub date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFundInput {
    pub petty_id: i64,
    pub name: Option<String>,
    pub responsible: Option<String>,
    pub spending_limit_milli: Option<i64>,
    pub notes: Option<String>,
}

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

#[derive(Debug, Serialize, Deserialize)]
pub struct CustodyReconciliation {
    pub subledger_balance_milli: i64,
    pub gl_balance_milli: i64,
    pub difference_milli: i64,
    pub is_reconciled: bool,
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
    pub vat_milli: Option<i64>,
    pub account_code: Option<String>,
    pub category: Option<String>,
    pub vendor: Option<String>,
    pub reference: Option<String>,
    pub notes: Option<String>,
    pub date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddFundInput {
    pub petty_id: i64,
    pub amount_milli: i64,
    pub date: String,
    pub source_type: Option<String>,
    pub cashbank_id: Option<i64>,
    pub method: Option<String>,
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
    user_id: i64,
    input: CreateFundInput,
) -> Result<CustodyAccount, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant"])?;
    let opening = input.opening_balance_milli.unwrap_or(0);
    if opening < 0 {
        return Err(AppError::validation("الرصيد الافتتاحي لا يمكن أن يكون سالبًا"));
    }

    let seq: i64 = conn.query_row(
        "SELECT COALESCE(MAX(id), 0) + 1 FROM petty_cash_accounts", [], |row| row.get(0)
    )?;
    let code = format!("PC-{:04}", seq);
    let tx = conn.transaction()?;

    tx.execute(
        "INSERT INTO petty_cash_accounts
         (code, name, responsible, employee_id, spending_limit_milli, balance_milli,
          account_code, notes, created_at)
         VALUES (?1,?2,?3,?4,?5,?6,'1110',?7,datetime('now'))",
        params![
            code, input.name, input.responsible, input.employee_id,
            input.spending_limit_milli.unwrap_or(0), opening, input.notes
        ],
    )?;
    let id = tx.last_insert_rowid();

    if opening > 0 {
        // Opening balance is a controlled migration/opening entry, not operating income.
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let lines = vec![
            ("1110".to_string(), opening, 0, Some("رصيد افتتاحي للعهدة".to_string())),
            ("3000".to_string(), 0, opening, Some("حقوق ملكية/رصيد افتتاحي".to_string())),
        ];
        let journal_id = crate::commands::accounting::post_to_journal(
            &tx, "custody_opening", id, &today,
            &format!("رصيد افتتاحي {}", code), &lines, &user_id.to_string()
        )?;
        tx.execute(
            "INSERT INTO petty_cash_transactions
             (ts, petty_id, ttype, debit_milli, credit_milli, balance_milli,
              account_code, notes, journal_id, user_id)
             VALUES(datetime('now'),?1,'Opening',?2,0,?2,'3000','Opening balance',?3,?4)",
            params![id, opening, journal_id, user_id],
        )?;
    }

    let _ = rbac::log_audit(
        &tx, Some(user_id), None, "create_custody_fund", "petty_cash_accounts",
        Some(id), None, Some(&code), None
    );
    tx.commit()?;

    Ok(conn.query_row(
        "SELECT id, code, name, responsible, employee_id, spending_limit_milli,
                balance_milli, active, notes, created_at
         FROM petty_cash_accounts WHERE id=?1",
        [id],
        |row| Ok(CustodyAccount {
            id: row.get(0)?, code: row.get(1)?, name: row.get(2)?,
            responsible: row.get(3)?, employee_id: row.get(4)?,
            spending_limit_milli: row.get(5)?, balance_milli: row.get(6)?,
            active: row.get(7)?, notes: row.get(8)?, created_at: row.get(9)?
        }),
    )?)
}

#[tauri::command]
pub fn add_custody_funding(
    state: State<'_, DbState>,
    user_id: i64,
    input: AddFundInput,
) -> Result<CustodyAccount, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    if input.amount_milli <= 0 {
        return Err(AppError::validation("مبلغ تمويل العهدة يجب أن يكون أكبر من صفر"));
    }
    let tx = conn.transaction()?;

    let (current_balance, custody_account): (i64, String) = tx.query_row(
        "SELECT balance_milli, COALESCE(NULLIF(trim(account_code),''),'1110')
         FROM petty_cash_accounts WHERE id=?1 AND active=1",
        [input.petty_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    ).map_err(|_| AppError::not_found("حساب العهدة غير موجود"))?;

    let source_type = input.source_type.unwrap_or_else(|| "company".to_string()).to_lowercase();
    let method = input.method.unwrap_or_else(|| "cash".to_string());
    let source_account = match source_type.as_str() {
        "company" => crate::commands::accounting::resolve_cash_account(&tx, input.cashbank_id, &method)?,
        "owner_saif" => "2310".to_string(),
        "owner_abu_saif" => "2320".to_string(),
        _ => return Err(AppError::validation("مصدر تمويل العهدة غير معروف")),
    };

    let new_balance = current_balance.checked_add(input.amount_milli)
        .ok_or_else(|| AppError::validation("رصيد العهدة يتجاوز الحد المسموح"))?;
    let lines = vec![
        (custody_account.clone(), input.amount_milli, 0, Some("تمويل عهدة".to_string())),
        (source_account.clone(), 0, input.amount_milli, Some(format!("مصدر التمويل: {}", source_type))),
    ];
    let journal_id = crate::commands::accounting::post_to_journal(
        &tx, "custody_funding", input.petty_id, &input.date,
        "تمويل عهدة / صرف نثري", &lines, &user_id.to_string()
    )?;

    tx.execute(
        "UPDATE petty_cash_accounts SET balance_milli=?1 WHERE id=?2",
        params![new_balance, input.petty_id],
    )?;
    tx.execute(
        "INSERT INTO petty_cash_transactions
         (ts, petty_id, ttype, debit_milli, credit_milli, balance_milli,
          account_code, cashbank_id, reference, notes, journal_id, user_id)
         VALUES(?1,?2,'Fund',?3,0,?4,?5,?6,?7,?8,?9,?10)",
        params![
            format!("{} 12:00:00", input.date), input.petty_id, input.amount_milli,
            new_balance, source_account, input.cashbank_id, input.reference,
            input.notes, journal_id, user_id
        ],
    )?;

    let _ = rbac::log_audit(
        &tx, Some(user_id), None, "add_custody_funding", "petty_cash_accounts",
        Some(input.petty_id), None,
        Some(&format!("source={} amount={}", source_type, input.amount_milli)), None
    );
    tx.commit()?;
    drop(conn);

    get_custody_account(state, input.petty_id)
}

#[tauri::command]
pub fn create_custody_spend(
    state: State<'_, DbState>,
    user_id: i64,
    input: CreateSpendInput,
) -> Result<CustodyAccount, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    if input.amount_milli <= 0 {
        return Err(AppError::validation("صافي المصروف يجب أن يكون أكبر من صفر"));
    }
    let vat = input.vat_milli.unwrap_or(0);
    if vat < 0 {
        return Err(AppError::validation("قيمة الضريبة لا يمكن أن تكون سالبة"));
    }
    let total = input.amount_milli.checked_add(vat)
        .ok_or_else(|| AppError::validation("إجمالي المصروف يتجاوز الحد المسموح"))?;
    let tx = conn.transaction()?;

    let (current_balance, custody_account): (i64, String) = tx.query_row(
        "SELECT balance_milli, COALESCE(NULLIF(trim(account_code),''),'1110')
         FROM petty_cash_accounts WHERE id=?1 AND active=1",
        [input.petty_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    ).map_err(|_| AppError::not_found("حساب العهدة غير موجود"))?;
    if current_balance < total {
        return Err(AppError::validation("رصيد العهدة غير كافٍ"));
    }

    let expense_account = input.account_code.clone()
        .filter(|x| !x.trim().is_empty())
        .unwrap_or_else(|| "5200".to_string());
    let acct_type: String = tx.query_row(
        "SELECT type FROM accounts WHERE code=?1", [&expense_account], |r| r.get(0)
    ).map_err(|_| AppError::validation("حساب المصروف غير موجود"))?;
    if acct_type.to_lowercase() != "expense" {
        return Err(AppError::validation("الحساب المحدد ليس حساب مصروف"));
    }

    let date = input.date.clone()
        .filter(|x| !x.trim().is_empty())
        .unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    let date_only = date.get(..10).unwrap_or(&date).to_string();
    let year = date_only.get(..4).unwrap_or("0000").to_string();
    let seq = next_sequence(&tx, "EXP", &year)?;
    let exp_no = format!("EXP-{}-{:04}", year, seq);

    tx.execute(
        "INSERT INTO expenses
         (exp_no, date, category, account_code, amount_milli, vat_milli, method,
          vendor, reference, notes, approval_status, paid_from_source,
          source_account_code, petty_id, reimbursement_status, created_by, created_at)
         VALUES(?1,?2,?3,?4,?5,?6,'cash',?7,?8,?9,'approved','custody',?10,?11,'none',?12,datetime('now'))",
        params![
            exp_no, date_only, input.category, expense_account, input.amount_milli,
            vat, input.vendor, input.reference, input.notes, custody_account,
            input.petty_id, user_id.to_string()
        ],
    )?;
    let expense_id = tx.last_insert_rowid();

    let mut lines = vec![
        (expense_account.clone(), input.amount_milli, 0, Some("صافي مصروف من العهدة".to_string())),
    ];
    if vat > 0 {
        lines.push(("2100".to_string(), vat, 0, Some("ضريبة مدخلات".to_string())));
    }
    lines.push((custody_account.clone(), 0, total, Some("صرف من العهدة".to_string())));
    let journal_id = crate::commands::accounting::post_to_journal(
        &tx, "expense", expense_id, &date_only, &format!("مصروف عهدة {}", exp_no),
        &lines, &user_id.to_string()
    )?;

    let new_balance = current_balance - total;
    tx.execute(
        "UPDATE petty_cash_accounts SET balance_milli=?1 WHERE id=?2",
        params![new_balance, input.petty_id],
    )?;
    tx.execute(
        "INSERT INTO petty_cash_transactions
         (ts, petty_id, ttype, debit_milli, credit_milli, balance_milli, category,
          account_code, expense_id, reference, notes, journal_id, user_id)
         VALUES(?1,?2,'Spend',0,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
        params![
            format!("{} 12:00:00", date_only), input.petty_id, total, new_balance,
            input.category, expense_account, expense_id, input.reference, input.notes,
            journal_id, user_id
        ],
    )?;
    let custody_txn_id = tx.last_insert_rowid();
    tx.execute(
        "UPDATE expenses SET journal_id=?1, custody_txn_id=?2 WHERE id=?3",
        params![journal_id, custody_txn_id, expense_id],
    )?;

    let _ = rbac::log_audit(
        &tx, Some(user_id), None, "create_custody_spend", "expenses",
        Some(expense_id), None,
        Some(&format!("custody={} total={}", input.petty_id, total)), None
    );
    tx.commit()?;
    drop(conn);

    get_custody_account(state, input.petty_id)
}

#[tauri::command]
pub fn create_custody_transfer(
    state: State<'_, DbState>,
    user_id: i64,
    input: CreateTransferInput,
) -> Result<Vec<CustodyAccount>, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant"])?;
    if input.amount_milli <= 0 {
        return Err(AppError::validation("مبلغ التحويل يجب أن يكون أكبر من صفر"));
    }
    if input.from_petty_id == input.to_petty_id {
        return Err(AppError::validation("لا يمكن التحويل من العهدة إلى نفسها"));
    }
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
        return Err(AppError::validation("الرصيد غير كافٍ في الحساب المصدر"));
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
pub fn get_custody_reconciliation(
    state: State<'_, DbState>,
) -> Result<CustodyReconciliation, AppError> {
    let conn = state.0.lock()?;
    let subledger: i64 = conn.query_row(
        "SELECT COALESCE(SUM(balance_milli),0) FROM petty_cash_accounts WHERE active=1",
        [], |r| r.get(0)
    ).unwrap_or(0);
    let (debit, credit): (i64, i64) = conn.query_row(
        "SELECT COALESCE(SUM(debit_milli),0), COALESCE(SUM(credit_milli),0)
         FROM journal_entry_lines WHERE account_code='1110'",
        [], |r| Ok((r.get(0)?, r.get(1)?))
    ).unwrap_or((0,0));
    let gl = debit - credit;
    let difference = subledger - gl;
    Ok(CustodyReconciliation {
        subledger_balance_milli: subledger,
        gl_balance_milli: gl,
        difference_milli: difference,
        is_reconciled: difference == 0,
    })
}

#[tauri::command]
pub fn get_custody_statement(
    state: State<'_, DbState>,
    petty_id: i64,
    date_from: Option<String>,
    date_to: Option<String>,
) -> Result<Vec<CustodyTransaction>, AppError> {
    let conn = state.0.lock()?;
    let mut sql = String::from(
        "SELECT id, ts, petty_id, ttype, debit_milli, credit_milli, balance_milli, category, reference, notes, journal_id
         FROM petty_cash_transactions WHERE petty_id = ?1"
    );
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(petty_id)];
    if let Some(ref d) = date_from {
        sql.push_str(" AND ts >= ?");
        param_values.push(Box::new(format!("{} 00:00:00", d)));
    }
    if let Some(ref d) = date_to {
        sql.push_str(" AND ts <= ?");
        param_values.push(Box::new(format!("{} 23:59:59", d)));
    }
    sql.push_str(" ORDER BY ts ASC, id ASC");

    let mut stmt = conn.prepare(&sql)?;
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
    let rows = stmt
        .query_map(param_refs.as_slice(), |row| {
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

#[tauri::command]
pub fn update_custody_spend(
    state: State<'_, DbState>,
    user_id: i64,
    input: UpdateSpendInput,
) -> Result<(), AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant"])?;

    if input.amount_milli.is_some() || input.date.is_some() {
        return Err(AppError::validation(
            "لا يمكن تغيير قيمة أو تاريخ حركة عهدة مالية؛ استخدم تصحيح/عكس موثق حتى يظل الأستاذ متطابقًا"
        ));
    }

    let expense_id: Option<i64> = conn.query_row(
        "SELECT expense_id FROM petty_cash_transactions WHERE id=?1",
        [input.txn_id],
        |r| r.get(0),
    ).unwrap_or(None);

    conn.execute(
        "UPDATE petty_cash_transactions
         SET category=COALESCE(?1,category), reference=COALESCE(?2,reference),
             notes=COALESCE(?3,notes)
         WHERE id=?4",
        params![input.category, input.reference, input.notes, input.txn_id],
    )?;

    if let Some(exp_id) = expense_id {
        conn.execute(
            "UPDATE expenses
             SET category=COALESCE(?1,category), reference=COALESCE(?2,reference),
                 notes=COALESCE(?3,notes)
             WHERE id=?4",
            params![input.category, input.reference, input.notes, exp_id],
        )?;
    }

    let _ = rbac::log_audit(
        &conn, Some(user_id), None, "update_custody_spend",
        "petty_cash_transactions", Some(input.txn_id), None, None, None
    );
    Ok(())
}

#[tauri::command]
pub fn update_custody_fund(
    state: State<'_, DbState>,
    user_id: i64,
    input: UpdateFundInput,
) -> Result<(), AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant"])?;
    conn.execute(
        "UPDATE petty_cash_accounts SET name = COALESCE(?1, name), responsible = COALESCE(?2, responsible), spending_limit_milli = COALESCE(?3, spending_limit_milli), notes = COALESCE(?4, notes) WHERE id = ?5",
        params![input.name, input.responsible, input.spending_limit_milli, input.notes, input.petty_id],
    )?;
    let _ = rbac::log_audit(&conn, Some(user_id), None, "update_custody_fund", "petty_cash_accounts", Some(input.petty_id), None, None, None);
    Ok(())
}
