import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import FieldError from "@/components/ui/FieldError";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, Save } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { validate, required, nonNegative, hasErrors, clearError } from "@/lib/validation";
import type { Machine } from "@/types";

export default function MaintenanceSheetCreatePage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const addNotification = useUIStore((s) => s.addNotification);
  const [saving, setSaving] = useState(false);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [machines, setMachines] = useState<Machine[]>([]);

  const [form, setForm] = useState({
    date: new Date().toISOString().split("T")[0],
    shift: "morning",
    maintenance_supervisor: "",
    machine_id: 0,
    area: "",
    fault_title: "",
    fault_description: "",
    severity: "medium",
    machine_stopped: false,
    downtime_start: "",
    downtime_end: "",
    downtime_minutes: 0,
    repair_action: "",
    parts_changed: "",
    spare_parts_cost_milli: 0,
    labor_cost_milli: 0,
    other_cost_milli: 0,
    root_cause: "",
    preventive_action: "",
    next_followup_date: "",
    notes: "",
  });

  useEffect(() => {
    invoke("list_machines").then((d) => setMachines(d as Machine[])).catch((e: unknown) => addNotification({ title: t('common.error'), message: String(e), type: 'error' }));
  }, [t]);

  const handleChange = (field: string, value: any) => {
    setForm((prev) => ({ ...prev, [field]: value }));
    setErrors((prev) => clearError(prev, field));
  };

  const totalCost = form.spare_parts_cost_milli + form.labor_cost_milli + form.other_cost_milli;

  const handleSubmit = async () => {
    const errs = validate(
      {
        fault_title: form.fault_title,
        area: form.area,
        maintenance_supervisor: form.maintenance_supervisor,
        downtime_minutes: form.downtime_minutes,
        spare_parts_cost_milli: form.spare_parts_cost_milli,
        labor_cost_milli: form.labor_cost_milli,
        other_cost_milli: form.other_cost_milli,
      },
      {
        fault_title: [required(t("maintenance.sheetCreate.faultTitleRequired"))],
        area: [required(t("maintenance.sheetCreate.areaRequired"))],
        maintenance_supervisor: [required(t("maintenance.sheetCreate.supervisorRequired"))],
        downtime_minutes: [nonNegative(t("maintenance.sheetCreate.downtimeNonNegative"))],
        spare_parts_cost_milli: [nonNegative()],
        labor_cost_milli: [nonNegative()],
        other_cost_milli: [nonNegative()],
      },
    );
    if (hasErrors(errs)) {
      setErrors(errs);
      addNotification({ id: crypto.randomUUID(), type: "warning", title: t("maintenance.notice"), message: t("maintenance.sheetCreate.completeRequiredData") });
      return;
    }
    setSaving(true);
    try {
      await invoke("create_maintenance_sheet", {
        input: {
          date: form.date,
          shift: form.shift,
          maintenance_supervisor: form.maintenance_supervisor || null,
          machine_id: form.machine_id || null,
          area: form.area || null,
          fault_title: form.fault_title || null,
          fault_description: form.fault_description || null,
          severity: form.severity,
          notes: form.notes || null,
          machine_stopped: form.machine_stopped ? 1 : 0,
          downtime_start: form.downtime_start || null,
          downtime_end: form.downtime_end || null,
          downtime_minutes: form.downtime_minutes || 0,
          repair_action: form.repair_action || null,
          parts_changed: form.parts_changed || null,
          spare_parts_cost_milli: form.spare_parts_cost_milli || 0,
          labor_cost_milli: form.labor_cost_milli || 0,
          other_cost_milli: form.other_cost_milli || 0,
          root_cause: form.root_cause || null,
          preventive_action: form.preventive_action || null,
          next_followup_date: form.next_followup_date || null,
        },
      });
      navigate("/maintenance");
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("maintenance.sheetCreate.saveError", { error: String(err) }) });
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div className="flex items-center gap-3">
          <button onClick={() => navigate("/maintenance")} className="btn-ghost p-2">
            <ArrowRight className="w-5 h-5" />
          </button>
          <div>
            <h1 className="page-title">{t("maintenance.sheetCreate.title")}</h1>
            <p className="page-subtitle">{t("maintenance.sheetCreate.subtitle")}</p>
          </div>
        </div>
        <Button icon={<Save className="w-4 h-4" />} onClick={handleSubmit} loading={saving}>
          {t("common.save")}
        </Button>
      </div>

      <Card>
        <h3 className="section-title mb-4">{t("maintenance.sheetCreate.faultData")}</h3>
        <div className="grid grid-cols-3 gap-4">
          <div>
            <label className="form-label">{t("common.date")}</label>
            <input type="date" value={form.date} onChange={(e) => handleChange("date", e.target.value)} className="input-field" aria-label={t("common.date")} />
          </div>
          <div>
            <label className="form-label">{t("maintenance.sheetCreate.machine")}</label>
            <select value={form.machine_id} onChange={(e) => handleChange("machine_id", Number(e.target.value))} className="input-field" aria-label={t("maintenance.sheetCreate.machine")}>
              <option value={0}>{t("maintenance.sheetCreate.noMachine")}</option>
              {machines.map((m) => (
                <option key={m.id} value={m.id}>{m.code} - {m.name}</option>
              ))}
            </select>
          </div>
          <div>
            <label className="form-label">{t("maintenance.sheetCreate.shift")}</label>
            <select value={form.shift} onChange={(e) => handleChange("shift", e.target.value)} className="input-field" aria-label={t("maintenance.sheetCreate.shift")}>
              <option value="morning">{t("maintenance.shift.morning")}</option>
              <option value="evening">{t("maintenance.shift.evening")}</option>
              <option value="night">{t("maintenance.shift.night")}</option>
            </select>
          </div>
          <div>
            <label className="form-label">{t("maintenance.sheetCreate.area")}</label>
            <input type="text" value={form.area} onChange={(e) => handleChange("area", e.target.value)} className="input-field" placeholder={t("maintenance.sheetCreate.areaPlaceholder")} aria-label={t("maintenance.sheetCreate.area")} />
            <FieldError message={errors.area} />
          </div>
          <div>
            <label className="form-label">{t("maintenance.sheetCreate.supervisor")}</label>
            <input type="text" value={form.maintenance_supervisor} onChange={(e) => handleChange("maintenance_supervisor", e.target.value)} className="input-field" placeholder={t("maintenance.sheetCreate.supervisorPlaceholder")} aria-label={t("maintenance.sheetCreate.supervisor")} />
            <FieldError message={errors.maintenance_supervisor} />
          </div>
          <div>
            <label className="form-label">{t("maintenance.sheetCreate.severity")}</label>
            <select value={form.severity} onChange={(e) => handleChange("severity", e.target.value)} className="input-field" aria-label={t("maintenance.sheetCreate.severity")}>
              <option value="critical">{t("maintenance.severity.critical")}</option>
              <option value="high">{t("maintenance.severity.high")}</option>
              <option value="medium">{t("maintenance.severity.medium")}</option>
              <option value="low">{t("maintenance.severity.low")}</option>
            </select>
          </div>
          <div className="col-span-3">
            <label className="form-label">{t("maintenance.sheetCreate.faultTitle")}</label>
            <input type="text" value={form.fault_title} onChange={(e) => handleChange("fault_title", e.target.value)} className="input-field" placeholder={t("maintenance.sheetCreate.faultTitlePlaceholder")} aria-label={t("maintenance.sheetCreate.faultTitle")} />
            <FieldError message={errors.fault_title} />
          </div>
          <div className="col-span-3">
            <label className="form-label">{t("maintenance.sheetCreate.faultDescription")}</label>
            <textarea
              value={form.fault_description}
              onChange={(e) => handleChange("fault_description", e.target.value)}
              className="input-field min-h-[80px]"
              placeholder={t("maintenance.sheetCreate.faultDescriptionPlaceholder")}
              aria-label={t("maintenance.sheetCreate.faultDescription")}
            />
          </div>
        </div>
      </Card>

      <Card>
        <h3 className="section-title mb-4">{t("maintenance.sheetCreate.downtimeRepair")}</h3>
        <label className="flex items-center gap-2 mb-4 cursor-pointer">
          <input type="checkbox" checked={form.machine_stopped} onChange={(e) => handleChange("machine_stopped", e.target.checked)} className="w-4 h-4" aria-label={t("maintenance.sheetCreate.machineStopped")} />
          <span className="text-sm">{t("maintenance.sheetCreate.machineStopped")}</span>
        </label>
        <div className="grid grid-cols-3 gap-4">
          <div>
            <label className="form-label">{t("maintenance.sheetCreate.downtimeStart")}</label>
            <input type="datetime-local" value={form.downtime_start} onChange={(e) => handleChange("downtime_start", e.target.value)} className="input-field" aria-label={t("maintenance.sheetCreate.downtimeStart")} />
          </div>
          <div>
            <label className="form-label">{t("maintenance.sheetCreate.downtimeEnd")}</label>
            <input type="datetime-local" value={form.downtime_end} onChange={(e) => handleChange("downtime_end", e.target.value)} className="input-field" aria-label={t("maintenance.sheetCreate.downtimeEnd")} />
          </div>
          <div>
            <label className="form-label">{t("maintenance.sheetCreate.downtimeMinutes")}</label>
            <input type="number" value={form.downtime_minutes} onChange={(e) => handleChange("downtime_minutes", Number(e.target.value))} className="input-field" min="0" aria-label={t("maintenance.sheetCreate.downtimeMinutes")} />
            <FieldError message={errors.downtime_minutes} />
          </div>
          <div>
            <label className="form-label">{t("maintenance.sheetCreate.rootCause")}</label>
            <textarea
              value={form.root_cause}
              onChange={(e) => handleChange("root_cause", e.target.value)}
              className="input-field min-h-[60px]"
              placeholder={t("maintenance.sheetCreate.rootCausePlaceholder")}
              aria-label={t("maintenance.sheetCreate.rootCause")}
            />
          </div>
          <div>
            <label className="form-label">{t("maintenance.sheetCreate.repairAction")}</label>
            <textarea
              value={form.repair_action}
              onChange={(e) => handleChange("repair_action", e.target.value)}
              className="input-field min-h-[60px]"
              placeholder={t("maintenance.sheetCreate.repairActionPlaceholder")}
              aria-label={t("maintenance.sheetCreate.repairAction")}
            />
          </div>
          <div>
            <label className="form-label">{t("maintenance.sheetCreate.preventiveAction")}</label>
            <textarea
              value={form.preventive_action}
              onChange={(e) => handleChange("preventive_action", e.target.value)}
              className="input-field min-h-[60px]"
              placeholder={t("maintenance.sheetCreate.preventiveActionPlaceholder")}
              aria-label={t("maintenance.sheetCreate.preventiveAction")}
            />
          </div>
          <div className="col-span-3">
            <label className="form-label">{t("maintenance.sheetCreate.replacedParts")}</label>
            <input type="text" value={form.parts_changed} onChange={(e) => handleChange("parts_changed", e.target.value)} className="input-field" placeholder={t("maintenance.sheetCreate.replacedPartsPlaceholder")} aria-label={t("maintenance.sheetCreate.replacedParts")} />
          </div>
          <div>
            <label className="form-label">{t("maintenance.sheetCreate.nextFollowUpDate")}</label>
            <input type="date" value={form.next_followup_date} onChange={(e) => handleChange("next_followup_date", e.target.value)} className="input-field" aria-label={t("maintenance.sheetCreate.nextFollowUpDate")} />
          </div>
        </div>
      </Card>

      <Card>
        <h3 className="section-title mb-4">{t("maintenance.sheetCreate.costSection")}</h3>
        <div className="grid grid-cols-3 gap-4">
          <div>
            <label className="form-label">{t("maintenance.sheetCreate.partsCostLabel")}</label>
            <input type="number" value={form.spare_parts_cost_milli} onChange={(e) => handleChange("spare_parts_cost_milli", Number(e.target.value))} className="input-field" min="0" aria-label={t("maintenance.sheetCreate.partsCostLabel")} />
            <FieldError message={errors.spare_parts_cost_milli} />
          </div>
          <div>
            <label className="form-label">{t("maintenance.sheetCreate.laborCostLabel")}</label>
            <input type="number" value={form.labor_cost_milli} onChange={(e) => handleChange("labor_cost_milli", Number(e.target.value))} className="input-field" min="0" aria-label={t("maintenance.sheetCreate.laborCostLabel")} />
            <FieldError message={errors.labor_cost_milli} />
          </div>
          <div>
            <label className="form-label">{t("maintenance.sheetCreate.otherCostLabel")}</label>
            <input type="number" value={form.other_cost_milli} onChange={(e) => handleChange("other_cost_milli", Number(e.target.value))} className="input-field" min="0" aria-label={t("maintenance.sheetCreate.otherCostLabel")} />
            <FieldError message={errors.other_cost_milli} />
          </div>
        </div>
        <div className="flex justify-between mt-4 pt-3 border-t border-surface-700/30">
          <span className="text-surface-400 text-sm">{t("maintenance.sheetCreate.totalRepairCost")}</span>
          <span className="font-bold text-brand-400">{totalCost.toLocaleString()} {t("maintenance.milliSuffix")}</span>
        </div>
      </Card>

      <Card>
        <label className="form-label">{t("maintenance.sheetCreate.notesLabel")}</label>
        <textarea
          value={form.notes}
          onChange={(e) => handleChange("notes", e.target.value)}
          className="input-field min-h-[60px]"
          placeholder={t("maintenance.sheetCreate.notesPlaceholder")}
          aria-label={t("maintenance.sheetCreate.notesLabel")}
        />
      </Card>
    </div>
  );
}
