import { useState } from "react";
import { useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, Save, Plus, Trash2, Upload } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

interface PartUsed {
  part_name: string;
  quantity: number;
  unit_cost_milli: number;
}

export default function MaintenanceSheetCreatePage() {
  const navigate = useNavigate();
  const addNotification = useUIStore((s) => s.addNotification);
  const [saving, setSaving] = useState(false);

  const [form, setForm] = useState({
    date: new Date().toISOString().split("T")[0],
    equipment_id: 0,
    equipment_name: "",
    fault_description: "",
    severity: "medium",
    downtime_hours: 0,
    root_cause: "",
    corrective_action: "",
    preventive_action: "",
    assigned_to: "",
    notes: "",
  });

  const [parts, setParts] = useState<PartUsed[]>([]);

  const handleChange = (field: string, value: any) => {
    setForm((prev) => ({ ...prev, [field]: value }));
  };

  const addPart = () => {
    setParts((prev) => [...prev, { part_name: "", quantity: 1, unit_cost_milli: 0 }]);
  };

  const updatePart = (index: number, field: keyof PartUsed, value: any) => {
    setParts((prev) => prev.map((p, i) => (i === index ? { ...p, [field]: value } : p)));
  };

  const removePart = (index: number) => {
    setParts((prev) => prev.filter((_, i) => i !== index));
  };

  const totalPartsCost = parts.reduce((s, p) => s + p.quantity * p.unit_cost_milli, 0);

  const handleSubmit = async () => {
    setSaving(true);
    try {
      await invoke("create_maintenance_sheet", {
        input: {
          date: form.date,
          equipment_name: form.equipment_name,
          fault_description: form.fault_description,
          severity: form.severity,
          downtime_hours: form.downtime_hours,
          root_cause: form.root_cause,
          corrective_action: form.corrective_action,
          preventive_action: form.preventive_action,
          assigned_to: form.assigned_to,
          notes: form.notes,
          parts,
        },
      });
      navigate("/maintenance");
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء الحفظ" });
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
            <h1 className="page-title">تذكرة صيانة جديدة</h1>
            <p className="page-subtitle">تسجيل عطل أو صيانة دورية</p>
          </div>
        </div>
        <Button icon={<Save className="w-4 h-4" />} onClick={handleSubmit} loading={saving}>
          حفظ
        </Button>
      </div>

      <Card>
        <h3 className="section-title mb-4">بيانات العطل</h3>
        <div className="grid grid-cols-3 gap-4">
          <div>
            <label className="form-label">التاريخ</label>
            <input type="date" value={form.date} onChange={(e) => handleChange("date", e.target.value)} className="input-field" />
          </div>
          <div>
            <label className="form-label">المعدة</label>
            <input
              type="text"
              value={form.equipment_name}
              onChange={(e) => handleChange("equipment_name", e.target.value)}
              className="input-field"
              placeholder="اسم المعدة أو Máy"
            />
          </div>
          <div>
            <label className="form-label">الخطورة</label>
            <select value={form.severity} onChange={(e) => handleChange("severity", e.target.value)} className="input-field">
              <option value="critical">حرج</option>
              <option value="high">مرتفع</option>
              <option value="medium">متوسط</option>
              <option value="low">منخفض</option>
            </select>
          </div>
        </div>
        <div className="mt-4">
          <label className="form-label">وصف العطل</label>
          <textarea
            value={form.fault_description}
            onChange={(e) => handleChange("fault_description", e.target.value)}
            className="input-field min-h-[80px]"
            placeholder="اشرح العطل بالتفصيل..."
          />
        </div>
      </Card>

      <Card>
        <h3 className="section-title mb-4">التحليل والإصلاح</h3>
        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="form-label">السبب الجذري</label>
            <textarea
              value={form.root_cause}
              onChange={(e) => handleChange("root_cause", e.target.value)}
              className="input-field min-h-[60px]"
              placeholder="ما هو السبب الجذري للعطل؟"
            />
          </div>
          <div>
            <label className="form-label">الإجراء التصحيحي</label>
            <textarea
              value={form.corrective_action}
              onChange={(e) => handleChange("corrective_action", e.target.value)}
              className="input-field min-h-[60px]"
              placeholder="الإجراءات التي تم اتخاذها..."
            />
          </div>
        </div>
        <div className="grid grid-cols-2 gap-4 mt-4">
          <div>
            <label className="form-label">الإجراء الوقائي</label>
            <textarea
              value={form.preventive_action}
              onChange={(e) => handleChange("preventive_action", e.target.value)}
              className="input-field min-h-[60px]"
              placeholder="إجراءات لمنع تكرار العطل..."
            />
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="form-label">وقت التوقف (ساعات)</label>
              <input type="number" value={form.downtime_hours} onChange={(e) => handleChange("downtime_hours", Number(e.target.value))} className="input-field" min="0" step="0.5" />
            </div>
            <div>
              <label className="form-label">المسؤول عن التنفيذ</label>
              <input type="text" value={form.assigned_to} onChange={(e) => handleChange("assigned_to", e.target.value)} className="input-field" placeholder="اسم الفني" />
            </div>
          </div>
        </div>
      </Card>

      <Card>
        <div className="flex items-center justify-between mb-4">
          <h3 className="section-title">قطع الغيار المستخدمة</h3>
          <Button variant="ghost" size="sm" icon={<Plus className="w-4 h-4" />} onClick={addPart}>
            إضافة قطعة
          </Button>
        </div>

        {parts.length > 0 && (
          <div className="space-y-3">
            {parts.map((part, idx) => (
              <div key={idx} className="flex items-center gap-3 p-3 bg-surface-800/50 rounded-xl border border-surface-700/30">
                <input
                  type="text"
                  value={part.part_name}
                  onChange={(e) => updatePart(idx, "part_name", e.target.value)}
                  className="input-field flex-1 text-sm"
                  placeholder="اسم القطعة"
                />
                <input
                  type="number"
                  value={part.quantity}
                  onChange={(e) => updatePart(idx, "quantity", Number(e.target.value))}
                  className="input-field w-20 text-sm"
                  min="1"
                />
                <input
                  type="number"
                  value={part.unit_cost_milli}
                  onChange={(e) => updatePart(idx, "unit_cost_milli", Number(e.target.value))}
                  className="input-field w-32 text-sm"
                  placeholder="التكلفة"
                  min="0"
                />
                <button onClick={() => removePart(idx)} className="text-red-400 hover:text-red-300 p-1">
                  <Trash2 className="w-4 h-4" />
                </button>
              </div>
            ))}
          </div>
        )}

        {parts.length > 0 && (
          <div className="flex justify-between mt-4 pt-3 border-t border-surface-700/30">
            <span className="text-surface-400 text-sm">إجمالي تكلفة القطع</span>
            <span className="font-bold text-brand-400">{totalPartsCost.toLocaleString()} م.ل</span>
          </div>
        )}
      </Card>

      <Card>
        <label className="form-label">ملاحظات إضافية</label>
        <textarea
          value={form.notes}
          onChange={(e) => handleChange("notes", e.target.value)}
          className="input-field min-h-[60px]"
          placeholder="أي ملاحظات أخرى..."
        />
      </Card>
    </div>
  );
}
