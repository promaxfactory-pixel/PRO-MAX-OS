import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, Save } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import type { Machine } from "@/types";

export default function MachineFormPage() {
  const { t } = useTranslation();
  const { id } = useParams();
  const navigate = useNavigate();
  const isEdit = Boolean(id);
  const addNotification = useUIStore((s) => s.addNotification);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [form, setForm] = useState({
    name: "",
    code: "",
    mtype: "single_die",
    supported_products: "",
    purchase_date: "",
    supplier: "",
    cost_milli: 0,
    capacity_cpm: 0,
    status: "active",
    notes: "",
  });

  useEffect(() => {
    if (isEdit) {
      setLoading(true);
      invoke<Machine>("get_machine", { id: Number(id) })
        .then((d) => {
          setForm({
            name: d.name || "",
            code: d.code || "",
            mtype: d.mtype || "single_die",
            supported_products: d.supported_products || "",
            purchase_date: d.purchase_date || "",
            supplier: d.supplier || "",
            cost_milli: d.cost_milli || 0,
            capacity_cpm: d.capacity_cpm || 0,
            status: d.status || "active",
            notes: d.notes || "",
          });
        })
        .catch((e: unknown) => addNotification({ title: t("common.error"), message: String(e), type: 'error' }))
        .finally(() => setLoading(false));
    }
  }, [id, isEdit, t]);

  const set = (key: string, val: string | number | boolean) => setForm((f) => ({ ...f, [key]: val }));

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!form.name.trim()) return addNotification({ id: crypto.randomUUID(), type: "warning", title: t("common.warning"), message: t("machineForm.errors.nameRequired") });
    setSaving(true);
    try {
      if (isEdit) {
        await invoke("update_machine", { id: Number(id), input: form });
      } else {
        await invoke("create_machine", { input: form });
      }
      addNotification({ id: crypto.randomUUID(), type: "success", title: t("common.success"), message: t("machineForm.notifications.saved") });
      navigate("/machines");
    } catch (err: unknown) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("machineForm.notifications.saveFailed") });
    } finally {
      setSaving(false);
    }
  };

  if (loading)
    return (
      <div className="flex items-center justify-center h-64">
        <div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" />
      </div>
    );

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div className="flex items-center gap-3">
          <button onClick={() => navigate("/machines")} className="btn-ghost p-2">
            <ArrowRight className="w-5 h-5" />
          </button>
          <div>
            <h1 className="page-title">{isEdit ? t("machineForm.titleEdit") : t("machineForm.titleNew")}</h1>
            <p className="page-subtitle">{isEdit ? t("machineForm.subtitleEdit", { name: form.name }) : t("machineForm.subtitleNew")}</p>
          </div>
        </div>
      </div>

      <form onSubmit={handleSubmit}>
        <Card>
          <div className="grid grid-cols-2 gap-6">
            <div className="input-group">
              <label className="input-label">{t("machineForm.name")} *</label>
              <input className="input-field" value={form.name} onChange={(e) => set("name", e.target.value)} required aria-label={t("machineForm.name")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("machineForm.code")}</label>
              <input className="input-field" value={form.code} onChange={(e) => set("code", e.target.value)} aria-label={t("machineForm.code")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("machineForm.type")}</label>
              <select className="input-field" value={form.mtype} onChange={(e) => set("mtype", e.target.value)} aria-label={t("machineForm.type")}>
                <option value="single_die">{t("machineForm.mtype.singleDie")}</option>
                <option value="multi_die">{t("machineForm.mtype.multiDie")}</option>
                <option value="punch">{t("machineForm.mtype.punch")}</option>
                <option value="printer">{t("machineForm.mtype.printer")}</option>
                <option value="gluer">{t("machineForm.mtype.gluer")}</option>
                <option value="other">{t("machineForm.mtype.other")}</option>
              </select>
            </div>
            <div className="input-group">
              <label className="input-label">{t("machineForm.supportedProducts")}</label>
              <input className="input-field" value={form.supported_products} onChange={(e) => set("supported_products", e.target.value)} aria-label={t("machineForm.supportedProducts")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("machineForm.purchaseDate")}</label>
              <input className="input-field" type="date" value={form.purchase_date} onChange={(e) => set("purchase_date", e.target.value)} aria-label={t("machineForm.purchaseDate")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("machineForm.supplier")}</label>
              <input className="input-field" value={form.supplier} onChange={(e) => set("supplier", e.target.value)} aria-label={t("machineForm.supplier")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("machineForm.cost")}</label>
              <input className="input-field" type="number" value={form.cost_milli} onChange={(e) => set("cost_milli", Number(e.target.value))} aria-label={t("machineForm.cost")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("machineForm.capacityCpm")}</label>
              <input className="input-field" type="number" value={form.capacity_cpm} onChange={(e) => set("capacity_cpm", Number(e.target.value))} aria-label={t("machineForm.capacityCpm")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("common.status")}</label>
              <select className="input-field" value={form.status} onChange={(e) => set("status", e.target.value)} aria-label={t("common.status")}>
                <option value="active">{t("common.active")}</option>
                <option value="inactive">{t("common.inactive")}</option>
                <option value="maintenance">{t("machineForm.status.maintenance")}</option>
              </select>
            </div>
            <div className="input-group col-span-2">
              <label className="input-label">{t("common.notes")}</label>
              <textarea className="input-field" rows={3} value={form.notes} onChange={(e) => set("notes", e.target.value)} aria-label={t("common.notes")} />
            </div>
          </div>
        </Card>
        <div className="flex justify-start gap-3 mt-4">
          <Button type="submit" loading={saving} icon={<Save className="w-4 h-4" />}>
            {isEdit ? t("machineForm.saveChanges") : t("machineForm.addMachine")}
          </Button>
          <Button variant="outline" type="button" onClick={() => navigate("/machines")}>
            {t("common.cancel")}
          </Button>
        </div>
      </form>
    </div>
  );
}
