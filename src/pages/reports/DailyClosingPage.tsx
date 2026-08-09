import { useState, useEffect, useCallback } from "react";
import Card from "@/components/ui/Card";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Calendar, Factory, ShoppingCart, Receipt, TrendingDown, Wallet } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { useTranslation } from "react-i18next";

export default function DailyClosingPage() {
  const { t } = useTranslation();
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
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("reports.loadError") }); }
    finally { setLoading(false); }
  }, [addNotification, date]);

  useEffect(() => { loadData(); }, [loadData]);

  if (loading || !data) {
    return <div className="flex items-center justify-center h-64"><div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" /></div>;
  }

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">{t("reports.dailyClosingTitle")}</h1>
          <p className="page-subtitle">{t("reports.dailyClosingSubtitle")}</p>
        </div>
        <div className="flex items-center gap-2">
          <Calendar className="w-4 h-4 text-surface-400" />
          <input type="date" value={date} onChange={(e) => setDate(e.target.value)} className="input-field" aria-label={t("common.date")} />
        </div>
      </div>

      <div className="grid grid-cols-3 gap-4">
        <Card>
          <div className="flex items-center gap-3 mb-3">
            <div className="p-2 rounded-lg bg-blue-500/10 text-blue-400"><Factory className="w-5 h-5" /></div>
            <h3 className="font-bold text-sm">{t("production.title")}</h3>
          </div>
          <div className="space-y-2">
            <div className="flex justify-between text-sm"><span className="text-surface-400">{t("production.orders")}</span><span className="font-bold">{data.production_order_count}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">{t("reports.goodCups")}</span><span className="font-bold">{data.production_total_cups.toLocaleString()}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">{t("reports.waste")}</span><span className="text-red-400">{data.production_total_waste.toLocaleString()}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">{t("reports.productionYield")}</span><span className="font-bold text-emerald-400">{data.production_yield_pct.toFixed(1)}%</span></div>
          </div>
        </Card>

        <Card>
          <div className="flex items-center gap-3 mb-3">
            <div className="p-2 rounded-lg bg-emerald-500/10 text-emerald-400"><ShoppingCart className="w-5 h-5" /></div>
            <h3 className="font-bold text-sm">{t("nav.sales")}</h3>
          </div>
          <div className="space-y-2">
            <div className="flex justify-between text-sm"><span className="text-surface-400">{t("reports.invoiceCount")}</span><span className="font-bold">{data.sales_invoice_count}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">{t("reports.netSales")}</span><span className="font-bold">{formatOMR(data.sales_net_milli)}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">{t("common.vat")}</span><span>{formatOMR(data.sales_vat_milli)}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">{t("reports.total")}</span><span className="font-bold gradient-text">{formatOMR(data.sales_total_milli)}</span></div>
          </div>
        </Card>

        <Card>
          <div className="flex items-center gap-3 mb-3">
            <div className="p-2 rounded-lg bg-gold-500/10 text-gold-400"><Wallet className="w-5 h-5" /></div>
            <h3 className="font-bold text-sm">{t("reports.collections")}</h3>
          </div>
          <div className="space-y-2">
            <div className="flex justify-between text-sm"><span className="text-surface-400">{t("reports.collected")}</span><span className="font-bold gradient-text">{formatOMR(data.receipts_milli)}</span></div>
          </div>
        </Card>

        <Card>
          <div className="flex items-center gap-3 mb-3">
            <div className="p-2 rounded-lg bg-orange-500/10 text-orange-400"><Receipt className="w-5 h-5" /></div>
            <h3 className="font-bold text-sm">{t("nav.purchases")}</h3>
          </div>
          <div className="space-y-2">
            <div className="flex justify-between text-sm"><span className="text-surface-400">{t("reports.net")}</span><span className="font-bold">{formatOMR(data.purchases_net_milli)}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">{t("common.vat")}</span><span>{formatOMR(data.purchases_vat_milli)}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">{t("reports.total")}</span><span className="font-bold">{formatOMR(data.purchases_total_milli)}</span></div>
          </div>
        </Card>

        <Card>
          <div className="flex items-center gap-3 mb-3">
            <div className="p-2 rounded-lg bg-red-500/10 text-red-400"><TrendingDown className="w-5 h-5" /></div>
            <h3 className="font-bold text-sm">{t("nav.expenses")}</h3>
          </div>
          <div className="space-y-2">
            <div className="flex justify-between text-sm"><span className="text-surface-400">{t("reports.generalExpenses")}</span><span className="font-bold">{formatOMR(data.expenses_milli)}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">{t("reports.pettyCash")}</span><span>{formatOMR(data.petty_spent_milli)}</span></div>
          </div>
        </Card>
      </div>
    </div>
  );
}
