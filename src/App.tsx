import { Suspense, lazy, useState, useEffect } from "react";
import { Routes, Route, Navigate, useNavigate } from "react-router-dom";
import { useAuthStore } from "./stores/authStore";
import LicenseGate from "./components/layout/LicenseGate";
import AppLayout from "./components/layout/AppLayout";
import LoadingSpinner from "./components/ui/LoadingSpinner";
import ErrorBoundary from "./components/ui/ErrorBoundary";

const LoginPage = lazy(() => import("./pages/auth/LoginPage"));
const DashboardPage = lazy(() => import("./pages/dashboard/DashboardPage"));
const DailyBriefPage = lazy(() => import("./pages/dashboard/DailyBriefPage"));
const CustomerListPage = lazy(() => import("./pages/customers/CustomerListPage"));
const CustomerDetailPage = lazy(() => import("./pages/customers/CustomerDetailPage"));
const CustomerStatementPage = lazy(() => import("./pages/customers/CustomerStatementPage"));
const CustomerFormPage = lazy(() => import("./pages/customers/CustomerFormPage"));
const CustomerPaymentPage = lazy(() => import("./pages/customers/CustomerPaymentPage"));
const InvoiceListPage = lazy(() => import("./pages/invoices/InvoiceListPage"));
const InvoiceCreatePage = lazy(() => import("./pages/invoices/InvoiceCreatePage"));
const InvoiceDetailPage = lazy(() => import("./pages/invoices/InvoiceDetailPage"));
const ProductListPage = lazy(() => import("./pages/inventory/ProductListPage"));
const ProductDetailPage = lazy(() => import("./pages/inventory/ProductDetailPage"));
const ProductFormPage = lazy(() => import("./pages/products/ProductFormPage"));
const InventoryListPage = lazy(() => import("./pages/inventory/InventoryListPage"));
const BOMPage = lazy(() => import("./pages/inventory/BOMPage"));
const StockTransfersPage = lazy(() => import("./pages/inventory/StockTransfersPage"));
const ProductionOrderListPage = lazy(() => import("./pages/production/ProductionOrderListPage"));
const ProductionOrderCreatePage = lazy(() => import("./pages/production/ProductionOrderCreatePage"));
const ProductionOrderDetailPage = lazy(() => import("./pages/production/ProductionOrderDetailPage"));
const LiveProductionPage = lazy(() => import("./pages/production/LiveProductionPage"));
const AccountsPage = lazy(() => import("./pages/accounting/AccountsPage"));
const JournalPage = lazy(() => import("./pages/accounting/JournalPage"));
const TrialBalancePage = lazy(() => import("./pages/accounting/TrialBalancePage"));
const FinancialStatementsPage = lazy(() => import("./pages/accounting/FinancialStatementsPage"));
const AuditLogPage = lazy(() => import("./pages/accounting/AuditLogPage"));
const EmployeeListPage = lazy(() => import("./pages/hr/EmployeeListPage"));
const EmployeeDetailPage = lazy(() => import("./pages/hr/EmployeeDetailPage"));
const EmployeeFormPage = lazy(() => import("./pages/hr/EmployeeFormPage"));
const EmployeeAdvancesPage = lazy(() => import("./pages/hr/EmployeeAdvancesPage"));
const PayrollPage = lazy(() => import("./pages/hr/PayrollPage"));
const OvertimePage = lazy(() => import("./pages/hr/OvertimePage"));
const OperationsSheetListPage = lazy(() => import("./pages/operations/OperationsSheetListPage"));
const OperationsSheetCreatePage = lazy(() => import("./pages/operations/OperationsSheetCreatePage"));
const OperationsSheetDetailPage = lazy(() => import("./pages/operations/OperationsSheetDetailPage"));
const MaintenanceSheetListPage = lazy(() => import("./pages/maintenance/MaintenanceSheetListPage"));
const MaintenanceSheetCreatePage = lazy(() => import("./pages/maintenance/MaintenanceSheetCreatePage"));
const MaintenanceSheetDetailPage = lazy(() => import("./pages/maintenance/MaintenanceSheetDetailPage"));
const MachineListPage = lazy(() => import("./pages/machines/MachineListPage"));
const MachineFormPage = lazy(() => import("./pages/machines/MachineFormPage"));
const QualityListPage = lazy(() => import("./pages/quality/QualityListPage"));
const QualityFormPage = lazy(() => import("./pages/quality/QualityFormPage"));
const ReportsPage = lazy(() => import("./pages/reports/ReportsPage"));
const ReportsAgingPage = lazy(() => import("./pages/reports/ReportsAgingPage"));
const ReportsLowStockPage = lazy(() => import("./pages/reports/ReportsLowStockPage"));
const ReportsVatReturnPage = lazy(() => import("./pages/reports/ReportsVatReturnPage"));
const DailyClosingPage = lazy(() => import("./pages/reports/DailyClosingPage"));
const OwnerSummaryPage = lazy(() => import("./pages/reports/OwnerSummaryPage"));
const InventoryMarginPage = lazy(() => import("./pages/reports/InventoryMarginPage"));
const SalesByCustomerPage = lazy(() => import("./pages/reports/SalesByCustomerPage"));
const UnpaidInvoicesPage = lazy(() => import("./pages/reports/UnpaidInvoicesPage"));
const SupplierListPage = lazy(() => import("./pages/suppliers/SupplierListPage"));
const SupplierDetailPage = lazy(() => import("./pages/suppliers/SupplierDetailPage"));
const SupplierFormPage = lazy(() => import("./pages/suppliers/SupplierFormPage"));
const SupplierStatementPage = lazy(() => import("./pages/suppliers/SupplierStatementPage"));
const SupplierPaymentPage = lazy(() => import("./pages/suppliers/SupplierPaymentPage"));
const PurchaseListPage = lazy(() => import("./pages/purchases/PurchaseListPage"));
const PurchaseCreatePage = lazy(() => import("./pages/purchases/PurchaseCreatePage"));
const ExpensesPage = lazy(() => import("./pages/expenses/ExpensesPage"));
const CashBankPage = lazy(() => import("./pages/cashbank/CashBankPage"));
const PettyCashPage = lazy(() => import("./pages/pettycash/PettyCashPage"));
const ChequesPage = lazy(() => import("./pages/finance/ChequesPage"));
const RenewalsPage = lazy(() => import("./pages/settings/RenewalsPage"));
const GovernmentDashboardPage = lazy(() => import("./pages/government/GovernmentDashboardPage"));
const AlertCenterPage = lazy(() => import("./pages/alerts/AlertCenterPage"));
const SettingsPage = lazy(() => import("./pages/settings/SettingsPage"));
const UsersPage = lazy(() => import("./pages/settings/UsersPage"));
const ChangePasswordPage = lazy(() => import("./pages/settings/ChangePasswordPage"));
const HistoricalImportPage = lazy(() => import("./pages/tools/HistoricalImportPage"));
const AiAssistantPage = lazy(() => import("./pages/tools/AiAssistantPage"));
const AiFileImportPage = lazy(() => import("./pages/tools/AiFileImportPage"));
const EnhancedOcrPage = lazy(() => import("./pages/tools/EnhancedOcrPage"));
const ExcelImportPage = lazy(() => import("./pages/tools/ExcelImportPage"));
const EInvoicePage = lazy(() => import("./pages/tools/EInvoicePage"));
const BackupPage = lazy(() => import("./pages/tools/BackupPage"));
const NotFoundPage = lazy(() => import("./pages/errors/NotFoundPage"));
const ForbiddenPage = lazy(() => import("./pages/errors/ForbiddenPage"));
const IntegrationsPage = lazy(() => import("./pages/tools/IntegrationsPage"));
const ImportTrackingPage = lazy(() => import("./pages/imports/ImportTrackingPage"));
const BarterExchangePage = lazy(() => import("./pages/barter/BarterExchangePage"));
const InstallmentTrackingPage = lazy(() => import("./pages/installments/InstallmentTrackingPage"));

