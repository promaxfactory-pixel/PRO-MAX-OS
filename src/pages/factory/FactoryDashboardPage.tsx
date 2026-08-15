import { useState, useEffect, useCallback, useMemo } from "react";
import Button from "@/components/ui/Button";
import { StatCard } from "@/components/ui/Card";
import DataTable, { Column } from "@/components/ui/DataTable";
import LoadingSpinner from "@/components/ui/LoadingSpinner";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@/lib/tauri";
import { useUIStore } from "@/stores/uiStore";
import { Factory, PackageCheck, Boxes, Wallet, Users, ArrowLeftRight, RefreshCw, FileClock } from "lucide-react";
import type { ExpenseSummary, InventoryItem } from "@/types";

interface LiveDashboard {
  today_total_cartons: number;
  today_total_cups: number;
  morning_shift_cartons: number;
  evening_shift_cartons: number;
  products: { product_id: number; product_name: string | null; customer_brand: string | null; total_cartons: number; total_cups: number; waste_cartons: number }[];
  recent_entries: { id: number; product_name: string | null; cartons_produced: number; worker_name: string | null; customer_brand: string | null }[];
}

interface WorkerDailySummary {
  employee_id: number;
  worker_name: string | null;
  total_cartons: number;
  total_cups: number;
  total_waste: number;
  products: { product_id: number; product_name: string | null; customer_brand: string | null; total_cartons: number; total_cups: number; waste_cartons: number }[];
}

function fmtInt(n: number | undefined | null): string {
  return (n ?? 0).toLocaleString("en-US");
}

function todayStr(): string {
  const d = new Date();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${d.getFullYear()}-${m}-${day}`;
}

function addDays(base: string, delta: number): string {
  const d = new Date(`${base}T00:00:00`);
  d.setDate(d.getDate() + delta);
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${d.getFullYear()}-${m}-${day}`;
}

