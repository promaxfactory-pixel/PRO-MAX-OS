import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { invoke } from "@/lib/tauri";
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
    code: "", name: "", nationality: "", job: "", phone: "",
    id_number: "", date_of_birth: "", gender: "", marital_status: "", email: "",
    salary_milli: 0, basic_salary_milli: 0, housing_allowance_milli: 0,
    transport_allowance_milli: 0, food_allowance_milli: 0, other_allowances_milli: 0,
    allowances_milli: 0, overtime_rate_milli: 0,
    passport_no: "", passport_expiry: "", residence_expiry: "", visa_expiry: "",
    workpermit_expiry: "", insurance_expiry: "", insurance_policy_no: "",
    insurance_premium_milli: 0, ticket_allowance_milli: 0,
    contract_end: "", joining_date: "",
    bank_name: "", bank_account_no: "", sponsor_name: "", sponsor_id: "",
    notes: "",
  });

  useEffect(() => {
    if (isEdit) {
      setLoading(true);
      invoke<Employee>("get_employee", { id: Number(id) }).then((d) => {
        setForm({
          code: d.code || "", name: d.name || "", nationality: d.nationality || "",
          job: d.job || "", phone: d.phone || "",
          id_number: d.id_number || "", date_of_birth: d.date_of_birth || "",
          gender: d.gender || "", marital_status: d.marital_status || "", email: d.email || "",
          salary_milli: d.salary_milli || 0, basic_salary_milli: d.basic_salary_milli || 0,
          housing_allowance_milli: d.housing_allowance_milli || 0,
          transport_allowance_milli: d.transport_allowance_milli || 0,
          food_allowance_milli: d.food_allowance_milli || 0,
          other_allowances_milli: d.other_allowances_milli || 0,
          allowances_milli: d.allowances_milli || 0, overtime_rate_milli: d.overtime_rate_milli || 0,
          passport_no: d.passport_no || "", passport_expiry: d.passport_expiry || "",
          residence_expiry: d.residence_expiry || "", visa_expiry: d.visa_expiry || "",
          workpermit_expiry: d.workpermit_expiry || "", insurance_expiry: d.insurance_expiry || "",
          insurance_policy_no: d.insurance_policy_no || "",
          insurance_premium_milli: d.insurance_premium_milli || 0,
          ticket_allowance_milli: d.ticket_allowance_milli || 0,
          contract_end: d.contract_end || "", joining_date: d.joining_date || "",
          bank_name: d.bank_name || "", bank_account_no: d.bank_account_no || "",
          sponsor_name: d.sponsor_name || "", sponsor_id: d.sponsor_id || "",
          notes: d.notes || "",
        });
      }).catch((e: unknown) => addNotification({ title: "خطأ", message: String(e), type: "error" })).finally(() => setLoading(false));
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
          <button onClick={() => navigate("/hr/employees")} className="btn-ghost p-2"><ArrowRight className="w-5 h-5" /></button>
          <div>
            <h1 className="page-title">{isEdit ? "تعديل بيانات الموظف" : "إضافة موظف جديد"}</h1>
            <p className="page-subtitle">{isEdit ? `تعديل ${form.name}` : "إضافة موظف جديد"}</p>
          </div>
        </div>
      </div>

      <form onSubmit={handleSubmit}>
        <Card>
          <h3 className="section-title mb-4">البيانات الشخصية</h3>
          <div className="grid grid-cols-3 gap-6">
            <div className="input-group">
              <label className="input-label">اسم الموظف *</label>
              <input className="input-field" value={form.name} onChange={(e) => set("name", e.target.value)} required aria-label="اسم الموظف" />
            </div>
            <div className="input-group">
              <label className="input-label">الرقم الوظيفي</label>
              <input className="input-field" value={form.code} onChange={(e) => set("code", e.target.value)} aria-label="الرقم الوظيفي" />
            </div>
            <div className="input-group">
              <label className="input-label">رقم الهوية</label>
              <input className="input-field" value={form.id_number} onChange={(e) => set("id_number", e.target.value)} aria-label="رقم الهوية" />
            </div>
            <div className="input-group">
              <label className="input-label">الجنسية</label>
              <input className="input-field" value={form.nationality} onChange={(e) => set("nationality", e.target.value)} aria-label="الجنسية" />
            </div>
            <div className="input-group">
              <label className="input-label">الوظيفة</label>
              <input className="input-field" value={form.job} onChange={(e) => set("job", e.target.value)} aria-label="الوظيفة" />
            </div>
            <div className="input-group">
              <label className="input-label">الجنس</label>
              <select className="input-field" value={form.gender} onChange={(e) => set("gender", e.target.value)} aria-label="الجنس">
                <option value="">-- اختر --</option>
                <option value="ذكر">ذكر</option>
                <option value="أنثى">أنثى</option>
              </select>
            </div>
            <div className="input-group">
              <label className="input-label">الحالة الاجتماعية</label>
              <select className="input-field" value={form.marital_status} onChange={(e) => set("marital_status", e.target.value)} aria-label="الحالة الاجتماعية">
                <option value="">-- اختر --</option>
                <option value="أعزب">أعزب</option>
                <option value="متزوج">متزوج</option>
                <option value="متزوجة">متزوجة</option>
              </select>
            </div>
            <div className="input-group">
              <label className="input-label">تاريخ الميلاد</label>
              <input className="input-field" type="date" value={form.date_of_birth} onChange={(e) => set("date_of_birth", e.target.value)} aria-label="تاريخ الميلاد" />
            </div>
            <div className="input-group">
              <label className="input-label">الهاتف</label>
              <input className="input-field" value={form.phone} onChange={(e) => set("phone", e.target.value)} aria-label="الهاتف" />
            </div>
            <div className="input-group">
              <label className="input-label">البريد الإلكتروني</label>
              <input className="input-field" type="email" value={form.email} onChange={(e) => set("email", e.target.value)} aria-label="البريد الإلكتروني" />
            </div>
            <div className="input-group">
              <label className="input-label">تاريخ الالتحاق</label>
              <input className="input-field" type="date" value={form.joining_date} onChange={(e) => set("joining_date", e.target.value)} aria-label="تاريخ الالتحاق" />
            </div>
          </div>
        </Card>

        <Card className="mt-4">
          <h3 className="section-title mb-4">الراتب والمزايا</h3>
          <div className="grid grid-cols-3 gap-6">
            <div className="input-group">
              <label className="input-label">الراتب الإجمالي (مليار)</label>
              <input className="input-field" type="number" value={form.salary_milli} onChange={(e) => set("salary_milli", Number(e.target.value))} aria-label="الراتب الإجمالي" />
            </div>
            <div className="input-group">
              <label className="input-label">الراتب الأساسي (مليار)</label>
              <input className="input-field" type="number" value={form.basic_salary_milli} onChange={(e) => set("basic_salary_milli", Number(e.target.value))} aria-label="الراتب الأساسي" />
            </div>
            <div className="input-group">
              <label className="input-label">بدل السكن (مليار)</label>
              <input className="input-field" type="number" value={form.housing_allowance_milli} onChange={(e) => set("housing_allowance_milli", Number(e.target.value))} aria-label="بدل السكن" />
            </div>
            <div className="input-group">
              <label className="input-label">بدل النقل (مليار)</label>
              <input className="input-field" type="number" value={form.transport_allowance_milli} onChange={(e) => set("transport_allowance_milli", Number(e.target.value))} aria-label="بدل النقل" />
            </div>
            <div className="input-group">
              <label className="input-label">بدل الطعام (مليار)</label>
              <input className="input-field" type="number" value={form.food_allowance_milli} onChange={(e) => set("food_allowance_milli", Number(e.target.value))} aria-label="بدل الطعام" />
            </div>
            <div className="input-group">
              <label className="input-label">بدلات أخرى (مليار)</label>
              <input className="input-field" type="number" value={form.other_allowances_milli} onChange={(e) => set("other_allowances_milli", Number(e.target.value))} aria-label="بدلات أخرى" />
            </div>
            <div className="input-group">
              <label className="input-label">البدلات الإجمالية (مليار)</label>
              <input className="input-field" type="number" value={form.allowances_milli} onChange={(e) => set("allowances_milli", Number(e.target.value))} aria-label="البدلات الإجمالية" />
            </div>
            <div className="input-group">
              <label className="input-label">أجر الساعة الإضافية (مليار)</label>
              <input className="input-field" type="number" value={form.overtime_rate_milli} onChange={(e) => set("overtime_rate_milli", Number(e.target.value))} aria-label="أجر الساعة الإضافية" />
            </div>
          </div>
        </Card>

        <Card className="mt-4">
          <h3 className="section-title mb-4">الوثائق والتواريخ</h3>
          <div className="grid grid-cols-3 gap-6">
            <div className="input-group">
              <label className="input-label">رقم الجواز</label>
              <input className="input-field" value={form.passport_no} onChange={(e) => set("passport_no", e.target.value)} aria-label="رقم الجواز" />
            </div>
            <div className="input-group">
              <label className="input-label">انتهاء الجواز</label>
              <input className="input-field" type="date" value={form.passport_expiry} onChange={(e) => set("passport_expiry", e.target.value)} aria-label="انتهاء الجواز" />
            </div>
            <div className="input-group">
              <label className="input-label">انتهاء الإقامة</label>
              <input className="input-field" type="date" value={form.residence_expiry} onChange={(e) => set("residence_expiry", e.target.value)} aria-label="انتهاء الإقامة" />
            </div>
            <div className="input-group">
              <label className="input-label">انتهاء التأشيرة</label>
              <input className="input-field" type="date" value={form.visa_expiry} onChange={(e) => set("visa_expiry", e.target.value)} aria-label="انتهاء التأشيرة" />
            </div>
            <div className="input-group">
              <label className="input-label">انتهاء تصريح العمل</label>
              <input className="input-field" type="date" value={form.workpermit_expiry} onChange={(e) => set("workpermit_expiry", e.target.value)} aria-label="انتهاء تصريح العمل" />
            </div>
            <div className="input-group">
              <label className="input-label">انتهاء التأمين</label>
              <input className="input-field" type="date" value={form.insurance_expiry} onChange={(e) => set("insurance_expiry", e.target.value)} aria-label="انتهاء التأمين" />
            </div>
            <div className="input-group">
              <label className="input-label">رقم بوليصة التأمين</label>
              <input className="input-field" value={form.insurance_policy_no} onChange={(e) => set("insurance_policy_no", e.target.value)} aria-label="رقم بوليصة التأمين" />
            </div>
            <div className="input-group">
              <label className="input-label">قسط التأمين الشهري (مليار)</label>
              <input className="input-field" type="number" value={form.insurance_premium_milli} onChange={(e) => set("insurance_premium_milli", Number(e.target.value))} aria-label="قسط التأمين الشهري" />
            </div>
            <div className="input-group">
              <label className="input-label">انتهاء العقد</label>
              <input className="input-field" type="date" value={form.contract_end} onChange={(e) => set("contract_end", e.target.value)} aria-label="انتهاء العقد" />
            </div>
            <div className="input-group">
              <label className="input-label">بدل تذاكر الطيران السنوي (مليار)</label>
              <input className="input-field" type="number" value={form.ticket_allowance_milli} onChange={(e) => set("ticket_allowance_milli", Number(e.target.value))} aria-label="بدل تذاكر الطيران السنوي" />
            </div>
          </div>
        </Card>

        <Card className="mt-4">
          <h3 className="section-title mb-4">البنك والكفيل</h3>
          <div className="grid grid-cols-2 gap-6">
            <div className="input-group">
              <label className="input-label">اسم البنك</label>
              <input className="input-field" value={form.bank_name} onChange={(e) => set("bank_name", e.target.value)} aria-label="اسم البنك" />
            </div>
            <div className="input-group">
              <label className="input-label">رقم الحساب البنكي</label>
              <input className="input-field" value={form.bank_account_no} onChange={(e) => set("bank_account_no", e.target.value)} aria-label="رقم الحساب البنكي" />
            </div>
            <div className="input-group">
              <label className="input-label">اسم الكفيل</label>
              <input className="input-field" value={form.sponsor_name} onChange={(e) => set("sponsor_name", e.target.value)} aria-label="اسم الكفيل" />
            </div>
            <div className="input-group">
              <label className="input-label">رقم الكفيل</label>
              <input className="input-field" value={form.sponsor_id} onChange={(e) => set("sponsor_id", e.target.value)} aria-label="رقم الكفيل" />
            </div>
          </div>
        </Card>

        <Card className="mt-4">
          <h3 className="section-title mb-4">ملاحظات</h3>
          <div className="input-group">
            <textarea className="input-field" rows={3} value={form.notes} onChange={(e) => set("notes", e.target.value)} aria-label="ملاحظات" />
          </div>
        </Card>

        <div className="flex justify-start gap-3 mt-4">
          <Button type="submit" loading={saving} icon={<Save className="w-4 h-4" />}>{isEdit ? "حفظ التعديلات" : "إضافة الموظف"}</Button>
          <Button variant="outline" type="button" onClick={() => navigate("/hr/employees")}>إلغاء</Button>
        </div>
      </form>
    </div>
  );
}
