import { memo } from "react";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import {
  LayoutDashboard, FileText, Users, Package, Factory,
  Calculator, UserCog, ClipboardList, Wrench, BarChart3,
  Settings, ChevronLeft, ChevronRight, Banknote,
  Truck, Shield, Database, FileSearch, TrendingUp,
  Receipt, CreditCard, Landmark, BookOpen,
  Cog, Eye, ShoppingCart, Wallet, Coins, ListChecks,
  ClipboardCheck, RefreshCw, Bell,
  HandCoins, FileClock, FileWarning, Clock,
  Building2, ScrollText, IdCard, Globe,
  Ship, ArrowLeftRight, Sparkles
} from "lucide-react";
import LanguageSwitcher from "./LanguageSwitcher";

interface SidebarProps {
  collapsed: boolean;
  onToggle: () => void;
  currentPath: string;
}

const menuSections = (t: (key: string) => string) => [
  {
    title: t("nav.dashboard"),
    items: [
      { label: t("nav.dashboard"), icon: LayoutDashboard, path: '/dashboard' },
      { label: t("dashboard.dailyBrief"), icon: FileSearch, path: '/dashboard/daily-brief' },
      { label: t("nav.alerts"), icon: Bell, path: '/alerts' },
    ],
  },
  {
    title: t("nav.sales"),
    items: [
      { label: t("nav.invoices"), icon: FileText, path: '/invoices' },
      { label: t("nav.customers"), icon: Users, path: '/customers' },
      { label: t("nav.suppliers"), icon: Truck, path: '/suppliers' },
      { label: t("nav.purchases"), icon: ShoppingCart, path: '/purchases' },
    ],
  },
  {
    title: t("inventory.title"),
    items: [
      { label: t("nav.products"), icon: Package, path: '/products' },
      { label: t("nav.inventory"), icon: Database, path: '/inventory' },
      { label: t("inventory.transfer"), icon: RefreshCw, path: '/stock-transfers' },
      { label: t("common.bom"), icon: ListChecks, path: '/bom' },
      { label: t("production.liveProduction"), icon: TrendingUp, path: '/live-production' },
      { label: t("production.orders"), icon: Factory, path: '/production' },
    ],
  },
  {
    title: t("accounting.title"),
    items: [
      { label: t("accounting.accounts"), icon: BookOpen, path: '/accounting/accounts' },
      { label: t("accounting.journal"), icon: Receipt, path: '/accounting/journal' },
      { label: t("accounting.trialBalance"), icon: Calculator, path: '/accounting/trial-balance' },
      { label: t("reports.title"), icon: TrendingUp, path: '/accounting/statements' },
      { label: t("nav.expenses"), icon: Wallet, path: '/expenses' },
      { label: t("nav.cashBankAccounts"), icon: Landmark, path: '/cashbank' },
      { label: t("nav.pettyCash"), icon: Coins, path: '/petty-cash' },
      { label: t("nav.cheques"), icon: FileWarning, path: '/cheques' },
      { label: t("nav.operatingAdvances"), icon: HandCoins, path: '/operating-advances' },
    ],
  },
  {
    title: t("nav.hr"),
    items: [
      { label: t("hr.employees"), icon: UserCog, path: '/hr/employees' },
      { label: t("nav.payroll"), icon: Banknote, path: '/payroll' },
      { label: t("nav.overtime"), icon: Clock, path: '/overtime' },
      { label: t("nav.employeeAdvances"), icon: HandCoins, path: '/employee-advances' },
      { label: t("nav.operations"), icon: ClipboardList, path: '/operations' },
      { label: t("nav.maintenance"), icon: Wrench, path: '/maintenance' },
      { label: t("nav.machines"), icon: Cog, path: '/machines' },
      { label: t("nav.quality"), icon: Eye, path: '/quality' },
    ],
  },
  {
    title: t("nav.government"),
    items: [
      { label: t("nav.governmentPortal"), icon: Building2, path: '/government' },
      { label: t("nav.ministryOfLabour"), icon: ScrollText, path: '/government/labour' },
      { label: t("nav.residencyPassports"), icon: IdCard, path: '/government/residency' },
      { label: t("nav.governmentIntegration"), icon: Globe, path: '/government/integrations' },
    ],
  },
  {
    title: t("nav.imports"),
    items: [
      { label: t("nav.shipmentTracking"), icon: Ship, path: '/imports' },
      { label: t("nav.barter"), icon: ArrowLeftRight, path: '/barter' },
      { label: t("nav.installments"), icon: CreditCard, path: '/installments' },
    ],
  },
  {
    title: t("nav.reports"),
    items: [
      { label: t("nav.reports"), icon: BarChart3, path: '/reports' },
    ],
  },
  {
    title: t("nav.tools"),
    items: [
      { label: t("nav.scan"), icon: FileSearch, path: '/tools/ocr' },
      { label: t("nav.aiAssistant"), icon: TrendingUp, path: '/tools/ai' },
      { label: t("nav.aiFileImport"), icon: Sparkles, path: '/tools/ai-file-import' },
      { label: t("nav.historicalImport"), icon: Database, path: '/tools/historical-import' },
      { label: t("nav.excelImport"), icon: Database, path: '/tools/excel-import' },
      { label: t("nav.einvoice"), icon: Receipt, path: '/tools/einvoice' },
      { label: t("nav.backup"), icon: Database, path: '/tools/backup' },
      { label: t("nav.integrations"), icon: CreditCard, path: '/tools/integrations' },
    ],
  },
  {
    title: t("nav.settings"),
    items: [
      { label: t("nav.settings"), icon: Settings, path: '/settings' },
      { label: t("nav.userManagement"), icon: Shield, path: '/settings/users' },
      { label: t("nav.renewals"), icon: FileClock, path: '/renewals' },
      { label: t("nav.auditLog"), icon: ClipboardCheck, path: '/audit-log' },
    ],
  },
];

