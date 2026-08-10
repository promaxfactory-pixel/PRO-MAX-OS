import { useState, useEffect, useCallback, useMemo } from "react";
import DataTable, { Column } from "@/components/ui/DataTable";
import Card, { StatCard } from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { useUIStore } from "@/stores/uiStore";
import { ArrowLeftRight, Plus, Scale, CheckCircle2 } from "lucide-react";

interface BarterExchange {
  id: number;
  exchange_no: string;
  date: string;
  supplier_id: number;
  supplier_name: string;
  product_id: number;
  product_name: string;
  cartons_given: number;
  carton_value_milli: number;
  received_item_id: number;
  received_item_name: string;
  bags_received: number;
  bag_value_milli: number;
  net_value_milli: number;
  settlement_status: string;
  reference: string;
  notes: string;
  created_by: string;
  created_at: string;
}

interface SupplierOption {
  id: number;
  name: string;
}

interface ProductOption {
  id: number;
  name_ar: string | null;
  code: string | null;
}

interface InventoryOption {
  id: number;
  name_ar: string;
  code: string;
}

const EMPTY_FORM = {
  local_supplier_id: 0,
  product_id: 0,
  cartons_given: 0,
  carton_value_milli: 0,
  received_item_id: 0,
  bags_received: 0,
  bag_value_milli: 0,
  reference: "",
  notes: "",
};

