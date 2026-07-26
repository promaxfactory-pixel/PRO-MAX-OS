# PRO MAX OS API Documentation

## Overview

PRO MAX OS is a Tauri 2 desktop application with a Rust backend and React/TypeScript frontend. The Rust backend exposes 288+ Tauri commands organized by domain module. All commands communicate via Tauri IPC; the frontend uses typed TypeScript wrappers auto-generated from Rust structs.

### Database

- **Engine:** SQLite with WAL mode, 5-second busy timeout
- **Tables:** 74+ (54 in base schema.sql + 20 added via migrations)
- **Monetary precision:** All values in milli (1/1000 OMR), stored as INTEGER
- **Encryption:** AES-256-GCM for sensitive fields at rest

### Authentication

- Argon2id password hashing
- JWT token (HMAC-SHA256), stored in Rust login_state for app lifetime
- Token restoration on app restart
- CSP hardened: `default-src 'self'; script-src 'self'; frame-ancestors 'none'`
- RBAC role-based access control (Admin, Manager, User, Viewer)
- Full audit logging for all mutation commands

---

## Authentication (auth)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `login` | `username: String, password: String` | `Result<LoginResult, String>` | Authenticates user with Argon2id |
| `get_current_user` | `user_id: i64` | `Result<User, String>` | Returns current user record |
| `change_password` | `user_id: i64, old: String, new: String` | `Result<String, String>` | Changes password with verification |
| `validate_token` | `token: String` | `Result<User, String>` | Restores session from JWT |

---

## Dashboard (dashboard)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `get_dashboard_stats` | (none) | `Result<DashboardStats, String>` | KPIs: sales, invoices, stock, production |
| `get_daily_brief` | (none) | `Result<DailyBrief, String>` | Daily operational summary |

---

## Customers (customers)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_customers` | (none) | `Result<Vec<Customer>, String>` | All active customers |
| `get_customer` | `id: i64` | `Result<Customer, String>` | Customer by ID |
| `create_customer` | `input: CreateCustomerInput` | `Result<i64, String>` | New customer, returns ID |
| `update_customer` | `id: i64, input: UpdateCustomerInput` | `Result<String, String>` | Update customer |
| `delete_customer` | `id: i64` | `Result<String, String>` | Soft-delete (active=0) |
| `get_customer_statement` | `customer_id: i64, from_date: Option<String>, to_date: Option<String>` | `Result<CustomerStatementData, String>` | Full transaction history |

---

## Products (products)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_products` | (none) | `Result<Vec<Product>, String>` | All active products |
| `list_products_for_select` | (none) | `Result<Vec<ProductSelectItem>, String>` | Lightweight list for dropdowns |
| `get_product` | `id: i64` | `Result<Product, String>` | Product by ID |
| `create_product` | `input: CreateProductInput` | `Result<i64, String>` | New product, returns ID |
| `update_product` | `id: i64, input: CreateProductInput` | `Result<String, String>` | Update product |
| `delete_product` | `id: i64` | `Result<String, String>` | Soft-delete |

---

## Inventory (inventory)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_inventory_items` | (none) | `Result<Vec<InventoryItem>, String>` | All items with current stock |
| `get_inventory_item` | `id: i64` | `Result<InventoryItem, String>` | Item by ID |
| `create_inventory_item` | `input: CreateItemInput` | `Result<InventoryItem, String>` | New item |
| `update_inventory_item` | `id: i64, input: UpdateItemInput` | `Result<InventoryItem, String>` | Update item |
| `adjust_stock` | `input: AdjustStockInput` | `Result<InventoryItem, String>` | Manual stock adjustment |
| `get_inventory_movements` | `item_id: i64` | `Result<Vec<InventoryMovement>, String>` | Movement history |

---

