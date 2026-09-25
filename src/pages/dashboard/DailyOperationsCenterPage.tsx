import { useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import { invoke } from "@/lib/tauri";
import { formatOMR } from "@/lib/utils";
import {
  Wallet, Landmark, Receipt, Users, Truck, Factory, Package,
  Clock, Banknote, FileText, AlertTriangle, ShoppingCart, UserCog,
  Wrench, ClipboardCheck, RefreshCw, ArrowLeft
} from "lucide-react";

interface DashboardStats {
  revenue_milli: number;
  expenses_milli: number;
  overdue_amount: number;
  inventory_value: number;
  low_stock_count: number;
  custody_total: number;
  bank_balance: number;
  pending_invoices: number;
}
interface LiveProduction {
  today_total_cartons: number;
  today_total_cups: number;
  morning_shift_cartons: number;
  evening_shift_cartons: number;
}

const actions = [
  { label: "تسجيل مصروف", desc: "عهدة / مالك / بنك / نقدي", path: "/expenses", icon: Receipt },
  { label: "العهدة والصرف النثري", desc: "استلام، صرف، تسوية ورصيد", path: "/custody", icon: Wallet },
  { label: "تحصيل من عميل", desc: "مديونيات ومدفوعات العملاء", path: "/customers", icon: Users },
  { label: "الموردون والمشتريات", desc: "فاتورة شراء، خامات، مدفوعات", path: "/purchases", icon: Truck },
  { label: "إنتاج الوردية", desc: "صنف + عامل + وردية + تالف", path: "/live-production", icon: Factory },
  { label: "المخزون", desc: "خامات وإنتاج تام وحركات", path: "/inventory", icon: Package },
  { label: "الرواتب", desc: "مسير راتب ومصدر الدفع", path: "/payroll", icon: Banknote },
  { label: "الأوفر تايم", desc: "الساعات الإضافية والاعتماد", path: "/overtime", icon: Clock },
  { label: "ملفات العاملين", desc: "جواز، إقامة، تأشيرة، تصريح", path: "/hr/employees", icon: UserCog },
  { label: "الصيانة والتشغيل", desc: "معدات وأعطال وتكاليف", path: "/maintenance", icon: Wrench },
  { label: "الفواتير والمبيعات", desc: "بيع، رصيد عميل، تحصيل", path: "/invoices", icon: FileText },
  { label: "الإقفال والتقارير", desc: "مراجعة يومية وقوائم وتقارير", path: "/reports/daily-closing", icon: ClipboardCheck },
];

export default function DailyOperationsCenterPage() {
  const navigate = useNavigate();
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [live, setLive] = useState<LiveProduction | null>(null);
  const [loading, setLoading] = useState(true);

  const load = async () => {
    setLoading(true);
    try {
      const [s, p] = await Promise.all([
        invoke<DashboardStats>("get_dashboard_stats"),
        invoke<LiveProduction>("get_live_dashboard").catch(() => null),
      ]);
      setStats(s);
      setLive(p);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => { load(); }, []);

  const attention = useMemo(() => [
    { label: "مديونيات متأخرة", value: formatOMR(stats?.overdue_amount || 0), path: "/reports/aging" },
    { label: "فواتير معلقة", value: String(stats?.pending_invoices || 0), path: "/reports/unpaid-invoices" },
    { label: "أصناف منخفضة", value: String(stats?.low_stock_count || 0), path: "/reports/low-stock" },
  ], [stats]);

  return (
    <div className="space-y-6" dir="rtl">
      <div className="page-header">
        <div>
          <h1 className="page-title">مركز الإدارة اليومية</h1>
          <p className="page-subtitle">شاشة العمل الرئيسية للمحاسبة والمالية وتشغيل المصنع</p>
        </div>
        <button onClick={load} className="btn-outline flex items-center gap-2">
          <RefreshCw className="w-4 h-4" />
          تحديث
        </button>
      </div>

      <div className="grid grid-cols-2 xl:grid-cols-5 gap-4">
        <Card><p className="text-xs text-surface-400">رصيد العهدة</p><p className="text-xl font-bold mt-2">{formatOMR(stats?.custody_total || 0)}</p></Card>
        <Card><p className="text-xs text-surface-400">البنوك والنقدية</p><p className="text-xl font-bold mt-2">{formatOMR(stats?.bank_balance || 0)}</p></Card>
        <Card><p className="text-xs text-surface-400">المصروفات</p><p className="text-xl font-bold mt-2">{formatOMR(stats?.expenses_milli || 0)}</p></Card>
        <Card><p className="text-xs text-surface-400">إنتاج اليوم</p><p className="text-xl font-bold mt-2">{(live?.today_total_cartons || 0).toFixed(0)} كرتون</p></Card>
        <Card><p className="text-xs text-surface-400">قيمة المخزون</p><p className="text-xl font-bold mt-2">{formatOMR(stats?.inventory_value || 0)}</p></Card>
      </div>

      <Card>
        <div className="flex items-center gap-2 mb-4">
          <AlertTriangle className="w-5 h-5 text-amber-400" />
          <h2 className="font-bold">يحتاج انتباهك</h2>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
          {attention.map((item) => (
            <button key={item.label} onClick={() => navigate(item.path)} className="p-4 rounded-xl border border-surface-700/50 bg-surface-800/40 text-right hover:border-brand-500/50 transition-colors">
              <p className="text-xs text-surface-400">{item.label}</p>
              <p className="text-lg font-bold mt-1">{item.value}</p>
            </button>
          ))}
        </div>
      </Card>

      <div>
        <h2 className="font-bold text-lg mb-3">الإدخال اليومي السريع</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
          {actions.map(({ label, desc, path, icon: Icon }) => (
            <button key={path} onClick={() => navigate(path)} className="card-hover text-right p-5 flex items-center gap-4">
              <div className="w-11 h-11 rounded-xl bg-brand-800/30 border border-brand-500/20 flex items-center justify-center">
                <Icon className="w-5 h-5 text-brand-400" />
              </div>
              <div className="min-w-0 flex-1">
                <p className="font-bold">{label}</p>
                <p className="text-xs text-surface-400 mt-1">{desc}</p>
              </div>
              <ArrowLeft className="w-4 h-4 text-surface-500" />
            </button>
          ))}
        </div>
      </div>

      <Card>
        <h2 className="font-bold mb-3">تسلسل الإقفال اليومي المقترح</h2>
        <div className="grid grid-cols-1 lg:grid-cols-4 gap-3 text-sm">
          {[
            "1. تأكيد كل المقبوضات والمصروفات ومصدر كل مبلغ.",
            "2. مراجعة الوردية: العامل، الصنف، الكمية، التالف، المشرف.",
            "3. مراجعة الخامات والإنتاج التام وحركات المخزون.",
            "4. مراجعة العملاء والموردين والرواتب والتنبيهات والمستندات."
          ].map((x) => <div key={x} className="p-3 rounded-xl bg-surface-800/40 border border-surface-700/40">{x}</div>)}
        </div>
        {loading && <p className="text-xs text-surface-500 mt-3">جاري تحديث مؤشرات اليوم...</p>}
      </Card>
    </div>
  );
}
