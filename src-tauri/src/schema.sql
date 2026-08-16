CREATE TABLE IF NOT EXISTS app_settings (
    key   TEXT PRIMARY KEY,
    value TEXT
);

CREATE TABLE IF NOT EXISTS doc_sequences (
    doc_type    TEXT NOT NULL,
    year        INTEGER NOT NULL,
    last_number INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (doc_type, year)
);

CREATE TABLE IF NOT EXISTS users (
    id                   INTEGER PRIMARY KEY AUTOINCREMENT,
    username             TEXT UNIQUE NOT NULL,
    full_name            TEXT,
    password_hash        TEXT NOT NULL,
    salt                 TEXT NOT NULL,
    role                 TEXT NOT NULL DEFAULT 'manager',
    active               INTEGER NOT NULL DEFAULT 1,
    must_change_password INTEGER NOT NULL DEFAULT 1,
    reset_token          TEXT,
    reset_token_expiry   TEXT,
    created_at           TEXT
);

CREATE TABLE IF NOT EXISTS company_settings (
    id             INTEGER PRIMARY KEY CHECK (id = 1),
    name           TEXT, factory_name TEXT, address TEXT, phone TEXT,
    email TEXT, vat_number TEXT, cr_number TEXT,
    logo_path TEXT, stamp_path TEXT, signature_path TEXT,
    footer_notes TEXT, bank_details TEXT,
    default_vat_pct REAL DEFAULT 5.0,
    currency TEXT NOT NULL DEFAULT 'OMR',
    fiscal_year_start TEXT NOT NULL DEFAULT '01-01',
    bank_name TEXT, bank_account_no TEXT, bank_iban TEXT, bank_swift TEXT
);

