mod db;
mod commands;
pub mod mcp;
pub mod crypto;
pub mod error;
pub mod validation;
mod zatca;
pub mod zatca2;
pub mod qayd;
pub mod fawtara;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let app_handle = app.handle().clone();
            let db_path = {
                let data_dir = app_handle
                    .path()
                    .app_data_dir()
                    .expect("failed to get app data dir");
                std::fs::create_dir_all(&data_dir).ok();
                data_dir.join("promax.db")
            };
            let conn = db::init_database(&db_path)
                .expect("failed to initialize database");
            crypto::init_secrets(&db_path);
            app.manage(db::DbState(std::sync::Mutex::new(conn)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Auth
            commands::auth::login,
            commands::auth::get_current_user,
            commands::auth::change_password,
            commands::auth::validate_token,
            // Dashboard
            commands::dashboard::get_dashboard_stats,
            commands::dashboard::get_daily_brief,
            // Customers
            commands::customers::list_customers,
            commands::customers::get_customer,
            commands::customers::create_customer,
            commands::customers::update_customer,
            commands::customers::delete_customer,
            commands::customers::get_customer_statement,
            commands::customers::create_customer_payment,
            // Invoices
            commands::invoices::list_invoices,
            commands::invoices::get_invoice,
            commands::invoices::create_invoice,
            commands::invoices::update_invoice,
            commands::invoices::post_invoice,
            commands::invoices::void_invoice,
            commands::invoices::duplicate_invoice,
            commands::invoices::get_invoice_lines,
            commands::invoices::get_invoice_for_print,
            commands::invoices::get_invoice_for_print_customs,
            commands::invoices::get_receipt_for_print,
            commands::invoices::get_delivery_note_for_print,
            commands::invoices::get_credit_note_for_print,
            commands::invoices::create_credit_note,
            commands::invoices::list_credit_notes,
            commands::invoices::get_invoice_credit_remaining,
            commands::invoices::get_supplier_receipt_for_print,
            // Products
            commands::products::list_products,
            commands::products::list_products_for_select,
            commands::products::get_product,
            commands::products::create_product,
            commands::products::update_product,
            commands::products::delete_product,
            // Inventory
            commands::inventory::list_inventory_items,
            commands::inventory::get_inventory_item,
            commands::inventory::create_inventory_item,
            commands::inventory::update_inventory_item,
            commands::inventory::get_inventory_movements,
            commands::inventory::adjust_stock,
            // Production
            commands::production::list_production_orders,
            commands::production::get_production_order,
            commands::production::create_production_order,
            commands::production::update_production_order,
            commands::production::approve_production_order,
            commands::production::get_production_lines,
            commands::production::add_production_line,
            // Accounting
            commands::accounting::list_accounts,
            commands::accounting::get_account,
            commands::accounting::create_account,
            commands::accounting::list_journal_entries,
            commands::accounting::get_journal_entry_lines,
            commands::accounting::create_journal_entry,
            commands::accounting::get_trial_balance,
            commands::accounting::get_balance_sheet,
            commands::accounting::get_income_statement,
            // Custody (petty cash legacy)
            commands::custody::list_custody_accounts,
            commands::custody::get_custody_account,
            commands::custody::create_custody_fund,
            commands::custody::create_custody_spend,
            commands::custody::create_custody_transfer,
            commands::custody::get_custody_statement,
            commands::custody::update_custody_spend,
            commands::custody::update_custody_fund,
            // HR
            commands::hr::list_employees,
            commands::hr::get_employee,
            commands::hr::create_employee,
            commands::hr::update_employee,
            commands::hr::delete_employee,
            commands::hr::list_employees_for_production,
            // Operations
            commands::operations::list_operations_sheets,
            commands::operations::get_operations_sheet,
            commands::operations::create_operations_sheet,
            // Maintenance
            commands::maintenance::list_maintenance_sheets,
            commands::maintenance::get_maintenance_sheet,
            commands::maintenance::create_maintenance_sheet,
            // Suppliers
            commands::suppliers::list_suppliers,
            commands::suppliers::get_supplier,
            commands::suppliers::create_supplier,
            commands::suppliers::update_supplier,
            commands::suppliers::delete_supplier,
            commands::suppliers::get_supplier_statement,
            // Purchases
            commands::purchases::list_purchases,
            commands::purchases::get_purchase,
            commands::purchases::create_purchase,
            commands::purchases::get_purchase_lines,
            commands::purchases::post_purchase,
            commands::purchases::void_purchase,
            commands::purchases::list_suppliers_for_select,
            commands::purchases::create_supplier_payment,
            // Expenses
            commands::expenses::list_expenses,
            commands::expenses::create_expense,
            commands::expenses::reimburse_expense,
            commands::expenses::approve_expense,
            commands::expenses::list_employees_for_select,
            commands::expenses::get_custody_accounts_for_select,
            // Petty Cash
            commands::petty_cash::list_petty_cash_accounts,
            commands::petty_cash::create_petty_cash_account,
            // Cashbank
            commands::cashbank::list_cashbank_accounts,
            commands::cashbank::create_cashbank_account,
            // Cheques
            commands::cheques::list_cheques,
            commands::cheques::create_cheque,
            // Renewals
            commands::renewals::list_renewals,
            commands::renewals::create_renewal,
            // Machines
            commands::machines::list_machines,
            commands::machines::get_machine,
            commands::machines::create_machine,
            commands::machines::update_machine,
            commands::machines::record_temperature,
            commands::machines::get_machine_temperatures,
            commands::machines::get_live_machine_temps,
            // Quality
            commands::quality::list_quality_inspections,
            commands::quality::create_quality_inspection,
            // BOM
            commands::bom::list_boms,
            commands::bom::create_bom,
            // Stock Transfers
            commands::stock_transfers::list_stock_transfers,
            commands::stock_transfers::create_stock_transfer,
            commands::stock_transfers::list_warehouses,
            // Payroll
            commands::payroll::list_payroll_runs,
            commands::payroll::create_payroll_run,
            commands::payroll::list_employee_advances,
            commands::payroll::create_employee_advance,
            // Overtime
            commands::overtime::list_overtime_records,
            commands::overtime::create_overtime_record,
            commands::overtime::approve_overtime,
            commands::overtime::reject_overtime,
            // Alerts
            commands::alerts::get_all_alerts,
            // Licensing
            commands::licensing::check_license,
            commands::licensing::activate_license,
            commands::licensing::get_license_info,
            commands::licensing::deactivate_license,
            commands::licensing::verify_developer_pin,
            commands::licensing::generate_license_key,
            commands::licensing::generate_tier_license,
            commands::licensing::get_tier_features,
            commands::licensing::list_tiers,
            // Reports
            commands::reports::low_stock_report,
            commands::reports::customers_aging,
            commands::reports::sales_report,
            commands::reports::production_report,
            commands::reports::vat_return,
            commands::reports::daily_factory_closing,
            commands::reports::owner_summary,
            commands::reports::inventory_margin_report,
            commands::reports::sales_by_customer_report,
            commands::reports::unpaid_invoices_report,
            // RBAC & Audit
            commands::rbac::list_audit_logs,
            // Settings
            commands::settings::get_company_settings,
            commands::settings::update_company_settings,
            commands::settings::list_users,
            commands::settings::create_user,
            commands::settings::update_user,
            commands::settings::reset_user_password,
            commands::settings::delete_user,
            // OCR
            commands::ocr::ocr_extract_from_file,
            commands::ocr::ocr_parse_invoice,
            commands::ocr::ocr_get_history,
            commands::ocr::ocr_save_scan,
            commands::ocr::ocr_enhance_with_ai,
            commands::ocr::ocr_get_suggestions,
            commands::ocr::ocr_create_invoice,
            commands::ocr::ocr_add_supplier,
            commands::ocr::ocr_register_expense,
            commands::ocr::ocr_update_prices,
            commands::ocr::ocr_detect_language,
            // AI
            commands::ai::ai_sales_forecast,
            commands::ai::ai_customer_risk,
            commands::ai::ai_production_analysis,
            commands::ai::ai_cost_analysis,
            commands::ai::ai_dashboard_insights,
            commands::ai::ai_inventory_optimization,
            commands::ai::ai_anomaly_detection,
            commands::ai::ai_generate_report,
            // Excel Import
            commands::excel_import::excel_read_preview,
            commands::excel_import::excel_list_sheets,
            commands::excel_import::excel_import_journal,
            commands::excel_import::excel_import_customers,
            commands::excel_import::excel_import_products,
            commands::excel_import::excel_import_inventory,
            commands::excel_import::excel_analyze_data,
            commands::excel_import::excel_get_import_history,
            // E-Invoice
            commands::einvoice::einvoice_generate,
            commands::einvoice::einvoice_validate,
            commands::einvoice::einvoice_get_status,
            commands::einvoice::einvoice_list,
            commands::einvoice::einvoice_mark_submitted,
            commands::einvoice::einvoice_summary_report,
            commands::einvoice::einvoice_cancel,
            commands::einvoice::einvoice_submit,
            commands::einvoice::einvoice_add_to_queue,
            commands::einvoice::einvoice_process_queue,
            commands::einvoice::einvoice_get_dashboard,
            commands::einvoice::einvoice_get_settings,
            commands::einvoice::einvoice_save_settings,
            commands::einvoice::einvoice_get_queue,
            commands::einvoice::einvoice_retry_queue_item,
            commands::einvoice::einvoice_get_xml,
            commands::einvoice::einvoice_bulk_generate,
            // Fawtara (Oman e-invoicing foundation)
            fawtara::fawtara_build_payload,
            fawtara::fawtara_readiness,
            fawtara::fawtara_connector_status,
            fawtara::fawtara_submit,
            // Backup
            commands::backup::backup_create,
            commands::backup::backup_restore,
            commands::backup::backup_list,
            commands::backup::backup_auto,
            commands::backup::backup_export_csv,
            // AI Assistant
            commands::ai_assistant::save_ai_settings,
            commands::ai_assistant::get_ai_settings,
            commands::ai_assistant::ai_chat,
            commands::ai_assistant::chat_with_ai,
            commands::ai_assistant::test_ai_connection,
            commands::ai_assistant::save_ai_provider_settings,
            commands::ai_assistant::ai_analyze_entity,
            commands::ai_assistant::ai_suggest_actions,
            commands::ai_assistant::ai_chat_with_provider,
            commands::ai_providers::ai_provider_catalog,
            commands::ai_providers::ai_provider_statuses,
            commands::ai_providers::ai_get_provider_settings,
            commands::ai_providers::ai_save_provider_config,
            commands::ai_providers::ai_test_provider,
            commands::ai_providers::ai_failover_chat,
            commands::ai_providers::ai_get_available_models,
            commands::ai_file_import::ai_analyze_document,
            commands::ai_file_import::ai_list_extractions,
            commands::ai_file_import::ai_get_extraction,
            commands::ai_file_import::ai_delete_extraction,
            commands::ai_file_import::ai_update_extraction,
            commands::ai_file_import::ai_commit_extraction,
            commands::ai_file_import::ai_duplicate_check,
            // Historical Import
            commands::historical_import::preview_import,
            commands::historical_import::execute_import,
            commands::historical_import::get_import_templates,
            commands::historical_import::import_get_history,
            // Integrations
            commands::integrations::integrations_get_settings,
            commands::integrations::integrations_save_settings,
            commands::integrations::integrations_test_whatsapp,
            commands::integrations::integrations_test_email,
            commands::integrations::integrations_test_printer,
            // File Reader
            commands::file_reader::file_read_text,
            commands::file_reader::file_read_spreadsheet,
            commands::file_reader::file_read_docx,
            commands::file_reader::file_read_any,
            commands::file_reader::file_get_info,
            // Device (Printer & Scanner)
            commands::device::list_printers,
            commands::device::print_html,
            commands::device::print_thermal,
            commands::device::list_scanners,
            commands::device::scan_document,
            // Live Production Shift
            commands::production_shift::get_shift_sheet,
            commands::production_shift::record_production,
            commands::production_shift::get_shift_lines,
            commands::production_shift::complete_shift,
            commands::production_shift::update_production_line,
            commands::production_shift::delete_production_line,
            commands::production_shift::get_live_dashboard,
            commands::production_shift::print_shift_report_thermal,
            commands::production_shift::get_worker_daily_report,
            commands::production_shift::record_shift_inventory_snapshot,
            commands::production_shift::get_shift_inventory_snapshots,
            // Production Reports
            commands::reports::get_daily_production_report,
            commands::reports::get_monthly_production_report,
            commands::reports::get_comprehensive_daily_report,
            // Government Integration
            commands::government::gov_get_dashboard,
            commands::government::gov_list_entities,
            commands::government::gov_list_submissions,
            commands::government::gov_get_employee_doc_status,
            commands::government::gov_submit_report,
            // Import Tracking
            commands::import_tracking::list_shipments,
            commands::import_tracking::get_shipment,
            commands::import_tracking::create_shipment,
            commands::import_tracking::update_shipment,
            commands::import_tracking::update_shipment_status,
            // Barter Exchange
            commands::barter_exchange::list_barter_exchanges,
            commands::barter_exchange::create_barter_exchange,
            commands::barter_exchange::get_barter_balance,
            // Installment Payments
            commands::installment_payments::list_installment_payments,
            commands::installment_payments::list_installments,
            commands::installment_payments::create_installment_payment,
            commands::installment_payments::mark_installment_paid,
            commands::installment_payments::get_installment_summary,
            // Approvals
            commands::approvals::list_approval_requests,
            commands::approvals::create_approval_request,
            commands::approvals::decide_approval,
            commands::approvals::get_approval_summary,
            // Budget Planning
            commands::budget::list_budgets,
            commands::budget::get_budget,
            commands::budget::get_budget_lines,
            commands::budget::create_budget,
            commands::budget::approve_budget,
            commands::budget::update_budget_actuals,
            commands::budget::get_budget_vs_actual,
            // Fixed Assets
            commands::assets::list_assets,
            commands::assets::get_asset,
            commands::assets::create_asset,
            commands::assets::list_asset_maintenance,
            commands::assets::create_asset_maintenance,
            commands::assets::get_asset_register_summary,
            commands::assets::calculate_depreciation,
            // Notifications
            commands::notifications::list_notifications,
            commands::notifications::create_notification,
            commands::notifications::mark_notification_read,
            commands::notifications::mark_all_notifications_read,
            commands::notifications::get_notification_count,
            // Operating Advances
            commands::operating_advance::list_operating_advances,
            commands::operating_advance::get_operating_advance,
            commands::operating_advance::create_operating_advance,
            commands::operating_advance::approve_advance,
            commands::operating_advance::reject_advance,
            commands::operating_advance::disburse_advance,
            commands::operating_advance::record_advance_spend,
            commands::operating_advance::submit_receipt,
            commands::operating_advance::approve_receipt,
            commands::operating_advance::return_advance,
            commands::operating_advance::reconcile_advance,
            commands::operating_advance::cancel_advance,
            commands::operating_advance::get_advance_transactions,
            commands::operating_advance::get_advance_receipts,
            commands::operating_advance::get_advance_summary,
            commands::operating_advance::list_pending_receipts,
            // ZATCA Phase 2
            commands::zatca2::zatca2_get_settings,
            commands::zatca2::zatca2_save_settings,
            commands::zatca2::zatca2_build_csr,
            commands::zatca2::zatca2_onboard,
            commands::zatca2::zatca2_generate,
            commands::zatca2::zatca2_validate,
            commands::zatca2::zatca2_submit,
            commands::zatca2::zatca2_list,
            // Qayd XBRL
            commands::qayd::qayd_generate_filing,
            commands::qayd::qayd_list_filings,
            commands::qayd::qayd_get_filing,
            commands::qayd::qayd_validate_filing,
            commands::qayd::qayd_delete_filing,
            commands::qayd::qayd_filing_totals,
            // Branches & offline sync
            commands::branches::branches_list,
            commands::branches::branches_create,
            commands::branches::branches_update,
            commands::branches::branches_delete,
            commands::branches::offline_queue_enqueue,
            commands::branches::offline_queue_list,
            commands::branches::offline_queue_mark_synced,
            commands::branches::offline_queue_retry,
            commands::branches::offline_queue_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ProMax ERP");
}
