import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";
import i18n from "@/i18n";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatOMR(milli: number): string {
  return (milli / 1000).toFixed(3) + ' ' + i18n.t("currency.omrSuffix");
}

export function formatNumber(n: number): string {
  return new Intl.NumberFormat('en-US').format(n);
}

function dateLocale(): string {
  switch (i18n.language) {
    case 'ar': return 'ar-OM';
    case 'ur': return 'ur-PK';
    case 'hi': return 'en-IN';
    default: return 'en-GB';
  }
}

export function formatDate(d: string): string {
  if (!d) return '';
  return new Date(d).toLocaleDateString(dateLocale(), {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}

export function formatDateTime(d: string): string {
  if (!d) return '';
  return new Date(d).toLocaleString(dateLocale(), {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

export function getStatusColor(status: string): string {
  const s = status.toLowerCase();
  if (s === 'draft') return 'badge-info';
  if (s === 'posted' || s === 'active' || s === 'open' || s === 'approved' || s === 'completed' || s === 'paid') return 'badge-success';
  if (s === 'void' || s === 'cancelled' || s === 'closed' || s === 'rejected') return 'badge-danger';
  if (s === 'partial' || s === 'pending' || s === 'processing' || s === 'submitted') return 'badge-warning';
  return 'badge-info';
}

export function getSeverityColor(severity: string): string {
  const s = severity.toLowerCase();
  if (s === 'high' || s === 'critical' || s === 'عالي' || s === 'حرج') return 'badge-danger';
  if (s === 'medium' || s === 'متوسط') return 'badge-warning';
  return 'badge-success';
}

export function debounce<T extends (...args: any[]) => any>(fn: T, ms: number): T {
  let timer: ReturnType<typeof setTimeout>;
  return ((...args: any[]) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn(...args), ms);
  }) as T;
}
