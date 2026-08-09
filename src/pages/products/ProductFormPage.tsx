import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, Save } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import type { Product } from "@/types";

export default function ProductFormPage() {
  const { t } = useTranslation();
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
      }).catch((e: unknown) => addNotification({ title: t('common.error'), message: String(e), type: 'error' })).finally(() => setLoading(false));
    }
  }, [id, isEdit, t]);

  const set = (key: string, val: string | number | boolean) => setForm((f) => ({ ...f, [key]: val }));

  const buildPayload = () => ({
    code: form.code || null,
    name_ar: form.name_ar || null,
    name_en: form.name_en || null,
    size: form.size || null,
    cup_type: form.cup_type || null,
    cups_per_carton: form.cups_per_carton || null,
    default_price_milli: form.default_price_milli || null,
    default_cost_milli: form.default_cost_milli || null,
    barcode: form.barcode || null,
    notes: form.notes || null,
    product_type: form.product_type || null,
    brand_name: form.brand_name || null,
    family_id: form.family_id ? Number(form.family_id) : null,
    cup_size_ml: form.cup_size_ml || null,
    cup_diameter_mm: form.cup_diameter_mm || null,
    paper_weight_gsm: form.paper_weight_gsm || null,
    lid_type: form.lid_type || null,
    material_type: form.material_type || null,
    color: form.color || null,
    print_colors: form.print_colors || null,
    carton_length_cm: form.carton_length_cm || null,
    carton_width_cm: form.carton_width_cm || null,
    carton_height_cm: form.carton_height_cm || null,
    weight_kg: form.weight_kg || null,
    min_stock: form.min_stock || null,
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!form.name_ar.trim()) return addNotification({ id: crypto.randomUUID(), type: "warning", title: t("common.warning"), message: t("productForm.nameRequired") });
    setSaving(true);
    try {
      if (isEdit) {
        await invoke("update_product", { id: Number(id), input: buildPayload() });
        addNotification({ id: crypto.randomUUID(), type: "success", title: t("common.success"), message: t("productForm.saveSuccess") });
        navigate(`/products/${id}`);
      } else {
        const newId = await invoke("create_product", { input: buildPayload() });
        addNotification({ id: crypto.randomUUID(), type: "success", title: t("common.success"), message: t("productForm.saveSuccess") });
        navigate(`/products/${newId}`);
      }
    } catch (err: unknown) { addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("productForm.saveFailed") }); }
    finally { setSaving(false); }
  };

  if (loading) return <div className="flex items-center justify-center h-64"><div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" /></div>;

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div className="flex items-center gap-3">
          <button onClick={() => navigate('/products')} className="btn-ghost p-2"><ArrowRight className="w-5 h-5" /></button>
          <div>
            <h1 className="page-title">{isEdit ? t("productForm.editTitle") : t("productForm.newTitle")}</h1>
            <p className="page-subtitle">{isEdit ? t("productForm.editSubtitle", { name: form.name_ar }) : t("productForm.addNewSubtitle")}</p>
          </div>
        </div>
      </div>

      <form onSubmit={handleSubmit}>
        <Card>
          <h3 className="section-title mb-4">{t("productForm.basicInfo")}</h3>
          <div className="grid grid-cols-3 gap-6">
            <div className="input-group">
              <label className="input-label">{t("productForm.nameArLabel")}</label>
              <input className="input-field" value={form.name_ar} onChange={(e) => set("name_ar", e.target.value)} required aria-label={t("productForm.nameArAria")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("productForm.nameEnLabel")}</label>
              <input className="input-field" value={form.name_en} onChange={(e) => set("name_en", e.target.value)} dir="ltr" aria-label={t("productForm.nameEnAria")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("inventory.code")}</label>
              <input className="input-field" value={form.code} onChange={(e) => set("code", e.target.value)} aria-label={t("inventory.code")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("productForm.size")}</label>
              <input className="input-field" value={form.size} onChange={(e) => set("size", e.target.value)} placeholder={t("productForm.sizePlaceholder")} aria-label={t("productForm.size")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("productForm.cupType")}</label>
              <input className="input-field" value={form.cup_type} onChange={(e) => set("cup_type", e.target.value)} placeholder={t("productForm.cupTypePlaceholder")} aria-label={t("productForm.cupType")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("productForm.barcode")}</label>
              <input className="input-field" value={form.barcode} onChange={(e) => set("barcode", e.target.value)} aria-label={t("productForm.barcode")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("productForm.productType")}</label>
              <select className="input-field" value={form.product_type} onChange={(e) => set("product_type", e.target.value)} aria-label={t("productForm.productType")}>
                <option value="">{t("productForm.selectPlaceholder")}</option>
                <option value="كوب">{t("productForm.typeCup")}</option>
                <option value="كرتون">{t("productForm.typeCarton")}</option>
                <option value="خام">{t("inventory.raw")}</option>
                <option value="جاهز">{t("productForm.typeReady")}</option>
              </select>
            </div>
            <div className="input-group">
              <label className="input-label">{t("productForm.brandName")}</label>
              <input className="input-field" value={form.brand_name} onChange={(e) => set("brand_name", e.target.value)} aria-label={t("productForm.brandNameAria")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("productForm.productFamily")}</label>
              <input className="input-field" value={form.family_id} onChange={(e) => set("family_id", e.target.value)} aria-label={t("productForm.productFamily")} />
            </div>
          </div>
        </Card>

        <Card className="mt-4">
          <h3 className="section-title mb-4">{t("productForm.cupSpecs")}</h3>
          <div className="grid grid-cols-3 gap-6">
            <div className="input-group">
              <label className="input-label">{t("productForm.cupSizeMl")}</label>
              <input className="input-field" type="number" value={form.cup_size_ml} onChange={(e) => set("cup_size_ml", Number(e.target.value))} aria-label={t("productForm.cupSizeAria")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("productForm.cupDiameterMm")}</label>
              <input className="input-field" type="number" value={form.cup_diameter_mm} onChange={(e) => set("cup_diameter_mm", Number(e.target.value))} aria-label={t("productForm.cupDiameterAria")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("productForm.paperWeight")}</label>
              <input className="input-field" type="number" value={form.paper_weight_gsm} onChange={(e) => set("paper_weight_gsm", Number(e.target.value))} aria-label={t("productForm.paperWeightAria")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("productForm.lidType")}</label>
              <select className="input-field" value={form.lid_type} onChange={(e) => set("lid_type", e.target.value)} aria-label={t("productForm.lidType")}>
                <option value="">{t("productForm.selectPlaceholder")}</option>
                <option value="بلا غطاء">{t("productForm.lidNone")}</option>
                <option value="غطاء مسطح">{t("productForm.lidFlat")}</option>
                <option value="غطاء قبة">{t("productForm.lidDome")}</option>
              </select>
            </div>
            <div className="input-group">
              <label className="input-label">{t("productForm.materialType")}</label>
              <select className="input-field" value={form.material_type} onChange={(e) => set("material_type", e.target.value)} aria-label={t("productForm.materialType")}>
                <option value="">{t("productForm.selectPlaceholder")}</option>
                <option value="بلاستيك PE">{t("productForm.materialPe")}</option>
                <option value="بلاستيك PLA">{t("productForm.materialPla")}</option>
                <option value="ورق">{t("productForm.materialPaper")}</option>
                <option value="أخرى">{t("productForm.other")}</option>
              </select>
            </div>
            <div className="input-group">
              <label className="input-label">{t("productForm.color")}</label>
              <input className="input-field" value={form.color} onChange={(e) => set("color", e.target.value)} aria-label={t("productForm.color")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("productForm.printColors")}</label>
              <input className="input-field" type="number" value={form.print_colors} onChange={(e) => set("print_colors", Number(e.target.value))} aria-label={t("productForm.printColors")} />
            </div>
          </div>
        </Card>

        <Card className="mt-4">
          <h3 className="section-title mb-4">{t("productForm.cartonSpecs")}</h3>
          <div className="grid grid-cols-3 gap-6">
            <div className="input-group">
              <label className="input-label">{t("productForm.cupsPerCarton")}</label>
              <input className="input-field" type="number" value={form.cups_per_carton} onChange={(e) => set("cups_per_carton", Number(e.target.value))} aria-label={t("productForm.cupsPerCartonAria")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("productForm.cartonType")}</label>
              <input className="input-field" value={form.carton_type} onChange={(e) => set("carton_type", e.target.value)} aria-label={t("productForm.cartonType")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("productForm.cartonLength")}</label>
              <input className="input-field" type="number" value={form.carton_length_cm} onChange={(e) => set("carton_length_cm", Number(e.target.value))} aria-label={t("productForm.cartonLengthAria")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("productForm.cartonWidth")}</label>
              <input className="input-field" type="number" value={form.carton_width_cm} onChange={(e) => set("carton_width_cm", Number(e.target.value))} aria-label={t("productForm.cartonWidthAria")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("productForm.cartonHeight")}</label>
              <input className="input-field" type="number" value={form.carton_height_cm} onChange={(e) => set("carton_height_cm", Number(e.target.value))} aria-label={t("productForm.cartonHeightAria")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("productForm.weightKg")}</label>
              <input className="input-field" type="number" step="0.01" value={form.weight_kg} onChange={(e) => set("weight_kg", Number(e.target.value))} aria-label={t("productForm.weightKgAria")} />
            </div>
          </div>
        </Card>

        <Card className="mt-4">
          <h3 className="section-title mb-4">{t("productForm.pricingInventory")}</h3>
          <div className="grid grid-cols-3 gap-6">
            <div className="input-group">
              <label className="input-label">{t("productForm.defaultPrice")}</label>
              <input className="input-field" type="number" value={form.default_price_milli} onChange={(e) => set("default_price_milli", Number(e.target.value))} aria-label={t("productForm.defaultPriceAria")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("productForm.defaultCost")}</label>
              <input className="input-field" type="number" value={form.default_cost_milli} onChange={(e) => set("default_cost_milli", Number(e.target.value))} aria-label={t("productForm.defaultCostAria")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("productForm.vatPct")}</label>
              <input className="input-field" type="number" step="0.1" value={form.vat_pct} onChange={(e) => set("vat_pct", Number(e.target.value))} aria-label={t("productForm.vatPctAria")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("productForm.minStock")}</label>
              <input className="input-field" type="number" value={form.min_stock} onChange={(e) => set("min_stock", Number(e.target.value))} aria-label={t("productForm.minStock")} />
            </div>
          </div>
        </Card>

        <Card className="mt-4">
          <h3 className="section-title mb-4">{t("common.notes")}</h3>
          <div className="input-group">
            <label className="input-label">{t("common.notes")}</label>
            <textarea className="input-field" rows={3} value={form.notes} onChange={(e) => set("notes", e.target.value)} aria-label={t("common.notes")} />
          </div>
        </Card>

        <div className="flex justify-start gap-3 mt-4">
          <Button type="submit" loading={saving} icon={<Save className="w-4 h-4" />}>{isEdit ? t("productForm.saveChanges") : t("productForm.addProduct")}</Button>
          <Button variant="outline" type="button" onClick={() => navigate('/products')}>{t("common.cancel")}</Button>
        </div>
      </form>
    </div>
  );
}
