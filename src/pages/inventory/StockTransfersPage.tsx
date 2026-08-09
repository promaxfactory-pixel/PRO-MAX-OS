import { useState, useEffect, useMemo, useCallback } from "react";
import { useTranslation } from "react-i18next";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import DataTable, { Column } from "@/components/ui/DataTable";
import { formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus, Save, ChevronUp } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import ConfirmDialog from "@/components/ui/ConfirmDialog";
import type { InventoryItem } from "@/types";

interface Warehouse {
  id: number;
  name: string;
}

interface StockTransfer {
  id: number;
  transfer_no: string;
  from_warehouse_id: number;
  from_warehouse: string;
  to_warehouse_id: number;
  to_warehouse: string;
  item_id: number;
  item_name: string;
  qty: number;
  status: string;
  notes: string;
  created_at: string;
}

export default function StockTransfersPage() {
  const { t } = useTranslation();
  const { addNotification } = useUIStore();
  const [transfers, setTransfers] = useState<StockTransfer[]>([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [saving, setSaving] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);

  const [warehouses, setWarehouses] = useState<Warehouse[]>([]);
  const [items, setItems] = useState<InventoryItem[]>([]);
  const [form, setForm] = useState({
    from_warehouse_id: 0,
    to_warehouse_id: 0,
    item_id: 0,
    qty: 0,
    notes: "",
  });

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const [transfersData, warehousesData, itemsData] = await Promise.all([
        invoke("list_stock_transfers"),
        invoke("list_warehouses"),
        invoke("list_inventory_items"),
      ]);
      setTransfers(transfersData as StockTransfer[]);
      setWarehouses(warehousesData as Warehouse[]);
      setItems(itemsData as InventoryItem[]);
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("stockTransfers.loadFailed") });
    } finally {
      setLoading(false);
    }
  }, [addNotification, t]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const set = (key: string, val: any) => setForm((f) => ({ ...f, [key]: val }));

  const handleSubmit = async (e?: React.FormEvent | Event) => {
    if (e && 'preventDefault' in e) e.preventDefault();
    if (!form.item_id) return addNotification({ id: crypto.randomUUID(), type: "warning", title: t("common.warning"), message: t("stockTransfers.selectItem") });
    if (!form.from_warehouse_id || !form.to_warehouse_id) return addNotification({ id: crypto.randomUUID(), type: "warning", title: t("common.warning"), message: t("stockTransfers.selectWarehouses") });
    setSaving(true);
    try {
      await invoke("create_stock_transfer", { input: form });
      setShowForm(false);
      setForm({ from_warehouse_id: 0, to_warehouse_id: 0, item_id: 0, qty: 0, notes: "" });
      await loadData();
    } catch (err: unknown) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: String(err) });
    } finally {
      setSaving(false);
    }
  };

  const pendingCount = transfers.filter((t) => t.status === "pending").length;
  const completedCount = transfers.filter((t) => t.status === "completed").length;

  const statusMap: Record<string, { label: string; variant: string }> = {
    pending: { label: t("badge.pending"), variant: "warning" },
    in_transit: { label: t("stockTransfers.statusInTransit"), variant: "info" },
    completed: { label: t("badge.completed"), variant: "success" },
    cancelled: { label: t("badge.cancelled"), variant: "danger" },
  };

  const columns: Column<StockTransfer>[] = useMemo(() => [
    { key: "transfer_no", header: t("stockTransfers.transferNo"), sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.transfer_no || "—"}</span> },
    { key: "from_warehouse", header: t("stockTransfers.from"), sortable: true, render: (r) => <span className="text-white">{r.from_warehouse || "—"}</span> },
    { key: "to_warehouse", header: t("stockTransfers.to"), sortable: true, render: (r) => <span className="text-white">{r.to_warehouse || "—"}</span> },
    { key: "item_name", header: t("stockTransfers.item"), sortable: true, render: (r) => <span className="text-gold-400">{r.item_name || "—"}</span> },
    { key: "qty", header: t("invoice.qty"), sortable: true, align: "center", render: (r) => <span className="font-bold text-white">{r.qty}</span> },
    { key: "status", header: t("common.status"), align: "center", render: (r) => {
      const s = statusMap[r.status] || { label: r.status, variant: "" };
      return <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${s.variant === "success" ? "bg-emerald-500/20 text-emerald-400" : s.variant === "warning" ? "bg-amber-500/20 text-amber-400" : s.variant === "info" ? "bg-blue-500/20 text-blue-400" : s.variant === "danger" ? "bg-red-500/20 text-red-400" : "bg-surface-700 text-surface-400"}`}>{s.label}</span>;
    }},
    { key: "created_at", header: t("common.date"), sortable: true, render: (r) => formatDate(r.created_at) },
  ], [t]);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">{t("stockTransfers.title")}</h1>
          <p className="page-subtitle">{t("stockTransfers.summary", { total: transfers.length, pending: pendingCount, completed: completedCount })}</p>
        </div>
        <Button icon={showForm ? <ChevronUp className="w-4 h-4" /> : <Plus className="w-4 h-4" />} onClick={() => setShowForm(!showForm)}>
          {showForm ? t("common.close") : t("stockTransfers.newTransfer")}
        </Button>
      </div>

      {showForm && (
        <Card>
          <h3 className="text-lg font-bold text-white mb-4">{t("stockTransfers.newTransferTitle")}</h3>
          <form onSubmit={(e) => { e.preventDefault(); setShowConfirm(true); }}>
            <div className="grid grid-cols-2 gap-6">
              <div className="input-group">
                <label className="input-label">{t("stockTransfers.fromWarehouseRequired")}</label>
                <select className="input-field" value={form.from_warehouse_id} onChange={(e) => set("from_warehouse_id", Number(e.target.value))} required aria-label={t("stockTransfers.fromWarehouse")}>
                  <option value={0}>{t("stockTransfers.selectWarehouse")}</option>
                  {warehouses.map((w) => (
                    <option key={w.id} value={w.id}>{w.name}</option>
                  ))}
                </select>
              </div>
              <div className="input-group">
                <label className="input-label">{t("stockTransfers.toWarehouseRequired")}</label>
                <select className="input-field" value={form.to_warehouse_id} onChange={(e) => set("to_warehouse_id", Number(e.target.value))} required aria-label={t("stockTransfers.toWarehouse")}>
                  <option value={0}>{t("stockTransfers.selectWarehouse")}</option>
                  {warehouses.map((w) => (
                    <option key={w.id} value={w.id}>{w.name}</option>
                  ))}
                </select>
              </div>
              <div className="input-group">
                <label className="input-label">{t("stockTransfers.itemRequired")}</label>
                <select className="input-field" value={form.item_id} onChange={(e) => set("item_id", Number(e.target.value))} required aria-label={t("stockTransfers.item")}>
                  <option value={0}>{t("stockTransfers.selectItemOption")}</option>
                  {items.map((i) => (
                    <option key={i.id} value={i.id}>{i.name_ar || i.name_en}</option>
                  ))}
                </select>
              </div>
              <div className="input-group">
                <label className="input-label">{t("stockTransfers.qtyRequired")}</label>
                <input className="input-field" type="number" min="0.01" step="0.01" value={form.qty} onChange={(e) => set("qty", Number(e.target.value))} required aria-label={t("invoice.qty")} />
              </div>
              <div className="input-group col-span-2">
                <label className="input-label">{t("common.notes")}</label>
                <textarea className="input-field" rows={3} value={form.notes} onChange={(e) => set("notes", e.target.value)} aria-label={t("common.notes")} />
              </div>
            </div>
            <div className="flex justify-start gap-3 mt-6">
              <Button type="submit" loading={saving} icon={<Save className="w-4 h-4" />}>{t("common.save")}</Button>
              <Button variant="outline" type="button" onClick={() => setShowForm(false)}>{t("common.cancel")}</Button>
            </div>
          </form>
        </Card>
      )}

      <DataTable columns={columns} data={transfers} loading={loading} emptyMessage={t("stockTransfers.empty")} />

      <ConfirmDialog
        open={showConfirm}
        title={t("stockTransfers.createTransferTitle")}
        message={t("stockTransfers.confirmMessage")}
        variant="warning"
        onConfirm={() => { handleSubmit(); setShowConfirm(false); }}
        onCancel={() => setShowConfirm(false)}
      />
    </div>
  );
}
