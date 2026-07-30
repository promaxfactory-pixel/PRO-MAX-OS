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
  success: { border: '1px solid color-mix(in srgb, #10b981 40%, transparent)', background: 'color-mix(in srgb, #10b981 10%, transparent)', color: '#10b981' },
  error: { border: '1px solid color-mix(in srgb, #ef4444 40%, transparent)', background: 'color-mix(in srgb, #ef4444 10%, transparent)', color: '#ef4444' },
  warning: { border: '1px solid color-mix(in srgb, #f59e0b 40%, transparent)', background: 'color-mix(in srgb, #f59e0b 10%, transparent)', color: '#f59e0b' },
  info: { border: '1px solid color-mix(in srgb, #3b82f6 40%, transparent)', background: 'color-mix(in srgb, #3b82f6 10%, transparent)', color: '#3b82f6' },
} as const;

const typeLabels = {
  success: "نجاح",
  error: "خطأ",
  warning: "تحذير",
  info: "معلومة",
} as const;

const Toast = memo(function Toast() {
  const notifications = useUIStore((s) => s.notifications);
  const removeNotification = useUIStore((s) => s.removeNotification);

  useEffect(() => {
    const timers: ReturnType<typeof setTimeout>[] = [];
    for (const n of notifications) {
      if (!n.id) continue;
      const duration = n.duration ?? 4000;
      timers.push(setTimeout(() => removeNotification(n.id || ''), duration));
    }
    return () => timers.forEach(clearTimeout);
  }, [notifications, removeNotification]);

  return (
    <div className="fixed top-4 left-4 z-[100] flex flex-col gap-3 pointer-events-none" role="region" aria-label="الإشعارات" aria-live="polite" aria-relevant="additions removals">
      {notifications.map((n) => {
        const Icon = icons[n.type] ?? Info;
        const colorStyle = styles[n.type] ?? styles.info;
        const typeLabel = typeLabels[n.type] ?? "معلومة";

        return (
          <div
            key={n.id ?? n.title}
            role="status"
            aria-live="polite"
            className="pointer-events-auto flex items-start gap-3 min-w-[320px] max-w-[420px] rounded-xl p-4 shadow-lg animate-slide-in-left"
            style={{
              ...colorStyle,
              background: 'var(--surface-card)',
            }}
          >
            <Icon className="mt-0.5 h-5 w-5 shrink-0" style={{ color: colorStyle.color }} aria-hidden="true" />
            <div className="flex-1 min-w-0">
              <p className="sr-only">{typeLabel}:</p>
              {n.title && (
                <p className="font-semibold text-sm" style={{ color: 'var(--text-primary)' }}>{n.title}</p>
              )}
              <p className="text-sm leading-relaxed" style={{ color: 'var(--text-secondary)' }}>{n.message}</p>
            </div>
            <button
              onClick={() => n.id && removeNotification(n.id)}
              aria-label="إغلاق الإشعار"
              className="shrink-0 rounded-lg p-1 transition-colors"
              style={{ color: 'var(--text-muted)' }}
              onMouseEnter={e => e.currentTarget.style.background = 'color-mix(in srgb, var(--border) 40%, transparent)'}
              onMouseLeave={e => e.currentTarget.style.background = 'transparent'}
            >
              <X className="h-4 w-4" aria-hidden="true" />
            </button>
          </div>
        );
      })}
    </div>
  );
});

export default Toast;
