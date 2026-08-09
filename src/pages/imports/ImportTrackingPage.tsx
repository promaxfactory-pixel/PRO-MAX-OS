import { useState, useEffect, useCallback, useMemo } from "react";
import { useTranslation } from "react-i18next";
import DataTable, { Column } from "@/components/ui/DataTable";
import Card, { StatCard } from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import FieldError from "@/components/ui/FieldError";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { useUIStore } from "@/stores/uiStore";
import { useAuthStore } from "@/stores/authStore";
import { validate, requiredPick, nonNegative, hasErrors, clearError } from "@/lib/validation";
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
  customs_clearance_date: string;
  duty_amount_milli: number;
  vat_on_import_milli: number;
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
  customs_clearance_date: "",
  actual_arrival: "",
  duty_amount_milli: 0,
  vat_on_import_milli: 0,
  total_landed_cost_milli: 0,
};

export default function ImportTrackingPage() {
  const { t } = useTranslation();
  const { addNotification } = useUIStore();
  const currentUser = useAuthStore((s) => s.user);
  const [shipments, setShipments] = useState<Shipment[]>([]);
  const [loading, setLoading] = useState(true);
  const [suppliers, setSuppliers] = useState<SupplierOption[]>([]);
  const [showForm, setShowForm] = useState(false);
  const [form, setForm] = useState(EMPTY_FORM);
  const [submitting, setSubmitting] = useState(false);
  const [formErrors, setFormErrors] = useState<Record<string, string>>({});
  const [selectedShipment, setSelectedShipment] = useState<Shipment | null>(null);
  const [statusForm, setStatusForm] = useState(EMPTY_STATUS_FORM);
  const [updating, setUpdating] = useState(false);

  const STATUS_LABELS: Record<string, string> = {
    Ordered: t("imports.status.ordered"),
    "In Transit": t("imports.status.inTransit"),
    "At Port": t("imports.status.atPort"),
    "Under Customs": t("imports.status.underCustoms"),
    Cleared: t("imports.status.cleared"),
    Delivered: t("imports.status.delivered"),
  };

  const loadShipments = useCallback(async () => {
    setLoading(true);
    try {
      const d = await invoke<Shipment[]>("list_shipments");
      setShipments(d);
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("imports.errorLoad", { error: String(err) }) });
    } finally {
      setLoading(false);
    }
  }, [addNotification, t]);

  const loadSuppliers = useCallback(async () => {
    try {
      const d = await invoke<SupplierOption[]>("list_suppliers");
      setSuppliers(d);
    } catch { /* ignore */ }
  }, []);

  const setField = (key: keyof typeof EMPTY_FORM, val: string | number) => {
    setForm((f) => ({ ...f, [key]: val }));
    setFormErrors((prev) => clearError(prev, key as string));
  };

  useEffect(() => {
    loadShipments();
    loadSuppliers();
  }, [loadShipments, loadSuppliers]);

  const handleCreate = async () => {
    const errs = validate(
      { supplier_id: form.supplier_id, gross_weight_kg: form.gross_weight_kg, cbm: form.cbm },
      {
        supplier_id: [requiredPick(t("imports.errors.supplierRequired"))],
        gross_weight_kg: [nonNegative(t("imports.errors.weightNonNegative"))],
        cbm: [nonNegative(t("imports.errors.cbmNonNegative"))],
      },
    );
    if (hasErrors(errs)) {
      setFormErrors(errs);
      addNotification({ id: crypto.randomUUID(), type: "warning", title: t("common.warning"), message: t("qualityForm.notifications.completeRequiredData") });
      return;
    }
    setSubmitting(true);
    try {
      await invoke("create_shipment", { input: { ...form, created_by: currentUser?.username ?? null } });
      setShowForm(false);
      setForm(EMPTY_FORM);
      await loadShipments();
      addNotification({ id: crypto.randomUUID(), type: "success", title: t("common.success"), message: t("imports.createdSuccess") });
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("imports.createFailed", { error: String(err) }) });
    }
    setSubmitting(false);
  };

  const handleUpdateStatus = async () => {
    if (!selectedShipment) return;
    setUpdating(true);
    try {
      await invoke("update_shipment_status", {
        id: selectedShipment.id,
        input: {
          status: statusForm.status,
          customs_declaration_no: statusForm.customs_declaration_no || null,
          customs_clearance_date: statusForm.customs_clearance_date || null,
          actual_arrival: statusForm.actual_arrival || null,
          duty_amount_milli: statusForm.duty_amount_milli || null,
          vat_on_import_milli: statusForm.vat_on_import_milli || null,
          total_landed_cost_milli: statusForm.total_landed_cost_milli || null,
        },
      });
      setSelectedShipment(null);
      setStatusForm({
        status: "",
        customs_declaration_no: "",
        customs_clearance_date: "",
        actual_arrival: "",
        duty_amount_milli: 0,
        vat_on_import_milli: 0,
        total_landed_cost_milli: 0,
      });
      await loadShipments();
      addNotification({ id: crypto.randomUUID(), type: "success", title: t("common.success"), message: t("imports.statusUpdated") });
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("imports.statusUpdateFailed", { error: String(err) }) });
    }
    setUpdating(false);
  };

  const totalShipments = shipments.length;
  const inTransit = shipments.filter((s) => s.status === "In Transit").length;
  const underCustoms = shipments.filter((s) => s.status === "Under Customs").length;
  const cleared = shipments.filter((s) => ["Cleared", "Delivered"].includes(s.status)).length;

  const columns: Column<Shipment>[] = useMemo(() => [
    { key: "shipment_no", header: t("imports.shipmentNo"), sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.shipment_no || "—"}</span> },
    { key: "supplier_name", header: t("print.supplierLabel"), sortable: true, render: (r) => r.supplier_name || "—" },
    { key: "status", header: t("common.status"), render: (r) => (
      <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${STATUS_BADGE[r.status] || "bg-surface-600 text-surface-300"}`}>
        {STATUS_LABELS[r.status] || r.status}
      </span>
    )},
    { key: "container_no", header: t("imports.containerNo"), render: (r) => r.container_no || "—" },
    { key: "bl_no", header: t("imports.blNo"), render: (r) => r.bl_no || "—" },
    { key: "estimated_arrival", header: t("imports.estimatedArrival"), sortable: true, render: (r) => formatDate(r.estimated_arrival) },
    { key: "total_landed_cost_milli", header: t("imports.totalCost"), sortable: true, align: "left", render: (r) => (
      <span className="font-bold text-gold-400">{r.total_landed_cost_milli ? formatOMR(r.total_landed_cost_milli) : "—"}</span>
    )},
  ], [t]);

  return (
    <div className="space-y-6" dir="rtl">
      <div className="page-header">
        <div>
          <h1 className="page-title">{t("imports.title")}</h1>
          <p className="page-subtitle">{t("imports.subtitle", { count: totalShipments })}</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => setShowForm(!showForm)}>{t("imports.newShipment")}</Button>
      </div>

      <div className="grid grid-cols-4 gap-4">
        <StatCard title={t("imports.totalShipments")} value={totalShipments} icon={<Ship className="w-6 h-6" />} />
        <StatCard title={t("imports.status.inTransit")} value={inTransit} icon={<Truck className="w-6 h-6" />} />
        <StatCard title={t("imports.status.underCustoms")} value={underCustoms} icon={<Anchor className="w-6 h-6" />} />
        <StatCard title={t("imports.status.cleared")} value={cleared} icon={<CheckCircle2 className="w-6 h-6" />} />
      </div>

      {showForm && (
        <Card className="border-brand-500/30">
          <h3 className="section-title mb-4"><Plus className="w-4 h-4" /> {t("imports.newShipment")}</h3>
          <div className="grid grid-cols-3 gap-4">
            <div>
              <label className="form-label">{t("imports.supplierRequired")}</label>
              <select className="input-field" value={form.supplier_id} onChange={(e) => setField("supplier_id", Number(e.target.value))} aria-label={t("print.supplierLabel")}>
                <option value={0}>{t("imports.selectSupplier")}</option>
                {suppliers.map((s) => <option key={s.id} value={s.id}>{s.name}</option>)}
              </select>
              <FieldError message={formErrors.supplier_id} />
            </div>
            <div>
              <label className="form-label">{t("imports.currency")}</label>
              <input className="input-field" value={form.currency} onChange={(e) => setField("currency", e.target.value)} placeholder={t("imports.currencyPlaceholder")} aria-label={t("imports.currency")} />
            </div>
            <div>
              <label className="form-label">{t("imports.shippingCompany")}</label>
              <input className="input-field" value={form.shipping_company} onChange={(e) => setField("shipping_company", e.target.value)} aria-label={t("imports.shippingCompany")} />
            </div>
            <div>
              <label className="form-label">{t("imports.containerNo")}</label>
              <input className="input-field" value={form.container_no} onChange={(e) => setField("container_no", e.target.value)} aria-label={t("imports.containerNo")} />
            </div>
            <div>
              <label className="form-label">{t("imports.blNo")}</label>
              <input className="input-field" value={form.bl_no} onChange={(e) => setField("bl_no", e.target.value)} aria-label={t("imports.blNo")} />
            </div>
            <div>
              <label className="form-label">{t("imports.vesselFlight")}</label>
              <input className="input-field" value={form.vessel_flight} onChange={(e) => setField("vessel_flight", e.target.value)} aria-label={t("imports.vesselAria")} />
            </div>
            <div>
              <label className="form-label">{t("imports.portOfLoading")}</label>
              <input className="input-field" value={form.port_of_loading} onChange={(e) => setField("port_of_loading", e.target.value)} aria-label={t("imports.portOfLoading")} />
            </div>
            <div>
              <label className="form-label">{t("imports.portOfDischarge")}</label>
              <input className="input-field" value={form.port_of_discharge} onChange={(e) => setField("port_of_discharge", e.target.value)} aria-label={t("imports.portOfDischarge")} />
            </div>
            <div>
              <label className="form-label">{t("imports.estimatedArrival")}</label>
              <input type="date" className="input-field" value={form.estimated_arrival} onChange={(e) => setField("estimated_arrival", e.target.value)} aria-label={t("imports.estimatedArrival")} />
            </div>
            <div>
              <label className="form-label">{t("imports.commercialInvoiceNo")}</label>
              <input className="input-field" value={form.commercial_invoice_no} onChange={(e) => setField("commercial_invoice_no", e.target.value)} aria-label={t("invoice.invoiceNo")} />
            </div>
            <div>
              <label className="form-label">{t("imports.packingListNo")}</label>
              <input className="input-field" value={form.packing_list_no} onChange={(e) => setField("packing_list_no", e.target.value)} aria-label={t("imports.packingList")} />
            </div>
            <div>
              <label className="form-label">{t("imports.originCountry")}</label>
              <input className="input-field" value={form.origin_country} onChange={(e) => setField("origin_country", e.target.value)} aria-label={t("imports.originCountry")} />
            </div>
            <div>
              <label className="form-label">{t("imports.grossWeightKg")}</label>
              <input type="number" className="input-field" value={form.gross_weight_kg || ""} onChange={(e) => setField("gross_weight_kg", Number(e.target.value) || 0)} aria-label={t("imports.grossWeightAria")} />
              <FieldError message={formErrors.gross_weight_kg} />
            </div>
            <div>
              <label className="form-label">{t("imports.cbm")}</label>
              <input type="number" className="input-field" value={form.cbm || ""} onChange={(e) => setField("cbm", Number(e.target.value) || 0)} aria-label={t("imports.cbm")} />
              <FieldError message={formErrors.cbm} />
            </div>
            <div>
              <label className="form-label">{t("imports.clearanceAgent")}</label>
              <input className="input-field" value={form.clearance_agent} onChange={(e) => setField("clearance_agent", e.target.value)} aria-label={t("imports.clearanceAgent")} />
            </div>
            <div className="col-span-3">
              <label className="form-label">{t("common.notes")}</label>
              <textarea className="input-field" rows={2} value={form.notes} onChange={(e) => setField("notes", e.target.value)} aria-label={t("common.notes")} />
            </div>
          </div>
          <div className="flex justify-end gap-2 mt-4">
            <Button variant="ghost" onClick={() => setShowForm(false)}>{t("common.cancel")}</Button>
            <Button variant="gold" loading={submitting} onClick={handleCreate} disabled={!form.supplier_id}>{t("imports.createShipment")}</Button>
          </div>
        </Card>
      )}

      <DataTable columns={columns} data={shipments} loading={loading} onRowClick={(r) => { setSelectedShipment(r); setStatusForm({ status: r.status, customs_declaration_no: r.customs_declaration_no || "", customs_clearance_date: r.customs_clearance_date || "", actual_arrival: r.actual_arrival || "", duty_amount_milli: r.duty_amount_milli || 0, vat_on_import_milli: r.vat_on_import_milli || 0, total_landed_cost_milli: r.total_landed_cost_milli || 0 }); }} emptyMessage={t("imports.empty")} />

      {selectedShipment && (
        <Card className="border-gold-500/30">
          <h3 className="section-title mb-4"><Truck className="w-4 h-4" /> {t("imports.updateStatusTitle", { shipmentNo: selectedShipment.shipment_no })}</h3>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="form-label">{t("imports.statusRequired")}</label>
              <select className="input-field" value={statusForm.status} onChange={(e) => setStatusForm({ ...statusForm, status: e.target.value })} aria-label={t("common.status")}>
                <option value="">{t("imports.selectStatus")}</option>
                {STATUS_OPTIONS.map((s) => <option key={s} value={s}>{STATUS_LABELS[s]}</option>)}
              </select>
            </div>
            <div>
              <label className="form-label">{t("imports.customsDeclarationNo")}</label>
              <input className="input-field" value={statusForm.customs_declaration_no} onChange={(e) => setStatusForm({ ...statusForm, customs_declaration_no: e.target.value })} aria-label={t("imports.customsDeclarationNo")} />
            </div>
            <div>
              <label className="form-label">{t("imports.customsClearanceDate")}</label>
              <input type="date" className="input-field" value={statusForm.customs_clearance_date} onChange={(e) => setStatusForm({ ...statusForm, customs_clearance_date: e.target.value })} aria-label={t("imports.customsClearanceDate")} />
            </div>
            <div>
              <label className="form-label">{t("imports.actualArrival")}</label>
              <input type="date" className="input-field" value={statusForm.actual_arrival} onChange={(e) => setStatusForm({ ...statusForm, actual_arrival: e.target.value })} aria-label={t("imports.actualArrival")} />
            </div>
            <div>
              <label className="form-label">{t("imports.dutyAmount")}</label>
              <input type="number" className="input-field" value={statusForm.duty_amount_milli || ""} onChange={(e) => setStatusForm({ ...statusForm, duty_amount_milli: Number(e.target.value) || 0 })} aria-label={t("imports.dutyAmount")} />
            </div>
            <div>
              <label className="form-label">{t("imports.vatOnImport")}</label>
              <input type="number" className="input-field" value={statusForm.vat_on_import_milli || ""} onChange={(e) => setStatusForm({ ...statusForm, vat_on_import_milli: Number(e.target.value) || 0 })} aria-label={t("imports.vatOnImport")} />
            </div>
            <div>
              <label className="form-label">{t("imports.totalLandedCostMilli")}</label>
              <input type="number" className="input-field" value={statusForm.total_landed_cost_milli || ""} onChange={(e) => setStatusForm({ ...statusForm, total_landed_cost_milli: Number(e.target.value) || 0 })} aria-label={t("imports.totalCost")} />
            </div>
          </div>
          <div className="flex justify-end gap-2 mt-4">
            <Button variant="ghost" onClick={() => setSelectedShipment(null)}>{t("common.cancel")}</Button>
            <Button variant="gold" loading={updating} onClick={handleUpdateStatus} disabled={!statusForm.status}>{t("imports.updateStatus")}</Button>
          </div>
        </Card>
      )}
    </div>
  );
}
