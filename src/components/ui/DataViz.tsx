import { MiniChart } from "./MiniChart";
import { cn } from "@/lib/utils";

export interface StatCardProps {
  title: string;
  value: string | number;
  change?: number;
  changeLabel?: string;
  trend?: "up" | "down" | "neutral";
  icon?: React.ReactNode;
  chartData?: Array<Record<string, any>>;
  chartXKey?: string;
  chartYKeys?: string[];
  chartType?: "line" | "area" | "bar";
  className?: string;
  onClick?: () => void;
}

export function StatCard({
  title,
  value,
  change,
  changeLabel,
  trend = "neutral",
  icon,
  chartData,
  chartXKey = "date",
  chartYKeys = ["value"],
  chartType = "line",
  className,
  onClick,
}: StatCardProps) {
  const trendColors = {
    up: "text-emerald-400",
    down: "text-rose-400",
    neutral: "text-surface-500",
  } as const;

  return (
    <article
      className={cn(
        "stat-card p-6 group",
        onClick && "cursor-pointer hover:shadow-brand-500/10",
        className
      )}
      onClick={onClick}
    >
      <div className="flex items-start justify-between mb-4">
        <div className="flex-1 min-w-0">
          <p className="kpi-label">{title}</p>
          <p className="text-3xl font-bold text-white mt-1 tabular-nums">{value}</p>
        </div>
        {icon && (
          <div className="flex-shrink-0 w-12 h-12 rounded-xl bg-brand-500/15 flex items-center justify-center text-brand-400">
            {icon}
          </div>
        )}
      </div>

      {chartData && chartData.length > 0 && chartYKeys && chartYKeys.length > 0 && (
        <MiniChart
          data={chartData}
          xKey={chartXKey}
          yKeys={chartYKeys}
          type={chartType}
          height={80}
        />
      )}

      {change !== undefined && (
        <div className="mt-4 flex items-center gap-2">
          <span className={["text-sm font-semibold flex items-center gap-1", trendColors[trend]].join(" ")}>
            <span>{trend === "up" ? "↑" : trend === "down" ? "↓" : "→"}</span>
            <span className="tabular-nums">{Math.abs(change)}%</span>
          </span>
          {changeLabel && <span className="text-xs text-surface-500">{changeLabel}</span>}
        </div>
      )}
    </article>
  );
}

export interface KPIWidgetProps {
  label: string;
  value: string | number;
  subValue?: string;
  icon?: React.ReactNode;
  trend?: "up" | "down" | "neutral";
  trendValue?: number;
  className?: string;
}

export function KPIWidget({
  label,
  value,
  subValue,
  icon,
  trend,
  trendValue,
  className,
}: KPIWidgetProps) {
  return (
    <div className={["card p-4 flex items-center gap-4", className].filter(Boolean).join(" ")}>
      {icon && (
        <div className="flex-shrink-0 w-12 h-12 rounded-xl bg-brand-500/15 flex items-center justify-center text-brand-400">
          {icon}
        </div>
      )}
      <div className="flex-1 min-w-0">
        <p className="text-xs text-surface-500 font-medium">{label}</p>
        <p className="text-xl font-bold text-white tabular-nums mt-0.5">{value}</p>
        {subValue && <p className="text-xs text-surface-500 mt-1">{subValue}</p>}
        {trend && trendValue !== undefined && (
          <div className="mt-2 flex items-center gap-1">
            <span className={["text-xs font-semibold", trend === "up" ? "text-emerald-400" : "text-rose-400"].join(" ")}>
              {trend === "up" ? "↑" : "↓"} {Math.abs(trendValue)}%
            </span>
            <span className="text-xs text-surface-500">vs الشهر الماضي</span>
          </div>
        )}
      </div>
    </div>
  );
}

export interface ProgressRingProps {
  value: number;
  max?: number;
  size?: number;
  strokeWidth?: number;
  showLabel?: boolean;
  label?: string;
  className?: string;
}

export function ProgressRing({
  value,
  max = 100,
  size = 64,
  strokeWidth = 6,
  showLabel = true,
  label,
  className,
}: ProgressRingProps) {
  const percentage = Math.min(Math.max(value / max, 0), 1);
  const radius = (size - strokeWidth) / 2;
  const circumference = 2 * Math.PI * radius;
  const strokeDashoffset = circumference * (1 - percentage);

  return (
    <div className={["flex flex-col items-center gap-3", className].filter(Boolean).join(" ")}>
      <div className="relative" style={{ width: size, height: size }}>
        <svg width={size} height={size} style={{ transform: "rotate(-90deg)" }}>
          <circle
            cx={size / 2}
            cy={size / 2}
            r={radius}
            fill="none"
            stroke="var(--border)"
            strokeWidth={strokeWidth}
          />
          <circle
            cx={size / 2}
            cy={size / 2}
            r={radius}
            fill="none"
            stroke="var(--mode-accent)"
            strokeWidth={strokeWidth}
            strokeLinecap="round"
            strokeDasharray={circumference}
            strokeDashoffset={strokeDashoffset}
            className="animate-draw-border"
            style={{ filter: "drop-shadow(0 0 8px var(--mode-glow))" }}
          />
        </svg>
        {(showLabel || label) && (
          <div className="absolute inset-0 flex flex-col items-center justify-center">
            {showLabel && (
              <span className="text-2xl font-bold text-white tabular-nums">
                {Math.round(percentage * 100)}%
              </span>
            )}
            {label && (
              <span className="text-xs text-surface-500 mt-1">{label}</span>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

export interface ChartCardProps {
  title: string;
  subtitle?: string;
  children: React.ReactNode;
  action?: React.ReactNode;
  className?: string;
}

export function ChartCard({ title, subtitle, children, action, className }: ChartCardProps) {
  return (
    <div className={["card p-6", className].filter(Boolean).join(" ")}>
      <div className="flex items-center justify-between mb-6">
        <div>
          <h3 className="page-title">{title}</h3>
          {subtitle && <p className="page-subtitle mt-1">{subtitle}</p>}
        </div>
        {action && <div>{action}</div>}
      </div>
      <div className="h-72">{children}</div>
    </div>
  );
}