import { useState, useEffect, useCallback, useMemo } from "react";
import DataTable, { Column } from "@/components/ui/DataTable";
import Card, { StatCard } from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@/lib/tauri";
import { useUIStore } from "@/stores/uiStore";
import { Ship, Plus, Truck, CheckCircle2, Anchor } from "lucide-react";

interface Shipment {
  id: number;
  shipment_no: string;
  supplier_id: number;
  supplier_name: string;
  status: string;
  currency: string;
  shipping_company: string;
  container_no: string;
  bl_no: string;
  vessel_flight: string;
  port_of_loading: string;
  port_of_discharge: string;
  estimated_arrival: string;
  actual_arrival: string;
  commercial_invoice_no: string;
  packing_list_no: string;
  origin_country: string;
  gross_weight_kg: number;
  cbm: number;
  clearance_agent: string;
  customs_declaration_no: string;
  customs_release_date: string;
  duty_amount_milli: number;
  total_landed_cost_milli: number;
  notes: string;
  created_by: string;
  created_at: string;
}

interface SupplierOption {
  id: number;
  name: string;
}

const STATUS_OPTIONS = ["Ordered", "In Transit", "At Port", "Under Customs", "Cleared", "Delivered"] as const;

const STATUS_LABELS: Record<string, string> = {
  Ordered: "تم الطلب",
  "In Transit": "في الطريق",
  "At Port": "في الميناء",
  "Under Customs": "تحت الجمارك",
  Cleared: "تم التخليص",
  Delivered: "تم التسليم",
};

const STATUS_BADGE: Record<string, string> = {
  Ordered: "bg-blue-500/20 text-blue-400",
  "In Transit": "bg-amber-500/20 text-amber-400",
  "At Port": "bg-purple-500/20 text-purple-400",
  "Under Customs": "bg-orange-500/20 text-orange-400",
  Cleared: "bg-emerald-500/20 text-emerald-400",
  Delivered: "bg-surface-600 text-surface-300",
};

const EMPTY_FORM = {
  supplier_id: 0,
  currency: "",
  shipping_company: "",
  container_no: "",
  bl_no: "",
  vessel_flight: "",
  port_of_loading: "",
  port_of_discharge: "",
  estimated_arrival: "",
  commercial_invoice_no: "",
  packing_list_no: "",
  origin_country: "",
  gross_weight_kg: 0,
  cbm: 0,
  clearance_agent: "",
  notes: "",
};

const EMPTY_STATUS_FORM = {
  status: "",
  customs_declaration_no: "",
  customs_release_date: "",
  duty_amount_milli: 0,
};

