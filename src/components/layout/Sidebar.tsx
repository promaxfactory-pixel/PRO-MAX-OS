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
  ClipboardCheck, RefreshCw, Bell, HandCoins, FileClock, FileWarning, Clock,
  Building2, ScrollText, IdCard, Globe,
  Ship, ArrowLeftRight, Sun, Moon, Sparkles, Undo2
} from "lucide-react";
import LanguageSwitcher from "./LanguageSwitcher";

interface SidebarProps {
  collapsed: boolean;
  onToggle: () => void;
  currentPath: string;
}

const sectionColors: Record<string, string> = {
  "nav.dashboard": "#8b5cf6",
  "nav.sales": "#10b981",
  "inventory.title": "#06b6d4",
  "accounting.title": "#d4af37",
  "nav.hr": "#f43f5e",
  "الحكومة": "#3b82f6",
  "الاستيراد والتعاملات": "#f97316",
  "nav.reports": "#a855f7",
  "nav.tools": "#64748b",
  "nav.settings": "#6b7280",
};

const sectionIcons: Record<string, React.ElementType> = {
  "nav.dashboard": LayoutDashboard,
  "nav.sales": ShoppingCart,
  "inventory.title": Package,
  "accounting.title": BookOpen,
  "nav.hr": Users,
  "الحكومة": Building2,
  "الاستيراد والتعاملات": Ship,
  "nav.reports": BarChart3,
  "nav.tools": Wrench,
  "nav.settings": Settings,
};

const menuSections = (t: (key: string) => string) => [
  {
    title: t("nav.dashboard"),
    items: [
      { label: t("nav.dashboard"), icon: LayoutDashboard, path: "/dashboard" },
      { label: t("dashboard.dailyBrief"), icon: FileSearch, path: "/dashboard/daily-brief" },
      { label: t("nav.alerts"), icon: Bell, path: "/alerts" },
    ],
  },
  {
    title: t("nav.sales"),
    items: [
      { label: t("nav.invoices"), icon: FileText, path: "/invoices" },
      { label: "الإشعارات الدائنة", icon: Undo2, path: "/credit-notes" },
      { label: t("nav.customers"), icon: Users, path: "/customers" },
      { label: t("nav.suppliers"), icon: Truck, path: "/suppliers" },
      { label: t("nav.purchases"), icon: ShoppingCart, path: "/purchases" },
    ],
  },
  {
    title: t("inventory.title"),
    items: [
      { label: t("nav.products"), icon: Package, path: "/products" },
      { label: t("nav.inventory"), icon: Database, path: "/inventory" },
      { label: t("inventory.transfer"), icon: RefreshCw, path: "/stock-transfers" },
      { label: "BOM", icon: ListChecks, path: "/bom" },
      { label: t("production.liveProduction"), icon: TrendingUp, path: "/live-production" },
      { label: t("production.orders"), icon: Factory, path: "/production" },
    ],
  },
  {
    title: t("accounting.title"),
    items: [
      { label: t("accounting.accounts"), icon: BookOpen, path: "/accounting/accounts" },
      { label: t("accounting.journal"), icon: Receipt, path: "/accounting/journal" },
      { label: t("accounting.trialBalance"), icon: Calculator, path: "/accounting/trial-balance" },
      { label: t("reports.title"), icon: TrendingUp, path: "/accounting/statements" },
      { label: t("nav.expenses"), icon: Wallet, path: "/expenses" },
      { label: "الحسابات النقدية والبنكية", icon: Landmark, path: "/cashbank" },
      { label: "العهد والصرف النثري", icon: Coins, path: "/custody" },
      { label: "الشيكات", icon: FileWarning, path: "/cheques" },
    ],
  },
  {
    title: t("nav.hr"),
    items: [
      { label: t("hr.employees"), icon: UserCog, path: "/hr/employees" },
      { label: t("nav.payroll"), icon: Banknote, path: "/payroll" },
      { label: "العمل الإضافي", icon: Clock, path: "/overtime" },
      { label: " سلف الموظفين", icon: HandCoins, path: "/employee-advances" },
      { label: t("nav.operations"), icon: ClipboardList, path: "/operations" },
      { label: t("nav.maintenance"), icon: Wrench, path: "/maintenance" },
      { label: t("nav.machines"), icon: Cog, path: "/machines" },
      { label: t("nav.quality"), icon: Eye, path: "/quality" },
    ],
  },
  {
    title: "الحكومة",
    items: [
      { label: "بوابة الحكومة", icon: Building2, path: "/government" },
      { label: "وزارة العمل", icon: ScrollText, path: "/government/labour" },
      { label: "الإقامة والجوازات", icon: IdCard, path: "/government/residency" },
      { label: "التكامل الحكومي", icon: Globe, path: "/government/integrations" },
    ],
  },
  {
    title: "الاستيراد والتعاملات",
    items: [
      { label: "تتبع الشحنات", icon: Ship, path: "/imports" },
      { label: "المبادلة والمقايضة", icon: ArrowLeftRight, path: "/barter" },
      { label: "الأقساط والقروض", icon: CreditCard, path: "/installments" },
    ],
  },
  {
    title: t("nav.reports"),
    items: [
      { label: t("nav.reports"), icon: BarChart3, path: "/reports" },
    ],
  },
  {
    title: t("nav.tools"),
    items: [
      { label: "مسح ضوئي", icon: FileSearch, path: "/tools/ocr" },
      { label: "مساعد الذكاء الاصطناعي", icon: TrendingUp, path: "/tools/ai" },
      { label: "استيراد الملفات بالذكاء الاصطناعي", icon: Sparkles, path: "/tools/ai-file-import" },
      { label: "استيراد تاريخي", icon: Database, path: "/tools/historical-import" },
      { label: "استيراد Excel", icon: Database, path: "/tools/excel-import" },
      { label: "الفوترة الإلكترونية — فاوترة (عمان)", icon: Receipt, path: "/tools/einvoice" },
      { label: "فاتورة - السعودية (المرحلة 2)", icon: Globe, path: "/tools/zatca2" },
      { label: "قيد - إيداع XBRL (الكويت)", icon: FileText, path: "/tools/qayd" },
      { label: "الفروع والمزامنة", icon: Building2, path: "/settings/branches" },
      { label: "النسخ الاحتياطي", icon: Database, path: "/tools/backup" },
      { label: "التكاملات", icon: CreditCard, path: "/tools/integrations" },
    ],
  },
  {
    title: t("nav.settings"),
    items: [
      { label: "الإعدادات", icon: Settings, path: "/settings" },
      { label: "إدارة المستخدمين", icon: Shield, path: "/settings/users" },
      { label: "التجديدات", icon: FileClock, path: "/renewals" },
      { label: "سجل التدقيق", icon: ClipboardCheck, path: "/audit-log" },
    ],
  },
];