## Invoices (invoices)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_invoices` | (none) | `Result<Vec<SalesInvoice>, String>` | All invoices |
| `get_invoice` | `id: i64` | `Result<SalesInvoice, String>` | Invoice with lines and payments |
| `get_invoice_lines` | `invoice_id: i64` | `Result<Vec<InvoiceLine>, String>` | Line items |
| `create_invoice` | `input: CreateInvoiceInput` | `Result<i64, String>` | New invoice (Draft) |
| `post_invoice` | `id: i64` | `Result<String, String>` | Post Draft to Posted |
| `void_invoice` | `id: i64, reason: Option<String>` | `Result<String, String>` | Void a posted invoice |
| `duplicate_invoice` | `id: i64` | `Result<i64, String>` | Clone invoice as Draft |
| `update_invoice` | `id: i64, notes: Option<String>` | `Result<String, String>` | Update notes only |
| `get_invoice_for_print` | `invoice_id: i64` | `Result<InvoicePrintData, String>` | A4 print format |
| `get_invoice_for_print_customs` | `invoice_id: i64` | `Result<InvoicePrintData, String>` | Customs format with HS codes |
| `get_receipt_for_print` | `payment_id: i64` | `Result<ReceiptPrintData, String>` | Thermal receipt format |
| `get_delivery_note_for_print` | `invoice_id: i64` | `Result<DeliveryNoteData, String>` | Delivery note format |
| `get_credit_note_for_print` | `credit_note_id: i64` | `Result<CreditNotePrintData, String>` | Credit note format |

---

## Suppliers (suppliers)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_suppliers` | (none) | `Result<Vec<Supplier>, String>` | All suppliers |
| `get_supplier` | `id: i64` | `Result<Supplier, String>` | Supplier by ID |
| `create_supplier` | `input: CreateSupplierInput` | `Result<i64, String>` | New supplier |
| `update_supplier` | `id: i64, input: UpdateSupplierInput` | `Result<String, String>` | Update supplier |
| `delete_supplier` | `id: i64` | `Result<String, String>` | Soft-delete |
| `get_supplier_statement` | `supplier_id: i64, from_date: Option<String>, to_date: Option<String>` | `Result<SupplierStatementData, String>` | Transaction history |

---

## Purchases (purchases)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_purchases` | (none) | `Result<Vec<Purchase>, String>` | All purchase orders |
| `get_purchase` | `id: i64` | `Result<Purchase, String>` | Purchase by ID |
| `get_purchase_lines` | `purchase_id: i64` | `Result<Vec<PurchaseLine>, String>` | Purchase line items |
| `create_purchase` | `input: CreatePurchaseInput` | `Result<i64, String>` | New PO |
| `list_suppliers_for_select` | (none) | `Result<Vec<JsonValue>, String>` | Supplier dropdown |
| `create_supplier_payment` | `input: CreateSupplierPaymentInput` | `Result<i64, String>` | Payment to supplier |

---

## Expenses (expenses)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_expenses` | (filter params) | `Result<Vec<Expense>, String>` | Expense list |
| `create_expense` | `input: CreateExpenseInput` | `Result<i64, String>` | New expense |
| `reimburse_expense` | `id: i64` | `Result<String, String>` | Mark as reimbursed |
| `approve_expense` | `id: i64` | `Result<String, String>` | Approve expense |

---

## Accounting (accounting)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_accounts` | (none) | `Result<Vec<Account>, String>` | Chart of accounts |
| `get_account` | `code: String` | `Result<Account, String>` | Account by code |
| `create_account` | `input: CreateAccountInput` | `Result<String, String>` | New account |
| `list_journal_entries` | (none) | `Result<Vec<JournalEntry>, String>` | All GL entries |
| `get_journal_entry_lines` | `entry_id: i64` | `Result<Vec<JournalLine>, String>` | Entry lines |
| `create_journal_entry` | `input: CreateJournalEntryInput` | `Result<i64, String>` | New GL entry |
| `get_trial_balance` | (none) | `Result<Vec<TrialBalanceRow>, String>` | Trial balance |
| `get_balance_sheet` | (none) | `Result<Vec<BalanceSheetRow>, String>` | Balance sheet |
| `get_income_statement` | (none) | `Result<Vec<IncomeStatementRow>, String>` | P&L statement |

---

## Custody (custody)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_custody_accounts` | (none) | `Result<Vec<CustodyAccount>, String>` | All custody accounts |
| `get_custody_account` | `id: i64` | `Result<CustodyAccount, String>` | Account by ID |
| `create_custody_fund` | `input: CreateFundInput` | `Result<CustodyAccount, String>` | New fund |
| `create_custody_spend` | `input: CreateSpendInput` | `Result<CustodyAccount, String>` | Record spend |
| `create_custody_transfer` | `input: CreateTransferInput` | `Result<Vec<CustodyAccount>, String>` | Transfer between accounts |
| `get_custody_statement` | `petty_id: i64` | `Result<Vec<CustodyTransaction>, String>` | Full statement |

