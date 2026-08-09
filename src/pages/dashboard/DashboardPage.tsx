import { useState, useEffect, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import Card, { StatCard } from "@/components/ui/Card";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
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
import { useTranslation } from "react-i18next";

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

const PIE_COLORS = ['var(--brand-primary)', 'var(--brand-gold)', 'var(--mode-accent)', '#6366f1', '#ec4899', '#10b981', '#f59e0b', '#ef4444', '#3b82f6', '#64748b'];

const ChartTooltipStyle = {
  backgroundColor: 'var(--surface-elevated)',
  border: '1px solid var(--border)',
  borderRadius: '12px',
  color: 'var(--text-primary)',
  fontSize: '13px',
  boxShadow: '0 8px 24px -6px rgba(0,0,0,0.5)',
};

export default function DashboardPage() {
  const { t } = useTranslation();
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
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("dashboard.loadError") });
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
    cartons: d.cartons,
    cups: Math.round(d.cups / 1000)
  })) || [];

  const topCustomers = stats?.top_customers?.map((d) => ({
    name: d.name?.length > 12 ? d.name.slice(0, 12) + '...' : d.name,
    sales: Math.round(d.total / 1000)
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
          <p className="text-surface-400 text-sm">{t("dashboard.loadingDashboard")}</p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-8">
      {/* Header */}
      <div className="page-header">
        <div>
          <h1 className="page-title">{t("dashboard.title")}</h1>
          <p className="page-subtitle">{t("dashboard.subtitle")}</p>
        </div>
        <div className="flex items-center gap-3">
          <button onClick={loadDashboard} className="btn-ghost flex items-center gap-2 text-sm">
            <RefreshCw className="w-4 h-4" /> {t("dashboard.refresh")}
          </button>
          <button onClick={() => navigate('/dashboard/daily-brief')} className="btn-primary">
            {t("dashboard.dailyBrief")}
          </button>
        </div>
      </div>

      {/* Stats Row 1 - Financial */}
      <div className="grid grid-cols-4 gap-5">
        <StatCard title={t("dashboard.totalRevenue")} value={formatOMR(stats?.revenue_milli || 0)} icon={<TrendingUp className="w-6 h-6" />} />
        <StatCard title={t("dashboard.expenses")} value={formatOMR(stats?.expenses_milli || 0)} icon={<TrendingDown className="w-6 h-6" />} />
        <StatCard title={t("dashboard.pendingInvoices")} value={stats?.pending_invoices || 0} subtitle={t("dashboard.overduePending", { amount: formatOMR(stats?.overdue_amount || 0) })} icon={<FileText className="w-6 h-6" />} />
        <StatCard title={t("dashboard.lowStock")} value={stats?.low_stock_count || 0} subtitle={t("dashboard.inventoryValueSuffix", { amount: formatOMR(stats?.inventory_value || 0) })} icon={<AlertTriangle className="w-6 h-6" />} />
      </div>

      {/* Stats Row 2 - Operational */}
      <div className="grid grid-cols-4 gap-5">
        <StatCard title={t("nav.customers")} value={stats?.total_customers || 0} icon={<Users className="w-6 h-6" />} />
        <StatCard title={t("nav.products")} value={stats?.total_products || 0} icon={<Package className="w-6 h-6" />} />
        <StatCard title={t("dashboard.productionToday")} value={t("dashboard.cartonsProduced", { count: `${((liveData?.today_total_cartons as number) || stats?.production_today || 0).toFixed(0)}` })} icon={<Factory className="w-6 h-6" style={{ color: liveData?.today_total_cartons ? '#fbbf24' : undefined }} />} />
        <StatCard title={t("dashboard.bankBalances")} value={formatOMR(stats?.bank_balance || 0)} icon={<Banknote className="w-6 h-6" />} />
      </div>

      {/* Charts Row 1: Sales + Production Daily */}
      <div className="grid grid-cols-2 gap-6">
        {/* Sales Trend - 30 days */}
        <Card>
          <h3 className="section-title">
            <TrendingUp className="w-5 h-5 text-gold-400" />
            {t("dashboard.salesTrendTitle")}
          </h3>
          <div className="h-72">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={salesTrend}>
                <defs>
                  <linearGradient id="salesGrad" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="var(--brand-gold)" stopOpacity={0.3} />
                    <stop offset="95%" stopColor="var(--brand-gold)" stopOpacity={0} />
                  </linearGradient>
                </defs>
                <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" />
                <XAxis dataKey="name" stroke="var(--text-muted)" fontSize={11} tickLine={false} />
                <YAxis stroke="var(--text-muted)" fontSize={11} tickLine={false} axisLine={false} />
                <Tooltip contentStyle={ChartTooltipStyle} formatter={(value: number) => [formatOMR(value * 1000), t("nav.sales")]} />
                <Area type="monotone" dataKey="amount" stroke="var(--brand-gold)" strokeWidth={2.5} fill="url(#salesGrad)" dot={false} activeDot={{ r: 5, fill: 'var(--brand-gold)' }} />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </Card>

        {/* Production Daily - 7 days */}
        <Card>
          <h3 className="section-title">
            <Factory className="w-5 h-5 text-gold-400" />
            {t("dashboard.productionDailyTitle")}
          </h3>
          <div className="h-72">
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={productionTrend.slice(-7)}>
                <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" />
                <XAxis dataKey="name" stroke="var(--text-muted)" fontSize={11} tickLine={false} />
                <YAxis stroke="var(--text-muted)" fontSize={11} tickLine={false} axisLine={false} />
                <Tooltip contentStyle={ChartTooltipStyle} />
                <Legend wrapperStyle={{ fontSize: '12px', color: 'var(--text-secondary)' }} />
                <Bar dataKey="good" fill="var(--brand-primary)" radius={[6, 6, 0, 0]} name={t("dashboard.good")} />
                <Bar dataKey="waste" fill="#ef4444" radius={[6, 6, 0, 0]} name={t("dashboard.waste")} />
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
            {t("dashboard.monthlyProduction")}
          </h3>
          <div className="h-72">
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={monthlyProd}>
                <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" />
                <XAxis dataKey="name" stroke="var(--text-muted)" fontSize={11} tickLine={false} />
                <YAxis stroke="var(--text-muted)" fontSize={11} tickLine={false} axisLine={false} />
                <Tooltip contentStyle={ChartTooltipStyle} />
                <Legend wrapperStyle={{ fontSize: '12px', color: 'var(--text-secondary)' }} />
                <Bar dataKey="cartons" fill="var(--brand-primary)" radius={[6, 6, 0, 0]} name={t("dashboard.cartons")} />
                <Bar dataKey="cups" fill="var(--brand-gold)" radius={[6, 6, 0, 0]} name={t("dashboard.cups")} />
              </BarChart>
            </ResponsiveContainer>
          </div>
        </Card>

        {/* Top Customers */}
        <Card>
          <h3 className="section-title">
            <ShoppingCart className="w-5 h-5 text-gold-400" />
            {t("dashboard.topFiveCustomers")}
          </h3>
          <div className="h-72">
            {topCustomers.length > 0 ? (
              <ResponsiveContainer width="100%" height="100%">
                <BarChart data={topCustomers} layout="vertical">
                  <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" horizontal={false} />
                  <XAxis type="number" stroke="var(--text-muted)" fontSize={11} tickLine={false} />
                  <YAxis type="category" dataKey="name" stroke="var(--text-muted)" fontSize={10} tickLine={false} width={90} />
                  <Tooltip contentStyle={ChartTooltipStyle} formatter={(value: number) => [formatOMR(value * 1000), t("nav.sales")]} />
                  <Bar dataKey="sales" fill="var(--brand-gold)" radius={[0, 6, 6, 0]} barSize={18} name={t("nav.sales")} />
                </BarChart>
              </ResponsiveContainer>
            ) : (
              <div className="flex items-center justify-center h-full text-surface-500 text-sm">{t("dashboard.noDataYet")}</div>
            )}
          </div>
        </Card>

        {/* Expenses by Category */}
        <Card>
          <h3 className="section-title">
            <PieIcon className="w-5 h-5 text-gold-400" />
            {t("dashboard.expensesByCategory")}
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
                  <Tooltip contentStyle={ChartTooltipStyle} formatter={(value: number) => [formatOMR(value * 1000), t("dashboard.amount")]} />
                  <Legend wrapperStyle={{ fontSize: '11px', color: '#94a3b8' }} />
                </PieChart>
              </ResponsiveContainer>
            ) : (
              <div className="flex items-center justify-center h-full text-surface-500 text-sm">{t("dashboard.noExpenses")}</div>
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
              {t("dashboard.liveProductionTitle")}
            </h3>
            <button onClick={() => navigate('/live-production')} className="text-xs text-brand-400 hover:text-gold-400 transition-colors font-medium">
              {t("dashboard.openLiveProduction")}
            </button>
          </div>
          <div className="grid grid-cols-4 gap-5">
            <div className="text-center p-4 bg-surface-800/50 rounded-2xl border border-surface-700/30">
              <p className="text-3xl font-bold text-white">{liveData.today_total_cartons?.toFixed(0) || 0}</p>
              <p className="text-xs text-surface-400 mt-2">{t("dashboard.totalCartons")}</p>
            </div>
            <div className="text-center p-4 bg-surface-800/50 rounded-2xl border border-amber-500/20">
              <p className="text-3xl font-bold text-amber-400">{liveData.morning_shift_cartons?.toFixed(0) || 0}</p>
              <p className="text-xs text-surface-400 mt-2">{t("dashboard.morningShift")}</p>
            </div>
            <div className="text-center p-4 bg-surface-800/50 rounded-2xl border border-indigo-500/20">
              <p className="text-3xl font-bold text-indigo-400">{liveData.evening_shift_cartons?.toFixed(0) || 0}</p>
              <p className="text-xs text-surface-400 mt-2">{t("dashboard.eveningShift")}</p>
            </div>
            <div className="text-center p-4 bg-surface-800/50 rounded-2xl border border-gold-500/20">
              <p className="text-3xl font-bold gradient-text">{(liveData.today_total_cups || 0).toLocaleString()}</p>
              <p className="text-xs text-surface-400 mt-2">{t("dashboard.totalCups")}</p>
            </div>
          </div>
          {liveData.recent_entries?.length > 0 && (
            <div className="mt-4 p-3 bg-surface-800/30 rounded-xl border border-surface-700/20">
              <p className="text-xs text-surface-400">
                {t("dashboard.lastEntry")} <span className="text-white font-medium">{liveData.recent_entries[0]?.product_name || t("dashboard.productFallback", { id: liveData.recent_entries[0]?.product_id })}</span>
                {' — '}
                <span className="text-gold-400 font-mono">{t("dashboard.cartonsProduced", { count: liveData.recent_entries[0]?.cartons_produced?.toFixed(0) })}</span>
              </p>
            </div>
          )}
        </Card>
      )}

      {/* Quick Actions */}
      <div className="grid grid-cols-5 gap-5">
        {[
          { label: t("dashboard.newInvoice"), icon: FileText, path: '/invoices/new', color: 'brand' },
          { label: t("dashboard.liveProduction"), icon: Factory, path: '/live-production', color: 'amber' },
          { label: t("dashboard.productionOrder"), icon: Package, path: '/production/new', color: 'brand' },
          { label: t("nav.inventory"), icon: Warehouse, path: '/reports/low-stock', color: 'brand' },
          { label: t("dashboard.dailyBrief"), icon: BarChart3, path: '/dashboard/daily-brief', color: 'brand' },
        ].map(({ label, icon: Icon, path, color }) => (
          <button key={path} onClick={() => navigate(path)} className="card-hover flex flex-col items-center gap-3 py-7 group">
            <div className={`w-14 h-14 rounded-2xl ${color === 'amber' ? 'bg-amber-500/20 border border-amber-500/30' : 'bg-brand-800/30 border border-brand-500/20'} flex items-center justify-center group-hover:shadow-glow transition-all`}>
              <Icon className={`w-7 h-7 ${color === 'amber' ? 'text-amber-400' : 'text-brand-400'} group-hover:text-gold-400 transition-colors`} />
            </div>
            <span className="text-sm font-medium text-surface-300 group-hover:text-white transition-colors">{label}</span>
          </button>
        ))}
      </div>
    </div>
  );
}
