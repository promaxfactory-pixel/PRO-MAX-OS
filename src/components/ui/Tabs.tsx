import { useState } from "react";
import { cn } from "@/lib/utils";

interface Tab {
  key: string;
  label: string;
  icon?: React.ReactNode;
  count?: number;
}

interface TabsProps {
  tabs: Tab[];
  activeKey?: string;
  onChange: (key: string) => void;
  className?: string;
}

export default function Tabs({ tabs, activeKey, onChange, className }: TabsProps) {
  // Use activeKey as the source of truth when provided; fall back to internal state
  const [internalActive, setInternalActive] = useState(tabs[0]?.key);
  const active = activeKey ?? internalActive;

  const handleChange = (key: string) => {
    if (!activeKey) setInternalActive(key);
    onChange(key);
  };

  return (
    <div
      className={cn("flex items-center gap-1 border-b", className)}
      style={{ borderColor: "var(--border)" }}
      role="tablist"
      aria-orientation="horizontal"
    >
      {tabs.map((tab) => (
        <button
          key={tab.key}
          role="tab"
          aria-selected={active === tab.key}
          aria-controls={`tabpanel-${tab.key}`}
          onClick={() => handleChange(tab.key)}
          className={cn(
            "flex items-center gap-2 px-4 py-3 text-sm font-medium border-b-2 transition-all duration-200",
          )}
          style={{
            color: active === tab.key ? "var(--brand-500)" : "var(--text-muted)",
            borderBottomColor: active === tab.key ? "var(--brand-500)" : "transparent",
          }}
        >
          {tab.icon}
          {tab.label}
          {tab.count !== undefined && (
            <span
              className={cn("text-[10px] px-1.5 py-0.5 rounded-full font-bold")}
              style={{
                background: active === tab.key ? "color-mix(in srgb, var(--brand-500) 15%, transparent)" : "color-mix(in srgb, var(--border) 50%, transparent)",
                color: active === tab.key ? "var(--brand-500)" : "var(--text-muted)",
              }}
            >
              {tab.count}
            </span>
          )}
        </button>
      ))}
    </div>
  );
}

