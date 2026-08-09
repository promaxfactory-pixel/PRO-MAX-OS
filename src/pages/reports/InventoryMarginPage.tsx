import { useState, useEffect, useMemo, useCallback } from "react";
import Card from "@/components/ui/Card";
import DataTable, { Column } from "@/components/ui/DataTable";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Package, TrendingUp } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { useTranslation } from "react-i18next";

interface MarginRow {
  code: string;
  name: string;
  qty_on_hand: number;
  avg_cost_milli: number;
  selling_price_milli: number;
  margin_milli: number;
  margin_pct: number;
  stock_value_milli: number;
  stock_revenue_milli: number;
}

export default function InventoryMarginPage() {
  const { t } = useTranslation();
  const addNotification = useUIStore((s) => s.addNotification);
  const [data, setData] = useState<MarginRow[]>([]);
  const [, setLoading] = useState(true);

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const result = await invoke("inventory_margin_report");
      setData(result as MarginRow[]);
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("reports.loadError") }); }
    finally { setLoading(false); }
  }, [addNotification]);

  useEffect(() => { loadData(); }, [loadData]);

  const totalStockValue = data.reduce((s, r) => s + r.stock_value_milli, 0);
  const totalRevenue = data.reduce((s, r) => s + r.stock_revenue_milli, 0);

  const columns: Column<MarginRow>[] = useMemo(() => [
    { key: "code", header: t("inventory.code"), render: (r) => r.code || "—" },
    { key: "name", header: t("reports.product"), render: (r) => r.name || "—" },
    { key: "qty_on_hand", header: t("invoice.qty"), align: "center", render: (r) => r.qty_on_hand.toLocaleString() },
    { key: "avg_cost_milli", header: t("inventory.avgCost"), align: "left", render: (r) => formatOMR(r.avg_cost_milli) },
    { key: "selling_price_milli", header: t("reports.sellingPrice"), align: "left", render: (r) => formatOMR(r.selling_price_milli) },
    { key: "margin_milli", header: t("productDetail.margin"), align: "left", render: (r) => <span className="font-bold text-emerald-400">{formatOMR(r.margin_milli)}</span> },
    { key: "margin_pct", header: t("reports.marginPct"), align: "center", render: (r) => <span className="font-bold">{r.margin_pct.toFixed(1)}%</span> },
    { key: "stock_value_milli", header: t("dashboard.inventoryValue"), align: "left", render: (r) => formatOMR(r.stock_value_milli) },
    { key: "stock_revenue_milli", header: t("reports.expectedRevenue"), align: "left", render: (r) => formatOMR(r.stock_revenue_milli) },
  ], [t]);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">{t("reports.profitMarginReport")}</h1>
          <p className="page-subtitle">{t("reports.inventoryMarginSubtitle")}</p>
        </div>
      </div>

      <div className="grid grid-cols-3 gap-4">
        <Card>
          <div className="flex items-center gap-3 mb-2">
            <div className="p-2 rounded-lg bg-brand-500/10 text-brand-400"><Package className="w-5 h-5" /></div>
            <p className="text-sm text-surface-400">{t("reports.productCount")}</p>
          </div>
          <p className="text-2xl font-bold gradient-text">{data.length}</p>
        </Card>
        <Card>
          <div className="flex items-center gap-3 mb-2">
            <div className="p-2 rounded-lg bg-blue-500/10 text-blue-400"><Package className="w-5 h-5" /></div>
            <p className="text-sm text-surface-400">{t("reports.totalStockValue")}</p>
          </div>
          <p className="text-2xl font-bold gradient-text">{formatOMR(totalStockValue)}</p>
        </Card>
        <Card>
          <div className="flex items-center gap-3 mb-2">
            <div className="p-2 rounded-lg bg-emerald-500/10 text-emerald-400"><TrendingUp className="w-5 h-5" /></div>
            <p className="text-sm text-surface-400">{t("reports.expectedRevenue")}</p>
          </div>
          <p className="text-2xl font-bold text-emerald-400">{formatOMR(totalRevenue)}</p>
        </Card>
      </div>

      <Card>
        <DataTable columns={columns} data={data} compact />
      </Card>
    </div>
  );
}
