import { useEffect, useRef, useCallback } from "react";
import { AlertTriangle, Shield, Info } from "lucide-react";
import Button from "@/components/ui/Button";

interface ConfirmDialogProps {
  open: boolean;
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  variant?: "danger" | "warning" | "info";
  onConfirm: () => void;
  onCancel: () => void;
  loading?: boolean;
}

const variantStyles = {
  danger: "bg-red-500 hover:bg-red-600 text-white",
  warning: "bg-amber-500 hover:bg-amber-600 text-white",
  info: "bg-brand-500 hover:bg-brand-600 text-white",
} as const;

const variantIcons = {
  danger: AlertTriangle,
  warning: Shield,
  info: Info,
} as const;

export default function ConfirmDialog({
  open,
  title,
  message,
  confirmLabel = "تأكيد",
  cancelLabel = "إلغاء",
  variant = "danger",
  onConfirm,
  onCancel,
  loading = false,
}: ConfirmDialogProps) {
  const overlayRef = useRef<HTMLDivElement>(null);
  const previousFocusRef = useRef<HTMLElement | null>(null);
  const confirmBtnRef = useRef<HTMLButtonElement>(null);
  const titleId = `confirm-title-${variant}`;
  const messageId = `confirm-message-${variant}`;

  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    if (e.key === "Escape") {
      onCancel();
      return;
    }
    if (e.key !== 'Tab') return;
    const focusable = overlayRef.current?.querySelectorAll<HTMLElement>(
      'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'
    );
    if (!focusable || focusable.length === 0) return;
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (e.shiftKey) {
      if (document.activeElement === first) {
        e.preventDefault();
        last.focus();
      }
    } else {
      if (document.activeElement === last) {
        e.preventDefault();
        first.focus();
      }
    }
  }, [onCancel]);

  useEffect(() => {
    if (!open) return;
    previousFocusRef.current = document.activeElement as HTMLElement;
    window.addEventListener("keydown", handleKeyDown);
    document.body.style.overflow = 'hidden';
    setTimeout(() => confirmBtnRef.current?.focus(), 0);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      document.body.style.overflow = '';
      previousFocusRef.current?.focus();
    };
  }, [open, handleKeyDown]);

  if (!open) return null;

  const Icon = variantIcons[variant];

  return (
    <div
      ref={overlayRef}
      role="alertdialog"
      aria-modal="true"
      aria-labelledby={titleId}
      aria-describedby={messageId}
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
      onClick={onCancel}
    >
      <div
        className="bg-surface-800 rounded-2xl border border-surface-700 p-6 max-w-md w-full mx-4 shadow-xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-start gap-4 mb-4">
          <div className={`shrink-0 rounded-xl p-3 ${variantStyles[variant]}`} aria-hidden="true">
            <Icon className="h-6 w-6" />
          </div>
          <div className="min-w-0">
            <h3 id={titleId} className="text-lg font-bold text-white">{title}</h3>
            <p id={messageId} className="mt-1 text-sm text-surface-400 leading-relaxed">{message}</p>
          </div>
        </div>

        <div className="flex items-center justify-end gap-3 mt-6">
          <Button
            variant="outline"
            onClick={onCancel}
            disabled={loading}
            aria-label={cancelLabel}
            className="border-surface-700 text-surface-400 hover:text-white hover:border-surface-500"
          >
            {cancelLabel}
          </Button>
          <button
            ref={confirmBtnRef}
            onClick={onConfirm}
            disabled={loading}
            aria-label={loading ? "جاري..." : confirmLabel}
            className={`rounded-xl px-5 py-2.5 text-sm font-semibold transition-colors disabled:opacity-50 ${variantStyles[variant]}`}
          >
            {loading ? "جاري..." : confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
