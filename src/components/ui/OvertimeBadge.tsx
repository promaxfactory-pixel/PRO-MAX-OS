import { cn } from "@/lib/utils";

interface OvertimeBadgeProps {
  status: string;
}

const config: Record<string, { bg: string; text: string; label: string }> = {
  Pending: { bg: "bg-yellow-500/15", text: "text-yellow-400", label: "قيد المراجعة" },
  Approved: { bg: "bg-emerald-500/15", text: "text-emerald-400", label: "موافق عليه" },
  Rejected: { bg: "bg-red-500/15", text: "text-red-400", label: "مرفوض" },
  Processing: { bg: "bg-blue-500/15", text: "text-blue-400", label: "قيد التنفيذ" },
};

export default function OvertimeBadge({ status }: OvertimeBadgeProps) {
  const c = config[status] || config.Pending;
  return (
    <span className={cn("inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium", c.bg, c.text)}>
      {c.label}
    </span>
  );
}