function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { isAuthenticated, validateToken } = useAuthStore();
  const [tokenValid, setTokenValid] = useState(true);
  const navigate = useNavigate();

  useEffect(() => {
    if (!isAuthenticated) {
      setTokenValid(false);
      return;
    }
    validateToken().then((valid) => {
      if (!valid) {
        setTokenValid(false);
        navigate("/login", { replace: true });
      }
    }).catch(() => {
      setTokenValid(false);
      navigate("/login", { replace: true });
    });
  }, [isAuthenticated, validateToken, navigate]);

  if (!isAuthenticated || !tokenValid) return <Navigate to="/login" replace />;
  return <AppLayout>{children}</AppLayout>;
}

export default function App() {
  return (
    <ErrorBoundary>
      <LicenseGate>
        <Suspense fallback={<LoadingSpinner />}>
          <Routes>
            <Route path="/login" element={<LoginPage />} />
            <Route path="/" element={<ProtectedRoute><DashboardPage /></ProtectedRoute>} />
            <Route path="/dashboard" element={<ProtectedRoute><DashboardPage /></ProtectedRoute>} />
            <Route path="/dashboard/daily-brief" element={<ProtectedRoute><DailyBriefPage /></ProtectedRoute>} />
            <Route path="/alerts" element={<ProtectedRoute><AlertCenterPage /></ProtectedRoute>} />

            {/* Customers */}
            <Route path="/customers" element={<ProtectedRoute><CustomerListPage /></ProtectedRoute>} />
            <Route path="/customers/new" element={<ProtectedRoute><CustomerFormPage /></ProtectedRoute>} />
            <Route path="/customers/:id" element={<ProtectedRoute><CustomerDetailPage /></ProtectedRoute>} />
            <Route path="/customers/:id/edit" element={<ProtectedRoute><CustomerFormPage /></ProtectedRoute>} />
            <Route path="/customers/:id/statement" element={<ProtectedRoute><CustomerStatementPage /></ProtectedRoute>} />
            <Route path="/customers/:id/pay" element={<ProtectedRoute><CustomerPaymentPage /></ProtectedRoute>} />

            {/* Invoices */}
            <Route path="/invoices" element={<ProtectedRoute><InvoiceListPage /></ProtectedRoute>} />
            <Route path="/invoices/new" element={<ProtectedRoute><InvoiceCreatePage /></ProtectedRoute>} />
            <Route path="/invoices/:id" element={<ProtectedRoute><InvoiceDetailPage /></ProtectedRoute>} />

            {/* Suppliers */}
            <Route path="/suppliers" element={<ProtectedRoute><SupplierListPage /></ProtectedRoute>} />
            <Route path="/suppliers/new" element={<ProtectedRoute><SupplierFormPage /></ProtectedRoute>} />
            <Route path="/suppliers/:id" element={<ProtectedRoute><SupplierDetailPage /></ProtectedRoute>} />
            <Route path="/suppliers/:id/edit" element={<ProtectedRoute><SupplierFormPage /></ProtectedRoute>} />
            <Route path="/suppliers/:id/statement" element={<ProtectedRoute><SupplierStatementPage /></ProtectedRoute>} />
            <Route path="/suppliers/:id/pay" element={<ProtectedRoute><SupplierPaymentPage /></ProtectedRoute>} />

            {/* Purchases */}
            <Route path="/purchases" element={<ProtectedRoute><PurchaseListPage /></ProtectedRoute>} />
            <Route path="/purchases/new" element={<ProtectedRoute><PurchaseCreatePage /></ProtectedRoute>} />

            {/* Products & Inventory */}
            <Route path="/products" element={<ProtectedRoute><ProductListPage /></ProtectedRoute>} />
            <Route path="/products/new" element={<ProtectedRoute><ProductFormPage /></ProtectedRoute>} />
            <Route path="/products/:id" element={<ProtectedRoute><ProductDetailPage /></ProtectedRoute>} />
            <Route path="/products/:id/edit" element={<ProtectedRoute><ProductFormPage /></ProtectedRoute>} />
            <Route path="/inventory" element={<ProtectedRoute><InventoryListPage /></ProtectedRoute>} />
            <Route path="/bom" element={<ProtectedRoute><BOMPage /></ProtectedRoute>} />
            <Route path="/stock-transfers" element={<ProtectedRoute><StockTransfersPage /></ProtectedRoute>} />

            {/* Production */}
            <Route path="/production" element={<ProtectedRoute><ProductionOrderListPage /></ProtectedRoute>} />
            <Route path="/production/new" element={<ProtectedRoute><ProductionOrderCreatePage /></ProtectedRoute>} />
            <Route path="/production/:id" element={<ProtectedRoute><ProductionOrderDetailPage /></ProtectedRoute>} />
            <Route path="/live-production" element={<ProtectedRoute><LiveProductionPage /></ProtectedRoute>} />

            {/* Accounting */}
            <Route path="/accounting/accounts" element={<ProtectedRoute><AccountsPage /></ProtectedRoute>} />
            <Route path="/accounting/journal" element={<ProtectedRoute><JournalPage /></ProtectedRoute>} />
            <Route path="/accounting/trial-balance" element={<ProtectedRoute><TrialBalancePage /></ProtectedRoute>} />
            <Route path="/accounting/statements" element={<ProtectedRoute><FinancialStatementsPage /></ProtectedRoute>} />
            <Route path="/audit-log" element={<ProtectedRoute><AuditLogPage /></ProtectedRoute>} />

            {/* Finance */}
            <Route path="/expenses" element={<ProtectedRoute><ExpensesPage /></ProtectedRoute>} />
            <Route path="/cashbank" element={<ProtectedRoute><CashBankPage /></ProtectedRoute>} />
            <Route path="/petty-cash" element={<ProtectedRoute><PettyCashPage /></ProtectedRoute>} />
            <Route path="/cheques" element={<ProtectedRoute><ChequesPage /></ProtectedRoute>} />

            {/* HR */}
            <Route path="/hr/employees" element={<ProtectedRoute><EmployeeListPage /></ProtectedRoute>} />
            <Route path="/hr/employees/new" element={<ProtectedRoute><EmployeeFormPage /></ProtectedRoute>} />
            <Route path="/hr/employees/:id" element={<ProtectedRoute><EmployeeDetailPage /></ProtectedRoute>} />
            <Route path="/hr/employees/:id/edit" element={<ProtectedRoute><EmployeeFormPage /></ProtectedRoute>} />
            <Route path="/payroll" element={<ProtectedRoute><PayrollPage /></ProtectedRoute>} />
            <Route path="/overtime" element={<ProtectedRoute><OvertimePage /></ProtectedRoute>} />
            <Route path="/employee-advances" element={<ProtectedRoute><EmployeeAdvancesPage /></ProtectedRoute>} />

            {/* Operations & Maintenance */}
            <Route path="/operations" element={<ProtectedRoute><OperationsSheetListPage /></ProtectedRoute>} />
            <Route path="/operations/new" element={<ProtectedRoute><OperationsSheetCreatePage /></ProtectedRoute>} />
            <Route path="/operations/sheets/:id" element={<ProtectedRoute><OperationsSheetDetailPage /></ProtectedRoute>} />
            <Route path="/maintenance" element={<ProtectedRoute><MaintenanceSheetListPage /></ProtectedRoute>} />
            <Route path="/maintenance/new" element={<ProtectedRoute><MaintenanceSheetCreatePage /></ProtectedRoute>} />
            <Route path="/maintenance/sheets/:id" element={<ProtectedRoute><MaintenanceSheetDetailPage /></ProtectedRoute>} />
            <Route path="/machines" element={<ProtectedRoute><MachineListPage /></ProtectedRoute>} />
            <Route path="/machines/new" element={<ProtectedRoute><MachineFormPage /></ProtectedRoute>} />
            <Route path="/machines/:id/edit" element={<ProtectedRoute><MachineFormPage /></ProtectedRoute>} />
            <Route path="/quality" element={<ProtectedRoute><QualityListPage /></ProtectedRoute>} />
            <Route path="/quality/new" element={<ProtectedRoute><QualityFormPage /></ProtectedRoute>} />

            {/* Reports */}
            <Route path="/reports" element={<ProtectedRoute><ReportsPage /></ProtectedRoute>} />
            <Route path="/reports/aging" element={<ProtectedRoute><ReportsAgingPage /></ProtectedRoute>} />
            <Route path="/reports/low-stock" element={<ProtectedRoute><ReportsLowStockPage /></ProtectedRoute>} />
            <Route path="/reports/vat-return" element={<ProtectedRoute><ReportsVatReturnPage /></ProtectedRoute>} />
            <Route path="/reports/daily-closing" element={<ProtectedRoute><DailyClosingPage /></ProtectedRoute>} />
            <Route path="/reports/owner-summary" element={<ProtectedRoute><OwnerSummaryPage /></ProtectedRoute>} />
            <Route path="/reports/inventory-margin" element={<ProtectedRoute><InventoryMarginPage /></ProtectedRoute>} />
            <Route path="/reports/sales-by-customer" element={<ProtectedRoute><SalesByCustomerPage /></ProtectedRoute>} />
            <Route path="/reports/unpaid-invoices" element={<ProtectedRoute><UnpaidInvoicesPage /></ProtectedRoute>} />

            {/* Import Tracking */}
            <Route path="/imports" element={<ProtectedRoute><ImportTrackingPage /></ProtectedRoute>} />

            {/* Barter Exchange */}
            <Route path="/barter" element={<ProtectedRoute><BarterExchangePage /></ProtectedRoute>} />

            {/* Installments */}
            <Route path="/installments" element={<ProtectedRoute><InstallmentTrackingPage /></ProtectedRoute>} />

            {/* Settings */}
            <Route path="/settings" element={<ProtectedRoute><SettingsPage /></ProtectedRoute>} />
            <Route path="/settings/users" element={<ProtectedRoute><UsersPage /></ProtectedRoute>} />
            <Route path="/settings/change-password" element={<ProtectedRoute><ChangePasswordPage /></ProtectedRoute>} />
            <Route path="/renewals" element={<ProtectedRoute><RenewalsPage /></ProtectedRoute>} />

            {/* Government Integration */}
            <Route path="/government" element={<ProtectedRoute><GovernmentDashboardPage /></ProtectedRoute>} />
            <Route path="/government/labour" element={<ProtectedRoute><GovernmentDashboardPage /></ProtectedRoute>} />
            <Route path="/government/residency" element={<ProtectedRoute><GovernmentDashboardPage /></ProtectedRoute>} />
            <Route path="/government/integrations" element={<ProtectedRoute><GovernmentDashboardPage /></ProtectedRoute>} />

            {/* Tools */}
            <Route path="/tools/ocr" element={<ProtectedRoute><EnhancedOcrPage /></ProtectedRoute>} />
            <Route path="/tools/ai" element={<ProtectedRoute><AiAssistantPage /></ProtectedRoute>} />
            <Route path="/tools/ai-file-import" element={<ProtectedRoute><AiFileImportPage /></ProtectedRoute>} />
            <Route path="/tools/historical-import" element={<ProtectedRoute><HistoricalImportPage /></ProtectedRoute>} />
            <Route path="/tools/excel-import" element={<ProtectedRoute><ExcelImportPage /></ProtectedRoute>} />
            <Route path="/tools/einvoice" element={<ProtectedRoute><EInvoicePage /></ProtectedRoute>} />
            <Route path="/tools/backup" element={<ProtectedRoute><BackupPage /></ProtectedRoute>} />
            <Route path="/tools/integrations" element={<ProtectedRoute><IntegrationsPage /></ProtectedRoute>} />

            <Route path="/403" element={<ProtectedRoute><ForbiddenPage /></ProtectedRoute>} />
            <Route path="*" element={<NotFoundPage />} />
          </Routes>
        </Suspense>
      </LicenseGate>
    </ErrorBoundary>
  );
}
