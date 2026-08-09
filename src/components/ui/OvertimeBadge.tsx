import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";

interface OvertimeBadgeProps {
  status: string;
}

const config: Record<string, { bg: string; text: string }> = {
  Pending: { bg: "bg-yellow-500/15", text: "text-yellow-400" },
  Approved: { bg: "bg-emerald-500/15", text: "text-emerald-400" },
  Rejected: { bg: "bg-red-500/15", text: "text-red-400" },
  Processing: { bg: "bg-blue-500/15", text: "text-blue-400" },
};

const labels: Record<string, string> = {
  Pending: "overtime.pending",
  Approved: "overtime.approved",
  Rejected: "overtime.rejected",
  Processing: "overtime.processing",
};

export default function OvertimeBadge({ status }: OvertimeBadgeProps) {
  const { t } = useTranslation();
  const c = config[status] || config.Pending;
  const labelKey = labels[status] || labels.Pending;
  return (
    <span className={cn("inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium", c.bg, c.text)}>
      {t(labelKey)}
    </span>
  );
}