export default function FactoryDashboardPage() {
  const addNotification = useUIStore((s) => s.addNotification);
  const [live, setLive] = useState<LiveDashboard | null>(null);
  const [workers, setWorkers] = useState<WorkerDailySummary[]>([]);
  const [summary, setSummary] = useState<ExpenseSummary | null>(null);
  const [inventory, setInventory] = useState<InventoryItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [range, setRange] = useState<"week" | "month">("week");
  const today = todayStr();

  const loadAll = useCallback(async () => {
    setLoading(true);
    try {
      const [liveRes, workerRes, invRes] = await Promise.all([
        invoke<LiveDashboard>("get_live_dashboard"),
        invoke<WorkerDailySummary[]>("get_worker_daily_report", { date: today }),
        invoke<InventoryItem[]>("list_inventory_items"),
      ]);
      setLive(liveRes);
      setWorkers(workerRes);
      setInventory(invRes);
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) });
    } finally {
      setLoading(false);
    }
  }, [addNotification, today]);

  useEffect(() => { loadAll(); }, [loadAll]);

  const loadSummary = useCallback(async (r: "week" | "month") => {
    const dateTo = todayStr();
    const dateFrom = r === "week" ? addDays(dateTo, -6) : `${dateTo.slice(0, 8)}01`;
    try {
      const res = await invoke<ExpenseSummary>("get_expense_summary", { dateFrom, dateTo });
      setSummary(res);
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) });
    }
  }, [addNotification]);

  useEffect(() => { loadSummary(range); }, [range, loadSummary]);

  const productionColumns: Column<LiveDashboard["products"][number]>[] = useMemo(() => [
    { key: "product_name", header: "الصنف", render: (r) => <span className="font-medium text-white">{r.product_name || "—"}</span> },
    { key: "customer_brand", header: "العميل", render: (r) => r.customer_brand || "—" },
    { key: "total_cartons", header: "الكراتين", align: "center", render: (r) => <span className="font-mono">{fmtInt(r.total_cartons)}</span> },
    { key: "total_cups", header: "الأكواب", align: "center", render: (r) => <span className="font-mono">{fmtInt(r.total_cups)}</span> },
    { key: "waste_cartons", header: "الهالك (كرتون)", align: "center", render: (r) => <span className="font-mono text-red-400">{fmtInt(r.waste_cartons)}</span> },
  ], []);

  const workerColumns: Column<WorkerDailySummary>[] = useMemo(() => [
    { key: "worker_name", header: "العامل", render: (r) => <span className="font-medium text-white">{r.worker_name || "—"}</span> },
    { key: "total_cartons", header: "كراتين", align: "center", render: (r) => <span className="font-mono">{fmtInt(r.total_cartons)}</span> },
    { key: "total_cups", header: "أكواب", align: "center", render: (r) => <span className="font-mono">{fmtInt(r.total_cups)}</span> },
    { key: "total_waste", header: "هالك", align: "center", render: (r) => <span className="font-mono text-red-400">{fmtInt(r.total_waste)}</span> },
  ], []);

  const lowStock = useMemo(
    () => inventory.filter((i) => i.kind === "finished" && i.qty_on_hand <= i.reorder_level).slice(0, 8),
    [inventory],
  );

  if (loading && !live) return <div className="flex items-center justify-center py-16"><LoadingSpinner size="lg" /></div>;

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">لوحة مصنع الأكواب</h1>
          <p className="page-subtitle">إنتاج اليوم، إنتاج العمال، المخزون، ومصاريف {range === "week" ? "الأسبوع" : "الشهر"}</p>
        </div>
        <div className="flex items-center gap-2">
          <Button size="sm" variant={range === "week" ? "primary" : "outline"} onClick={() => setRange("week")} icon={<FileClock className="w-4 h-4" />}>أسبوع</Button>
          <Button size="sm" variant={range === "month" ? "primary" : "outline"} onClick={() => setRange("month")} icon={<FileClock className="w-4 h-4" />}>شهر</Button>
          <Button variant="outline" onClick={loadAll} icon={<RefreshCw className="w-4 h-4" />}>تحديث</Button>
        </div>
      </div>

      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard title="إنتاج اليوم (كرتون)" value={fmtInt(live?.today_total_cartons)} icon={<Boxes className="w-6 h-6" />} />
        <StatCard title="إنتاج اليوم (أكواب)" value={fmtInt(live?.today_total_cups)} icon={<Factory className="w-6 h-6" />} />
        <StatCard title="وردية صباحية" value={fmtInt(live?.morning_shift_cartons)} icon={<PackageCheck className="w-6 h-6" />} />
        <StatCard title="وردية مسائية" value={fmtInt(live?.evening_shift_cartons)} icon={<PackageCheck className="w-6 h-6" />} />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="card">
          <div className="flex items-center justify-between px-5 py-4 border-b border-[var(--border)]">
            <h2 className="text-sm font-bold text-white flex items-center gap-2"><Factory className="w-4 h-4 text-gold-400" />إنتاج اليوم حسب الصنف</h2>
            <span className="text-xs text-surface-500">{live?.products.length ?? 0} صنف</span>
          </div>
          <DataTable columns={productionColumns} data={live?.products ?? []} loading={false} emptyMessage="لا يوجد إنتاج مسجل اليوم — سجّل الوردية من شاشة الإنتاج المباشر" />
        </div>

        <div className="card">
          <div className="flex items-center justify-between px-5 py-4 border-b border-[var(--border)]">
            <h2 className="text-sm font-bold text-white flex items-center gap-2"><Users className="w-4 h-4 text-gold-400" />إنتاج العمال اليوم</h2>
            <span className="text-xs text-surface-500">بالكراتين لكل عامل</span>
          </div>
          <DataTable columns={workerColumns} data={workers} loading={false} emptyMessage="لا توجد بيانات إنتاج للعمال اليوم" />
        </div>
      </div>

      <div className="card">
        <div className="px-5 py-4 border-b border-[var(--border)]">
          <h2 className="text-sm font-bold text-white flex items-center gap-2"><Wallet className="w-4 h-4 text-gold-400" />مصاريف {range === "week" ? "الأسبوع" : "الشهر"}</h2>
          <p className="text-xs text-surface-500 mt-1">إجمالي {fmtInt(summary?.count ?? 0)} حركة • {formatOMR(summary?.total_milli ?? 0)}</p>
        </div>
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-4 p-5">
          <div className="space-y-3">
            <h3 className="text-xs font-bold text-surface-400">حسب المصدر</h3>
            {(summary?.by_source ?? []).map((s) => (
              <div key={s.source} className="flex items-center justify-between rounded-xl bg-surface-900/50 px-4 py-3">
                <span className="text-sm text-surface-300">{s.label}</span>
                <span className="font-mono font-bold text-gold-400">{formatOMR(s.total_milli)}</span>
              </div>
            ))}
            {(summary?.by_source ?? []).length === 0 && <p className="text-sm text-surface-500">لا توجد مصاريف في هذه الفترة</p>}
          </div>
          <div className="space-y-3">
            <h3 className="text-xs font-bold text-surface-400">حسب التصنيف</h3>
            {(summary?.by_category ?? []).slice(0, 8).map((c) => (
              <div key={c.category} className="flex items-center justify-between rounded-xl bg-surface-900/50 px-4 py-3">
                <span className="text-sm text-surface-300">{c.category}</span>
                <span className="font-mono font-bold text-gold-400">{formatOMR(c.total_milli)}</span>
              </div>
            ))}
            {(summary?.by_category ?? []).length === 0 && <p className="text-sm text-surface-500">لا توجد تصنيفات</p>}
          </div>
          <div className="space-y-3">
            <h3 className="text-xs font-bold text-surface-400">آخر المصاريف</h3>
            {(summary?.details ?? []).slice(0, 8).map((e) => (
              <div key={e.id} className="rounded-xl bg-surface-900/50 px-4 py-3">
                <div className="flex items-center justify-between">
                  <span className="text-sm text-surface-300">{e.category || "عام"}</span>
                  <span className="font-mono font-bold text-gold-400">{formatOMR((e.amount_milli || 0) + (e.vat_milli || 0))}</span>
                </div>
                <div className="text-[11px] text-surface-500 mt-1">{e.date}{e.vendor ? ` • ${e.vendor}` : ""}</div>
              </div>
            ))}
            {(summary?.details ?? []).length === 0 && <p className="text-sm text-surface-500">لا توجد تفاصيل</p>}
          </div>
        </div>
      </div>

      <div className="card">
        <div className="flex items-center justify-between px-5 py-4 border-b border-[var(--border)]">
          <h2 className="text-sm font-bold text-white flex items-center gap-2"><ArrowLeftRight className="w-4 h-4 text-gold-400" />تنبيهات المخزون (أقل من حد الطلب)</h2>
          <span className="text-xs text-surface-500">{lowStock.length} صنف</span>
        </div>
        {lowStock.length === 0 ? (
          <p className="px-5 py-6 text-sm text-surface-500">لا توجد أصناف منخفضة حالياً</p>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead><tr className="bg-surface-800 text-surface-400">
                <th className="px-5 py-2 text-right font-medium">الصنف</th>
                <th className="px-5 py-2 text-left font-medium">المتوفر</th>
                <th className="px-5 py-2 text-left font-medium">حد الطلب</th>
                <th className="px-5 py-2 text-left font-medium">القيمة</th>
              </tr></thead>
              <tbody>
                {lowStock.map((i) => (
                  <tr key={i.id} className="border-t border-surface-700">
                    <td className="px-5 py-2 text-white font-medium">{i.name_ar || i.name_en}</td>
                    <td className="px-5 py-2 font-mono text-red-400">{fmtInt(i.qty_on_hand)}</td>
                    <td className="px-5 py-2 font-mono text-surface-400">{fmtInt(i.reorder_level)}</td>
                    <td className="px-5 py-2 font-mono">{formatOMR(i.qty_on_hand * (i.avg_cost_milli || 0))}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}
