import { useState, useRef, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useUIStore, type WorkMode } from "@/stores/uiStore";
import { Zap, Anchor, Crosshair, Sparkles, Moon, Briefcase } from "lucide-react";

const modeIcons: Record<WorkMode, React.ElementType> = {
  power: Zap, stability: Anchor, focus: Crosshair,
  creative: Sparkles, night: Moon, professional: Briefcase,
};

export default function ModeSelector({ showLabel = false }: { showLabel?: boolean }) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  const { workMode, setWorkMode } = useUIStore();
  const { t } = useTranslation();

  useEffect(() => {
    const handleClick = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  }, []);

  const modeIds: WorkMode[] = ["power", "stability", "focus", "creative", "night", "professional"];
  const Icon = modeIcons[workMode];

  return (
    <div className="relative" ref={ref}>
      <button
        onClick={() => setOpen(!open)}
        className="btn-ghost flex items-center gap-2 px-3 py-1.5"
        title={t("mode." + workMode)}
      >
        <Icon className="w-4 h-4" style={{ color: "var(--brand-500)" }} />
        {showLabel && <span className="text-xs" style={{ color: "var(--text-muted)" }}>{t("mode." + workMode)}</span>}
      </button>

      {open && (
        <div
          className="absolute right-0 top-full mt-2 w-64 rounded-2xl overflow-hidden animate-scale-in z-50"
          style={{
            background: "var(--surface-card)",
            border: "1px solid var(--border)",
            boxShadow: "var(--shadow-modal)",
          }}
        >
          <div
            className="p-3 border-b"
            style={{ borderColor: "var(--border)" }}
          >
            <p className="text-xs font-bold uppercase tracking-wider" style={{ color: "var(--text-muted)" }}>{t("mode.title")}</p>
            <p className="text-[10px]" style={{ color: "var(--text-muted)", marginTop: "0.125rem" }}>{t("mode.subtitle")}</p>
          </div>
          <div className="p-1.5">
            {modeIds.map((id) => {
              const ModeIcon = modeIcons[id];
              const isActive = workMode === id;
              return (
                <button
                  key={id}
                  onClick={() => { setWorkMode(id); setOpen(false); }}
                  className="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm transition-all duration-200"
                  style={{
                    color: isActive ? "var(--brand-500)" : "var(--text-muted)",
                    background: isActive ? "color-mix(in srgb, var(--brand-500) 10%, transparent)" : "transparent",
                  }}
                  onMouseEnter={e => { if (!isActive) e.currentTarget.style.background = "color-mix(in srgb, var(--border) 40%, transparent)"; }}
                  onMouseLeave={e => { if (!isActive) e.currentTarget.style.background = "transparent"; }}
                >
                  <div
                    className="w-8 h-8 rounded-lg flex items-center justify-center"
                    style={{ background: "color-mix(in srgb, var(--brand-500) 10%, transparent)" }}
                  >
                    <ModeIcon className="w-4 h-4" style={{ color: "var(--brand-500)" }} />
                  </div>
                  <div className="text-right flex-1">
                    <p className="font-medium text-xs" style={{ color: isActive ? "var(--text-primary)" : "var(--text-primary)" }}>
                      {t("mode." + id)}
                    </p>
                    <p className="text-[10px]" style={{ color: "var(--text-muted)" }}>{t("mode." + id + "Desc")}</p>
                  </div>
                  {isActive && (
                    <div className="w-1.5 h-1.5 rounded-full" style={{ background: "var(--brand-500)" }} />
                  )}
                </button>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}

