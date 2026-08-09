import { useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import { BarChart3, Clock, Package, FileText, Users, TrendingUp, Receipt, Truck, Calendar, DollarSign, AlertTriangle } from "lucide-react";
import { useTranslation } from "react-i18next";

interface ReportCard {
  title: string;
  description: string;
  icon: React.ReactNode;
  path: string;
  color: string;
}

export default function ReportsPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();

  const reports: ReportCard[] = [
    {
      title: t("reports.dailyClosingTitle"),
      description: t("reports.dailyClosingDesc"),
      icon: <Calendar className="w-6 h-6" />,
      path: "/reports/daily-closing",
      color: "text-emerald-400",
    },
    {
      title: t("reports.ownerSummary"),
      description: t("reports.ownerSummaryDesc"),
      icon: <DollarSign className="w-6 h-6" />,
      path: "/reports/owner-summary",
      color: "text-gold-400",
    },
    {
      title: t("reports.profitMargin"),
      description: t("reports.inventoryMarginSubtitle"),
      icon: <TrendingUp className="w-6 h-6" />,
      path: "/reports/inventory-margin",
      color: "text-purple-400",
    },
    {
      title: t("reports.salesByCustomer"),
      description: t("reports.salesByCustomerDesc"),
      icon: <Users className="w-6 h-6" />,
      path: "/reports/sales-by-customer",
      color: "text-blue-400",
    },
    {
      title: t("reports.unpaidInvoicesTitle"),
      description: t("reports.unpaidInvoicesDesc"),
      icon: <AlertTriangle className="w-6 h-6" />,
      path: "/reports/unpaid-invoices",
      color: "text-gold-400",
    },
    {
      title: t("reports.agingTitle"),
      description: t("reports.agingDesc"),
      icon: <Clock className="w-6 h-6" />,
      path: "/reports/aging",
      color: "text-amber-400",
    },
    {
      title: t("reports.lowStock"),
      description: t("reports.lowStockDesc"),
      icon: <Package className="w-6 h-6" />,
      path: "/reports/low-stock",
      color: "text-red-400",
    },
    {
      title: t("reports.vatReturnTitle"),
      description: t("reports.vatReturnDesc"),
      icon: <Receipt className="w-6 h-6" />,
      path: "/reports/vat-return",
      color: "text-emerald-400",
    },
    {
      title: t("accounting.incomeStatement"),
      description: t("reports.incomeStatementDesc"),
      icon: <TrendingUp className="w-6 h-6" />,
      path: "/accounting/statements",
      color: "text-purple-400",
    },
    {
      title: t("accounting.trialBalance"),
      description: t("reports.trialBalanceDesc"),
      icon: <BarChart3 className="w-6 h-6" />,
      path: "/accounting/trial-balance",
      color: "text-cyan-400",
    },
    {
      title: t("reports.purchasesReport"),
      description: t("reports.purchasesReportDesc"),
      icon: <Truck className="w-6 h-6" />,
      path: "/purchases",
      color: "text-orange-400",
    },
    {
      title: t("reports.customReports"),
      description: t("reports.customReportsDesc"),
      icon: <FileText className="w-6 h-6" />,
      path: "/reports",
      color: "text-brand-400",
    },
  ];

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">{t("reports.title")}</h1>
          <p className="page-subtitle">{t("reports.pageSubtitle")}</p>
        </div>
      </div>

      <div className="grid grid-cols-3 gap-4">
        {reports.map((report) => (
          <Card
            key={report.path}
            className="cursor-pointer hover:border-brand-500/50 transition-all duration-300 group"
            onClick={() => navigate(report.path)}
          >
            <div className="flex items-start gap-4">
              <div className={`p-3 rounded-xl bg-surface-800/80 ${report.color} group-hover:scale-110 transition-transform`}>
                {report.icon}
              </div>
              <div>
                <h3 className="font-bold text-white group-hover:text-brand-300 transition-colors">{report.title}</h3>
                <p className="text-sm text-surface-400 mt-1">{report.description}</p>
              </div>
            </div>
          </Card>
        ))}
      </div>
    </div>
  );
}