const Sidebar = memo(function Sidebar({ collapsed, onToggle, currentPath }: SidebarProps) {
  const navigate = useNavigate();
  const { t } = useTranslation();

  const isActive = (path: string) => {
    if (path === '/dashboard' && (currentPath === '/' || currentPath === '/dashboard')) return true;
    return currentPath.startsWith(path) && path !== '/';
  };

  return (
    <aside
      role="navigation"
      aria-label={t("common.mainMenu")}
      className="fixed right-0 top-0 h-full bg-surface-900/80 backdrop-blur-xl border-l border-surface-700/50 z-40 flex flex-col"
      style={{ width: collapsed ? '72px' : 'var(--sidebar-width, 260px)', transition: 'width 0.3s cubic-bezier(0.4, 0, 0.2, 1)' }}
    >
      <div className="h-16 flex items-center px-4 border-b border-surface-700/50">
        {!collapsed ? (
          <div className="flex items-center justify-between w-full cursor-pointer" onClick={() => navigate('/')}>
            <div className="flex items-center gap-3">
              <div className="w-9 h-9 rounded-xl bg-gradient-to-br from-brand-700 to-brand-900 flex items-center justify-center shadow-glow" aria-hidden="true">
                <Factory className="w-5 h-5 text-gold-400" />
              </div>
              <div>
                <h1 className="text-sm font-bold text-white leading-none">PRO MAX OS</h1>
                <p className="text-[10px] text-gold-400 font-medium">بروماكس</p>
              </div>
            </div>
            <LanguageSwitcher />
          </div>
        ) : (
          <div className="flex flex-col items-center gap-1">
            <div className="w-9 h-9 rounded-xl bg-gradient-to-br from-brand-700 to-brand-900 flex items-center justify-center shadow-glow mx-auto cursor-pointer" onClick={() => navigate('/')} aria-label={t("common.home")}>
              <Factory className="w-5 h-5 text-gold-400" aria-hidden="true" />
            </div>
            <LanguageSwitcher />
          </div>
        )}
      </div>

      <nav className="flex-1 overflow-y-auto py-3 px-2 space-y-1" aria-label={t("common.navigation")}>
        {menuSections(t).map((section) => (
          <div key={section.title} role="group" aria-label={section.title}>
            {!collapsed && (
              <div className="sidebar-section-title" aria-hidden="true">{section.title}</div>
            )}
            {section.items.map((item) => {
              const active = isActive(item.path);
              const Icon = item.icon;
              return (
                <button
                  key={item.path}
                  onClick={() => navigate(item.path)}
                  aria-current={active ? "page" : undefined}
                  aria-label={item.label}
                  className={`sidebar-link w-full ${active ? 'active' : ''} ${collapsed ? 'justify-center px-2' : ''}`}
                  title={collapsed ? item.label : undefined}
                >
                  <Icon className="w-[18px] h-[18px] flex-shrink-0" aria-hidden="true" />
                  {!collapsed && <span>{item.label}</span>}
                </button>
              );
            })}
          </div>
        ))}
      </nav>

      <div className="p-2 border-t border-surface-700/50">
        <button
          onClick={onToggle}
          aria-label={collapsed ? t("common.expandMenu") : t("common.collapseMenu")}
          aria-expanded={!collapsed}
          className="btn-ghost w-full flex items-center justify-center gap-2 py-2"
        >
          {collapsed ? <ChevronLeft className="w-4 h-4" aria-hidden="true" /> : <ChevronRight className="w-4 h-4" aria-hidden="true" />}
          {!collapsed && <span className="text-xs">{t("common.close")}</span>}
        </button>
      </div>
    </aside>
  );
});

export default Sidebar;
