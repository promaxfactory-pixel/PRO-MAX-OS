import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, Save } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import type { Supplier } from "@/types";

export default function SupplierFormPage() {
  const { id } = useParams();
  const navigate = useNavigate();
  const isEdit = Boolean(id);
  const addNotification = useUIStore((s) => s.addNotification);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [form, setForm] = useState({
    name: "", code: "", contact: "", phone: "", email: "",
    address: "", vat_number: "", currency: "OMR", payment_terms: "",
    opening_balance_milli: 0, notes: "",
  });

  useEffect(() => {
    if (isEdit) {
      setLoading(true);
      invoke<Supplier>("get_supplier", { id: Number(id) }).then((d) => {
        setForm({
          name: d.name || "", code: d.code || "", contact: d.contact || "",
          phone: d.phone || "", email: d.email || "", address: d.address || "",
          vat_number: "", currency: d.currency || "OMR",
          payment_terms: d.payment_terms || "", opening_balance_milli: d.opening_balance_milli || 0,
          notes: d.notes || "",
        });
      }).catch((e: any) => addNotification({ title: 'خطأ', message: String(e), type: 'error' })).finally(() => setLoading(false));
    }
  }, [id, isEdit]);

  const set = (key: string, val: string | number | boolean) => setForm((f) => ({ ...f, [key]: val }));

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!form.name.trim()) return addNotification({ id: crypto.randomUUID(), type: "warning", title: "تنبيه", message: "اسم المورد مطلوب" });
    setSaving(true);
    try {
      if (isEdit) {
        await invoke("update_supplier", { id: Number(id), input: form });
        addNotification({ id: crypto.randomUUID(), type: "success", title: "تم بنجاح", message: "تم حفظ بيانات المورد بنجاح" });
        navigate(`/suppliers/${id}`);
      } else {
        const newId = await invoke("create_supplier", { input: form });
        addNotification({ id: crypto.randomUUID(), type: "success", title: "تم بنجاح", message: "تم حفظ بيانات المورد بنجاح" });
        navigate(`/suppliers/${newId}`);
      }
    } catch (err: unknown) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "فشل في حفظ بيانات المورد" }); }
    finally { setSaving(false); }
  };

  if (loading) return <div className="flex items-center justify-center h-64"><div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" /></div>;

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div className="flex items-center gap-3">
          <button onClick={() => navigate('/suppliers')} className="btn-ghost p-2"><ArrowRight className="w-5 h-5" /></button>
          <div>
            <h1 className="page-title">{isEdit ? "تعديل بيانات المورد" : "إضافة مورد جديد"}</h1>
            <p className="page-subtitle">{isEdit ? `تعديل ${form.name}` : "إضافة مورد جديد"}</p>
          </div>
        </div>
      </div>

      <form onSubmit={handleSubmit}>
        <Card>
          <div className="grid grid-cols-2 gap-6">
            <div className="input-group">
              <label className="input-label">اسم المورد *</label>
              <input className="input-field" value={form.name} onChange={(e) => set("name", e.target.value)} required />
            </div>
            <div className="input-group">
              <label className="input-label">الكود</label>
              <input className="input-field" value={form.code} onChange={(e) => set("code", e.target.value)} />
            </div>
            <div className="input-group">
              <label className="input-label">جهة الاتصال</label>
              <input className="input-field" value={form.contact} onChange={(e) => set("contact", e.target.value)} />
            </div>
            <div className="input-group">
              <label className="input-label">الهاتف</label>
              <input className="input-field" value={form.phone} onChange={(e) => set("phone", e.target.value)} />
            </div>
            <div className="input-group">
              <label className="input-label">البريد الإلكتروني</label>
              <input className="input-field" type="email" value={form.email} onChange={(e) => set("email", e.target.value)} />
            </div>
            <div className="input-group">
              <label className="input-label">رقم الضريبة</label>
              <input className="input-field" value={form.vat_number} onChange={(e) => set("vat_number", e.target.value)} />
            </div>
            <div className="input-group col-span-2">
              <label className="input-label">العنوان</label>
              <input className="input-field" value={form.address} onChange={(e) => set("address", e.target.value)} />
            </div>
            <div className="input-group">
              <label className="input-label">العملة</label>
              <select className="input-field" value={form.currency} onChange={(e) => set("currency", e.target.value)}>
                <option value="OMR">ريال عmani (OMR)</option>
                <option value="USD">دولار أمريكي (USD)</option>
                <option value="SAR">ريال سعودي (SAR)</option>
                <option value="AED">درهم إماراتي (AED)</option>
              </select>
            </div>
            <div className="input-group">
              <label className="input-label">شروط الدفع</label>
              <input className="input-field" value={form.payment_terms} onChange={(e) => set("payment_terms", e.target.value)} placeholder="مثال: 30 يوم" />
            </div>
            <div className="input-group">
              <label className="input-label">الرصيد الافتتاحي (مليار)</label>
              <input className="input-field" type="number" value={form.opening_balance_milli} onChange={(e) => set("opening_balance_milli", Number(e.target.value))} />
            </div>
            <div className="input-group">
              <label className="input-label">ملاحظات</label>
              <input className="input-field" value={form.notes} onChange={(e) => set("notes", e.target.value)} />
            </div>
          </div>
        </Card>
        <div className="flex justify-start gap-3 mt-4">
          <Button type="submit" loading={saving} icon={<Save className="w-4 h-4" />}>{isEdit ? "حفظ التعديلات" : "إضافة المورد"}</Button>
          <Button variant="outline" type="button" onClick={() => navigate('/suppliers')}>إلغاء</Button>
        </div>
      </form>
    </div>
  );
}
