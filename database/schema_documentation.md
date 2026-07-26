# Database Schema Documentation — PRO MAX OS

**Database:** SQLite (WAL mode, 5s busy timeout)
**Tables:** 74+ (54 base + 20 migration additions)
**Schema Version:** 22
**Monetary Precision:** All amounts in milli (1/1000 OMR), INTEGER

---

## Table Catalog

### Core Configuration
| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `app_settings` | Key-value app settings | `key` PK, `value` |
| `doc_sequences` | Document numbering | `doc_type`, `year`, `last_number` PK |
| `company_settings` | Factory profile | `id`=1 PK, name, vat, logo, defaults |

### Users & Security
| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `users` | System users | `id` PK, `username` UNIQUE, `password_hash`, `salt`, `role`, `active` |
| `roles` | Role definitions | `id` PK, `code` UNIQUE, `name` |
| `permissions` | Permission definitions | `id` PK, `code` UNIQUE, `name` |
| `role_permissions` | Role→Permission mapping | `role_code`, `perm_code` PK |
| `user_roles` | User→Role mapping | `user_id`, `role_code` PK |
| `login_attempts` | Rate limiting | `username`, `ts`, `ok` |
| `password_change_attempts` | Password rate limiting | `user_id`, `ts`, `ok` |

### Chart of Accounts & GL
| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `accounts` | Chart of accounts | `code` PK, `name_ar`, `name_en`, `type`, `parent` |
| `journal_entries` | General ledger entries | `id` PK, `entry_no`, `date`, `memo`, `ref_type`, `ref_id` |
| `journal_entry_lines` | GL entry lines | `id` PK, `entry_id` FK, `account_code` FK, `debit_milli`, `credit_milli` |

### Products & Inventory
| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `products` | Finished goods (cups/cartons) | `id` PK, `code`, `name_ar`, `cup_type`, `cups_per_carton`, `default_price_milli`, `default_cost_milli`, `product_type`, `brand_name`, `family_id`, `min_stock` |
| `product_prices` | Per-customer pricing | `id` PK, `product_id` FK, `customer_id`, `price_milli` |
| `product_price_history` | Price change audit | `product_id`, `price_milli`, `effective_date`, `changed_by` |
| `product_families` | Product groupings | `id` PK, `code`, `name_ar`, `name_en`, `category` |
| `inventory_items` | Raw materials, packaging | `id` PK, `code`, `name_ar`, `kind` (raw/pack/finished), `uom`, `product_id` FK, `qty_on_hand`, `avg_cost_milli`, `reorder_level`, `supplier_id` |
| `inventory_movements` | Stock movement log | `id` PK, `ts`, `item_id` FK, `mtype`, `qty_in`, `qty_out`, `unit_cost_milli`, `ref_type`, `ref_id` |
| `inventory_adjustments` | Manual count corrections | `id` PK, `adj_no`, `date`, `item_id`, `direction`, `qty`, `reason`, `status` |
| `bom` | Bill of Materials | `id` PK, `product_id` FK, `item_id` FK, `qty_per_carton`, `waste_pct` |

### Customers & Sales
| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `customers` | Customer master | `id` PK, `code`, `name`, `ctype`, `contact`, `phone`, `email`, `address`, `vat_number`, `credit_limit_milli`, `payment_terms`, `balance_milli`, `credit_days`, `default_discount_pct`, `route` |
| `customer_aliases` | Alias matching | `id` PK, `customer_id` FK, `alias`, `source` |
| `sales_invoices` | Sales invoices | `id` PK, `inv_no`, `date`, `customer_id` FK, `payment_type`, `vat_enabled`, `net_milli`, `vat_milli`, `discount_milli`, `total_milli`, `paid_milli`, `status` |
| `sales_invoice_lines` | Invoice lines | `id` PK, `invoice_id` FK, `product_id` FK, `cartons`, `cups_per_carton`, `qty_cups`, `unit_price_milli`, `line_net_milli`, `vat_pct`, `vat_milli`, `customs_price_milli` |
| `customer_payments` | Receipts | `id` PK, `rec_no`, `date`, `customer_id` FK, `amount_milli`, `method`, `cashbank_id` |
| `payment_allocations` | Payment→Invoice allocation | `id` PK, `payment_id` FK, `invoice_id` FK, `amount_milli` |
| `credit_notes` | Credit notes | `id` PK, `cn_no`, `date`, `customer_id`, `invoice_id`, `net_milli`, `vat_milli`, `total_milli`, `cogs_milli`, `reason`, `status` |
| `credit_note_lines` | CN lines | `id` PK, `cn_id`, `product_id`, `cartons`, `cups_per_carton`, `qty_cups`, `unit_price_milli`, `line_net_milli`, `vat_pct`, `vat_milli` |

