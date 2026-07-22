import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, Save } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import type { Employee } from "@/types";

export default function EmployeeFormPage() {
  const { id } = useParams();
  const navigate = useNavigate();
  const isEdit = Boolean(id);
  const addNotification = useUIStore((s) => s.addNotification);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [form, setForm] = useState({
    code: "", name: "", nationality: "", job: "", salary_milli: 0,
    allowances_milli: 0, phone: "", passport_no: "", passport_expiry: "",
    residence_expiry: "", visa_expiry: "", workpermit_expiry: "",
    insurance_expiry: "", contract_end: "", joining_date: "", notes: "",
  });

  useEffect(() => {
    if (isEdit) {
      setLoading(true);
      invoke<Employee>("get_employee", { id: Number(id) }).then((d) => {
        setForm({
          code: d.code || "", name: d.name || "", nationality: d.nationality || "",
          job: d.job || "", salary_milli: d.salary_milli || 0,
          allowances_milli: d.allowances_milli || 0, phone: d.phone || "",
          passport_no: d.passport_no || "", passport_expiry: d.passport_expiry || "",
          residence_expiry: d.residence_expiry || "", visa_expiry: d.visa_expiry || "",
          workpermit_expiry: d.workpermit_expiry || "", insurance_expiry: d.insurance_expiry || "",
          contract_end: d.contract_end || "", joining_date: d.joining_date || "", notes: d.notes || "",
        });
      }).catch((e: any) => addNotification({ title: 'خطأ', message: String(e), type: 'error' })).finally(() => setLoading(false));
    }
  }, [id, isEdit]);

  const set = (key: string, val: string | number | boolean) => setForm((f) => ({ ...f, [key]: val }));

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!form.name.trim()) return addNotification({ id: crypto.randomUUID(), type: "warning", title: "تنبيه", message: "اسم الموظف مطلوب" });
    setSaving(true);
    try {
      if (isEdit) {
        await invoke("update_employee", { id: Number(id), input: form });
        addNotification({ id: crypto.randomUUID(), type: "success", title: "تم بنجاح", message: "تم حفظ بيانات الموظف بنجاح" });
        navigate(`/hr/employees/${id}`);
      } else {
        const newId = await invoke("create_employee", { input: form });
        addNotification({ id: crypto.randomUUID(), type: "success", title: "تم بنجاح", message: "تم حفظ بيانات الموظف بنجاح" });
        navigate(`/hr/employees/${newId}`);
      }
    } catch (err: unknown) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "فشل في حفظ بيانات الموظف" }); }
    finally { setSaving(false); }
  };

  if (loading) return <div className="flex items-center justify-center h-64"><div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" /></div>;

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div className="flex items-center gap-3">
          <button onClick={() => navigate('/hr/employees')} className="btn-ghost p-2"><ArrowRight className="w-5 h-5" /></button>
          <div>
            <h1 className="page-title">{isEdit ? "تعديل بيانات الموظف" : "إضافة موظف جديد"}</h1>
            <p className="page-subtitle">{isEdit ? `تعديل ${form.name}` : "إضافة موظف جديد"}</p>
          </div>
        </div>
      </div>

      <form onSubmit={handleSubmit}>
        <Card>
          <h3 className="section-title mb-4">البيانات الأساسية</h3>
          <div className="grid grid-cols-3 gap-6">
            <div className="input-group">
              <label className="input-label">اسم الموظف *</label>
              <input className="input-field" value={form.name} onChange={(e) => set("name", e.target.value)} required />
            </div>
            <div className="input-group">
              <label className="input-label">الرقم الوظيفي</label>
              <input className="input-field" value={form.code} onChange={(e) => set("code", e.target.value)} />
            </div>
            <div className="input-group">
              <label className="input-label">الجنسية</label>
              <input className="input-field" value={form.nationality} onChange={(e) => set("nationality", e.target.value)} />
            </div>
            <div className="input-group">
              <label className="input-label">الوظيفة</label>
              <input className="input-field" value={form.job} onChange={(e) => set("job", e.target.value)} />
            </div>
            <div className="input-group">
              <label className="input-label">الهاتف</label>
              <input className="input-field" value={form.phone} onChange={(e) => set("phone", e.target.value)} />
            </div>
            <div className="input-group">
              <label className="input-label">تاريخ الالتحاق</label>
              <input className="input-field" type="date" value={form.joining_date} onChange={(e) => set("joining_date", e.target.value)} />
            </div>
          </div>
        </Card>

        <Card className="mt-4">
          <h3 className="section-title mb-4">الراتب والمزايا</h3>
          <div className="grid grid-cols-2 gap-6">
            <div className="input-group">
              <label className="input-label">الراتب الأساسي (مليار)</label>
              <input className="input-field" type="number" value={form.salary_milli} onChange={(e) => set("salary_milli", Number(e.target.value))} />
            </div>
            <div className="input-group">
              <label className="input-label">البدلات (مليار)</label>
              <input className="input-field" type="number" value={form.allowances_milli} onChange={(e) => set("allowances_milli", Number(e.target.value))} />
            </div>
          </div>
        </Card>

        <Card className="mt-4">
          <h3 className="section-title mb-4">الوثائق والتواريخ</h3>
          <div className="grid grid-cols-3 gap-6">
            <div className="input-group">
              <label className="input-label">رقم الجواز</label>
              <input className="input-field" value={form.passport_no} onChange={(e) => set("passport_no", e.target.value)} />
            </div>
            <div className="input-group">
              <label className="input-label">انتهاء الجواز</label>
              <input className="input-field" type="date" value={form.passport_expiry} onChange={(e) => set("passport_expiry", e.target.value)} />
            </div>
            <div className="input-group">
              <label className="input-label">انتهاء الإقامة</label>
              <input className="input-field" type="date" value={form.residence_expiry} onChange={(e) => set("residence_expiry", e.target.value)} />
            </div>
            <div className="input-group">
              <label className="input-label">انتهاء التأشيرة</label>
              <input className="input-field" type="date" value={form.visa_expiry} onChange={(e) => set("visa_expiry", e.target.value)} />
            </div>
            <div className="input-group">
              <label className="input-label">انتهاء تصريح العمل</label>
              <input className="input-field" type="date" value={form.workpermit_expiry} onChange={(e) => set("workpermit_expiry", e.target.value)} />
            </div>
            <div className="input-group">
              <label className="input-label">انتهاء التأمين</label>
              <input className="input-field" type="date" value={form.insurance_expiry} onChange={(e) => set("insurance_expiry", e.target.value)} />
            </div>
            <div className="input-group">
              <label className="input-label">انتهاء العقد</label>
              <input className="input-field" type="date" value={form.contract_end} onChange={(e) => set("contract_end", e.target.value)} />
            </div>
          </div>
        </Card>

        <div className="input-group mt-4">
          <label className="input-label">ملاحظات</label>
          <textarea className="input-field" rows={2} value={form.notes} onChange={(e) => set("notes", e.target.value)} />
        </div>

        <div className="flex justify-start gap-3 mt-4">
          <Button type="submit" loading={saving} icon={<Save className="w-4 h-4" />}>{isEdit ? "حفظ التعديلات" : "إضافة الموظف"}</Button>
          <Button variant="outline" type="button" onClick={() => navigate('/hr/employees')}>إلغاء</Button>
        </div>
      </form>
    </div>
  );
}