CREATE TABLE IF NOT EXISTS accounts (
    code      TEXT PRIMARY KEY,
    name_ar   TEXT, name_en TEXT,
    type      TEXT NOT NULL,
    parent    TEXT,
    is_system INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS journal_entries (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_no    TEXT,
    date        TEXT NOT NULL,
    memo        TEXT,
    ref_type    TEXT, ref_id INTEGER,
    created_by  TEXT, created_at TEXT,
    reversed_by INTEGER
);

CREATE TABLE IF NOT EXISTS journal_entry_lines (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_id     INTEGER NOT NULL REFERENCES journal_entries(id),
    account_code TEXT NOT NULL REFERENCES accounts(code),
    debit_milli  INTEGER NOT NULL DEFAULT 0,
    credit_milli INTEGER NOT NULL DEFAULT 0,
    memo         TEXT
);

CREATE TABLE IF NOT EXISTS products (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    code                TEXT, name_ar TEXT, name_en TEXT,
    size                TEXT, cup_type TEXT,
    cups_per_carton     INTEGER NOT NULL DEFAULT 1000,
    carton_type         TEXT,
    default_price_milli INTEGER NOT NULL DEFAULT 0,
    default_cost_milli  INTEGER NOT NULL DEFAULT 0,
    vat_pct             REAL NOT NULL DEFAULT 5.0,
    barcode             TEXT, notes TEXT,
    active              INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS product_prices (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    product_id  INTEGER NOT NULL REFERENCES products(id),
    customer_id INTEGER REFERENCES customers(id),
    price_milli INTEGER NOT NULL,
    note        TEXT
);
CREATE INDEX IF NOT EXISTS idx_pp_product ON product_prices(product_id);

CREATE TABLE IF NOT EXISTS inventory_items (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    code            TEXT, name_ar TEXT, name_en TEXT,
    kind            TEXT NOT NULL DEFAULT 'raw',
    uom             TEXT NOT NULL DEFAULT 'pcs',
    product_id      INTEGER REFERENCES products(id),
    qty_on_hand     REAL NOT NULL DEFAULT 0,
    avg_cost_milli  INTEGER NOT NULL DEFAULT 0,
    reorder_level   REAL NOT NULL DEFAULT 0,
    supplier_id     INTEGER REFERENCES suppliers(id),
    warehouse_id    INTEGER REFERENCES multi_warehouse(id),
    notes           TEXT,
    active          INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS inventory_movements (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    ts              TEXT NOT NULL,
    item_id         INTEGER NOT NULL REFERENCES inventory_items(id),
    mtype           TEXT NOT NULL,
    qty_in          REAL NOT NULL DEFAULT 0,
    qty_out         REAL NOT NULL DEFAULT 0,
    unit_cost_milli INTEGER NOT NULL DEFAULT 0,
    ref_type        TEXT, ref_id INTEGER,
    location        TEXT, user_id INTEGER REFERENCES users(id), notes TEXT
);

CREATE TABLE IF NOT EXISTS customers (
    id                    INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT, name TEXT NOT NULL, ctype TEXT DEFAULT 'credit',
    contact TEXT, phone TEXT, email TEXT, address TEXT, vat_number TEXT,
    credit_limit_milli    INTEGER NOT NULL DEFAULT 0,
    payment_terms         TEXT,
    payment_terms_days    INTEGER NOT NULL DEFAULT 30,
    opening_balance_milli INTEGER NOT NULL DEFAULT 0,
    balance_milli         INTEGER NOT NULL DEFAULT 0,
    notes TEXT, active INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS suppliers (
    id                    INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT, name TEXT NOT NULL, is_foreign INTEGER DEFAULT 0,
    contact TEXT, phone TEXT, email TEXT, address TEXT,
    currency TEXT DEFAULT 'OMR', payment_terms TEXT,
    opening_balance_milli INTEGER NOT NULL DEFAULT 0,
    balance_milli         INTEGER NOT NULL DEFAULT 0,
    bank_details TEXT, notes TEXT, active INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS machines (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT, name TEXT NOT NULL, mtype TEXT, supported_products TEXT,
    purchase_date TEXT, supplier TEXT, cost_milli INTEGER DEFAULT 0,
    capacity_cpm INTEGER DEFAULT 0, status TEXT DEFAULT 'active',
    notes TEXT, active INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS employees (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT, name TEXT NOT NULL, nationality TEXT, job TEXT,
    salary_milli INTEGER DEFAULT 0, allowances_milli INTEGER DEFAULT 0,
    phone TEXT, passport_no TEXT,
    passport_expiry TEXT, residence_expiry TEXT, visa_expiry TEXT,
    workpermit_expiry TEXT, insurance_expiry TEXT, contract_end TEXT,
    joining_date TEXT, active INTEGER NOT NULL DEFAULT 1, notes TEXT
);

CREATE TABLE IF NOT EXISTS bom (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    product_id INTEGER NOT NULL REFERENCES products(id),
    item_id    INTEGER NOT NULL REFERENCES inventory_items(id),
    qty_per_carton REAL NOT NULL DEFAULT 0,
    waste_pct REAL NOT NULL DEFAULT 0,
    active INTEGER NOT NULL DEFAULT 1
);
CREATE INDEX IF NOT EXISTS idx_bom_product ON bom(product_id);
CREATE INDEX IF NOT EXISTS idx_bom_item ON bom(item_id);

CREATE TABLE IF NOT EXISTS production_orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    prod_no TEXT, date TEXT NOT NULL, shift TEXT, machine_id INTEGER REFERENCES machines(id),
    operator TEXT, supervisor TEXT,
    run_minutes INTEGER DEFAULT 0, downtime_minutes INTEGER DEFAULT 0, downtime_reason TEXT,
    status TEXT NOT NULL DEFAULT 'Draft',
    notes TEXT, approved_by TEXT, approved_at TEXT,
    created_by TEXT, created_at TEXT, journal_id INTEGER REFERENCES journal_entries(id)
);

CREATE TABLE IF NOT EXISTS production_lines (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id INTEGER NOT NULL REFERENCES production_orders(id),
    product_id INTEGER NOT NULL REFERENCES products(id),
    cups_per_carton INTEGER NOT NULL DEFAULT 1000,
    cartons_good REAL NOT NULL DEFAULT 0,
    cups_good REAL NOT NULL DEFAULT 0,
    cartons_waste REAL NOT NULL DEFAULT 0,
    cups_waste REAL NOT NULL DEFAULT 0,
    unit_cost_milli INTEGER NOT NULL DEFAULT 0,
    worker TEXT,
    brand_type TEXT NOT NULL DEFAULT 'factory',
    customer_id INTEGER REFERENCES customers(id),
    customer_brand_name TEXT,
    batch_no TEXT,
    quality_status TEXT,
    quality_notes TEXT
);

CREATE TABLE IF NOT EXISTS operations_daily_sheets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sheet_no TEXT, date TEXT NOT NULL, shift TEXT,
    supervisor_name TEXT, worker_name TEXT, attendance TEXT,
    start_time TEXT, end_time TEXT, normal_hours REAL DEFAULT 0,
    overtime_hours REAL DEFAULT 0, overtime_reason TEXT, overtime_approved INTEGER DEFAULT 0,
    product_id INTEGER REFERENCES products(id), customer_brand_name TEXT,
    cartons_produced REAL DEFAULT 0, cups_per_carton INTEGER DEFAULT 1000,
    total_cups REAL DEFAULT 0, waste_cartons REAL DEFAULT 0, waste_cups REAL DEFAULT 0,
    cups_quality TEXT, carton_quality TEXT, packing_quality TEXT, cleaning_quality TEXT,
    safety_notes TEXT, notes TEXT, worker_signature TEXT, supervisor_signature TEXT,
    status TEXT NOT NULL DEFAULT 'Draft',
    created_by TEXT, created_at TEXT, completed_by TEXT, completed_at TEXT,
    approved_by TEXT, approved_at TEXT
);

CREATE TABLE IF NOT EXISTS maintenance_daily_sheets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sheet_no TEXT, date TEXT NOT NULL, shift TEXT,
    maintenance_supervisor TEXT, machine_id INTEGER REFERENCES machines(id), area TEXT,
    fault_title TEXT, fault_description TEXT, severity TEXT,
    machine_stopped INTEGER DEFAULT 0,
    downtime_start TEXT, downtime_end TEXT, downtime_minutes INTEGER DEFAULT 0,
    repair_status TEXT DEFAULT 'open', repair_action TEXT, parts_changed TEXT,
    spare_parts_cost_milli INTEGER DEFAULT 0, labor_cost_milli INTEGER DEFAULT 0,
    other_cost_milli INTEGER DEFAULT 0, total_repair_cost_milli INTEGER DEFAULT 0,
    root_cause TEXT, preventive_action TEXT, next_followup_date TEXT,
    attachment_note TEXT, approval TEXT, close_date TEXT, notes TEXT,
    status TEXT NOT NULL DEFAULT 'Open',
    created_by TEXT, created_at TEXT, approved_by TEXT, approved_at TEXT,
    closed_by TEXT, closed_at TEXT
);

CREATE TABLE IF NOT EXISTS sales_invoices (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    inv_no TEXT, date TEXT NOT NULL, customer_id INTEGER NOT NULL REFERENCES customers(id),
    payment_type TEXT DEFAULT 'credit', vat_enabled INTEGER NOT NULL DEFAULT 1,
    net_milli INTEGER DEFAULT 0, vat_milli INTEGER DEFAULT 0,
    discount_milli INTEGER DEFAULT 0, total_milli INTEGER DEFAULT 0,
    discount_reason TEXT,
    cogs_milli INTEGER DEFAULT 0, paid_milli INTEGER DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'Draft',
    cashbank_id INTEGER REFERENCES cashbank_accounts(id), delivery TEXT, notes TEXT,
    created_by TEXT, created_at TEXT, journal_id INTEGER REFERENCES journal_entries(id),
    is_commercial INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS sales_invoice_lines (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    invoice_id INTEGER NOT NULL REFERENCES sales_invoices(id),
    product_id INTEGER NOT NULL REFERENCES products(id),
    cartons REAL NOT NULL DEFAULT 0,
    cups_per_carton INTEGER NOT NULL DEFAULT 1000,
    qty_cups REAL NOT NULL DEFAULT 0,
    unit_price_milli INTEGER NOT NULL DEFAULT 0,
    suggested_price_milli INTEGER NOT NULL DEFAULT 0,
    line_gross_milli INTEGER NOT NULL DEFAULT 0,
    line_discount_pct REAL NOT NULL DEFAULT 0,
    line_discount_milli INTEGER NOT NULL DEFAULT 0,
    discount_reason TEXT,
    line_net_milli INTEGER NOT NULL DEFAULT 0,
    vat_pct REAL NOT NULL DEFAULT 5.0,
    vat_milli INTEGER NOT NULL DEFAULT 0,
    customs_price_milli INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS customer_payments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    rec_no TEXT, date TEXT NOT NULL, customer_id INTEGER NOT NULL REFERENCES customers(id),
    amount_milli INTEGER NOT NULL DEFAULT 0, method TEXT DEFAULT 'cash',
    cashbank_id INTEGER REFERENCES cashbank_accounts(id), reference TEXT, notes TEXT,
    created_by TEXT, created_at TEXT, journal_id INTEGER REFERENCES journal_entries(id)
);

CREATE TABLE IF NOT EXISTS payment_allocations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    payment_id INTEGER NOT NULL REFERENCES customer_payments(id),
    invoice_id INTEGER NOT NULL REFERENCES sales_invoices(id),
    amount_milli INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS purchases (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pur_no TEXT, date TEXT NOT NULL, supplier_id INTEGER NOT NULL REFERENCES suppliers(id),
    supplier_invoice_no TEXT, vat_enabled INTEGER NOT NULL DEFAULT 1,
    net_milli INTEGER DEFAULT 0, vat_milli INTEGER DEFAULT 0,
    total_milli INTEGER DEFAULT 0, paid_milli INTEGER DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'Draft',
    notes TEXT, created_by TEXT, created_at TEXT, journal_id INTEGER REFERENCES journal_entries(id)
);

CREATE TABLE IF NOT EXISTS purchase_lines (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    purchase_id INTEGER NOT NULL REFERENCES purchases(id),
    item_id INTEGER NOT NULL REFERENCES inventory_items(id),
    qty REAL NOT NULL DEFAULT 0,
    unit_cost_milli INTEGER NOT NULL DEFAULT 0,
    line_net_milli INTEGER NOT NULL DEFAULT 0,
    vat_pct REAL NOT NULL DEFAULT 5.0,
    vat_milli INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS supplier_payments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pay_no TEXT, date TEXT NOT NULL, supplier_id INTEGER NOT NULL REFERENCES suppliers(id),
    amount_milli INTEGER NOT NULL DEFAULT 0, method TEXT DEFAULT 'cash',
    cashbank_id INTEGER REFERENCES cashbank_accounts(id), reference TEXT, notes TEXT,
    created_by TEXT, created_at TEXT, journal_id INTEGER REFERENCES journal_entries(id)
);

CREATE TABLE IF NOT EXISTS expenses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    exp_no TEXT, date TEXT NOT NULL, category TEXT, account_code TEXT REFERENCES accounts(code),
    amount_milli INTEGER NOT NULL DEFAULT 0, vat_milli INTEGER DEFAULT 0,
    method TEXT DEFAULT 'cash', paid_from_source TEXT, cashbank_id INTEGER REFERENCES cashbank_accounts(id), petty_id INTEGER REFERENCES petty_cash_accounts(id),
    vendor TEXT, reference TEXT, notes TEXT,
    attachment_required INTEGER DEFAULT 0, approval_status TEXT DEFAULT 'posted',
    created_by TEXT, created_at TEXT, journal_id INTEGER REFERENCES journal_entries(id),
    paid_by_employee_id INTEGER, custody_txn_id INTEGER,
    reimbursement_status TEXT DEFAULT 'none', reimbursement_date TEXT, reimbursed_by TEXT
);

CREATE TABLE IF NOT EXISTS cashbank_accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT, name TEXT NOT NULL, atype TEXT DEFAULT 'cash',
    account_code TEXT REFERENCES accounts(code),
    balance_milli INTEGER NOT NULL DEFAULT 0,
    active INTEGER NOT NULL DEFAULT 1
);
CREATE INDEX IF NOT EXISTS idx_cba_account ON cashbank_accounts(account_code);

CREATE TABLE IF NOT EXISTS renewals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL, category TEXT, authority TEXT,
    issue_date TEXT, expiry_date TEXT, cost_milli INTEGER DEFAULT 0,
    responsible TEXT, alert_days INTEGER DEFAULT 30,
    status TEXT DEFAULT 'active', notes TEXT
);

CREATE TABLE IF NOT EXISTS installments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL, source TEXT, original_milli INTEGER DEFAULT 0,
    currency TEXT DEFAULT 'OMR', start_date TEXT, due_date TEXT,
    paid_milli INTEGER DEFAULT 0, status TEXT DEFAULT 'open', notes TEXT
);

