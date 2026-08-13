import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { motion } from "framer-motion";
import { useTranslation } from "react-i18next";
import { useAuthStore } from "@/stores/authStore";
import { ArrowLeft, Lock, CheckCircle2, AlertCircle, Eye, EyeOff } from "lucide-react";

export default function ChangePasswordPage() {
  const { t, i18n } = useTranslation();
  const navigate = useNavigate();
  const { user } = useAuthStore();
  const [oldPassword, setOldPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [success, setSuccess] = useState(false);
  const [showOld, setShowOld] = useState(false);
  const [showNew, setShowNew] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);
  const isRtl = i18n.language === "ar" || i18n.language === "ur";

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");

    if (newPassword.length < 8) {
      setError(t("settings.passwordMinLength"));
      return;
    }
    if (newPassword !== confirmPassword) {
      setError(t("settings.passwordMismatch"));
      return;
    }

    setLoading(true);
    try {
      const { invoke } = await import("@/lib/tauri");
      await invoke("change_password", {
        userId: user?.id,
        oldPassword,
        newPassword,
      });
      setSuccess(true);
      setTimeout(() => {
        navigate("/");
      }, 500);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : t("settings.changeError"));
    } finally {
      setLoading(false);
    }
  };

  const eyePos = isRtl ? "left-3" : "right-3";
  const paddingStart = isRtl ? "pl-12" : "pr-12";
  const paddingEnd = isRtl ? "pr-12" : "pl-12";

  return (
    <div className="min-h-screen flex items-center justify-center bg-surface-950 relative overflow-hidden">
      <motion.div
        className="relative z-20 w-full max-w-md mx-auto px-6"
        initial={{ opacity: 0, y: 30 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5, ease: [0.16, 1, 0.3, 1] }}
      >
        <div className="absolute -inset-1 bg-gradient-to-r from-brand-500/20 via-gold-400/10 to-brand-500/20 rounded-[2rem] blur-xl opacity-30" />

        <form
          onSubmit={handleSubmit}
          className="relative bg-surface-800/70 backdrop-blur-2xl border border-surface-600/30 rounded-[2rem] p-8 shadow-[0_25px_60px_-15px_rgba(0,0,0,0.5)]"
        >
          <div className="text-center mb-8">
            <div className="w-16 h-16 mx-auto mb-4 rounded-2xl bg-gradient-to-br from-brand-700 to-brand-900 flex items-center justify-center border border-gold-400/20">
              <Lock className="w-8 h-8 text-gold-400" />
            </div>
            <h2 className="text-2xl font-bold text-white mb-1">{t("settings.changePassword")}</h2>
            <p className="text-sm text-surface-400">
              {user?.must_change_password
                ? t("settings.mustChangePassword")
                : t("settings.enterPasswords")}
            </p>
          </div>

          {error && (
            <motion.div
              className="mb-6 p-4 bg-red-500/10 border border-red-500/30 rounded-2xl text-red-400 text-sm flex items-center gap-3"
              initial={{ opacity: 0, y: -10 }}
              animate={{ opacity: 1, y: 0 }}
            >
              <AlertCircle className="w-5 h-5 flex-shrink-0" />
              <span>{error}</span>
            </motion.div>
          )}

          {success ? (
            <motion.div
              className="text-center py-8"
              initial={{ opacity: 0, scale: 0.9 }}
              animate={{ opacity: 1, scale: 1 }}
            >
              <CheckCircle2 className="w-16 h-16 text-emerald-400 mx-auto mb-4" />
              <h3 className="text-xl font-bold text-white mb-2">{t("common.success")}</h3>
              <p className="text-sm text-surface-400">
                {t("settings.passwordChanged")}
              </p>
            </motion.div>
          ) : (
            <div className="space-y-5">
              <div className="relative">
                <label className="input-label mb-1.5 block">{t("settings.currentPassword")}</label>
                <div className="relative">
                  <input
                    type={showOld ? "text" : "password"}
                    value={oldPassword}
                    onChange={(e) => setOldPassword(e.target.value)}
                    placeholder={t("settings.currentPasswordPlaceholder")}
                    className={`w-full ${paddingStart} ${paddingEnd} transition-all duration-300 focus:border-gold-400/50 focus:shadow-[0_0_0_3px_rgba(212,175,55,0.1)] border border-surface-600/50 rounded-2xl py-3.5 px-4 bg-surface-900/50 text-white placeholder-surface-500 text-sm outline-none`}
                    required
                    onFocus={() => setShowOld(true)}
                    onBlur={() => setShowOld(false)}
                    autoComplete="current-password"
                  />
                  <button
                    type="button"
                    onClick={() => setShowOld(!showOld)}
                    className={`absolute ${eyePos} top-1/2 -translate-y-1/2 text-surface-400 hover:text-gold-400 transition-colors p-1`}
                    aria-label={showOld ? t("auth.hidePassword") : t("auth.showPassword")}
                  >
                    {showOld ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                  </button>
                </div>
              </div>

              <div className="relative">
                <label className="input-label mb-1.5 block">{t("settings.newPassword")}</label>
                <div className="relative">
                  <input
                    type={showNew ? "text" : "password"}
                    value={newPassword}
                    onChange={(e) => setNewPassword(e.target.value)}
                    placeholder={t("settings.newPasswordPlaceholder")}
                    className={`w-full ${paddingStart} ${paddingEnd} transition-all duration-300 focus:border-gold-400/50 focus:shadow-[0_0_0_3px_rgba(212,175,55,0.1)] border border-surface-600/50 rounded-2xl py-3.5 px-4 bg-surface-900/50 text-white placeholder-surface-500 text-sm outline-none`}
                    required
                    minLength={8}
                    onFocus={() => setShowNew(true)}
                    onBlur={() => setShowNew(false)}
                    autoComplete="new-password"
                  />
                  <button
                    type="button"
                    onClick={() => setShowNew(!showNew)}
                    className={`absolute ${eyePos} top-1/2 -translate-y-1/2 text-surface-400 hover:text-gold-400 transition-colors p-1`}
                    aria-label={showNew ? t("auth.hidePassword") : t("auth.showPassword")}
                  >
                    {showNew ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                  </button>
                </div>
              </div>

              <div className="relative">
                <label className="input-label mb-1.5 block">{t("settings.confirmPassword")}</label>
                <div className="relative">
                  <input
                    type={showConfirm ? "text" : "password"}
                    value={confirmPassword}
                    onChange={(e) => setConfirmPassword(e.target.value)}
                    placeholder={t("settings.confirmPasswordPlaceholder")}
                    className={`w-full ${paddingStart} ${paddingEnd} transition-all duration-300 focus:border-gold-400/50 focus:shadow-[0_0_0_3px_rgba(212,175,55,0.1)] border border-surface-600/50 rounded-2xl py-3.5 px-4 bg-surface-900/50 text-white placeholder-surface-500 text-sm outline-none`}
                    required
                    onFocus={() => setShowConfirm(true)}
                    onBlur={() => setShowConfirm(false)}
                    autoComplete="new-password"
                  />
                  <button
                    type="button"
                    onClick={() => setShowConfirm(!showConfirm)}
                    className={`absolute ${eyePos} top-1/2 -translate-y-1/2 text-surface-400 hover:text-gold-400 transition-colors p-1`}
                    aria-label={showConfirm ? t("auth.hidePassword") : t("auth.showPassword")}
                  >
                    {showConfirm ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                  </button>
                </div>
              </div>

              <motion.button
                type="submit"
                disabled={loading}
                className="w-full mt-6 py-4 rounded-2xl font-bold text-pure-white text-base relative overflow-hidden disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-3"
                style={{
                  background: loading
                    ? "linear-gradient(to left, var(--brand-800), var(--brand-900))"
                    : "linear-gradient(to left, var(--brand-gold), var(--gold-600))",
                  boxShadow: loading
                    ? "0 4px 20px rgba(76,29,149,0.3)"
                    : "0 4px 20px rgba(212,175,55,0.3)",
                }}
                whileHover={!loading ? { scale: 1.01 } : undefined}
                whileTap={!loading ? { scale: 0.98 } : undefined}
              >
                {loading ? (
                  <>
                    <div className="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                    <span className="text-surface-200">{t("auth.changing")}</span>
                  </>
                ) : (
                  <>
                    <Lock className="w-5 h-5" />
                    <span>{t("settings.changePasswordBtn")}</span>
                    <ArrowLeft className="w-4 h-4" />
                  </>
                )}
              </motion.button>
            </div>
          )}
        </form>

        <motion.button
          className="w-full mt-6 text-center text-sm text-surface-500 hover:text-gold-400 transition-colors flex items-center justify-center gap-2"
          onClick={() => navigate("/settings")}
          whileHover={{ opacity: 0.8 }}
        >
          <ArrowLeft className="w-4 h-4" />
          {t("nav.settings")}
        </motion.button>
      </motion.div>
    </div>
  );
}