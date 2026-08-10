import { memo } from "react";
import { cn } from "@/lib/utils";

interface CardProps {
  children: React.ReactNode;
  className?: string;
  hover?: boolean;
  padding?: "none" | "sm" | "md" | "lg";
  onClick?: () => void;
}

const Card = memo(function Card({ children, className, hover = false, padding = "md", onClick }: CardProps) {
  const paddings = { none: "", sm: "p-4", md: "p-6", lg: "p-8" };
  return (
    <div
      onClick={onClick}
      className={cn(
        hover ? "card-hover" : "card",
        paddings[padding],
        onClick && "cursor-pointer",
        className
      )}
    >
      {children}
    </div>
  );
});

export { Card };
export default Card;

export function StatCard({ title, value, subtitle, icon, trend, trendValue, className }: {
  title: string;
  value: string | number;
  subtitle?: string;
  icon?: React.ReactNode;
  trend?: "up" | "down" | "neutral";
  trendValue?: string;
  className?: string;
}) {
  return (
    <div className={cn("stat-card", className)}>
      <div className="flex items-start justify-between">
        <div>
          <p className="text-sm text-surface-400 font-medium">{title}</p>
          <p className="text-2xl font-bold text-white mt-1">{value}</p>
          {subtitle && <p className="text-xs text-surface-400 mt-1">{subtitle}</p>}
          {trend && trendValue && (
            <div className={`flex items-center gap-1 mt-2 text-xs font-medium ${trend === "up" ? "text-emerald-400" : trend === "down" ? "text-red-400" : "text-surface-400"}`}>
              <span>{trend === "up" ? "↑" : trend === "down" ? "↓" : "→"}</span>
              <span>{trendValue}</span>
            </div>
          )}
        </div>
        {icon && (
          <div className="w-12 h-12 rounded-xl bg-brand-800/30 border border-brand-500/20 flex items-center justify-center text-brand-400">
            {icon}
          </div>
        )}
      </div>
    </div>
  );
}
