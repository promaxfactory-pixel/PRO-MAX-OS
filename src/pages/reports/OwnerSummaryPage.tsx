import { useState, useEffect, useCallback } from "react";
import Card from "@/components/ui/Card";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { TrendingUp, TrendingDown, DollarSign, Users, Truck } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { useTranslation } from "react-i18next";

export default function OwnerSummaryPage() {
  const { t } = useTranslation();
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
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("reports.loadError") }); }
    finally { setLoading(false); }
  }, [addNotification, fromDate, toDate]);

  useEffect(() => { loadData(); }, [loadData]);

  if (loading || !data) {
    return <div className="flex items-center justify-center h-64"><div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" /></div>;
  }

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">{t("reports.ownerSummary")}</h1>
          <p className="page-subtitle">{t("reports.ownerSummarySubtitle")}</p>
        </div>
        <div className="flex items-center gap-2">
          <input type="date" value={fromDate} onChange={(e) => setFromDate(e.target.value)} className="input-field text-sm" aria-label={t("common.fromDate")} />
          <span className="text-surface-500">{t("stockTransfers.to")}</span>
          <input type="date" value={toDate} onChange={(e) => setToDate(e.target.value)} className="input-field text-sm" aria-label={t("common.toDate")} />
        </div>
      </div>

      <div className="grid grid-cols-4 gap-4">
        <Card>
          <div className="flex items-center gap-3 mb-2">
            <div className="p-2 rounded-lg bg-emerald-500/10 text-emerald-400"><TrendingUp className="w-5 h-5" /></div>
            <p className="text-sm text-surface-400">{t("reports.totalSales")}</p>
          </div>
          <p className="text-2xl font-bold gradient-text">{formatOMR(data.sales_total_milli)}</p>
          <p className="text-xs text-surface-500 mt-1">{t("reports.vatAmount", { amount: formatOMR(data.sales_vat_milli) })}</p>
        </Card>

        <Card>
          <div className="flex items-center gap-3 mb-2">
            <div className="p-2 rounded-lg bg-gold-500/10 text-gold-400"><DollarSign className="w-5 h-5" /></div>
            <p className="text-sm text-surface-400">{t("reports.grossProfit")}</p>
          </div>
          <p className="text-2xl font-bold" style={{ color: data.gross_profit_milli >= 0 ? "#10b981" : "#ef4444" }}>{formatOMR(data.gross_profit_milli)}</p>
          <p className="text-xs text-surface-500 mt-1">{t("reports.marginAmount", { value: data.gross_margin_pct.toFixed(1) })}</p>
        </Card>

        <Card>
          <div className="flex items-center gap-3 mb-2">
            <div className="p-2 rounded-lg bg-purple-500/10 text-purple-400"><TrendingDown className="w-5 h-5" /></div>
            <p className="text-sm text-surface-400">{t("reports.netProfit")}</p>
          </div>
          <p className="text-2xl font-bold" style={{ color: data.net_profit_milli >= 0 ? "#10b981" : "#ef4444" }}>{formatOMR(data.net_profit_milli)}</p>
          <p className="text-xs text-surface-500 mt-1">{t("reports.marginAmount", { value: data.net_margin_pct.toFixed(1) })}</p>
        </Card>

        <Card>
          <div className="flex items-center gap-3 mb-2">
            <div className="p-2 rounded-lg bg-blue-500/10 text-blue-400"><DollarSign className="w-5 h-5" /></div>
            <p className="text-sm text-surface-400">{t("reports.cashPosition")}</p>
          </div>
          <p className="text-2xl font-bold gradient-text">{formatOMR(data.cash_position_milli)}</p>
        </Card>
      </div>

      <div className="grid grid-cols-3 gap-4">
        <Card>
          <div className="flex items-center gap-2 mb-3">
            <Users className="w-5 h-5 text-brand-400" />
            <h3 className="font-bold text-sm">{t("reports.receivables")}</h3>
          </div>
          <p className="text-xl font-bold gradient-text">{formatOMR(data.accounts_receivable_milli)}</p>
        </Card>

        <Card>
          <div className="flex items-center gap-2 mb-3">
            <Truck className="w-5 h-5 text-orange-400" />
            <h3 className="font-bold text-sm">{t("reports.payables")}</h3>
          </div>
          <p className="text-xl font-bold text-orange-400">{formatOMR(data.accounts_payable_milli)}</p>
        </Card>

        <Card>
          <div className="flex items-center gap-2 mb-3">
            <TrendingDown className="w-5 h-5 text-red-400" />
            <h3 className="font-bold text-sm">{t("reports.operatingExpenses")}</h3>
          </div>
          <p className="text-xl font-bold text-red-400">{formatOMR(data.operating_expenses_milli)}</p>
        </Card>
      </div>
    </div>
  );
}