---

## HR (hr)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_employees` | (none) | `Result<Vec<Employee>, String>` | All employees |
| `get_employee` | `id: i64` | `Result<Employee, String>` | Employee by ID |
| `create_employee` | `input: CreateEmployeeInput` | `Result<i64, String>` | New employee |
| `update_employee` | `id: i64, input: UpdateEmployeeInput` | `Result<String, String>` | Update employee |
| `delete_employee` | `id: i64` | `Result<String, String>` | Soft-delete |
| `list_employees_for_production` | (none) | `Result<Vec<EmployeeListItem>, String>` | Light list for production floor |

---

## Payroll (payroll)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_payroll_runs` | (none) | `Result<Vec<PayrollRun>, String>` | All payroll runs |
| `create_payroll_run` | `input: CreatePayrollRunInput` | `Result<i64, String>` | New payroll run |
| `list_employee_advances` | `employee_id: i64` | `Result<Vec<EmployeeAdvance>, String>` | Employee advances |
| `create_employee_advance` | `input: CreateAdvanceInput` | `Result<i64, String>` | Record advance |

---

## Production (production)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_production_orders` | (none) | `Result<Vec<ProductionOrder>, String>` | All orders |
| `get_production_order` | `id: i64` | `Result<ProductionOrder, String>` | Order by ID |
| `create_production_order` | `input: CreateOrderInput` | `Result<i64, String>` | New order |
| `update_production_order` | `id: i64, input: UpdateOrderInput` | `Result<String, String>` | Update order |
| `approve_production_order` | `id: i64, approved_by: String` | `Result<String, String>` | Approve order |
| `get_production_lines` | `order_id: i64` | `Result<Vec<ProductionLine>, String>` | Lines for order |
| `add_production_line` | `input: AddLineInput` | `Result<i64, String>` | Add line to order |

---

## Production Shift (production_shift)

Shift-based floor production tracking:

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `get_shift_sheet` | `sheet_id: i64` | `Result<ShiftSheet, String>` | Full shift sheet |
| `record_production` | `input: RecordProductionInput` | `Result<String, String>` | Record worker output |
| `get_shift_lines` | `sheet_id: i64` | `Result<Vec<ShiftLine>, String>` | Production lines |
| `complete_shift` | `sheet_id: i64` | `Result<String, String>` | Close shift, calc totals |
| `update_production_line` | `line_id: i64, input: UpdateLineInput` | `Result<String, String>` | Update a line |
| `delete_production_line` | `line_id: i64` | `Result<String, String>` | Remove line |
| `get_live_dashboard` | (none) | `Result<LiveDashboard, String>` | Real-time floor data |
| `print_shift_report_thermal` | `sheet_id: i64` | `Result<String, String>` | Thermal print output |
| `get_worker_daily_report` | `employee_id: i64, date: String` | `Result<WorkerReport, String>` | Worker productivity |
| `record_shift_inventory_snapshot` | `input: SnapshotInput` | `Result<i64, String>` | Record shift stock count |
| `get_shift_inventory_snapshots` | `sheet_id: i64` | `Result<Vec<Snapshot>, String>` | Inventory snapshots |

---

## Maintenance (maintenance)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_maintenance_sheets` | (none) | `Result<Vec<MaintenanceSheet>, String>` | All sheets |
| `get_maintenance_sheet` | `id: i64` | `Result<MaintenanceSheet, String>` | Sheet by ID |
| `create_maintenance_sheet` | `input: CreateMaintenanceSheetInput` | `Result<MaintenanceSheet, String>` | New sheet |

---

## Operations (operations)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_operations_sheets` | (none) | `Result<Vec<OperationsSheet>, String>` | All operation sheets |
| `get_operations_sheet` | `id: i64` | `Result<OperationsSheet, String>` | Sheet by ID |
| `create_operations_sheet` | `input: CreateOperationsSheetInput` | `Result<OperationsSheet, String>` | New sheet |

---

