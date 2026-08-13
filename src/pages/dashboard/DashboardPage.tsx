import { useState, useEffect, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import Card, { StatCard } from "@/components/ui/Card";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@/lib/tauri";
import {
  Users, Package, FileText, TrendingUp, TrendingDown,
  Factory, AlertTriangle, Banknote, Warehouse, RefreshCw,
  ShoppingCart, BarChart3, PieChart as PieIcon, Calendar
} from "lucide-react";
import {
  AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip,
  ResponsiveContainer, BarChart, Bar, PieChart, Pie, Cell, Legend
} from "recharts";
import { useUIStore } from "@/stores/uiStore";

interface TrendPoint { date: string; amount: number; }
interface ProdTrendPoint { date: string; good: number; waste: number; }
interface MonthlyProdPoint { month: string; cartons: number; cups: number; }
interface TopCustomerPoint { name: string; total: number; }
interface CategoryAmountPoint { category: string; amount: number; }
interface DashboardData {
  total_customers: number; total_products: number; total_invoices: number;
  revenue_milli: number; expenses_milli: number; total_employees: number;
  inventory_value: number; overdue_amount: number; low_stock_count: number;
  pending_invoices: number; production_today: number; waste_today: number;
  custody_total: number; bank_balance: number;
  sales_trend: TrendPoint[]; production_trend: ProdTrendPoint[];
  monthly_production: MonthlyProdPoint[]; top_customers: TopCustomerPoint[];
  expenses_by_category: CategoryAmountPoint[];
}
interface LiveData {
  today_total_cartons: number; morning_shift_cartons: number;
  evening_shift_cartons: number; today_total_cups: number;
  recent_entries: Array<{ product_name?: string; product_id?: number; cartons_produced?: number; }>;
}

const PIE_COLORS = ["#4c1d95", "#d4af37", "#8b5cf6", "#6366f1", "#ec4899", "#10b981", "#f59e0b", "#ef4444", "#3b82f6", "#64748b"];

const ChartTooltipStyle = { backgroundColor: "#1e293b", border: "1px solid #334155", borderRadius: "12px", color: "#f8fafc", fontSize: "13px" };

export default function DashboardPage() {
  const navigate = useNavigate();
  const addNotification = useUIStore((s) => s.addNotification);
  const [stats, setStats] = useState<DashboardData | null>(null);
  const [liveData, setLiveData] = useState<LiveData | null>(null);
  const [loading, setLoading] = useState(true);

  const loadDashboard = useCallback(async () => {
    setLoading(true);
    try {
      const [statsData, liveResult] = await Promise.all([
        invoke("get_dashboard_stats") as Promise<DashboardData>,
        invoke("get_live_dashboard").catch(() => null) as Promise<LiveData | null>
      ]);
      setStats(statsData);
      if (liveResult) setLiveData(liveResult);
    } catch {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء تحميل البيانات" });
    } finally {
      setLoading(false);
    }
  }, [addNotification]);

  useEffect(() => { loadDashboard(); }, [loadDashboard]);

  const salesTrend = stats?.sales_trend?.map((d) => ({
    name: d.date?.slice(5) || d.date,
    amount: Math.round(d.amount / 1000)
  })) || [];

  const productionTrend = stats?.production_trend?.map((d) => ({
    name: d.date?.slice(5) || d.date,
    good: d.good,
    waste: d.waste
  })) || [];

  const monthlyProd = stats?.monthly_production?.map((d) => ({
    name: d.month?.slice(5) || d.month,
    كرتون: d.cartons,
    أكواب: Math.round(d.cups / 1000)
  })) || [];

  const topCustomers = stats?.top_customers?.map((d) => ({
    name: d.name?.length > 12 ? d.name.slice(0, 12) + "..." : d.name,
    المبيعات: Math.round(d.total / 1000)
  })) || [];

  const expensesPie = stats?.expenses_by_category?.map((d) => ({
    name: d.category,
    value: Math.round(d.amount / 1000)
  })) || [];

  if (loading) {
    return (
      <div className="flex items-center justify-center h-96">
        <div className="flex flex-col items-center gap-4">
          <div className="w-14 h-14 border-2 border-brand-700 border-t-gold-400 rounded-full animate-spin" />
          <p className="text-surface-400 text-sm">جاري تحميل لوحة التحكم...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-8">
      {/* Header */}
      <div className="page-header">
        <div>
          <h1 className="page-title">لوحة التحكم</h1>
          <p className="page-subtitle">نظرة شاملة على أداء المصنع والإنتاج</p>
        </div>
        <div className="flex items-center gap-3">
          <button onClick={loadDashboard} className="btn-ghost flex items-center gap-2 text-sm">
            <RefreshCw className="w-4 h-4" /> تحديث
          </button>
          <button onClick={() => navigate("/dashboard/daily-brief")} className="btn-primary">
            الموجز اليومي
          </button>
        </div>
      </div>

      {/* Stats Row 1 - Financial */}
      <div className="grid grid-cols-4 gap-5">
        <StatCard title="إجمالي الإيرادات" value={formatOMR(stats?.revenue_milli || 0)} icon={<TrendingUp className="w-6 h-6" />} />
        <StatCard title="المصروفات" value={formatOMR(stats?.expenses_milli || 0)} icon={<TrendingDown className="w-6 h-6" />} />
        <StatCard title="الفواتير المعلقة" value={stats?.pending_invoices || 0} subtitle={formatOMR(stats?.overdue_amount || 0) + " معلق"} icon={<FileText className="w-6 h-6" />} />
        <StatCard title="المخزون منخفض" value={stats?.low_stock_count || 0} subtitle={formatOMR(stats?.inventory_value || 0) + " قيمة"} icon={<AlertTriangle className="w-6 h-6" />} />
      </div>

      {/* Stats Row 2 - Operational */}
      <div className="grid grid-cols-4 gap-5">
        <StatCard title="العملاء" value={stats?.total_customers || 0} icon={<Users className="w-6 h-6" />} />
        <StatCard title="المنتجات" value={stats?.total_products || 0} icon={<Package className="w-6 h-6" />} />
        <StatCard title="الإنتاج اليوم" value={`${((liveData?.today_total_cartons as number) || stats?.production_today || 0).toFixed(0)} كرتون`} icon={<Factory className="w-6 h-6" style={{ color: liveData?.today_total_cartons ? "#fbbf24" : undefined }} />} />
        <StatCard title="أرصدة البنوك" value={formatOMR(stats?.bank_balance || 0)} icon={<Banknote className="w-6 h-6" />} />
      </div>

      {/* Charts Row 1: Sales + Production Daily */}
      <div className="grid grid-cols-2 gap-6">
        {/* Sales Trend - 30 days */}
        <Card>
          <h3 className="section-title">
            <TrendingUp className="w-5 h-5 text-gold-400" />
            حركة المبيعات — آخر 30 يوم
          </h3>
          <div className="h-72">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={salesTrend}>
                <defs>
                  <linearGradient id="salesGrad" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#d4af37" stopOpacity={0.3} />
                    <stop offset="95%" stopColor="#d4af37" stopOpacity={0} />
                  </linearGradient>
                </defs>
                <CartesianGrid strokeDasharray="3 3" stroke="#1e293b" />
                <XAxis dataKey="name" stroke="#64748b" fontSize={11} tickLine={false} />
                <YAxis stroke="#64748b" fontSize={11} tickLine={false} axisLine={false} />
                <Tooltip contentStyle={ChartTooltipStyle} formatter={(value: number) => [formatOMR(value * 1000), "المبيعات"]} />
                <Area type="monotone" dataKey="amount" stroke="#d4af37" strokeWidth={2.5} fill="url(#salesGrad)" dot={false} activeDot={{ r: 5, fill: "#d4af37" }} />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </Card>

        {/* Production Daily - 7 days */}
        <Card>
          <h3 className="section-title">
            <Factory className="w-5 h-5 text-gold-400" />
            الإنتاج اليومي — آخر 7 أيام
          </h3>
          <div className="h-72">
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={productionTrend.slice(-7)}>
                <CartesianGrid strokeDasharray="3 3" stroke="#1e293b" />
                <XAxis dataKey="name" stroke="#64748b" fontSize={11} tickLine={false} />
                <YAxis stroke="#64748b" fontSize={11} tickLine={false} axisLine={false} />
                <Tooltip contentStyle={ChartTooltipStyle} />
                <Legend wrapperStyle={{ fontSize: "12px", color: "#94a3b8" }} />
                <Bar dataKey="good" fill="#4c1d95" radius={[6, 6, 0, 0]} name="صالح" />
                <Bar dataKey="waste" fill="#ef4444" radius={[6, 6, 0, 0]} name="هالك" />
              </BarChart>
            </ResponsiveContainer>
          </div>
        </Card>
      </div>

      {/* Charts Row 2: Monthly Production + Top Customers + Expenses Pie */}
      <div className="grid grid-cols-3 gap-6">
        {/* Monthly Production */}
        <Card>
          <h3 className="section-title">
            <Calendar className="w-5 h-5 text-gold-400" />
            الإنتاج الشهري
          </h3>
          <div className="h-72">
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={monthlyProd}>
                <CartesianGrid strokeDasharray="3 3" stroke="#1e293b" />
                <XAxis dataKey="name" stroke="#64748b" fontSize={11} tickLine={false} />
                <YAxis stroke="#64748b" fontSize={11} tickLine={false} axisLine={false} />
                <Tooltip contentStyle={ChartTooltipStyle} />
                <Legend wrapperStyle={{ fontSize: "12px", color: "#94a3b8" }} />
                <Bar dataKey="كرتون" fill="#4c1d95" radius={[6, 6, 0, 0]} />
                <Bar dataKey="أكواب" fill="#d4af37" radius={[6, 6, 0, 0]} />
              </BarChart>
            </ResponsiveContainer>
          </div>
        </Card>

        {/* Top Customers */}
        <Card>
          <h3 className="section-title">
            <ShoppingCart className="w-5 h-5 text-gold-400" />
            أفضل 5 عملاء
          </h3>
          <div className="h-72">
            {topCustomers.length > 0 ? (
              <ResponsiveContainer width="100%" height="100%">
                <BarChart data={topCustomers} layout="vertical">
                  <CartesianGrid strokeDasharray="3 3" stroke="#1e293b" horizontal={false} />
                  <XAxis type="number" stroke="#64748b" fontSize={11} tickLine={false} />
                  <YAxis type="category" dataKey="name" stroke="#64748b" fontSize={10} tickLine={false} width={90} />
                  <Tooltip contentStyle={ChartTooltipStyle} formatter={(value: number) => [formatOMR(value * 1000), "المبيعات"]} />
                  <Bar dataKey="المبيعات" fill="#d4af37" radius={[0, 6, 6, 0]} barSize={18} />
                </BarChart>
              </ResponsiveContainer>
            ) : (
              <div className="flex items-center justify-center h-full text-surface-500 text-sm">لا توجد بيانات بعد</div>
            )}
          </div>
        </Card>

        {/* Expenses by Category */}
        <Card>
          <h3 className="section-title">
            <PieIcon className="w-5 h-5 text-gold-400" />
            المصروفات حسب التصنيف
          </h3>
          <div className="h-72">
            {expensesPie.length > 0 ? (
              <ResponsiveContainer width="100%" height="100%">
                <PieChart>
                  <Pie
                    data={expensesPie}
                    cx="50%"
                    cy="50%"
                    innerRadius={55}
                    outerRadius={90}
                    paddingAngle={3}
                    dataKey="value"
                    nameKey="name"
                  >
                    {expensesPie.map((_, index) => (
                      <Cell key={`cell-${index}`} fill={PIE_COLORS[index % PIE_COLORS.length]} />
                    ))}
                  </Pie>
                  <Tooltip contentStyle={ChartTooltipStyle} formatter={(value: number) => [formatOMR(value * 1000), "المبلغ"]} />
                  <Legend wrapperStyle={{ fontSize: "11px", color: "#94a3b8" }} />
                </PieChart>
              </ResponsiveContainer>
            ) : (
              <div className="flex items-center justify-center h-full text-surface-500 text-sm">لا توجد مصروفات</div>
            )}
          </div>
        </Card>
      </div>

      {/* Live Production Widget */}
      {liveData && (
        <Card className="relative overflow-hidden">
          <div className="absolute top-0 left-0 w-full h-1 bg-gradient-to-l from-gold-400 via-brand-500 to-gold-400" />
          <div className="flex items-center justify-between mb-5">
            <h3 className="section-title mb-0">
              <Factory className="w-5 h-5 text-gold-400" />
              الإنتاج المباشر — اليوم
            </h3>
            <button onClick={() => navigate("/live-production")} className="text-xs text-brand-400 hover:text-gold-400 transition-colors font-medium">
              فتح الإنتاج المباشر ←
            </button>
          </div>
          <div className="grid grid-cols-4 gap-5">
            <div className="text-center p-4 bg-surface-800/50 rounded-2xl border border-surface-700/30">
              <p className="text-3xl font-bold text-white">{liveData.today_total_cartons?.toFixed(0) || 0}</p>
              <p className="text-xs text-surface-400 mt-2">إجمالي الكرتون</p>
            </div>
            <div className="text-center p-4 bg-surface-800/50 rounded-2xl border border-amber-500/20">
              <p className="text-3xl font-bold text-amber-400">{liveData.morning_shift_cartons?.toFixed(0) || 0}</p>
              <p className="text-xs text-surface-400 mt-2">الوردية الصباحية</p>
            </div>
            <div className="text-center p-4 bg-surface-800/50 rounded-2xl border border-indigo-500/20">
              <p className="text-3xl font-bold text-indigo-400">{liveData.evening_shift_cartons?.toFixed(0) || 0}</p>
              <p className="text-xs text-surface-400 mt-2">الوردية المسائية</p>
            </div>
            <div className="text-center p-4 bg-surface-800/50 rounded-2xl border border-gold-500/20">
              <p className="text-3xl font-bold gradient-text">{(liveData.today_total_cups || 0).toLocaleString()}</p>
              <p className="text-xs text-surface-400 mt-2">إجمالي الأكواب</p>
            </div>
          </div>
          {liveData.recent_entries?.length > 0 && (
            <div className="mt-4 p-3 bg-surface-800/30 rounded-xl border border-surface-700/20">
              <p className="text-xs text-surface-400">
                آخر تسجيل: <span className="text-white font-medium">{liveData.recent_entries[0]?.product_name || `منتج #${liveData.recent_entries[0]?.product_id}`}</span>
                {" — "}
                <span className="text-gold-400 font-mono">{liveData.recent_entries[0]?.cartons_produced?.toFixed(0)} كرتون</span>
              </p>
            </div>
          )}
        </Card>
      )}

      {/* Quick Actions */}
      <div className="grid grid-cols-5 gap-5">
        {[
          { label: "فاتورة جديدة", icon: FileText, path: "/invoices/new", color: "brand" },
          { label: "إنتاج مباشر", icon: Factory, path: "/live-production", color: "amber" },
          { label: "أمر إنتاج", icon: Package, path: "/production/new", color: "brand" },
          { label: "المخزون", icon: Warehouse, path: "/reports/low-stock", color: "brand" },
          { label: "الموجز اليومي", icon: BarChart3, path: "/dashboard/daily-brief", color: "brand" },
        ].map(({ label, icon: Icon, path, color }) => (
          <button key={path} onClick={() => navigate(path)} className="card-hover flex flex-col items-center gap-3 py-7 group">
            <div className={`w-14 h-14 rounded-2xl ${color === "amber" ? "bg-amber-500/20 border border-amber-500/30" : "bg-brand-800/30 border border-brand-500/20"} flex items-center justify-center group-hover:shadow-glow transition-all`}>
              <Icon className={`w-7 h-7 ${color === "amber" ? "text-amber-400" : "text-brand-400"} group-hover:text-gold-400 transition-colors`} />
            </div>
            <span className="text-sm font-medium text-surface-300 group-hover:text-white transition-colors">{label}</span>
          </button>
        ))}
      </div>
    </div>
  );
}