export default function ImportTrackingPage() {
  const { addNotification } = useUIStore();
  const [shipments, setShipments] = useState<Shipment[]>([]);
  const [loading, setLoading] = useState(true);
  const [suppliers, setSuppliers] = useState<SupplierOption[]>([]);
  const [showForm, setShowForm] = useState(false);
  const [form, setForm] = useState(EMPTY_FORM);
  const [submitting, setSubmitting] = useState(false);
  const [selectedShipment, setSelectedShipment] = useState<Shipment | null>(null);
  const [statusForm, setStatusForm] = useState(EMPTY_STATUS_FORM);
  const [updating, setUpdating] = useState(false);

  const loadShipments = useCallback(async () => {
    setLoading(true);
    try {
      const d = await invoke<Shipment[]>("list_shipments");
      setShipments(d);
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء تحميل الشحنات: " + String(err) });
    } finally {
      setLoading(false);
    }
  }, [addNotification]);

  const loadSuppliers = useCallback(async () => {
    try {
      const d = await invoke<SupplierOption[]>("list_suppliers");
      setSuppliers(d);
    } catch { /* ignore */ }
  }, []);

  useEffect(() => {
    loadShipments();
    loadSuppliers();
  }, [loadShipments, loadSuppliers]);

  const handleCreate = async () => {
    if (!form.supplier_id) return;
    setSubmitting(true);
    try {
      await invoke("create_shipment", { input: form });
      setShowForm(false);
      setForm(EMPTY_FORM);
      await loadShipments();
      addNotification({ id: crypto.randomUUID(), type: "success", title: "تم", message: "تم إنشاء الشحنة بنجاح" });
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "فشل إنشاء الشحنة: " + String(err) });
    }
    setSubmitting(false);
  };

  const handleUpdateStatus = async () => {
    if (!selectedShipment) return;
    setUpdating(true);
    try {
      await invoke("update_shipment_status", {
        shipmentId: selectedShipment.id,
        status: statusForm.status,
        customsDeclarationNo: statusForm.customs_declaration_no || null,
        customsReleaseDate: statusForm.customs_release_date || null,
        dutyAmountMilli: statusForm.duty_amount_milli || null,
      });
      setSelectedShipment(null);
      setStatusForm(EMPTY_STATUS_FORM);
      await loadShipments();
      addNotification({ id: crypto.randomUUID(), type: "success", title: "تم", message: "تم تحديث الحالة بنجاح" });
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "فشل تحديث الحالة: " + String(err) });
    }
    setUpdating(false);
  };

  const totalShipments = shipments.length;
  const inTransit = shipments.filter((s) => s.status === "In Transit").length;
  const underCustoms = shipments.filter((s) => s.status === "Under Customs").length;
  const cleared = shipments.filter((s) => ["Cleared", "Delivered"].includes(s.status)).length;

  const columns: Column<Shipment>[] = useMemo(() => [
    { key: "shipment_no", header: "رقم الشحنة", sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.shipment_no || "—"}</span> },
    { key: "supplier_name", header: "المورد", sortable: true, render: (r) => r.supplier_name || "—" },
    { key: "status", header: "الحالة", render: (r) => (
      <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${STATUS_BADGE[r.status] || "bg-surface-600 text-surface-300"}`}>
        {STATUS_LABELS[r.status] || r.status}
      </span>
    )},
    { key: "container_no", header: "رقم الحاوية", render: (r) => r.container_no || "—" },
    { key: "bl_no", header: "رقم B/L", render: (r) => r.bl_no || "—" },
    { key: "estimated_arrival", header: "الوصول المتوقع", sortable: true, render: (r) => formatDate(r.estimated_arrival) },
    { key: "total_landed_cost_milli", header: "التكلفة الإجمالية", sortable: true, align: "left", render: (r) => (
      <span className="font-bold text-gold-400">{r.total_landed_cost_milli ? formatOMR(r.total_landed_cost_milli) : "—"}</span>
    )},
  ], []);

  return (
    <div className="space-y-6" dir="rtl">
      <div className="page-header">
        <div>
          <h1 className="page-title">تتبع الاستيراد</h1>
          <p className="page-subtitle">{totalShipments} شحنة مسجلة</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => setShowForm(!showForm)}>شحنة جديدة</Button>
      </div>

      <div className="grid grid-cols-4 gap-4">
        <StatCard title="إجمالي الشحنات" value={totalShipments} icon={<Ship className="w-6 h-6" />} />
        <StatCard title="في الطريق" value={inTransit} icon={<Truck className="w-6 h-6" />} />
        <StatCard title="تحت الجمارك" value={underCustoms} icon={<Anchor className="w-6 h-6" />} />
        <StatCard title="تم التخليص" value={cleared} icon={<CheckCircle2 className="w-6 h-6" />} />
      </div>

      {showForm && (
        <Card className="border-brand-500/30">
          <h3 className="section-title mb-4"><Plus className="w-4 h-4" /> شحنة جديدة</h3>
          <div className="grid grid-cols-3 gap-4">
            <div>
              <label className="form-label">المورد *</label>
              <select className="input-field" value={form.supplier_id} onChange={(e) => setForm({ ...form, supplier_id: Number(e.target.value) })} aria-label="المورد">
                <option value={0}>— اختر المورد —</option>
                {suppliers.map((s) => <option key={s.id} value={s.id}>{s.name}</option>)}
              </select>
            </div>
            <div>
              <label className="form-label">العملة</label>
              <input className="input-field" value={form.currency} onChange={(e) => setForm({ ...form, currency: e.target.value })} placeholder="مثال: USD" aria-label="العملة" />
            </div>
            <div>
              <label className="form-label">شركة الشحن</label>
              <input className="input-field" value={form.shipping_company} onChange={(e) => setForm({ ...form, shipping_company: e.target.value })} aria-label="شركة الشحن" />
            </div>
            <div>
              <label className="form-label">رقم الحاوية</label>
              <input className="input-field" value={form.container_no} onChange={(e) => setForm({ ...form, container_no: e.target.value })} aria-label="رقم الحاوية" />
            </div>
            <div>
              <label className="form-label">رقم B/L</label>
              <input className="input-field" value={form.bl_no} onChange={(e) => setForm({ ...form, bl_no: e.target.value })} aria-label="رقم B/L" />
            </div>
            <div>
              <label className="form-label">السفينة / الرحلة</label>
              <input className="input-field" value={form.vessel_flight} onChange={(e) => setForm({ ...form, vessel_flight: e.target.value })} aria-label="السفينة" />
            </div>
            <div>
              <label className="form-label">ميناء الشحن</label>
              <input className="input-field" value={form.port_of_loading} onChange={(e) => setForm({ ...form, port_of_loading: e.target.value })} aria-label="ميناء الشحن" />
            </div>
            <div>
              <label className="form-label">ميناء التفريغ</label>
              <input className="input-field" value={form.port_of_discharge} onChange={(e) => setForm({ ...form, port_of_discharge: e.target.value })} aria-label="ميناء التفريغ" />
            </div>
            <div>
              <label className="form-label">الوصول المتوقع</label>
              <input type="date" className="input-field" value={form.estimated_arrival} onChange={(e) => setForm({ ...form, estimated_arrival: e.target.value })} aria-label="الوصول المتوقع" />
            </div>
            <div>
              <label className="form-label">رقم الفاتورة التجارية</label>
              <input className="input-field" value={form.commercial_invoice_no} onChange={(e) => setForm({ ...form, commercial_invoice_no: e.target.value })} aria-label="رقم الفاتورة" />
            </div>
            <div>
              <label className="form-label">رقم قائمة التعبئة</label>
              <input className="input-field" value={form.packing_list_no} onChange={(e) => setForm({ ...form, packing_list_no: e.target.value })} aria-label="قائمة التعبئة" />
            </div>
            <div>
              <label className="form-label">بلد المنشأ</label>
              <input className="input-field" value={form.origin_country} onChange={(e) => setForm({ ...form, origin_country: e.target.value })} aria-label="بلد المنشأ" />
            </div>
            <div>
              <label className="form-label">الوزن الإجمالي (كغ)</label>
              <input type="number" className="input-field" value={form.gross_weight_kg || ""} onChange={(e) => setForm({ ...form, gross_weight_kg: Number(e.target.value) || 0 })} aria-label="الوزن" />
            </div>
            <div>
              <label className="form-label">CBM</label>
              <input type="number" className="input-field" value={form.cbm || ""} onChange={(e) => setForm({ ...form, cbm: Number(e.target.value) || 0 })} aria-label="CBM" />
            </div>
            <div>
              <label className="form-label">وكيل التخليص</label>
              <input className="input-field" value={form.clearance_agent} onChange={(e) => setForm({ ...form, clearance_agent: e.target.value })} aria-label="وكيل التخليص" />
            </div>
            <div className="col-span-3">
              <label className="form-label">ملاحظات</label>
              <textarea className="input-field" rows={2} value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} aria-label="ملاحظات" />
            </div>
          </div>
          <div className="flex justify-end gap-2 mt-4">
            <Button variant="ghost" onClick={() => setShowForm(false)}>إلغاء</Button>
            <Button variant="gold" loading={submitting} onClick={handleCreate} disabled={!form.supplier_id}>إنشاء الشحنة</Button>
          </div>
        </Card>
      )}

      <DataTable columns={columns} data={shipments} loading={loading} onRowClick={(r) => { setSelectedShipment(r); setStatusForm({ status: r.status, customs_declaration_no: r.customs_declaration_no || "", customs_release_date: r.customs_release_date || "", duty_amount_milli: r.duty_amount_milli || 0 }); }} emptyMessage="لا توجد شحنات" />

      {selectedShipment && (
        <Card className="border-gold-500/30">
          <h3 className="section-title mb-4"><Truck className="w-4 h-4" /> تحديث حالة الشحنة — {selectedShipment.shipment_no}</h3>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="form-label">الحالة *</label>
              <select className="input-field" value={statusForm.status} onChange={(e) => setStatusForm({ ...statusForm, status: e.target.value })} aria-label="الحالة">
                <option value="">— اختر الحالة —</option>
                {STATUS_OPTIONS.map((s) => <option key={s} value={s}>{STATUS_LABELS[s]}</option>)}
              </select>
            </div>
            <div>
              <label className="form-label">رقم البيان الجمركي</label>
              <input className="input-field" value={statusForm.customs_declaration_no} onChange={(e) => setStatusForm({ ...statusForm, customs_declaration_no: e.target.value })} aria-label="رقم البيان" />
            </div>
            <div>
              <label className="form-label">تاريخ الإفراج الجمركي</label>
              <input type="date" className="input-field" value={statusForm.customs_release_date} onChange={(e) => setStatusForm({ ...statusForm, customs_release_date: e.target.value })} aria-label="تاريخ الإفراج" />
            </div>
            <div>
              <label className="form-label">مبلغ الرسوم (ملي)</label>
              <input type="number" className="input-field" value={statusForm.duty_amount_milli || ""} onChange={(e) => setStatusForm({ ...statusForm, duty_amount_milli: Number(e.target.value) || 0 })} aria-label="الرسوم" />
            </div>
          </div>
          <div className="flex justify-end gap-2 mt-4">
            <Button variant="ghost" onClick={() => setSelectedShipment(null)}>إلغاء</Button>
            <Button variant="gold" loading={updating} onClick={handleUpdateStatus} disabled={!statusForm.status}>تحديث الحالة</Button>
          </div>
        </Card>
      )}
    </div>
  );
}