const Sidebar = memo(function Sidebar({ collapsed, onToggle, currentPath }: SidebarProps) {
  const navigate = useNavigate();
  const { t, i18n } = useTranslation();
  const isDark = document.documentElement.getAttribute("data-theme") !== "light";
  const isRtl = i18n.language === "ar" || i18n.language === "ur";

  const sidebarPosition = isRtl ? "right-0 border-l" : "left-0 border-r";

  const isActive = (path: string) => {
    if (path === "/dashboard" && (currentPath === "/" || currentPath === "/dashboard")) return true;
    return currentPath.startsWith(path) && path !== "/";
  };

  const toggleTheme = () => {
    const current = document.documentElement.getAttribute("data-theme");
    const next = current === "light" ? "dark" : "light";
    document.documentElement.setAttribute("data-theme", next);
    localStorage.setItem("promax-theme", next);
  };

  return (
    <aside
      role="navigation"
      aria-label="القائمة الرئيسية"
      className={`fixed top-0 h-full z-40 flex flex-col ${sidebarPosition}`}
      style={{
        width: collapsed ? "var(--sidebar-collapsed-width)" : "var(--sidebar-width)",
        background: "var(--surface-sidebar)",
        borderColor: "var(--border)",
        transition: "width 0.3s cubic-bezier(0.4, 0, 0.2, 1)",
      }}
    >
      <div
        className="h-16 flex items-center px-4 border-b cursor-pointer"
        style={{ borderColor: "var(--border)" }}
        onClick={() => navigate("/")}
      >
        {!collapsed ? (
          <div className="flex items-center justify-between w-full">
            <div className="flex items-center gap-3">
              <div
                className="w-9 h-9 rounded-xl flex items-center justify-center"
                style={{
                  background: "linear-gradient(135deg, var(--brand-500), var(--brand-700))",
                }}
              >
                <Factory className="w-5 h-5" style={{ color: "var(--brand-gold)" }} />
              </div>
              <div>
                <h1 className="text-sm font-bold leading-none" style={{ color: "var(--text-primary)" }}>PRO MAX OS</h1>
                <p className="text-[10px] font-medium" style={{ color: "var(--brand-gold)" }}>بروماكس</p>
              </div>
            </div>
            <LanguageSwitcher />
          </div>
        ) : (
          <div className="flex flex-col items-center gap-1 w-full">
            <div
              className="w-9 h-9 rounded-xl flex items-center justify-center mx-auto"
              style={{
                background: "linear-gradient(135deg, var(--brand-500), var(--brand-700))",
              }}
              aria-label="الصفحة الرئيسية"
            >
              <Factory className="w-5 h-5" style={{ color: "var(--brand-gold)" }} />
            </div>
          </div>
        )}
      </div>

      <nav className="flex-1 overflow-y-auto py-3 px-2 space-y-1" aria-label="التنقل">
        {menuSections(t).map((section) => {
          const color = sectionColors[section.title] || "#8b5cf6";
          const SectionIcon = sectionIcons[section.title];
          const hasActive = section.items.some(i => isActive(i.path));

          return (
            <div key={section.title} role="group" aria-label={section.title}>
              {!collapsed && (
                <div
                  className="sidebar-section-title flex items-center gap-2"
                  style={{ opacity: hasActive ? 1 : 0.6 }}
                >
                  {SectionIcon && (
                    <SectionIcon className="w-3 h-3" style={{ color }} />
                  )}
                  <span>{section.title}</span>
                </div>
              )}
              {section.items.map((item) => {
                const active = isActive(item.path);
                const Icon = item.icon;

                if (collapsed) {
                  return (
                    <div key={item.path} className="relative group">
                      <button
                        onClick={() => navigate(item.path)}
                        aria-current={active ? "page" : undefined}
                        aria-label={item.label}
                        className="sidebar-link w-full justify-center px-0 py-2.5"
                        style={{
                          color: active ? "var(--brand-500)" : "var(--text-muted)",
                          background: active ? "color-mix(in srgb, var(--brand-500) 12%, transparent)" : "transparent",
                        }}
                      >
                        <Icon className="w-[18px] h-[18px] flex-shrink-0" aria-hidden="true" />
                      </button>
                      <div
                        className={`absolute top-1/2 -translate-y-1/2 px-2.5 py-1.5 rounded-lg text-xs whitespace-nowrap pointer-events-none opacity-0 group-hover:opacity-100 transition-opacity z-50 ${isRtl ? "right-full mr-2" : "left-full ml-2"}`}
                        style={{
                          background: "var(--surface-elevated)",
                          color: "var(--text-primary)",
                          border: "1px solid var(--border)",
                          boxShadow: "var(--shadow-elevated)",
                        }}
                      >
                        {item.label}
                      </div>
                    </div>
                  );
                }

                return (
                  <button
                    key={item.path}
                    onClick={() => navigate(item.path)}
                    aria-current={active ? "page" : undefined}
                    aria-label={item.label}
                    className="sidebar-link w-full"
                    style={{
                      color: active ? "var(--brand-500)" : "var(--text-muted)",
                      background: active ? "color-mix(in srgb, var(--brand-500) 12%, transparent)" : "transparent",
                      fontWeight: active ? 600 : 500,
                    }}
                  >
                    <Icon className="w-[18px] h-[18px] flex-shrink-0" aria-hidden="true" />
                    <span className="truncate">{item.label}</span>
                    {active && (
                      <span
                        className={`w-1.5 h-1.5 rounded-full ${isRtl ? "mr-auto" : "ml-auto"}`}
                        style={{ background: "var(--brand-500)", boxShadow: "0 0 6px var(--brand-500)" }}
                      />
                    )}
                  </button>
                );
              })}
            </div>
          );
        })}
      </nav>

      <div
        className="p-3 border-t"
        style={{ borderColor: "var(--border)" }}
      >
        {!collapsed ? (
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-1">
              <button
                onClick={toggleTheme}
                className="p-2 rounded-lg transition-colors"
                style={{ color: "var(--text-muted)" }}
                onMouseEnter={e => e.currentTarget.style.background = "color-mix(in srgb, var(--border) 40%, transparent)"}
                onMouseLeave={e => e.currentTarget.style.background = "transparent"}
                aria-label="تبديل الثيم"
              >
                {isDark ? <Sun className="w-4 h-4" /> : <Moon className="w-4 h-4" />}
              </button>
              <button
                onClick={() => navigate("/settings")}
                className="p-2 rounded-lg transition-colors"
                style={{ color: "var(--text-muted)" }}
                onMouseEnter={e => e.currentTarget.style.background = "color-mix(in srgb, var(--border) 40%, transparent)"}
                onMouseLeave={e => e.currentTarget.style.background = "transparent"}
                aria-label="الإعدادات"
              >
                <Settings className="w-4 h-4" />
              </button>
            </div>
            <button
              onClick={onToggle}
              className="p-2 rounded-lg transition-colors"
              style={{ color: "var(--text-muted)" }}
              onMouseEnter={e => e.currentTarget.style.background = "color-mix(in srgb, var(--border) 40%, transparent)"}
              onMouseLeave={e => e.currentTarget.style.background = "transparent"}
              aria-label="طي القائمة"
            >
              <ChevronRight className={`w-4 h-4 ${isRtl ? "" : "rotate-180"}`} />
            </button>
          </div>
        ) : (
          <div className="flex flex-col items-center gap-1">
            <button
              onClick={toggleTheme}
              className="p-2 rounded-lg transition-colors"
              style={{ color: "var(--text-muted)" }}
              onMouseEnter={e => e.currentTarget.style.background = "color-mix(in srgb, var(--border) 40%, transparent)"}
              onMouseLeave={e => e.currentTarget.style.background = "transparent"}
              aria-label="تبديل الثيم"
            >
              {isDark ? <Sun className="w-4 h-4" /> : <Moon className="w-4 h-4" />}
            </button>
            <button
              onClick={onToggle}
              className="p-2 rounded-lg transition-colors"
              style={{ color: "var(--text-muted)" }}
              onMouseEnter={e => e.currentTarget.style.background = "color-mix(in srgb, var(--border) 40%, transparent)"}
              onMouseLeave={e => e.currentTarget.style.background = "transparent"}
              aria-label="توسيع القائمة"
            >
              <ChevronLeft className={`w-4 h-4 ${isRtl ? "" : "rotate-180"}`} />
            </button>
          </div>
        )}
      </div>
    </aside>
  );
});

export default Sidebar;
