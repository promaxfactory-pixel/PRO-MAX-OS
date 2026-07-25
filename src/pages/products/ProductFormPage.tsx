import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, Save } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import type { Product } from "@/types";

export default function ProductFormPage() {
  const { id } = useParams();
  const navigate = useNavigate();
  const isEdit = Boolean(id);
  const addNotification = useUIStore((s) => s.addNotification);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [form, setForm] = useState({
    code: "", name_ar: "", name_en: "", size: "", cup_type: "",
    cups_per_carton: 1000, carton_type: "", default_price_milli: 0,
    default_cost_milli: 0, vat_pct: 5, barcode: "", notes: "",
  });

  useEffect(() => {
    if (isEdit) {
      setLoading(true);
      invoke<Product>("get_product", { id: Number(id) }).then((d) => {
        setForm({
          code: d.code || "", name_ar: d.name_ar || "", name_en: d.name_en || "",
          size: d.size || "", cup_type: d.cup_type || "",
          cups_per_carton: d.cups_per_carton || 1000, carton_type: d.carton_type || "",
          default_price_milli: d.default_price_milli || 0, default_cost_milli: d.default_cost_milli || 0,
          vat_pct: d.vat_pct || 5, barcode: d.barcode || "", notes: d.notes || "",
        });
      }).catch((e: unknown) => addNotification({ title: 'خطأ', message: String(e), type: 'error' })).finally(() => setLoading(false));
    }
  }, [id, isEdit]);

  const set = (key: string, val: string | number | boolean) => setForm((f) => ({ ...f, [key]: val }));

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!form.name_ar.trim()) return addNotification({ id: crypto.randomUUID(), type: "warning", title: "تنبيه", message: "اسم المنتج مطلوب" });
    setSaving(true);
    try {
      if (isEdit) {
        await invoke("update_product", { id: Number(id), input: form });
        addNotification({ id: crypto.randomUUID(), type: "success", title: "تم بنجاح", message: "تم حفظ بيانات المنتج بنجاح" });
        navigate(`/products/${id}`);
      } else {
        const newId = await invoke("create_product", { input: form });
        addNotification({ id: crypto.randomUUID(), type: "success", title: "تم بنجاح", message: "تم حفظ بيانات المنتج بنجاح" });
        navigate(`/products/${newId}`);
      }
    } catch (err: unknown) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "فشل في حفظ بيانات المنتج" }); }
    finally { setSaving(false); }
  };

  if (loading) return <div className="flex items-center justify-center h-64"><div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" /></div>;

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div className="flex items-center gap-3">
          <button onClick={() => navigate('/products')} className="btn-ghost p-2"><ArrowRight className="w-5 h-5" /></button>
          <div>
            <h1 className="page-title">{isEdit ? "تعديل المنتج" : "منتج جديد"}</h1>
            <p className="page-subtitle">{isEdit ? `تعديل ${form.name_ar}` : "إضافة منتج جديد"}</p>
          </div>
        </div>
      </div>

      <form onSubmit={handleSubmit}>
        <Card>
          <h3 className="section-title mb-4">بيانات المنتج</h3>
          <div className="grid grid-cols-3 gap-6">
            <div className="input-group">
              <label className="input-label">اسم المنتج (عربي) *</label>
              <input className="input-field" value={form.name_ar} onChange={(e) => set("name_ar", e.target.value)} required />
            </div>
            <div className="input-group">
              <label className="input-label">اسم المنتج (إنجليزي)</label>
              <input className="input-field" value={form.name_en} onChange={(e) => set("name_en", e.target.value)} dir="ltr" />
            </div>
            <div className="input-group">
              <label className="input-label">الكود</label>
              <input className="input-field" value={form.code} onChange={(e) => set("code", e.target.value)} />
            </div>
            <div className="input-group">
              <label className="input-label">المقاس</label>
              <input className="input-field" value={form.size} onChange={(e) => set("size", e.target.value)} placeholder="مثال: 8oz" />
            </div>
            <div className="input-group">
              <label className="input-label">نوع الكوب</label>
              <input className="input-field" value={form.cup_type} onChange={(e) => set("cup_type", e.target.value)} placeholder="مثال: A-line" />
            </div>
            <div className="input-group">
              <label className="input-label">أكواب/كرتون</label>
              <input className="input-field" type="number" value={form.cups_per_carton} onChange={(e) => set("cups_per_carton", Number(e.target.value))} />
            </div>
            <div className="input-group">
              <label className="input-label">نوع الكرتون</label>
              <input className="input-field" value={form.carton_type} onChange={(e) => set("carton_type", e.target.value)} />
            </div>
            <div className="input-group">
              <label className="input-label">الباركود</label>
              <input className="input-field" value={form.barcode} onChange={(e) => set("barcode", e.target.value)} />
            </div>
            <div className="input-group">
              <label className="input-label">ضريبة %</label>
              <input className="input-field" type="number" step="0.1" value={form.vat_pct} onChange={(e) => set("vat_pct", Number(e.target.value))} />
            </div>
          </div>
        </Card>

        <Card className="mt-4">
          <h3 className="section-title mb-4">التسعير</h3>
          <div className="grid grid-cols-2 gap-6">
            <div className="input-group">
              <label className="input-label">سعر البيع الافتراضي (مليار)</label>
              <input className="input-field" type="number" value={form.default_price_milli} onChange={(e) => set("default_price_milli", Number(e.target.value))} />
            </div>
            <div className="input-group">
              <label className="input-label">التكلفة الافتراضية (مليار)</label>
              <input className="input-field" type="number" value={form.default_cost_milli} onChange={(e) => set("default_cost_milli", Number(e.target.value))} />
            </div>
          </div>
          <div className="input-group mt-4">
            <label className="input-label">ملاحظات</label>
            <textarea className="input-field" rows={2} value={form.notes} onChange={(e) => set("notes", e.target.value)} />
          </div>
        </Card>

        <div className="flex justify-start gap-3 mt-4">
          <Button type="submit" loading={saving} icon={<Save className="w-4 h-4" />}>{isEdit ? "حفظ التعديلات" : "إضافة المنتج"}</Button>
          <Button variant="outline" type="button" onClick={() => navigate('/products')}>إلغاء</Button>
        </div>
      </form>
    </div>
  );
}