## Machine Temperature (machines)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_machines` | (none) | `Result<Vec<Machine>, String>` | All machines |
| `get_machine` | `id: i64` | `Result<Machine, String>` | Machine by ID |
| `create_machine` | `input: CreateMachineInput` | `Result<i64, String>` | New machine |
| `update_machine` | `id: i64, input: UpdateMachineInput` | `Result<String, String>` | Update machine |
| `record_temperature` | `machine_id: i64, temperature: f64` | `Result<String, String>` | Log temperature |
| `get_machine_temperatures` | `machine_id: i64, hours: Option<i64>` | `Result<Vec<TemperatureLog>, String>` | Temp history |
| `get_live_machine_temps` | (none) | `Result<Vec<LiveMachineTemp>, String>` | Real-time temps |

---

## Quality (quality)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_quality_inspections` | (none) | `Result<Vec<QualityInspection>, String>` | All inspections |
| `create_quality_inspection` | `input: CreateQualityInspectionInput` | `Result<i64, String>` | New inspection |

---

## BOM (bom) - Bill of Materials

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_boms` | (none) | `Result<Vec<BomEntry>, String>` | All BOM entries |
| `create_bom` | `input: CreateBomInput` | `Result<i64, String>` | New BOM entry |

---

## Stock Transfers (stock_transfers)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_stock_transfers` | (none) | `Result<Vec<StockTransfer>, String>` | All transfers |
| `create_stock_transfer` | `input: CreateStockTransferInput` | `Result<i64, String>` | New transfer |

---

## Alerts (alerts)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `get_all_alerts` | (none) | `Result<Vec<Alert>, String>` | Pending alerts |

---

## Licensing (licensing)

4-tier B2B commercial license system (Free, Basic, Professional, Enterprise) with 22 feature flags:

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `check_license` | (none) | `LicenseStatus` | Current license status |
| `activate_license` | `license_key: String` | `LicenseStatus` | Activate a key |
| `get_license_info` | (none) | `LicenseStatus` | Full license details |
| `deactivate_license` | (none) | `LicenseStatus` | Deactivate license |
| `verify_developer_pin` | `pin: String` | `bool` | Developer PIN verification |
| `generate_license_key` | `pin, customer_name, license_type, expires_days, max_users, features` | `Result<String, String>` | Generate key (dev only) |
| `generate_tier_license` | `pin, customer_name, tier, expires_days, max_users, target_hardware_id` | `Result<String, String>` | Tier-specific key |
| `get_tier_features` | `tier: String` | `Vec<String>` | Feature flags for tier |
| `list_tiers` | (none) | `Vec<JsonValue>` | All available tiers |

---

## OCR (ocr)

AI-powered receipt/invoice scanning:

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `ocr_extract_from_file` | `path: String` | `Result<OcrResult, String>` | Extract all text |
| `ocr_parse_invoice` | `path: String` | `Result<ExtractionResult, String>` | Parse invoice data |
| `ocr_enhance_with_ai` | `path: String, raw_text: String` | `Result<ExtractionResult, String>` | AI-enhanced extraction |
| `ocr_get_suggestions` | `result: ExtractionResult` | `Result<Vec<Suggestion>, String>` | Field suggestions |
| `ocr_create_invoice` | `data: JsonValue` | `Result<String, String>` | Auto-create invoice from OCR |
| `ocr_add_supplier` | `data: JsonValue` | `Result<String, String>` | Auto-create supplier |
| `ocr_register_expense` | `data: JsonValue` | `Result<String, String>` | Auto-record expense |
| `ocr_update_prices` | `data: JsonValue` | `Result<String, String>` | Update product prices |
| `ocr_detect_language` | `text: String` | `Result<String, String>` | Language detection |
| `ocr_get_history` | (none) | `Result<Vec<OcrScan>, String>` | Past scans |
| `ocr_save_scan` | `input: JsonValue` | `Result<i64, String>` | Save scan to DB |

---

## AI (ai)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `ai_sales_forecast` | `days: i32` | `Result<SalesForecast, String>` | N-day sales prediction |
| `ai_customer_risk` | (none) | `Result<Vec<CustomerRisk>, String>` | High-risk customers |
| `ai_production_analysis` | (none) | `Result<ProductionAnalysis, String>` | Efficiency analysis |
| `ai_cost_analysis` | (none) | `Result<CostAnalysis, String>` | Cost breakdown |
| `ai_dashboard_insights` | (none) | `Result<Vec<Insight>, String>` | NL business summary |
| `ai_inventory_optimization` | (none) | `Result<InventoryOptimization, String>` | Stock optimization |
| `ai_anomaly_detection` | (none) | `Result<Vec<Anomaly>, String>` | Unusual pattern detection |
| `ai_generate_report` | `report_type: String` | `Result<AiReport, String>` | AI-generated report |