CREATE TABLE IF NOT EXISTS cheques (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    kind TEXT NOT NULL, cheque_no TEXT, bank TEXT, party TEXT,
    amount_milli INTEGER DEFAULT 0, due_date TEXT,
    status TEXT DEFAULT 'issued', link_type TEXT, link_id INTEGER, notes TEXT
);

CREATE TABLE IF NOT EXISTS audit_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ts TEXT NOT NULL, user_id INTEGER REFERENCES users(id), username TEXT,
    action TEXT, entity TEXT, entity_id INTEGER,
    old_value TEXT, new_value TEXT, reason TEXT
);

CREATE TABLE IF NOT EXISTS payroll_payments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id INTEGER NOT NULL REFERENCES payroll_runs(id),
    payment_date TEXT NOT NULL,
    source_type TEXT NOT NULL,
    source_id INTEGER,
    amount_milli INTEGER NOT NULL,
    journal_id INTEGER REFERENCES journal_entries(id),
    reversed INTEGER DEFAULT 0,
    reversal_journal_id INTEGER REFERENCES journal_entries(id),
    reversed_by TEXT, reversed_at TEXT,
    paid_by TEXT, paid_at TEXT
);

CREATE TABLE IF NOT EXISTS cashbank_transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ts TEXT NOT NULL, cashbank_id INTEGER NOT NULL REFERENCES cashbank_accounts(id),
    debit_milli INTEGER DEFAULT 0, credit_milli INTEGER DEFAULT 0,
    balance_milli INTEGER DEFAULT 0,
    method TEXT, ref_type TEXT, ref_id INTEGER, journal_id INTEGER REFERENCES journal_entries(id),
    notes TEXT, user_id INTEGER REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS document_status_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ts TEXT NOT NULL, entity_type TEXT, entity_id INTEGER,
    old_status TEXT, new_status TEXT, user_id INTEGER REFERENCES users(id), username TEXT, reason TEXT
);

CREATE TABLE IF NOT EXISTS document_voids (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ts TEXT NOT NULL, entity_type TEXT, entity_id INTEGER,
    reversal_journal_id INTEGER REFERENCES journal_entries(id), user_id INTEGER REFERENCES users(id), username TEXT, reason TEXT
);

CREATE TABLE IF NOT EXISTS credit_notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cn_no TEXT, date TEXT, customer_id INTEGER REFERENCES customers(id), invoice_id INTEGER REFERENCES sales_invoices(id),
    net_milli INTEGER DEFAULT 0, vat_milli INTEGER DEFAULT 0, total_milli INTEGER DEFAULT 0,
    cogs_milli INTEGER DEFAULT 0, reason TEXT, status TEXT DEFAULT 'Posted',
    journal_id INTEGER REFERENCES journal_entries(id), created_by TEXT, created_at TEXT
);

CREATE TABLE IF NOT EXISTS credit_note_lines (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cn_id INTEGER REFERENCES credit_notes(id), product_id INTEGER REFERENCES products(id), cartons REAL, cups_per_carton INTEGER,
    qty_cups REAL, unit_price_milli INTEGER, line_net_milli INTEGER,
    vat_pct REAL, vat_milli INTEGER
);

CREATE TABLE IF NOT EXISTS inventory_adjustments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    adj_no TEXT, date TEXT, item_id INTEGER REFERENCES inventory_items(id), direction TEXT, qty REAL,
    unit_cost_milli INTEGER DEFAULT 0, reason TEXT, status TEXT DEFAULT 'Draft',
    approved_by TEXT, approved_at TEXT, journal_id INTEGER REFERENCES journal_entries(id),
    created_by TEXT, created_at TEXT
);

CREATE TABLE IF NOT EXISTS petty_cash_accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT, name TEXT NOT NULL, responsible TEXT, role TEXT, employee_id INTEGER REFERENCES employees(id),
    spending_limit_milli INTEGER DEFAULT 0, requires_approval INTEGER DEFAULT 0,
    balance_milli INTEGER DEFAULT 0, status TEXT DEFAULT 'open',
    active INTEGER DEFAULT 1, notes TEXT, created_at TEXT
);

CREATE TABLE IF NOT EXISTS petty_cash_transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ts TEXT NOT NULL, petty_id INTEGER NOT NULL REFERENCES petty_cash_accounts(id), ttype TEXT,
    debit_milli INTEGER DEFAULT 0, credit_milli INTEGER DEFAULT 0, balance_milli INTEGER DEFAULT 0,
    category TEXT, account_code TEXT REFERENCES accounts(code), cashbank_id INTEGER REFERENCES cashbank_accounts(id), counter_petty_id INTEGER REFERENCES petty_cash_accounts(id),
    expense_id INTEGER REFERENCES expenses(id), attachment_status TEXT DEFAULT 'not_required',
    reference TEXT, notes TEXT, journal_id INTEGER REFERENCES journal_entries(id), user_id INTEGER REFERENCES users(id)
);

-- ============================================================
-- OPERATING ADVANCES (سلف العمليات) - Professional Module
-- ============================================================

