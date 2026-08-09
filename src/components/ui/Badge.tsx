import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";

interface BadgeProps {
  children: React.ReactNode;
  variant?: 'success' | 'warning' | 'danger' | 'info' | 'gold' | 'default';
  size?: 'sm' | 'md';
  className?: string;
}

export default function Badge({ children, variant = 'default', size = 'sm', className }: BadgeProps) {
  const variants = {
    success: 'badge-success',
    warning: 'badge-warning',
    danger: 'badge-danger',
    info: 'badge-info',
    gold: 'badge-gold',
    default: 'badge badge-default',
  };
  const sizes = {
    sm: 'text-xs px-2 py-0.5',
    md: 'text-xs px-3 py-1',
  };
  return (
    <span className={cn(variants[variant], sizes[size], className)}>
      {children}
    </span>
  );
}

export function StatusBadge({ status }: { status: string }) {
  const { t } = useTranslation();
  const s = status.toLowerCase();
  let variant: BadgeProps['variant'] = 'info';
  if (['posted', 'active', 'open', 'approved', 'completed', 'paid', 'reconciled'].includes(s)) variant = 'success';
  if (['draft', 'new', 'generated'].includes(s)) variant = 'info';
  if (['void', 'cancelled', 'closed', 'rejected', 'reversed'].includes(s)) variant = 'danger';
  if (['partial', 'pending', 'processing', 'submitted', 'ordered'].includes(s)) variant = 'warning';

  const labels: Record<string, string> = {
    draft: t('badge.draft'), posted: t('badge.posted'), active: t('common.active'),
    open: t('badge.open'), closed: t('badge.closed'), void: t('badge.void'),
    cancelled: t('badge.cancelled'), approved: t('badge.approved'),
    completed: t('badge.completed'), paid: t('common.paid'),
    pending: t('badge.pending'), processing: t('badge.processing'),
    submitted: t('badge.submitted'), partial: t('badge.partial'),
    rejected: t('badge.rejected'), ordered: t('badge.ordered'),
    generated: t('badge.generated'),
  };

  return <Badge variant={variant}>{labels[s] || status}</Badge>;
}
