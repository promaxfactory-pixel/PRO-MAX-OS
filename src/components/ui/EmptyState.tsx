import Button from "./Button";

interface EmptyStateProps {
  icon?: React.ReactNode;
  title: string;
  description?: string;
  actionLabel?: string;
  onAction?: () => void;
}

export default function EmptyState({ icon, title, description, actionLabel, onAction }: EmptyStateProps) {
  return (
    <div className="empty-state animate-fade-in">
      {icon && <div className="empty-state-icon">{icon}</div>}
      <h3 className="text-lg font-bold text-[var(--text-primary)] mb-1 font-display">{title}</h3>
      {description && <p className="text-sm text-[var(--text-secondary)] max-w-sm leading-relaxed">{description}</p>}
      {actionLabel && onAction && (
        <Button onClick={onAction} className="mt-5">{actionLabel}</Button>
      )}
    </div>
  );
}
