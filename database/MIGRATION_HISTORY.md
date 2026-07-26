# Migration History — PRO MAX OS

**Database Engine:** SQLite with WAL mode, 5-second busy timeout
**Schema Version:** 22 (tracked in `app_settings` table, key `schema_version`)
**Monetary Precision:** All amounts in milli (1/1000 OMR), stored as INTEGER

---

## Migration Overview

| Version | Type | Description | Tables Affected |
|---------|------|-------------|-----------------|
| 0 | Base Schema | Full schema.sql applied (54 tables) | All base tables |
| 1–12 | No-op | Tables already existed in schema.sql | — |
| 13 | New Table | Overtime records | `overtime_records` |
| 14 | New Table | Production shift lines | `production_shift_lines` |
| 15 | ALTER + Indexes | Invoice customs price, performance indexes | `sales_invoice_lines` + 14 indexes |
| 16 | New Table | Machine temperature logs | `machine_temp_logs` |
| 17 | New Tables | Government entities (4 tables + seed) | `gov_entities`, `gov_integrations`, `gov_report_templates`, `gov_submissions` |
| 18 | New Tables | Multi-company accounting (5 tables + ALTERs) | `companies`, `fiscal_years`, `tax_rates`, `currencies`, `exchange_rates` + `customers`, `suppliers` |
| 19 | New Tables | E-Invoice enhancements (2 tables + ALTERs) | `einvoice_settings`, `einvoice_queue` + `e_invoices` |
| 20 | New Table | Password change attempts | `password_change_attempts` |
| 21 | ALTER + Indexes | Expense reimbursement workflow | `expenses` (7 new columns) + 4 indexes |
| 22 | Massive Enhancement | Factory ERP complete (15 ALTERs, 4 new tables, seed) | `products`, `employees`, `production_shift_lines`, `operations_daily_sheets`, `import_shipments`, `installments`, `suppliers`, `customers` + `local_supplier_exchanges`, `installment_payments`, `shift_inventory_snapshots`, `employee_leave_types`, `employee_leave_requests`, `worker_daily_production` |

---

## Detailed Migration Log

### Migration 0 — Base Schema (schema.sql)
**Applied:** Initial database creation
**Tables Created:** 54 tables (see schema_documentation.md for full list)
**Key Tables:** users, company_settings, accounts, journal_entries, products, inventory_items, customers, suppliers, machines, employees, production_orders, sales_invoices, purchases, expenses, cashbank_accounts, audit_logs, and all supporting tables.

---

### Migrations 1–12 — No-ops
**Reason:** All tables for these phases (custody, maintenance, invoicing, accounting, production, payroll, etc.) were already defined in the base `schema.sql` file. The migration system applies `schema.sql` wholesale for new databases, then runs incremental migrations 13–22 for upgrades.

---

### Migration 13 — Overtime Records
**Created Table:** `overtime_records`
| Column | Type | Constraints |
|--------|------|-------------|
| id | INTEGER | PRIMARY KEY AUTOINCREMENT |
| employee_id | INTEGER | NOT NULL |
| date | TEXT | NOT NULL |
| hours | REAL | NOT NULL DEFAULT 0 |
| rate_multiplier | REAL | NOT NULL DEFAULT 1.5 |
| reason | TEXT | — |
| approved | INTEGER | NOT NULL DEFAULT 0 |
| approved_by | TEXT | — |
| approved_at | TEXT | — |
| status | TEXT | NOT NULL DEFAULT 'Pending' |
| notes | TEXT | — |
| created_by | TEXT | — |
| created_at | TEXT | — |

**Indexes:** `idx_ot_employee`, `idx_ot_date`, `idx_ot_status`

---

