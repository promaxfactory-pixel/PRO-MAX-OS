use crate::commands::rbac;
use crate::db::DbState;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct SalesInvoice {
    pub id: i64,
    pub inv_no: Option<String>,
    pub date: String,
    pub customer_id: i64,
    pub customer_name: Option<String>,
    pub payment_type: Option<String>,
    pub vat_enabled: i64,
    pub net_milli: i64,
    pub vat_milli: i64,
    pub discount_milli: i64,
    pub total_milli: i64,
    pub discount_reason: Option<String>,
    pub cogs_milli: i64,
    pub paid_milli: i64,
    pub status: String,
    pub notes: Option<String>,
    pub created_by: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceLine {
    pub id: i64,
    pub invoice_id: i64,
    pub product_id: i64,
    pub product_name: Option<String>,
    pub cartons: f64,
    pub cups_per_carton: i64,
    pub qty_cups: f64,
    pub unit_price_milli: i64,
    pub customs_price_milli: i64,
    pub line_net_milli: i64,
    pub vat_pct: f64,
    pub vat_milli: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateInvoiceInput {
    pub customer_id: i64,
    pub payment_type: Option<String>,
    pub notes: Option<String>,
    pub lines: Vec<CreateInvoiceLineInput>,
}

#[derive(Debug, Deserialize)]
pub struct CreateInvoiceLineInput {
    pub product_id: i64,
    pub cartons: f64,
    pub unit_price_milli: i64,
    pub customs_price_milli: Option<i64>,
}

struct ProductInfo {
    cups_per_carton: i64,
    vat_pct: f64,
}

#[tauri::command]
pub fn list_invoices(state: State<'_, DbState>) -> Result<Vec<SalesInvoice>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn.prepare(
        "SELECT si.id, si.inv_no, si.date, si.customer_id, c.name, si.payment_type, si.vat_enabled, si.net_milli, si.vat_milli, si.discount_milli, si.total_milli, si.discount_reason, si.cogs_milli, si.paid_milli, si.status, si.notes, si.created_by, si.created_at FROM sales_invoices si LEFT JOIN customers c ON si.customer_id=c.id ORDER BY si.id DESC"
    )?;
    
    let rows = stmt.query_map([], |row| {
        Ok(SalesInvoice {
            id: row.get(0)?,
            inv_no: row.get(1)?,
            date: row.get(2)?,
            customer_id: row.get(3)?,
            customer_name: row.get(4)?,
            payment_type: row.get(5)?,
            vat_enabled: row.get(6)?,
            net_milli: row.get(7)?,
            vat_milli: row.get(8)?,
            discount_milli: row.get(9)?,
            total_milli: row.get(10)?,
            discount_reason: row.get(11)?,
            cogs_milli: row.get(12)?,
            paid_milli: row.get(13)?,
            status: row.get(14)?,
            notes: row.get(15)?,
            created_by: row.get(16)?,
            created_at: row.get(17)?,
        })
    })?;
    
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[tauri::command]
pub fn get_invoice(state: State<'_, DbState>, id: i64) -> Result<SalesInvoice, AppError> {
    let conn = state.0.lock()?;
    Ok(conn.query_row(
        "SELECT si.id, si.inv_no, si.date, si.customer_id, c.name, si.payment_type, si.vat_enabled, si.net_milli, si.vat_milli, si.discount_milli, si.total_milli, si.discount_reason, si.cogs_milli, si.paid_milli, si.status, si.notes, si.created_by, si.created_at FROM sales_invoices si LEFT JOIN customers c ON si.customer_id=c.id WHERE si.id=?",
        [id],
        |row| {
            Ok(SalesInvoice {
                id: row.get(0)?, inv_no: row.get(1)?, date: row.get(2)?, customer_id: row.get(3)?,
                customer_name: row.get(4)?, payment_type: row.get(5)?, vat_enabled: row.get(6)?,
                net_milli: row.get(7)?, vat_milli: row.get(8)?, discount_milli: row.get(9)?,
                total_milli: row.get(10)?, discount_reason: row.get(11)?, cogs_milli: row.get(12)?,
                paid_milli: row.get(13)?, status: row.get(14)?, notes: row.get(15)?,
                created_by: row.get(16)?, created_at: row.get(17)?,
            })
        },
    )?)
}

#[tauri::command]
pub fn get_invoice_lines(state: State<'_, DbState>, invoice_id: i64) -> Result<Vec<InvoiceLine>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn.prepare(
        "SELECT sil.id, sil.invoice_id, sil.product_id, p.name_ar, sil.cartons, sil.cups_per_carton, sil.qty_cups, sil.unit_price_milli, COALESCE(sil.customs_price_milli, 0), sil.line_net_milli, sil.vat_pct, sil.vat_milli FROM sales_invoice_lines sil LEFT JOIN products p ON sil.product_id=p.id WHERE sil.invoice_id=?"
    )?;
    
    let rows = stmt.query_map([invoice_id], |row| {
        Ok(InvoiceLine {
            id: row.get(0)?, invoice_id: row.get(1)?, product_id: row.get(2)?,
            product_name: row.get(3)?, cartons: row.get(4)?, cups_per_carton: row.get(5)?,
            qty_cups: row.get(6)?, unit_price_milli: row.get(7)?,
            customs_price_milli: row.get(8)?,
            line_net_milli: row.get(9)?, vat_pct: row.get(10)?, vat_milli: row.get(11)?,
        })
    })?;
    
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[tauri::command]
pub fn create_invoice(state: State<'_, DbState>, input: CreateInvoiceInput) -> Result<i64, AppError> {
    let mut conn = state.0.lock()?;
    let tx = conn.transaction()?;
    let now = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let year = chrono::Utc::now().format("%Y").to_string();
    
    let seq: i64 = tx.query_row(
        "SELECT COALESCE(last_number,0)+1 FROM doc_sequences WHERE doc_type='INV' AND year=?",
        [&year],
        |r| r.get(0),
    ).unwrap_or(1);
    tx.execute(
        "INSERT INTO doc_sequences(doc_type, year, last_number) VALUES('INV',?,?) ON CONFLICT(doc_type, year) DO UPDATE SET last_number=excluded.last_number",
        rusqlite::params![year, seq],
    )
    .map_err(|e| format!("Failed to increment invoice sequence: {}", e))?;
    let inv_no = format!("INV-{}-{:04}", year, seq);
    
    let mut net: i64 = 0;
    let mut vat: i64 = 0;
    let mut product_infos: Vec<ProductInfo> = Vec::new();

    for line in &input.lines {
        let info: ProductInfo = tx.query_row(
            "SELECT cups_per_carton, vat_pct FROM products WHERE id=?",
            [line.product_id],
            |row| Ok(ProductInfo {
                cups_per_carton: row.get(0)?,
                vat_pct: row.get(1)?,
            }),
        ).map_err(|e| format!("Product {} not found: {}", line.product_id, e))?;
        product_infos.push(info);
    }

    for (i, line) in input.lines.iter().enumerate() {
        let info = &product_infos[i];
        let line_net = (line.cartons * line.unit_price_milli as f64).round() as i64;
        let line_vat = (line_net as f64 * info.vat_pct / 100.0).round() as i64;
        net += line_net;
        vat += line_vat;
    }
    let total = net + vat;
    
    tx.execute(
        "INSERT INTO sales_invoices(inv_no, date, customer_id, payment_type, net_milli, vat_milli, total_milli, status, notes) VALUES(?,?,?,?,?,?,?,'Draft',?)",
        rusqlite::params![inv_no, now, input.customer_id, input.payment_type.unwrap_or_else(|| "credit".into()), net, vat, total, input.notes],
    )?;
    let inv_id = tx.last_insert_rowid();
    
    for (i, line) in input.lines.iter().enumerate() {
        let info = &product_infos[i];
        let qty_cups = line.cartons * info.cups_per_carton as f64;
        let line_net = (line.cartons * line.unit_price_milli as f64).round() as i64;
        let line_vat = (line_net as f64 * info.vat_pct / 100.0).round() as i64;
        let customs = line.customs_price_milli.unwrap_or(line.unit_price_milli);
        tx.execute(
            "INSERT INTO sales_invoice_lines(invoice_id, product_id, cartons, cups_per_carton, qty_cups, unit_price_milli, customs_price_milli, line_net_milli, vat_pct, vat_milli) VALUES(?,?,?,?,?,?,?,?,?,?)",
            rusqlite::params![inv_id, line.product_id, line.cartons, info.cups_per_carton, qty_cups, line.unit_price_milli, customs, line_net, info.vat_pct, line_vat],
        )?;
    }

    let _ = rbac::log_audit(&tx, None, None, "create_invoice", "sales_invoices", Some(inv_id), None, Some(&inv_no), None);

    tx.commit()?;
    
    Ok(inv_id)
}

#[tauri::command]
pub fn post_invoice(state: State<'_, DbState>, id: i64) -> Result<String, AppError> {
    let mut conn = state.0.lock()?;
    let tx = conn.transaction()?;

    let current_status: String = tx
        .query_row("SELECT status FROM sales_invoices WHERE id=?", [id], |r| r.get(0))
        .map_err(|_| AppError::not_found("الفاتورة غير موجودة"))?;
    if current_status != "Draft" {
        return Err(AppError::validation("يمكن ترحيل الفواتير المسودة فقط"));
    }

    let lines: Vec<(i64, f64, i64)> = {
        let mut stmt = tx
            .prepare("SELECT product_id, cartons, unit_price_milli FROM sales_invoice_lines WHERE invoice_id=?")
            ?;
        let rows = stmt
            .query_map([id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            ?;
        let mut v = Vec::new();
        for r in rows {
            v.push(r?);
        }
        v
    };

    let mut total_cogs: i64 = 0;
    for (product_id, cartons, _unit_price) in &lines {
        let inv_items: Vec<(i64, f64, f64)> = {
            let mut stmt = tx
                .prepare("SELECT id, qty_on_hand, avg_cost_milli FROM inventory_items WHERE product_id=? AND active=1 ORDER BY kind DESC LIMIT 1")
                ?;
            let rows = stmt
                .query_map([product_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
                ?;
            let mut v = Vec::new();
            for r in rows {
                v.push(r?);
            }
            v
        };
        for (inv_id, _qty_on_hand, avg_cost) in &inv_items {
            let qty_to_deduct = *cartons;
            let cost_milli = (qty_to_deduct * avg_cost * 1000.0).round() as i64;
            total_cogs += cost_milli;

            tx.execute(
                "UPDATE inventory_items SET qty_on_hand = qty_on_hand - ?1 WHERE id = ?2",
                rusqlite::params![qty_to_deduct, inv_id],
            )
            ?;

            tx.execute(
                "INSERT INTO inventory_movements(ts, item_id, mtype, qty_in, qty_out, unit_cost_milli, ref_type, ref_id, notes)
                 VALUES(datetime('now'), ?1, 'sale', 0, ?2, ?3, 'invoice', ?4, 'فاتورة مبيعات')",
                rusqlite::params![inv_id, qty_to_deduct, (avg_cost * 1000.0).round() as i64, id],
            )
            ?;
        }
    }

    tx.execute(
        "UPDATE sales_invoices SET status='Posted', cogs_milli=?1 WHERE id=?2",
        rusqlite::params![total_cogs, id],
    )
    ?;

    let _ = rbac::log_audit(&tx, None, None, "post_invoice", "sales_invoices", Some(id), None, Some(&format!("COGS: {} mil, status: Posted", total_cogs)), None);

    tx.commit()?;
    Ok("تم ترحيل الفاتورة بنجاح".to_string())
}

#[tauri::command]
pub fn void_invoice(state: State<'_, DbState>, id: i64, reason: Option<String>) -> Result<String, AppError> {
    let conn = state.0.lock()?;

    let status: String = conn.query_row(
        "SELECT status FROM sales_invoices WHERE id=?",
        [id],
        |r| r.get(0),
    ).map_err(|e| format!("Invoice not found: {}", e))?;

    if status != "Draft" && status != "Posted" {
        return Err(AppError::validation("يمكن إلغاء الفواتير المسودة أو المرحلة فقط"));
    }

    let notes_addon = reason.unwrap_or_default();
    if notes_addon.is_empty() {
        conn.execute("UPDATE sales_invoices SET status='Void' WHERE id=?", [id])?;
    } else {
        conn.execute(
            "UPDATE sales_invoices SET status='Void', notes=COALESCE(notes,'') || '\n[إلغاء] ' || ? WHERE id=?",
            rusqlite::params![notes_addon, id],
        )?;
    }

    let _ = rbac::log_audit(&conn, None, None, "void_invoice", "sales_invoices", Some(id), None, Some("Void"), Some(&notes_addon));
    Ok("تم إلغاء الفاتورة".to_string())
}

#[tauri::command]
pub fn duplicate_invoice(state: State<'_, DbState>, id: i64) -> Result<i64, AppError> {
    let conn = state.0.lock()?;
    
    let inv: SalesInvoice = conn.query_row(
        "SELECT si.id, si.inv_no, si.date, si.customer_id, c.name, si.payment_type, si.vat_enabled, si.net_milli, si.vat_milli, si.discount_milli, si.total_milli, si.discount_reason, si.cogs_milli, si.paid_milli, si.status, si.notes, si.created_by, si.created_at FROM sales_invoices si LEFT JOIN customers c ON si.customer_id=c.id WHERE si.id=?",
        [id],
        |row| {
            Ok(SalesInvoice {
                id: row.get(0)?, inv_no: row.get(1)?, date: row.get(2)?, customer_id: row.get(3)?,
                customer_name: row.get(4)?, payment_type: row.get(5)?, vat_enabled: row.get(6)?,
                net_milli: row.get(7)?, vat_milli: row.get(8)?, discount_milli: row.get(9)?,
                total_milli: row.get(10)?, discount_reason: row.get(11)?, cogs_milli: row.get(12)?,
                paid_milli: row.get(13)?, status: row.get(14)?, notes: row.get(15)?,
                created_by: row.get(16)?, created_at: row.get(17)?,
            })
        },
    ).map_err(|e| format!("Invoice not found: {}", e))?;
    
    let source_lines: Vec<(i64, f64, i64)> = {
        let mut stmt = conn.prepare(
            "SELECT product_id, cartons, unit_price_milli FROM sales_invoice_lines WHERE invoice_id=?"
        )?;
        let rows = stmt.query_map([id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?;
        rows.collect::<Result<Vec<_>, _>>()?
    };
    
    let now = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let year = chrono::Utc::now().format("%Y").to_string();
    
    let seq: i64 = conn.query_row(
        "SELECT COALESCE(last_number,0)+1 FROM doc_sequences WHERE doc_type='INV' AND year=?",
        [&year],
        |r| r.get(0),
    ).unwrap_or(1);
    conn.execute(
        "INSERT INTO doc_sequences(doc_type, year, last_number) VALUES('INV',?,?) ON CONFLICT(doc_type, year) DO UPDATE SET last_number=excluded.last_number",
        rusqlite::params![year, seq],
    ).map_err(|e| format!("Failed to increment invoice sequence: {}", e))?;
    let inv_no = format!("INV-{}-{:04}", year, seq);
    
    let mut net: i64 = 0;
    let mut vat: i64 = 0;
    let mut line_data: Vec<(i64, f64, i64, ProductInfo)> = Vec::new();

    for (product_id, cartons, unit_price) in &source_lines {
        let info: ProductInfo = conn.query_row(
            "SELECT cups_per_carton, vat_pct FROM products WHERE id=?",
            [*product_id],
            |row| Ok(ProductInfo {
                cups_per_carton: row.get(0)?,
                vat_pct: row.get(1)?,
            }),
        ).map_err(|e| format!("Product {} not found: {}", product_id, e))?;
        let line_net = (*cartons * *unit_price as f64).round() as i64;
        let line_vat = (line_net as f64 * info.vat_pct / 100.0).round() as i64;
        net += line_net;
        vat += line_vat;
        line_data.push((*product_id, *cartons, *unit_price, info));
    }
    let total = net + vat;
    
    let note = format!("نسخة من {}", inv.inv_no.unwrap_or_default());
    
    conn.execute(
        "INSERT INTO sales_invoices(inv_no, date, customer_id, payment_type, net_milli, vat_milli, total_milli, status, notes) VALUES(?,?,?,?,?,?,?,'Draft',?)",
        rusqlite::params![inv_no, now, inv.customer_id, inv.payment_type.unwrap_or_else(|| "credit".into()), net, vat, total, note],
    )?;
    let new_id = conn.last_insert_rowid();
    
    for (product_id, cartons, unit_price, info) in &line_data {
        let qty_cups = *cartons * info.cups_per_carton as f64;
        let line_net = (*cartons * *unit_price as f64).round() as i64;
        let line_vat = (line_net as f64 * info.vat_pct / 100.0).round() as i64;
        conn.execute(
            "INSERT INTO sales_invoice_lines(invoice_id, product_id, cartons, cups_per_carton, qty_cups, unit_price_milli, line_net_milli, vat_pct, vat_milli) VALUES(?,?,?,?,?,?,?,?,?)",
            rusqlite::params![new_id, product_id, cartons, info.cups_per_carton, qty_cups, unit_price, line_net, info.vat_pct, line_vat],
        )?;
    }
    
    Ok(new_id)
}

#[tauri::command]
pub fn update_invoice(state: State<'_, DbState>, id: i64, notes: Option<String>) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    if let Some(n) = notes {
        conn.execute("UPDATE sales_invoices SET notes=? WHERE id=?", rusqlite::params![n, id])?;
    }
    Ok("تم التحديث".to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompanyPrintInfo {
    pub name: Option<String>,
    pub factory_name: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub vat_number: Option<String>,
    pub logo_path: Option<String>,
    pub stamp_path: Option<String>,
    pub signature_path: Option<String>,
    pub footer_notes: Option<String>,
    pub bank_details: Option<String>,
    pub default_vat_pct: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerPrintInfo {
    pub id: i64,
    pub name: String,
    pub address: Option<String>,
    pub vat_number: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoicePrintData {
    pub invoice: SalesInvoice,
    pub customer: CustomerPrintInfo,
    pub lines: Vec<InvoiceLine>,
    pub company: CompanyPrintInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentPrintInfo {
    pub id: i64,
    pub receipt_no: Option<String>,
    pub date: String,
    pub amount_milli: i64,
    pub method: Option<String>,
    pub reference: Option<String>,
    pub status: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReceiptPrintData {
    pub payment: PaymentPrintInfo,
    pub customer: CustomerPrintInfo,
    pub company: CompanyPrintInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeliveryNoteData {
    pub invoice: SalesInvoice,
    pub customer: CustomerPrintInfo,
    pub lines: Vec<InvoiceLine>,
    pub company: CompanyPrintInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreditNoteInfo {
    pub id: i64,
    pub cn_no: Option<String>,
    pub date: String,
    pub invoice_no: Option<String>,
    pub reason: Option<String>,
    pub net_milli: i64,
    pub vat_milli: i64,
    pub total_milli: i64,
    pub status: String,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreditNoteLineInfo {
    pub id: i64,
    pub product_name: Option<String>,
    pub cartons: f64,
    pub qty_cups: f64,
    pub unit_price_milli: i64,
    pub line_net_milli: i64,
    pub vat_pct: f64,
    pub vat_milli: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreditNotePrintData {
    pub credit_note: CreditNoteInfo,
    pub customer: CustomerPrintInfo,
    pub lines: Vec<CreditNoteLineInfo>,
    pub company: CompanyPrintInfo,
}

fn get_company_info(conn: &rusqlite::Connection) -> Result<CompanyPrintInfo, AppError> {
    conn.query_row(
        "SELECT name, factory_name, address, phone, email, vat_number, logo_path, stamp_path, signature_path, footer_notes, bank_details, default_vat_pct FROM company_settings LIMIT 1",
        [],
        |row| Ok(CompanyPrintInfo {
            name: row.get(0)?,
            factory_name: row.get(1)?,
            address: row.get(2)?,
            phone: row.get(3)?,
            email: row.get(4)?,
            vat_number: row.get(5)?,
            logo_path: row.get(6)?,
            stamp_path: row.get(7)?,
            signature_path: row.get(8)?,
            footer_notes: row.get(9)?,
            bank_details: row.get(10)?,
            default_vat_pct: row.get(11)?,
        }),
    ).map_err(|e| AppError::business(format!("Company settings not found: {}", e)))
}

fn get_customer_info(conn: &rusqlite::Connection, customer_id: i64) -> Result<CustomerPrintInfo, AppError> {
    conn.query_row(
        "SELECT id, name, address, vat_number, phone FROM customers WHERE id=?",
        [customer_id],
        |row| Ok(CustomerPrintInfo {
            id: row.get(0)?,
            name: row.get(1)?,
            address: row.get(2)?,
            vat_number: row.get(3)?,
            phone: row.get(4)?,
        }),
    ).map_err(|e| AppError::business(format!("Customer not found: {}", e)))
}

#[tauri::command]
pub fn get_invoice_for_print(state: State<'_, DbState>, invoice_id: i64) -> Result<InvoicePrintData, AppError> {
    let conn = state.0.lock()?;
    let invoice = get_invoice_by_conn(&conn, invoice_id)?;
    let customer = get_customer_info(&conn, invoice.customer_id)?;
    let lines = get_invoice_lines_internal(&conn, invoice_id)?;
    let company = get_company_info(&conn)?;
    Ok(InvoicePrintData { invoice, customer, lines, company })
}

#[tauri::command]
pub fn get_invoice_for_print_customs(state: State<'_, DbState>, invoice_id: i64) -> Result<InvoicePrintData, AppError> {
    let conn = state.0.lock()?;
    let invoice = get_invoice_by_conn(&conn, invoice_id)?;
    let customer = get_customer_info(&conn, invoice.customer_id)?;
    let mut lines = get_invoice_lines_internal(&conn, invoice_id)?;
    let company = get_company_info(&conn)?;

    // Swap real prices with customs prices for customs clearance printing
    let mut new_net: i64 = 0;
    let mut new_vat: i64 = 0;
    for line in &mut lines {
        let customs_price = if line.customs_price_milli > 0 {
            line.customs_price_milli
        } else {
            line.unit_price_milli
        };
        let _qty_cups = line.cartons * line.cups_per_carton as f64;
        let line_net = (line.cartons * customs_price as f64).round() as i64;
        let line_vat = (line_net as f64 * line.vat_pct / 100.0).round() as i64;
        line.unit_price_milli = customs_price;
        line.line_net_milli = line_net;
        line.vat_milli = line_vat;
        new_net += line_net;
        new_vat += line_vat;
    }

    // Return modified print data with customs pricing
    let mut inv = invoice;
    inv.net_milli = new_net;
    inv.vat_milli = new_vat;
    inv.total_milli = new_net + new_vat;

    Ok(InvoicePrintData { invoice: inv, customer, lines, company })
}

#[tauri::command]
pub fn get_receipt_for_print(state: State<'_, DbState>, payment_id: i64) -> Result<ReceiptPrintData, AppError> {
    let conn = state.0.lock()?;
    let payment = conn.query_row(
        "SELECT cp.id, cp.rec_no, cp.date, cp.amount_milli, cp.method, cp.reference, cp.notes FROM customer_payments cp WHERE cp.id=?",
        [payment_id],
        |row| Ok(PaymentPrintInfo {
            id: row.get(0)?,
            receipt_no: row.get(1)?,
            date: row.get(2)?,
            amount_milli: row.get(3)?,
            method: row.get(4)?,
            reference: row.get(5)?,
            status: None,
            notes: row.get(6)?,
        }),
    ).map_err(|e| format!("Payment not found: {}", e))?;
    let customer_id: i64 = conn.query_row(
        "SELECT cp.customer_id FROM customer_payments cp WHERE cp.id=?",
        [payment_id],
        |r| r.get(0),
    ).unwrap_or(0);
    let customer = get_customer_info(&conn, customer_id)?;
    let company = get_company_info(&conn)?;
    Ok(ReceiptPrintData { payment, customer, company })
}

#[tauri::command]
pub fn get_delivery_note_for_print(state: State<'_, DbState>, invoice_id: i64) -> Result<DeliveryNoteData, AppError> {
    let conn = state.0.lock()?;
    let invoice = get_invoice_by_conn(&conn, invoice_id)?;
    let customer = get_customer_info(&conn, invoice.customer_id)?;
    let lines = get_invoice_lines_internal(&conn, invoice_id)?;
    let company = get_company_info(&conn)?;
    Ok(DeliveryNoteData { invoice, customer, lines, company })
}

#[tauri::command]
pub fn get_credit_note_for_print(state: State<'_, DbState>, credit_note_id: i64) -> Result<CreditNotePrintData, AppError> {
    let conn = state.0.lock()?;
    let cn = conn.query_row(
        "SELECT cn.id, cn.cn_no, cn.date, si.inv_no, cn.reason, cn.net_milli, cn.vat_milli, cn.total_milli, cn.status, cn.notes FROM credit_notes cn LEFT JOIN sales_invoices si ON cn.invoice_id=si.id WHERE cn.id=?",
        [credit_note_id],
        |row| Ok(CreditNoteInfo {
            id: row.get(0)?,
            cn_no: row.get(1)?,
            date: row.get(2)?,
            invoice_no: row.get(3)?,
            reason: row.get(4)?,
            net_milli: row.get(5)?,
            vat_milli: row.get(6)?,
            total_milli: row.get(7)?,
            status: row.get(8)?,
            notes: row.get(9)?,
        }),
    ).map_err(|e| format!("Credit note not found: {}", e))?;
    let customer_id: i64 = conn.query_row(
        "SELECT customer_id FROM credit_notes WHERE id=?",
        [credit_note_id],
        |r| r.get(0),
    ).unwrap_or(0);
    let customer = get_customer_info(&conn, customer_id)?;
    let company = get_company_info(&conn)?;
    let mut stmt = conn.prepare(
        "SELECT cnl.id, p.name_ar, cnl.cartons, cnl.qty_cups, cnl.unit_price_milli, cnl.line_net_milli, cnl.vat_pct, cnl.vat_milli FROM credit_note_lines cnl LEFT JOIN products p ON cnl.product_id=p.id WHERE cnl.cn_id=?"
    )?;
    let lines = stmt.query_map([credit_note_id], |row| {
        Ok(CreditNoteLineInfo {
            id: row.get(0)?,
            product_name: row.get(1)?,
            cartons: row.get(2)?,
            qty_cups: row.get(3)?,
            unit_price_milli: row.get(4)?,
            line_net_milli: row.get(5)?,
            vat_pct: row.get(6)?,
            vat_milli: row.get(7)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;
    Ok(CreditNotePrintData { credit_note: cn, customer, lines, company })
}

// Internal helpers that work with an existing connection (not State)
fn get_invoice_by_conn(conn: &rusqlite::Connection, id: i64) -> Result<SalesInvoice, AppError> {
    conn.query_row(
        "SELECT si.id, si.inv_no, si.date, si.customer_id, c.name, si.payment_type, si.vat_enabled, si.net_milli, si.vat_milli, si.discount_milli, si.total_milli, si.discount_reason, si.cogs_milli, si.paid_milli, si.status, si.notes, si.created_by, si.created_at FROM sales_invoices si LEFT JOIN customers c ON si.customer_id=c.id WHERE si.id=?",
        [id],
        |row| {
            Ok(SalesInvoice {
                id: row.get(0)?, inv_no: row.get(1)?, date: row.get(2)?, customer_id: row.get(3)?,
                customer_name: row.get(4)?, payment_type: row.get(5)?, vat_enabled: row.get(6)?,
                net_milli: row.get(7)?, vat_milli: row.get(8)?, discount_milli: row.get(9)?,
                total_milli: row.get(10)?, discount_reason: row.get(11)?, cogs_milli: row.get(12)?,
                paid_milli: row.get(13)?, status: row.get(14)?, notes: row.get(15)?,
                created_by: row.get(16)?, created_at: row.get(17)?,
            })
        },
    ).map_err(|e| AppError::not_found(format!("Invoice not found: {}", e)))
}

fn get_invoice_lines_internal(conn: &rusqlite::Connection, invoice_id: i64) -> Result<Vec<InvoiceLine>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT sil.id, sil.invoice_id, sil.product_id, p.name_ar, sil.cartons, sil.cups_per_carton, sil.qty_cups, sil.unit_price_milli, COALESCE(sil.customs_price_milli, 0), sil.line_net_milli, sil.vat_pct, sil.vat_milli FROM sales_invoice_lines sil LEFT JOIN products p ON sil.product_id=p.id WHERE sil.invoice_id=?"
    )?;
    let rows = stmt.query_map([invoice_id], |row| {
        Ok(InvoiceLine {
            id: row.get(0)?, invoice_id: row.get(1)?, product_id: row.get(2)?,
            product_name: row.get(3)?, cartons: row.get(4)?, cups_per_carton: row.get(5)?,
            qty_cups: row.get(6)?, unit_price_milli: row.get(7)?,
            customs_price_milli: row.get(8)?,
            line_net_milli: row.get(9)?, vat_pct: row.get(10)?, vat_milli: row.get(11)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}