### Suppliers & Purchases
| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `suppliers` | Supplier master | `id` PK, `code`, `name`, `is_foreign`, `contact`, `phone`, `email`, `address`, `currency`, `payment_terms`, `balance_milli`, `bank_details`, `supplier_type`, `lead_time_days`, `local_exchange_enabled` |
| `supplier_price_history` | Purchase price audit | `id` PK, `supplier_id`, `item_id`, `cost_milli`, `effective_date` |
| `purchases` | Purchase orders | `id` PK, `pur_no`, `date`, `supplier_id` FK, `supplier_invoice_no`, `vat_enabled`, `net_milli`, `vat_milli`, `total_milli`, `paid_milli`, `status` |
| `purchase_lines` | PO lines | `id` PK, `purchase_id` FK, `item_id` FK, `qty`, `unit_cost_milli`, `line_net_milli`, `vat_pct`, `vat_milli` |
| `supplier_payments` | Payments to suppliers | `id` PK, `pay_no`, `date`, `supplier_id` FK, `amount_milli`, `method`, `cashbank_id` |

### Production
| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `production_orders` | Production orders | `id` PK, `prod_no`, `date`, `shift`, `machine_id`, `operator`, `supervisor`, `run_minutes`, `downtime_minutes`, `status`, `journal_id` |
| `production_lines` | Order lines | `id` PK, `order_id` FK, `product_id` FK, `cups_per_carton`, `cartons_good`, `cups_good`, `cartons_waste`, `cups_waste`, `unit_cost_milli`, `worker`, `brand_type`, `customer_id`, `quality_status` |
| `operations_daily_sheets` | Daily shift sheets | `id` PK, `sheet_no`, `date`, `shift`, `supervisor_name`, `worker_name`, `attendance`, `start_time`, `end_time`, `normal_hours`, `overtime_hours`, `product_id`, `cartons_produced`, `total_cups`, `waste_cartons`, `worker_id`, `machine_id`, `starting_qty`, `ending_qty`, `break_minutes` |
| `production_shift_lines` | Shift line items | `id` PK, `sheet_id` FK, `product_id` FK, `customer_brand`, `cartons_produced`, `cups_per_carton`, `waste_cartons`, `ts`, `recorded_by`, `worker_id` |
| `worker_daily_production` | Per-worker output | `id` PK, `employee_id` FK, `date`, `shift`, `total_cartons`, `total_cups`, `total_waste_cartons`, `products_breakdown`, `recorded_by` |
| `shift_inventory_snapshots` | Shift stock counts | `id` PK, `date`, `shift`, `item_id` FK, `opening_qty`, `received_qty`, `consumed_qty`, `produced_qty`, `waste_qty`, `closing_qty` |

### Maintenance & Operations
| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `maintenance_daily_sheets` | Maintenance logs | `id` PK, `sheet_no`, `date`, `shift`, `maintenance_supervisor`, `machine_id`, `area`, `fault_title`, `fault_description`, `severity`, `machine_stopped`, `downtime_minutes`, `repair_status`, `total_repair_cost_milli`, `root_cause` |
| `machine_temp_logs` | Machine temperature | `id` PK, `machine_id` FK, `temperature`, `ts`, `recorded_by` |

