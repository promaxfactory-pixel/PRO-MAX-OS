import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useLicenseStore } from "@/stores/licenseStore";
import { Shield, CheckCircle, XCircle, Key, Info } from "lucide-react";
import { useTranslation } from "react-i18next";

export default function LicenseActivationPage() {
  const { t } = useTranslation();
  const [licenseKey, setLicenseKey] = useState("");
  const { activateLicense, isLoading, message, isLicensed, license } = useLicenseStore();
  const navigate = useNavigate();

  const handleActivate = async () => {
    if (!licenseKey.trim()) return;
    await activateLicense(licenseKey.trim());
  };

  return (
    <div className="min-h-screen bg-zinc-950 flex items-center justify-center p-4" dir="rtl">
      <div className="max-w-lg w-full">
        <div className="text-center mb-8">
          <div className="inline-flex items-center justify-center w-20 h-20 rounded-full bg-brand-900/50 border border-gold-500/30 mb-4">
            <Shield className="w-10 h-10 text-gold-400" />
          </div>
          <h1 className="text-3xl font-bold text-white mb-2">PRO MAX OS</h1>
          <p className="text-gray-400">{t("license.tagline")}</p>
        </div>

        <div className="bg-zinc-900 border border-zinc-800 rounded-2xl p-8">
          {!isLicensed ? (
            <>
              <div className="flex items-center gap-3 mb-6">
                <Key className="w-6 h-6 text-gold-400" />
                <h2 className="text-xl font-bold text-white">{t("license.activate")}</h2>
              </div>

              <p className="text-gray-400 mb-6 text-sm">
                {t("license.enterKeyHint")}
              </p>

              <div className="space-y-4">
                <input
                  type="text"
                  value={licenseKey}
                  onChange={(e) => setLicenseKey(e.target.value)}
                  placeholder={t("license.enterKey")}
                  className="w-full px-4 py-3 bg-zinc-800 border border-zinc-700 rounded-xl text-white placeholder-gray-500 focus:outline-none focus:border-gold-500/50 focus:ring-1 focus:ring-gold-500/20 text-center text-sm font-mono"
                  onKeyDown={(e) => e.key === "Enter" && handleActivate()}
                />

                <button
                  onClick={handleActivate}
                  disabled={isLoading || !licenseKey.trim()}
                  className="w-full py-3 bg-gradient-to-r from-gold-500 to-amber-600 text-black font-bold rounded-xl hover:from-gold-400 hover:to-amber-500 disabled:opacity-50 disabled:cursor-not-allowed transition-all"
                >
                  {isLoading ? t("license.activating") : t("license.activate")}
                </button>
              </div>

              {message && (
                <div className={`mt-4 p-3 rounded-xl text-sm flex items-center gap-2 ${
                  message.includes("بن") || message.includes("صالح") || message.includes("خطأ") || message.includes("تالف") || message.includes("آخر") || message.includes("انتهت") || message.includes("خاطئ")
                    ? "bg-red-900/30 border border-red-800/50 text-red-400"
                    : "bg-green-900/30 border border-green-800/50 text-green-400"
                }`}>
                  {message.includes("بن") || message.includes("صالح") || message.includes("خطأ") || message.includes("تالف") || message.includes("آخر") || message.includes("انتهت") || message.includes("خاطئ") ? (
                    <XCircle className="w-5 h-5 flex-shrink-0" />
                  ) : (
                    <CheckCircle className="w-5 h-5 flex-shrink-0" />
                  )}
                  <span>{message}</span>
                </div>
              )}

              <div className="mt-6 p-4 bg-zinc-800/50 rounded-xl border border-zinc-700/50">
                <div className="flex items-start gap-3">
                  <Info className="w-5 h-5 text-gold-400 mt-0.5 flex-shrink-0" />
                  <div className="text-xs text-gray-500 space-y-1">
                    <p>{t("license.noKeyYet")}</p>
                    <p>{t("license.contactSupportHint")}</p>
                    <p className="text-gold-400">license@promax-os.com</p>
                  </div>
                </div>
              </div>
            </>
          ) : (
            <>
              <div className="text-center">
                <div className="inline-flex items-center justify-center w-16 h-16 rounded-full bg-green-900/30 border border-green-500/30 mb-4">
                  <CheckCircle className="w-8 h-8 text-green-400" />
                </div>
                <h2 className="text-xl font-bold text-white mb-2">{t("license.activatedSuccess")}</h2>
                <p className="text-gray-400 mb-6">{t("license.readyToUse")}</p>

                {license && (
                  <div className="text-right bg-zinc-800/50 rounded-xl p-4 space-y-2 text-sm">
                    <div className="flex justify-between">
                      <span className="text-gray-400">{t("license.customerName")}</span>
                      <span className="text-white font-medium">{license.customer_name}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-400">{t("license.licenseType")}</span>
                      <span className="text-white font-medium">
                        {license.license_type === "perpetual" ? t("license.perpetual") :
                         license.license_type === "subscription" ? t("license.subscription") :
                         license.license_type === "trial" ? t("license.trial") : license.license_type}
                      </span>
                    </div>
                    {license.expires_at && (
                      <div className="flex justify-between">
                        <span className="text-gray-400">{t("license.expiry")}</span>
                        <span className="text-white font-medium">{license.expires_at}</span>
                      </div>
                    )}
                    {license.days_remaining !== null && (
                      <div className="flex justify-between">
                        <span className="text-gray-400">{t("license.daysRemaining")}</span>
                        <span className={`font-medium ${license.days_remaining < 30 ? "text-amber-400" : "text-green-400"}`}>
                          {t("license.days", { days: license.days_remaining })}
                        </span>
                      </div>
                    )}
                    <div className="flex justify-between">
                      <span className="text-gray-400">{t("license.maxUsers")}</span>
                      <span className="text-white font-medium">{license.max_users}</span>
                    </div>
                  </div>
                )}

                <button
                  onClick={() => navigate("/login")}
                  className="mt-6 w-full py-3 bg-gradient-to-r from-brand-800 to-brand-900 text-white font-bold rounded-xl hover:from-brand-700 hover:to-brand-800 border border-brand-700 transition-all"
                >
                  {t("license.enterApp")}
                </button>
              </div>
            </>
          )}
        </div>

        <p className="text-center mt-6 text-xs text-[var(--text-muted)]">
          PRO MAX OS v2.0.0 &copy; 2026
        </p>
      </div>
    </div>
  );
}
