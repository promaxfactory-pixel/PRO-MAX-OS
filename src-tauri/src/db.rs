use rusqlite::{Connection, Result};
use std::path::Path;
use std::sync::Mutex;

pub struct DbState(pub Mutex<Connection>);

pub fn init_database(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA busy_timeout = 5000;")?;
    
    // Core schema
    conn.execute_batch(include_str!("schema.sql"))?;
    
    // Run migrations
    migrations::run(&conn)?;
    
    Ok(conn)
}

mod migrations {
    use rusqlite::{Connection, Result};
    
    const SCHEMA_VERSION: i32 = 19;
    
    pub fn run(conn: &Connection) -> Result<()> {
        let current: i32 = conn
            .query_row(
                "SELECT COALESCE(CAST(value AS INTEGER), 0) FROM app_settings WHERE key='schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        
        if current < SCHEMA_VERSION {
            for v in (current + 1)..=SCHEMA_VERSION {
                apply_migration(conn, v)?;
            }
            conn.execute(
                "INSERT INTO app_settings(key, value) VALUES('schema_version', ?) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
                [SCHEMA_VERSION.to_string()],
            )?;
        }
        Ok(())
    }
    
    fn apply_migration(conn: &Connection, version: i32) -> Result<()> {
        match version {
            1 => { /* Base schema already in schema.sql */ }
            2 => { /* Phase 2 tables */ }
            3 => { /* Pricing */ }
            4 => { /* Operations */ }
            5 => { /* Custody */ }
            6 => { /* Maintenance */ }
            7 => { /* Invoicing */ }
            8 => { /* Accounting */ }
            9 => { /* Production */ }
            10 => { /* Payroll */ }
            11 => { /* Payroll payments */ }
            12 => { /* Payroll payment workflow */ }
            13 => {
                // Overtime records table
                conn.execute_batch(
                    "CREATE TABLE IF NOT EXISTS overtime_records (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        employee_id INTEGER NOT NULL,
                        date TEXT NOT NULL,
                        hours REAL NOT NULL DEFAULT 0,
                        rate_multiplier REAL NOT NULL DEFAULT 1.5,
                        reason TEXT,
                        approved INTEGER NOT NULL DEFAULT 0,
                        approved_by TEXT,
                        approved_at TEXT,
                        status TEXT NOT NULL DEFAULT 'Pending',
                        notes TEXT,
                        created_by TEXT,
                        created_at TEXT
                    );
                    CREATE INDEX IF NOT EXISTS idx_ot_employee ON overtime_records(employee_id);
                    CREATE INDEX IF NOT EXISTS idx_ot_date ON overtime_records(date);
                    CREATE INDEX IF NOT EXISTS idx_ot_status ON overtime_records(status);"
                ).ok();
            }
            14 => {
                conn.execute_batch(
                    "CREATE TABLE IF NOT EXISTS production_shift_lines (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        sheet_id INTEGER NOT NULL REFERENCES operations_daily_sheets(id),
                        product_id INTEGER NOT NULL REFERENCES products(id),
                        customer_brand TEXT,
                        cartons_produced REAL NOT NULL DEFAULT 0,
                        cups_per_carton INTEGER NOT NULL DEFAULT 1000,
                        waste_cartons REAL NOT NULL DEFAULT 0,
                        ts TEXT NOT NULL DEFAULT (datetime('now')),
                        recorded_by TEXT
                    );
                    CREATE INDEX IF NOT EXISTS idx_psl_sheet ON production_shift_lines(sheet_id);
                    CREATE INDEX IF NOT EXISTS idx_psl_product ON production_shift_lines(product_id);
                    CREATE INDEX IF NOT EXISTS idx_psl_date ON production_shift_lines(ts);"
                ).ok();
            }
            15 => {
                conn.execute_batch(
                    "-- Add custom/duty price column for dual pricing (customs vs real)
                    ALTER TABLE sales_invoice_lines ADD COLUMN customs_price_milli INTEGER NOT NULL DEFAULT 0;

                    -- Add missing FK indexes for performance
                    CREATE INDEX IF NOT EXISTS idx_inv_items_supplier ON inventory_items(supplier_id);
                    CREATE INDEX IF NOT EXISTS idx_ops_product ON operations_daily_sheets(product_id);
                    CREATE INDEX IF NOT EXISTS idx_mnt_machine_id ON maintenance_daily_sheets(machine_id);
                    CREATE INDEX IF NOT EXISTS idx_cn_customer_id ON credit_notes(customer_id);
                    CREATE INDEX IF NOT EXISTS idx_cnlines_cn ON credit_note_lines(cn_id);
                    CREATE INDEX IF NOT EXISTS idx_cnlines_product ON credit_note_lines(product_id);
                    CREATE INDEX IF NOT EXISTS idx_imp_supplier ON import_shipments(supplier_id);
                    CREATE INDEX IF NOT EXISTS idx_impcost_shipment ON import_shipment_costs(shipment_id);
                    CREATE INDEX IF NOT EXISTS idx_impalloc_shipment ON import_shipment_allocations(shipment_id);
                    CREATE INDEX IF NOT EXISTS idx_impalloc_item ON import_shipment_allocations(item_id);
                    CREATE INDEX IF NOT EXISTS idx_st_transfer_from ON stock_transfers(from_warehouse_id);
                    CREATE INDEX IF NOT EXISTS idx_st_transfer_to ON stock_transfers(to_warehouse_id);
                    CREATE INDEX IF NOT EXISTS idx_quality_pline ON quality_inspections(production_line_id);
                    CREATE INDEX IF NOT EXISTS idx_docflow_entity ON docflow_documents(entity_type, entity_id);
                    
                    -- Daily closings performance
                    CREATE INDEX IF NOT EXISTS idx_dc_date ON daily_closings(date);

                    -- Login attempts table fix (add id PK for new databases; for existing, ensure index)
                    CREATE INDEX IF NOT EXISTS idx_login_attempts_user ON login_attempts(username);"
                ).ok();
            }
            16 => {
                conn.execute_batch(
                    "CREATE TABLE IF NOT EXISTS machine_temp_logs (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        machine_id INTEGER NOT NULL REFERENCES machines(id),
                        temperature REAL NOT NULL,
                        ts TEXT NOT NULL DEFAULT (datetime('now')),
                        recorded_by TEXT
                    );
                    CREATE INDEX IF NOT EXISTS idx_mtl_machine ON machine_temp_logs(machine_id);
                    CREATE INDEX IF NOT EXISTS idx_mtl_ts ON machine_temp_logs(ts);"
                ).ok();
            }
            17 => {
                conn.execute_batch(
                    "-- Government entities registry
                    CREATE TABLE IF NOT EXISTS gov_entities (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        code TEXT NOT NULL UNIQUE,
                        name_ar TEXT NOT NULL,
                        name_en TEXT,
                        category TEXT NOT NULL DEFAULT 'ministry',
                        website TEXT,
                        api_endpoint TEXT,
                        api_key_required INTEGER NOT NULL DEFAULT 0,
                        active INTEGER NOT NULL DEFAULT 1,
                        notes TEXT
                    );
                    -- Government integration configurations
                    CREATE TABLE IF NOT EXISTS gov_integrations (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        entity_id INTEGER NOT NULL REFERENCES gov_entities(id),
                        config_key TEXT NOT NULL,
                        config_value TEXT,
                        encrypted INTEGER NOT NULL DEFAULT 0,
                        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
                    );
                    -- Government report templates
                    CREATE TABLE IF NOT EXISTS gov_report_templates (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        entity_id INTEGER NOT NULL REFERENCES gov_entities(id),
                        code TEXT NOT NULL UNIQUE,
                        name_ar TEXT NOT NULL,
                        name_en TEXT,
                        report_type TEXT NOT NULL DEFAULT 'periodic',
                        period TEXT NOT NULL DEFAULT 'monthly',
                        format TEXT NOT NULL DEFAULT 'json',
                        active INTEGER NOT NULL DEFAULT 1
                    );
                    -- Government submission logs
                    CREATE TABLE IF NOT EXISTS gov_submissions (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        entity_id INTEGER NOT NULL REFERENCES gov_entities(id),
                        report_template_id INTEGER REFERENCES gov_report_templates(id),
                        status TEXT NOT NULL DEFAULT 'pending',
                        payload TEXT,
                        response TEXT,
                        reference_no TEXT,
                        submitted_at TEXT,
                        submitted_by TEXT,
                        created_at TEXT NOT NULL DEFAULT (datetime('now'))
                    );
                    -- Seed Omani government entities
                    INSERT OR IGNORE INTO gov_entities(code, name_ar, name_en, category) VALUES
                        ('mol', 'وزارة العمل', 'Ministry of Labour', 'ministry'),
                        ('moci', 'وزارة التجارة والصناعة', 'Ministry of Commerce', 'ministry'),
                        ('rop', 'الشرطة السلطانية', 'Royal Oman Police', 'security'),
                        ('ncsi', 'المركز الوطني للإحصاء', 'NCSI', 'statistics'),
                        ('pasi', 'التأمينات الاجتماعية', 'PASI', 'insurance'),
                        ('mone', 'وزارة الاقتصاد', 'Ministry of Economy', 'ministry'),
                        ('tax', 'الهيئة العامة للزكاة والضريبة', 'Tax Authority', 'tax'),
                        ('moci_cr', 'السجل التجاري', 'Commercial Registry', 'business'),
                        ('mol_wp', 'تصاريح العمل', 'Work Permits', 'labour'),
                        ('rop_res', 'الإقامة', 'Residency', 'immigration');
                    CREATE INDEX IF NOT EXISTS idx_gov_ent_cat ON gov_entities(category);
                    CREATE INDEX IF NOT EXISTS idx_gov_ent_active ON gov_entities(active);
                    CREATE INDEX IF NOT EXISTS idx_gov_sub_entity ON gov_submissions(entity_id);
                    CREATE INDEX IF NOT EXISTS idx_gov_sub_status ON gov_submissions(status);"
                ).ok();
            }
            18 => {
                conn.execute_batch(
                    "-- Professional Accounting: Multi-company support
                    CREATE TABLE IF NOT EXISTS companies (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        code TEXT NOT NULL UNIQUE,
                        name_ar TEXT NOT NULL,
                        name_en TEXT,
                        cr_number TEXT,
                        vat_number TEXT,
                        address TEXT,
                        phone TEXT,
                        email TEXT,
                        website TEXT,
                        logo_path TEXT,
                        default_currency TEXT NOT NULL DEFAULT 'OMR',
                        default_vat_pct REAL NOT NULL DEFAULT 5.0,
                        fiscal_year_start TEXT NOT NULL DEFAULT '01-01',
                        active INTEGER NOT NULL DEFAULT 1,
                        created_at TEXT NOT NULL DEFAULT (datetime('now'))
                    );
                    -- Convert old single-row company_settings into companies table
                    INSERT OR IGNORE INTO companies(code, name_ar, vat_number, address, phone, email, default_vat_pct)
                    SELECT 'MAIN', COALESCE(name, 'My Company'), vat_number, address, phone, email, default_vat_pct
                    FROM company_settings LIMIT 1;
                    -- Fiscal years
                    CREATE TABLE IF NOT EXISTS fiscal_years (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        company_id INTEGER NOT NULL REFERENCES companies(id),
                        name TEXT NOT NULL,
                        start_date TEXT NOT NULL,
                        end_date TEXT NOT NULL,
                        is_closed INTEGER NOT NULL DEFAULT 0,
                        closed_at TEXT,
                        notes TEXT
                    );
                    -- Tax rates (configurable per company)
                    CREATE TABLE IF NOT EXISTS tax_rates (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        company_id INTEGER NOT NULL REFERENCES companies(id),
                        code TEXT NOT NULL,
                        name_ar TEXT NOT NULL,
                        rate_pct REAL NOT NULL,
                        is_default INTEGER NOT NULL DEFAULT 0,
                        active INTEGER NOT NULL DEFAULT 1
                    );
                    INSERT OR IGNORE INTO tax_rates(company_id, code, name_ar, rate_pct, is_default)
                    SELECT id, 'VAT', 'ضريبة القيمة المضافة', default_vat_pct, 1 FROM companies;
                    -- Currencies
                    CREATE TABLE IF NOT EXISTS currencies (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        code TEXT NOT NULL UNIQUE,
                        name_ar TEXT NOT NULL,
                        symbol TEXT NOT NULL,
                        decimal_places INTEGER NOT NULL DEFAULT 3,
                        is_default INTEGER NOT NULL DEFAULT 0,
                        active INTEGER NOT NULL DEFAULT 1
                    );
                    INSERT OR IGNORE INTO currencies(code, name_ar, symbol, decimal_places, is_default) VALUES
                        ('OMR', 'ريال عماني', 'ر.ع.', 3, 1),
                        ('USD', 'دولار أمريكي', '$', 2, 0),
                        ('EUR', 'يورو', '€', 2, 0),
                        ('AED', 'درهم إماراتي', 'د.إ.', 2, 0),
                        ('SAR', 'ريال سعودي', 'ر.س.', 2, 0),
                        ('QAR', 'ريال قطري', 'ر.ق.', 2, 0),
                        ('INR', 'روبية هندية', '₹', 2, 0),
                        ('PKR', 'روبية باكستانية', '₨', 2, 0);
                    -- Exchange rates
                    CREATE TABLE IF NOT EXISTS exchange_rates (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        from_currency TEXT NOT NULL,
                        to_currency TEXT NOT NULL DEFAULT 'OMR',
                        rate REAL NOT NULL,
                        effective_date TEXT NOT NULL DEFAULT (date('now'))
                    );
                    INSERT OR IGNORE INTO exchange_rates(from_currency, to_currency, rate) VALUES
                        ('USD', 'OMR', 0.385), ('EUR', 'OMR', 0.420), ('AED', 'OMR', 0.105),
                        ('SAR', 'OMR', 0.103), ('QAR', 'OMR', 0.106), ('INR', 'OMR', 0.0046),
                        ('PKR', 'OMR', 0.0014);
                    -- Contact person for customer/supplier
                    ALTER TABLE customers ADD COLUMN contact_person TEXT;
                    ALTER TABLE customers ADD COLUMN company_type TEXT;
                    ALTER TABLE customers ADD COLUMN industry TEXT;
                    ALTER TABLE suppliers ADD COLUMN contact_person TEXT;
                    ALTER TABLE suppliers ADD COLUMN payment_terms_days INTEGER NOT NULL DEFAULT 30;"
                ).ok();
            }
            19 => {
                conn.execute_batch(
                    "-- E-Invoice settings per company
                    CREATE TABLE IF NOT EXISTS einvoice_settings (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        company_id INTEGER NOT NULL REFERENCES companies(id),
                        tax_authority_endpoint TEXT,
                        api_key TEXT,
                        api_secret TEXT,
                        portal_username TEXT,
                        portal_password TEXT,
                        environment TEXT NOT NULL DEFAULT 'sandbox',
                        auto_submit INTEGER NOT NULL DEFAULT 0,
                        submit_on_post INTEGER NOT NULL DEFAULT 0,
                        compliance_certificate TEXT,
                        certificate_expiry TEXT,
                        active INTEGER NOT NULL DEFAULT 1,
                        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
                    );
                    -- E-Invoice queue for pending submissions
                    CREATE TABLE IF NOT EXISTS einvoice_queue (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        invoice_id INTEGER NOT NULL REFERENCES sales_invoices(id),
                        action TEXT NOT NULL DEFAULT 'submit',
                        priority INTEGER NOT NULL DEFAULT 0,
                        retry_count INTEGER NOT NULL DEFAULT 0,
                        max_retries INTEGER NOT NULL DEFAULT 3,
                        last_error TEXT,
                        next_retry_at TEXT,
                        status TEXT NOT NULL DEFAULT 'pending',
                        created_at TEXT NOT NULL DEFAULT (datetime('now')),
                        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
                    );
                    -- Add cancel support to e_invoices
                    ALTER TABLE e_invoices ADD COLUMN cancel_reason TEXT;
                    ALTER TABLE e_invoices ADD COLUMN cancelled_at TEXT;
                    ALTER TABLE e_invoices ADD COLUMN cancelled_by TEXT;
                    -- Index for queue
                    CREATE INDEX IF NOT EXISTS idx_einv_q_status ON einvoice_queue(status);
                    CREATE INDEX IF NOT EXISTS idx_einv_q_next ON einvoice_queue(next_retry_at);
                    CREATE INDEX IF NOT EXISTS idx_einv_comp ON einvoice_settings(company_id);"
                ).ok();
            }
            _ => {}
        }
        Ok(())
    }
}
