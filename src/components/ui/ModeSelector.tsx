import { useState, useRef, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useUIStore, type WorkMode, type Density, type Motion } from "@/stores/uiStore";
import {
  Layers, Zap, Anchor, Crosshair, Sparkles, Moon, Briefcase,
  ChevronDown, Ruler, Gauge, Move, Wind,
} from "lucide-react";
import { cn } from "@/lib/utils";

const modeIcons: Record<WorkMode, React.ElementType> = {
  default: Layers,
  power: Zap,
  stability: Anchor,
  focus: Crosshair,
  creative: Sparkles,
  night: Moon,
  professional: Briefcase,
};

const modeOrder: WorkMode[] = ["default", "professional", "power", "stability", "focus", "creative", "night"];

const modeAccent: Record<WorkMode, string> = {
  default: "#8b5cf6",
  professional: "#3b82f6",
  power: "#ef4444",
  stability: "#3b82f6",
  focus: "#14b8a6",
  creative: "#a78bfa",
  night: "#818cf8",
};

export default function ModeSelector({ showLabel = false }: { showLabel?: boolean }) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  const { workMode, setWorkMode, density, setDensity, motion, setMotion } = useUIStore();
  const { t } = useTranslation();

  useEffect(() => {
    const handleClick = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  }, []);

  const Icon = modeIcons[workMode] ?? Layers;

  const densityOptions: { id: Density; label: string }[] = [
    { id: "comfortable", label: t("appearance.densityComfortable") || "مريح" },
    { id: "compact", label: t("appearance.densityCompact") || "مضغوط" },
  ];

  const motionOptions: { id: Motion; label: string }[] = [
    { id: "full", label: t("appearance.motionFull") || "كامل" },
    { id: "reduced", label: t("appearance.motionReduced") || "مخفّض" },
  ];

  return (
    <div className="relative" ref={ref}>
      <button
        onClick={() => setOpen(!open)}
        className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-xl hover:bg-surface-800/50 transition-all duration-200 border border-transparent hover:border-surface-700/50"
        title={t("appearance.title") || "المظهر"}
        aria-haspopup="true"
        aria-expanded={open}
      >
        <Icon className="w-4 h-4" style={{ color: `var(--mode-accent)` }} />
        {showLabel && <span className="text-xs text-surface-400">{t("mode." + workMode)}</span>}
        <ChevronDown className={cn("w-3 h-3 text-surface-500 transition-transform duration-200", open && "rotate-180")} />
      </button>

      {open && (
        <div className="absolute left-0 top-full mt-2 w-80 bg-surface-800 border border-surface-700 rounded-2xl shadow-luxury overflow-hidden animate-scale-in z-50">
          <div className="p-4 border-b border-surface-700/50 flex items-center gap-2">
            <div className="w-8 h-8 rounded-xl flex items-center justify-center" style={{ background: `var(--mode-glow)` }}>
              <Icon className="w-4 h-4" style={{ color: `var(--mode-accent)` }} />
            </div>
            <div>
              <p className="text-xs font-bold text-surface-300">{t("appearance.title") || "المظهر"}</p>
              <p className="text-[10px] text-surface-500 mt-0.5">{t("mode.subtitle")}</p>
            </div>
          </div>

          <div className="p-3">
            <p className="text-[10px] font-bold text-surface-500 uppercase tracking-wider mb-2 px-1">
              {t("appearance.modes") || "الأوضاع"}
            </p>
            <div className="grid grid-cols-2 gap-1.5">
              {modeOrder.map((id) => {
                const isActive = workMode === id;
                return (
                  <button
                    key={id}
                    onClick={() => setWorkMode(id)}
                    className={cn(
                      "flex items-center gap-2 px-2.5 py-2 rounded-xl text-xs font-medium transition-all duration-200 border",
                      isActive
                        ? "text-white border-transparent"
                        : "text-surface-400 hover:text-white hover:bg-surface-700/40 border-transparent hover:border-surface-600/40"
                    )}
                    style={isActive ? { backgroundColor: `var(--mode-glow)`, boxShadow: `0 0 0 1px var(--mode-accent)` } : {}}
                    aria-pressed={isActive}
                  >
                    <span
                      className="flex-shrink-0 w-4 h-4 rounded-full"
                      style={{ background: modeAccent[id], boxShadow: `0 0 8px ${modeAccent[id]}66` }}
                      aria-hidden="true"
                    />
                    <span className="truncate">{t("mode." + id)}</span>
                  </button>
                );
              })}
            </div>
          </div>

          <div className="px-3 pb-1 space-y-2.5">
            <div className="flex items-center justify-between gap-2">
              <div className="flex items-center gap-1.5 text-[11px] text-surface-400">
                <Ruler className="w-3.5 h-3.5" aria-hidden="true" />
                <span>{t("appearance.density") || "الكثافة"}</span>
              </div>
              <div className="flex rounded-lg overflow-hidden border border-surface-600/60">
                {densityOptions.map((opt) => (
                  <button
                    key={opt.id}
                    onClick={() => setDensity(opt.id)}
                    className={cn(
                      "px-3 py-1 text-[11px] font-medium transition-colors duration-150",
                      density === opt.id ? "text-white" : "text-surface-500 hover:text-surface-300"
                    )}
                    style={density === opt.id ? { backgroundColor: `var(--mode-glow)` } : {}}
                    aria-pressed={density === opt.id}
                  >
                    {opt.label}
                  </button>
                ))}
              </div>
            </div>

            <div className="flex items-center justify-between gap-2">
              <div className="flex items-center gap-1.5 text-[11px] text-surface-400">
                <Wind className="w-3.5 h-3.5" aria-hidden="true" />
                <span>{t("appearance.motion") || "الحركات"}</span>
              </div>
              <div className="flex rounded-lg overflow-hidden border border-surface-600/60">
                {motionOptions.map((opt) => (
                  <button
                    key={opt.id}
                    onClick={() => setMotion(opt.id)}
                    className={cn(
                      "px-3 py-1 text-[11px] font-medium transition-colors duration-150",
                      motion === opt.id ? "text-white" : "text-surface-500 hover:text-surface-300"
                    )}
                    style={motion === opt.id ? { backgroundColor: `var(--mode-glow)` } : {}}
                    aria-pressed={motion === opt.id}
                  >
                    {opt.label}
                  </button>
                ))}
              </div>
            </div>
          </div>

          <div className="p-3 flex items-center justify-between border-t border-surface-700/50 mt-2">
            <span className="text-[10px] text-surface-500 flex items-center gap-1">
              <Gauge className="w-3 h-3" aria-hidden="true" />
              {t("mode.subtitle")}
            </span>
            <span className="text-[10px] text-surface-600 font-mono flex items-center gap-1">
              <Move className="w-3 h-3" aria-hidden="true" />
              {t("appearance.livePreview") || "تطبيق فوري"}
            </span>
          </div>
        </div>
      )}
    </div>
  );
}
