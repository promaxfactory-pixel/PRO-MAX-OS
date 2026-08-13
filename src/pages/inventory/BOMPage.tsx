import { useState, useEffect, useMemo, useCallback } from "react";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import DataTable, { Column } from "@/components/ui/DataTable";
import { invoke } from "@/lib/tauri";
import { Plus, Save, ChevronUp } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import ConfirmDialog from "@/components/ui/ConfirmDialog";
import type { Product, InventoryItem } from "@/types";

interface Bom {
  id: number;
  product_id: number;
  product_name: string;
  item_id: number;
  item_name: string;
  qty_per_carton: number;
  waste_pct: number;
  active: number;
}

export default function BOMPage() {
  const { addNotification } = useUIStore();
  const [boms, setBoms] = useState<Bom[]>([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [saving, setSaving] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);

  const [products, setProducts] = useState<Product[]>([]);
  const [items, setItems] = useState<InventoryItem[]>([]);
  const [form, setForm] = useState({
    product_id: 0,
    item_id: 0,
    qty_per_carton: 0,
    waste_pct: 0,
  });

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const [bomsData, productsData, itemsData] = await Promise.all([
        invoke("list_boms"),
        invoke("list_products"),
        invoke("list_inventory_items"),
      ]);
      setBoms(bomsData as Bom[]);
      setProducts(productsData as Product[]);
      setItems(itemsData as InventoryItem[]);
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء تحميل البيانات" });
    } finally {
      setLoading(false);
    }
  }, [addNotification]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const set = (key: string, val: any) => setForm((f) => ({ ...f, [key]: val }));

  const handleSubmit = async (e?: React.FormEvent | Event) => {
    if (e && "preventDefault" in e) e.preventDefault();
    if (!form.product_id || !form.item_id) return addNotification({ id: crypto.randomUUID(), type: "warning", title: "تنبيه", message: "يرجى اختيار المنتج والمادة الخام" });
    setSaving(true);
    try {
      await invoke("create_bom", { input: form });
      setShowForm(false);
      setForm({ product_id: 0, item_id: 0, qty_per_carton: 0, waste_pct: 0 });
      await loadData();
    } catch (err: unknown) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) });
    } finally {
      setSaving(false);
    }
  };

  const activeCount = boms.filter((b) => b.active).length;

  const columns: Column<Bom>[] = useMemo(() => [
    { key: "product_name", header: "المنتج", sortable: true, render: (r) => <span className="font-medium text-gold-400">{r.product_name || "—"}</span> },
    { key: "item_name", header: "المادة الخام", sortable: true, render: (r) => <span className="text-brand-400">{r.item_name || "—"}</span> },
    { key: "qty_per_carton", header: "الكمية لكل كرتون", sortable: true, align: "center", render: (r) => <span className="font-bold text-white">{r.qty_per_carton}</span> },
    { key: "waste_pct", header: "نسبة التالف %", align: "center", render: (r) => <span className={r.waste_pct > 0 ? "text-red-400" : "text-emerald-400"}>{r.waste_pct}%</span> },
    { key: "active", header: "الحالة", align: "center", render: (r) => (
      <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${r.active ? "bg-emerald-500/20 text-emerald-400" : "bg-surface-700 text-surface-400"}`}>
        {r.active ? "نشط" : "غير نشط"}
      </span>
    )},
  ], []);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">Bill of Materials</h1>
          <p className="page-subtitle">{boms.length} BOM • {activeCount} نشط</p>
        </div>
        <Button icon={showForm ? <ChevronUp className="w-4 h-4" /> : <Plus className="w-4 h-4" />} onClick={() => setShowForm(!showForm)}>
          {showForm ? "إغلاق" : "إضافة BOM"}
        </Button>
      </div>

      {showForm && (
        <Card>
          <h3 className="text-lg font-bold text-white mb-4">إضافة BOM جديد</h3>
          <form onSubmit={(e) => { e.preventDefault(); setShowConfirm(true); }}>
            <div className="grid grid-cols-2 gap-6">
              <div className="input-group">
                <label className="input-label">المنتج *</label>
                <select className="input-field" value={form.product_id} onChange={(e) => set("product_id", Number(e.target.value))} required aria-label="المنتج">
                  <option value={0}>اختر المنتج</option>
                  {products.map((p) => (
                    <option key={p.id} value={p.id}>{p.name_ar || p.name_en}</option>
                  ))}
                </select>
              </div>
              <div className="input-group">
                <label className="input-label">المادة الخام *</label>
                <select className="input-field" value={form.item_id} onChange={(e) => set("item_id", Number(e.target.value))} required aria-label="المادة الخام">
                  <option value={0}>اختر المادة الخام</option>
                  {items.map((i) => (
                    <option key={i.id} value={i.id}>{i.name_ar || i.name_en}</option>
                  ))}
                </select>
              </div>
              <div className="input-group">
                <label className="input-label">الكمية لكل كرتون</label>
                <input className="input-field" type="number" step="0.01" value={form.qty_per_carton} onChange={(e) => set("qty_per_carton", Number(e.target.value))} required aria-label="الكمية لكل كرتون" />
              </div>
              <div className="input-group">
                <label className="input-label">نسبة التالف %</label>
                <input className="input-field" type="number" step="0.01" min="0" max="100" value={form.waste_pct} onChange={(e) => set("waste_pct", Number(e.target.value))} aria-label="نسبة التالف" />
              </div>
            </div>
            <div className="flex justify-start gap-3 mt-6">
              <Button type="submit" loading={saving} icon={<Save className="w-4 h-4" />}>حفظ</Button>
              <Button variant="outline" type="button" onClick={() => setShowForm(false)}>إلغاء</Button>
            </div>
          </form>
        </Card>
      )}

      <DataTable columns={columns} data={boms} loading={loading} emptyMessage="لا توجد BOM" />

      <ConfirmDialog
        open={showConfirm}
        title="إضافة BOM جديد"
        message="هل أنت متأكد من إضافة Bill of Materials جديد؟"
        variant="warning"
        onConfirm={() => { handleSubmit(); setShowConfirm(false); }}
        onCancel={() => setShowConfirm(false)}
      />
    </div>
  );
}
