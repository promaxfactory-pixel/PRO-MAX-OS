import { useState, useEffect, useCallback } from "react";
import Card from "@/components/ui/Card";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Calendar, Factory, ShoppingCart, Receipt, TrendingDown, Wallet } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

interface DailyClosingData {
  date: string; total_sales_milli: number; total_purchases_milli: number;
  total_expenses_milli: number; net_profit_milli: number;
  cash_balance_milli: number; bank_balance_milli: number;
}

export default function DailyClosingPage() {
  const addNotification = useUIStore((s) => s.addNotification);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [data, setData] = useState<any>(null);
  const [loading, setLoading] = useState(true);
  const [date, setDate] = useState(new Date().toISOString().split("T")[0]);

  useEffect(() => { loadData(); }, [date]);

  const loadData = async () => {
    setLoading(true);
    try {
      const result = await invoke("daily_factory_closing", { date: date || null });
      setData(result);
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء تحميل البيانات" }); }
    finally { setLoading(false); }
  };

  if (loading || !data) {
    return <div className="flex items-center justify-center h-64"><div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" /></div>;
  }

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">الإقفال اليومي للمصنع</h1>
          <p className="page-subtitle">ملخص شامل لأنشطة اليوم</p>
        </div>
        <div className="flex items-center gap-2">
          <Calendar className="w-4 h-4 text-surface-400" />
          <input type="date" value={date} onChange={(e) => setDate(e.target.value)} className="input-field" />
        </div>
      </div>

      <div className="grid grid-cols-3 gap-4">
        <Card>
          <div className="flex items-center gap-3 mb-3">
            <div className="p-2 rounded-lg bg-blue-500/10 text-blue-400"><Factory className="w-5 h-5" /></div>
            <h3 className="font-bold text-sm">الإنتاج</h3>
          </div>
          <div className="space-y-2">
            <div className="flex justify-between text-sm"><span className="text-surface-400">أوامر الإنتاج</span><span className="font-bold">{data.production_order_count}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">أكواب جيدة</span><span className="font-bold">{data.production_total_cups.toLocaleString()}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">هدر</span><span className="text-red-400">{data.production_total_waste.toLocaleString()}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">كفاءة الإنتاج</span><span className="font-bold text-emerald-400">{data.production_yield_pct.toFixed(1)}%</span></div>
          </div>
        </Card>

        <Card>
          <div className="flex items-center gap-3 mb-3">
            <div className="p-2 rounded-lg bg-emerald-500/10 text-emerald-400"><ShoppingCart className="w-5 h-5" /></div>
            <h3 className="font-bold text-sm">المبيعات</h3>
          </div>
          <div className="space-y-2">
            <div className="flex justify-between text-sm"><span className="text-surface-400">عدد الفواتير</span><span className="font-bold">{data.sales_invoice_count}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">صافي المبيعات</span><span className="font-bold">{formatOMR(data.sales_net_milli)}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">الضريبة</span><span>{formatOMR(data.sales_vat_milli)}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">الإجمالي</span><span className="font-bold gradient-text">{formatOMR(data.sales_total_milli)}</span></div>
          </div>
        </Card>

        <Card>
          <div className="flex items-center gap-3 mb-3">
            <div className="p-2 rounded-lg bg-gold-500/10 text-gold-400"><Wallet className="w-5 h-5" /></div>
            <h3 className="font-bold text-sm">التحصيلات</h3>
          </div>
          <div className="space-y-2">
            <div className="flex justify-between text-sm"><span className="text-surface-400">المتحصل</span><span className="font-bold gradient-text">{formatOMR(data.receipts_milli)}</span></div>
          </div>
        </Card>

        <Card>
          <div className="flex items-center gap-3 mb-3">
            <div className="p-2 rounded-lg bg-orange-500/10 text-orange-400"><Receipt className="w-5 h-5" /></div>
            <h3 className="font-bold text-sm">المشتريات</h3>
          </div>
          <div className="space-y-2">
            <div className="flex justify-between text-sm"><span className="text-surface-400">صافي</span><span className="font-bold">{formatOMR(data.purchases_net_milli)}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">الضريبة</span><span>{formatOMR(data.purchases_vat_milli)}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">الإجمالي</span><span className="font-bold">{formatOMR(data.purchases_total_milli)}</span></div>
          </div>
        </Card>

        <Card>
          <div className="flex items-center gap-3 mb-3">
            <div className="p-2 rounded-lg bg-red-500/10 text-red-400"><TrendingDown className="w-5 h-5" /></div>
            <h3 className="font-bold text-sm">المصروفات</h3>
          </div>
          <div className="space-y-2">
            <div className="flex justify-between text-sm"><span className="text-surface-400">مصروفات عامة</span><span className="font-bold">{formatOMR(data.expenses_milli)}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">الصندوق الصغير</span><span>{formatOMR(data.petty_spent_milli)}</span></div>
          </div>
        </Card>
      </div>
    </div>
  );
}
