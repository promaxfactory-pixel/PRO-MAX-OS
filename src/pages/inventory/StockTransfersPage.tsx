import { useState, useEffect } from "react";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import DataTable, { Column } from "@/components/ui/DataTable";
import { formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus, Save, ChevronDown, ChevronUp } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import ConfirmDialog from "@/components/ui/ConfirmDialog";

export default function StockTransfersPage() {
  const { addNotification } = useUIStore();
  const [transfers, setTransfers] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [saving, setSaving] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);

  const [warehouses, setWarehouses] = useState<any[]>([]);
  const [items, setItems] = useState<any[]>([]);
  const [form, setForm] = useState({
    from_warehouse_id: 0,
    to_warehouse_id: 0,
    item_id: 0,
    qty: 0,
    notes: "",
  });

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    setLoading(true);
    try {
      const [transfersData, warehousesData, itemsData] = await Promise.all([
        invoke("list_stock_transfers"),
        invoke("list_warehouses"),
        invoke("list_inventory_items"),
      ]);
      setTransfers(transfersData as any[]);
      setWarehouses(warehousesData as any[]);
      setItems(itemsData as any[]);
    } catch (err) {
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  const set = (key: string, val: any) => setForm((f) => ({ ...f, [key]: val }));

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!form.item_id) return addNotification({ id: crypto.randomUUID(), type: "warning", title: "تنبيه", message: "يرجى اختيار الصنف" });
    if (!form.from_warehouse_id || !form.to_warehouse_id) return addNotification({ id: crypto.randomUUID(), type: "warning", title: "تنبيه", message: "يرجى اختيار المستودعات" });
    setSaving(true);
    try {
      await invoke("create_stock_transfer", { input: form });
      setShowForm(false);
      setForm({ from_warehouse_id: 0, to_warehouse_id: 0, item_id: 0, qty: 0, notes: "" });
      await loadData();
    } catch (err: any) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: err.toString() });
    } finally {
      setSaving(false);
    }
  };

  const pendingCount = transfers.filter((t) => t.status === "pending").length;
  const completedCount = transfers.filter((t) => t.status === "completed").length;

  const statusMap: Record<string, { label: string; variant: string }> = {
    pending: { label: "قيد الانتظار", variant: "warning" },
    in_transit: { label: "قيد النقل", variant: "info" },
    completed: { label: "مكتمل", variant: "success" },
    cancelled: { label: "ملغي", variant: "danger" },
  };

  const columns: Column<any>[] = [
    { key: "transfer_no", header: "رقم التحويل", sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.transfer_no || "—"}</span> },
    { key: "from_warehouse", header: "من", sortable: true, render: (r) => <span className="text-white">{r.from_warehouse || "—"}</span> },
    { key: "to_warehouse", header: "إلى", sortable: true, render: (r) => <span className="text-white">{r.to_warehouse || "—"}</span> },
    { key: "item_name", header: "الصنف", sortable: true, render: (r) => <span className="text-gold-400">{r.item_name || "—"}</span> },
    { key: "qty", header: "الكمية", sortable: true, align: "center", render: (r) => <span className="font-bold text-white">{r.qty}</span> },
    { key: "status", header: "الحالة", align: "center", render: (r) => {
      const s = statusMap[r.status] || { label: r.status, variant: "" };
      return <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${s.variant === "success" ? "bg-emerald-500/20 text-emerald-400" : s.variant === "warning" ? "bg-amber-500/20 text-amber-400" : s.variant === "info" ? "bg-blue-500/20 text-blue-400" : s.variant === "danger" ? "bg-red-500/20 text-red-400" : "bg-surface-700 text-surface-400"}`}>{s.label}</span>;
    }},
    { key: "created_at", header: "التاريخ", sortable: true, render: (r) => formatDate(r.created_at) },
  ];

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">تحويلات المخزون</h1>
          <p className="page-subtitle">{transfers.length} تحويل • {pendingCount} قيد الانتظار • {completedCount} مكتمل</p>
        </div>
        <Button icon={showForm ? <ChevronUp className="w-4 h-4" /> : <Plus className="w-4 h-4" />} onClick={() => setShowForm(!showForm)}>
          {showForm ? "إغلاق" : "تحويل جديد"}
        </Button>
      </div>

      {showForm && (
        <Card>
          <h3 className="text-lg font-bold text-white mb-4">تحويل مخزون جديد</h3>
          <form onSubmit={(e) => { e.preventDefault(); setShowConfirm(true); }}>
            <div className="grid grid-cols-2 gap-6">
              <div className="input-group">
                <label className="input-label">من مستودع *</label>
                <select className="input-field" value={form.from_warehouse_id} onChange={(e) => set("from_warehouse_id", Number(e.target.value))} required>
                  <option value={0}>اختر المستودع</option>
                  {warehouses.map((w) => (
                    <option key={w.id} value={w.id}>{w.name}</option>
                  ))}
                </select>
              </div>
              <div className="input-group">
                <label className="input-label">إلى مستودع *</label>
                <select className="input-field" value={form.to_warehouse_id} onChange={(e) => set("to_warehouse_id", Number(e.target.value))} required>
                  <option value={0}>اختر المستودع</option>
                  {warehouses.map((w) => (
                    <option key={w.id} value={w.id}>{w.name}</option>
                  ))}
                </select>
              </div>
              <div className="input-group">
                <label className="input-label">الصنف *</label>
                <select className="input-field" value={form.item_id} onChange={(e) => set("item_id", Number(e.target.value))} required>
                  <option value={0}>اختر الصنف</option>
                  {items.map((i) => (
                    <option key={i.id} value={i.id}>{i.name_ar || i.name_en}</option>
                  ))}
                </select>
              </div>
              <div className="input-group">
                <label className="input-label">الكمية *</label>
                <input className="input-field" type="number" min="0.01" step="0.01" value={form.qty} onChange={(e) => set("qty", Number(e.target.value))} required />
              </div>
              <div className="input-group col-span-2">
                <label className="input-label">ملاحظات</label>
                <textarea className="input-field" rows={3} value={form.notes} onChange={(e) => set("notes", e.target.value)} />
              </div>
            </div>
            <div className="flex justify-start gap-3 mt-6">
              <Button type="submit" loading={saving} icon={<Save className="w-4 h-4" />}>حفظ</Button>
              <Button variant="outline" type="button" onClick={() => setShowForm(false)}>إلغاء</Button>
            </div>
          </form>
        </Card>
      )}

      <DataTable columns={columns} data={transfers} loading={loading} emptyMessage="لا توجد تحويلات" />

      <ConfirmDialog
        open={showConfirm}
        title="إنشاء تحويل مخزون"
        message="هل أنت متأكد من إنشاء تحويل مخزون جديد؟"
        variant="warning"
        onConfirm={() => { handleSubmit(new Event('submit') as any); setShowConfirm(false); }}
        onCancel={() => setShowConfirm(false)}
      />
    </div>
  );
}
