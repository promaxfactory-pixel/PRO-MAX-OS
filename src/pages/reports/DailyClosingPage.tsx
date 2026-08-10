import { useState, useEffect, useCallback } from "react";
import Card from "@/components/ui/Card";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Calendar, Factory, ShoppingCart, Receipt, TrendingDown, Wallet } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

export default function DailyClosingPage() {
  const addNotification = useUIStore((s) => s.addNotification);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [data, setData] = useState<any>(null);
  const [loading, setLoading] = useState(true);
  const [date, setDate] = useState(new Date().toISOString().split("T")[0]);

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const result = await invoke("daily_factory_closing", { date: date || null });
      setData(result);
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: "ط®ط·ط£", message: "ط­ط¯ط« ط®ط·ط£ ط£ط«ظ†ط§ط، طھط­ظ…ظٹظ„ ط§ظ„ط¨ظٹط§ظ†ط§طھ" }); }
    finally { setLoading(false); }
  }, [addNotification, date]);

  useEffect(() => { loadData(); }, [loadData]);

  if (loading) {
    return <div className="flex items-center justify-center h-64"><div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" /></div>;
  }

  if (!data) {
    return <div className="flex flex-col items-center justify-center h-64 gap-4"><p className="text-surface-400">تعذر تحميل الإغلاق اليومي</p><button className="btn-outline px-4 py-2 rounded-xl text-sm" onClick={() => window.location.reload()}>إعادة المحاولة</button></div>;
  }

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">ط§ظ„ط¥ظ‚ظپط§ظ„ ط§ظ„ظٹظˆظ…ظٹ ظ„ظ„ظ…طµظ†ط¹</h1>
          <p className="page-subtitle">ظ…ظ„ط®طµ ط´ط§ظ…ظ„ ظ„ط£ظ†ط´ط·ط© ط§ظ„ظٹظˆظ…</p>
        </div>
        <div className="flex items-center gap-2">
          <Calendar className="w-4 h-4 text-surface-400" />
          <input type="date" value={date} onChange={(e) => setDate(e.target.value)} className="input-field" aria-label="ط§ظ„طھط§ط±ظٹط®" />
        </div>
      </div>

      <div className="grid grid-cols-3 gap-4">
        <Card>
          <div className="flex items-center gap-3 mb-3">
            <div className="p-2 rounded-lg bg-blue-500/10 text-blue-400"><Factory className="w-5 h-5" /></div>
            <h3 className="font-bold text-sm">ط§ظ„ط¥ظ†طھط§ط¬</h3>
          </div>
          <div className="space-y-2">
            <div className="flex justify-between text-sm"><span className="text-surface-400">ط£ظˆط§ظ…ط± ط§ظ„ط¥ظ†طھط§ط¬</span><span className="font-bold">{data.production_order_count}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">ط£ظƒظˆط§ط¨ ط¬ظٹط¯ط©</span><span className="font-bold">{data.production_total_cups.toLocaleString()}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">ظ‡ط¯ط±</span><span className="text-red-400">{data.production_total_waste.toLocaleString()}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">ظƒظپط§ط،ط© ط§ظ„ط¥ظ†طھط§ط¬</span><span className="font-bold text-emerald-400">{data.production_yield_pct.toFixed(1)}%</span></div>
          </div>
        </Card>

        <Card>
          <div className="flex items-center gap-3 mb-3">
            <div className="p-2 rounded-lg bg-emerald-500/10 text-emerald-400"><ShoppingCart className="w-5 h-5" /></div>
            <h3 className="font-bold text-sm">ط§ظ„ظ…ط¨ظٹط¹ط§طھ</h3>
          </div>
          <div className="space-y-2">
            <div className="flex justify-between text-sm"><span className="text-surface-400">ط¹ط¯ط¯ ط§ظ„ظپظˆط§طھظٹط±</span><span className="font-bold">{data.sales_invoice_count}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">طµط§ظپظٹ ط§ظ„ظ…ط¨ظٹط¹ط§طھ</span><span className="font-bold">{formatOMR(data.sales_net_milli)}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„ط¶ط±ظٹط¨ط©</span><span>{formatOMR(data.sales_vat_milli)}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„ط¥ط¬ظ…ط§ظ„ظٹ</span><span className="font-bold gradient-text">{formatOMR(data.sales_total_milli)}</span></div>
          </div>
        </Card>

        <Card>
          <div className="flex items-center gap-3 mb-3">
            <div className="p-2 rounded-lg bg-gold-500/10 text-gold-400"><Wallet className="w-5 h-5" /></div>
            <h3 className="font-bold text-sm">ط§ظ„طھط­طµظٹظ„ط§طھ</h3>
          </div>
          <div className="space-y-2">
            <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„ظ…طھط­طµظ„</span><span className="font-bold gradient-text">{formatOMR(data.receipts_milli)}</span></div>
          </div>
        </Card>

        <Card>
          <div className="flex items-center gap-3 mb-3">
            <div className="p-2 rounded-lg bg-orange-500/10 text-orange-400"><Receipt className="w-5 h-5" /></div>
            <h3 className="font-bold text-sm">ط§ظ„ظ…ط´طھط±ظٹط§طھ</h3>
          </div>
          <div className="space-y-2">
            <div className="flex justify-between text-sm"><span className="text-surface-400">طµط§ظپظٹ</span><span className="font-bold">{formatOMR(data.purchases_net_milli)}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„ط¶ط±ظٹط¨ط©</span><span>{formatOMR(data.purchases_vat_milli)}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„ط¥ط¬ظ…ط§ظ„ظٹ</span><span className="font-bold">{formatOMR(data.purchases_total_milli)}</span></div>
          </div>
        </Card>

        <Card>
          <div className="flex items-center gap-3 mb-3">
            <div className="p-2 rounded-lg bg-red-500/10 text-red-400"><TrendingDown className="w-5 h-5" /></div>
            <h3 className="font-bold text-sm">ط§ظ„ظ…طµط±ظˆظپط§طھ</h3>
          </div>
          <div className="space-y-2">
            <div className="flex justify-between text-sm"><span className="text-surface-400">ظ…طµط±ظˆظپط§طھ ط¹ط§ظ…ط©</span><span className="font-bold">{formatOMR(data.expenses_milli)}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„طµظ†ط¯ظˆظ‚ ط§ظ„طµط؛ظٹط±</span><span>{formatOMR(data.petty_spent_milli)}</span></div>
          </div>
        </Card>
      </div>
    </div>
  );
}



