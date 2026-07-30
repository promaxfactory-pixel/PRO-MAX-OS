import { useState, useEffect, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { motion, AnimatePresence } from "framer-motion";
import { useTranslation } from "react-i18next";
import { useAuthStore } from "@/stores/authStore";
import { Factory, Eye, EyeOff, ArrowLeft, Shield, Zap } from "lucide-react";
import LanguageSwitcher from "@/components/layout/LanguageSwitcher";

function Particle({ index }: { index: number }) {
  const style = useMemo(() => ({
    left: `${Math.random() * 100}%`,
    top: `${Math.random() * 100}%`,
    width: `${2 + Math.random() * 4}px`,
    height: `${2 + Math.random() * 4}px`,
    animationDelay: `${Math.random() * 8}s`,
    animationDuration: `${6 + Math.random() * 10}s`,
  }), []);
  return (
    <div
      className="absolute rounded-full bg-gold-400/20 animate-pulse-slow"
      style={style}
      aria-hidden="true"
    />
  );
}

export default function LoginPage() {
  const { t, i18n } = useTranslation();
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [focusedField, setFocusedField] = useState<string | null>(null);
  const navigate = useNavigate();
  const { login, isAuthenticated } = useAuthStore();
  const isRtl = i18n.language === "ar" || i18n.language === "ur";

  useEffect(() => {
    if (isAuthenticated) {
      navigate("/", { replace: true });
    }
  }, [isAuthenticated, navigate]);

  const particles = useMemo(() => Array.from({ length: 30 }, (_, i) => i), []);

   const handleSubmit = async (e: React.FormEvent) => {
     e.preventDefault();
     setLoading(true);
     setError("");
     try {
       const result = await login(username, password);
       if (result?.user?.must_change_password) {
         navigate("/settings/change-password");
       } else {
         navigate("/");
       }
     } catch (err: unknown) {
       setError(err instanceof Error ? err.message : String(err) || t("auth.loginError"));
     } finally {
       setLoading(false);
     }
   };

  return (
    <div className="min-h-screen flex items-center justify-center relative overflow-hidden" data-theme="dark" style={{ background: 'var(--surface-950)' }}>
      <div className={`absolute top-6 ${isRtl ? 'left-6' : 'right-6'} z-30`}>
        <LanguageSwitcher />
      </div>

      <div className="absolute inset-0">
        <div className="absolute top-0 left-0 w-full h-full bg-gradient-to-br from-brand-950 via-surface-950 to-brand-950" />
        <div className="absolute top-1/3 right-1/4 w-[500px] h-[500px] bg-brand-800/15 rounded-full blur-[150px] animate-pulse-slow" />
        <div className="absolute bottom-1/3 left-1/4 w-[400px] h-[400px] bg-gold-400/5 rounded-full blur-[120px] animate-pulse-slow" style={{ animationDelay: '2s' }} />
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[600px] bg-brand-700/8 rounded-full blur-[200px] animate-pulse-slow" style={{ animationDelay: '4s' }} />
      </div>

      <div className="absolute inset-0 overflow-hidden">
        {particles.map((i) => (
          <Particle key={i} index={i} />
        ))}
      </div>

      <div className="absolute inset-0 opacity-[0.02]" style={{
        backgroundImage: `linear-gradient(rgba(212,175,55,0.3) 1px, transparent 1px), linear-gradient(90deg, rgba(212,175,55,0.3) 1px, transparent 1px)`,
        backgroundSize: '80px 80px'
      }} />

      <motion.div
        className="relative z-20 w-full max-w-md mx-auto px-6"
        initial={{ opacity: 0, y: 30 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.8, ease: [0.16, 1, 0.3, 1] }}
      >
        <motion.div
          className="text-center mb-12"
          initial={{ opacity: 0, scale: 0.8 }}
          animate={{ opacity: 1, scale: 1 }}
          transition={{ duration: 0.6, delay: 0.2, ease: [0.16, 1, 0.3, 1] }}
        >
          <motion.div
            className="w-24 h-24 mx-auto mb-8 relative"
            whileHover={{ scale: 1.05 }}
            transition={{ duration: 0.3 }}
          >
            <div className="absolute inset-0 rounded-3xl bg-gradient-to-br from-gold-400/20 via-brand-500/10 to-gold-400/20 blur-xl animate-pulse-slow" />
            <div className="relative w-full h-full rounded-3xl bg-gradient-to-br from-brand-700 via-brand-800 to-brand-900 flex items-center justify-center border border-gold-400/20 shadow-[0_0_40px_rgba(212,175,55,0.15)]">
              <Factory className="w-12 h-12 text-gold-400" />
            </div>
          </motion.div>

          <motion.h1
            className="text-4xl font-bold font-display text-white mb-3 tracking-tight"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 0.4 }}
          >
            PRO <span className="gradient-text">MAX</span> OS
          </motion.h1>
          <motion.p
            className="text-gold-400/80 font-medium text-base"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 0.5 }}
          >
            {t("app.tagline")}
          </motion.p>
        </motion.div>

        <motion.div
          className="relative"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.4, ease: [0.16, 1, 0.3, 1] }}
        >
          <div className="absolute -inset-1 bg-gradient-to-r from-brand-500/20 via-gold-400/10 to-brand-500/20 rounded-[2rem] blur-xl opacity-50" />

          <form
            onSubmit={handleSubmit}
            className="relative bg-surface-800/70 backdrop-blur-2xl border border-surface-600/30 rounded-[2rem] p-8 shadow-[0_25px_60px_-15px_rgba(0,0,0,0.5)]"
          >
            <div className="mb-8">
              <h2 className="text-2xl font-bold text-white mb-1">{t("auth.login")}</h2>
              <p className="text-sm text-surface-400">{t("auth.enterCredentials")}</p>
            </div>

            <div className="mb-4 p-3 bg-gold-400/10 border border-gold-400/20 rounded-2xl text-gold-400 text-xs text-center">
              {t("auth.firstTimeHint")}: <span className="font-bold">admin</span> / <span className="font-bold">{t("auth.checkConsoleForPassword")}</span>
            </div>

            <AnimatePresence>
              {error && (
                <motion.div
                  key="login-error"
                  className="mb-6 p-4 bg-red-500/10 border border-red-500/30 rounded-2xl text-red-400 text-sm flex items-center gap-3"
                  initial={{ opacity: 0, y: -10, scale: 0.95 }}
                  animate={{ opacity: 1, y: 0, scale: 1 }}
                  exit={{ opacity: 0, y: -10, scale: 0.95 }}
                  transition={{ type: "spring", stiffness: 300, damping: 25 }}
                >
                  <Shield className="w-5 h-5 flex-shrink-0" />
                  <span>{error}</span>
                </motion.div>
              )}
            </AnimatePresence>

            <div className="space-y-5">
              <div className="relative">
                <label className="input-label mb-1.5 block">{t("auth.username")}</label>
                <input
                  type="text"
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                  placeholder={t("auth.usernamePlaceholder")}
                  className={`w-full transition-all duration-300 ${focusedField === 'username' ? 'border-gold-400/50 shadow-[0_0_0_3px_rgba(212,175,55,0.1)]' : ''}`}
                  required
                  autoFocus
                  onFocus={() => setFocusedField('username')}
                  onBlur={() => setFocusedField(null)}
                  aria-label={t("auth.username")}
                />
              </div>

              <div className="relative">
                <label className="input-label mb-1.5 block">{t("auth.password")}</label>
                <div className="relative">
                  <input
                    type={showPassword ? "text" : "password"}
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    placeholder={t("auth.passwordPlaceholder")}
                    className={`w-full pr-12 transition-all duration-300 ${focusedField === 'password' ? 'border-gold-400/50 shadow-[0_0_0_3px_rgba(212,175,55,0.1)]' : ''}`}
                    required
                    onFocus={() => setFocusedField('password')}
                    onBlur={() => setFocusedField(null)}
                    aria-label={t("auth.password")}
                  />
                  <button
                    type="button"
                    onClick={() => setShowPassword(!showPassword)}
                    className="absolute right-3 top-1/2 -translate-y-1/2 text-surface-400 hover:text-gold-400 transition-colors p-1"
                    aria-label={showPassword ? t("auth.hidePassword") : t("auth.showPassword")}
                  >
                    {showPassword ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                  </button>
                </div>
              </div>
            </div>

            <motion.button
              type="submit"
              disabled={loading || !username || !password}
              className="w-full mt-8 py-4 rounded-2xl font-bold text-pure-white text-base relative overflow-hidden disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-3"
              style={{
                background: loading
                  ? 'linear-gradient(to left, #4c1d95, #312e81)'
                  : 'linear-gradient(to left, #d4af37, #b8860b)',
                boxShadow: loading
                  ? '0 4px 20px rgba(76,29,149,0.3)'
                  : '0 4px 20px rgba(212,175,55,0.3)',
              }}
              whileHover={!loading ? { scale: 1.01, boxShadow: '0 6px 30px rgba(212,175,55,0.4)' } : undefined}
              whileTap={!loading ? { scale: 0.98 } : undefined}
            >
              {loading ? (
                <>
                  <div className="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                  <span className="text-surface-200">{t("auth.verifying")}</span>
                  <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/5 to-transparent animate-slide-in-left" />
                </>
              ) : (
                <>
                  <Zap className="w-5 h-5" />
                  <span>{t("auth.loginBtn")}</span>
                  <ArrowLeft className="w-4 h-4" />
                </>
              )}
            </motion.button>

            <div className="mt-8 pt-6 border-t border-surface-700/50">
              <div className="flex items-center justify-between">
                <motion.p
                  className="text-xs text-surface-500"
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  transition={{ delay: 0.8 }}
                >
                  PRO MAX OS v2.1.0
                </motion.p>
                <motion.div
                  className="flex items-center gap-1.5"
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  transition={{ delay: 0.9 }}
                >
                  <div className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
                  <span className="text-[10px] text-surface-500 font-medium">{t("auth.secureSystem")}</span>
                </motion.div>
              </div>
            </div>
          </form>
        </motion.div>

        <motion.p
          className="text-center text-[11px] text-surface-600 mt-8"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 1 }}
        >
          &copy; {new Date().getFullYear()} Mayadeen Bahla National Company. All rights reserved.
        </motion.p>
      </motion.div>
    </div>
  );
}