CREATE TABLE IF NOT EXISTS operating_advances (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    advance_no TEXT UNIQUE NOT NULL,
    date TEXT NOT NULL,
    employee_id INTEGER NOT NULL,
    employee_name TEXT,
    department TEXT,
    purpose TEXT NOT NULL,
    description TEXT,
    amount_milli INTEGER NOT NULL,
    currency TEXT DEFAULT 'OMR',
    exchange_rate REAL DEFAULT 1.0,
    status TEXT NOT NULL DEFAULT 'draft',  -- draft, approved, disbursed, partially_spent, reconciled, closed, cancelled
    approval_status TEXT NOT NULL DEFAULT 'pending',  -- pending, approved, rejected
    approved_by INTEGER,
    approved_at TEXT,
    disbursed_by INTEGER,
    disbursed_at TEXT,
    source_account_code TEXT,  -- Cash/Bank account code
    advance_gl_account_code TEXT DEFAULT '1320',  -- Advances to Employees (Asset)
    default_expense_account_code TEXT,  -- Default expense account for quick entry
    expected_return_date TEXT,
    actual_return_date TEXT,
    total_spent_milli INTEGER DEFAULT 0,
    total_returned_milli INTEGER DEFAULT 0,
    balance_milli INTEGER DEFAULT 0,
    notes TEXT,
    created_by TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT,
    FOREIGN KEY (employee_id) REFERENCES employees(id)
);

CREATE TABLE IF NOT EXISTS advance_transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    advance_id INTEGER NOT NULL,
    ts TEXT NOT NULL DEFAULT (datetime('now')),
    ttype TEXT NOT NULL,  -- disburse, spend, receipt, return, adjustment, close
    amount_milli INTEGER NOT NULL,
    balance_after_milli INTEGER NOT NULL,
    account_code TEXT REFERENCES accounts(code),  -- GL account affected
    category TEXT,  -- expense category for spend/receipt
    vendor_name TEXT,
    invoice_no TEXT,
    invoice_date TEXT,
    reference TEXT,
    notes TEXT,
    attachment_ids TEXT,  -- comma-separated attachment IDs
    journal_id INTEGER REFERENCES journal_entries(id),
    created_by TEXT NOT NULL,
    FOREIGN KEY (advance_id) REFERENCES operating_advances(id)
);

CREATE TABLE IF NOT EXISTS advance_receipts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    advance_id INTEGER NOT NULL,
    transaction_id INTEGER,
    receipt_no TEXT,
    date TEXT NOT NULL,
    vendor_name TEXT,
    amount_milli INTEGER NOT NULL,
    vat_milli INTEGER DEFAULT 0,
    net_milli INTEGER NOT NULL,
    category TEXT,
    account_code TEXT,  -- Expense account
    description TEXT,
    attachment_ids TEXT,
    status TEXT DEFAULT 'submitted',  -- submitted, approved, rejected
    approved_by INTEGER,
    approved_at TEXT,
    journal_id INTEGER,
    created_by TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (advance_id) REFERENCES operating_advances(id),
    FOREIGN KEY (transaction_id) REFERENCES advance_transactions(id)
);

CREATE INDEX IF NOT EXISTS idx_oa_employee ON operating_advances(employee_id);
CREATE INDEX IF NOT EXISTS idx_oa_status ON operating_advances(status);
CREATE INDEX IF NOT EXISTS idx_oa_approval ON operating_advances(approval_status);
CREATE INDEX IF NOT EXISTS idx_oa_date ON operating_advances(date);
CREATE INDEX IF NOT EXISTS idx_at_advance ON advance_transactions(advance_id);
CREATE INDEX IF NOT EXISTS idx_at_type ON advance_transactions(ttype);
CREATE INDEX IF NOT EXISTS idx_ar_advance ON advance_receipts(advance_id);
CREATE INDEX IF NOT EXISTS idx_ar_status ON advance_receipts(status);
CREATE INDEX IF NOT EXISTS idx_art_txn ON advance_receipts(transaction_id);

CREATE TABLE IF NOT EXISTS daily_closings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT UNIQUE, snapshot_json TEXT, notes TEXT,
    status TEXT DEFAULT 'Draft', prepared_by TEXT, approved_by TEXT,
    created_at TEXT, approved_at TEXT
);

CREATE TABLE IF NOT EXISTS roles (
    id INTEGER PRIMARY KEY AUTOINCREMENT, code TEXT UNIQUE, name TEXT, description TEXT
);
CREATE TABLE IF NOT EXISTS permissions (
    id INTEGER PRIMARY KEY AUTOINCREMENT, code TEXT UNIQUE, name TEXT
);
CREATE TABLE IF NOT EXISTS role_permissions (role_code TEXT NOT NULL, perm_code TEXT NOT NULL, PRIMARY KEY (role_code, perm_code));
CREATE TABLE IF NOT EXISTS user_roles (user_id INTEGER NOT NULL REFERENCES users(id), role_code TEXT NOT NULL, PRIMARY KEY (user_id, role_code));

CREATE TABLE IF NOT EXISTS customer_price_lists (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    customer_id INTEGER REFERENCES customers(id), product_id INTEGER REFERENCES products(id), price_milli INTEGER, active INTEGER DEFAULT 1
);

CREATE TABLE IF NOT EXISTS attachments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_type TEXT, entity_id INTEGER, original_filename TEXT, stored_filename TEXT,
    mime_type TEXT, size_bytes INTEGER, uploaded_by TEXT, uploaded_at TEXT, notes TEXT
);

CREATE TABLE IF NOT EXISTS import_shipments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    shipment_no TEXT, supplier_id INTEGER REFERENCES suppliers(id), currency TEXT DEFAULT 'USD',
    exchange_rate REAL DEFAULT 1, status TEXT DEFAULT 'Ordered',
    notes TEXT, created_by TEXT, created_at TEXT
);
CREATE TABLE IF NOT EXISTS import_shipment_costs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    shipment_id INTEGER REFERENCES import_shipments(id), cost_type TEXT, amount_milli INTEGER DEFAULT 0
);
CREATE TABLE IF NOT EXISTS import_shipment_allocations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    shipment_id INTEGER REFERENCES import_shipments(id), item_id INTEGER REFERENCES inventory_items(id), qty REAL, allocated_cost_milli INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS worker_sheet_templates (
    id INTEGER PRIMARY KEY AUTOINCREMENT, code TEXT, name TEXT, kind TEXT, active INTEGER DEFAULT 1
);
CREATE TABLE IF NOT EXISTS worker_sheets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    template_id INTEGER REFERENCES worker_sheet_templates(id), worker TEXT, date TEXT, lang TEXT,
    status TEXT DEFAULT 'generated', notes TEXT, created_by TEXT, created_at TEXT
);

CREATE TABLE IF NOT EXISTS login_attempts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT, ts REAL, ok INTEGER
);

CREATE TABLE IF NOT EXISTS customer_aliases (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    customer_id INTEGER NOT NULL REFERENCES customers(id), alias TEXT NOT NULL, source TEXT, notes TEXT
);
CREATE TABLE IF NOT EXISTS product_price_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    product_id INTEGER NOT NULL REFERENCES products(id), price_milli INTEGER, effective_date TEXT,
    changed_by TEXT, note TEXT
);
CREATE TABLE IF NOT EXISTS supplier_price_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    supplier_id INTEGER REFERENCES suppliers(id), item_id INTEGER REFERENCES inventory_items(id), cost_milli INTEGER, effective_date TEXT,
    changed_by TEXT, note TEXT
);
CREATE TABLE IF NOT EXISTS product_families (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT, name_ar TEXT, name_en TEXT, category TEXT, active INTEGER DEFAULT 1, notes TEXT
);

