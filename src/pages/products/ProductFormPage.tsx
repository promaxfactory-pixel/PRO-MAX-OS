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
    product_type: "", brand_name: "", family_id: "",
    cup_size_ml: 0, cup_diameter_mm: 0, paper_weight_gsm: 0,
    lid_type: "", material_type: "", color: "", print_colors: 0,
    carton_length_cm: 0, carton_width_cm: 0, carton_height_cm: 0, weight_kg: 0,
    min_stock: 0,
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
          product_type: d.product_type || "", brand_name: d.brand_name || "",
          family_id: d.family_id || "",
          cup_size_ml: d.cup_size_ml || 0, cup_diameter_mm: d.cup_diameter_mm || 0,
          paper_weight_gsm: d.paper_weight_gsm || 0, lid_type: d.lid_type || "",
          material_type: d.material_type || "", color: d.color || "",
          print_colors: d.print_colors || 0,
          carton_length_cm: d.carton_length_cm || 0, carton_width_cm: d.carton_width_cm || 0,
          carton_height_cm: d.carton_height_cm || 0, weight_kg: d.weight_kg || 0,
          min_stock: d.min_stock || 0,
        });
      }).catch((e: unknown) => addNotification({ title: "خطأ", message: String(e), type: "error" })).finally(() => setLoading(false));
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
          <button onClick={() => navigate("/products")} className="btn-ghost p-2"><ArrowRight className="w-5 h-5" /></button>
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
              <input className="input-field" value={form.name_ar} onChange={(e) => set("name_ar", e.target.value)} required aria-label="اسم المنتج بالعربي" />
            </div>
            <div className="input-group">
              <label className="input-label">اسم المنتج (إنجليزي)</label>
              <input className="input-field" value={form.name_en} onChange={(e) => set("name_en", e.target.value)} dir="ltr" aria-label="اسم المنتج بالإنجليزي" />
            </div>
            <div className="input-group">
              <label className="input-label">الكود</label>
              <input className="input-field" value={form.code} onChange={(e) => set("code", e.target.value)} aria-label="الكود" />
            </div>
            <div className="input-group">
              <label className="input-label">المقاس</label>
              <input className="input-field" value={form.size} onChange={(e) => set("size", e.target.value)} placeholder="مثال: 8oz" aria-label="المقاس" />
            </div>
            <div className="input-group">
              <label className="input-label">نوع الكوب</label>
              <input className="input-field" value={form.cup_type} onChange={(e) => set("cup_type", e.target.value)} placeholder="مثال: A-line" aria-label="نوع الكوب" />
            </div>
            <div className="input-group">
              <label className="input-label">الباركود</label>
              <input className="input-field" value={form.barcode} onChange={(e) => set("barcode", e.target.value)} aria-label="الباركود" />
            </div>
            <div className="input-group">
              <label className="input-label">نوع المنتج</label>
              <select className="input-field" value={form.product_type} onChange={(e) => set("product_type", e.target.value)} aria-label="نوع المنتج">
                <option value="">— اختر —</option>
                <option value="كوب">كوب</option>
                <option value="كرتون">كرتون</option>
                <option value="خام">خام</option>
                <option value="جاهز">جاهز</option>
              </select>
            </div>
            <div className="input-group">
              <label className="input-label">اسم البراند / المصنع</label>
              <input className="input-field" value={form.brand_name} onChange={(e) => set("brand_name", e.target.value)} aria-label="اسم البراند" />
            </div>
            <div className="input-group">
              <label className="input-label">عائلة المنتج</label>
              <input className="input-field" value={form.family_id} onChange={(e) => set("family_id", e.target.value)} aria-label="عائلة المنتج" />
            </div>
          </div>
        </Card>

        <Card className="mt-4">
          <h3 className="section-title mb-4">مواصفات الكوب</h3>
          <div className="grid grid-cols-3 gap-6">
            <div className="input-group">
              <label className="input-label">حجم الكوب (مل)</label>
              <input className="input-field" type="number" value={form.cup_size_ml} onChange={(e) => set("cup_size_ml", Number(e.target.value))} aria-label="حجم الكوب بالمل" />
            </div>
            <div className="input-group">
              <label className="input-label">قطر الكوب (مم)</label>
              <input className="input-field" type="number" value={form.cup_diameter_mm} onChange={(e) => set("cup_diameter_mm", Number(e.target.value))} aria-label="قطر الكوب بالملم" />
            </div>
            <div className="input-group">
              <label className="input-label">وزن الورق (جرام/م²)</label>
              <input className="input-field" type="number" value={form.paper_weight_gsm} onChange={(e) => set("paper_weight_gsm", Number(e.target.value))} aria-label="وزن الورق" />
            </div>
            <div className="input-group">
              <label className="input-label">نوع الغطاء</label>
              <select className="input-field" value={form.lid_type} onChange={(e) => set("lid_type", e.target.value)} aria-label="نوع الغطاء">
                <option value="">— اختر —</option>
                <option value="بلا غطاء">بلا غطاء</option>
                <option value="غطاء مسطح">غطاء مسطح</option>
                <option value="غطاء قبة">غطاء قبة</option>
              </select>
            </div>
            <div className="input-group">
              <label className="input-label">نوع المادة</label>
              <select className="input-field" value={form.material_type} onChange={(e) => set("material_type", e.target.value)} aria-label="نوع المادة">
                <option value="">— اختر —</option>
                <option value="بلاستيك PE">بلاستيك PE</option>
                <option value="بلاستيك PLA">بلاستيك PLA</option>
                <option value="ورق">ورق</option>
                <option value="أخرى">أخرى</option>
              </select>
            </div>
            <div className="input-group">
              <label className="input-label">اللون</label>
              <input className="input-field" value={form.color} onChange={(e) => set("color", e.target.value)} aria-label="اللون" />
            </div>
            <div className="input-group">
              <label className="input-label">عدد ألوان الطباعة</label>
              <input className="input-field" type="number" value={form.print_colors} onChange={(e) => set("print_colors", Number(e.target.value))} aria-label="عدد ألوان الطباعة" />
            </div>
          </div>
        </Card>

        <Card className="mt-4">
          <h3 className="section-title mb-4">مواصفات الكرتون</h3>
          <div className="grid grid-cols-3 gap-6">
            <div className="input-group">
              <label className="input-label">أكواب/كرتون</label>
              <input className="input-field" type="number" value={form.cups_per_carton} onChange={(e) => set("cups_per_carton", Number(e.target.value))} aria-label="أكواب لكل كرتون" />
            </div>
            <div className="input-group">
              <label className="input-label">نوع الكرتون</label>
              <input className="input-field" value={form.carton_type} onChange={(e) => set("carton_type", e.target.value)} aria-label="نوع الكرتون" />
            </div>
            <div className="input-group">
              <label className="input-label">طول الكرتون (سم)</label>
              <input className="input-field" type="number" value={form.carton_length_cm} onChange={(e) => set("carton_length_cm", Number(e.target.value))} aria-label="طول الكرتون" />
            </div>
            <div className="input-group">
              <label className="input-label">عرض الكرتون (سم)</label>
              <input className="input-field" type="number" value={form.carton_width_cm} onChange={(e) => set("carton_width_cm", Number(e.target.value))} aria-label="عرض الكرتون" />
            </div>
            <div className="input-group">
              <label className="input-label">ارتفاع الكرتون (سم)</label>
              <input className="input-field" type="number" value={form.carton_height_cm} onChange={(e) => set("carton_height_cm", Number(e.target.value))} aria-label="ارتفاع الكرتون" />
            </div>
            <div className="input-group">
              <label className="input-label">الوزن (كجم)</label>
              <input className="input-field" type="number" step="0.01" value={form.weight_kg} onChange={(e) => set("weight_kg", Number(e.target.value))} aria-label="الوزن بالكيلوجرام" />
            </div>
          </div>
        </Card>

        <Card className="mt-4">
          <h3 className="section-title mb-4">التسعير والمخزون</h3>
          <div className="grid grid-cols-3 gap-6">
            <div className="input-group">
              <label className="input-label">سعر البيع الافتراضي (مليار)</label>
              <input className="input-field" type="number" value={form.default_price_milli} onChange={(e) => set("default_price_milli", Number(e.target.value))} aria-label="سعر البيع الافتراضي" />
            </div>
            <div className="input-group">
              <label className="input-label">التكلفة الافتراضية (مليار)</label>
              <input className="input-field" type="number" value={form.default_cost_milli} onChange={(e) => set("default_cost_milli", Number(e.target.value))} aria-label="التكلفة الافتراضية" />
            </div>
            <div className="input-group">
              <label className="input-label">ضريبة %</label>
              <input className="input-field" type="number" step="0.1" value={form.vat_pct} onChange={(e) => set("vat_pct", Number(e.target.value))} aria-label="نسبة الضريبة" />
            </div>
            <div className="input-group">
              <label className="input-label">الحد الأدنى للمخزون</label>
              <input className="input-field" type="number" value={form.min_stock} onChange={(e) => set("min_stock", Number(e.target.value))} aria-label="الحد الأدنى للمخزون" />
            </div>
          </div>
        </Card>

        <Card className="mt-4">
          <h3 className="section-title mb-4">ملاحظات</h3>
          <div className="input-group">
            <label className="input-label">ملاحظات</label>
            <textarea className="input-field" rows={3} value={form.notes} onChange={(e) => set("notes", e.target.value)} aria-label="ملاحظات" />
          </div>
        </Card>

        <div className="flex justify-start gap-3 mt-4">
          <Button type="submit" loading={saving} icon={<Save className="w-4 h-4" />}>{isEdit ? "حفظ التعديلات" : "إضافة المنتج"}</Button>
          <Button variant="outline" type="button" onClick={() => navigate("/products")}>إلغاء</Button>
        </div>
      </form>
    </div>
  );
}
