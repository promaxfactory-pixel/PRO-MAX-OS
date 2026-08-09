import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import FieldError from "@/components/ui/FieldError";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, Save } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { validate, required, nonNegative, hasErrors, clearError } from "@/lib/validation";

export default function QualityFormPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const addNotification = useUIStore((s) => s.addNotification);
  const [saving, setSaving] = useState(false);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [form, setForm] = useState({
    date: "",
    inspector: "",
    production_line_id: 0,
    result: "pass",
    defect_type: "",
    defect_qty: 0,
    notes: "",
    status: "Pending",
  });

  const set = (key: string, val: any) => { setForm((f) => ({ ...f, [key]: val })); setErrors((prev) => clearError(prev, key)); };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const errs = validate(
      { date: form.date, inspector: form.inspector, defect_qty: form.defect_qty },
      {
        date: [required(t("qualityForm.errors.dateRequired"))],
        inspector: [required(t("qualityForm.errors.inspectorRequired"))],
        defect_qty: [nonNegative(t("qualityForm.errors.defectQtyNonNegative"))],
      },
    );
    if (hasErrors(errs)) {
      setErrors(errs);
      addNotification({ id: crypto.randomUUID(), type: "warning", title: t("common.warning"), message: t("qualityForm.notifications.completeRequiredData") });
      return;
    }
    setSaving(true);
    try {
      await invoke("create_quality_inspection", { input: { ...form, production_line_id: form.production_line_id || null } });
      addNotification({ id: crypto.randomUUID(), type: "success", title: t("common.success"), message: t("qualityForm.notifications.saved") });
      navigate("/quality");
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("qualityForm.notifications.saveFailed", { error: String(err) }) });
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div className="flex items-center gap-3">
          <button onClick={() => navigate("/quality")} className="btn-ghost p-2">
            <ArrowRight className="w-5 h-5" />
          </button>
          <div>
            <h1 className="page-title">{t("qualityForm.title")}</h1>
            <p className="page-subtitle">{t("qualityForm.subtitle")}</p>
          </div>
        </div>
      </div>

      <form onSubmit={handleSubmit}>
        <Card>
          <div className="grid grid-cols-2 gap-6">
            <div className="input-group">
              <label className="input-label">{t("common.date")} *</label>
              <input className="input-field" type="date" value={form.date} onChange={(e) => set("date", e.target.value)} required aria-label={t("common.date")} />
              <FieldError message={errors.date} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("qualityForm.inspector")} *</label>
              <input className="input-field" value={form.inspector} onChange={(e) => set("inspector", e.target.value)} required aria-label={t("qualityForm.inspector")} />
              <FieldError message={errors.inspector} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("qualityForm.productionLineId")}</label>
              <input className="input-field" type="number" min="0" value={form.production_line_id} onChange={(e) => set("production_line_id", Number(e.target.value))} aria-label={t("qualityForm.productionLineId")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("qualityForm.result")}</label>
              <select className="input-field" value={form.result} onChange={(e) => set("result", e.target.value)} aria-label={t("qualityForm.result")}>
                <option value="pass">{t("qualityForm.resultOptions.pass")}</option>
                <option value="fail">{t("qualityForm.resultOptions.fail")}</option>
                <option value="rework">{t("qualityForm.resultOptions.rework")}</option>
              </select>
            </div>
            <div className="input-group">
              <label className="input-label">{t("qualityForm.defectType")}</label>
              <input className="input-field" value={form.defect_type} onChange={(e) => set("defect_type", e.target.value)} aria-label={t("qualityForm.defectType")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("qualityForm.defectQty")}</label>
              <input className="input-field" type="number" min="0" value={form.defect_qty} onChange={(e) => set("defect_qty", Number(e.target.value))} aria-label={t("qualityForm.defectQty")} />
              <FieldError message={errors.defect_qty} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("common.status")}</label>
              <select className="input-field" value={form.status} onChange={(e) => set("status", e.target.value)} aria-label={t("common.status")}>
                <option value="Pending">{t("badge.pending")}</option>
                <option value="In Progress">{t("qualityForm.statusOptions.inProgress")}</option>
                <option value="Completed">{t("badge.completed")}</option>
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
            {t("qualityForm.save")}
          </Button>
          <Button variant="outline" type="button" onClick={() => navigate("/quality")}>
            {t("common.cancel")}
          </Button>
        </div>
      </form>
    </div>
  );
}
