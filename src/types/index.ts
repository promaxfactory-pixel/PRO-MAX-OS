export interface User {
  id: number;
  username: string;
  full_name: string;
  role: string;
  active: number;
  must_change_password: number;
  created_at: string;
}

export interface CompanySettings {
  id: number;
  name: string;
  factory_name: string;
  address: string;
  phone: string;
  email: string;
  vat_number: string;
  logo_path: string;
  stamp_path: string;
  signature_path: string;
  footer_notes: string;
  bank_details: string;
  default_vat_pct: number;
}

export interface Account {
  code: string;
  name_ar: string;
  name_en: string;
  type: 'asset' | 'liability' | 'equity' | 'revenue' | 'expense';
  parent: string | null;
  is_system: number;
}

export interface JournalEntry {
  id: number;
  entry_no: string;
  date: string;
  memo: string;
  ref_type: string;
  ref_id: number;
  created_by: string;
  created_at: string;
  reversed_by: number | null;
  lines?: JournalEntryLine[];
}

export interface JournalEntryLine {
  id: number;
  entry_id: number;
  account_code: string;
  account_name?: string;
  debit_milli: number;
  credit_milli: number;
  memo: string;
}

export interface Product {
  id: number;
  code: string;
  name_ar: string;
  name_en: string;
  size: string;
  cup_type: string;
  cups_per_carton: number;
  carton_type: string;
  default_price_milli: number;
  default_cost_milli: number;
  vat_pct: number;
  barcode: string;
  notes: string;
  active: number;
}

export interface InventoryItem {
  id: number;
  code: string;
  name_ar: string;
  name_en: string;
  kind: 'finished' | 'raw' | 'packaging' | 'spare' | 'consumable';
  uom: string;
  product_id: number | null;
  qty_on_hand: number;
  avg_cost_milli: number;
  reorder_level: number;
  supplier_id: number | null;
  notes: string;
  active: number;
}

export interface InventoryMovement {
  id: number;
  ts: string;
  item_id: number;
  item_name?: string;
  mtype: string;
  qty_in: number;
  qty_out: number;
  unit_cost_milli: number;
  ref_type: string;
  ref_id: number;
  location: string;
  user_id: number;
  notes: string;
}

export interface Customer {
  id: number;
  code: string;
  name: string;
  ctype: string;
  contact: string;
  phone: string;
  email: string;
  address: string;
  vat_number: string;
  credit_limit_milli: number;
  payment_terms: string;
  opening_balance_milli: number;
  balance_milli: number;
  notes: string;
  active: number;
}

export interface Supplier {
  id: number;
  code: string;
  name: string;
  is_foreign: number;
  contact: string;
  phone: string;
  email: string;
  address: string;
  currency: string;
  payment_terms: string;
  opening_balance_milli: number;
  balance_milli: number;
  bank_details: string;
  notes: string;
  active: number;
}

export interface SalesInvoice {
  id: number;
  inv_no: string;
  date: string;
  customer_id: number;
  customer_name?: string;
  payment_type: string;
  vat_enabled: number;
  net_milli: number;
  vat_milli: number;
  discount_milli: number;
  total_milli: number;
  discount_reason: string;
  cogs_milli: number;
  paid_milli: number;
  status: string;
  cashbank_id: number | null;
  delivery: string;
  notes: string;
  created_by: string;
  created_at: string;
  journal_id: number | null;
  lines?: InvoiceLine[];
}

export interface InvoiceLine {
  id: number;
  invoice_id: number;
  product_id: number;
  product_name?: string;
  cartons: number;
  cups_per_carton: number;
  qty_cups: number;
  unit_price_milli: number;
  suggested_price_milli: number;
  line_gross_milli: number;
  line_discount_pct: number;
  line_discount_milli: number;
  discount_reason: string;
  line_net_milli: number;
  vat_pct: number;
  vat_milli: number;
}

