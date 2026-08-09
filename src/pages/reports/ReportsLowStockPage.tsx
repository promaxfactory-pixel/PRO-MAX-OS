import { useState, useEffect, useMemo } from "react";
import DataTable, { Column } from "@/components/ui/DataTable";
import Card from "@/components/ui/Card";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, AlertTriangle } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { useUIStore } from "@/stores/uiStore";
import { useTranslation } from "react-i18next";

interface LowStockItem {
  item_code: string;
  item_name: string;
  warehouse: string;
  current_qty: number;
  min_level: number;
  deficit: number;
  unit_cost_milli: number;
  cost_gap_milli: number;
}

export default function ReportsLowStockPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const addNotification = useUIStore((s) => s.addNotification);
  const [items, setItems] = useState<LowStockItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [summary, setSummary] = useState({ totalItems: 0, totalDeficit: 0, totalCostGap: 0 });

  useEffect(() => {
    invoke("low_stock_report")
      .then((d: unknown) => {
        const arr = (d || []) as LowStockItem[];
        setItems(arr);
        setSummary({
          totalItems: arr.length,
          totalDeficit: arr.reduce((s: number, i: LowStockItem) => s + (i.deficit || 0), 0),
          totalCostGap: arr.reduce((s: number, i: LowStockItem) => s + (i.cost_gap_milli || 0), 0),
        });
      })
      .catch((e: unknown) => addNotification({ title: t('common.error'), message: String(e), type: 'error' }))
      .finally(() => setLoading(false));
  }, []);

  const columns: Column<LowStockItem>[] = useMemo(() => [
    { key: "item_code", header: t("inventory.code"), render: (r) => <span className="font-mono text-brand-400">{r.item_code}</span> },
    { key: "item_name", header: t("stockTransfers.item"), sortable: true, render: (r) => <span className="font-medium">{r.item_name}</span> },
    { key: "warehouse", header: t("reports.warehouse"), render: (r) => r.warehouse || "—" },
    { key: "current_qty", header: t("reports.currentQty"), align: "left", render: (r) => <span className="text-amber-400 font-bold">{r.current_qty}</span> },
    { key: "min_level", header: t("reports.minLevel"), align: "left", render: (r) => r.min_level },
    { key: "deficit", header: t("reports.deficit"), align: "left", render: (r) => <span className="text-red-400 font-bold">-{r.deficit}</span> },
    { key: "unit_cost_milli", header: t("reports.unitCost"), align: "left", render: (r) => formatOMR(r.unit_cost_milli) },
    { key: "cost_gap_milli", header: t("reports.deficitValue"), align: "left", render: (r) => <span className="text-red-400 font-bold">{formatOMR(r.cost_gap_milli)}</span> },
  ], [t]);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div className="flex items-center gap-3">
          <button onClick={() => navigate("/reports")} className="btn-ghost p-2">
            <ArrowRight className="w-5 h-5" />
          </button>
          <div>
            <h1 className="page-title">{t("reports.lowStock")}</h1>
            <p className="page-subtitle">{t("reports.belowMinLevel")}</p>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-3 gap-4">
        <Card>
          <div className="flex items-center gap-3">
            <div className="p-3 rounded-xl bg-amber-500/10">
              <AlertTriangle className="w-6 h-6 text-amber-400" />
            </div>
            <div>
              <p className="text-2xl font-bold text-white">{summary.totalItems}</p>
              <p className="text-xs text-surface-400">{t("reports.lowStockItems")}</p>
            </div>
          </div>
        </Card>
        <Card>
          <div className="text-center">
            <p className="text-2xl font-bold text-red-400">{summary.totalDeficit}</p>
            <p className="text-xs text-surface-400">{t("reports.totalDeficitUnits")}</p>
          </div>
        </Card>
        <Card>
          <div className="text-center">
            <p className="text-2xl font-bold gradient-text">{formatOMR(summary.totalCostGap)}</p>
            <p className="text-xs text-surface-400">{t("reports.totalDeficitValue")}</p>
          </div>
        </Card>
      </div>

      <DataTable columns={columns} data={items} loading={loading} emptyMessage={t("reports.lowStockEmpty")} />
    </div>
  );
}
