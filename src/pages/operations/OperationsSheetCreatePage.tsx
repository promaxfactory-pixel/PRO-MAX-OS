import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, Save, Plus, Trash2 } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

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
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء الحفظ" });
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
            <h1 className="page-title">ورقة عمليات جديدة</h1>
            <p className="page-subtitle">تسجيل بيانات اليومية</p>
          </div>
        </div>
        <Button icon={<Save className="w-4 h-4" />} onClick={handleSubmit} loading={saving}>
          حفظ
        </Button>
      </div>

      <div className="grid grid-cols-4 gap-4">
        <Card className="text-center">
          <p className="text-2xl font-bold gradient-text">{entries.length}</p>
          <p className="text-xs text-surface-400">إجمالي العمال</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-emerald-400">{presentCount}</p>
          <p className="text-xs text-surface-400">حاضرين</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-blue-400">{totalProduction}</p>
          <p className="text-xs text-surface-400">الإنتاج (طن)</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-amber-400">{totalOvertime} س</p>
          <p className="text-xs text-surface-400">العمل الإضافي</p>
        </Card>
      </div>

      <Card>
        <h3 className="section-title mb-4">بيانات الوردية</h3>
        <div className="grid grid-cols-3 gap-4">
          <div>
            <label className="form-label">التاريخ</label>
            <input type="date" value={form.date} onChange={(e) => setForm({ ...form, date: e.target.value })} className="input-field" />
          </div>
          <div>
            <label className="form-label">الوردية</label>
            <select value={form.shift} onChange={(e) => setForm({ ...form, shift: e.target.value })} className="input-field">
              <option value="morning">صباحي</option>
              <option value="evening">مسائي</option>
              <option value="night">ليلي</option>
            </select>
          </div>
          <div>
            <label className="form-label">ملاحظات</label>
            <input type="text" value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} className="input-field" placeholder="ملاحظات عامة..." />
          </div>
        </div>
      </Card>

      <Card>
        <div className="flex items-center justify-between mb-4">
          <h3 className="section-title">بيانات العمال</h3>
          <Button variant="ghost" size="sm" icon={<Plus className="w-4 h-4" />} onClick={addWorkerEntry}>
            إضافة عامل
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
                    placeholder="اسم العامل"
                  />
                </div>
                <button onClick={() => removeEntry(idx)} className="text-red-400 hover:text-red-300 p-1">
                  <Trash2 className="w-4 h-4" />
                </button>
              </div>

              <div className="grid grid-cols-6 gap-3">
                <div>
                  <label className="form-label text-xs">الحضور</label>
                  <select value={entry.attendance} onChange={(e) => updateEntry(idx, "attendance", e.target.value)} className="input-field text-sm">
                    <option value="present">حاضر</option>
                    <option value="absent">غائب</option>
                    <option value="late">متأخر</option>
                  </select>
                </div>
                <div>
                  <label className="form-label text-xs">ساعات إضافية</label>
                  <input type="number" value={entry.overtime_hours} onChange={(e) => updateEntry(idx, "overtime_hours", Number(e.target.value))} className="input-field text-sm" min="0" />
                </div>
                <div>
                  <label className="form-label text-xs">الإنتاج (طن)</label>
                  <input type="number" value={entry.production_qty} onChange={(e) => updateEntry(idx, "production_qty", Number(e.target.value))} className="input-field text-sm" min="0" />
                </div>
                <div>
                  <label className="form-label text-xs">جودة الإنتاج</label>
                  <select value={entry.quality_grade} onChange={(e) => updateEntry(idx, "quality_grade", e.target.value)} className="input-field text-sm">
                    <option value="A">ممتاز (A)</option>
                    <option value="B">جيد (B)</option>
                    <option value="C">مقبول (C)</option>
                    <option value="D">غير مقبول (D)</option>
                  </select>
                </div>
                <div className="flex flex-col items-center justify-center">
                  <label className="form-label text-xs">حادث أمان</label>
                  <input
                    type="checkbox"
                    checked={entry.safety_incident}
                    onChange={(e) => updateEntry(idx, "safety_incident", e.target.checked)}
                    className="w-5 h-5 rounded bg-surface-700 border-surface-600 text-red-500 focus:ring-red-500"
                  />
                </div>
                <div>
                  <label className="form-label text-xs">ملاحظة أمان</label>
                  <input
                    type="text"
                    value={entry.safety_note}
                    onChange={(e) => updateEntry(idx, "safety_note", e.target.value)}
                    className="input-field text-sm"
                    placeholder="تفاصيل..."
                    disabled={!entry.safety_incident}
                  />
                </div>
              </div>
            </div>
          ))}
        </div>
      </Card>

      {safetyCount > 0 && (
        <div className="p-4 bg-red-500/10 border border-red-500/30 rounded-xl text-red-400 text-sm text-center">
          ⚠ يوجد {safetyCount} حادث(ات) أمان مسجل(ة) في هذه الوردية
        </div>
      )}
    </div>
  );
}
