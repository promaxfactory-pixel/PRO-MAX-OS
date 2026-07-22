use crate::commands::rbac;
use crate::db::DbState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct Supplier {
    pub id: i64,
    pub code: Option<String>,
    pub name: String,
    pub contact: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub vat_number: Option<String>,
    pub currency: Option<String>,
    pub payment_terms: Option<String>,
    pub opening_balance_milli: i64,
    pub balance_milli: i64,
    pub notes: Option<String>,
    pub active: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateSupplierInput {
    pub name: String,
    pub code: Option<String>,
    pub contact: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub vat_number: Option<String>,
    pub currency: Option<String>,
    pub payment_terms: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSupplierInput {
    pub name: Option<String>,
    pub code: Option<String>,
    pub contact: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub vat_number: Option<String>,
    pub currency: Option<String>,
    pub payment_terms: Option<String>,
    pub notes: Option<String>,
    pub active: Option<i64>,
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
pub struct SupplierStatementData {
    pub supplier: Supplier,
    pub opening_balance_milli: i64,
    pub transactions: Vec<StatementTransaction>,
    pub closing_balance_milli: i64,
    pub total_debit_milli: i64,
    pub total_credit_milli: i64,
}

fn row_to_supplier(row: &rusqlite::Row) -> rusqlite::Result<Supplier> {
    Ok(Supplier {
        id: row.get(0)?,
        code: row.get(1)?,
        name: row.get(2)?,
        contact: row.get(3)?,
        phone: row.get(4)?,
        email: row.get(5)?,
        address: row.get(6)?,
        vat_number: row.get(7)?,
        currency: row.get(8)?,
        payment_terms: row.get(9)?,
        opening_balance_milli: row.get(10)?,
        balance_milli: row.get(11)?,
        notes: row.get(12)?,
        active: row.get(13)?,
    })
}

fn get_supplier_by_conn(conn: &rusqlite::Connection, id: i64) -> Result<Supplier, String> {
    conn.query_row(
        "SELECT id, code, name, contact, phone, email, address, vat_number, currency, payment_terms, opening_balance_milli, balance_milli, notes, active FROM suppliers WHERE id=?",
        [id],
        row_to_supplier,
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_suppliers(state: State<'_, DbState>) -> Result<Vec<Supplier>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, code, name, contact, phone, email, address, vat_number, currency, payment_terms, opening_balance_milli, balance_milli, notes, active FROM suppliers WHERE active=1 ORDER BY name"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], row_to_supplier).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_supplier(state: State<'_, DbState>, id: i64) -> Result<Supplier, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT id, code, name, contact, phone, email, address, vat_number, currency, payment_terms, opening_balance_milli, balance_milli, notes, active FROM suppliers WHERE id=?",
        [id],
        row_to_supplier,
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_supplier(state: State<'_, DbState>, input: CreateSupplierInput) -> Result<i64, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO suppliers(code, name, contact, phone, email, address, vat_number, currency, payment_terms, notes) VALUES(?,?,?,?,?,?,?,?,?,?)",
        rusqlite::params![
            input.code, input.name, input.contact, input.phone, input.email,
            input.address, input.vat_number, input.currency.unwrap_or_else(|| "OMR".into()),
            input.payment_terms, input.notes
        ],
    ).map_err(|e| e.to_string())?;
    let sid = conn.last_insert_rowid();
    let _ = rbac::log_audit(&conn, None, None, "create_supplier", "suppliers", Some(sid), None, Some(&input.name), None);
    Ok(sid)
}

#[tauri::command]
pub fn update_supplier(state: State<'_, DbState>, id: i64, input: UpdateSupplierInput) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut sets = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(v) = &input.name { sets.push("name=?"); params.push(Box::new(v.clone())); }
    if let Some(v) = &input.code { sets.push("code=?"); params.push(Box::new(v.clone())); }
    if let Some(v) = &input.contact { sets.push("contact=?"); params.push(Box::new(v.clone())); }
    if let Some(v) = &input.phone { sets.push("phone=?"); params.push(Box::new(v.clone())); }
    if let Some(v) = &input.email { sets.push("email=?"); params.push(Box::new(v.clone())); }
    if let Some(v) = &input.address { sets.push("address=?"); params.push(Box::new(v.clone())); }
    if let Some(v) = &input.vat_number { sets.push("vat_number=?"); params.push(Box::new(v.clone())); }
    if let Some(v) = &input.currency { sets.push("currency=?"); params.push(Box::new(v.clone())); }
    if let Some(v) = &input.payment_terms { sets.push("payment_terms=?"); params.push(Box::new(v.clone())); }
    if let Some(v) = &input.notes { sets.push("notes=?"); params.push(Box::new(v.clone())); }
    if let Some(v) = input.active { sets.push("active=?"); params.push(Box::new(v)); }

    if sets.is_empty() { return Err("لا توجد تعديلات".to_string()); }

    params.push(Box::new(id));
    let sql = format!("UPDATE suppliers SET {} WHERE id=?", sets.join(", "));
    conn.execute(&sql, rusqlite::params_from_iter(params.iter())).map_err(|e| e.to_string())?;
    let _ = rbac::log_audit(&conn, None, None, "update_supplier", "suppliers", Some(id), None, None, None);
    Ok("تم التحديث بنجاح".to_string())
}

#[tauri::command]
pub fn delete_supplier(state: State<'_, DbState>, id: i64) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("UPDATE suppliers SET active=0 WHERE id=?", [id]).map_err(|e| e.to_string())?;
    let _ = rbac::log_audit(&conn, None, None, "delete_supplier", "suppliers", Some(id), None, None, None);
    Ok("تم الحذف بنجاح".to_string())
}

#[tauri::command]
pub fn get_supplier_statement(
    state: State<'_, DbState>,
    supplier_id: i64,
    from_date: Option<String>,
    to_date: Option<String>,
) -> Result<SupplierStatementData, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let supplier = get_supplier_by_conn(&conn, supplier_id)?;
    let from = from_date.unwrap_or_else(|| "2000-01-01".into());
    let to = to_date.unwrap_or_else(|| "2099-12-31".into());

    let mut transactions: Vec<StatementTransaction> = Vec::new();

    // Purchases
    {
        let mut stmt = conn.prepare(
            "SELECT p.date, p.purchase_no, p.total_milli, p.notes FROM purchases p WHERE p.supplier_id=? AND p.date BETWEEN ? AND ? AND p.status != 'Void' ORDER BY p.date ASC"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map(rusqlite::params![supplier_id, from, to], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        }).map_err(|e| e.to_string())?;
        for r in rows {
            let (date, purchase_no, total, notes) = r.map_err(|e| e.to_string())?;
            transactions.push(StatementTransaction {
                date,
                ref_no: purchase_no,
                txn_type: "purchase".to_string(),
                debit_milli: 0,
                credit_milli: total,
                balance_milli: 0,
                notes,
            });
        }
    }

    // Supplier payments
    {
        let mut stmt = conn.prepare(
            "SELECT sp.date, sp.receipt_no, sp.amount_milli, sp.method, sp.notes FROM supplier_payments sp WHERE sp.supplier_id=? AND sp.date BETWEEN ? AND ? AND sp.status != 'Void' ORDER BY sp.date ASC"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map(rusqlite::params![supplier_id, from, to], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
            ))
        }).map_err(|e| e.to_string())?;
        for r in rows {
            let (date, receipt_no, amount, method, notes) = r.map_err(|e| e.to_string())?;
            let note_str = notes.map(|n| format!("{} - {}", method.clone().unwrap_or_default(), n)).unwrap_or_else(|| method.unwrap_or_default());
            transactions.push(StatementTransaction {
                date,
                ref_no: receipt_no,
                txn_type: "payment".to_string(),
                debit_milli: amount,
                credit_milli: 0,
                balance_milli: 0,
                notes: if note_str.is_empty() { None } else { Some(note_str) },
            });
        }
    }

    // Sort by date
    transactions.sort_by(|a, b| a.date.cmp(&b.date));

    // Calculate running balance (opening_balance is what we owe the supplier)
    let opening = supplier.opening_balance_milli;
    let mut balance = opening;
    let mut total_debit: i64 = 0;
    let mut total_credit: i64 = 0;

    for txn in &mut transactions {
        // Debit = payments to supplier (reduces what we owe), Credit = purchases (increases what we owe)
        balance += txn.credit_milli - txn.debit_milli;
        txn.balance_milli = balance;
        total_debit += txn.debit_milli;
        total_credit += txn.credit_milli;
    }

    Ok(SupplierStatementData {
        supplier,
        opening_balance_milli: opening,
        transactions,
        closing_balance_milli: balance,
        total_debit_milli: total_debit,
        total_credit_milli: total_credit,
    })
}