CREATE TABLE IF NOT EXISTS payroll_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_no TEXT, period_start TEXT, period_end TEXT, status TEXT DEFAULT 'Draft',
    total_gross_milli INTEGER DEFAULT 0, total_deductions_milli INTEGER DEFAULT 0,
    total_net_milli INTEGER DEFAULT 0,
    created_by TEXT, created_at TEXT, processed_by TEXT, processed_at TEXT,
    approved_by TEXT, approved_at TEXT, paid_by TEXT, paid_at TEXT,
    accrual_journal_id INTEGER REFERENCES journal_entries(id), journal_id INTEGER REFERENCES journal_entries(id)
);

CREATE TABLE IF NOT EXISTS payroll_run_lines (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id INTEGER NOT NULL REFERENCES payroll_runs(id), employee_id INTEGER NOT NULL REFERENCES employees(id),
    basic_milli INTEGER DEFAULT 0, allowance_milli INTEGER DEFAULT 0,
    overtime_milli INTEGER DEFAULT 0, bonus_milli INTEGER DEFAULT 0,
    deduction_milli INTEGER DEFAULT 0, advance_deduction_milli INTEGER DEFAULT 0,
    insurance_deduction_milli INTEGER DEFAULT 0, tax_deduction_milli INTEGER DEFAULT 0,
    net_milli INTEGER DEFAULT 0, notes TEXT
);

CREATE TABLE IF NOT EXISTS employee_advances (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    employee_id INTEGER NOT NULL REFERENCES employees(id), amount_milli INTEGER NOT NULL,
    date TEXT NOT NULL, reason TEXT, status TEXT DEFAULT 'open',
    remaining_milli INTEGER NOT NULL DEFAULT 0,
    deduction_per_payroll_milli INTEGER DEFAULT 0,
    journal_id INTEGER REFERENCES journal_entries(id), source_type TEXT, source_id INTEGER,
    created_by TEXT, created_at TEXT
);

CREATE TABLE IF NOT EXISTS bank_statement_lines (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL REFERENCES cashbank_accounts(id), date TEXT NOT NULL,
    description TEXT, debit_milli INTEGER DEFAULT 0, credit_milli INTEGER DEFAULT 0,
    matched_to_type TEXT, matched_to_id INTEGER, notes TEXT
);

CREATE TABLE IF NOT EXISTS bank_reconciliation_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL REFERENCES cashbank_accounts(id), statement_date TEXT, statement_balance_milli INTEGER DEFAULT 0,
    reconciled_balance_milli INTEGER DEFAULT 0, status TEXT DEFAULT 'Open',
    notes TEXT, created_by TEXT, created_at TEXT, completed_at TEXT
);

CREATE TABLE IF NOT EXISTS quality_inspections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    production_line_id INTEGER REFERENCES production_lines(id), date TEXT, inspector TEXT,
    result TEXT, defect_type TEXT, defect_qty INTEGER DEFAULT 0,
    notes TEXT, status TEXT DEFAULT 'Pending'
);

CREATE TABLE IF NOT EXISTS multi_warehouse (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT, name TEXT NOT NULL, location TEXT, manager TEXT,
    active INTEGER NOT NULL DEFAULT 1, notes TEXT
);

CREATE TABLE IF NOT EXISTS stock_transfers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    transfer_no TEXT, from_warehouse_id INTEGER REFERENCES multi_warehouse(id), to_warehouse_id INTEGER REFERENCES multi_warehouse(id),
    item_id INTEGER REFERENCES inventory_items(id), qty REAL NOT NULL DEFAULT 0,
    status TEXT DEFAULT 'Draft', notes TEXT,
    created_by TEXT, created_at TEXT, completed_at TEXT
);

CREATE TABLE IF NOT EXISTS docflow_documents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    doc_no TEXT, doc_type TEXT, date TEXT, entity_type TEXT, entity_id INTEGER,
    from_party TEXT, to_party TEXT, subject TEXT, body TEXT,
    status TEXT DEFAULT 'Draft', notes TEXT,
    created_by TEXT, created_at TEXT
);

-- ============================================================
-- PRODUCTION SHIFT LINES (Live Production Tracking)
-- ============================================================
CREATE TABLE IF NOT EXISTS production_shift_lines (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sheet_id INTEGER NOT NULL REFERENCES operations_daily_sheets(id),
    product_id INTEGER NOT NULL REFERENCES products(id),
    customer_brand TEXT,
    cartons_produced REAL NOT NULL DEFAULT 0,
    cups_per_carton INTEGER NOT NULL DEFAULT 1000,
    waste_cartons REAL NOT NULL DEFAULT 0,
    unit_cost_milli INTEGER NOT NULL DEFAULT 0,
    material_cost_milli INTEGER NOT NULL DEFAULT 0,
    ts TEXT NOT NULL DEFAULT (datetime('now')),
    recorded_by TEXT
);
CREATE INDEX IF NOT EXISTS idx_psl_sheet ON production_shift_lines(sheet_id);
CREATE INDEX IF NOT EXISTS idx_psl_product ON production_shift_lines(product_id);
CREATE INDEX IF NOT EXISTS idx_psl_date ON production_shift_lines(ts);

-- ============================================================
-- E-INVOICES (FATOORA / ZATCA)
-- ============================================================
CREATE TABLE IF NOT EXISTS e_invoices (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    invoice_id INTEGER NOT NULL REFERENCES sales_invoices(id),
    xml_content TEXT,
    qr_code TEXT,
    status TEXT DEFAULT 'Draft',
    zatca_uuid TEXT,
    compliance_score REAL DEFAULT 0,
    submitted_at TEXT,
    acknowledged_at TEXT,
    rejection_reason TEXT,
    created_by TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    icv INTEGER,
    pih TEXT,
    invoice_hash TEXT,
    qr_content TEXT,
    signed_xml TEXT,
    signature_value TEXT,
    zatca_stage TEXT,
    zatca_environment TEXT,
    zatca_submitted_at TEXT,
    zatca_rejection_code TEXT
);

-- ============================================================
-- MULTI-BRANCH + OFFLINE SYNC
-- ============================================================
CREATE TABLE IF NOT EXISTS branches (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    code TEXT,
    address TEXT,
    is_head_office INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT DEFAULT (datetime('now'))
);
INSERT OR IGNORE INTO branches(id, name, code, is_head_office, is_active) VALUES (1, 'الفرع الرئيسي', 'HQ', 1, 1);

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
);

-- ============================================================
-- ZATCA PHASE 2 SETTINGS
-- ============================================================
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
);

-- ============================================================
-- QAYD XBRL FILINGS (Kuwait)
-- ============================================================
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
);

-- ============================================================
-- OCR SCAN HISTORY
-- ============================================================
CREATE TABLE IF NOT EXISTS ocr_scans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_name TEXT NOT NULL,
    file_path TEXT,
    file_type TEXT,
    extracted_text TEXT,
    parsed_data TEXT,
    invoice_no TEXT,
    invoice_date TEXT,
    vendor_name TEXT,
    total_amount_milli INTEGER DEFAULT 0,
    vat_amount_milli INTEGER DEFAULT 0,
    confidence REAL DEFAULT 0,
    status TEXT DEFAULT 'parsed',
    linked_invoice_id INTEGER,
    created_by TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);