### Migration 14 — Production Shift Lines
**Created Table:** `production_shift_lines`
| Column | Type | Constraints |
|--------|------|-------------|
| id | INTEGER | PRIMARY KEY AUTOINCREMENT |
| sheet_id | INTEGER | NOT NULL, REFERENCES operations_daily_sheets(id) |
| product_id | INTEGER | NOT NULL, REFERENCES products(id) |
| customer_brand | TEXT | — |
| cartons_produced | REAL | NOT NULL DEFAULT 0 |
| cups_per_carton | INTEGER | NOT NULL DEFAULT 1000 |
| waste_cartons | REAL | NOT NULL DEFAULT 0 |
| ts | TEXT | NOT NULL DEFAULT (datetime('now')) |
| recorded_by | TEXT | — |

**Indexes:** `idx_psl_sheet`, `idx_psl_product`, `idx_psl_date`

---

### Migration 15 — ALTER TABLE + Performance Indexes
**ALTER TABLE:** `sales_invoice_lines` ADD `customs_price_milli` INTEGER NOT NULL DEFAULT 0

**Indexes Created (14):**
- `idx_inv_items_supplier` on `inventory_items(supplier_id)`
- `idx_ops_product` on `operations_daily_sheets(product_id)`
- `idx_mnt_machine_id` on `maintenance_daily_sheets(machine_id)`
- `idx_cn_customer_id` on `credit_notes(customer_id)`
- `idx_cnlines_cn` on `credit_note_lines(cn_id)`
- `idx_cnlines_product` on `credit_note_lines(product_id)`
- `idx_imp_supplier` on `import_shipments(supplier_id)`
- `idx_impcost_shipment` on `import_shipment_costs(shipment_id)`
- `idx_impalloc_shipment` on `import_shipment_allocations(shipment_id)`
- `idx_impalloc_item` on `import_shipment_allocations(item_id)`
- `idx_st_transfer_from` on `stock_transfers(from_warehouse_id)`
- `idx_st_transfer_to` on `stock_transfers(to_warehouse_id)`
- `idx_quality_pline` on `quality_inspections(production_line_id)`
- `idx_docflow_entity` on `docflow_documents(entity_type, entity_id)`
- `idx_dc_date` on `daily_closings(date)`
- `idx_login_attempts_user` on `login_attempts(username)`

---

### Migration 16 — Machine Temperature Logs
**Created Table:** `machine_temp_logs`
| Column | Type | Constraints |
|--------|------|-------------|
| id | INTEGER | PRIMARY KEY AUTOINCREMENT |
| machine_id | INTEGER | NOT NULL, REFERENCES machines(id) |
| temperature | REAL | NOT NULL |
| ts | TEXT | NOT NULL DEFAULT (datetime('now')) |
| recorded_by | TEXT | — |

**Indexes:** `idx_mtl_machine`, `idx_mtl_ts`

---

### Migration 17 — Government Entities (Omani Compliance)
**Created Tables (4):**
1. `gov_entities` — 10 seeded Omani entities (MOL, MoCI, ROP, NCSI, PASI, Ministry of Economy, Tax Authority, Commercial Registry, Work Permits, Residency)
2. `gov_integrations` — API config per entity
3. `gov_report_templates` — Report templates per entity
4. `gov_submissions` — Submission tracking

**Indexes:** `idx_gov_ent_cat`, `idx_gov_ent_active`, `idx_gov_sub_entity`, `idx_gov_sub_status`

---

### Migration 18 — Multi-Company Accounting
**Created Tables (5):**
1. `companies` — Company profiles (multi-tenant foundation)
2. `fiscal_years` — Fiscal year definitions per company
3. `tax_rates` — Tax rates per company
4. `currencies` — Currency master data
5. `exchange_rates` — Daily exchange rates

**ALTER TABLE:**
- `customers` ADD `contact_person` TEXT, ADD `company_type` TEXT, ADD `industry` TEXT
- `suppliers` ADD `contact_person` TEXT, ADD `payment_terms_days` INTEGER NOT NULL DEFAULT 30

---

### Migration 19 — E-Invoice Enhancements (ZATCA/FATOORA)
**Created Tables (2):**
1. `einvoice_settings` — Per-company ZATCA configuration
2. `einvoice_queue` — Processing queue with retry logic

