import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, Save, Plus, Trash2 } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { useTranslation } from "react-i18next";

interface WorkerEntry {
  worker_id: number;
  worker_name: string;
  attendance: "present" | "absent" | "late";
  overtime_hours: number;
  production_qty: number;
  quality_grade: "A" | "B" | "C" | "D";
  safety_incident: boolean;
  safety_note: string;
}

export default function OperationsSheetCreatePage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const addNotification = useUIStore((s) => s.addNotification);
  const [saving, setSaving] = useState(false);

  const [form, setForm] = useState({
    date: new Date().toISOString().split("T")[0],
    shift: "morning",
    notes: "",
  });

  const [entries, setEntries] = useState<WorkerEntry[]>([]);

  useEffect(() => {
    setEntries([
      { worker_id: 0, worker_name: "", attendance: "present", overtime_hours: 0, production_qty: 0, quality_grade: "A", safety_incident: false, safety_note: "" },
    ]);
  }, []);

  const updateEntry = (index: number, field: keyof WorkerEntry, value: any) => {
    setEntries((prev) => prev.map((e, i) => (i === index ? { ...e, [field]: value } : e)));
  };

  const addWorkerEntry = () => {
    setEntries((prev) => [
      ...prev,
      { worker_id: 0, worker_name: "", attendance: "present", overtime_hours: 0, production_qty: 0, quality_grade: "A", safety_incident: false, safety_note: "" },
    ]);
  };

  const removeEntry = (index: number) => {
    setEntries((prev) => prev.filter((_, i) => i !== index));
  };

  const handleSubmit = async () => {
    setSaving(true);
    try {
      await invoke("create_operations_sheet", {
        date: form.date,
        shift: form.shift,
        notes: form.notes,
        entries,
      });
      navigate("/operations");
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("operations.saveError") });
    } finally {
      setSaving(false);
    }
  };

  const totalProduction = entries.reduce((s, e) => s + (e.production_qty || 0), 0);
  const totalOvertime = entries.reduce((s, e) => s + (e.overtime_hours || 0), 0);
  const presentCount = entries.filter((e) => e.attendance === "present").length;
  const safetyCount = entries.filter((e) => e.safety_incident).length;

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div className="flex items-center gap-3">
          <button onClick={() => navigate("/operations")} className="btn-ghost p-2">
            <ArrowRight className="w-5 h-5" />
          </button>
          <div>
            <h1 className="page-title">{t("operations.createTitle")}</h1>
            <p className="page-subtitle">{t("operations.createSubtitle")}</p>
          </div>
        </div>
        <Button icon={<Save className="w-4 h-4" />} onClick={handleSubmit} loading={saving}>
          {t("common.save")}
        </Button>
      </div>

      <div className="grid grid-cols-4 gap-4">
        <Card className="text-center">
          <p className="text-2xl font-bold gradient-text">{entries.length}</p>
          <p className="text-xs text-surface-400">{t("operations.totalWorkers")}</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-emerald-400">{presentCount}</p>
          <p className="text-xs text-surface-400">{t("operations.present")}</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-blue-400">{totalProduction}</p>
          <p className="text-xs text-surface-400">{t("operations.productionTons")}</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-amber-400">{t("maintenance.hoursShort", { hours: totalOvertime })}</p>
          <p className="text-xs text-surface-400">{t("operations.overtime")}</p>
        </Card>
      </div>

      <Card>
        <h3 className="section-title mb-4">{t("operations.shiftData")}</h3>
        <div className="grid grid-cols-3 gap-4">
          <div>
            <label className="form-label">{t("common.date")}</label>
            <input type="date" value={form.date} onChange={(e) => setForm({ ...form, date: e.target.value })} className="input-field" aria-label={t("common.date")} />
          </div>
          <div>
            <label className="form-label">{t("production.shift")}</label>
            <select value={form.shift} onChange={(e) => setForm({ ...form, shift: e.target.value })} className="input-field" aria-label={t("production.shift")}>
              <option value="morning">{t("operations.shift.morning")}</option>
              <option value="evening">{t("operations.shift.evening")}</option>
              <option value="night">{t("operations.shift.night")}</option>
            </select>
          </div>
          <div>
            <label className="form-label">{t("common.notes")}</label>
            <input type="text" value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} className="input-field" placeholder={t("operations.notesPlaceholder")} aria-label={t("common.notes")} />
          </div>
        </div>
      </Card>

      <Card>
        <div className="flex items-center justify-between mb-4">
          <h3 className="section-title">{t("operations.workersData")}</h3>
          <Button variant="ghost" size="sm" icon={<Plus className="w-4 h-4" />} onClick={addWorkerEntry}>
            {t("operations.addWorker")}
          </Button>
        </div>

        <div className="space-y-4">
          {entries.map((entry, idx) => (
            <div key={idx} className="p-4 bg-surface-800/50 rounded-xl border border-surface-700/30 space-y-3">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <span className="text-sm font-bold text-brand-400">#{idx + 1}</span>
                  <input
                    type="text"
                    value={entry.worker_name}
                    onChange={(e) => {
                      updateEntry(idx, "worker_name", e.target.value);
                    }}
                    className="input-field w-48"
                    placeholder={t("operations.workerName")}
                    aria-label={t("operations.workerName")}
                  />
                </div>
                <button onClick={() => removeEntry(idx)} className="text-red-400 hover:text-red-300 p-1">
                  <Trash2 className="w-4 h-4" />
                </button>
              </div>

              <div className="grid grid-cols-6 gap-3">
                <div>
                  <label className="form-label text-xs">{t("operations.attendance")}</label>
                  <select value={entry.attendance} onChange={(e) => updateEntry(idx, "attendance", e.target.value)} className="input-field text-sm" aria-label={t("operations.attendance")}>
                    <option value="present">{t("operations.attendanceStatus.present")}</option>
                    <option value="absent">{t("operations.attendanceStatus.absent")}</option>
                    <option value="late">{t("operations.attendanceStatus.late")}</option>
                  </select>
                </div>
                <div>
                  <label className="form-label text-xs">{t("operations.overtimeHours")}</label>
                  <input type="number" value={entry.overtime_hours} onChange={(e) => updateEntry(idx, "overtime_hours", Number(e.target.value))} className="input-field text-sm" min="0" aria-label={t("operations.overtimeHours")} />
                </div>
                <div>
                  <label className="form-label text-xs">{t("operations.productionTons")}</label>
                  <input type="number" value={entry.production_qty} onChange={(e) => updateEntry(idx, "production_qty", Number(e.target.value))} className="input-field text-sm" min="0" aria-label={t("operations.productionTonsAria")} />
                </div>
                <div>
                  <label className="form-label text-xs">{t("operations.qualityGrade")}</label>
                  <select value={entry.quality_grade} onChange={(e) => updateEntry(idx, "quality_grade", e.target.value)} className="input-field text-sm" aria-label={t("operations.qualityGrade")}>
                    <option value="A">{t("operations.qualityGradeOptions.excellent")}</option>
                    <option value="B">{t("operations.qualityGradeOptions.good")}</option>
                    <option value="C">{t("operations.qualityGradeOptions.fair")}</option>
                    <option value="D">{t("operations.qualityGradeOptions.poor")}</option>
                  </select>
                </div>
                <div className="flex flex-col items-center justify-center">
                  <label className="form-label text-xs">{t("operations.safetyIncident")}</label>
                  <input
                    type="checkbox"
                    checked={entry.safety_incident}
                    onChange={(e) => updateEntry(idx, "safety_incident", e.target.checked)}
                    className="w-5 h-5 rounded bg-surface-700 border-surface-600 text-red-500 focus:ring-red-500"
                    aria-label={t("operations.safetyIncident")}
                  />
                </div>
                <div>
                  <label className="form-label text-xs">{t("operations.safetyNote")}</label>
                  <input
                    type="text"
                    value={entry.safety_note}
                    onChange={(e) => updateEntry(idx, "safety_note", e.target.value)}
                    className="input-field text-sm"
                    placeholder={t("operations.detailsPlaceholder")}
                    disabled={!entry.safety_incident}
                    aria-label={t("operations.safetyNote")}
                  />
                </div>
              </div>
            </div>
          ))}
        </div>
      </Card>

      {safetyCount > 0 && (
        <div className="p-4 bg-red-500/10 border border-red-500/30 rounded-xl text-red-400 text-sm text-center">
          {t("operations.safetyCountWarning", { count: safetyCount })}
        </div>
      )}
    </div>
  );
}