-- ============================================================
-- INTEGRATIONS SETTINGS
-- ============================================================
CREATE TABLE IF NOT EXISTS integrations_settings (
    key TEXT PRIMARY KEY,
    value TEXT,
    updated_at TEXT
);

CREATE TABLE IF NOT EXISTS overtime_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    employee_id INTEGER NOT NULL REFERENCES employees(id),
    date TEXT NOT NULL,
    hours REAL NOT NULL DEFAULT 0,
    rate_multiplier REAL NOT NULL DEFAULT 1.5,
    reason TEXT,
    status TEXT NOT NULL DEFAULT 'Pending',
    approved INTEGER NOT NULL DEFAULT 0,
    approved_by TEXT,
    approved_at TEXT,
    notes TEXT,
    created_by TEXT,
    created_at TEXT
);

-- ============================================================
-- DATABASE PERFORMANCE INDEXES
-- ============================================================

-- Users
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);
CREATE INDEX IF NOT EXISTS idx_users_active ON users(active);

-- Customers & Suppliers
CREATE INDEX IF NOT EXISTS idx_customers_code ON customers(code);
CREATE INDEX IF NOT EXISTS idx_customers_name ON customers(name);
CREATE INDEX IF NOT EXISTS idx_customers_active ON customers(active);
CREATE INDEX IF NOT EXISTS idx_customers_balance ON customers(balance_milli);
CREATE INDEX IF NOT EXISTS idx_suppliers_code ON suppliers(code);
CREATE INDEX IF NOT EXISTS idx_suppliers_name ON suppliers(name);
CREATE INDEX IF NOT EXISTS idx_suppliers_active ON suppliers(active);

-- Products
CREATE INDEX IF NOT EXISTS idx_products_code ON products(code);
CREATE INDEX IF NOT EXISTS idx_products_active ON products(active);
CREATE INDEX IF NOT EXISTS idx_products_barcode ON products(barcode);

-- Inventory
CREATE INDEX IF NOT EXISTS idx_inventory_items_code ON inventory_items(code);
CREATE INDEX IF NOT EXISTS idx_inventory_items_kind ON inventory_items(kind);
CREATE INDEX IF NOT EXISTS idx_inventory_items_active ON inventory_items(active);
CREATE INDEX IF NOT EXISTS idx_inventory_items_product ON inventory_items(product_id);
CREATE INDEX IF NOT EXISTS idx_inventory_movements_item ON inventory_movements(item_id);
CREATE INDEX IF NOT EXISTS idx_inventory_movements_ts ON inventory_movements(ts);
CREATE INDEX IF NOT EXISTS idx_inventory_movements_type ON inventory_movements(mtype);
CREATE INDEX IF NOT EXISTS idx_im_item_ts ON inventory_movements(item_id, ts);

-- Sales Invoices
CREATE INDEX IF NOT EXISTS idx_si_customer ON sales_invoices(customer_id);
CREATE INDEX IF NOT EXISTS idx_si_date ON sales_invoices(date);
CREATE INDEX IF NOT EXISTS idx_si_status ON sales_invoices(status);
CREATE INDEX IF NOT EXISTS idx_si_inv_no ON sales_invoices(inv_no);
CREATE INDEX IF NOT EXISTS idx_si_customer_date ON sales_invoices(customer_id, date);
CREATE INDEX IF NOT EXISTS idx_si_lines_invoice ON sales_invoice_lines(invoice_id);
CREATE INDEX IF NOT EXISTS idx_si_lines_product ON sales_invoice_lines(product_id);

-- Customer Payments
CREATE INDEX IF NOT EXISTS idx_cp_customer ON customer_payments(customer_id);
CREATE INDEX IF NOT EXISTS idx_cp_date ON customer_payments(date);
CREATE INDEX IF NOT EXISTS idx_pa_payment ON payment_allocations(payment_id);
CREATE INDEX IF NOT EXISTS idx_pa_invoice ON payment_allocations(invoice_id);

-- Purchases
CREATE INDEX IF NOT EXISTS idx_pur_supplier ON purchases(supplier_id);
CREATE INDEX IF NOT EXISTS idx_pur_date ON purchases(date);
CREATE INDEX IF NOT EXISTS idx_pur_status ON purchases(status);
CREATE INDEX IF NOT EXISTS idx_pur_supplier_date ON purchases(supplier_id, date);
CREATE INDEX IF NOT EXISTS idx_pur_lines_purchase ON purchase_lines(purchase_id);
CREATE INDEX IF NOT EXISTS idx_pur_lines_item ON purchase_lines(item_id);

-- Supplier Payments
CREATE INDEX IF NOT EXISTS idx_sp_supplier ON supplier_payments(supplier_id);
CREATE INDEX IF NOT EXISTS idx_sp_date ON supplier_payments(date);

-- Journal Entries
CREATE INDEX IF NOT EXISTS idx_je_date ON journal_entries(date);
CREATE INDEX IF NOT EXISTS idx_je_entry_no ON journal_entries(entry_no);
CREATE INDEX IF NOT EXISTS idx_jel_entry ON journal_entry_lines(entry_id);
CREATE INDEX IF NOT EXISTS idx_jel_account ON journal_entry_lines(account_code);
CREATE INDEX IF NOT EXISTS idx_jel_entry_account ON journal_entry_lines(entry_id, account_code);

-- Production
CREATE INDEX IF NOT EXISTS idx_po_date ON production_orders(date);
CREATE INDEX IF NOT EXISTS idx_po_status ON production_orders(status);
CREATE INDEX IF NOT EXISTS idx_po_prod_no ON production_orders(prod_no);
CREATE INDEX IF NOT EXISTS idx_pl_order ON production_lines(order_id);
CREATE INDEX IF NOT EXISTS idx_pl_product ON production_lines(product_id);
CREATE INDEX IF NOT EXISTS idx_pl_product_order ON production_lines(product_id, order_id);

-- Operations
CREATE INDEX IF NOT EXISTS idx_ops_date ON operations_daily_sheets(date);
CREATE INDEX IF NOT EXISTS idx_ops_status ON operations_daily_sheets(status);

-- Maintenance
CREATE INDEX IF NOT EXISTS idx_mnt_date ON maintenance_daily_sheets(date);
CREATE INDEX IF NOT EXISTS idx_mnt_status ON maintenance_daily_sheets(status);
CREATE INDEX IF NOT EXISTS idx_mnt_machine ON maintenance_daily_sheets(machine_id);

-- Employees
CREATE INDEX IF NOT EXISTS idx_emp_code ON employees(code);
CREATE INDEX IF NOT EXISTS idx_emp_active ON employees(active);
CREATE INDEX IF NOT EXISTS idx_emp_job ON employees(job);

-- Petty Cash
CREATE INDEX IF NOT EXISTS idx_pc_active ON petty_cash_accounts(active);
CREATE INDEX IF NOT EXISTS idx_pct_petty ON petty_cash_transactions(petty_id);
CREATE INDEX IF NOT EXISTS idx_pct_ts ON petty_cash_transactions(ts);

-- Cash & Bank
CREATE INDEX IF NOT EXISTS idx_cb_code ON cashbank_accounts(code);
CREATE INDEX IF NOT EXISTS idx_cb_active ON cashbank_accounts(active);
CREATE INDEX IF NOT EXISTS idx_cbt_cashbank ON cashbank_transactions(cashbank_id);
CREATE INDEX IF NOT EXISTS idx_cbt_ts ON cashbank_transactions(ts);
CREATE INDEX IF NOT EXISTS idx_cbt_cashbank_ts ON cashbank_transactions(cashbank_id, ts);

