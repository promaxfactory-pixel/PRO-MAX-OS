use rusqlite::{Connection, Result};
use std::path::Path;
use std::sync::Mutex;

pub struct DbState(pub Mutex<Connection>);

/// Atomically allocates the next per-year sequence number for a document type.
///
/// The single UPSERT both increments `last_number` and returns the new value,
/// so concurrent callers can never observe the same number twice (unlike the
/// former read-then-write pattern which could duplicate invoice/expense/... numbers).
pub fn next_sequence(conn: &Connection, doc_type: &str, year: &str) -> Result<i64> {
    conn.query_row(
        "INSERT INTO doc_sequences(doc_type, year, last_number) VALUES(?1, ?2, 1)
         ON CONFLICT(doc_type, year) DO UPDATE SET last_number = doc_sequences.last_number + 1
         RETURNING last_number",
        rusqlite::params![doc_type, year],
        |r| r.get(0),
    )
}

pub fn init_database(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA busy_timeout = 5000;")?;
    
    // Core schema
    conn.execute_batch(include_str!("schema.sql"))?;
    
    // Ensure admin user exists with auto-generated password
    ensure_admin_user(&conn)?;
    
    // Run migrations
    migrations::run(&conn)?;
    
    Ok(conn)
}

fn ensure_admin_user(conn: &Connection) -> Result<()> {
    let admin_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM users WHERE username='admin'",
            [],
            |r| r.get::<_, i64>(0),
        )
        .unwrap_or(0) > 0;

    if !admin_exists {
        // Deterministic default so freshly installed copies can be logged into
        // out of the box. The user is forced to change it on first login.
        let temp_password = "Admin@2026".to_string();
        let hash = crate::crypto::hash_password(&temp_password)
            .unwrap_or_else(|_| "argon2id$v=19$m=19456,t=2,p=1$FALLBACK".into());

        conn.execute(
            "INSERT INTO users(username, full_name, password_hash, salt, role, active, must_change_password, created_at)
             VALUES('admin', 'مدير النظام', ?, '', 'admin', 1, 1, datetime('now'))",
            [&hash],
        )?;

        eprintln!("========================================");
        eprintln!("   PRO MAX OS - FIRST TIME SETUP");
        eprintln!("========================================");
        eprintln!("  Admin username: admin");
        eprintln!("  Admin password: {}", temp_password);
        eprintln!("  ** CHANGE THIS PASSWORD ON FIRST LOGIN **");
        eprintln!("========================================");
    }

    Ok(())
}

mod migrations {
    use rusqlite::{Connection, Result};
    
