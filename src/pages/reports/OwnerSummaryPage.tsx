import { useState, useEffect, useCallback } from "react";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Calendar, TrendingUp, TrendingDown, DollarSign, Users, Truck } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

interface OwnerSummaryData {
  period: { from: string; to: string };
  total_revenue_milli: number; total_cogs_milli: number; gross_profit_milli: number;
  total_expenses_milli: number; net_profit_milli: number;
  top_customers: { name: string; amount_milli: number }[];
  top_suppliers: { name: string; amount_milli: number }[];
}

export default function OwnerSummaryPage() {
  const addNotification = useUIStore((s) => s.addNotification);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [data, setData] = useState<any>(null);
  const [loading, setLoading] = useState(true);
  const [fromDate, setFromDate] = useState(new Date(new Date().getFullYear(), new Date().getMonth(), 1).toISOString().split("T")[0]);
  const [toDate, setToDate] = useState(new Date().toISOString().split("T")[0]);

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const result = await invoke("owner_summary", { fromDate, toDate });
      setData(result);
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: "ط®ط·ط£", message: "ط­ط¯ط« ط®ط·ط£ ط£ط«ظ†ط§ط، طھط­ظ…ظٹظ„ ط§ظ„ط¨ظٹط§ظ†ط§طھ" }); }
    finally { setLoading(false); }
  }, [addNotification, fromDate, toDate]);

  useEffect(() => { loadData(); }, [loadData]);

  if (loading) {
    return <div className="flex items-center justify-center h-64"><div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" /></div>;
  }

  if (!data) {
    return <div className="flex flex-col items-center justify-center h-64 gap-4"><p className="text-surface-400">تعذر تحميل ملخص المالك</p><button className="btn-outline px-4 py-2 rounded-xl text-sm" onClick={() => window.location.reload()}>إعادة المحاولة</button></div>;
  }

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">ظ…ظ„ط®طµ ط§ظ„ظ…ط§ظ„ظƒ</h1>
          <p className="page-subtitle">ظ†ط¸ط±ط© ط¹ط§ظ…ط© ط¹ظ„ظ‰ ط§ظ„ط£ط¯ط§ط، ط§ظ„ظ…ط§ظ„ظٹ</p>
        </div>
        <div className="flex items-center gap-2">
          <input type="date" value={fromDate} onChange={(e) => setFromDate(e.target.value)} className="input-field text-sm" aria-label="ظ…ظ† طھط§ط±ظٹط®" />
          <span className="text-surface-500">ط¥ظ„ظ‰</span>
          <input type="date" value={toDate} onChange={(e) => setToDate(e.target.value)} className="input-field text-sm" aria-label="ط¥ظ„ظ‰ طھط§ط±ظٹط®" />
        </div>
      </div>

      <div className="grid grid-cols-4 gap-4">
        <Card>
          <div className="flex items-center gap-3 mb-2">
            <div className="p-2 rounded-lg bg-emerald-500/10 text-emerald-400"><TrendingUp className="w-5 h-5" /></div>
            <p className="text-sm text-surface-400">ط¥ط¬ظ…ط§ظ„ظٹ ط§ظ„ظ…ط¨ظٹط¹ط§طھ</p>
          </div>
          <p className="text-2xl font-bold gradient-text">{formatOMR(data.sales_total_milli)}</p>
          <p className="text-xs text-surface-500 mt-1">ط¶ط±ظٹط¨ط©: {formatOMR(data.sales_vat_milli)}</p>
        </Card>

        <Card>
          <div className="flex items-center gap-3 mb-2">
            <div className="p-2 rounded-lg bg-gold-500/10 text-gold-400"><DollarSign className="w-5 h-5" /></div>
            <p className="text-sm text-surface-400">طµط§ظپظٹ ط§ظ„ط±ط¨ط­ ط§ظ„ط¥ط¬ظ…ط§ظ„ظٹ</p>
          </div>
          <p className="text-2xl font-bold" style={{ color: data.gross_profit_milli >= 0 ? "#10b981" : "#ef4444" }}>{formatOMR(data.gross_profit_milli)}</p>
          <p className="text-xs text-surface-500 mt-1">ظ‡ط§ظ…ط´: {data.gross_margin_pct.toFixed(1)}%</p>
        </Card>

        <Card>
          <div className="flex items-center gap-3 mb-2">
            <div className="p-2 rounded-lg bg-purple-500/10 text-purple-400"><TrendingDown className="w-5 h-5" /></div>
            <p className="text-sm text-surface-400">طµط§ظپظٹ ط§ظ„ط±ط¨ط­</p>
          </div>
          <p className="text-2xl font-bold" style={{ color: data.net_profit_milli >= 0 ? "#10b981" : "#ef4444" }}>{formatOMR(data.net_profit_milli)}</p>
          <p className="text-xs text-surface-500 mt-1">ظ‡ط§ظ…ط´: {data.net_margin_pct.toFixed(1)}%</p>
        </Card>

        <Card>
          <div className="flex items-center gap-3 mb-2">
            <div className="p-2 rounded-lg bg-blue-500/10 text-blue-400"><DollarSign className="w-5 h-5" /></div>
            <p className="text-sm text-surface-400">ط§ظ„ظˆط¶ط¹ ط§ظ„ظ†ظ‚ط¯ظٹ</p>
          </div>
          <p className="text-2xl font-bold gradient-text">{formatOMR(data.cash_position_milli)}</p>
        </Card>
      </div>

      <div className="grid grid-cols-3 gap-4">
        <Card>
          <div className="flex items-center gap-2 mb-3">
            <Users className="w-5 h-5 text-brand-400" />
            <h3 className="font-bold text-sm">ط§ظ„ظ…ط³طھط­ظ‚ط§طھ ظ…ظ† ط§ظ„ط¹ظ…ظ„ط§ط،</h3>
          </div>
          <p className="text-xl font-bold gradient-text">{formatOMR(data.accounts_receivable_milli)}</p>
        </Card>

        <Card>
          <div className="flex items-center gap-2 mb-3">
            <Truck className="w-5 h-5 text-orange-400" />
            <h3 className="font-bold text-sm">ط§ظ„ظ…ط³طھط­ظ‚ط§طھ ظ„ظ„ظ…ظˆط±ط¯ظٹظ†</h3>
          </div>
          <p className="text-xl font-bold text-orange-400">{formatOMR(data.accounts_payable_milli)}</p>
        </Card>

        <Card>
          <div className="flex items-center gap-2 mb-3">
            <TrendingDown className="w-5 h-5 text-red-400" />
            <h3 className="font-bold text-sm">ظ…طµط±ظˆظپط§طھ طھط´ط؛ظٹظ„ظٹط©</h3>
          </div>
          <p className="text-xl font-bold text-red-400">{formatOMR(data.operating_expenses_milli)}</p>
        </Card>
      </div>
    </div>
  );
}



