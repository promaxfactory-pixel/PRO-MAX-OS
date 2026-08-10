import { useState, useEffect } from "react";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { invoke } from "@tauri-apps/api/core";
import { useLicenseStore } from "@/stores/licenseStore";
import { Save, Building2, Shield, Key, Eye, EyeOff, CheckCircle, Copy } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

export default function SettingsPage() {
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
    company_name_ar: "",
    company_name_en: "",
    vat_number: "",
    cr_number: "",
    address: "",
    phone: "",
    email: "",
    currency: "OMR",
    fiscal_year_start: "01-01",
    vat_rate: 5,
    logo_path: "",
    bank_name: "",
    bank_account_no: "",
    bank_iban: "",
    bank_swift: "",
  });

  useEffect(() => {
    invoke("get_company_settings")
      .then((d: any) => setForm((prev) => ({ ...prev, ...d })))
      .catch((e: unknown) => addNotification({ title: "خطأ", message: String(e), type: "error" }));
  }, []);

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
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء الحفظ" });
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">إعدادات الشركة</h1>
          <p className="page-subtitle">بيانات الشركة والإعدادات العامة</p>
        </div>
        <Button icon={<Save className="w-4 h-4" />} onClick={handleSubmit} loading={saving}>
          {saved ? "تم الحفظ ✓" : "حفظ"}
        </Button>
      </div>

      <div className="grid grid-cols-2 gap-6">
        <Card>
          <div className="flex items-center gap-2 mb-4">
            <Building2 className="w-5 h-5 text-brand-400" />
            <h3 className="section-title">بيانات الشركة</h3>
          </div>
          <div className="space-y-4">
            <div>
              <label className="form-label">اسم الشركة (عربي)</label>
              <input type="text" value={form.company_name_ar} onChange={(e) => handleChange("company_name_ar", e.target.value)} className="input-field" aria-label="اسم الشركة (عربي)" />
            </div>
            <div>
              <label className="form-label">اسم الشركة (إنجليزي)</label>
              <input type="text" value={form.company_name_en} onChange={(e) => handleChange("company_name_en", e.target.value)} className="input-field" dir="ltr" aria-label="اسم الشركة (إنجليزي)" />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="form-label">الرقم الضريبي</label>
                <input type="text" value={form.vat_number} onChange={(e) => handleChange("vat_number", e.target.value)} className="input-field" aria-label="الرقم الضريبي" />
              </div>
              <div>
                <label className="form-label">رقم السجل التجاري</label>
                <input type="text" value={form.cr_number} onChange={(e) => handleChange("cr_number", e.target.value)} className="input-field" aria-label="رقم السجل التجاري" />
              </div>
            </div>
            <div>
              <label className="form-label">العنوان</label>
              <input type="text" value={form.address} onChange={(e) => handleChange("address", e.target.value)} className="input-field" aria-label="العنوان" />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="form-label">الهاتف</label>
                <input type="text" value={form.phone} onChange={(e) => handleChange("phone", e.target.value)} className="input-field" dir="ltr" aria-label="الهاتف" />
              </div>
              <div>
                <label className="form-label">البريد الإلكتروني</label>
                <input type="email" value={form.email} onChange={(e) => handleChange("email", e.target.value)} className="input-field" dir="ltr" aria-label="البريد الإلكتروني" />
              </div>
            </div>
          </div>
        </Card>

        <div className="space-y-6">
          <Card>
            <h3 className="section-title mb-4">الإعدادات المالية</h3>
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="form-label">العملة</label>
                  <select value={form.currency} onChange={(e) => handleChange("currency", e.target.value)} className="input-field" aria-label="العملة">
                    <option value="OMR">ريال عُماني (OMR)</option>
                    <option value="SAR">ريال سعودي (SAR)</option>
                    <option value="AED">درهم إماراتي (AED)</option>
                    <option value="USD">دولار أمريكي (USD)</option>
                  </select>
                </div>
                <div>
                  <label className="form-label">نسبة الضريبة (%)</label>
                  <input type="number" value={form.vat_rate} onChange={(e) => handleChange("vat_rate", Number(e.target.value))} className="input-field" min="0" max="100" aria-label="نسبة الضريبة" />
                </div>
              </div>
              <div>
                <label className="form-label">بداية السنة المالية</label>
                <select value={form.fiscal_year_start} onChange={(e) => handleChange("fiscal_year_start", e.target.value)} className="input-field" aria-label="بداية السنة المالية">
                  <option value="01-01">1 يناير</option>
                  <option value="04-01">1 أبريل</option>
                  <option value="07-01">1 يوليو</option>
                  <option value="10-01">1 أكتوبر</option>
                </select>
              </div>
            </div>
          </Card>

          <Card>
            <h3 className="section-title mb-4">الحساب البنكي</h3>
            <div className="space-y-4">
              <div>
                <label className="form-label">اسم البنك</label>
                <input type="text" value={form.bank_name} onChange={(e) => handleChange("bank_name", e.target.value)} className="input-field" aria-label="اسم البنك" />
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="form-label">رقم الحساب</label>
                  <input type="text" value={form.bank_account_no} onChange={(e) => handleChange("bank_account_no", e.target.value)} className="input-field" dir="ltr" aria-label="رقم الحساب" />
                </div>
                <div>
                  <label className="form-label">IBAN</label>
                  <input type="text" value={form.bank_iban} onChange={(e) => handleChange("bank_iban", e.target.value)} className="input-field" dir="ltr" aria-label="رقم الحساب البنكي" />
                </div>
              </div>
              <div>
                <label className="form-label">SWIFT Code</label>
                <input type="text" value={form.bank_swift} onChange={(e) => handleChange("bank_swift", e.target.value)} className="input-field dir-ltr" dir="ltr" aria-label="رمز السويفت" />
              </div>
            </div>
          </Card>

          {/* Developer License Manager */}
          <Card>
            <div className="flex items-center gap-2 mb-4">
              <Shield className="w-5 h-5 text-gold-400" />
              <h3 className="section-title">حالة الترخيص</h3>
            </div>
            <div className="space-y-3">
              <div className="flex items-center justify-between p-3 rounded-lg bg-zinc-800/50">
                <span className="text-gray-400">حالة الترخيص</span>
                <span className={`flex items-center gap-1.5 font-medium ${isLicensed ? "text-green-400" : "text-red-400"}`}>
                  <span className={`w-2 h-2 rounded-full ${isLicensed ? "bg-green-400" : "bg-red-400"}`} />
                  {isLicensed ? "مفعل" : "غير مفعل"}
                </span>
              </div>
              {license && (
                <>
                  <div className="flex justify-between p-3 rounded-lg bg-zinc-800/50">
                    <span className="text-gray-400">العميل</span>
                    <span className="text-white font-medium">{license.customer_name}</span>
                  </div>
                  <div className="flex justify-between p-3 rounded-lg bg-zinc-800/50">
                    <span className="text-gray-400">النوع</span>
                    <span className="text-white font-medium">
                      {license.license_type === "perpetual" ? "دائم" :
                        license.license_type === "subscription" ? "اشتراك" :
                          license.license_type === "trial" ? "تجريبي" : license.license_type}
                    </span>
                  </div>
                  {license.expires_at && (
                    <div className="flex justify-between p-3 rounded-lg bg-zinc-800/50">
                      <span className="text-gray-400">تاريخ الانتهاء</span>
                      <span className="text-white font-medium">{license.expires_at}</span>
                    </div>
                  )}
                  {license.days_remaining !== null && (
                    <div className="flex justify-between p-3 rounded-lg bg-zinc-800/50">
                      <span className="text-gray-400">الأيام المتبقية</span>
                      <span className={`font-medium ${license.days_remaining < 30 ? "text-amber-400" : "text-green-400"}`}>
                        {license.days_remaining} يوم
                      </span>
                    </div>
                  )}
                </>
              )}
              <p className="text-xs text-gray-600 mt-2">
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
                <h3 className="section-title">إدارة التراخيص (المطور)</h3>
              </div>

              {!pinVerified ? (
                <div className="space-y-3">
                  <p className="text-sm text-gray-400">أدخل رمز المطور للوصول إلى إدارة التراخيص</p>
                  <div className="flex gap-2">
                    <div className="relative flex-1">
                      <input
                        type={showPin ? "text" : "password"}
                        value={devPin}
                        onChange={(e) => { setDevPin(e.target.value); setPinError(""); }}
                        placeholder="رقم التعريف"
                        className="input-field w-full pl-10 font-mono"
                        aria-label="رقم التعريف"
                        onKeyDown={(e) => e.key === "Enter" && (async () => {
                          try {
                            const ok = await invoke("verify_developer_pin", { pin: devPin });
                            if (ok) { setPinVerified(true); setPinError(""); }
                            else setPinError("رقم التعريف غير صحيح");
                          } catch { setPinError("خطأ في التحقق"); }
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
                        else setPinError("رقم التعريف غير صحيح");
                      } catch { setPinError("خطأ في التحقق"); }
                    }}>دخول</Button>
                  </div>
                  {pinError && <p className="text-red-400 text-sm">{pinError}</p>}
                </div>
              ) : (
                <div className="space-y-4">
                  <div className="flex items-center gap-2 text-green-400 text-sm">
                    <CheckCircle className="w-4 h-4" />
                    <span>تم التحقق - وضع المطور</span>
                  </div>

                  <div className="border-t border-zinc-700 pt-4">
                    <h4 className="text-sm font-medium text-white mb-3">إنشاء ترخيص جديد</h4>
                    <div className="space-y-3">
                      <div>
                        <label className="form-label">اسم العميل</label>
                        <input type="text" value={genCustName} onChange={(e) => setGenCustName(e.target.value)} className="input-field" placeholder="اسم الشركة" aria-label="اسم العميل" />
                      </div>
                      <div>
                        <label className="form-label">نوع الترخيص</label>
                        <select value={genLicType} onChange={(e) => setGenLicType(e.target.value)} className="input-field" aria-label="نوع الترخيص">
                          <option value="perpetual">دائم</option>
                          <option value="subscription">اشتراك</option>
                          <option value="trial">تجريبي</option>
                          <option value="enterprise">مؤسسة</option>
                        </select>
                      </div>
                      <div className="grid grid-cols-2 gap-3">
                        <div>
                          <label className="form-label">عدد الأيام (0 = دائم)</label>
                          <input type="number" value={genDays} onChange={(e) => setGenDays(e.target.value)} className="input-field" min="0" aria-label="عدد الأيام" />
                        </div>
                        <div>
                          <label className="form-label">عدد المستخدمين</label>
                          <input type="number" value={genMaxUsers} onChange={(e) => setGenMaxUsers(e.target.value)} className="input-field" min="1" aria-label="عدد المستخدمين" />
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
                          setGenResult(`خطأ: ${String(err)}`);
                        } finally { setGenLoading(false); }
                      }} loading={genLoading} icon={<Key className="w-4 h-4" />}>
                        إنشاء مفتاح الترخيص
                      </Button>

                      {genResult && (
                        <div className="mt-2">
                          <label className="form-label">مفتاح الترخيص</label>
                          <div className="relative">
                            <textarea
                              value={genResult}
                              readOnly
                              className="input-field w-full h-24 font-mono text-xs"
                              dir="ltr"
                              aria-label="مفتاح الترخيص"
                            />
                            <button onClick={() => { navigator.clipboard.writeText(genResult); }} className="absolute top-2 right-2 p-1.5 rounded bg-zinc-700 hover:bg-zinc-600 text-gray-300 transition-colors">
                              <Copy className="w-4 h-4" />
                            </button>
                          </div>
                          <p className="text-xs text-gray-500 mt-1">انسخ المفتاح وأرسله للعميل للتفعيل</p>
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
