import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, Save } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { useTranslation } from "react-i18next";
import type { Supplier } from "@/types";

export default function SupplierFormPage() {
  const { t } = useTranslation();
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
      }).catch((e: unknown) => addNotification({ title: t("common.error"), message: String(e), type: 'error' })).finally(() => setLoading(false));
    }
  }, [id, isEdit]);

  const set = (key: string, val: string | number | boolean) => setForm((f) => ({ ...f, [key]: val }));

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!form.name.trim()) return addNotification({ id: crypto.randomUUID(), type: "warning", title: t("supplier.notice"), message: t("supplier.nameRequired") });
    setSaving(true);
    try {
      if (isEdit) {
        await invoke("update_supplier", { id: Number(id), input: form });
        addNotification({ id: crypto.randomUUID(), type: "success", title: t("common.success"), message: t("supplier.saveSuccess") });
        navigate(`/suppliers/${id}`);
      } else {
        const newId = await invoke("create_supplier", { input: form });
        addNotification({ id: crypto.randomUUID(), type: "success", title: t("common.success"), message: t("supplier.saveSuccess") });
        navigate(`/suppliers/${newId}`);
      }
    } catch (err: unknown) { addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("supplier.saveError") }); }
    finally { setSaving(false); }
  };

  if (loading) return <div className="flex items-center justify-center h-64"><div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" /></div>;

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div className="flex items-center gap-3">
          <button onClick={() => navigate('/suppliers')} className="btn-ghost p-2"><ArrowRight className="w-5 h-5" /></button>
          <div>
            <h1 className="page-title">{isEdit ? t("supplier.editTitle") : t("supplier.addNewSupplier")}</h1>
            <p className="page-subtitle">{isEdit ? t("supplier.editSubtitle", { name: form.name }) : t("supplier.addNewSupplier")}</p>
          </div>
        </div>
      </div>

      <form onSubmit={handleSubmit}>
        <Card>
          <div className="grid grid-cols-2 gap-6">
            <div className="input-group">
              <label className="input-label">{t("supplier.nameLabel")}</label>
              <input className="input-field" value={form.name} onChange={(e) => set("name", e.target.value)} required aria-label={t("supplier.nameAria")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("supplier.code")}</label>
              <input className="input-field" value={form.code} onChange={(e) => set("code", e.target.value)} aria-label={t("supplier.code")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("supplier.contactPerson")}</label>
              <input className="input-field" value={form.contact} onChange={(e) => set("contact", e.target.value)} aria-label={t("supplier.contactPerson")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("supplier.phone")}</label>
              <input className="input-field" value={form.phone} onChange={(e) => set("phone", e.target.value)} aria-label={t("supplier.phone")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("supplier.email")}</label>
              <input className="input-field" type="email" value={form.email} onChange={(e) => set("email", e.target.value)} aria-label={t("supplier.email")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("supplier.vatNumber")}</label>
              <input className="input-field" value={form.vat_number} onChange={(e) => set("vat_number", e.target.value)} aria-label={t("supplier.vatNumber")} />
            </div>
            <div className="input-group col-span-2">
              <label className="input-label">{t("supplier.address")}</label>
              <input className="input-field" value={form.address} onChange={(e) => set("address", e.target.value)} aria-label={t("supplier.address")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("supplier.currency")}</label>
              <select className="input-field" value={form.currency} onChange={(e) => set("currency", e.target.value)} aria-label={t("supplier.currency")}>
                <option value="OMR">{t("supplier.currencyOptionOMR")}</option>
                <option value="USD">{t("supplier.currencyOptionUSD")}</option>
                <option value="SAR">{t("supplier.currencyOptionSAR")}</option>
                <option value="AED">{t("supplier.currencyOptionAED")}</option>
              </select>
            </div>
            <div className="input-group">
              <label className="input-label">{t("supplier.paymentTerms")}</label>
              <input className="input-field" value={form.payment_terms} onChange={(e) => set("payment_terms", e.target.value)} placeholder={t("supplier.paymentTermsPlaceholder")} aria-label={t("supplier.paymentTerms")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("supplier.openingBalanceMilli")}</label>
              <input className="input-field" type="number" value={form.opening_balance_milli} onChange={(e) => set("opening_balance_milli", Number(e.target.value))} aria-label={t("supplier.openingBalance")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("common.notes")}</label>
              <input className="input-field" value={form.notes} onChange={(e) => set("notes", e.target.value)} aria-label={t("common.notes")} />
            </div>
          </div>
        </Card>
        <div className="flex justify-start gap-3 mt-4">
          <Button type="submit" loading={saving} icon={<Save className="w-4 h-4" />}>{isEdit ? t("supplier.saveChanges") : t("supplier.addSupplier")}</Button>
          <Button variant="outline" type="button" onClick={() => navigate('/suppliers')}>{t("common.cancel")}</Button>
        </div>
      </form>
    </div>
  );
}
