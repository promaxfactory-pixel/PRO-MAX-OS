import { useState } from "react";
import { useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, Save } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

export default function QualityFormPage() {
  const navigate = useNavigate();
  const addNotification = useUIStore((s) => s.addNotification);
  const [saving, setSaving] = useState(false);
  const [form, setForm] = useState({
    date: "",
    inspector: "",
    production_line_id: "",
    result: "pass",
    defect_type: "",
    defect_qty: 0,
    notes: "",
    status: "Pending",
  });

  const set = (key: string, val: any) => setForm((f) => ({ ...f, [key]: val }));

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!form.inspector.trim()) return addNotification({ id: crypto.randomUUID(), type: "warning", title: "تنبيه", message: "المفتتش مطلوب" });
    setSaving(true);
    try {
      await invoke("create_quality_inspection", { input: form });
      addNotification({ id: crypto.randomUUID(), type: "success", title: "تم بنجاح", message: "تم تسجيل الفحص بنجاح" });
      navigate("/quality");
    } catch (err: any) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "فشل في تسجيل الفحص" });
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
            <h1 className="page-title">فحص جودة جديد</h1>
            <p className="page-subtitle">إضافة تقرير فحص جودة</p>
          </div>
        </div>
      </div>

      <form onSubmit={handleSubmit}>
        <Card>
          <div className="grid grid-cols-2 gap-6">
            <div className="input-group">
              <label className="input-label">التاريخ *</label>
              <input className="input-field" type="date" value={form.date} onChange={(e) => set("date", e.target.value)} required />
            </div>
            <div className="input-group">
              <label className="input-label">المفتتش *</label>
              <input className="input-field" value={form.inspector} onChange={(e) => set("inspector", e.target.value)} required />
            </div>
            <div className="input-group">
              <label className="input-label">رقم خط الإنتاج</label>
              <input className="input-field" value={form.production_line_id} onChange={(e) => set("production_line_id", e.target.value)} />
            </div>
            <div className="input-group">
              <label className="input-label">النتيجة</label>
              <select className="input-field" value={form.result} onChange={(e) => set("result", e.target.value)}>
                <option value="pass">ناجح</option>
                <option value="fail">غير ناجح</option>
                <option value="rework">إعادة العمل</option>
              </select>
            </div>
            <div className="input-group">
              <label className="input-label">نوع العيب</label>
              <input className="input-field" value={form.defect_type} onChange={(e) => set("defect_type", e.target.value)} />
            </div>
            <div className="input-group">
              <label className="input-label">كمية العيب</label>
              <input className="input-field" type="number" min="0" value={form.defect_qty} onChange={(e) => set("defect_qty", Number(e.target.value))} />
            </div>
            <div className="input-group">
              <label className="input-label">الحالة</label>
              <select className="input-field" value={form.status} onChange={(e) => set("status", e.target.value)}>
                <option value="Pending">قيد الانتظار</option>
                <option value="In Progress">قيد التنفيذ</option>
                <option value="Completed">مكتمل</option>
              </select>
            </div>
            <div className="input-group col-span-2">
              <label className="input-label">ملاحظات</label>
              <textarea className="input-field" rows={3} value={form.notes} onChange={(e) => set("notes", e.target.value)} />
            </div>
          </div>
        </Card>
        <div className="flex justify-start gap-3 mt-4">
          <Button type="submit" loading={saving} icon={<Save className="w-4 h-4" />}>
            حفظ الفحص
          </Button>
          <Button variant="outline" type="button" onClick={() => navigate("/quality")}>
            إلغاء
          </Button>
        </div>
      </form>
    </div>
  );
}
