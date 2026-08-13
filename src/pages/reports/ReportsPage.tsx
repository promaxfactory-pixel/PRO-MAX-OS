import { useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import { BarChart3, Clock, Package, FileText, Users, TrendingUp, Receipt, Truck, Calendar, DollarSign, AlertTriangle } from "lucide-react";

interface ReportCard {
  title: string;
  description: string;
  icon: React.ReactNode;
  path: string;
  color: string;
}

export default function ReportsPage() {
  const navigate = useNavigate();

  const reports: ReportCard[] = [
    {
      title: "الإقفال اليومي للمصنع",
      description: "ملخص شامل لأنشطة اليوم: إنتا̡ مبيعاʡ مشترياʡ تحصيلات",
      icon: <Calendar className="w-6 h-6" />,
      path: "/reports/daily-closing",
      color: "text-emerald-400",
    },
    {
      title: "ملخص المالك",
      description: "نظرة عامة على الأداء المالي وربحية الأعمال",
      icon: <DollarSign className="w-6 h-6" />,
      path: "/reports/owner-summary",
      color: "text-gold-400",
    },
    {
      title: "هامش الربح",
      description: "تحليل هامش الربح لكل منتج في المخزون",
      icon: <TrendingUp className="w-6 h-6" />,
      path: "/reports/inventory-margin",
      color: "text-purple-400",
    },
    {
      title: "المبيعات حسب العميل",
      description: "تحليل المبيعات والفواتير لكل عميل",
      icon: <Users className="w-6 h-6" />,
      path: "/reports/sales-by-customer",
      color: "text-blue-400",
    },
    {
      title: "الفواتير غير المحصلة",
      description: "جميع الفواتير التي لم تُسدد بالكامل",
      icon: <AlertTriangle className="w-6 h-6" />,
      path: "/reports/unpaid-invoices",
      color: "text-gold-400",
    },
    {
      title: "أعمار الذمم",
      description: "تقرير أعمار المستحقات من العملاء",
      icon: <Clock className="w-6 h-6" />,
      path: "/reports/aging",
      color: "text-amber-400",
    },
    {
      title: "المخزون المنخفض",
      description: "الأصناف تحت الحد الأدنى",
      icon: <Package className="w-6 h-6" />,
      path: "/reports/low-stock",
      color: "text-red-400",
    },
    {
      title: "إقرار ضريبة القيمة المضافة",
      description: "تقرير VAT للتقديم للهيئة",
      icon: <Receipt className="w-6 h-6" />,
      path: "/reports/vat-return",
      color: "text-emerald-400",
    },
    {
      title: "قائمة الدخل",
      description: "الإيرادات والمصروفات وصافي الربح",
      icon: <TrendingUp className="w-6 h-6" />,
      path: "/accounting/statements",
      color: "text-purple-400",
    },
    {
      title: "ميزان المراجعة",
      description: "أرصدة الحسابات والمطابقة",
      icon: <BarChart3 className="w-6 h-6" />,
      path: "/accounting/trial-balance",
      color: "text-cyan-400",
    },
    {
      title: "تقرير المشتريات",
      description: "أوامر الشراء والموردين",
      icon: <Truck className="w-6 h-6" />,
      path: "/purchases",
      color: "text-orange-400",
    },
    {
      title: "التقارير المخصصة",
      description: "إنشاء تقرير حسب الطلب",
      icon: <FileText className="w-6 h-6" />,
      path: "/reports",
      color: "text-brand-400",
    },
  ];

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">التقارير</h1>
          <p className="page-subtitle">جميع التقارير المالية والتشغيلية</p>
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