    pub(crate) const SCHEMA_VERSION: i32 = 35;
    
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
            20 => {
                conn.execute_batch(
                    "CREATE TABLE IF NOT EXISTS password_change_attempts (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        user_id INTEGER NOT NULL REFERENCES users(id),
                        ts REAL NOT NULL,
                        ok INTEGER NOT NULL DEFAULT 0
                    );
                    CREATE INDEX IF NOT EXISTS idx_pca_user ON password_change_attempts(user_id);
                    CREATE INDEX IF NOT EXISTS idx_pca_ts ON password_change_attempts(ts);"
                ).ok();
            }
            21 => {
                conn.execute_batch(
                    "-- Enhanced expenses: custody/personal tracking
                    ALTER TABLE expenses ADD COLUMN paid_by_employee_id INTEGER;
                    ALTER TABLE expenses ADD COLUMN paid_from_source TEXT DEFAULT 'company';
                    ALTER TABLE expenses ADD COLUMN petty_id INTEGER;
                    ALTER TABLE expenses ADD COLUMN custody_txn_id INTEGER;
                    ALTER TABLE expenses ADD COLUMN reimbursement_status TEXT DEFAULT 'none';
                    ALTER TABLE expenses ADD COLUMN reimbursement_date TEXT;
                    ALTER TABLE expenses ADD COLUMN reimbursed_by TEXT;
                    CREATE INDEX IF NOT EXISTS idx_exp_paid_by ON expenses(paid_by_employee_id);
                    CREATE INDEX IF NOT EXISTS idx_exp_source ON expenses(paid_from_source);
                    CREATE INDEX IF NOT EXISTS idx_exp_petty ON expenses(petty_id);
                    CREATE INDEX IF NOT EXISTS idx_exp_reimburse ON expenses(reimbursement_status);"
                ).ok();
            }
            22 => {
                conn.execute_batch(
                    "-- ============================================================
                    -- MIGRATION 22: Factory ERP Complete Enhancement
                    -- ============================================================

                    -- PRODUCTS: Add factory-specific fields
                    ALTER TABLE products ADD COLUMN brand_name TEXT;
                    ALTER TABLE products ADD COLUMN cup_size_ml REAL;
                    ALTER TABLE products ADD COLUMN cup_diameter_mm REAL;
                    ALTER TABLE products ADD COLUMN paper_weight_gsm REAL;
                    ALTER TABLE products ADD COLUMN lid_type TEXT;
                    ALTER TABLE products ADD COLUMN print_colors INTEGER DEFAULT 0;
                    ALTER TABLE products ADD COLUMN carton_length_cm REAL;
                    ALTER TABLE products ADD COLUMN carton_width_cm REAL;
                    ALTER TABLE products ADD COLUMN carton_height_cm REAL;
                    ALTER TABLE products ADD COLUMN color TEXT;
                    ALTER TABLE products ADD COLUMN material_type TEXT;
                    ALTER TABLE products ADD COLUMN product_type TEXT DEFAULT 'cup';
                    ALTER TABLE products ADD COLUMN family_id INTEGER;
                    ALTER TABLE products ADD COLUMN min_stock REAL DEFAULT 0;
                    ALTER TABLE products ADD COLUMN weight_kg REAL;

                    -- EMPLOYEES: Fix broken fields + salary breakdown
                    ALTER TABLE employees ADD COLUMN id_number TEXT;
                    ALTER TABLE employees ADD COLUMN date_of_birth TEXT;
                    ALTER TABLE employees ADD COLUMN gender TEXT;
                    ALTER TABLE employees ADD COLUMN marital_status TEXT;
                    ALTER TABLE employees ADD COLUMN email TEXT;
                    ALTER TABLE employees ADD COLUMN bank_name TEXT;
                    ALTER TABLE employees ADD COLUMN bank_account_no TEXT;
                    ALTER TABLE employees ADD COLUMN basic_salary_milli INTEGER DEFAULT 0;
                    ALTER TABLE employees ADD COLUMN housing_allowance_milli INTEGER DEFAULT 0;
                    ALTER TABLE employees ADD COLUMN transport_allowance_milli INTEGER DEFAULT 0;
                    ALTER TABLE employees ADD COLUMN food_allowance_milli INTEGER DEFAULT 0;
                    ALTER TABLE employees ADD COLUMN other_allowances_milli INTEGER DEFAULT 0;
                    ALTER TABLE employees ADD COLUMN overtime_rate_milli REAL DEFAULT 0;
                    ALTER TABLE employees ADD COLUMN insurance_policy_no TEXT;
                    ALTER TABLE employees ADD COLUMN insurance_premium_milli INTEGER DEFAULT 0;
                    ALTER TABLE employees ADD COLUMN ticket_allowance_milli INTEGER DEFAULT 0;
                    ALTER TABLE employees ADD COLUMN sponsor_name TEXT;
                    ALTER TABLE employees ADD COLUMN sponsor_id TEXT;

                    -- PRODUCTION: Add worker tracking
                    ALTER TABLE production_shift_lines ADD COLUMN worker_id INTEGER;
                    ALTER TABLE operations_daily_sheets ADD COLUMN worker_id INTEGER;
                    ALTER TABLE operations_daily_sheets ADD COLUMN machine_id INTEGER;
                    ALTER TABLE operations_daily_sheets ADD COLUMN starting_qty REAL DEFAULT 0;
                    ALTER TABLE operations_daily_sheets ADD COLUMN ending_qty REAL DEFAULT 0;
                    ALTER TABLE operations_daily_sheets ADD COLUMN break_minutes REAL DEFAULT 0;

                    -- IMPORT SHIPMENTS: Enrich for Chinese imports
                    ALTER TABLE import_shipments ADD COLUMN shipping_company TEXT;
                    ALTER TABLE import_shipments ADD COLUMN container_no TEXT;
                    ALTER TABLE import_shipments ADD COLUMN bl_no TEXT;
                    ALTER TABLE import_shipments ADD COLUMN vessel_flight TEXT;
                    ALTER TABLE import_shipments ADD COLUMN port_of_loading TEXT;
                    ALTER TABLE import_shipments ADD COLUMN port_of_discharge TEXT DEFAULT 'Port Sultan Qaboos';
                    ALTER TABLE import_shipments ADD COLUMN estimated_arrival TEXT;
                    ALTER TABLE import_shipments ADD COLUMN actual_arrival TEXT;
                    ALTER TABLE import_shipments ADD COLUMN customs_declaration_no TEXT;
                    ALTER TABLE import_shipments ADD COLUMN customs_clearance_date TEXT;
                    ALTER TABLE import_shipments ADD COLUMN duty_amount_milli INTEGER DEFAULT 0;
                    ALTER TABLE import_shipments ADD COLUMN vat_on_import_milli INTEGER DEFAULT 0;
                    ALTER TABLE import_shipments ADD COLUMN freight_cost_milli INTEGER DEFAULT 0;
                    ALTER TABLE import_shipments ADD COLUMN insurance_cost_milli INTEGER DEFAULT 0;
                    ALTER TABLE import_shipments ADD COLUMN handling_cost_milli INTEGER DEFAULT 0;
                    ALTER TABLE import_shipments ADD COLUMN commercial_invoice_no TEXT;
                    ALTER TABLE import_shipments ADD COLUMN packing_list_no TEXT;
                    ALTER TABLE import_shipments ADD COLUMN origin_country TEXT DEFAULT 'China';
                    ALTER TABLE import_shipments ADD COLUMN gross_weight_kg REAL DEFAULT 0;
                    ALTER TABLE import_shipments ADD COLUMN cbm REAL DEFAULT 0;
                    ALTER TABLE import_shipments ADD COLUMN clearance_agent TEXT;
                    ALTER TABLE import_shipments ADD COLUMN total_landed_cost_milli INTEGER DEFAULT 0;

                    -- INSTALLMENTS: Enrich for factory loans
                    ALTER TABLE installments ADD COLUMN interest_pct REAL DEFAULT 0;
                    ALTER TABLE installments ADD COLUMN monthly_installment_milli INTEGER DEFAULT 0;
                    ALTER TABLE installments ADD COLUMN num_installments INTEGER DEFAULT 0;
                    ALTER TABLE installments ADD COLUMN paid_installments INTEGER DEFAULT 0;
                    ALTER TABLE installments ADD COLUMN penalty_pct REAL DEFAULT 0;
                    ALTER TABLE installments ADD COLUMN collateral TEXT;
                    ALTER TABLE installments ADD COLUMN guarantor TEXT;

                    -- SUPPLIERS: Add type and exchange support
                    ALTER TABLE suppliers ADD COLUMN supplier_type TEXT DEFAULT 'local';
                    ALTER TABLE suppliers ADD COLUMN lead_time_days INTEGER DEFAULT 0;
                    ALTER TABLE suppliers ADD COLUMN local_exchange_enabled INTEGER DEFAULT 0;

                    -- CUSTOMERS: Add credit days
                    ALTER TABLE customers ADD COLUMN credit_days INTEGER DEFAULT 0;
                    ALTER TABLE customers ADD COLUMN default_discount_pct REAL DEFAULT 0;
                    ALTER TABLE customers ADD COLUMN route TEXT;

                    -- NEW TABLE: Local Supplier Exchanges (Barter: Bags for Cartons)
                    CREATE TABLE IF NOT EXISTS local_supplier_exchanges (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        exchange_no TEXT NOT NULL,
                        date TEXT NOT NULL,
                        local_supplier_id INTEGER NOT NULL REFERENCES suppliers(id),
                        product_id INTEGER REFERENCES products(id),
                        cartons_given REAL DEFAULT 0,
                        carton_value_milli INTEGER DEFAULT 0,
                        received_item_id INTEGER REFERENCES inventory_items(id),
                        bags_received REAL DEFAULT 0,
                        bag_value_milli INTEGER DEFAULT 0,
                        net_value_milli INTEGER DEFAULT 0,
                        balance_milli INTEGER DEFAULT 0,
                        settlement_status TEXT DEFAULT 'open',
                        reference TEXT,
                        notes TEXT,
                        status TEXT DEFAULT 'Draft',
                        created_by TEXT,
                        created_at TEXT
                    );

                    -- NEW TABLE: Installment Payment Schedule
                    CREATE TABLE IF NOT EXISTS installment_payments (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        installment_id INTEGER NOT NULL REFERENCES installments(id),
                        installment_number INTEGER NOT NULL,
                        due_date TEXT NOT NULL,
                        amount_milli INTEGER NOT NULL,
                        paid_milli INTEGER DEFAULT 0,
                        paid_date TEXT,
                        penalty_milli INTEGER DEFAULT 0,
                        status TEXT DEFAULT 'pending',
                        notes TEXT,
                        journal_id INTEGER
                    );

                    -- NEW TABLE: Shift Inventory Snapshots
                    CREATE TABLE IF NOT EXISTS shift_inventory_snapshots (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        date TEXT NOT NULL,
                        shift TEXT NOT NULL,
                        item_id INTEGER NOT NULL REFERENCES inventory_items(id),
                        opening_qty REAL NOT NULL DEFAULT 0,
                        received_qty REAL NOT NULL DEFAULT 0,
                        consumed_qty REAL NOT NULL DEFAULT 0,
                        produced_qty REAL NOT NULL DEFAULT 0,
                        waste_qty REAL NOT NULL DEFAULT 0,
                        closing_qty REAL NOT NULL DEFAULT 0,
                        recorded_by TEXT,
                        created_at TEXT
                    );

                    -- NEW TABLE: Employee Leave
                    CREATE TABLE IF NOT EXISTS employee_leave_types (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        code TEXT UNIQUE NOT NULL,
                        name TEXT NOT NULL,
                        default_days_per_year INTEGER DEFAULT 0,
                        paid INTEGER DEFAULT 1,
                        active INTEGER DEFAULT 1
                    );

                    CREATE TABLE IF NOT EXISTS employee_leave_requests (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        employee_id INTEGER NOT NULL REFERENCES employees(id),
                        leave_type_id INTEGER NOT NULL REFERENCES employee_leave_types(id),
                        from_date TEXT NOT NULL,
                        to_date TEXT NOT NULL,
                        days REAL NOT NULL DEFAULT 1,
                        reason TEXT,
                        status TEXT DEFAULT 'Pending',
                        approved_by TEXT,
                        approved_at TEXT,
                        created_by TEXT,
                        created_at TEXT
                    );

                    -- NEW TABLE: Worker Production Summary (per worker per day)
                    CREATE TABLE IF NOT EXISTS worker_daily_production (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        employee_id INTEGER NOT NULL REFERENCES employees(id),
                        date TEXT NOT NULL,
                        shift TEXT NOT NULL,
                        total_cartons REAL DEFAULT 0,
                        total_cups REAL DEFAULT 0,
                        total_waste_cartons REAL DEFAULT 0,
                        products_breakdown TEXT,
                        recorded_by TEXT,
                        created_at TEXT
                    );

                    -- SEED: Default leave types
                    INSERT OR IGNORE INTO employee_leave_types(code, name, default_days_per_year, paid) VALUES
                        ('annual', 'إجازة سنوية', 30, 1),
                        ('sick', 'إجازة مرضية', 15, 1),
                        ('casual', 'إجازة طارئة', 5, 1),
                        ('hajj', 'إجازة حج', 15, 1),
                        ('maternity', 'إجازة أمومة', 60, 1),
                        ('unpaid', 'إجازة بدون راتب', 0, 0);

                    -- INDEXES for performance
                    CREATE INDEX IF NOT EXISTS idx_products_brand ON products(brand_name);
                    CREATE INDEX IF NOT EXISTS idx_products_type ON products(product_type);
                    CREATE INDEX IF NOT EXISTS idx_products_family ON products(family_id);
                    CREATE INDEX IF NOT EXISTS idx_psl_worker ON production_shift_lines(worker_id);
                    CREATE INDEX IF NOT EXISTS idx_ops_worker ON operations_daily_sheets(worker_id);
                    CREATE INDEX IF NOT EXISTS idx_ops_machine ON operations_daily_sheets(machine_id);
                    CREATE INDEX IF NOT EXISTS idx_lse_supplier ON local_supplier_exchanges(local_supplier_id);
                    CREATE INDEX IF NOT EXISTS idx_lse_date ON local_supplier_exchanges(date);
                    CREATE INDEX IF NOT EXISTS idx_inst_pay_installment ON installment_payments(installment_id);
                    CREATE INDEX IF NOT EXISTS idx_inst_pay_status ON installment_payments(status);
                    CREATE INDEX IF NOT EXISTS idx_shift_inv_date ON shift_inventory_snapshots(date, shift);
                    CREATE INDEX IF NOT EXISTS idx_emp_leave_emp ON employee_leave_requests(employee_id);
                    CREATE INDEX IF NOT EXISTS idx_emp_leave_type ON employee_leave_requests(leave_type_id);
                    CREATE INDEX IF NOT EXISTS idx_emp_leave_status ON employee_leave_requests(status);
                    CREATE INDEX IF NOT EXISTS idx_wdp_emp ON worker_daily_production(employee_id);
                    CREATE INDEX IF NOT EXISTS idx_wdp_date ON worker_daily_production(date);
                    CREATE INDEX IF NOT EXISTS idx_emp_insurance ON employees(insurance_expiry);
                    CREATE INDEX IF NOT EXISTS idx_emp_contract ON employees(contract_end);"
                ).ok();
            }
            23 => {
                conn.execute_batch(
                    "
                    -- MIGRATION 23: Missing FK indexes + composite reporting indexes

                    -- FK indexes (14 missing foreign key columns)
                    CREATE INDEX IF NOT EXISTS idx_pp_product ON product_prices(product_id);
                    CREATE INDEX IF NOT EXISTS idx_bom_product ON bom(product_id);
                    CREATE INDEX IF NOT EXISTS idx_bom_item ON bom(item_id);
                    CREATE INDEX IF NOT EXISTS idx_cba_account ON cashbank_accounts(account_code);
                    CREATE INDEX IF NOT EXISTS idx_art_txn ON advance_receipts(transaction_id);
                    CREATE INDEX IF NOT EXISTS idx_gi_entity ON gov_integrations(entity_id);
                    CREATE INDEX IF NOT EXISTS idx_grt_entity ON gov_report_templates(entity_id);
                    CREATE INDEX IF NOT EXISTS idx_gs_template ON gov_submissions(report_template_id);
                    CREATE INDEX IF NOT EXISTS idx_fy_company ON fiscal_years(company_id);
                    CREATE INDEX IF NOT EXISTS idx_tr_company ON tax_rates(company_id);
                    CREATE INDEX IF NOT EXISTS idx_einvq_invoice ON einvoice_queue(invoice_id);
                    CREATE INDEX IF NOT EXISTS idx_lse_product ON local_supplier_exchanges(product_id);
                    CREATE INDEX IF NOT EXISTS idx_lse_rcvd_item ON local_supplier_exchanges(received_item_id);
                    CREATE INDEX IF NOT EXISTS idx_sis_item ON shift_inventory_snapshots(item_id);

                    -- Composite indexes for common reporting patterns
                    CREATE INDEX IF NOT EXISTS idx_si_customer_date ON sales_invoices(customer_id, date);
                    CREATE INDEX IF NOT EXISTS idx_pur_supplier_date ON purchases(supplier_id, date);
                    CREATE INDEX IF NOT EXISTS idx_im_item_ts ON inventory_movements(item_id, ts);
                    CREATE INDEX IF NOT EXISTS idx_audit_entity_ts ON audit_logs(entity, entity_id, ts);
                    CREATE INDEX IF NOT EXISTS idx_cbt_cashbank_ts ON cashbank_transactions(cashbank_id, ts);
                    CREATE INDEX IF NOT EXISTS idx_pl_product_order ON production_lines(product_id, order_id);
                    CREATE INDEX IF NOT EXISTS idx_jel_entry_account ON journal_entry_lines(entry_id, account_code);
                    "
                ).ok();
            }
            24 => {
                conn.execute_batch(
                    "
                    -- MIGRATION 24: Fix login_attempts table + additional indexes

                    -- Fix login_attempts: add id PK (recreate table)
                    CREATE TABLE IF NOT EXISTS login_attempts_new (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        username TEXT,
                        ts REAL,
                        ok INTEGER
                    );
                    INSERT INTO login_attempts_new (username, ts, ok) SELECT username, ts, ok FROM login_attempts;
                    DROP TABLE IF EXISTS login_attempts;
                    ALTER TABLE login_attempts_new RENAME TO login_attempts;
                    CREATE INDEX IF NOT EXISTS idx_login_attempts_uname ON login_attempts(username);

                    -- FK indexes for existing columns
                    CREATE INDEX IF NOT EXISTS idx_si_created_by ON sales_invoices(created_by);
                    CREATE INDEX IF NOT EXISTS idx_pur_created_by ON purchases(created_by);
                    CREATE INDEX IF NOT EXISTS idx_po_created_by ON production_orders(created_by);
                    CREATE INDEX IF NOT EXISTS idx_je_created_by ON journal_entries(created_by);
                    "
                ).map_err(|e| {
                    eprintln!("Migration 24 failed: {}", e);
                    e
                })?;
            }
            25 => {
                conn.execute_batch(
                    "
                    -- MIGRATION 25: Missing indexes for 18 untables tables
                    CREATE INDEX IF NOT EXISTS idx_accounts_code ON accounts(code);
                    CREATE INDEX IF NOT EXISTS idx_app_settings_key ON app_settings(key);
                    CREATE INDEX IF NOT EXISTS idx_dsh_entity ON document_status_history(entity_type, entity_id);
                    CREATE INDEX IF NOT EXISTS idx_dv_entity ON document_voids(entity_type, entity_id);
                    CREATE INDEX IF NOT EXISTS idx_cnl_cn ON credit_note_lines(cn_id);
                    CREATE INDEX IF NOT EXISTS idx_ia_item ON inventory_adjustments(item_id);
                    CREATE INDEX IF NOT EXISTS idx_imp_supplier ON import_shipments(supplier_id);
                    CREATE INDEX IF NOT EXISTS idx_imp_created ON import_shipments(created_at);
                    CREATE INDEX IF NOT EXISTS idx_impcost_shipment ON import_shipment_costs(shipment_id);
                    CREATE INDEX IF NOT EXISTS idx_impalloc_shipment ON import_shipment_allocations(shipment_id);
                    CREATE INDEX IF NOT EXISTS idx_roles_code ON roles(code);
                    CREATE INDEX IF NOT EXISTS idx_user_roles_user ON user_roles(user_id);
                    CREATE INDEX IF NOT EXISTS idx_emp_adv_employee ON employee_advances(employee_id);
                    CREATE INDEX IF NOT EXISTS idx_pr_period ON payroll_runs(period_start);
                    CREATE INDEX IF NOT EXISTS idx_st_created ON stock_transfers(created_at);
                    CREATE INDEX IF NOT EXISTS idx_df_entity ON docflow_documents(entity_type, entity_id);
                    CREATE INDEX IF NOT EXISTS idx_qi_date ON quality_inspections(date);
                    CREATE INDEX IF NOT EXISTS idx_dc_date ON daily_closings(date);
                    "
                ).map_err(|e| {
                    eprintln!("Migration 25 failed: {}", e);
                    e
                })?;
            }
            26 => {
                conn.execute_batch(
                    "CREATE TABLE IF NOT EXISTS import_history (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        import_type TEXT NOT NULL,
                        file_name TEXT NOT NULL,
                        total_rows INTEGER NOT NULL DEFAULT 0,
                        imported INTEGER NOT NULL DEFAULT 0,
                        skipped INTEGER NOT NULL DEFAULT 0,
                        status TEXT NOT NULL DEFAULT 'completed',
                        created_at TEXT NOT NULL DEFAULT (datetime('now','localtime')),
                        created_by TEXT NOT NULL DEFAULT 'system'
                    );"
                ).map_err(|e| {
                    eprintln!("Migration 26 failed: {}", e);
                    e
                })?;
            }
            27 => {
                conn.execute_batch(
                    "ALTER TABLE users ADD COLUMN reset_token TEXT;
                     ALTER TABLE users ADD COLUMN reset_token_expiry TEXT;"
                ).ok();
            }
            28 => {
                conn.execute_batch(
                    "ALTER TABLE inventory_items ADD COLUMN avg_cost_milli_int INTEGER DEFAULT 0;
                     UPDATE inventory_items SET avg_cost_milli_int = CAST(avg_cost_milli AS INTEGER);
                     ALTER TABLE inventory_items DROP COLUMN avg_cost_milli;
                     ALTER TABLE inventory_items RENAME COLUMN avg_cost_milli_int TO avg_cost_milli;"
                ).map_err(|e| {
                    eprintln!("Migration 28 failed: {}", e);
                    e
                })?;
            }
            29 => {
                conn.execute_batch(
                    "-- ============================================================
                    -- MIGRATION 29: AI file extractions (any-file -> LLM -> structured)
                    -- ============================================================

                    CREATE TABLE IF NOT EXISTS ai_extractions (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        file_path TEXT NOT NULL,
                        file_name TEXT NOT NULL,
                        file_type TEXT NOT NULL DEFAULT 'unknown',
                        doc_type TEXT NOT NULL DEFAULT 'invoice',
                        provider TEXT NOT NULL DEFAULT 'unknown',
                        model TEXT NOT NULL DEFAULT '',
                        raw_text TEXT,
                        extracted_json TEXT NOT NULL,
                        fields_json TEXT,
                        confidence REAL NOT NULL DEFAULT 0,
                        status TEXT NOT NULL DEFAULT 'draft',
                        target_table TEXT,
                        target_id INTEGER,
                        created_by TEXT,
                        created_at TEXT NOT NULL DEFAULT (datetime('now')),
                        updated_at TEXT
                    );
                    CREATE INDEX IF NOT EXISTS idx_ai_ext_status ON ai_extractions(status);
                    CREATE INDEX IF NOT EXISTS idx_ai_ext_created ON ai_extractions(created_at);"
                ).map_err(|e| {
                    eprintln!("Migration 29 failed: {}", e);
                    e
                })?;
            }
            30 => {
                conn.execute_batch(
                    "-- ============================================================
                    -- MIGRATION 30: Seed chart of accounts for automatic journal posting
                    -- ============================================================
                    INSERT OR IGNORE INTO accounts(code, name_ar, type, is_system) VALUES ('1000', 'الأصول', 'asset', 1);
                    INSERT OR IGNORE INTO accounts(code, name_ar, type, parent, is_system) VALUES ('1100', 'النقدية', 'asset', '1000', 1);
                    INSERT OR IGNORE INTO accounts(code, name_ar, type, parent, is_system) VALUES ('1101', 'البنك', 'asset', '1000', 1);
                    INSERT OR IGNORE INTO accounts(code, name_ar, type, parent, is_system) VALUES ('1200', 'الذمم المدينة - العملاء', 'asset', '1000', 1);
                    INSERT OR IGNORE INTO accounts(code, name_ar, type, parent, is_system) VALUES ('1320', 'سلف الموظفين', 'asset', '1000', 1);
                    INSERT OR IGNORE INTO accounts(code, name_ar, type, parent, is_system) VALUES ('1400', 'المخزون', 'asset', '1000', 1);
                    INSERT OR IGNORE INTO accounts(code, name_ar, type, is_system) VALUES ('2000', 'الخصوم', 'liability', 1);
                    INSERT OR IGNORE INTO accounts(code, name_ar, type, parent, is_system) VALUES ('2100', 'ضريبة القيمة المضافة المستحقة', 'liability', '2000', 1);
                    INSERT OR IGNORE INTO accounts(code, name_ar, type, parent, is_system) VALUES ('2200', 'الذمم الدائنة - الموردون', 'liability', '2000', 1);
                    INSERT OR IGNORE INTO accounts(code, name_ar, type, is_system) VALUES ('3000', 'حقوق الملكية', 'equity', 1);
                    INSERT OR IGNORE INTO accounts(code, name_ar, type, parent, is_system) VALUES ('3100', 'رأس المال', 'equity', '3000', 1);
                    INSERT OR IGNORE INTO accounts(code, name_ar, type, parent, is_system) VALUES ('3200', 'الأرباح المحتجزة', 'equity', '3000', 1);
                    INSERT OR IGNORE INTO accounts(code, name_ar, type, is_system) VALUES ('4000', 'الإيرادات', 'revenue', 1);
                    INSERT OR IGNORE INTO accounts(code, name_ar, type, parent, is_system) VALUES ('4100', 'إيرادات المبيعات', 'revenue', '4000', 1);
                    INSERT OR IGNORE INTO accounts(code, name_ar, type, parent, is_system) VALUES ('4200', 'إيرادات أخرى', 'revenue', '4000', 1);
                    INSERT OR IGNORE INTO accounts(code, name_ar, type, is_system) VALUES ('5000', 'المصروفات', 'expense', 1);
                    INSERT OR IGNORE INTO accounts(code, name_ar, type, parent, is_system) VALUES ('5100', 'تكلفة البضاعة المباعة', 'expense', '5000', 1);
                    INSERT OR IGNORE INTO accounts(code, name_ar, type, parent, is_system) VALUES ('5200', 'مصروفات عمومية وإدارية', 'expense', '5000', 1);"
                ).map_err(|e| {
                    eprintln!("Migration 30 failed: {}", e);
                    e
                })?;
            }
            31 => {
                let has_col: bool = conn
                    .prepare("SELECT COUNT(*) FROM pragma_table_info('customers') WHERE name='payment_terms_days'")
                    .and_then(|mut stmt| stmt.query_row([], |r| r.get::<_, i64>(0)))
                    .map(|c| c > 0)
                    .unwrap_or(false);
                if !has_col {
                    conn.execute_batch(
                        "-- Customer credit terms: net payment days for AR aging
                        ALTER TABLE customers ADD COLUMN payment_terms_days INTEGER NOT NULL DEFAULT 30;"
                    ).map_err(|e| {
                        eprintln!("Migration 31 failed: {}", e);
                        e
                    })?;
                }
            }
            32 => {
                let has_col: bool = conn
                    .prepare("SELECT COUNT(*) FROM pragma_table_info('suppliers') WHERE name='vat_number'")
                    .and_then(|mut stmt| stmt.query_row([], |r| r.get::<_, i64>(0)))
                    .map(|c| c > 0)
                    .unwrap_or(false);
                if !has_col {
                    conn.execute_batch(
                        "-- Supplier VAT number for tax reporting (queried by suppliers module)
                        ALTER TABLE suppliers ADD COLUMN vat_number TEXT;"
                    ).map_err(|e| {
                        eprintln!("Migration 32 failed: {}", e);
                        e
                    })?;
                }
            }
            33 => {
                let has_ts: bool = conn
                    .prepare("SELECT COUNT(*) FROM pragma_table_info('shift_inventory_snapshots') WHERE name='ts'")
                    .and_then(|mut stmt| stmt.query_row([], |r| r.get::<_, i64>(0)))
                    .map(|c| c > 0)
                    .unwrap_or(false);
                if !has_ts {
                    conn.execute_batch(
                        "-- Shift inventory snapshots timestamp (queried by production_shift module)
                        ALTER TABLE shift_inventory_snapshots ADD COLUMN ts TEXT;"
                    ).map_err(|e| {
                        eprintln!("Migration 33a failed: {}", e);
                        e
                    })?;
                }
                let has_notes: bool = conn
                    .prepare("SELECT COUNT(*) FROM pragma_table_info('credit_notes') WHERE name='notes'")
                    .and_then(|mut stmt| stmt.query_row([], |r| r.get::<_, i64>(0)))
                    .map(|c| c > 0)
                    .unwrap_or(false);
                if !has_notes {
                    conn.execute_batch(
                        "-- Credit note free-text notes (queried by credit note print)
                        ALTER TABLE credit_notes ADD COLUMN notes TEXT;"
                    ).map_err(|e| {
                        eprintln!("Migration 33b failed: {}", e);
                        e
                    })?;
                }
            }
            34 => {
                // Multi-branch operation
                conn.execute_batch(
                    "-- Multi-branch operation
                    CREATE TABLE IF NOT EXISTS branches (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        name TEXT NOT NULL,
                        code TEXT,
                        address TEXT,
                        is_head_office INTEGER NOT NULL DEFAULT 0,
                        is_active INTEGER NOT NULL DEFAULT 1,
                        created_at TEXT DEFAULT (datetime('now'))
                    );"
                ).map_err(|e| {
                    eprintln!("Migration 34a failed: {}", e);
                    e
                })?;
                // Offline-first sync queue (branch disconnected mode)
                conn.execute_batch(
                    "-- Offline-first sync queue
                    CREATE TABLE IF NOT EXISTS offline_sync_queue (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        branch_id INTEGER,
                        operation TEXT NOT NULL,
                        entity TEXT NOT NULL,
                        entity_id INTEGER,
                        payload TEXT NOT NULL,
                        status TEXT NOT NULL DEFAULT 'pending',
                        created_at TEXT DEFAULT (datetime('now')),
                        synced_at TEXT
                    );"
                ).map_err(|e| {
                    eprintln!("Migration 34b failed: {}", e);
                    e
                })?;
                // ZATCA Phase 2 settings
                conn.execute_batch(
                    "-- ZATCA (Phase 2) onboarding settings
                    CREATE TABLE IF NOT EXISTS zatca_settings (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        company_id INTEGER NOT NULL DEFAULT 1,
                        environment TEXT NOT NULL DEFAULT 'sandbox',
                        vat_number TEXT,
                        organization_name TEXT,
                        csid_stage TEXT NOT NULL DEFAULT 'none',
                        certificate_der TEXT,
                        signing_key TEXT,
                        onboarding_request_id TEXT,
                        icv_counter INTEGER NOT NULL DEFAULT 0,
                        last_invoice_hash TEXT,
                        updated_at TEXT
                    );"
                ).map_err(|e| {
                    eprintln!("Migration 34c failed: {}", e);
                    e
                })?;
                // Qayd XBRL filings
                conn.execute_batch(
                    "-- Qayd XBRL annual filings
                    CREATE TABLE IF NOT EXISTS qayd_filings (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        company_id INTEGER NOT NULL DEFAULT 1,
                        fiscal_year INTEGER NOT NULL,
                        currency TEXT NOT NULL DEFAULT 'KWD',
                        cr_number TEXT,
                        status TEXT NOT NULL DEFAULT 'draft',
                        instance_xml TEXT NOT NULL,
                        validation_report TEXT,
                        submitted_at TEXT,
                        created_by INTEGER,
                        created_at TEXT DEFAULT (datetime('now'))
                    );"
                ).map_err(|e| {
                    eprintln!("Migration 34d failed: {}", e);
                    e
                })?;
                // e_invoices Phase-2 metadata columns (idempotent)
                for (col, ddl) in [
                    ("icv", "ALTER TABLE e_invoices ADD COLUMN icv INTEGER;"),
                    ("pih", "ALTER TABLE e_invoices ADD COLUMN pih TEXT;"),
                    ("invoice_hash", "ALTER TABLE e_invoices ADD COLUMN invoice_hash TEXT;"),
                    ("qr_content", "ALTER TABLE e_invoices ADD COLUMN qr_content TEXT;"),
                    ("signed_xml", "ALTER TABLE e_invoices ADD COLUMN signed_xml TEXT;"),
                    ("signature_value", "ALTER TABLE e_invoices ADD COLUMN signature_value TEXT;"),
                    ("zatca_stage", "ALTER TABLE e_invoices ADD COLUMN zatca_stage TEXT;"),
                    ("zatca_environment", "ALTER TABLE e_invoices ADD COLUMN zatca_environment TEXT;"),
                    ("zatca_submitted_at", "ALTER TABLE e_invoices ADD COLUMN zatca_submitted_at TEXT;"),
                    ("zatca_rejection_code", "ALTER TABLE e_invoices ADD COLUMN zatca_rejection_code TEXT;"),
                ] {
                    let has_col: bool = conn
                        .prepare("SELECT COUNT(*) FROM pragma_table_info('e_invoices') WHERE name=?1")
                        .and_then(|mut stmt| stmt.query_row([col], |r| r.get::<_, i64>(0)))
                        .map(|c| c > 0)
                        .unwrap_or(false);
                    if !has_col {
                        conn.execute_batch(ddl).map_err(|e| {
                            eprintln!("Migration 34e failed for {}: {}", col, e);
                            e
                        })?;
                    }
                }
                // Seed head-office branch
                conn.execute(
                    "INSERT OR IGNORE INTO branches(id, name, code, is_head_office, is_active) VALUES(1, 'الفرع الرئيسي', 'HQ', 1, 1)",
                    [],
                )                .map_err(|e| {
                    eprintln!("Migration 34f failed: {}", e);
                    e
                })?;
            }
            35 => {
                // Complete company profile: the settings UI exposes CR number,
                // currency, fiscal-year start and structured bank details, but the
                // table never had storage for them. Backfill the schema so saved
                // settings actually persist (previously serde silently dropped
                // those fields -> printed docs had no company header/VAT).
                let cols: &[(&str, &str)] = &[
                    ("cr_number", "ALTER TABLE company_settings ADD COLUMN cr_number TEXT"),
                    ("currency", "ALTER TABLE company_settings ADD COLUMN currency TEXT NOT NULL DEFAULT 'OMR'"),
                    ("fiscal_year_start", "ALTER TABLE company_settings ADD COLUMN fiscal_year_start TEXT NOT NULL DEFAULT '01-01'"),
                    ("bank_name", "ALTER TABLE company_settings ADD COLUMN bank_name TEXT"),
                    ("bank_account_no", "ALTER TABLE company_settings ADD COLUMN bank_account_no TEXT"),
                    ("bank_iban", "ALTER TABLE company_settings ADD COLUMN bank_iban TEXT"),
                    ("bank_swift", "ALTER TABLE company_settings ADD COLUMN bank_swift TEXT"),
                ];
                for (col, ddl) in cols {
                    let has_col: bool = conn
                        .prepare("SELECT COUNT(*) FROM pragma_table_info('company_settings') WHERE name=?1")
                        .and_then(|mut stmt| stmt.query_row([col], |r| r.get::<_, i64>(0)))
                        .map(|c| c > 0)
                        .unwrap_or(false);
                    if !has_col {
                        conn.execute_batch(ddl).map_err(|e| {
                            eprintln!("Migration 35 failed for {}: {}", col, e);
                            e
                        })?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_conn() -> Connection {
        // Use a temp file for WAL mode support
        let db_path = std::env::temp_dir().join(format!("promax_test_{}.db", uuid::Uuid::new_v4()));
        let conn = Connection::open(&db_path).expect("Failed to open test DB");
        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA busy_timeout = 5000;").unwrap();
        conn.execute_batch(include_str!("schema.sql")).expect("Failed to apply schema");

        // Ensure admin user exists (same as init_database)
        crate::crypto::hash_password("test").ok(); // warm up crypto
        let admin_exists: bool = conn
            .query_row("SELECT COUNT(*) FROM users WHERE username='admin'", [], |r| r.get::<_, i64>(0))
            .unwrap_or(0) > 0;
        if !admin_exists {
            let hash = crate::crypto::hash_password("temppass123").unwrap_or_else(|_| "fallback".into());
            conn.execute(
                "INSERT INTO users(username, full_name, password_hash, salt, role, active, must_change_password, created_at) VALUES('admin', 'Admin', ?, '', 'admin', 1, 1, datetime('now'))",
                [&hash],
            ).ok();
        }

        migrations::run(&conn).expect("Migrations failed");
        conn
    }

    #[allow(dead_code)]
    fn cleanup_db(path: &std::path::Path) {
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(path.with_extension("db-wal"));
        let _ = std::fs::remove_file(path.with_extension("db-shm"));
    }

    #[test]
    fn fresh_install_admin_logs_in_with_default_password() {
        let db_path = std::env::temp_dir().join(format!("promax_fresh_{}.db", uuid::Uuid::new_v4()));
        let conn = super::init_database(&db_path).expect("fresh init_database must succeed");

        let (hash, must_change): (String, i64) = conn
            .query_row(
                "SELECT password_hash, must_change_password FROM users WHERE username='admin'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();

        assert_eq!(must_change, 1, "fresh admin must be forced to change password");
        assert!(
            crate::crypto::verify_password("Admin@2026", &hash).unwrap(),
            "freshly installed admin must accept the documented default password"
        );

        cleanup_db(&db_path);
    }

    #[test]
    fn fresh_db_suppliers_module_columns_work() {
        let db_path = std::env::temp_dir().join(format!("promax_suppliers_{}.db", uuid::Uuid::new_v4()));
        let conn = super::init_database(&db_path).expect("fresh init_database must succeed");
        let sql = "SELECT id, code, name, contact, phone, email, address, vat_number, currency, payment_terms, opening_balance_milli, balance_milli, notes, active FROM suppliers WHERE active=1 ORDER BY name";
        conn.prepare(sql).expect("suppliers list query must run on a fresh DB");
        cleanup_db(&db_path);
    }

    #[test]
    fn fresh_db_all_known_schema_mismatch_queries_run() {
        let db_path = std::env::temp_dir().join(format!("promax_schema_{}.db", uuid::Uuid::new_v4()));
        let conn = super::init_database(&db_path).expect("fresh init_database must succeed");

        let queries: Vec<(&str, &str)> = vec![
            // suppliers.vat_number (migration 32)
            ("suppliers", "SELECT id, code, name, vat_number, currency FROM suppliers WHERE active=1"),
            // shift_inventory_snapshots.ts (migration 33)
            ("shift_snapshot_ts", "SELECT id, date, shift, item_id, ts FROM shift_inventory_snapshots"),
            // credit_notes.notes (migration 33)
            ("credit_note_notes", "SELECT id, cn_no, reason, notes FROM credit_notes"),
            // ocr expense insert shape (uses notes column)
            ("ocr_expense", "INSERT INTO expenses (exp_no, date, category, amount_milli, method, notes) VALUES ('EXP-2026-0001', '2026-01-01', 'general', 1000, 'OCR', 'OCR Expense')"),
        ];
        for (name, sql) in &queries {
            conn.execute_batch(sql).unwrap_or_else(|e| panic!("{name} query failed on fresh DB: {e}"));
        }
        cleanup_db(&db_path);
    }

    #[test]
    fn receipt_print_flow_works_end_to_end() {
        let db_path = std::env::temp_dir().join(format!("promax_receipt_{}.db", uuid::Uuid::new_v4()));
        let conn = super::init_database(&db_path).expect("fresh init_database must succeed");

        conn.execute(
            "INSERT INTO customers(code, name, ctype, contact, phone, email, address, vat_number, credit_limit_milli, payment_terms, payment_terms_days, notes) VALUES('C1','عميل تجربة','credit',NULL,'99001122',NULL,NULL,NULL,0,'net',30,NULL)",
            [],
        ).unwrap();
        let customer_id = conn.last_insert_rowid();

        // Same insert shape as commands::customers::create_customer_payment.
        conn.execute(
            "INSERT INTO customer_payments(rec_no, date, customer_id, amount_milli, method, cashbank_id, reference, notes, created_by, created_at) VALUES('RCP-2026-0001', date('now'), ?1, 125000, 'cash', NULL, 'REF-1', NULL, 'admin', datetime('now'))",
            [customer_id],
        ).unwrap();
        let payment_id = conn.last_insert_rowid();

        // Exact query used by commands::invoices::get_receipt_for_print.
        let (rec_no, amount, method, reference): (String, i64, String, Option<String>) = conn.query_row(
            "SELECT cp.rec_no, cp.amount_milli, cp.method, cp.reference FROM customer_payments cp WHERE cp.id=?",
            [payment_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        ).unwrap();
        assert_eq!(rec_no, "RCP-2026-0001");
        assert_eq!(amount, 125000);
        assert_eq!(method, "cash");
        assert_eq!(reference.as_deref(), Some("REF-1"));

        // Customer info used by the print payload.
        let (cname, _addr, _vat, _phone): (String, Option<String>, Option<String>, Option<String>) = conn.query_row(
            "SELECT name, address, vat_number, phone FROM customers WHERE id=?",
            [customer_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        ).unwrap();
        assert_eq!(cname, "عميل تجربة");

        cleanup_db(&db_path);
    }

    #[test]
    fn test_migration_35_adds_company_profile_columns() {
        // Simulate a v34 install: app_settings + company_settings WITHOUT the
        // profile columns, then run the migration chain and verify they appear.
        let db_path = std::env::temp_dir().join(format!("promax_m35_{}.db", uuid::Uuid::new_v4()));
        let conn = Connection::open(&db_path).expect("open");
        conn.execute_batch(
            "CREATE TABLE app_settings (key TEXT PRIMARY KEY, value TEXT);
             INSERT INTO app_settings(key, value) VALUES('schema_version', '34');
             CREATE TABLE company_settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                name TEXT, factory_name TEXT, address TEXT, phone TEXT,
                email TEXT, vat_number TEXT,
                logo_path TEXT, stamp_path TEXT, signature_path TEXT,
                footer_notes TEXT, bank_details TEXT,
                default_vat_pct REAL DEFAULT 5.0
             );
             INSERT INTO company_settings(id) VALUES(1);",
        ).unwrap();

        super::migrations::run(&conn).expect("migrations must apply");

        let cols: Vec<String> = conn
            .prepare("SELECT name FROM pragma_table_info('company_settings')")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        for required in ["cr_number", "currency", "fiscal_year_start", "bank_name", "bank_account_no", "bank_iban", "bank_swift"] {
            assert!(cols.iter().any(|c| c == required), "missing column {}", required);
        }

        // Defaults must be present on the existing row.
        let (currency, fy): (String, String) = conn
            .query_row("SELECT currency, fiscal_year_start FROM company_settings WHERE id=1", [], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap();
        assert_eq!(currency, "OMR");
        assert_eq!(fy, "01-01");

        cleanup_db(&db_path);
    }

    #[test]
    #[allow(clippy::type_complexity)]
    fn test_company_settings_round_trip_persists_profile() {
        // Mirrors update_company_settings (settings.rs) against a fresh DB and
        // verifies every field that the settings UI submits actually persists
        // (regression for the silent serde-drop bug that left printed docs blank).
        let db_path = std::env::temp_dir().join(format!("promax_set_{}.db", uuid::Uuid::new_v4()));
        let conn = super::init_database(&db_path).expect("init");

        conn.execute("INSERT OR IGNORE INTO company_settings(id) VALUES(1)", []).unwrap();

        conn.execute(
            "UPDATE company_settings SET
               name=?, factory_name=?, address=?, phone=?, email=?, vat_number=?,
               cr_number=?, default_vat_pct=?, currency=?, fiscal_year_start=?,
               bank_name=?, bank_account_no=?, bank_iban=?, bank_swift=?, bank_details=?
             WHERE id=1",
            rusqlite::params![
                "شركة الخماس", "Al Khumas Co", "مسقط، العذيبة", "24560000", "info@example.com",
                "OM1122334455", "CR-2026-12345", 7.5, "SAR", "04-01",
                "بنك مسقط", "0123456789", "OM00BBKS0000123456789", "MAQYOMRUXXX",
                "بنك مسقط | رقم الحساب: 0123456789 | IBAN: OM00BBKS0000123456789 | SWIFT: MAQYOMRUXXX",
            ],
        ).unwrap();

        // The exact SELECT used by get_company_settings.
        let (name, vat_number, cr_number, vat_pct, currency, fy, bank_name, iban, bank_details): (
            Option<String>, Option<String>, Option<String>, f64, String, String,
            Option<String>, Option<String>, Option<String>,
        ) = conn.query_row(
            "SELECT name, vat_number, cr_number, default_vat_pct, currency, fiscal_year_start, bank_name, bank_iban, bank_details FROM company_settings WHERE id=1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?, r.get(8)?)),
        ).unwrap();

        assert_eq!(name.as_deref(), Some("شركة الخماس"));
        assert_eq!(vat_number.as_deref(), Some("OM1122334455"));
        assert_eq!(cr_number.as_deref(), Some("CR-2026-12345"));
        assert_eq!(vat_pct, 7.5);
        assert_eq!(currency, "SAR");
        assert_eq!(fy, "04-01");
        assert_eq!(bank_name.as_deref(), Some("بنك مسقط"));
        assert_eq!(iban.as_deref(), Some("OM00BBKS0000123456789"));
        assert!(bank_details.as_deref().unwrap().contains("بنك مسقط"));

        cleanup_db(&db_path);
    }

    #[test]
    fn test_next_sequence_increments_per_doc_type_and_year() {
        let conn = test_conn();

        // First allocation starts at 1 and the upsert persists it.
        assert_eq!(next_sequence(&conn, "INV", "2026").unwrap(), 1);
        // Subsequent allocations for the same doc_type/year increment atomically.
        assert_eq!(next_sequence(&conn, "INV", "2026").unwrap(), 2);
        assert_eq!(next_sequence(&conn, "INV", "2026").unwrap(), 3);

        // Different doc_types and years are independent sequences.
        assert_eq!(next_sequence(&conn, "EXP", "2026").unwrap(), 1);
        assert_eq!(next_sequence(&conn, "INV", "2027").unwrap(), 1);

        // The stored value matches the last returned number.
        let stored: i64 = conn
            .query_row(
                "SELECT last_number FROM doc_sequences WHERE doc_type='INV' AND year='2026'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(stored, 3);
    }

    #[test]
    fn test_schema_creates_all_core_tables() {
        let conn = test_conn();
        let tables_empty = [
            "customers", "suppliers", "inventory_items",
            "sales_invoices", "purchases", "production_orders",
            "journal_entries", "audit_logs",
            "einvoice_settings", "cashbank_accounts",
        ];
        for table in &tables_empty {
            let count: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM {}", table), [], |r| r.get(0))
                .unwrap_or_else(|e| panic!("Table '{}' not found: {}", table, e));
            assert_eq!(count, 0, "Table '{}' should be empty after init", table);
        }
        // Chart of accounts is seeded by migration 30
        let account_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM accounts", [], |r| r.get(0))
            .unwrap();
        assert!(account_count >= 16, "accounts table should be seeded by migration 30, found {}", account_count);
        // Users table has admin user
        let user_count: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |r| r.get(0)).unwrap();
        assert!(user_count >= 1, "users table should have at least admin user");
    }

    #[test]
    fn test_schema_creates_new_tables() {
        let conn = test_conn();
        let tables = [
            "approval_requests", "budgets", "budget_lines",
            "fixed_assets", "asset_maintenance_logs", "notifications",
        ];
        for table in &tables {
            let count: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM {}", table), [], |r| r.get(0))
                .unwrap_or_else(|e| panic!("New table '{}' not found: {}", table, e));
            assert_eq!(count, 0, "Table '{}' should be empty after init", table);
        }
    }

    #[test]
    fn test_schema_version_is_current() {
        let conn = test_conn();
        let version: i32 = conn
            .query_row(
                "SELECT CAST(value AS INTEGER) FROM app_settings WHERE key='schema_version'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        assert_eq!(version, migrations::SCHEMA_VERSION, "Schema version should match SCHEMA_VERSION constant");
    }

    #[test]
    fn test_admin_user_exists() {
        let conn = test_conn();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM users WHERE username='admin'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1, "Admin user should exist after init");
    }

    #[test]
    fn test_insert_and_query_customer() {
        let conn = test_conn();
        conn.execute(
            "INSERT INTO customers (name, phone, address, credit_limit_milli, active) VALUES ('Test Corp', '99887766', '123 Main St', 500000, 1)",
            [],
        ).unwrap();
        let name: String = conn
            .query_row("SELECT name FROM customers LIMIT 1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(name, "Test Corp");
    }

    #[test]
    fn test_insert_and_query_approval_request() {
        let conn = test_conn();
        conn.execute(
            "INSERT INTO approval_requests (request_type, entity_type, entity_id, entity_number, requested_by, requested_at, amount_milli, status, priority) VALUES ('purchase', 'purchase', 1, 'PO-001', 'admin', datetime('now'), 100000, 'pending', 'high')",
            [],
        ).unwrap();
        let status: String = conn
            .query_row("SELECT status FROM approval_requests LIMIT 1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(status, "pending");
    }

    #[test]
    fn test_insert_and_query_budget() {
        let conn = test_conn();
        conn.execute(
            "INSERT INTO budgets (budget_no, name, department, year, period, status, total_planned_milli, total_actual_milli, created_by, created_at) VALUES ('BUD-2026-001', 'Production Budget', 'Production', 2026, 'annual', 'draft', 10000000, 0, 'admin', datetime('now'))",
            [],
        ).unwrap();
        let name: String = conn
            .query_row("SELECT name FROM budgets LIMIT 1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(name, "Production Budget");
    }

    #[test]
    fn test_insert_and_query_fixed_asset() {
        let conn = test_conn();
        conn.execute(
            "INSERT INTO fixed_assets (asset_no, name, category, purchase_date, purchase_cost_milli, current_value_milli, status, active, created_at) VALUES ('FA-001', 'Cup Machine', 'machinery', '2026-01-15', 50000000, 45000000, 'active', 1, datetime('now'))",
            [],
        ).unwrap();
        let name: String = conn
            .query_row("SELECT name FROM fixed_assets LIMIT 1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(name, "Cup Machine");
    }

    #[test]
    fn test_insert_and_query_notification() {
        let conn = test_conn();
        conn.execute(
            "INSERT INTO notifications (notification_type, title, message, severity, read_status, created_at) VALUES ('alert', 'Low Stock', 'Item X is low', 'warning', 'unread', datetime('now'))",
            [],
        ).unwrap();
        let title: String = conn
            .query_row("SELECT title FROM notifications LIMIT 1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(title, "Low Stock");
    }

    #[test]
    fn test_audit_log_insert() {
        let conn = test_conn();
        conn.execute(
            "INSERT INTO audit_logs (ts, user_id, username, action, entity, entity_id) VALUES (datetime('now'), 1, 'admin', 'create', 'customers', 1)",
            [],
        ).unwrap();
        let action: String = conn
            .query_row("SELECT action FROM audit_logs LIMIT 1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(action, "create");
    }

    #[test]
    fn test_wal_mode_enabled() {
        let conn = test_conn();
        let mode: String = conn
            .query_row("PRAGMA journal_mode", [], |r| r.get(0))
            .unwrap();
        assert_eq!(mode, "wal");
    }

    #[test]
    fn test_foreign_keys_enabled() {
        let conn = test_conn();
        let fk_on: i32 = conn
            .query_row("PRAGMA foreign_keys", [], |r| r.get(0))
            .unwrap();
        assert_eq!(fk_on, 1);
    }

    // ─── Business Logic Integration Tests ─────────────────────────────

    #[test]
    fn test_full_invoice_flow() {
        let conn = test_conn();

        // Create customer
        conn.execute(
            "INSERT INTO customers (name, credit_limit_milli, active) VALUES ('Test Customer', 1000000, 1)",
            [],
        ).unwrap();
        let cust_id: i64 = conn.last_insert_rowid();

        // Create product
        conn.execute(
            "INSERT INTO products (name_ar, default_price_milli, default_cost_milli, vat_pct, cups_per_carton, active) VALUES ('Cup Product', 5000, 2000, 5.0, 50, 1)",
            [],
        ).unwrap();
        let prod_id: i64 = conn.last_insert_rowid();

        // Create invoice header
        conn.execute(
            "INSERT INTO sales_invoices (inv_no, date, customer_id, vat_enabled, net_milli, vat_milli, total_milli, paid_milli, status) VALUES ('INV-TEST-001', '2026-07-26', ?, 1, 5000, 250, 5250, 0, 'draft')",
            [cust_id],
        ).unwrap();
        let inv_id: i64 = conn.last_insert_rowid();

        // Create invoice line
        conn.execute(
            "INSERT INTO sales_invoice_lines (invoice_id, product_id, cartons, unit_price_milli, line_net_milli, vat_pct, vat_milli) VALUES (?, ?, 1, 5000, 5000, 5.0, 250)",
            rusqlite::params![inv_id, prod_id],
        ).unwrap();

        // Verify invoice exists with line
        let inv_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sales_invoices WHERE id = ?", [inv_id], |r| r.get(0),
        ).unwrap();
        assert_eq!(inv_count, 1);

        let line_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sales_invoice_lines WHERE invoice_id = ?", [inv_id], |r| r.get(0),
        ).unwrap();
        assert_eq!(line_count, 1);

        // Post invoice
        conn.execute(
            "UPDATE sales_invoices SET status = 'posted' WHERE id = ?", [inv_id],
        ).unwrap();

        let status: String = conn.query_row(
            "SELECT status FROM sales_invoices WHERE id = ?", [inv_id], |r| r.get(0),
        ).unwrap();
        assert_eq!(status, "posted");
    }

    #[test]
    fn test_void_invoice_restores_stock() {
        let conn = test_conn();

        // Product + inventory item (finished) with 10 cartons on hand
        conn.execute(
            "INSERT INTO products(code, name_ar, default_price_milli) VALUES('P-VOID', 'Void Test', 5000)",
            [],
        )
        .unwrap();
        let prod_id: i64 = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO inventory_items(code, name_ar, kind, uom, product_id, qty_on_hand, avg_cost_milli, active)
             VALUES('IV-VOID', 'Void Item', 'finished', 'carton', ?, 10, 2000, 1)",
            [prod_id],
        )
        .unwrap();
        let item_id: i64 = conn.last_insert_rowid();

        // Customer + invoice header + line
        conn.execute(
            "INSERT INTO customers(code, name, ctype) VALUES('C-VOID', 'Void Customer', 'credit')",
            [],
        )
        .unwrap();
        let cust_id: i64 = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO sales_invoices(inv_no, date, customer_id, vat_enabled, net_milli, vat_milli, total_milli, status)
             VALUES('INV-VOID-1', '2026-08-01', ?, 1, 5000, 250, 5250, 'Posted')",
            [cust_id],
        )
        .unwrap();
        let inv_id: i64 = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO sales_invoice_lines(invoice_id, product_id, cartons, unit_price_milli, line_net_milli, vat_pct, vat_milli)
             VALUES(?, ?, 4, 5000, 5000, 5.0, 250)",
            rusqlite::params![inv_id, prod_id],
        )
        .unwrap();

        // Simulate the deduction performed by post_invoice
        conn.execute(
            "UPDATE inventory_items SET qty_on_hand = qty_on_hand - 4 WHERE id = ?",
            [item_id],
        )
        .unwrap();
        let after_post: f64 = conn
            .query_row("SELECT qty_on_hand FROM inventory_items WHERE id=?", [item_id], |r| r.get(0))
            .unwrap();
        assert_eq!(after_post, 6.0);

        // Void restores the sold quantities
        crate::commands::invoices::restore_invoice_stock(&conn, inv_id).unwrap();

        let after_void: f64 = conn
            .query_row("SELECT qty_on_hand FROM inventory_items WHERE id=?", [item_id], |r| r.get(0))
            .unwrap();
        assert_eq!(after_void, 10.0);

        let reversal_movements: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM inventory_movements WHERE item_id=? AND mtype='sale_reversal' AND ref_id=?",
                rusqlite::params![item_id, inv_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(reversal_movements, 1);
    }

    #[test]
    fn test_post_cogs_uses_milli_unit_cost() {
        let conn = test_conn();

        conn.execute(
            "INSERT INTO products(code, name_ar, default_price_milli) VALUES('P-COGS', 'Cogs Test', 5000)",
            [],
        )
        .unwrap();
        let prod_id: i64 = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO inventory_items(code, name_ar, kind, uom, product_id, qty_on_hand, avg_cost_milli, active)
             VALUES('IV-COGS', 'Cogs Item', 'finished', 'carton', ?, 100, 2000, 1)",
            [prod_id],
        )
        .unwrap();
        let item_id: i64 = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO customers(code, name, ctype) VALUES('C-COGS', 'Cogs Customer', 'credit')",
            [],
        )
        .unwrap();
        let cust_id: i64 = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO sales_invoices(inv_no, date, customer_id, vat_enabled, net_milli, vat_milli, total_milli, status)
             VALUES('INV-COGS-1', '2026-08-01', ?, 1, 10000, 500, 10500, 'draft')",
            [cust_id],
        )
        .unwrap();
        let inv_id: i64 = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO sales_invoice_lines(invoice_id, product_id, cartons, unit_price_milli, line_net_milli, vat_pct, vat_milli)
             VALUES(?, ?, 5, 2000, 10000, 5.0, 500)",
            rusqlite::params![inv_id, prod_id],
        )
        .unwrap();

        // avg_cost_milli=2000 (2 OMR) per carton x 5 cartons = 10000 milli (10 OMR),
        // NOT 10,000,000 (which a stray x1000 would produce).
        let cogs = crate::commands::invoices::deduct_invoice_stock(&conn, inv_id).unwrap();
        assert_eq!(cogs, 10000);

        let unit_cost: i64 = conn
            .query_row(
                "SELECT unit_cost_milli FROM inventory_movements WHERE item_id=? AND mtype='sale'",
                [item_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(unit_cost, 2000);

        let qty: f64 = conn
            .query_row(
                "SELECT qty_on_hand FROM inventory_items WHERE id=?",
                [item_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(qty, 95.0);
    }

    #[test]
    fn test_journal_entry_double_entry() {
        let conn = test_conn();

        // Create accounts for FK
        conn.execute("INSERT OR IGNORE INTO accounts (code, name_ar, type, is_system) VALUES ('1100', 'Cash', 'asset', 1)", []).unwrap();
        conn.execute("INSERT OR IGNORE INTO accounts (code, name_ar, type, is_system) VALUES ('4100', 'Sales Revenue', 'revenue', 1)", []).unwrap();

        // Create journal entry
        conn.execute(
            "INSERT INTO journal_entries (entry_no, date, memo, created_by) VALUES ('JE-001', '2026-07-26', 'Test entry', 'admin')",
            [],
        ).unwrap();
        let je_id: i64 = conn.last_insert_rowid();

        // Debit side (account must exist in accounts table)
        conn.execute(
            "INSERT INTO journal_entry_lines (entry_id, account_code, debit_milli, credit_milli) VALUES (?, '1100', 100000, 0)",
            [je_id],
        ).unwrap();

        // Credit side
        conn.execute(
            "INSERT INTO journal_entry_lines (entry_id, account_code, debit_milli, credit_milli) VALUES (?, '4100', 0, 100000)",
            [je_id],
        ).unwrap();

        // Verify balanced entry
        let total_debit: i64 = conn.query_row(
            "SELECT COALESCE(SUM(debit_milli), 0) FROM journal_entry_lines WHERE entry_id = ?",
            [je_id], |r| r.get(0),
        ).unwrap();
        let total_credit: i64 = conn.query_row(
            "SELECT COALESCE(SUM(credit_milli), 0) FROM journal_entry_lines WHERE entry_id = ?",
            [je_id], |r| r.get(0),
        ).unwrap();
        assert_eq!(total_debit, total_credit, "Journal entry must be balanced");
        assert_eq!(total_debit, 100000);
    }

    #[test]
    fn test_trial_balance_query() {
        let conn = test_conn();

        // Create accounts (must exist for FK)
        conn.execute("INSERT OR IGNORE INTO accounts (code, name_ar, type, is_system) VALUES ('1100', 'Cash', 'asset', 1)", []).unwrap();
        conn.execute("INSERT OR IGNORE INTO accounts (code, name_ar, type, is_system) VALUES ('4100', 'Sales Revenue', 'revenue', 1)", []).unwrap();

        // Create balanced journal entry
        conn.execute(
            "INSERT INTO journal_entries (entry_no, date, memo, created_by) VALUES ('JE-TB-001', '2026-07-26', 'TB test', 'admin')",
            [],
        ).unwrap();
        let je_id: i64 = conn.last_insert_rowid();
        conn.execute("INSERT INTO journal_entry_lines (entry_id, account_code, debit_milli, credit_milli) VALUES (?, '1100', 50000, 0)", [je_id]).unwrap();
        conn.execute("INSERT INTO journal_entry_lines (entry_id, account_code, debit_milli, credit_milli) VALUES (?, '4100', 0, 50000)", [je_id]).unwrap();

        // Query trial balance
        let total_debit: i64 = conn.query_row(
            "SELECT COALESCE(SUM(debit_milli), 0) FROM journal_entry_lines", [], |r| r.get(0),
        ).unwrap();
        let total_credit: i64 = conn.query_row(
            "SELECT COALESCE(SUM(credit_milli), 0) FROM journal_entry_lines", [], |r| r.get(0),
        ).unwrap();
        assert_eq!(total_debit, total_credit);
    }

    #[test]
    fn test_stock_adjustment() {
        let conn = test_conn();

        // Create inventory item
        conn.execute(
            "INSERT INTO inventory_items (name_ar, kind, uom, qty_on_hand, avg_cost_milli, active) VALUES ('Paper Roll', 'raw_material', 'kg', 100.0, 5000, 1)",
            [],
        ).unwrap();
        let item_id: i64 = conn.last_insert_rowid();

        // Adjust stock down
        conn.execute(
            "UPDATE inventory_items SET qty_on_hand = qty_on_hand - 25.0 WHERE id = ?",
            [item_id],
        ).unwrap();

        let qty: f64 = conn.query_row(
            "SELECT qty_on_hand FROM inventory_items WHERE id = ?", [item_id], |r| r.get(0),
        ).unwrap();
        assert!((qty - 75.0).abs() < 0.01);
    }

    #[test]
    fn test_employee_leave_request() {
        let conn = test_conn();

        // Create employee
        conn.execute(
            "INSERT INTO employees (code, name, nationality, job, active) VALUES ('EMP-001', 'Test Employee', 'Omani', 'Operator', 1)",
            [],
        ).unwrap();
        let emp_id: i64 = conn.last_insert_rowid();

        // Create leave type (use INSERT OR IGNORE since schema seeds default types)
        conn.execute(
            "INSERT OR IGNORE INTO employee_leave_types (code, name, default_days_per_year) VALUES ('annual', 'Annual Leave', 21)",
            [],
        ).unwrap();
        let lt_id: i64 = conn.last_insert_rowid();
        // If insert was ignored (already exists), fetch existing id
        let lt_id = if lt_id == 0 {
            conn.query_row("SELECT id FROM employee_leave_types WHERE code = 'annual'", [], |r| r.get(0)).unwrap()
        } else {
            lt_id
        };

        // Create leave request (uses leave_type_id, from_date, to_date)
        conn.execute(
            "INSERT INTO employee_leave_requests (employee_id, leave_type_id, from_date, to_date, days, status) VALUES (?, ?, '2026-08-01', '2026-08-05', 5, 'Pending')",
            rusqlite::params![emp_id, lt_id],
        ).unwrap();

        let status: String = conn.query_row(
            "SELECT status FROM employee_leave_requests WHERE employee_id = ?", [emp_id], |r| r.get(0),
        ).unwrap();
        assert_eq!(status, "Pending");

        // Approve
        conn.execute(
            "UPDATE employee_leave_requests SET status = 'Approved' WHERE employee_id = ?",
            [emp_id],
        ).unwrap();

        let status: String = conn.query_row(
            "SELECT status FROM employee_leave_requests WHERE employee_id = ?", [emp_id], |r| r.get(0),
        ).unwrap();
        assert_eq!(status, "Approved");
    }

    #[test]
    fn test_supplier_payment_flow() {
        let conn = test_conn();

        // Create supplier
        conn.execute(
            "INSERT INTO suppliers (name, active) VALUES ('Paper Supplier', 1)",
            [],
        ).unwrap();
        let supp_id: i64 = conn.last_insert_rowid();

        // Create supplier payment (no purchase_id column exists)
        conn.execute(
            "INSERT INTO supplier_payments (supplier_id, amount_milli, method, date) VALUES (?, 250000, 'cash', '2026-07-26')",
            [supp_id],
        ).unwrap();

        let pay_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM supplier_payments WHERE supplier_id = ?", [supp_id], |r| r.get(0),
        ).unwrap();
        assert_eq!(pay_count, 1);
    }

    #[test]
    fn test_post_purchase_posts_journal() {
        let conn = test_conn();

        conn.execute(
            "INSERT INTO suppliers (name, active) VALUES ('Paper Supplier', 1)",
            [],
        ).unwrap();
        let supp_id: i64 = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO purchases (pur_no, date, supplier_id, vat_enabled, net_milli, vat_milli, total_milli, paid_milli, status)
             VALUES ('PUR-2026-0001', '2026-07-26', ?, 1, 1000000, 50000, 1050000, 0, 'draft')",
            [supp_id],
        ).unwrap();
        let pur_id: i64 = conn.last_insert_rowid();

        let journal_id = crate::commands::purchases::post_purchase_inner(&conn, pur_id).unwrap();

        let line_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM journal_entry_lines WHERE entry_id = ?", [journal_id], |r| r.get(0),
        ).unwrap();
        assert_eq!(line_count, 3);

        let (inventory_dr, vat_dr): (i64, i64) = conn.query_row(
            "SELECT SUM(CASE WHEN account_code='1400' THEN debit_milli ELSE 0 END),
                    SUM(CASE WHEN account_code='2100' THEN debit_milli ELSE 0 END)
             FROM journal_entry_lines WHERE entry_id = ?",
            [journal_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        ).unwrap();
        assert_eq!(inventory_dr, 1000000);
        assert_eq!(vat_dr, 50000);

        let ap_credit: i64 = conn.query_row(
            "SELECT credit_milli FROM journal_entry_lines WHERE entry_id = ? AND account_code = '2200'",
            [journal_id], |r| r.get(0),
        ).unwrap();
        assert_eq!(ap_credit, 1050000);

        let (status, stored_journal): (String, i64) = conn.query_row(
            "SELECT status, journal_id FROM purchases WHERE id = ?", [pur_id], |r| Ok((r.get(0)?, r.get(1)?)),
        ).unwrap();
        assert_eq!(status, "Posted");
        assert_eq!(stored_journal, journal_id);

        assert!(crate::commands::purchases::post_purchase_inner(&conn, pur_id).is_err());
    }

    #[test]
    fn test_post_purchase_updates_stock_and_avg_cost() {
        let conn = test_conn();

        conn.execute("INSERT INTO suppliers (name, active) VALUES ('Stock Supplier', 1)", []).unwrap();
        let supp_id: i64 = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO inventory_items(code, name_ar, kind, uom, qty_on_hand, avg_cost_milli, active)
             VALUES('IV-PUR', 'Raw Item', 'raw_material', 'kg', 100, 3000, 1)",
            [],
        )
        .unwrap();
        let item_id: i64 = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO purchases (pur_no, date, supplier_id, vat_enabled, net_milli, vat_milli, total_milli, paid_milli, status)
             VALUES ('PUR-2026-0002', '2026-08-01', ?, 1, 500000, 25000, 525000, 0, 'draft')",
            [supp_id],
        )
        .unwrap();
        let pur_id: i64 = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO purchase_lines(purchase_id, item_id, qty, unit_cost_milli, line_net_milli, vat_pct, vat_milli)
             VALUES(?, ?, 50, 4000, 200000, 5.0, 10000)",
            rusqlite::params![pur_id, item_id],
        )
        .unwrap();

        crate::commands::purchases::post_purchase_inner(&conn, pur_id).unwrap();

        // qty 100 + 50 = 150; weighted avg = (100*3000 + 50*4000)/150 = 3333.33 -> 3333
        let (qty, avg_cost): (f64, i64) = conn
            .query_row(
                "SELECT qty_on_hand, avg_cost_milli FROM inventory_items WHERE id=?",
                [item_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(qty, 150.0);
        assert_eq!(avg_cost, 3333);

        let movement_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM inventory_movements WHERE item_id=? AND mtype='purchase' AND ref_id=?",
                rusqlite::params![item_id, pur_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(movement_count, 1);
    }

    #[test]
    fn test_supplier_payment_fifo_allocation() {
        let conn = test_conn();

        conn.execute("INSERT INTO suppliers (name, active) VALUES ('Fifo Supplier', 1)", []).unwrap();
        let supp_id: i64 = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO purchases(pur_no, date, supplier_id, total_milli, paid_milli, status)
             VALUES('PUR-FIFO-1','2026-07-01',?,100000,0,'Posted')",
            [supp_id],
        )
        .unwrap();
        let p1: i64 = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO purchases(pur_no, date, supplier_id, total_milli, paid_milli, status)
             VALUES('PUR-FIFO-2','2026-07-10',?,50000,0,'Posted')",
            [supp_id],
        )
        .unwrap();
        let p2: i64 = conn.last_insert_rowid();
        // Draft purchases must never absorb payments.
        conn.execute(
            "INSERT INTO purchases(pur_no, date, supplier_id, total_milli, paid_milli, status)
             VALUES('PUR-FIFO-3','2026-07-11',?,50000,0,'draft')",
            [supp_id],
        )
        .unwrap();

        // Pay 120,000: oldest (P1) absorbs 100,000, P2 absorbs the remaining 20,000.
        let leftover =
            crate::commands::purchases::allocate_payment_fifo(&conn, supp_id, 120000, None).unwrap();
        assert_eq!(leftover, 0);
        let (paid1, paid2): (i64, i64) = conn
            .query_row(
                "SELECT (SELECT paid_milli FROM purchases WHERE id=?),
                        (SELECT paid_milli FROM purchases WHERE id=?)",
                rusqlite::params![p1, p2],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(paid1, 100000);
        assert_eq!(paid2, 20000);

        // Overpay: P2 absorbs its remaining 30,000, the excess stays on account.
        let leftover2 =
            crate::commands::purchases::allocate_payment_fifo(&conn, supp_id, 40000, None).unwrap();
        assert_eq!(leftover2, 10000);
        let paid2_after: i64 = conn
            .query_row("SELECT paid_milli FROM purchases WHERE id=?", [p2], |r| r.get(0))
            .unwrap();
        assert_eq!(paid2_after, 50000);
    }

    #[test]
    fn test_customer_payment_fifo_allocation() {
        let conn = test_conn();

        conn.execute("INSERT INTO customers (name, active) VALUES ('Fifo Customer', 1)", []).unwrap();
        let cust_id: i64 = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO sales_invoices(inv_no, date, customer_id, payment_type, total_milli, paid_milli, status)
             VALUES('INV-FIFO-1','2026-07-01',?, 'credit', 100000, 0, 'Posted')",
            [cust_id],
        )
        .unwrap();
        let i1: i64 = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO sales_invoices(inv_no, date, customer_id, payment_type, total_milli, paid_milli, status)
             VALUES('INV-FIFO-2','2026-07-10',?, 'credit', 50000, 0, 'Posted')",
            [cust_id],
        )
        .unwrap();
        let i2: i64 = conn.last_insert_rowid();
        // Cash invoices and drafts must never absorb credit payments.
        conn.execute(
            "INSERT INTO sales_invoices(inv_no, date, customer_id, payment_type, total_milli, paid_milli, status)
             VALUES('INV-FIFO-3','2026-07-11',?, 'cash', 50000, 0, 'Posted')",
            [cust_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO sales_invoices(inv_no, date, customer_id, payment_type, total_milli, paid_milli, status)
             VALUES('INV-FIFO-4','2026-07-12',?, 'credit', 50000, 0, 'draft')",
            [cust_id],
        )
        .unwrap();

        // Pay 120,000: oldest (I1) absorbs 100,000, I2 absorbs the remaining 20,000.
        let leftover =
            crate::commands::customers::allocate_customer_payment_fifo(&conn, cust_id, 120000, None).unwrap();
        assert_eq!(leftover, 0);
        let (paid1, paid2): (i64, i64) = conn
            .query_row(
                "SELECT (SELECT paid_milli FROM sales_invoices WHERE id=?),
                        (SELECT paid_milli FROM sales_invoices WHERE id=?)",
                rusqlite::params![i1, i2],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(paid1, 100000);
        assert_eq!(paid2, 20000);

        // Overpay: I2 absorbs its remaining 30,000, the excess stays on account.
        let leftover2 =
            crate::commands::customers::allocate_customer_payment_fifo(&conn, cust_id, 40000, None).unwrap();
        assert_eq!(leftover2, 10000);
        let paid2_after: i64 = conn
            .query_row("SELECT paid_milli FROM sales_invoices WHERE id=?", [i2], |r| r.get(0))
            .unwrap();
        assert_eq!(paid2_after, 50000);

        // exclude_id: allocating while skipping a fully-open invoice leaves it untouched.
        conn.execute("UPDATE sales_invoices SET paid_milli = 0", []).unwrap();
        let leftover3 =
            crate::commands::customers::allocate_customer_payment_fifo(&conn, cust_id, 50000, Some(i1)).unwrap();
        assert_eq!(leftover3, 0);
        let (paid1b, paid2b): (i64, i64) = conn
            .query_row(
                "SELECT (SELECT paid_milli FROM sales_invoices WHERE id=?),
                        (SELECT paid_milli FROM sales_invoices WHERE id=?)",
                rusqlite::params![i1, i2],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(paid1b, 0);
        assert_eq!(paid2b, 50000);
    }

    #[test]
    fn test_void_purchase_reverses_stock_and_journal() {
        let mut conn = test_conn();

        conn.execute("INSERT INTO suppliers (name, active) VALUES ('Void Supplier', 1)", []).unwrap();
        let supp_id: i64 = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO inventory_items(code, name_ar, kind, uom, qty_on_hand, avg_cost_milli, active)
             VALUES('IV-VP', 'VP Item', 'raw_material', 'kg', 100, 3000, 1)",
            [],
        )
        .unwrap();
        let item_id: i64 = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO purchases(pur_no, date, supplier_id, vat_enabled, net_milli, vat_milli, total_milli, status)
             VALUES('PUR-VP-1','2026-08-01',?,1,500000,25000,525000,'draft')",
            [supp_id],
        )
        .unwrap();
        let pur_id: i64 = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO purchase_lines(purchase_id, item_id, qty, unit_cost_milli, line_net_milli, vat_pct, vat_milli)
             VALUES(?,?,100,5000,500000,5.0,25000)",
            rusqlite::params![pur_id, item_id],
        )
        .unwrap();

        let journal = crate::commands::purchases::post_purchase_inner(&conn, pur_id).unwrap();
        // 100@3000 + 100@5000 -> 200 qty @ 4000 (exact weighted average).
        let (qty, avg): (f64, i64) = conn
            .query_row(
                "SELECT qty_on_hand, avg_cost_milli FROM inventory_items WHERE id=?",
                [item_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(qty, 200.0);
        assert_eq!(avg, 4000);

        crate::commands::purchases::void_purchase_inner(&mut conn, 1, pur_id, Some("خطأ في التوريد".to_string())).unwrap();

        let (qty2, avg2): (f64, i64) = conn
            .query_row(
                "SELECT qty_on_hand, avg_cost_milli FROM inventory_items WHERE id=?",
                [item_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(qty2, 100.0);
        assert_eq!(avg2, 3000);

        let (status, stored_journal): (String, i64) = conn
            .query_row(
                "SELECT status, journal_id FROM purchases WHERE id=?",
                [pur_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(status, "Void");
        assert_eq!(stored_journal, journal);

        let reversed_by: Option<i64> = conn
            .query_row(
                "SELECT reversed_by FROM journal_entries WHERE id=?",
                [journal],
                |r| r.get(0),
            )
            .unwrap();
        assert!(reversed_by.is_some());

        let reversal_movements: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM inventory_movements WHERE item_id=? AND mtype='purchase_reversal' AND ref_id=?",
                rusqlite::params![item_id, pur_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(reversal_movements, 1);
    }

    #[test]
    fn test_supplier_statement_columns_exist() {
        let conn = test_conn();
        let pur_cols: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('purchases') WHERE name='pur_no'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(pur_cols, 1);
        let pay_cols: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('supplier_payments') WHERE name='pay_no'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(pay_cols, 1);
    }

    #[test]
    fn test_supplier_balance_lifecycle() {
        let mut conn = test_conn();

        conn.execute("INSERT INTO suppliers (name, active) VALUES ('Bal Supplier', 1)", []).unwrap();
        let supp_id: i64 = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO purchases(pur_no, date, supplier_id, vat_enabled, net_milli, vat_milli, total_milli, status)
             VALUES('PUR-BAL-1','2026-08-05',?,1,400000,20000,420000,'draft')",
            [supp_id],
        ).unwrap();
        let pur_id: i64 = conn.last_insert_rowid();

        // A draft purchase creates no obligation yet.
        let draft_balance: i64 = conn.query_row("SELECT balance_milli FROM suppliers WHERE id=?", [supp_id], |r| r.get(0)).unwrap();
        assert_eq!(draft_balance, 0);

        // Posting adds the full invoice total to what we owe the supplier.
        crate::commands::purchases::post_purchase_inner(&conn, pur_id).unwrap();
        let posted_balance: i64 = conn.query_row("SELECT balance_milli FROM suppliers WHERE id=?", [supp_id], |r| r.get(0)).unwrap();
        assert_eq!(posted_balance, 420000);

        // Payment reduces the balance (mirror of create_supplier_payment).
        conn.execute(
            "INSERT INTO supplier_payments(pay_no, supplier_id, date, amount_milli, method, created_by)
             VALUES('PAY-BAL-1', ?, '2026-08-06', 200000, 'cash', 'system')",
            [supp_id],
        ).unwrap();
        let leftover = crate::commands::purchases::allocate_payment_fifo(&conn, supp_id, 200000, None).unwrap();
        assert_eq!(leftover, 0);
        conn.execute(
            "UPDATE suppliers SET balance_milli = COALESCE(balance_milli, 0) - 200000 WHERE id = ?",
            [supp_id],
        ).unwrap();
        let after_payment: i64 = conn.query_row("SELECT balance_milli FROM suppliers WHERE id=?", [supp_id], |r| r.get(0)).unwrap();
        assert_eq!(after_payment, 220000);

        // Voiding the posted purchase drops the obligation; the 200,000 already paid
        // becomes an on-account credit, so the running balance mirrors that.
        crate::commands::purchases::void_purchase_inner(&mut conn, 1, pur_id, None).unwrap();
        let after_void: i64 = conn.query_row("SELECT balance_milli FROM suppliers WHERE id=?", [supp_id], |r| r.get(0)).unwrap();
        assert_eq!(after_void, -200000);
    }

    #[test]
    fn test_customer_balance_lifecycle() {
        let mut conn = test_conn();

        conn.execute("INSERT INTO customers (name, active) VALUES ('Bal Customer', 1)", []).unwrap();
        let cust_id: i64 = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO sales_invoices(inv_no, date, customer_id, payment_type, net_milli, vat_milli, total_milli, status)
             VALUES('INV-BAL-1','2026-08-05',?,'credit',100000,5000,105000,'Draft')",
            [cust_id],
        ).unwrap();
        let inv_id: i64 = conn.last_insert_rowid();

        // A draft invoice creates no receivable yet.
        let draft_balance: i64 = conn.query_row("SELECT balance_milli FROM customers WHERE id=?", [cust_id], |r| r.get(0)).unwrap();
        assert_eq!(draft_balance, 0);

        // Posting a credit invoice adds the full total to the customer's balance.
        crate::commands::invoices::post_invoice_inner(&mut conn, 1, inv_id).unwrap();
        let posted_balance: i64 = conn.query_row("SELECT balance_milli FROM customers WHERE id=?", [cust_id], |r| r.get(0)).unwrap();
        assert_eq!(posted_balance, 105000);

        // Voiding the posted invoice reverses the receivable.
        crate::commands::invoices::void_invoice_inner(&mut conn, 1, inv_id, None).unwrap();
        let after_void: i64 = conn.query_row("SELECT balance_milli FROM customers WHERE id=?", [cust_id], |r| r.get(0)).unwrap();
        assert_eq!(after_void, 0);
    }

    #[test]
    fn test_cash_invoice_does_not_affect_customer_balance() {
        let mut conn = test_conn();

        conn.execute("INSERT INTO customers (name, active) VALUES ('Cash Customer', 1)", []).unwrap();
        let cust_id: i64 = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO sales_invoices(inv_no, date, customer_id, payment_type, net_milli, vat_milli, total_milli, status)
             VALUES('INV-CASH-1','2026-08-06',?,'cash',50000,0,50000,'Draft')",
            [cust_id],
        ).unwrap();
        let inv_id: i64 = conn.last_insert_rowid();

        // Cash sales settle immediately: no AR receivable, no balance change.
        crate::commands::invoices::post_invoice_inner(&mut conn, 1, inv_id).unwrap();
        let balance: i64 = conn.query_row("SELECT balance_milli FROM customers WHERE id=?", [cust_id], |r| r.get(0)).unwrap();
        assert_eq!(balance, 0);
    }

    #[test]
    fn test_production_order_flow() {
        let conn = test_conn();

        // Create production order (schema: prod_no, date, status)
        conn.execute(
            "INSERT INTO production_orders (prod_no, date, status) VALUES ('PRO-001', '2026-07-26', 'Draft')",
            [],
        ).unwrap();
        let order_id: i64 = conn.last_insert_rowid();

        // Create product for FK reference
        conn.execute("INSERT INTO products (name_ar, active) VALUES ('Test Product', 1)", []).unwrap();
        let prod_id: i64 = conn.last_insert_rowid();

        // Add production line (requires product_id FK)
        conn.execute(
            "INSERT INTO production_lines (order_id, product_id) VALUES (?, ?)",
            rusqlite::params![order_id, prod_id],
        ).unwrap();

        let line_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM production_lines WHERE order_id = ?", [order_id], |r| r.get(0),
        ).unwrap();
        assert_eq!(line_count, 1);
    }

    #[test]
    fn test_concurrent_read_write() {
        let conn = test_conn();

        // Insert data
        conn.execute(
            "INSERT INTO customers (name, active) VALUES ('Concurrent Test', 1)",
            [],
        ).unwrap();

        // Read while write is happening (WAL mode allows this)
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM customers", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 1);

        // Insert more while reading
        conn.execute("INSERT INTO customers (name, active) VALUES ('C2', 1)", []).unwrap();
        conn.execute("INSERT INTO customers (name, active) VALUES ('C3', 1)", []).unwrap();

        let count: i64 = conn.query_row("SELECT COUNT(*) FROM customers", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_schema_has_all_expected_indexes() {
        let conn = test_conn();
        let index_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name NOT LIKE 'sqlite_%'",
            [], |r| r.get(0),
        ).unwrap();
        // Should have many indexes for performance
        assert!(index_count >= 30, "Expected at least 30 indexes, found {}", index_count);
    }

    #[test]
    fn test_notification_read_unread_flow() {
        let conn = test_conn();

        // Create notifications
        conn.execute("INSERT INTO notifications (notification_type, title, message, severity, read_status, created_at) VALUES ('alert', 'Stock Low', 'Item X', 'warning', 'unread', datetime('now'))", []).unwrap();
        conn.execute("INSERT INTO notifications (notification_type, title, message, severity, read_status, created_at) VALUES ('info', 'System', 'Welcome', 'info', 'unread', datetime('now'))", []).unwrap();

        let unread: i64 = conn.query_row("SELECT COUNT(*) FROM notifications WHERE read_status = 'unread'", [], |r| r.get(0)).unwrap();
        assert_eq!(unread, 2);

        // Mark one as read
        conn.execute("UPDATE notifications SET read_status = 'read' WHERE id = 1", []).unwrap();

        let unread: i64 = conn.query_row("SELECT COUNT(*) FROM notifications WHERE read_status = 'unread'", [], |r| r.get(0)).unwrap();
        assert_eq!(unread, 1);

        // Mark all as read
        conn.execute("UPDATE notifications SET read_status = 'read' WHERE read_status = 'unread'", []).unwrap();

        let unread: i64 = conn.query_row("SELECT COUNT(*) FROM notifications WHERE read_status = 'unread'", [], |r| r.get(0)).unwrap();
        assert_eq!(unread, 0);
    }

    #[test]
    fn test_budget_vs_actual_calculation() {
        let conn = test_conn();

        // Create budget
        conn.execute(
            "INSERT INTO budgets (budget_no, name, department, year, period, status, total_planned_milli, total_actual_milli, created_by, created_at) VALUES ('BUD-VA-001', 'Q3 Budget', 'Sales', 2026, 'quarterly', 'approved', 5000000, 0, 'admin', datetime('now'))",
            [],
        ).unwrap();
        let budget_id: i64 = conn.last_insert_rowid();

        // Ensure accounts exist for FK references
        conn.execute("INSERT OR IGNORE INTO accounts(code, name_ar, type) VALUES ('4100', 'إيرادات المبيعات', 'revenue')", []).unwrap();
        conn.execute("INSERT OR IGNORE INTO accounts(code, name_ar, type) VALUES ('4200', 'إيرادات أخرى', 'revenue')", []).unwrap();

        // Create budget lines (must include required `category` column)
        conn.execute("INSERT INTO budget_lines (budget_id, category, account_code, planned_milli, actual_milli) VALUES (?, 'revenue', '4100', 3000000, 0)", [budget_id]).unwrap();
        conn.execute("INSERT INTO budget_lines (budget_id, category, account_code, planned_milli, actual_milli) VALUES (?, 'revenue', '4200', 2000000, 0)", [budget_id]).unwrap();

        // Update actuals (simulate spending)
        conn.execute("UPDATE budget_lines SET actual_milli = 1500000 WHERE budget_id = ? AND account_code = '4100'", [budget_id]).unwrap();

        // Query variance
        let planned: i64 = conn.query_row("SELECT SUM(planned_milli) FROM budget_lines WHERE budget_id = ?", [budget_id], |r| r.get(0)).unwrap();
        let actual: i64 = conn.query_row("SELECT SUM(actual_milli) FROM budget_lines WHERE budget_id = ?", [budget_id], |r| r.get(0)).unwrap();
        let variance = planned - actual;

        assert_eq!(planned, 5000000);
        assert_eq!(actual, 1500000);
        assert_eq!(variance, 3500000);
    }

    #[test]
    fn test_asset_depreciation_calculation() {
        let conn = test_conn();

        // Create asset
        conn.execute(
            "INSERT INTO fixed_assets (asset_no, name, category, purchase_date, purchase_cost_milli, current_value_milli, depreciation_method, depreciation_rate_pct, useful_life_months, accumulated_depreciation_milli, status, active, created_at) VALUES ('FA-DEP-001', 'Printer', 'equipment', '2026-01-01', 12000000, 12000000, 'straight_line', 20.0, 60, 0, 'active', 1, datetime('now'))",
            [],
        ).unwrap();
        let asset_id: i64 = conn.last_insert_rowid();

        // Calculate depreciation for 6 months (straight-line: 20% per year = ~1.67% per month)
        let cost: i64 = conn.query_row("SELECT purchase_cost_milli FROM fixed_assets WHERE id = ?", [asset_id], |r| r.get(0)).unwrap();
        let rate_pct: f64 = conn.query_row("SELECT depreciation_rate_pct FROM fixed_assets WHERE id = ?", [asset_id], |r| r.get(0)).unwrap();
        let months = 6i32;
        let annual_depr = cost as f64 * rate_pct / 100.0;
        let monthly_depr = annual_depr / 12.0;
        let total_depr = monthly_depr * months as f64;

        assert!(total_depr > 0.0, "Depreciation should be positive");
        assert!((total_depr - 1200000.0).abs() < 1.0, "6 months of 20% annual depreciation on 12M should be ~1.2M");
    }
}