### HR & Payroll
| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `employees` | Employee master | `id` PK, `code`, `name`, `nationality`, `job`, `salary_milli`, `allowances_milli`, `id_number`, `date_of_birth`, `gender`, `marital_status`, `passport_no`, `passport_expiry`, `residence_expiry`, `visa_expiry`, `workpermit_expiry`, `insurance_expiry`, `contract_end`, `joining_date`, `bank_name`, `bank_account_no`, `basic_salary_milli`, `housing_allowance_milli`, `transport_allowance_milli`, `food_allowance_milli`, `other_allowances_milli`, `overtime_rate_milli`, `insurance_policy_no`, `insurance_premium_milli`, `ticket_allowance_milli`, `sponsor_name`, `sponsor_id` |
| `payroll_runs` | Payroll periods | `id` PK, `run_no`, `period_start`, `period_end`, `status`, `total_gross_milli`, `total_deductions_milli`, `total_net_milli`, `accrual_journal_id`, `journal_id` |
| `payroll_run_lines` | Payroll per employee | `id` PK, `run_id` FK, `employee_id` FK, `basic_milli`, `allowance_milli`, `overtime_milli`, `bonus_milli`, `deduction_milli`, `advance_deduction_milli`, `insurance_deduction_milli`, `tax_deduction_milli`, `net_milli` |
| `employee_advances` | Salary advances | `id` PK, `employee_id` FK, `amount_milli`, `date`, `reason`, `status`, `remaining_milli`, `deduction_per_payroll_milli` |
| `overtime_records` | Overtime tracking | `id` PK, `employee_id` FK, `date`, `hours`, `rate_multiplier`, `reason`, `approved`, `approved_by`, `approved_at`, `status` |
| `employee_leave_types` | Leave type master | `id` PK, `code` UNIQUE, `name`, `default_days_per_year`, `paid`, `active` |
| `employee_leave_requests` | Leave workflow | `id` PK, `employee_id` FK, `leave_type_id` FK, `from_date`, `to_date`, `days`, `reason`, `status`, `approved_by`, `approved_at` |

### Finance & Cash
| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `cashbank_accounts` | Bank/cash accounts | `id` PK, `code`, `name`, `atype`, `account_code` FK, `balance_milli` |
| `cashbank_transactions` | Cash flow log | `id` PK, `ts`, `cashbank_id` FK, `debit_milli`, `credit_milli`, `balance_milli`, `method`, `ref_type`, `ref_id` |
| `cheques` | Cheque register | `id` PK, `kind`, `cheque_no`, `bank`, `party`, `amount_milli`, `due_date`, `status`, `link_type`, `link_id` |
| `renewals` | Document renewals | `id` PK, `name`, `category`, `authority`, `issue_date`, `expiry_date`, `cost_milli`, `responsible`, `alert_days`, `status` |
| `installments` | Loan/installment plans | `id` PK, `name`, `source`, `original_milli`, `currency`, `start_date`, `due_date`, `paid_milli`, `status`, `interest_pct`, `monthly_installment_milli`, `num_installments`, `paid_installments`, `penalty_pct`, `collateral`, `guarantor` |
| `installment_payments` | Per-installment payments | `id` PK, `installment_id` FK, `installment_number`, `due_date`, `amount_milli`, `paid_milli`, `paid_date`, `penalty_milli`, `status` |
| `petty_cash_accounts` | Petty cash funds | `id` PK, `code`, `name`, `responsible`, `role`, `employee_id`, `spending_limit_milli`, `requires_approval`, `balance_milli`, `status` |
| `petty_cash_transactions` | Petty cash log | `id` PK, `ts`, `petty_id` FK, `ttype`, `debit_milli`, `credit_milli`, `balance_milli`, `category`, `account_code`, `cashbank_id`, `counter_petty_id`, `expense_id`, `attachment_status` |

### Custody (Petty Cash Funds)
| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `custody_accounts` | Custody fund master | Same structure as petty_cash_accounts |

