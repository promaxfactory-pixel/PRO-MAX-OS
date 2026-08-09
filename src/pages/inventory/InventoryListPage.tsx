import { useState, useEffect, useMemo } from "react";
import { useTranslation } from "react-i18next";
import DataTable, { Column } from "@/components/ui/DataTable";
import Badge from "@/components/ui/Badge";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { AlertTriangle } from "lucide-react";
import { useUIStore } from "../../stores/uiStore";
import { InventoryItem } from "@/types";

export default function InventoryListPage() {
  const { t } = useTranslation();
  const { addNotification } = useUIStore();
  const [items, setItems] = useState<InventoryItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [kindFilter, setKindFilter] = useState("all");

  useEffect(() => { invoke("list_inventory_items").then((d) => setItems(d as InventoryItem[])).catch((e: unknown) => addNotification({ title: t('common.error'), message: String(e), type: 'error' })).finally(() => setLoading(false)); }, [t]);

  const filtered = kindFilter === "all" ? items : items.filter(i => i.kind === kindFilter);
  const lowStock = items.filter(i => i.reorder_level > 0 && i.qty_on_hand <= i.reorder_level);

  const columns: Column<any>[] = useMemo(() => [
    { key: "code", header: t("inventory.code"), sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.code || "—"}</span> },
    { key: "name_ar", header: t("inventory.name"), sortable: true, render: (r) => r.name_ar || r.name_en || "—" },
    { key: "kind", header: t("inventory.kind"), sortable: true, render: (r) => <Badge variant={r.kind === 'finished' ? 'success' : r.kind === 'raw' ? 'info' : 'default'}>{r.kind}</Badge> },
    { key: "uom", header: t("inventoryList.uom"), align: "center" },
    { key: "qty_on_hand", header: t("inventory.title"), sortable: true, align: "center", render: (r) => <span className={`font-bold ${r.reorder_level > 0 && r.qty_on_hand <= r.reorder_level ? 'text-red-400' : 'text-emerald-400'}`}>{r.qty_on_hand}</span> },
    { key: "reorder_level", header: t("inventory.reorderLevel"), align: "center" },
    { key: "avg_cost_milli", header: t("inventory.avgCost"), align: "left", render: (r) => formatOMR(r.avg_cost_milli) },
  ], [t]);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div><h1 className="page-title">{t("inventory.title")}</h1><p className="page-subtitle">{t("inventoryList.summary", { items: items.length, low: lowStock.length })}</p></div>
        {lowStock.length > 0 && (
          <div className="flex items-center gap-2 px-4 py-2 bg-red-500/10 border border-red-500/30 rounded-xl">
            <AlertTriangle className="w-4 h-4 text-red-400" />
            <span className="text-sm text-red-400 font-medium">{t("inventoryList.lowStockCount", { count: lowStock.length })}</span>
          </div>
        )}
      </div>
      <div className="flex items-center gap-2 mb-4">
        {["all", "finished", "raw", "packaging", "spare", "consumable"].map((k) => (
          <button key={k} onClick={() => setKindFilter(k)}
            className={`px-3 py-1 rounded-full text-xs font-medium transition-all ${kindFilter === k ? 'bg-brand-800 text-gold-400 border border-brand-500/30' : 'bg-surface-800 text-surface-400 border border-surface-700 hover:text-white'}`}>
            {k === "all" ? t("common.all") : k === "finished" ? t("inventory.finished") : k === "raw" ? t("inventory.raw") : k === "packaging" ? t("inventory.packaging") : k === "spare" ? t("inventory.spare") : t("inventory.consumable")}
          </button>
        ))}
      </div>
      <DataTable columns={columns} data={filtered} loading={loading} emptyMessage={t("inventoryList.empty")} />
    </div>
  );
}
