import { memo, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { useTranslation } from "react-i18next";
import { useUIStore } from "@/stores/uiStore";
import { CheckCircle, XCircle, AlertTriangle, Info, X, Loader2 } from "lucide-react";
import { cn } from "@/lib/utils";

const icons = {
  success: CheckCircle,
  error: XCircle,
  warning: AlertTriangle,
  info: Info,
  loading: Loader2,
} as const;

const styles = {
  success: "border-emerald-500/50 bg-emerald-500/10 text-emerald-400 shadow-emerald-500/10",
  error: "border-rose-500/50 bg-rose-500/10 text-rose-400 shadow-rose-500/10",
  warning: "border-amber-500/50 bg-amber-500/10 text-amber-400 shadow-amber-500/10",
  info: "border-brand-500/50 bg-brand-500/10 text-brand-400 shadow-brand-500/10",
  loading: "border-brand-500/50 bg-brand-500/10 text-brand-400 shadow-brand-500/10",
} as const;

const typeLabels = {
  success: "success",
  error: "error",
  warning: "warning",
  info: "info",
  loading: "loading",
} as const;

interface ToastProps {
  id: string;
  title?: string;
  message: string;
  type: "success" | "error" | "warning" | "info" | "loading";
  duration?: number;
  action?: { label: string; onClick: () => void; id?: string };
  onClose: (id: string) => void;
}

const ToastItem = memo(function ToastItem({ id, title, message, type, duration = 5000, action, onClose }: ToastProps) {
  const { t } = useTranslation();
  const Icon = icons[type] ?? Info;
  const colorClass = styles[type] ?? styles.info;
  const typeLabel = typeLabels[type] ?? "info";

  useEffect(() => {
    if (type === "loading" || duration <= 0) return;
    const timer = setTimeout(() => onClose(id), duration);
    return () => clearTimeout(timer);
  }, [id, duration, onClose, type]);

  return (
    <motion.div
      initial={{ opacity: 0, x: 300, scale: 0.95, rotate: 2 }}
      animate={{ opacity: 1, x: 0, scale: 1, rotate: 0 }}
      exit={{ opacity: 0, x: 300, scale: 0.95, rotate: -2 }}
      transition={{ type: "spring", damping: 25, stiffness: 300 }}
      className={cn(
        "pointer-events-auto flex items-start gap-3 min-w-[320px] max-w-[420px] rounded-2xl border backdrop-blur-xl p-4 shadow-2xl",
        colorClass
      )}
      role="status"
      aria-live={type === "error" ? "assertive" : "polite"}
      aria-label={t(`common.${typeLabel}`)}
    >
      <motion.div
        initial={{ scale: 0, rotate: -180 }}
        animate={{ scale: 1, rotate: 0 }}
        transition={{ type: "spring", damping: 15, stiffness: 200, delay: 0.1 }}
        className="flex-shrink-0 w-8 h-8 rounded-xl flex items-center justify-center"
        style={{ background: `color-mix(in srgb, currentColor 15%, transparent)` }}
      >
        {type === "loading" ? (
          <motion.svg
            animate={{ rotate: 360 }}
            transition={{ duration: 1, repeat: Infinity, ease: "linear" }}
            className="w-5 h-5"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
          >
            <circle cx="12" cy="12" r="10" strokeWidth="3" strokeLinecap="round" strokeDasharray="30 60" />
          </motion.svg>
        ) : (
          <Icon className="w-5 h-5" strokeWidth={2.5} />
        )}
      </motion.div>

      <div className="flex-1 min-w-0">
        <p className="sr-only">{t(`common.${typeLabel}`)}:</p>
        {title && (
          <motion.p
            initial={{ opacity: 0, y: 5 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.15 }}
            className="font-semibold text-sm text-white"
          >
            {title}
          </motion.p>
        )}
        <motion.p
          initial={{ opacity: 0, y: 5 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.2 }}
          className="text-sm text-surface-300 leading-relaxed mt-1"
        >
          {message}
        </motion.p>

        {action && (
          <motion.button
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.3 }}
            onClick={() => { action.onClick(); onClose(action.id || id); }}
            className="mt-3 px-3 py-1.5 text-xs font-semibold rounded-lg bg-white/10 hover:bg-white/20 text-white transition-colors"
          >
            {action.label}
          </motion.button>
        )}
      </div>

      <motion.button
        initial={{ opacity: 0, scale: 0.8 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ delay: 0.2 }}
        whileHover={{ scale: 1.1 }}
        whileTap={{ scale: 0.9 }}
        onClick={() => onClose(id)}
        aria-label={t("common.closeNotification") || "إغلاق"}
        className="shrink-0 ml-2 p-1.5 rounded-lg text-surface-500 hover:text-white hover:bg-white/10 transition-colors"
      >
        <X className="w-4 h-4" />
      </motion.button>

      <motion.div
        initial={{ scaleX: 1 }}
        animate={{ scaleX: 0 }}
        transition={{ duration: duration / 1000, ease: "linear" }}
        className="absolute bottom-0 left-0 right-0 h-1 rounded-b-2xl"
        style={{ background: "currentColor", transformOrigin: "left" }}
      />
    </motion.div>
  );
});

const Toast = memo(function Toast() {
  const { t } = useTranslation();
  const notifications = useUIStore((s) => s.notifications);
  const removeNotification = useUIStore((s) => s.removeNotification);

  return (
    <AnimatePresence>
      <div className="fixed top-4 right-4 z-[100] flex flex-col gap-3 pointer-events-none w-[380px] max-w-[90vw]" role="region" aria-label={t("common.notifications")} aria-live="polite" aria-relevant="additions removals">
        {notifications.map((n) => (
          <ToastItem
            key={n.id ?? n.title}
            id={n.id ?? n.title}
            title={n.title}
            message={n.message}
            type={n.type}
            duration={n.duration ?? 5000}
            action={n.action ? { label: n.action.label, onClick: n.action.onClick, id: n.id } : undefined}
            onClose={removeNotification}
          />
        ))}
      </div>
    </AnimatePresence>
  );
});

export default Toast;