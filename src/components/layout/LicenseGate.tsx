import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useLicenseStore } from "@/stores/licenseStore";
import LicenseActivationPage from "@/pages/auth/LicenseActivationPage";

export default function LicenseGate({ children }: { children: React.ReactNode }) {
  const { t } = useTranslation();
  const { isLicensed, isChecking, checkLicense } = useLicenseStore();

  useEffect(() => { checkLicense(); }, []);

  if (isChecking) {
    return (
      <div className="min-h-screen bg-zinc-950 flex items-center justify-center" dir="rtl">
        <div className="text-center">
          <div className="w-16 h-16 mx-auto mb-6 border-4 border-brand-800 border-t-gold-400 rounded-full animate-spin" />
          <p className="text-gray-400 text-xl">{t("license.checking")}</p>
        </div>
      </div>
    );
  }

  if (!isLicensed) {
    return <LicenseActivationPage />;
  }

  return <>{children}</>;
}