export interface CustomerPayment {
  id: number;
  rec_no: string;
  date: string;
  customer_id: number;
  customer_name?: string;
  amount_milli: number;
  method: string;
  cashbank_id: number | null;
  reference: string;
  notes: string;
  created_by: string;
  created_at: string;
  journal_id: number | null;
}

export interface ProductionOrder {
  id: number;
  prod_no: string;
  date: string;
  shift: string;
  machine_id: number;
  machine_name?: string;
  operator: string;
  supervisor: string;
  run_minutes: number;
  downtime_minutes: number;
  downtime_reason: string;
  status: string;
  notes: string;
  approved_by: string;
  approved_at: string;
  created_by: string;
  created_at: string;
  journal_id: number | null;
  lines?: ProductionLine[];
}

export interface ProductionLine {
  id: number;
  order_id: number;
  product_id: number;
  product_name?: string;
  cups_per_carton: number;
  cartons_good: number;
  cups_good: number;
  cartons_waste: number;
  cups_waste: number;
  unit_cost_milli: number;
  worker: string;
  brand_type: string;
  customer_id: number | null;
  customer_brand_name: string;
  batch_no: string;
  quality_status: string;
  quality_notes: string;
}

export interface Machine {
  id: number;
  code: string;
  name: string;
  mtype: string;
  supported_products: string;
  purchase_date: string;
  supplier: string;
  cost_milli: number;
  capacity_cpm: number;
  status: string;
  notes: string;
  active: number;
}

export interface Employee {
  id: number;
  code: string;
  name: string;
  nationality: string;
  job: string;
  salary_milli: number;
  allowances_milli: number;
  phone: string;
  passport_no: string;
  passport_expiry: string;
  residence_expiry: string;
  visa_expiry: string;
  workpermit_expiry: string;
  insurance_expiry: string;
  contract_end: string;
  joining_date: string;
  active: number;
  notes: string;
}

export interface OperationsSheet {
  id: number;
  sheet_no: string;
  date: string;
  shift: string;
  supervisor_name: string;
  worker_name: string;
  attendance: string;
  start_time: string;
  end_time: string;
  normal_hours: number;
  overtime_hours: number;
  overtime_reason: string;
  overtime_approved: number;
  product_id: number | null;
  customer_brand_name: string;
  cartons_produced: number;
  cups_per_carton: number;
  total_cups: number;
  waste_cartons: number;
  waste_cups: number;
  cups_quality: string;
  carton_quality: string;
  packing_quality: string;
  cleaning_quality: string;
  safety_notes: string;
  notes: string;
  worker_signature: string;
  supervisor_signature: string;
  status: string;
  created_by: string;
  created_at: string;
}

export interface MaintenanceSheet {
  id: number;
  sheet_no: string;
  date: string;
  shift: string;
  maintenance_supervisor: string;
  machine_id: number | null;
  machine_name?: string;
  area: string;
  fault_title: string;
  fault_description: string;
  severity: string;
  machine_stopped: number;
  downtime_start: string;
  downtime_end: string;
  downtime_minutes: number;
  repair_status: string;
  repair_action: string;
  parts_changed: string;
  spare_parts_cost_milli: number;
  labor_cost_milli: number;
  other_cost_milli: number;
  total_repair_cost_milli: number;
  root_cause: string;
  preventive_action: string;
  next_followup_date: string;
  attachment_note: string;
  approval: string;
  close_date: string;
  notes: string;
  status: string;
  created_by: string;
  created_at: string;
  approved_by: string;
  approved_at: string;
  closed_by: string;
  closed_at: string;
}

export interface PettyCashAccount {
  id: number;
  code: string;
  name: string;
  responsible: string;
  role: string;
  employee_id: number | null;
  spending_limit_milli: number;
  requires_approval: number;
  balance_milli: number;
  status: string;
  active: number;
  notes: string;
  created_at: string;
}

export interface CashBankAccount {
  id: number;
  code: string;
  name: string;
  atype: string;
  account_code: string;
  balance_milli: number;
  active: number;
}