---

## E-Invoice (einvoice)

ZATCA/FATOORA compliance for Saudi Arabian e-invoicing:

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `einvoice_generate` | `invoice_id: i64` | `Result<String, String>` | Generate encrypted XML |
| `einvoice_validate` | `invoice_id: i64` | `Result<String, String>` | Validate XML against ZATCA schema |
| `einvoice_get_status` | `invoice_id: i64` | `Result<String, String>` | Submission status |
| `einvoice_list` | (none) | `Result<Vec<EInvoice>, String>` | All e-invoices |
| `einvoice_mark_submitted` | `invoice_id: i64` | `Result<String, String>` | Mark as submitted |
| `einvoice_summary_report` | (none) | `Result<String, String>` | ZATCA summary |
| `einvoice_cancel` | `invoice_id: i64, reason: String` | `Result<String, String>` | Cancel e-invoice |
| `einvoice_submit` | `invoice_id: i64` | `Result<String, String>` | Submit to ZATCA portal |
| `einvoice_add_to_queue` | `invoice_id: i64` | `Result<String, String>` | Add to processing queue |
| `einvoice_process_queue` | (none) | `Result<String, String>` | Process pending queue |
| `einvoice_get_dashboard` | (none) | `Result<EInvoiceDashboard, String>` | Queue dashboard |
| `einvoice_get_settings` | (none) | `Result<EInvoiceSettings, String>` | E-invoice config |
| `einvoice_save_settings` | `input: JsonValue` | `Result<String, String>` | Save config |
| `einvoice_get_queue` | (none) | `Result<Vec<QueueItem>, String>` | Pending queue items |
| `einvoice_retry_queue_item` | `queue_id: i64` | `Result<String, String>` | Retry failed item |
| `einvoice_get_xml` | `invoice_id: i64` | `Result<String, String>` | Get encrypted XML |
| `einvoice_bulk_generate` | `invoice_ids: Vec<i64>` | `Result<String, String>` | Batch generation |

---

## Backup (backup)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `backup_create` | (none) | `Result<String, String>` | Create timestamped backup |
| `backup_restore` | `backup_path: String` | `Result<String, String>` | Restore from file |
| `backup_list` | (none) | `Result<Vec<BackupInfo>, String>` | Available backups |
| `backup_get_info` | `backup_id: String` | `Result<BackupInfo, String>` | Backup metadata |
| `backup_auto` | (none) | `Result<String, String>` | Scheduled auto-backup |
| `backup_export_csv` | (none) | `Result<String, String>` | Export all to CSV |

---

## Government (government)

Omani government compliance reporting:

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `gov_get_dashboard` | (none) | `Result<GovDashboard, String>` | Submission overview |
| `gov_list_entities` | (none) | `Result<Vec<GovEntity>, String>` | Government entities |
| `gov_list_submissions` | (none) | `Result<Vec<GovSubmission>, String>` | All submissions |
| `gov_get_employee_doc_status` | `employee_id: i64` | `Result<DocStatus, String>` | Document compliance |
| `gov_submit_report` | `entity_id: i64, report_type: String` | `Result<String, String>` | Submit to portal |

---

## Import Tracking (import_tracking)

Chinese import shipment management:

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_shipments` | (none) | `Result<Vec<Shipment>, String>` | All shipments |
| `get_shipment` | `id: i64` | `Result<Shipment, String>` | Shipment details |
| `create_shipment` | `input: CreateShipmentInput` | `Result<i64, String>` | New shipment |
| `update_shipment` | `id: i64, input: UpdateShipmentInput` | `Result<String, String>` | Update shipment |
| `update_shipment_status` | `id: i64, status: String` | `Result<String, String>` | Advance status |

---

## Barter Exchange (barter_exchange)

Local supplier barter (bags for cartons):

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_barter_exchanges` | (none) | `Result<Vec<BarterExchange>, String>` | All barter records |
| `create_barter_exchange` | `input: CreateBarterInput` | `Result<i64, String>` | New barter record |
| `get_barter_balance` | `local_supplier_id: i64` | `Result<BarterBalance, String>` | Net balance |