### Expenses
| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `expenses` | Expense records | `id` PK, `exp_no`, `date`, `category`, `account_code`, `amount_milli`, `vat_milli`, `method`, `paid_from_source`, `cashbank_id`, `petty_id`, `vendor`, `reference`, `attachment_required`, `approval_status`, `paid_by_employee_id`, `custody_txn_id`, `reimbursement_status`, `reimbursement_date`, `reimbursed_by` |

### E-Invoice (ZATCA/FATOORA)
| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `e_invoices` | Generated e-invoices | `id` PK, `invoice_id` FK, `xml_hash`, `status`, `uuid`, `qr_code`, `submission_id`, `cancel_reason`, `cancelled_at`, `cancelled_by` |
| `einvoice_settings` | Per-company ZATCA config | `id` PK, `company_id` FK, `tax_authority_endpoint`, `api_key`, `api_secret`, `environment`, `auto_submit`, `compliance_certificate`, `certificate_expiry` |
| `einvoice_queue` | Processing queue | `id` PK, `invoice_id` FK, `action`, `priority`, `retry_count`, `max_retries`, `last_error`, `next_retry_at`, `status` |

### Government Compliance (Oman)
| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `gov_entities` | Government bodies | `id` PK, `code` UNIQUE, `name_ar`, `name_en`, `category`, `api_endpoint`, `api_key_required`, `active` |
| `gov_integrations` | Entity API config | `id` PK, `entity_id` FK, `config_key`, `config_value`, `encrypted` |
| `gov_report_templates` | Report definitions | `id` PK, `entity_id` FK, `code` UNIQUE, `name_ar`, `report_type`, `period`, `format`, `active` |
| `gov_submissions` | Submission tracking | `id` PK, `entity_id` FK, `report_template_id`, `status`, `payload`, `response`, `reference_no`, `submitted_at` |

### Import Tracking (China Imports)
| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `import_shipments` | Shipment master | `id` PK, `shipment_no`, `supplier_id` FK, `currency`, `exchange_rate`, `status`, `shipping_company`, `container_no`, `bl_no`, `vessel_flight`, `port_of_loading`, `port_of_discharge`, `estimated_arrival`, `actual_arrival`, `customs_declaration_no`, `customs_clearance_date`, `duty_amount_milli`, `vat_on_import_milli`, `freight_cost_milli`, `insurance_cost_milli`, `handling_cost_milli`, `commercial_invoice_no`, `packing_list_no`, `origin_country`, `gross_weight_kg`, `cbm`, `clearance_agent`, `total_landed_cost_milli` |
| `import_shipment_costs` | Shipment cost breakdown | `id` PK, `shipment_id` FK, `cost_type`, `amount_milli` |
| `import_shipment_allocations` | Item cost allocation | `id` PK, `shipment_id` FK, `item_id` FK, `qty`, `allocated_cost_milli` |

### Barter Exchange (Local Suppliers)
| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `local_supplier_exchanges` | Barter: bags ↔ cartons | `id` PK, `exchange_no`, `date`, `local_supplier_id` FK, `product_id` FK, `cartons_given`, `carton_value_milli`, `received_item_id` FK, `bags_received`, `bag_value_milli`, `net_value_milli`, `balance_milli`, `settlement_status`, `status` |

### Audit & Compliance
| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `audit_logs` | Full audit trail | `id` PK, `ts`, `user_id`, `username`, `action`, `entity`, `entity_id`, `old_value`, `new_value`, `reason` |
| `document_status_history` | Status changes | `id` PK, `ts`, `entity_type`, `entity_id`, `old_status`, `new_status`, `user_id`, `username`, `reason` |
| `document_voids` | Void records | `id` PK, `ts`, `entity_type`, `entity_id`, `reversal_journal_id`, `user_id`, `username`, `reason` |
| `daily_closings` | Daily close records | `id` PK, `date` UNIQUE, `snapshot_json`, `notes`, `status`, `prepared_by`, `approved_by` |

### Multi-Warehouse
| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `multi_warehouse` | Warehouse master | `id` PK, `code`, `name`, `location`, `manager`, `active` |
| `stock_transfers` | Inter-warehouse moves | `id` PK, `transfer_no`, `from_warehouse_id`, `to_warehouse_id`, `item_id`, `qty`, `status` |