export interface Expense {
  id: number;
  exp_no: string;
  date: string;
  category: string;
  account_code: string;
  amount_milli: number;
  vat_milli: number;
  method: string;
  paid_from_source: string;
  cashbank_id: number | null;
  petty_id: number | null;
  vendor: string;
  reference: string;
  notes: string;
  attachment_required: number;
  approval_status: string;
  created_by: string;
  created_at: string;
  journal_id: number | null;
}

export interface Renewal {
  id: number;
  name: string;
  category: string;
  authority: string;
  issue_date: string;
  expiry_date: string;
  cost_milli: number;
  responsible: string;
  alert_days: number;
  status: string;
  notes: string;
}

export interface DashboardStats {
  total_customers: number;
  total_products: number;
  total_employees: number;
  total_invoices: number;
  revenue_milli: number;
  expenses_milli: number;
  pending_invoices: number;
  overdue_amount: number;
  inventory_value: number;
  low_stock_count: number;
  production_today: number;
  waste_today: number;
  custody_total: number;
  bank_balance: number;
  recent_invoices: SalesInvoice[];
  top_customers: { name: string; total: number }[];
  production_trend: { date: string; good: number; waste: number }[];
  sales_trend: { date: string; amount: number }[];
}

export interface DailyBrief {
  unpaid_count: number;
  unpaid_total: number;
  overdue_total: number;
  waste_yesterday: number;
  last_backup_days: number;
  backup_status: 'green' | 'amber' | 'red';
  overdue_customers: { name: string; amount: number; days: number }[];
  low_stock: { name: string; on_hand: number; reorder: number }[];
  oldest_custodies: { name: string; balance: number; negative: boolean }[];
  renewals_due: { name: string; expiry: string }[];
}

export interface LowStockItem {
  id: number;
  code: string;
  name_ar: string;
  name_en: string;
  kind: string;
  uom: string;
  qty_on_hand: number;
  reorder_level: number;
  deficit: number;
  avg_cost_milli: number;
  value_gap_milli: number;
}

export interface AgingBucket {
  customer_id: number;
  customer_name: string;
  invoice_count: number;
  oldest_days: number;
  bucket_0_30: number;
  bucket_31_60: number;
  bucket_61_90: number;
  bucket_91_plus: number;
  total: number;
}

export interface TrialBalanceRow {
  account_code: string;
  account_name: string;
  account_type: string;
  debit_milli: number;
  credit_milli: number;
}

export interface PettyCashTransaction {
  id: number;
  ts: string;
  petty_id: number;
  ttype: string;
  debit_milli: number;
  credit_milli: number;
  balance_milli: number;
  category: string;
  account_code: string;
  cashbank_id: number | null;
  expense_id: number | null;
  attachment_status: string;
  reference: string;
  notes: string;
  journal_id: number | null;
  user_id: number;
}

export interface CustodyStatement {
  entries: PettyCashTransaction[];
  balance: number;
}

export interface PayrollRun {
  id: number;
  run_no: string;
  period_start: string;
  period_end: string;
  status: string;
  total_gross_milli: number;
  total_deductions_milli: number;
  total_net_milli: number;
  created_by: string;
  created_at: string;
  processed_by: string;
  processed_at: string;
  approved_by: string;
  approved_at: string;
  paid_by: string;
  paid_at: string;
}

export interface PayrollRunLine {
  id: number;
  run_id: number;
  employee_id: number;
  employee_name?: string;
  basic_milli: number;
  allowance_milli: number;
  overtime_milli: number;
  bonus_milli: number;
  deduction_milli: number;
  advance_deduction_milli: number;
  insurance_deduction_milli: number;
  tax_deduction_milli: number;
  net_milli: number;
  notes: string;
}

export interface Attachment {
  id: number;
  entity_type: string;
  entity_id: number;
  original_filename: string;
  stored_filename: string;
  mime_type: string;
  size_bytes: number;
  uploaded_by: string;
  uploaded_at: string;
  notes: string;
}

export function milliToNumber(milli: number): number {
  return milli / 1000;
}