---

## Installment Payments (installment_payments)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_installment_payments` | `installment_id: i64` | `Result<Vec<InstallmentPayment>, String>` | All payments |
| `create_installment_payment` | `input: CreateInstallmentPaymentInput` | `Result<i64, String>` | Record payment |
| `mark_installment_paid` | `id: i64` | `Result<String, String>` | Mark as paid |
| `get_installment_summary` | `installment_id: i64` | `Result<InstallmentSummary, String>` | Plan overview |

---

## File Reader (file_reader)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `file_read_text` | `path: String` | `Result<String, String>` | Read text file |
| `file_read_spreadsheet` | `path: String` | `Result<SpreadsheetData, String>` | Read Excel/XLSX |
| `file_read_docx` | `path: String` | `Result<DocxData, String>` | Read Word DOCX |
| `file_read_any` | `path: String` | `Result<AnyFileData, String>` | Auto-detect format |
| `file_get_info` | `path: String` | `Result<FileInfo, String>` | File metadata |

---

## Device (device)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_printers` | (none) | `Result<Vec<Printer>, String>` | Installed printers |
| `print_html` | `printer_name: String, html: String` | `Result<String, String>` | Print HTML |
| `print_thermal` | `printer_name: String, html: String` | `Result<String, String>` | Thermal print |
| `list_scanners` | (none) | `Result<Vec<Scanner>, String>` | Connected scanners |
| `scan_document` | `scanner_id: String` | `Result<ScanResult, String>` | Scan document |

---

## Historical Import (historical_import)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `preview_import` | `file_path: String, template_type: String` | `Result<ImportPreview, String>` | Preview data |
| `execute_import` | `file_path: String, template_type: String, mapping: JsonValue` | `Result<ImportResult, String>` | Execute import |
| `get_import_templates` | (none) | `Result<Vec<Template>, String>` | Available templates |

---

## AI Assistant (ai_assistant)

Conversational AI integrated into ERP:

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `save_ai_settings` | `settings: JsonValue` | `Result<String, String>` | Configure AI |
| `get_ai_settings` | (none) | `Result<AiSettings, String>` | Current AI config |
| `ai_chat` | `message: String` | `Result<AiResponse, String>` | Chat with AI |
| `chat_with_ai` | `message: String, context: String` | `Result<AiResponse, String>` | Chat with business context |
| `test_ai_connection` | (none) | `Result<bool, String>` | Verify provider connectivity |
| `save_ai_provider_settings` | `settings: JsonValue` | `Result<String, String>` | Save API keys (encrypted) |
| `ai_analyze_entity` | `entity_type: String, entity_id: i64` | `Result<AiAnalysis, String>` | AI entity analysis |
| `ai_suggest_actions` | (none) | `Result<Vec<Suggestion>, String>` | Next-action suggestions |

---

## RBAC & Audit (rbac)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_audit_logs` | `entity, action, user_id, date_from, date_to, limit` | `Result<Vec<AuditLogEntry>, String>` | Filtered audit query |

---

## Settings (settings)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `get_company_settings` | (none) | `Result<CompanySettings, String>` | Full company profile |
| `update_company_settings` | `input: UpdateSettingsInput` | `Result<String, String>` | Update settings |
| `list_users` | (none) | `Result<Vec<SettingsUser>, String>` | All users |
| `create_user` | `caller_id: i64, input: CreateUserInput` | `Result<i64, String>` | New user |
| `update_user` | `id: i64, input: UpdateUserInput` | `Result<String, String>` | Update user |
| `reset_user_password` | `caller_id: i64, id: i64, new_password: String` | `Result<String, String>` | Reset password |
| `delete_user` | `caller_id: i64, id: i64` | `Result<String, String>` | Delete user |

---

## Cashbank (cashbank)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_cashbank_accounts` | (none) | `Result<Vec<CashbankAccount>, String>` | All bank accounts |
| `create_cashbank_account` | `input: CreateCashbankInput` | `Result<i64, String>` | New account |

---

