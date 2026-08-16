use crate::commands::rbac;
use crate::db::{next_sequence, DbState};
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct Customer {
    pub id: i64,
    pub code: Option<String>,
    pub name: String,
    pub ctype: Option<String>,
    pub contact: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub vat_number: Option<String>,
    pub credit_limit_milli: i64,
    pub payment_terms: Option<String>,
    pub payment_terms_days: i64,
    pub opening_balance_milli: i64,
    pub balance_milli: i64,
    pub notes: Option<String>,
    pub active: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateCustomerInput {
    pub name: String,
    pub code: Option<String>,
    pub ctype: Option<String>,
    pub contact: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub vat_number: Option<String>,
    pub credit_limit_milli: Option<i64>,
    pub payment_terms: Option<String>,
    pub payment_terms_days: Option<i64>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCustomerInput {
    pub name: Option<String>,
    pub code: Option<String>,
    pub ctype: Option<String>,
    pub contact: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub vat_number: Option<String>,
    pub credit_limit_milli: Option<i64>,
    pub payment_terms: Option<String>,
    pub payment_terms_days: Option<i64>,
    pub notes: Option<String>,
    pub active: Option<i64>,
}

fn row_to_customer(row: &rusqlite::Row) -> rusqlite::Result<Customer> {
    Ok(Customer {
        id: row.get(0)?,
        code: row.get(1)?,
        name: row.get(2)?,
        ctype: row.get(3)?,
        contact: row.get(4)?,
        phone: row.get(5)?,
        email: row.get(6)?,
        address: row.get(7)?,
        vat_number: row.get(8)?,
        credit_limit_milli: row.get(9)?,
        payment_terms: row.get(10)?,
        payment_terms_days: row.get(11)?,
        opening_balance_milli: row.get(12)?,
        balance_milli: row.get(13)?,
        notes: row.get(14)?,
        active: row.get(15)?,
    })
}

#[tauri::command]
pub fn list_customers(state: State<'_, DbState>) -> Result<Vec<Customer>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn.prepare(
        "SELECT id, code, name, ctype, contact, phone, email, address, vat_number, credit_limit_milli, payment_terms, payment_terms_days, opening_balance_milli, balance_milli, notes, active FROM customers WHERE active=1 ORDER BY name"
    )?;
    let rows = stmt.query_map([], row_to_customer)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

#[tauri::command]
pub fn get_customer(state: State<'_, DbState>, id: i64) -> Result<Customer, AppError> {
    let conn = state.0.lock()?;
    conn.query_row(
        "SELECT id, code, name, ctype, contact, phone, email, address, vat_number, credit_limit_milli, payment_terms, payment_terms_days, opening_balance_milli, balance_milli, notes, active FROM customers WHERE id=?",
        [id],
        row_to_customer,
    ).map_err(|_| AppError::not_found("العميل غير موجود"))
}

fn get_customer_by_conn(conn: &rusqlite::Connection, id: i64) -> Result<Customer, AppError> {
    conn.query_row(
        "SELECT id, code, name, ctype, contact, phone, email, address, vat_number, credit_limit_milli, payment_terms, payment_terms_days, opening_balance_milli, balance_milli, notes, active FROM customers WHERE id=?",
        [id],
        row_to_customer,
    ).map_err(|_| AppError::not_found("العميل غير موجود"))
}

#[tauri::command]
pub fn create_customer(state: State<'_, DbState>, user_id: i64, input: CreateCustomerInput) -> Result<i64, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "manager"])?;
    conn.execute(
        "INSERT INTO customers(code, name, ctype, contact, phone, email, address, vat_number, credit_limit_milli, payment_terms, payment_terms_days, notes) VALUES(?,?,?,?,?,?,?,?,?,?,?,?)",
        rusqlite::params![
            input.code, input.name, input.ctype.unwrap_or_else(|| "credit".into()),
            input.contact, input.phone, input.email, input.address, input.vat_number,
            input.credit_limit_milli.unwrap_or(0), input.payment_terms,
            input.payment_terms_days.unwrap_or(30), input.notes
        ],
    )?;
    let cid = conn.last_insert_rowid();
    let _ = rbac::log_audit(&conn, None, None, "create_customer", "customers", Some(cid), None, Some(&input.name), None);
    Ok(cid)
}

#[tauri::command]
pub fn update_customer(state: State<'_, DbState>, user_id: i64, id: i64, input: UpdateCustomerInput) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "manager"])?;
    let mut sets = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(v) = &input.name { sets.push("name=?"); params.push(Box::new(v.clone())); }
    if let Some(v) = &input.code { sets.push("code=?"); params.push(Box::new(v.clone())); }
    if let Some(v) = &input.ctype { sets.push("ctype=?"); params.push(Box::new(v.clone())); }
    if let Some(v) = &input.contact { sets.push("contact=?"); params.push(Box::new(v.clone())); }
    if let Some(v) = &input.phone { sets.push("phone=?"); params.push(Box::new(v.clone())); }
    if let Some(v) = &input.email { sets.push("email=?"); params.push(Box::new(v.clone())); }
    if let Some(v) = &input.address { sets.push("address=?"); params.push(Box::new(v.clone())); }
    if let Some(v) = &input.vat_number { sets.push("vat_number=?"); params.push(Box::new(v.clone())); }
    if let Some(v) = input.credit_limit_milli { sets.push("credit_limit_milli=?"); params.push(Box::new(v)); }
    if let Some(v) = &input.payment_terms { sets.push("payment_terms=?"); params.push(Box::new(v.clone())); }
    if let Some(v) = input.payment_terms_days { sets.push("payment_terms_days=?"); params.push(Box::new(v)); }
    if let Some(v) = &input.notes { sets.push("notes=?"); params.push(Box::new(v.clone())); }
    if let Some(v) = input.active { sets.push("active=?"); params.push(Box::new(v)); }

    if sets.is_empty() { return Err(AppError::validation("لا توجد تعديلات")); }

    params.push(Box::new(id));
    let sql = format!("UPDATE customers SET {} WHERE id=?", sets.join(", "));
    conn.execute(&sql, rusqlite::params_from_iter(params.iter()))?;
    let _ = rbac::log_audit(&conn, None, None, "update_customer", "customers", Some(id), None, None, None);
    Ok("تم التحديث بنجاح".to_string())
}

#[tauri::command]
pub fn delete_customer(state: State<'_, DbState>, user_id: i64, id: i64) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "manager"])?;
    conn.execute("UPDATE customers SET active=0 WHERE id=?", [id])?;
    let _ = rbac::log_audit(&conn, None, None, "delete_customer", "customers", Some(id), None, None, None);
    Ok("تم الحذف بنجاح".to_string())
}

#[derive(Debug, Deserialize)]
pub struct CreateCustomerPaymentInput {
    pub date: String,
    pub amount_milli: i64,
    pub method: Option<String>,
    pub cashbank_id: Option<i64>,
    pub reference: Option<String>,
    pub notes: Option<String>,
}

#[tauri::command]
pub fn create_customer_payment(
    state: State<'_, DbState>,
    user_id: i64,
    customer_id: i64,
    input: CreateCustomerPaymentInput,
) -> Result<i64, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    if input.amount_milli <= 0 {
        return Err(AppError::validation("المبلغ يجب أن يكون أكبر من صفر"));
    }
    let tx = conn.transaction()?;

    let customer_exists: i64 = tx
        .query_row("SELECT COUNT(*) FROM customers WHERE id=?1", [customer_id], |r| r.get(0))?;
    if customer_exists == 0 {
        return Err(AppError::not_found("العميل غير موجود"));
    }

    let year = chrono::Utc::now().format("%Y").to_string();
    let seq = next_sequence(&tx, "RCP", &year)?;
    let rec_no = format!("RCP-{}-{:04}", year, seq);

    let created_by: String = tx
        .query_row("SELECT username FROM users WHERE id=?", [user_id], |r| r.get(0))
        .unwrap_or_else(|_| "user".into());

    let method = input.method.clone().unwrap_or_else(|| "cash".into());
    tx.execute(
        "INSERT INTO customer_payments(rec_no, date, customer_id, amount_milli, method, cashbank_id, reference, notes, created_by, created_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9, datetime('now'))",
        rusqlite::params![
            rec_no,
            input.date.clone(),
            customer_id,
            input.amount_milli,
            method,
            input.cashbank_id,
            input.reference,
            input.notes,
            created_by,
        ],
    )?;
    let payment_id = tx.last_insert_rowid();

    tx.execute(
        "UPDATE customers SET balance_milli = balance_milli - ?1 WHERE id=?2",
        [input.amount_milli, customer_id],
    )?;

    // Spread the payment across the customer's open (posted, unpaid) credit invoices,
    // oldest first (FIFO), mirroring the supplier-side allocation. This keeps
    // per-invoice outstanding amounts (and thus aging/dunning) accurate.
    allocate_customer_payment_fifo(&tx, customer_id, input.amount_milli, None)?;

    let cash_account = crate::commands::accounting::resolve_cash_account(&tx, input.cashbank_id, &method)?;
    let lines: Vec<(String, i64, i64, Option<String>)> = vec![
        (cash_account, input.amount_milli, 0, Some("سند قبض".to_string())),
        ("1200".to_string(), 0, input.amount_milli, None),
    ];
    let journal_id = crate::commands::accounting::post_to_journal(
        &tx,
        "customer_payment",
        payment_id,
        &input.date,
        &format!("سند قبض {}", rec_no),
        &lines,
        &created_by,
    )?;
    tx.execute(
        "UPDATE customer_payments SET journal_id=?1 WHERE id=?2",
        rusqlite::params![journal_id, payment_id],
    )?;

    let _ = rbac::log_audit(&tx, Some(user_id), None, "create_customer_payment", "customer_payments", Some(payment_id), None, None, None);
    tx.commit()?;
    Ok(payment_id)
}

/// Spreads `amount` across the customer's open (posted, unpaid) credit invoices,
/// oldest first (FIFO). `exclude_id` is skipped (used when redistributing the
/// payments of an invoice being voided). Returns the leftover that no open
/// invoice absorbed (an on-account credit). Mirrors `allocate_payment_fifo`.
pub(crate) fn allocate_customer_payment_fifo(
    conn: &rusqlite::Connection,
    customer_id: i64,
    mut amount: i64,
    exclude_id: Option<i64>,
) -> Result<i64, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, total_milli, paid_milli FROM sales_invoices
         WHERE customer_id=?1 AND LOWER(status) = 'posted' AND LOWER(COALESCE(payment_type,'credit')) = 'credit'
               AND total_milli > paid_milli
         ORDER BY date ASC, id ASC",
    )?;
    let rows = stmt.query_map([customer_id], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, i64>(2)?))
    })?;
    let open_invoices: Vec<(i64, i64, i64)> = rows.collect::<Result<_, _>>()?;
    drop(stmt);

    for (inv_id, total, paid) in open_invoices {
        if amount <= 0 {
            break;
        }
        if Some(inv_id) == exclude_id {
            continue;
        }
        let outstanding = total - paid;
        let apply = amount.min(outstanding);
        if apply > 0 {
            conn.execute(
                "UPDATE sales_invoices SET paid_milli = paid_milli + ?1 WHERE id = ?2",
                rusqlite::params![apply, inv_id],
            )?;
            amount -= apply;
        }
    }
    Ok(amount)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatementTransaction {
    pub date: String,
    pub ref_no: Option<String>,
    pub txn_type: String,
    pub debit_milli: i64,
    pub credit_milli: i64,
    pub balance_milli: i64,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerStatementData {
    pub customer: Customer,
    pub opening_balance_milli: i64,
    pub transactions: Vec<StatementTransaction>,
    pub closing_balance_milli: i64,
    pub total_debit_milli: i64,
    pub total_credit_milli: i64,
}

#[tauri::command]
pub fn get_customer_statement(
    state: State<'_, DbState>,
    customer_id: i64,
    from_date: Option<String>,
    to_date: Option<String>,
) -> Result<CustomerStatementData, AppError> {
    let conn = state.0.lock()?;
    let customer = get_customer_by_conn(&conn, customer_id)?;
    let from = from_date.unwrap_or_else(|| "2000-01-01".into());
    let to = to_date.unwrap_or_else(|| "2099-12-31".into());

    let mut transactions: Vec<StatementTransaction> = Vec::new();

    {
        let mut stmt = conn.prepare(
            "SELECT si.date, si.inv_no, si.total_milli, si.discount_milli, si.notes FROM sales_invoices si
             WHERE si.customer_id=? AND si.date BETWEEN ? AND ? AND LOWER(si.status) = 'posted' AND LOWER(COALESCE(si.payment_type,'credit')) = 'credit'
             ORDER BY si.date ASC"
        )?;
        let rows = stmt.query_map(rusqlite::params![customer_id, from, to], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, Option<String>>(4)?,
            ))
        })?;
        for r in rows {
            let (date, inv_no, total, discount, notes) = r?;
            let amount = total - discount;
            transactions.push(StatementTransaction {
                date,
                ref_no: inv_no,
                txn_type: "invoice".to_string(),
                debit_milli: amount,
                credit_milli: 0,
                balance_milli: 0,
                notes,
            });
        }
    }

    {
        let mut stmt = conn.prepare(
            "SELECT cp.date, cp.rec_no, cp.amount_milli, cp.method, cp.notes FROM customer_payments cp WHERE cp.customer_id=? AND cp.date BETWEEN ? AND ? ORDER BY cp.date ASC"
        )?;
        let rows = stmt.query_map(rusqlite::params![customer_id, from, to], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
            ))
        })?;
        for r in rows {
            let (date, receipt_no, amount, method, notes) = r?;
            let note_str = notes.map(|n| format!("{} - {}", method.clone().unwrap_or_default(), n)).unwrap_or_else(|| method.unwrap_or_default());
            transactions.push(StatementTransaction {
                date,
                ref_no: receipt_no,
                txn_type: "payment".to_string(),
                debit_milli: 0,
                credit_milli: amount,
                balance_milli: 0,
                notes: if note_str.is_empty() { None } else { Some(note_str) },
            });
        }
    }

    {
        let mut stmt = conn.prepare(
            "SELECT cn.date, cn.cn_no, cn.total_milli, cn.reason FROM credit_notes cn WHERE cn.customer_id=? AND cn.date BETWEEN ? AND ? AND cn.status != 'Void' ORDER BY cn.date ASC"
        )?;
        let rows = stmt.query_map(rusqlite::params![customer_id, from, to], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        })?;
        for r in rows {
            let (date, cn_no, total, reason) = r?;
            transactions.push(StatementTransaction {
                date,
                ref_no: cn_no,
                txn_type: "credit_note".to_string(),
                debit_milli: 0,
                credit_milli: total,
                balance_milli: 0,
                notes: reason,
            });
        }
    }

    transactions.sort_by(|a, b| a.date.cmp(&b.date));

    let opening = customer.opening_balance_milli;
    let mut balance = opening;
    let mut total_debit: i64 = 0;
    let mut total_credit: i64 = 0;

    for txn in &mut transactions {
        balance += txn.debit_milli - txn.credit_milli;
        txn.balance_milli = balance;
        total_debit += txn.debit_milli;
        total_credit += txn.credit_milli;
    }

    Ok(CustomerStatementData {
        customer,
        opening_balance_milli: opening,
        transactions,
        closing_balance_milli: balance,
        total_debit_milli: total_debit,
        total_credit_milli: total_credit,
    })
}
