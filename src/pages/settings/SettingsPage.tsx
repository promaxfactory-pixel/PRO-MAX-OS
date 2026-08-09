import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { invoke } from "@tauri-apps/api/core";
import { useLicenseStore } from "@/stores/licenseStore";
import { Save, Building2, Shield, Key, Eye, EyeOff, CheckCircle, Copy } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

export default function SettingsPage() {
  const { t } = useTranslation();
  const addNotification = useUIStore((s) => s.addNotification);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);
  const [clickCount, setClickCount] = useState(0);
  const [showDevMode, setShowDevMode] = useState(false);
  const [devPin, setDevPin] = useState("");
  const [pinVerified, setPinVerified] = useState(false);
  const [pinError, setPinError] = useState("");
  const [showPin, setShowPin] = useState(false);
  const [genCustName, setGenCustName] = useState("");
  const [genLicType, setGenLicType] = useState("perpetual");
  const [genDays, setGenDays] = useState("365");
  const [genMaxUsers, setGenMaxUsers] = useState("5");
  const [genResult, setGenResult] = useState("");
  const [genLoading, setGenLoading] = useState(false);
  const { isLicensed, license } = useLicenseStore();

  const [form, setForm] = useState({
    name: "",
    factory_name: "",
    address: "",
    phone: "",
    email: "",
    vat_number: "",
    logo_path: "",
    stamp_path: "",
    signature_path: "",
    footer_notes: "",
    bank_details: "",
    default_vat_pct: 5,
  });

  useEffect(() => {
    invoke("get_company_settings")
      .then((d: any) => setForm((prev) => ({ ...prev, ...d })))
      .catch((e: unknown) => addNotification({ title: t('common.error'), message: String(e), type: 'error' }));
  }, [t]);

  const handleChange = (field: string, value: any) => {
    setForm((prev) => ({ ...prev, [field]: value }));
    setSaved(false);
  };

  const handleSubmit = async () => {
    setSaving(true);
    try {
      await invoke("update_company_settings", { input: form });
      setSaved(true);
      setTimeout(() => setSaved(false), 3000);
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("settings.page.saveError") });
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">{t("settings.page.title")}</h1>
          <p className="page-subtitle">{t("settings.page.subtitle")}</p>
        </div>
        <Button icon={<Save className="w-4 h-4" />} onClick={handleSubmit} loading={saving}>
          {saved ? t("settings.page.saved") : t("common.save")}
        </Button>
      </div>

      <div className="grid grid-cols-2 gap-6">
        <Card>
          <div className="flex items-center gap-2 mb-4">
            <Building2 className="w-5 h-5 text-brand-400" />
            <h3 className="section-title">{t("settings.company")}</h3>
          </div>
          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="form-label">{t("settings.companyName")}</label>
                <input type="text" value={form.name} onChange={(e) => handleChange("name", e.target.value)} className="input-field" aria-label={t("settings.companyName")} />
              </div>
              <div>
                <label className="form-label">{t("settings.page.factoryName")}</label>
                <input type="text" value={form.factory_name} onChange={(e) => handleChange("factory_name", e.target.value)} className="input-field" aria-label={t("settings.page.factoryName")} />
              </div>
            </div>
            <div>
              <label className="form-label">{t("settings.vatNumber")}</label>
              <input type="text" value={form.vat_number} onChange={(e) => handleChange("vat_number", e.target.value)} className="input-field" aria-label={t("settings.vatNumber")} />
            </div>
            <div>
              <label className="form-label">{t("settings.address")}</label>
              <input type="text" value={form.address} onChange={(e) => handleChange("address", e.target.value)} className="input-field" aria-label={t("settings.address")} />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="form-label">{t("settings.phone")}</label>
                <input type="text" value={form.phone} onChange={(e) => handleChange("phone", e.target.value)} className="input-field" dir="ltr" aria-label={t("settings.phone")} />
              </div>
              <div>
                <label className="form-label">{t("settings.email")}</label>
                <input type="email" value={form.email} onChange={(e) => handleChange("email", e.target.value)} className="input-field" dir="ltr" aria-label={t("settings.email")} />
              </div>
            </div>
          </div>
        </Card>

        <div className="space-y-6">
          <Card>
            <h3 className="section-title mb-4">{t("settings.page.logoDocs")}</h3>
            <div className="space-y-4">
              <div>
                <label className="form-label">{t("settings.page.logoPath")}</label>
                <input type="text" value={form.logo_path} onChange={(e) => handleChange("logo_path", e.target.value)} className="input-field" dir="ltr" aria-label={t("settings.page.logoPath")} />
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="form-label">{t("settings.page.stampPath")}</label>
                  <input type="text" value={form.stamp_path} onChange={(e) => handleChange("stamp_path", e.target.value)} className="input-field" dir="ltr" aria-label={t("settings.page.stampPath")} />
                </div>
                <div>
                  <label className="form-label">{t("settings.page.signaturePath")}</label>
                  <input type="text" value={form.signature_path} onChange={(e) => handleChange("signature_path", e.target.value)} className="input-field" dir="ltr" aria-label={t("settings.page.signaturePath")} />
                </div>
              </div>
              <div>
                <label className="form-label">{t("settings.page.footerNotes")}</label>
                <textarea value={form.footer_notes} onChange={(e) => handleChange("footer_notes", e.target.value)} className="input-field min-h-[70px]" placeholder={t("settings.page.footerNotesPlaceholder")} aria-label={t("settings.page.footerNotes")} />
              </div>
            </div>
          </Card>

          <Card>
            <h3 className="section-title mb-4">{t("settings.page.financialSettings")}</h3>
            <div className="space-y-4">
              <div>
                <label className="form-label">{t("settings.page.defaultVatPct")}</label>
                <input type="number" value={form.default_vat_pct} onChange={(e) => handleChange("default_vat_pct", Number(e.target.value))} className="input-field" min="0" max="100" step="0.01" aria-label={t("settings.page.defaultVatPct")} />
              </div>
              <div>
                <label className="form-label">{t("settings.page.bankDetailsLabel")}</label>
                <textarea value={form.bank_details} onChange={(e) => handleChange("bank_details", e.target.value)} className="input-field min-h-[70px]" dir="ltr" placeholder={t("settings.page.bankDetailsPlaceholder")} aria-label={t("settings.page.bankDetailsLabel")} />
              </div>
            </div>
          </Card>

          {/* Developer License Manager */}
          <Card>
            <div className="flex items-center gap-2 mb-4">
              <Shield className="w-5 h-5 text-gold-400" />
              <h3 className="section-title">{t("settings.page.licenseStatus")}</h3>
            </div>
            <div className="space-y-3">
              <div className="flex items-center justify-between p-3 rounded-lg bg-zinc-800/50">
                <span className="text-gray-400">{t("settings.page.licenseStatus")}</span>
                <span className={`flex items-center gap-1.5 font-medium ${isLicensed ? "text-green-400" : "text-red-400"}`}>
                  <span className={`w-2 h-2 rounded-full ${isLicensed ? "bg-green-400" : "bg-red-400"}`} />
                  {isLicensed ? t("settings.page.licenseActive") : t("settings.page.licenseInactive")}
                </span>
              </div>
              {license && (
                <>
                  <div className="flex justify-between p-3 rounded-lg bg-zinc-800/50">
                    <span className="text-gray-400">{t("license.customerName")}</span>
                    <span className="text-white font-medium">{license.customer_name}</span>
                  </div>
                  <div className="flex justify-between p-3 rounded-lg bg-zinc-800/50">
                    <span className="text-gray-400">{t("license.type")}</span>
                    <span className="text-white font-medium">
                      {license.license_type === "perpetual" ? t("license.perpetual") :
                       license.license_type === "subscription" ? t("license.subscription") :
                       license.license_type === "trial" ? t("license.trial") : license.license_type}
                    </span>
                  </div>
                  {license.expires_at && (
                    <div className="flex justify-between p-3 rounded-lg bg-zinc-800/50">
                      <span className="text-gray-400">{t("license.expiry")}</span>
                      <span className="text-white font-medium">{license.expires_at}</span>
                    </div>
                  )}
                  {license.days_remaining !== null && (
                    <div className="flex justify-between p-3 rounded-lg bg-zinc-800/50">
                      <span className="text-gray-400">{t("license.daysRemaining")}</span>
                      <span className={`font-medium ${license.days_remaining < 30 ? "text-amber-400" : "text-green-400"}`}>
                        {t("settings.page.daysRemainingCount", { days: license.days_remaining })}
                      </span>
                    </div>
                  )}
                </>
              )}
              <p className="text-xs text-[var(--text-muted)] mt-2">
                <button onClick={() => { const c = clickCount + 1; setClickCount(c); if (c >= 5) { setShowDevMode(true); setClickCount(0); } }} className="hover:text-gray-400 transition-colors">
                  PRO MAX OS v2.0.0
                </button>
              </p>
            </div>
          </Card>

          {showDevMode && (
            <Card>
              <div className="flex items-center gap-2 mb-4">
                <Key className="w-5 h-5 text-amber-400" />
                <h3 className="section-title">{t("settings.page.devLicenseManager")}</h3>
              </div>

              {!pinVerified ? (
                <div className="space-y-3">
                  <p className="text-sm text-gray-400">{t("settings.page.enterDevPin")}</p>
                  <div className="flex gap-2">
                    <div className="relative flex-1">
                      <input
                        type={showPin ? "text" : "password"}
                        value={devPin}
                        onChange={(e) => { setDevPin(e.target.value); setPinError(""); }}
                        placeholder={t("settings.page.pinPlaceholder")}
                        className="input-field w-full pl-10 font-mono"
                        aria-label={t("settings.page.pinPlaceholder")}
                        onKeyDown={(e) => e.key === "Enter" && (async () => {
                          try {
                            const ok = await invoke("verify_developer_pin", { pin: devPin });
                            if (ok) { setPinVerified(true); setPinError(""); }
                            else setPinError(t("settings.page.pinInvalid"));
                          } catch { setPinError(t("settings.page.pinVerifyError")); }
                        })()}
                      />
                      <button onClick={() => setShowPin(!showPin)} className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-500 hover:text-gray-300">
                        {showPin ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                      </button>
                    </div>
                    <Button onClick={async () => {
                      try {
                        const ok = await invoke("verify_developer_pin", { pin: devPin });
                        if (ok) { setPinVerified(true); setPinError(""); }
                        else setPinError(t("settings.page.pinInvalid"));
                      } catch { setPinError(t("settings.page.pinVerifyError")); }
                    }}>{t("settings.page.enter")}</Button>
                  </div>
                  {pinError && <p className="text-red-400 text-sm">{pinError}</p>}
                </div>
              ) : (
                <div className="space-y-4">
                  <div className="flex items-center gap-2 text-green-400 text-sm">
                    <CheckCircle className="w-4 h-4" />
                    <span>{t("settings.page.devModeVerified")}</span>
                  </div>

                  <div className="border-t border-zinc-700 pt-4">
                    <h4 className="text-sm font-medium text-white mb-3">{t("settings.page.generateLicense")}</h4>
                    <div className="space-y-3">
                      <div>
                        <label className="form-label">{t("settings.page.customerNameLabel")}</label>
                        <input type="text" value={genCustName} onChange={(e) => setGenCustName(e.target.value)} className="input-field" placeholder={t("settings.page.customerNamePlaceholder")} aria-label={t("settings.page.customerNameLabel")} />
                      </div>
                      <div>
                        <label className="form-label">{t("settings.page.licenseTypeLabel")}</label>
                        <select value={genLicType} onChange={(e) => setGenLicType(e.target.value)} className="input-field" aria-label={t("settings.page.licenseTypeLabel")}>
                          <option value="perpetual">{t("license.perpetual")}</option>
                          <option value="subscription">{t("license.subscription")}</option>
                          <option value="trial">{t("license.trial")}</option>
                          <option value="enterprise">{t("license.enterprise")}</option>
                        </select>
                      </div>
                      <div className="grid grid-cols-2 gap-3">
                        <div>
                          <label className="form-label">{t("settings.page.daysCount")}</label>
                          <input type="number" value={genDays} onChange={(e) => setGenDays(e.target.value)} className="input-field" min="0" aria-label={t("settings.page.daysCount")} />
                        </div>
                        <div>
                          <label className="form-label">{t("settings.page.maxUsers")}</label>
                          <input type="number" value={genMaxUsers} onChange={(e) => setGenMaxUsers(e.target.value)} className="input-field" min="1" aria-label={t("settings.page.maxUsers")} />
                        </div>
                      </div>
                      <Button onClick={async () => {
                        if (!genCustName.trim()) return;
                        setGenLoading(true);
                        setGenResult("");
                        try {
                          const key = await invoke<string>("generate_license_key", {
                            pin: devPin,
                            customerName: genCustName.trim(),
                            licenseType: genLicType,
                            expiresDays: genLicType === "perpetual" ? null : parseInt(genDays) || 365,
                            maxUsers: parseInt(genMaxUsers) || 5,
                            features: ["all"],
                          });
                          setGenResult(key);
                        } catch (err: unknown) {
                          setGenResult(t("settings.page.generatedError", { error: String(err) }));
                        } finally { setGenLoading(false); }
                      }} loading={genLoading} icon={<Key className="w-4 h-4" />}>
                        {t("settings.page.generateKey")}
                      </Button>

                      {genResult && (
                        <div className="mt-2">
                          <label className="form-label">{t("settings.page.licenseKey")}</label>
                          <div className="relative">
                            <textarea
                              value={genResult}
                              readOnly
                              className="input-field w-full h-24 font-mono text-xs"
                              dir="ltr"
                              aria-label={t("settings.page.licenseKey")}
                            />
                            <button onClick={() => { navigator.clipboard.writeText(genResult); }} className="absolute top-2 right-2 p-1.5 rounded bg-zinc-700 hover:bg-zinc-600 text-gray-300 transition-colors">
                              <Copy className="w-4 h-4" />
                            </button>
                          </div>
                          <p className="text-xs text-gray-500 mt-1">{t("settings.page.copyKeyHint")}</p>
                        </div>
                      )}
                    </div>
                  </div>
                </div>
              )}
            </Card>
          )}
        </div>
      </div>
    </div>
  );
}