## Cheques (cheques)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_cheques` | (none) | `Result<Vec<Cheque>, String>` | All cheques |
| `create_cheque` | `input: CreateChequeInput` | `Result<i64, String>` | New cheque record |

---

## Renewals (renewals)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_renewals` | (none) | `Result<Vec<Renewal>, String>` | All renewals |
| `create_renewal` | `input: CreateRenewalInput` | `Result<i64, String>` | New renewal reminder |

---

## Petty Cash (petty_cash)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_petty_cash_accounts` | (none) | `Result<Vec<PettyCashAccount>, String>` | All petty funds |
| `create_petty_cash_account` | `input: CreatePettyInput` | `Result<i64, String>` | New petty fund |

---

## MCP (mcp)

Model Context Protocol server for AI integration — exposes all ERP data and operations as MCP tools for AI assistants and agent systems.

---

## Encryption (crypto)

AES-256-GCM encryption utilities for sensitive data at rest. All API keys, license keys, and payment credentials are encrypted before storage in SQLite using `crypto.encrypt()` and decrypted with `crypto.decrypt()`.

---

## Approval Workflows (approvals)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_approval_requests` | `filter: Option<ApprovalListFilter>` | `Result<Vec<ApprovalRequest>, String>` | List approval requests with optional filters (status, type, entity) |
| `create_approval_request` | `input: CreateApprovalInput` | `Result<i64, String>` | Create a new approval request for any entity |
| `decide_approval` | `input: DecideApprovalInput` | `Result<String, String>` | Approve or reject a pending request (with reason) |
| `get_approval_summary` | (none) | `Result<Value, String>` | Dashboard: pending count, approved/rejected today, pending amount |

### Structs

```rust
struct ApprovalRequest {
    id: i64, request_type: String, entity_type: String, entity_id: i64,
    entity_number: String, requested_by: String, requested_at: String,
    amount_milli: Option<i64>, description: Option<String>, status: String,
    approved_by: Option<String>, approved_at: Option<String>,
    rejection_reason: Option<String>, priority: String,
}

struct CreateApprovalInput {
    request_type: String, entity_type: String, entity_id: i64,
    entity_number: String, requested_by: String,
    amount_milli: Option<i64>, description: Option<String>,
    priority: Option<String>,  // "urgent" | "high" | "normal" (default)
}

struct DecideApprovalInput {
    id: i64, decision: String, decided_by: String, reason: Option<String>,
}
```

---

## Budget Planning (budget)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_budgets` | (none) | `Result<Vec<Budget>, String>` | List all budgets |
| `get_budget` | `id: i64` | `Result<Budget, String>` | Get budget by ID with computed variance |
| `get_budget_lines` | `budget_id: i64` | `Result<Vec<BudgetLine>, String>` | Get line items for a budget |
| `create_budget` | `input: CreateBudgetInput` | `Result<i64, String>` | Create budget with line items |
| `approve_budget` | `id: i64, approved_by: String` | `Result<String, String>` | Approve a draft budget |
| `update_budget_actuals` | `budget_id: i64` | `Result<String, String>` | Recalculate actuals from journal entries |
| `get_budget_vs_actual` | `budget_id: i64` | `Result<Value, String>` | Full variance report by account |

### Structs

```rust
struct Budget {
    id: i64, budget_no: String, name: String, department: Option<String>,
    year: i64, period: String, status: String,
    total_planned_milli: i64, total_actual_milli: i64, variance_milli: i64,
    notes: Option<String>, created_by: String, created_at: String,
    approved_by: Option<String>, approved_at: Option<String>,
}

struct BudgetLine {
    id: i64, budget_id: i64, account_code: String, account_name: Option<String>,
    planned_milli: i64, actual_milli: i64, variance_milli: i64, notes: Option<String>,
}

struct CreateBudgetInput {
    name: String, department: Option<String>, year: i64, period: String,
    created_by: String, notes: Option<String>,
    lines: Vec<BudgetLineInput>,  // { account_code, planned_milli, notes }
}
```

---