**ALTER TABLE:** `e_invoices` ADD `cancel_reason` TEXT, ADD `cancelled_at` TEXT, ADD `cancelled_by` TEXT

**Indexes:** `idx_einv_q_status`, `idx_einv_q_next`, `idx_einv_comp`

---

### Migration 20 — Password Change Attempts
**Created Table:** `password_change_attempts`
| Column | Type | Constraints |
|--------|------|-------------|
| id | INTEGER | PRIMARY KEY AUTOINCREMENT |
| user_id | INTEGER | NOT NULL, REFERENCES users(id) |
| ts | REAL | NOT NULL |
| ok | INTEGER | NOT NULL DEFAULT 0 |

**Indexes:** `idx_pca_user`, `idx_pca_ts`

---

### Migration 21 — Expense Reimbursement Workflow
**ALTER TABLE `expenses` (7 new columns):**
- `paid_by_employee_id` INTEGER
- `paid_from_source` TEXT DEFAULT 'company'
- `petty_id` INTEGER
- `custody_txn_id` INTEGER
- `reimbursement_status` TEXT DEFAULT 'none'
- `reimbursement_date` TEXT
- `reimbursed_by` TEXT

**Indexes:** `idx_exp_paid_by`, `idx_exp_source`, `idx_exp_petty`, `idx_exp_reimburse`

---

### Migration 22 — Factory ERP Complete Enhancement
**Largest migration — 15 ALTER TABLEs, 4 new tables, seed data**

**ALTER TABLE `products` (15 columns):**
`brand_name`, `cup_size_ml`, `cup_diameter_mm`, `paper_weight_gsm`, `lid_type`, `print_colors`, `carton_length_cm`, `carton_width_cm`, `carton_height_cm`, `color`, `material_type`, `product_type` DEFAULT 'cup', `family_id`, `min_stock`, `weight_kg`

**ALTER TABLE `employees` (19 columns):**
`id_number`, `date_of_birth`, `gender`, `marital_status`, `email`, `bank_name`, `bank_account_no`, `basic_salary_milli`, `housing_allowance_milli`, `transport_allowance_milli`, `food_allowance_milli`, `other_allowances_milli`, `overtime_rate_milli`, `insurance_policy_no`, `insurance_premium_milli`, `ticket_allowance_milli`, `sponsor_name`, `sponsor_id`

**ALTER TABLE `production_shift_lines`:** ADD `worker_id` INTEGER
**ALTER TABLE `operations_daily_sheets` (5 columns):** `worker_id`, `machine_id`, `starting_qty`, `ending_qty`, `break_minutes`
**ALTER TABLE `import_shipments` (19 columns):** Full logistics tracking (container, B/L, vessel, ports, customs, costs, weights)
**ALTER TABLE `installments` (7 columns):** Interest, monthly amount, installment count, penalties, collateral, guarantor
**ALTER TABLE `suppliers` (3 columns):** `supplier_type`, `lead_time_days`, `local_exchange_enabled`
**ALTER TABLE `customers` (3 columns):** `credit_days`, `default_discount_pct`, `route`

**New Table: `local_supplier_exchanges`** — Barter system (bags ↔ cartons)
**New Table: `installment_payments`** — Per-installment payment tracking
**New Table: `shift_inventory_snapshots`** — Shift-level stock counts
**New Table: `employee_leave_types`** — 6 seeded types (annual, sick, casual, hajj, maternity, unpaid)
**New Table: `employee_leave_requests`** — Leave workflow
**New Table: `worker_daily_production`** — Per-worker daily output

**Indexes (24):** On all new columns and foreign keys

---

## Migration Execution Notes

- **New databases:** `schema.sql` applied in full, then migrations 13–22 run sequentially
- **Upgrade path:** Existing databases run migrations 1–22 sequentially (1–12 are no-ops)
- **Rollback:** Not supported — migrations are additive only
- **Version tracking:** `app_settings` table, key `schema_version`, updated after each migration
- **Idempotency:** Each migration checks current schema version before applying