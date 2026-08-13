import { create } from "zustand";

interface LicenseInfo {
  customer_name: string;
  license_type: string;
  expires_at: string | null;
  features: string[];
  max_users: number;
  days_remaining: number | null;
}

interface LicenseState {
  isLicensed: boolean;
  isLoading: boolean;
  isChecking: boolean;
  license: LicenseInfo | null;
  message: string;
  checkLicense: () => Promise<void>;
  activateLicense: (key: string) => Promise<boolean>;
}

export const useLicenseStore = create<LicenseState>((set) => ({
  isLicensed: false,
  isLoading: false,
  isChecking: true,
  license: null,
  message: "",
  checkLicense: async () => {
    set({ isChecking: true });
    try {
      const { invoke } = await import("@/lib/tauri");
      const result = await invoke<{ valid: boolean; message: string; license: LicenseInfo | null }>("check_license");
      set({
        isLicensed: result.valid,
        isChecking: false,
        license: result.license,
        message: result.message,
      });
    } catch {
      set({ isLicensed: false, isChecking: false, message: "تعذر التحقق من الترخيص" });
    }
  },
  activateLicense: async (key: string) => {
    set({ isLoading: true });
    try {
      const { invoke } = await import("@/lib/tauri");
      const result = await invoke<{ valid: boolean; message: string; license: LicenseInfo | null }>("activate_license", { licenseKey: key });
      if (result.valid) {
        set({ isLicensed: true, isLoading: false, license: result.license, message: result.message });
        return true;
      } else {
        set({ isLoading: false, message: result.message });
        return false;
      }
    } catch (err: unknown) {
      set({ isLoading: false, message: err instanceof Error ? err.message : String(err) });
      return false;
    }
  },
}));