## Fixed Asset Management (assets)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_assets` | (none) | `Result<Vec<Asset>, String>` | List all fixed assets |
| `get_asset` | `id: i64` | `Result<Asset, String>` | Get asset by ID |
| `create_asset` | `input: CreateAssetInput` | `Result<i64, String>` | Register a new fixed asset |
| `list_asset_maintenance` | `asset_id: i64` | `Result<Vec<AssetMaintenanceLog>, String>` | Maintenance history for an asset |
| `create_asset_maintenance` | `input: CreateMaintenanceInput` | `Result<i64, String>` | Log a maintenance record |
| `get_asset_register_summary` | (none) | `Result<Value, String>` | Summary: total cost, value, depreciation |
| `calculate_depreciation` | `asset_id: i64, months: i32` | `Result<Value, String>` | Calculate depreciation for N months |

### Structs

```rust
struct Asset {
    id: i64, asset_no: String, name: String, category: String,
    description: Option<String>, serial_number: Option<String>,
    purchase_date: String, purchase_cost_milli: i64,
    current_value_milli: i64, depreciation_method: Option<String>,
    depreciation_rate_pct: Option<f64>, useful_life_months: Option<i32>,
    accumulated_depreciation_milli: i64, location: Option<String>,
    department: Option<String>, assigned_to: Option<String>,
    supplier: Option<String>, warranty_expiry: Option<String>,
    last_maintenance: Option<String>, next_maintenance: Option<String>,
    condition_status: Option<String>, status: String,
    notes: Option<String>, active: bool, created_at: String,
}

struct CreateAssetInput {
    name: String, category: String, purchase_date: String,
    purchase_cost_milli: i64, description: Option<String>,
    serial_number: Option<String>, depreciation_method: Option<String>,
    depreciation_rate_pct: Option<f64>, useful_life_months: Option<i32>,
    location: Option<String>, department: Option<String>,
    assigned_to: Option<String>, supplier: Option<String>,
    warranty_expiry: Option<String>, notes: Option<String>,
}
```

---

## Notifications (notifications)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_notifications` | `filter: Option<NotificationFilter>` | `Result<Vec<Notification>, String>` | List notifications with optional filters |
| `create_notification` | `input: CreateNotificationInput` | `Result<i64, String>` | Create a new notification |
| `mark_notification_read` | `id: i64` | `Result<String, String>` | Mark a single notification as read |
| `mark_all_notifications_read` | `user_id: Option<i64>` | `Result<String, String>` | Mark all (or user-specific) notifications as read |
| `get_notification_count` | `user_id: Option<i64>` | `Result<Value, String>` | Get unread count and critical count |

### Structs

```rust
struct Notification {
    id: i64, user_id: Option<i64>, notification_type: String,
    title: String, message: String, entity_type: Option<String>,
    entity_id: Option<i64>, severity: String, read_status: String,
    action_url: Option<String>, created_at: String, read_at: Option<String>,
}

struct CreateNotificationInput {
    user_id: Option<i64>, notification_type: String, title: String,
    message: String, entity_type: Option<String>, entity_id: Option<i64>,
    severity: Option<String>,  // "info" (default) | "warning" | "critical"
    action_url: Option<String>,
}
```

---

## Data Types Reference

### Monetary Values
All monetary values are stored as INTEGER in milli (1/1000 OMR). For example, 1234.567 OMR is stored as 1234567. This avoids floating-point precision errors in financial calculations.

### Status Enums (used across multiple tables)
- Invoice status: Draft → Posted → Void
- Purchase status: Draft → Posted
- Payment status: pending → paid
- Employee status: active / inactive
- General: Draft, Open, Closed, Pending, Approved, Rejected

---

## Security Notes

1. **Password hashing:** Argon2id with configurable salt
2. **Encryption at rest:** AES-256-GCM for API keys, license keys, payment credentials, and government API keys stored in `einvoice_settings`, `ai_assistant_settings`, and `gov_integrations` tables
3. **CSP headers:** `default-src 'self'; script-src 'self'; frame-ancestors 'none'`
4. **RBAC:** Every mutation command checks the caller's role permissions
5. **Audit trail:** Every mutation writes to `audit_logs` with before/after values
6. **Session:** JWT token stored in Rust `login_state`, validated on every command call
7. **Developer PIN:** Auto-generated per machine during first run, stored in `app_settings` table

---

## Error Handling

All commands return `Result<T, String>`:
- `Ok(value)` — Successful operation
- `Err(message)` — Human-readable error string

The frontend should display `Err` messages to the user and log them for audit purposes.