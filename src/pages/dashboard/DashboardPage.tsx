import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import Card, { StatCard } from "@/components/ui/Card";
import { StatusBadge } from "@/components/ui/Badge";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import {
  Users, Package, FileText, TrendingUp, TrendingDown,
  Factory, AlertTriangle, Banknote, Warehouse, ArrowUpLeft,
  ArrowDownRight, Clock, RefreshCw, Sun, Moon
} from "lucide-react";
import {
  AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip,
  ResponsiveContainer, BarChart, Bar, PieChart, Pie, Cell
} from "recharts";

export default function DashboardPage() {
  const navigate = useNavigate();
  const [stats, setStats] = useState<any>(null);
  const [liveData, setLiveData] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadDashboard();
    loadLive();
  }, []);

  const loadDashboard = async () => {
    setLoading(true);
    try {
      const data = await invoke("get_dashboard_stats");
      setStats(data);
    } catch (err) {
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  const loadLive = async () => {
    try {
      const data = await invoke("get_live_dashboard");
      setLiveData(data);
    } catch {}
  };

  const salesTrend = stats?.sales_trend?.map((d: any) => ({ name: d.date?.slice(5) || d.date, amount: d.amount / 1000 })) || [];
  const productionTrend = stats?.production_trend?.map((d: any) => ({ name: d.date?.slice(5) || d.date, good: d.good, waste: d.waste })) || [];

  const COLORS = ['#4c1d95', '#312e81', '#d4af37', '#8b5cf6', '#6366f1'];

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="page-header">
        <div>
          <h1 className="page-title">لوحة التحكم</h1>
          <p className="page-subtitle">نظرة عامة على أداء المصنع</p>
        </div>
        <div className="flex items-center gap-3">
          <button onClick={loadDashboard} className="btn-ghost flex items-center gap-2 text-sm">
            <RefreshCw className="w-4 h-4" /> تحديث
          </button>
          <button onClick={() => navigate('/dashboard/daily-brief')} className="btn-primary">
            الموجز اليومي
          </button>
        </div>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-4 gap-4">
        <StatCard
          title="إجمالي الإيرادات"
          value={formatOMR(stats?.revenue_milli || 0)}
          icon={<TrendingUp className="w-6 h-6" />}
          trend="up"
          trendValue="+12% هذا الشهر"
        />
        <StatCard
          title="المصروفات"
          value={formatOMR(stats?.expenses_milli || 0)}
          icon={<TrendingDown className="w-6 h-6" />}
        />
        <StatCard
          title="الفواتير المعلقة"
          value={stats?.pending_invoices || 0}
          subtitle={formatOMR(stats?.overdue_amount || 0) + " معلق"}
          icon={<FileText className="w-6 h-6" />}
        />
        <StatCard
          title="المخزون منخفض"
          value={stats?.low_stock_count || 0}
          subtitle={formatOMR(stats?.inventory_value || 0) + " قيمة"}
          icon={<AlertTriangle className="w-6 h-6" />}
          trend={stats?.low_stock_count > 0 ? "down" : "neutral"}
          trendValue={stats?.low_stock_count > 0 ? "يحتاج إعادة طلب" : "ممتلئ"}
        />
      </div>

      {/* Second row stats */}
      <div className="grid grid-cols-4 gap-4">
        <StatCard
          title="العملاء"
          value={stats?.total_customers || 0}
          icon={<Users className="w-6 h-6" />}
        />
        <StatCard
          title="المنتجات"
          value={stats?.total_products || 0}
          icon={<Package className="w-6 h-6" />}
        />
        <StatCard
          title="الإنتاج اليوم"
          value={((liveData?.today_total_cartons as number) || stats?.production_today || 0).toFixed(0) + " كرتون"}
          icon={<Factory className="w-6 h-6" style={{ color: liveData?.today_total_cartons ? '#fbbf24' : undefined }} />}
        />
        <StatCard
          title="أرصدة الخزينة"
          value={formatOMR(stats?.bank_balance || 0)}
          icon={<Banknote className="w-6 h-6" />}
        />
      </div>

      {/* Charts Row */}
      <div className="grid grid-cols-2 gap-6">
        {/* Sales Trend */}
        <Card>
          <h3 className="section-title">
            <TrendingUp className="w-5 h-5 text-gold-400" />
            اتجاه المبيعات (آخر 7 أيام)
          </h3>
          <div className="h-64">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={salesTrend}>
                <defs>
                  <linearGradient id="salesGrad" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#4c1d95" stopOpacity={0.3} />
                    <stop offset="95%" stopColor="#4c1d95" stopOpacity={0} />
                  </linearGradient>
                </defs>
                <CartesianGrid strokeDasharray="3 3" stroke="#334155" />
                <XAxis dataKey="name" stroke="#64748b" fontSize={12} />
                <YAxis stroke="#64748b" fontSize={12} />
                <Tooltip
                  contentStyle={{ backgroundColor: '#1e293b', border: '1px solid #334155', borderRadius: '12px', color: '#f8fafc' }}
                  formatter={(value: number) => [formatOMR(value * 1000), 'المبيعات']}
                />
                <Area type="monotone" dataKey="amount" stroke="#d4af37" strokeWidth={2} fill="url(#salesGrad)" />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </Card>

        {/* Production Trend */}
        <Card>
          <h3 className="section-title">
            <Factory className="w-5 h-5 text-gold-400" />
            اتجاه الإنتاج (آخر 7 أيام)
          </h3>
          <div className="h-64">
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={productionTrend}>
                <CartesianGrid strokeDasharray="3 3" stroke="#334155" />
                <XAxis dataKey="name" stroke="#64748b" fontSize={12} />
                <YAxis stroke="#64748b" fontSize={12} />
                <Tooltip
                  contentStyle={{ backgroundColor: '#1e293b', border: '1px solid #334155', borderRadius: '12px', color: '#f8fafc' }}
                />
                <Bar dataKey="good" fill="#4c1d95" radius={[4, 4, 0, 0]} name="صالح" />
                <Bar dataKey="waste" fill="#ef4444" radius={[4, 4, 0, 0]} name="هالك" />
              </BarChart>
            </ResponsiveContainer>
          </div>
        </Card>
      </div>

      {/* Live Production Widget */}
      {liveData && (
        <Card>
          <div className="flex items-center justify-between mb-4">
            <h3 className="section-title">
              <Factory className="w-5 h-5 text-gold-400" />
              الإنتاج المباشر — اليوم
            </h3>
            <button onClick={() => navigate('/live-production')} className="text-xs text-brand-400 hover:text-gold-400 transition-colors">
              فتح الإنتاج المباشر ←
            </button>
          </div>
          <div className="grid grid-cols-4 gap-4">
            <div className="text-center p-3 bg-surface-800/50 rounded-xl">
              <p className="text-2xl font-bold text-white">{liveData.today_total_cartons?.toFixed(0) || 0}</p>
              <p className="text-xs text-surface-400 mt-1">إجمالي الكرتون</p>
            </div>
            <div className="text-center p-3 bg-surface-800/50 rounded-xl">
              <p className="text-2xl font-bold text-amber-400">{liveData.morning_shift_cartons?.toFixed(0) || 0}</p>
              <p className="text-xs text-surface-400 mt-1">صباحي</p>
            </div>
            <div className="text-center p-3 bg-surface-800/50 rounded-xl">
              <p className="text-2xl font-bold text-indigo-400">{liveData.evening_shift_cartons?.toFixed(0) || 0}</p>
              <p className="text-xs text-surface-400 mt-1">مسائي</p>
            </div>
            <div className="text-center p-3 bg-surface-800/50 rounded-xl">
              <p className="text-2xl font-bold gradient-text">{(liveData.today_total_cups || 0).toLocaleString()}</p>
              <p className="text-xs text-surface-400 mt-1">إجمالي الأكواب</p>
            </div>
          </div>
          {liveData.recent_entries?.length > 0 && (
            <div className="mt-3 text-xs text-surface-500">
              آخر تسجيل: {liveData.recent_entries[0]?.product_name || `منتج #${liveData.recent_entries[0]?.product_id}`} — {liveData.recent_entries[0]?.cartons_produced?.toFixed(0)} كرتون
            </div>
          )}
        </Card>
      )}

      {/* Quick Actions */}
      <div className="grid grid-cols-5 gap-4">
        <button onClick={() => navigate('/invoices/new')} className="card-hover flex flex-col items-center gap-3 py-6 group">
          <div className="w-14 h-14 rounded-2xl bg-brand-800/30 border border-brand-500/20 flex items-center justify-center group-hover:shadow-glow transition-all">
            <FileText className="w-7 h-7 text-brand-400 group-hover:text-gold-400 transition-colors" />
          </div>
          <span className="text-sm font-medium text-surface-300 group-hover:text-white transition-colors">فاتورة جديدة</span>
        </button>

        <button onClick={() => navigate('/live-production')} className="card-hover flex flex-col items-center gap-3 py-6 group">
          <div className="w-14 h-14 rounded-2xl bg-amber-500/20 border border-amber-500/30 flex items-center justify-center group-hover:shadow-glow-gold transition-all">
            <Factory className="w-7 h-7 text-amber-400 group-hover:text-gold-400 transition-colors" />
          </div>
          <span className="text-sm font-medium text-surface-300 group-hover:text-white transition-colors">إنتاج مباشر</span>
        </button>

        <button onClick={() => navigate('/production/new')} className="card-hover flex flex-col items-center gap-3 py-6 group">
          <div className="w-14 h-14 rounded-2xl bg-brand-800/30 border border-brand-500/20 flex items-center justify-center group-hover:shadow-glow transition-all">
            <Package className="w-7 h-7 text-brand-400 group-hover:text-gold-400 transition-colors" />
          </div>
          <span className="text-sm font-medium text-surface-300 group-hover:text-white transition-colors">أمر إنتاج</span>
        </button>

        <button onClick={() => navigate('/reports/low-stock')} className="card-hover flex flex-col items-center gap-3 py-6 group">
          <div className="w-14 h-14 rounded-2xl bg-brand-800/30 border border-brand-500/20 flex items-center justify-center group-hover:shadow-glow transition-all">
            <Warehouse className="w-7 h-7 text-brand-400 group-hover:text-gold-400 transition-colors" />
          </div>
          <span className="text-sm font-medium text-surface-300 group-hover:text-white transition-colors">المخزون</span>
        </button>

        <button onClick={() => navigate('/dashboard/daily-brief')} className="card-hover flex flex-col items-center gap-3 py-6 group">
          <div className="w-14 h-14 rounded-2xl bg-brand-800/30 border border-brand-500/20 flex items-center justify-center group-hover:shadow-glow transition-all">
            <Clock className="w-7 h-7 text-brand-400 group-hover:text-gold-400 transition-colors" />
          </div>
          <span className="text-sm font-medium text-surface-300 group-hover:text-white transition-colors">الموجز اليومي</span>
        </button>
      </div>
    </div>
  );
}