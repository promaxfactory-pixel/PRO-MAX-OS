import { memo, useEffect } from "react";
import { useUIStore } from "@/stores/uiStore";
import { CheckCircle, XCircle, AlertTriangle, Info, X } from "lucide-react";

const icons = {
  success: CheckCircle,
  error: XCircle,
  warning: AlertTriangle,
  info: Info,
} as const;

const styles = {
  success: "border-emerald-500 bg-emerald-500/10 text-emerald-400",
  error: "border-red-500 bg-red-500/10 text-red-400",
  warning: "border-amber-500 bg-amber-500/10 text-amber-400",
  info: "border-brand-500 bg-brand-500/10 text-brand-400",
} as const;

const Toast = memo(function Toast() {
  const notifications = useUIStore((s) => s.notifications);
  const removeNotification = useUIStore((s) => s.removeNotification);

  useEffect(() => {
    const timers: ReturnType<typeof setTimeout>[] = [];
    for (const n of notifications) {
      if (!n.id) continue;
      const duration = n.duration ?? 4000;
      timers.push(setTimeout(() => removeNotification(n.id!), duration));
    }
    return () => timers.forEach(clearTimeout);
  }, [notifications, removeNotification]);

  return (
    <div className="fixed top-4 left-4 z-[100] flex flex-col gap-3 pointer-events-none">
      {notifications.map((n) => {
        const Icon = icons[n.type] ?? Info;
        const colorClass = styles[n.type] ?? styles.info;

        return (
          <div
            key={n.id ?? n.title}
            className={`pointer-events-auto flex items-start gap-3 min-w-[320px] max-w-[420px] rounded-xl border bg-surface-900 p-4 shadow-lg animate-slide-in-left ${colorClass}`}
          >
            <Icon className="mt-0.5 h-5 w-5 shrink-0" />
            <div className="flex-1 min-w-0">
              {n.title && (
                <p className="font-semibold text-sm text-white">{n.title}</p>
              )}
              <p className="text-sm text-surface-400 leading-relaxed">{n.message}</p>
            </div>
            <button
              onClick={() => n.id && removeNotification(n.id)}
              className="shrink-0 rounded-lg p-1 text-surface-400 hover:text-white hover:bg-surface-700 transition-colors"
            >
              <X className="h-4 w-4" />
            </button>
          </div>
        );
      })}
    </div>
  );
});

export default Toast;