### Document Flow
| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `docflow_documents` | Internal docs | `id` PK, `doc_no`, `doc_type`, `date`, `entity_type`, `entity_id`, `from_party`, `to_party`, `subject`, `body`, `status` |

### Attachments & Imports
| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `attachments` | File attachments | `id` PK, `entity_type`, `entity_id`, `original_filename`, `stored_filename`, `mime_type`, `size_bytes`, `uploaded_by`, `uploaded_at` |
| `worker_sheet_templates` | Worker sheet templates | `id` PK, `code`, `name`, `kind`, `active` |
| `worker_sheets` | Generated sheets | `id` PK, `template_id`, `worker`, `date`, `lang`, `status` |
| `historical_imports` | Excel import log | `id` PK, `import_type`, `file_name`, `status`, `records_processed`, `records_failed`, `created_by`, `created_at` |

### Approval Workflows
| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `approval_requests` | Approval workflow requests | `id` PK, `request_type`, `entity_type`, `entity_id`, `entity_number`, `requested_by`, `requested_at`, `amount_milli`, `description`, `status`, `approved_by`, `approved_at`, `rejection_reason`, `priority` |

### Budget Planning
| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `budgets` | Budget headers | `id` PK, `budget_no`, `name`, `department`, `year`, `period`, `status`, `total_planned_milli`, `total_actual_milli`, `notes`, `created_by`, `created_at`, `approved_by`, `approved_at` |
| `budget_lines` | Budget line items | `id` PK, `budget_id` FK, `account_code`, `planned_milli`, `actual_milli`, `notes` |

### Fixed Assets
| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `fixed_assets` | Fixed asset register | `id` PK, `asset_no`, `name`, `category`, `description`, `serial_number`, `purchase_date`, `purchase_cost_milli`, `current_value_milli`, `depreciation_method`, `depreciation_rate_pct`, `useful_life_months`, `accumulated_depreciation_milli`, `location`, `department`, `assigned_to`, `supplier`, `warranty_expiry`, `status`, `active` |
| `asset_maintenance_logs` | Maintenance history | `id` PK, `asset_id` FK, `date`, `description`, `cost_milli`, `performed_by`, `next_due`, `notes` |

### Notifications
| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `notifications` | System notifications | `id` PK, `user_id`, `notification_type`, `title`, `message`, `entity_type`, `entity_id`, `severity`, `read_status`, `action_url`, `created_at`, `read_at` |

---

## Key Design Decisions

1. **Monetary Precision:** All amounts stored as INTEGER milli (1/1000 OMR). Zero floating-point errors.
2. **Soft Deletes:** `active` flag (0/1) on master tables — never hard delete.
3. **Audit Trail:** Every mutation writes to `audit_logs` with before/after JSON.
4. **Referential Integrity:** Foreign keys enabled; cascade rules defined per relationship.
5. **Indexes:** Created on all FK columns, date columns, status columns, and search fields.
6. **WAL Mode:** Write-Ahead Logging for concurrent read performance.
7. **Encryption at Rest:** AES-256-GCM for `einvoice_settings.api_secret`, `gov_integrations.config_value` (when encrypted=1), `ai_assistant_settings.api_key`.

---

## Query Patterns

- **Dashboard KPIs:** Aggregate queries on `sales_invoices`, `production_orders`, `inventory_items`, `customers`
- **Aging Reports:** Date-range filters on `sales_invoices`, `customer_payments`, `purchases`, `supplier_payments`
- **Production Analytics:** Joins across `production_orders`, `production_lines`, `operations_daily_sheets`, `production_shift_lines`
- **Inventory Valuation:** `inventory_items.qty_on_hand * avg_cost_milli`
- **Payroll:** `payroll_runs` + `payroll_run_lines` + `employee_advances` + `overtime_records`

---

## Migration History

See `MIGRATION_HISTORY.md` for the complete 22-version migration log.