-- Expenses
CREATE INDEX IF NOT EXISTS idx_exp_date ON expenses(date);
CREATE INDEX IF NOT EXISTS idx_exp_category ON expenses(category);
CREATE INDEX IF NOT EXISTS idx_exp_account ON expenses(account_code);

-- Cheques
CREATE INDEX IF NOT EXISTS idx_chq_status ON cheques(status);
CREATE INDEX IF NOT EXISTS idx_chq_due ON cheques(due_date);
CREATE INDEX IF NOT EXISTS idx_chq_kind ON cheques(kind);

-- Renewals & Installments
CREATE INDEX IF NOT EXISTS idx_renew_expiry ON renewals(expiry_date);
CREATE INDEX IF NOT EXISTS idx_renew_status ON renewals(status);
CREATE INDEX IF NOT EXISTS idx_inst_due ON installments(due_date);
CREATE INDEX IF NOT EXISTS idx_inst_status ON installments(status);

-- Audit Log
CREATE INDEX IF NOT EXISTS idx_audit_ts ON audit_logs(ts);
CREATE INDEX IF NOT EXISTS idx_audit_user ON audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_entity ON audit_logs(entity, entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_entity_ts ON audit_logs(entity, entity_id, ts);

-- E-Invoices
CREATE INDEX IF NOT EXISTS idx_einv_invoice ON e_invoices(invoice_id);
CREATE INDEX IF NOT EXISTS idx_einv_status ON e_invoices(status);

-- OCR Scans
CREATE INDEX IF NOT EXISTS idx_ocr_status ON ocr_scans(status);
CREATE INDEX IF NOT EXISTS idx_ocr_created ON ocr_scans(created_at);

-- Credit Notes
CREATE INDEX IF NOT EXISTS idx_cn_customer ON credit_notes(customer_id);
CREATE INDEX IF NOT EXISTS idx_cn_date ON credit_notes(date);

-- Payroll
CREATE INDEX IF NOT EXISTS idx_pr_run ON payroll_run_lines(run_id);
CREATE INDEX IF NOT EXISTS idx_pr_employee ON payroll_run_lines(employee_id);

-- Machines
CREATE INDEX IF NOT EXISTS idx_machines_status ON machines(status);
CREATE INDEX IF NOT EXISTS idx_machines_active ON machines(active);

-- Doc Sequences
CREATE INDEX IF NOT EXISTS idx_ds_type_year ON doc_sequences(doc_type, year);

-- Approval Workflow
CREATE TABLE IF NOT EXISTS approval_requests (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    request_type TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id INTEGER NOT NULL,
    entity_number TEXT NOT NULL,
    requested_by TEXT NOT NULL,
    requested_at TEXT NOT NULL,
    amount_milli INTEGER,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    approved_by TEXT,
    approved_at TEXT,
    rejection_reason TEXT,
    priority TEXT NOT NULL DEFAULT 'normal'
);
CREATE INDEX IF NOT EXISTS idx_ar_status ON approval_requests(status);
CREATE INDEX IF NOT EXISTS idx_ar_type ON approval_requests(request_type);
CREATE INDEX IF NOT EXISTS idx_ar_entity ON approval_requests(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_ar_priority ON approval_requests(priority);

-- Budget Planning
CREATE TABLE IF NOT EXISTS budgets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    budget_no TEXT NOT NULL,
    name TEXT NOT NULL,
    department TEXT,
    year INTEGER NOT NULL,
    period TEXT NOT NULL DEFAULT 'annual',
    status TEXT NOT NULL DEFAULT 'Draft',
    total_planned_milli INTEGER NOT NULL DEFAULT 0,
    total_actual_milli INTEGER NOT NULL DEFAULT 0,
    notes TEXT,
    created_by TEXT,
    created_at TEXT,
    approved_by TEXT,
    approved_at TEXT
);
CREATE INDEX IF NOT EXISTS idx_budget_year ON budgets(year);
CREATE INDEX IF NOT EXISTS idx_budget_status ON budgets(status);
CREATE INDEX IF NOT EXISTS idx_budget_dept ON budgets(department);

CREATE TABLE IF NOT EXISTS budget_lines (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    budget_id INTEGER NOT NULL,
    category TEXT NOT NULL,
    account_code TEXT REFERENCES accounts(code),
    description TEXT,
    planned_milli INTEGER NOT NULL DEFAULT 0,
    actual_milli INTEGER NOT NULL DEFAULT 0,
    notes TEXT,
    FOREIGN KEY (budget_id) REFERENCES budgets(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_bl_budget ON budget_lines(budget_id);
CREATE INDEX IF NOT EXISTS idx_bl_category ON budget_lines(category);

-- Fixed Assets
CREATE TABLE IF NOT EXISTS fixed_assets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    asset_no TEXT NOT NULL,
    name TEXT NOT NULL,
    category TEXT NOT NULL,
    description TEXT,
    serial_number TEXT,
    purchase_date TEXT,
    purchase_cost_milli INTEGER NOT NULL DEFAULT 0,
    current_value_milli INTEGER NOT NULL DEFAULT 0,
    depreciation_method TEXT NOT NULL DEFAULT 'straight_line',
    depreciation_rate_pct REAL NOT NULL DEFAULT 0,
    useful_life_months INTEGER NOT NULL DEFAULT 60,
    accumulated_depreciation_milli INTEGER NOT NULL DEFAULT 0,
    location TEXT,
    department TEXT,
    assigned_to TEXT,
    supplier TEXT,
    warranty_expiry TEXT,
    last_maintenance TEXT,
    next_maintenance TEXT,
    condition_status TEXT NOT NULL DEFAULT 'good',
    status TEXT NOT NULL DEFAULT 'active',
    notes TEXT,
    active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT
);
CREATE INDEX IF NOT EXISTS idx_fa_category ON fixed_assets(category);
CREATE INDEX IF NOT EXISTS idx_fa_status ON fixed_assets(status);
CREATE INDEX IF NOT EXISTS idx_fa_location ON fixed_assets(location);
CREATE INDEX IF NOT EXISTS idx_fa_next_maint ON fixed_assets(next_maintenance);
CREATE INDEX IF NOT EXISTS idx_fa_warranty ON fixed_assets(warranty_expiry);

CREATE TABLE IF NOT EXISTS asset_maintenance_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    asset_id INTEGER NOT NULL,
    maintenance_type TEXT NOT NULL,
    date TEXT NOT NULL,
    description TEXT NOT NULL,
    cost_milli INTEGER NOT NULL DEFAULT 0,
    performed_by TEXT,
    next_due TEXT,
    notes TEXT,
    FOREIGN KEY (asset_id) REFERENCES fixed_assets(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_aml_asset ON asset_maintenance_logs(asset_id);
CREATE INDEX IF NOT EXISTS idx_aml_date ON asset_maintenance_logs(date);

-- Notifications
CREATE TABLE IF NOT EXISTS notifications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER,
    notification_type TEXT NOT NULL,
    title TEXT NOT NULL,
    message TEXT NOT NULL,
    entity_type TEXT,
    entity_id INTEGER,
    severity TEXT NOT NULL DEFAULT 'info',
    read_status TEXT NOT NULL DEFAULT 'unread',
    action_url TEXT,
    created_at TEXT NOT NULL,
    read_at TEXT,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_notif_user ON notifications(user_id);
CREATE INDEX IF NOT EXISTS idx_notif_read ON notifications(read_status);
CREATE INDEX IF NOT EXISTS idx_notif_type ON notifications(notification_type);
CREATE INDEX IF NOT EXISTS idx_notif_severity ON notifications(severity);
CREATE INDEX IF NOT EXISTS idx_notif_created ON notifications(created_at);

-- Critical missing indexes for query performance
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

-- Missing FK-column indexes (added during DB review)
CREATE INDEX IF NOT EXISTS idx_at_account ON advance_transactions(account_code);
CREATE INDEX IF NOT EXISTS idx_at_journal ON advance_transactions(journal_id);
CREATE INDEX IF NOT EXISTS idx_brs_account ON bank_reconciliation_sessions(account_id);
CREATE INDEX IF NOT EXISTS idx_bsl_account ON bank_statement_lines(account_id);
CREATE INDEX IF NOT EXISTS idx_bl_account ON budget_lines(account_code);
CREATE INDEX IF NOT EXISTS idx_cbt_journal ON cashbank_transactions(journal_id);
CREATE INDEX IF NOT EXISTS idx_cbt_user ON cashbank_transactions(user_id);
CREATE INDEX IF NOT EXISTS idx_cn_invoice ON credit_notes(invoice_id);
CREATE INDEX IF NOT EXISTS idx_cn_journal ON credit_notes(journal_id);
CREATE INDEX IF NOT EXISTS idx_ca_customer ON customer_aliases(customer_id);
CREATE INDEX IF NOT EXISTS idx_cp_cashbank ON customer_payments(cashbank_id);
CREATE INDEX IF NOT EXISTS idx_cp_journal ON customer_payments(journal_id);
CREATE INDEX IF NOT EXISTS idx_cpl_customer ON customer_price_lists(customer_id);
CREATE INDEX IF NOT EXISTS idx_cpl_product ON customer_price_lists(product_id);
CREATE INDEX IF NOT EXISTS idx_dsh_user ON document_status_history(user_id);
CREATE INDEX IF NOT EXISTS idx_dv_journal ON document_voids(reversal_journal_id);
CREATE INDEX IF NOT EXISTS idx_dv_user ON document_voids(user_id);
CREATE INDEX IF NOT EXISTS idx_ea_journal ON employee_advances(journal_id);
CREATE INDEX IF NOT EXISTS idx_exp_cashbank ON expenses(cashbank_id);
CREATE INDEX IF NOT EXISTS idx_exp_journal ON expenses(journal_id);
CREATE INDEX IF NOT EXISTS idx_ia_journal ON inventory_adjustments(journal_id);
CREATE INDEX IF NOT EXISTS idx_im_user ON inventory_movements(user_id);
CREATE INDEX IF NOT EXISTS idx_ppay_run ON payroll_payments(run_id);
CREATE INDEX IF NOT EXISTS idx_ppay_journal ON payroll_payments(journal_id);
CREATE INDEX IF NOT EXISTS idx_ppay_reversal ON payroll_payments(reversal_journal_id);
CREATE INDEX IF NOT EXISTS idx_pr_accrual ON payroll_runs(accrual_journal_id);
CREATE INDEX IF NOT EXISTS idx_pr_journal ON payroll_runs(journal_id);
CREATE INDEX IF NOT EXISTS idx_pca_employee ON petty_cash_accounts(employee_id);
CREATE INDEX IF NOT EXISTS idx_pct_account ON petty_cash_transactions(account_code);
CREATE INDEX IF NOT EXISTS idx_pct_cashbank ON petty_cash_transactions(cashbank_id);
CREATE INDEX IF NOT EXISTS idx_pct_counter ON petty_cash_transactions(counter_petty_id);
CREATE INDEX IF NOT EXISTS idx_pct_expense ON petty_cash_transactions(expense_id);
CREATE INDEX IF NOT EXISTS idx_pct_journal ON petty_cash_transactions(journal_id);
CREATE INDEX IF NOT EXISTS idx_pct_user ON petty_cash_transactions(user_id);
CREATE INDEX IF NOT EXISTS idx_pph_product ON product_price_history(product_id);
CREATE INDEX IF NOT EXISTS idx_pp_customer ON product_prices(customer_id);
CREATE INDEX IF NOT EXISTS idx_pl_customer ON production_lines(customer_id);
CREATE INDEX IF NOT EXISTS idx_po_machine ON production_orders(machine_id);
CREATE INDEX IF NOT EXISTS idx_po_journal ON production_orders(journal_id);
CREATE INDEX IF NOT EXISTS idx_pur_journal ON purchases(journal_id);
CREATE INDEX IF NOT EXISTS idx_si_cashbank ON sales_invoices(cashbank_id);
CREATE INDEX IF NOT EXISTS idx_si_journal ON sales_invoices(journal_id);
CREATE INDEX IF NOT EXISTS idx_st_item ON stock_transfers(item_id);
CREATE INDEX IF NOT EXISTS idx_sp_cashbank ON supplier_payments(cashbank_id);
CREATE INDEX IF NOT EXISTS idx_sp_journal ON supplier_payments(journal_id);
CREATE INDEX IF NOT EXISTS idx_sph_item ON supplier_price_history(item_id);
CREATE INDEX IF NOT EXISTS idx_sph_supplier ON supplier_price_history(supplier_id);
CREATE INDEX IF NOT EXISTS idx_ws_template ON worker_sheets(template_id);

-- Indexes for tables that had none
CREATE INDEX IF NOT EXISTS idx_att_entity ON attachments(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_mw_code ON multi_warehouse(code);
CREATE INDEX IF NOT EXISTS idx_pf_code ON product_families(code);
CREATE INDEX IF NOT EXISTS idx_wst_name ON worker_sheet_templates(name);

-- Professional quotations (قوائم أسعار / عروض) with free-form client details
-- and per-line cup specs, and commercial (non-tax) factory invoices
CREATE TABLE IF NOT EXISTS quotations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    quote_no TEXT,
    date TEXT NOT NULL,
    customer_id INTEGER,
    client_name TEXT,
    client_contact TEXT,
    client_phone TEXT,
    client_email TEXT,
    client_address TEXT,
    title TEXT,
    notes TEXT,
    terms TEXT,
    validity_days INTEGER DEFAULT 7,
    net_milli INTEGER DEFAULT 0,
    discount_milli INTEGER DEFAULT 0,
    total_milli INTEGER DEFAULT 0,
    currency TEXT DEFAULT 'OMR',
    status TEXT DEFAULT 'Draft',
    created_by TEXT,
    created_at TEXT,
    updated_at TEXT
);

CREATE TABLE IF NOT EXISTS quotation_lines (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    quote_id INTEGER NOT NULL REFERENCES quotations(id) ON DELETE CASCADE,
    product_id INTEGER,
    item_name TEXT,
    cup_size TEXT,
    cups_per_carton INTEGER DEFAULT 1000,
    cartons REAL DEFAULT 0,
    unit_price_milli INTEGER DEFAULT 0,
    line_total_milli INTEGER DEFAULT 0,
    notes TEXT
);

CREATE INDEX IF NOT EXISTS idx_quotation_lines_quote ON quotation_lines(quote_id);
CREATE INDEX IF NOT EXISTS idx_quotations_date ON quotations(date);