export default function BarterExchangePage() {
  const { addNotification } = useUIStore();
  const [exchanges, setExchanges] = useState<BarterExchange[]>([]);
  const [loading, setLoading] = useState(true);
  const [suppliers, setSuppliers] = useState<SupplierOption[]>([]);
  const [products, setProducts] = useState<ProductOption[]>([]);
  const [inventoryItems, setInventoryItems] = useState<InventoryOption[]>([]);
  const [showForm, setShowForm] = useState(false);
  const [form, setForm] = useState(EMPTY_FORM);
  const [submitting, setSubmitting] = useState(false);
  const [selectedSupplier, setSelectedSupplier] = useState<number | null>(null);
  const [supplierBalance, setSupplierBalance] = useState<number | null>(null);

  const loadExchanges = useCallback(async () => {
    setLoading(true);
    try {
      const d = await invoke<BarterExchange[]>("list_barter_exchanges");
      setExchanges(d);
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء تحميل البيانات: " + String(err) });
    } finally {
      setLoading(false);
    }
  }, [addNotification]);

  const loadLookups = useCallback(async () => {
    try {
      const [sup, prods, inv] = await Promise.all([
        invoke<SupplierOption[]>("list_suppliers"),
        invoke<ProductOption[]>("list_products", {}),
        invoke<InventoryOption[]>("list_inventory_items"),
      ]);
      setSuppliers(sup);
      setProducts(prods.filter((p) => p.name_ar));
      setInventoryItems(inv);
    } catch { /* ignore */ }
  }, []);

  useEffect(() => {
    loadExchanges();
    loadLookups();
  }, [loadExchanges, loadLookups]);

  useEffect(() => {
    if (!selectedSupplier) { setSupplierBalance(null); return; }
    invoke<number>("get_barter_balance", { supplierId: selectedSupplier })
      .then((b) => setSupplierBalance(b))
      .catch(() => setSupplierBalance(null));
  }, [selectedSupplier]);

  const handleCreate = async () => {
    if (!form.local_supplier_id || !form.product_id) return;
    setSubmitting(true);
    try {
      await invoke("create_barter_exchange", { input: form });
      setShowForm(false);
      setForm(EMPTY_FORM);
      await loadExchanges();
      addNotification({ id: crypto.randomUUID(), type: "success", title: "تم", message: "تم إنشاء عملية المقايضة بنجاح" });
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "فشل إنشاء المقايضة: " + String(err) });
    }
    setSubmitting(false);
  };

  const totalExchanges = exchanges.length;
  const openBalance = exchanges.filter((e) => e.settlement_status === "Open").reduce((s, e) => s + e.net_value_milli, 0);
  const settledCount = exchanges.filter((e) => e.settlement_status === "Settled").length;

  const columns: Column<BarterExchange>[] = useMemo(() => [
    { key: "exchange_no", header: "رقم العملية", sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.exchange_no || "—"}</span> },
    { key: "date", header: "التاريخ", sortable: true, render: (r) => formatDate(r.date) },
    { key: "supplier_name", header: "المورد", sortable: true, render: (r) => r.supplier_name || "—" },
    { key: "product_name", header: "المنتج", render: (r) => r.product_name || "—" },
    { key: "cartons_given", header: "كرتون معطى", sortable: true, align: "center", render: (r) => r.cartons_given },
    { key: "carton_value_milli", header: "قيمة الكرتون", align: "left", render: (r) => formatOMR(r.carton_value_milli) },
    { key: "bags_received", header: "أكياس مستلمة", sortable: true, align: "center", render: (r) => r.bags_received },
    { key: "bag_value_milli", header: "قيمة الكيس", align: "left", render: (r) => formatOMR(r.bag_value_milli) },
    { key: "net_value_milli", header: "الصافي", sortable: true, align: "left", render: (r) => (
      <span className={`font-bold ${r.net_value_milli >= 0 ? "text-gold-400" : "text-red-400"}`}>{formatOMR(r.net_value_milli)}</span>
    )},
    { key: "settlement_status", header: "الحالة", render: (r) => (
      <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
        r.settlement_status === "Settled" ? "bg-emerald-500/20 text-emerald-400" : "bg-amber-500/20 text-amber-400"
      }`}>{r.settlement_status === "Settled" ? "مسوية" : "مفتوحة"}</span>
    )},
  ], []);

  return (
    <div className="space-y-6" dir="rtl">
      <div className="page-header">
        <div>
          <h1 className="page-title">عمليات المقايضة</h1>
          <p className="page-subtitle">{totalExchanges} عملية مسجلة</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => setShowForm(!showForm)}>مقايضة جديدة</Button>
      </div>

      <div className="grid grid-cols-3 gap-4">
        <StatCard title="إجمالي العمليات" value={totalExchanges} icon={<ArrowLeftRight className="w-6 h-6" />} />
        <StatCard title="الرصيد المفتوح" value={formatOMR(openBalance)} icon={<Scale className="w-6 h-6" />} />
        <StatCard title="العمليات المسوية" value={settledCount} icon={<CheckCircle2 className="w-6 h-6" />} />
      </div>

      {selectedSupplier && supplierBalance !== null && (
        <Card className="border-brand-500/30">
          <p className="text-sm text-surface-400">رصيد المقايضة للمورد المحدد: <span className="font-bold text-gold-400">{formatOMR(supplierBalance)}</span></p>
        </Card>
      )}

      {showForm && (
        <Card className="border-brand-500/30">
          <h3 className="section-title mb-4"><Plus className="w-4 h-4" /> مقايضة جديدة</h3>
          <div className="grid grid-cols-3 gap-4">
            <div>
              <label className="form-label">المورد المحلي *</label>
              <select className="input-field" value={form.local_supplier_id} onChange={(e) => { const v = Number(e.target.value); setForm({ ...form, local_supplier_id: v }); setSelectedSupplier(v || null); }} aria-label="المورد">
                <option value={0}>— اختر المورد —</option>
                {suppliers.map((s) => <option key={s.id} value={s.id}>{s.name}</option>)}
              </select>
            </div>
            <div>
              <label className="form-label">المنتج (كرتون) *</label>
              <select className="input-field" value={form.product_id} onChange={(e) => setForm({ ...form, product_id: Number(e.target.value) })} aria-label="المنتج">
                <option value={0}>— اختر المنتج —</option>
                {products.map((p) => <option key={p.id} value={p.id}>{p.name_ar || p.code}</option>)}
              </select>
            </div>
            <div>
              <label className="form-label">عدد الكراتين</label>
              <input type="number" className="input-field" value={form.cartons_given || ""} onChange={(e) => setForm({ ...form, cartons_given: Number(e.target.value) || 0 })} aria-label="الكراتين" />
            </div>
            <div>
              <label className="form-label">قيمة الكرتون (ملي)</label>
              <input type="number" className="input-field" value={form.carton_value_milli || ""} onChange={(e) => setForm({ ...form, carton_value_milli: Number(e.target.value) || 0 })} aria-label="قيمة الكرتون" />
            </div>
            <div>
              <label className="form-label">البند المستلم</label>
              <select className="input-field" value={form.received_item_id} onChange={(e) => setForm({ ...form, received_item_id: Number(e.target.value) })} aria-label="البند المستلم">
                <option value={0}>— اختر البند —</option>
                {inventoryItems.map((i) => <option key={i.id} value={i.id}>{i.name_ar} ({i.code})</option>)}
              </select>
            </div>
            <div>
              <label className="form-label">عدد الأكياس</label>
              <input type="number" className="input-field" value={form.bags_received || ""} onChange={(e) => setForm({ ...form, bags_received: Number(e.target.value) || 0 })} aria-label="الأكياس" />
            </div>
            <div>
              <label className="form-label">قيمة الكيس (ملي)</label>
              <input type="number" className="input-field" value={form.bag_value_milli || ""} onChange={(e) => setForm({ ...form, bag_value_milli: Number(e.target.value) || 0 })} aria-label="قيمة الكيس" />
            </div>
            <div>
              <label className="form-label">المرجع</label>
              <input className="input-field" value={form.reference} onChange={(e) => setForm({ ...form, reference: e.target.value })} aria-label="المرجع" />
            </div>
            <div className="col-span-3">
              <label className="form-label">ملاحظات</label>
              <textarea className="input-field" rows={2} value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} aria-label="ملاحظات" />
            </div>
          </div>
          <div className="flex justify-end gap-2 mt-4">
            <Button variant="ghost" onClick={() => { setShowForm(false); setSelectedSupplier(null); }}>إلغاء</Button>
            <Button variant="gold" loading={submitting} onClick={handleCreate} disabled={!form.local_supplier_id || !form.product_id}>إنشاء المقايضة</Button>
          </div>
        </Card>
      )}

      <DataTable columns={columns} data={exchanges} loading={loading} emptyMessage="لا توجد عمليات مقايضة" />
    </div>
  );
}
