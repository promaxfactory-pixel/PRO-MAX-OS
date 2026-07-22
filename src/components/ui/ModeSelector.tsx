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
        className="flex items-center gap-2 px-3 py-1.5 rounded-xl hover:bg-surface-800/50 transition-all duration-200 border border-transparent hover:border-surface-700/50"
        title={t("mode." + workMode)}
      >
        <Icon className="w-4 h-4" style={{ color: `var(--mode-accent)` }} />
        {showLabel && <span className="text-xs text-surface-400">{t("mode." + workMode)}</span>}
      </button>

      {open && (
        <div className="absolute left-0 top-full mt-2 w-64 bg-surface-800 border border-surface-700 rounded-2xl shadow-luxury overflow-hidden animate-scale-in z-50">
          <div className="p-3 border-b border-surface-700/50">
            <p className="text-xs font-bold text-surface-400 uppercase tracking-wider">{t("mode.title")}</p>
            <p className="text-[10px] text-surface-500 mt-0.5">{t("mode.subtitle")}</p>
          </div>
          <div className="p-1.5">
            {modeIds.map((id) => {
              const ModeIcon = modeIcons[id];
              const isActive = workMode === id;
              return (
                <button
                  key={id}
                  onClick={() => { setWorkMode(id); setOpen(false); }}
                  className={`w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm transition-all duration-200 ${
                    isActive
                      ? "text-white"
                      : "text-surface-400 hover:text-white hover:bg-surface-700/50"
                  }`}
                  style={isActive ? { backgroundColor: `var(--mode-glow)`, color: `var(--mode-accent)` } : {}}
                >
                  <div
                    className="w-8 h-8 rounded-lg flex items-center justify-center"
                    style={{ backgroundColor: `var(--mode-glow)` }}
                  >
                    <ModeIcon className="w-4 h-4" style={{ color: `var(--mode-accent)` }} />
                  </div>
                  <div className="text-right flex-1">
                    <p className="font-medium text-xs">{t("mode." + id)}</p>
                    <p className="text-[10px] text-surface-500">{t("mode." + id + "Desc")}</p>
                  </div>
                  {isActive && (
                    <div className="w-1.5 h-1.5 rounded-full" style={{ backgroundColor: `var(--mode-accent)` }} />
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