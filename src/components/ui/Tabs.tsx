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
  const [active, setActive] = useState(activeKey || tabs[0]?.key);

  const handleChange = (key: string) => {
    setActive(key);
    onChange(key);
  };

  return (
    <div className={cn('flex items-center gap-1 border-b border-surface-700/50', className)}>
      {tabs.map((tab) => (
        <button
          key={tab.key}
          onClick={() => handleChange(tab.key)}
          className={cn(
            'flex items-center gap-2 px-4 py-3 text-sm font-medium border-b-2 transition-all duration-200',
            active === tab.key
              ? 'text-gold-400 border-gold-400'
              : 'text-surface-400 border-transparent hover:text-white hover:border-surface-500'
          )}
        >
          {tab.icon}
          {tab.label}
          {tab.count !== undefined && (
            <span className={cn(
              'text-[10px] px-1.5 py-0.5 rounded-full font-bold',
              active === tab.key ? 'bg-gold-400/20 text-gold-400' : 'bg-surface-700 text-surface-400'
            )}>
              {tab.count}
            </span>
          )}
        </button>
      ))}
    </div>
  );
}
