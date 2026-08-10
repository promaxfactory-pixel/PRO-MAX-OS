import { ReactNode, useMemo, useState, useCallback } from "react";
import { cn } from "@/lib/utils";
import { ChevronDown, ChevronUp, ChevronsUpDown } from "lucide-react";

export type BadgeVariant = "success" | "warning" | "danger" | "info" | "gold" | "default";

export interface Column<T> {
  key: string;
  header: string;
  sortable?: boolean;
  align?: "left" | "center" | "right";
  width?: string;
  render?: (item: T, index: number) => ReactNode;
}

interface DataTableProps<T> {
  columns: Column<T>[];
  data: T[];
  loading?: boolean;
  onRowClick?: (item: T) => void;
  emptyMessage?: string;
  className?: string;
  compact?: boolean;
}

export default function DataTable<T>({
  columns, data, loading, onRowClick, emptyMessage = "لا توجد بيانات", className, compact = false
}: DataTableProps<T>) {
  const [sortKey, setSortKey] = useState<string | null>(null);
  const [sortDir, setSortDir] = useState<"asc" | "desc">("asc");

  const handleSort = useCallback((key: string) => {
    setSortKey(prev => {
      if (prev === key) {
        setSortDir(d => d === "asc" ? "desc" : "asc");
        return key;
      }
      setSortDir("asc");
      return key;
    });
  }, []);

  const sorted = useMemo(() => {
    return [...data].sort((a, b) => {
      if (!sortKey) return 0;
      const av = (a as Record<string, unknown>)[sortKey];
      const bv = (b as Record<string, unknown>)[sortKey];
      if (av === bv) return 0;
      if (av === null || av === undefined) return 1;
      if (bv === null || bv === undefined) return -1;
      const cmp = typeof av === "string" ? av.localeCompare(String(bv)) : Number(av) - Number(bv);
      return sortDir === "asc" ? cmp : -cmp;
    });
  }, [data, sortKey, sortDir]);

  const alignClass = useCallback((a?: string) => a === "left" ? "text-left" : a === "center" ? "text-center" : "text-right", []);

  return (
    <div className={cn("table-container", className)}>
      <div className="overflow-x-auto">
        <table className="data-table" role="grid" aria-label="جدول البيانات">
          <thead>
            <tr role="row">
              {columns.map((col) => {
                const isSorted = sortKey === col.key;
                const ariaSort = isSorted ? (sortDir === "asc" ? "ascending" : "descending") : "none" as const;
                return (
                  <th
                    key={col.key}
                    role="columnheader"
                    aria-sort={col.sortable ? ariaSort : undefined}
                    className={cn(alignClass(col.align), col.sortable && "sortable")}
                    style={{ width: col.width }}
                    onClick={() => col.sortable && handleSort(col.key)}
                    onKeyDown={(e) => { if (col.sortable && (e.key === "Enter" || e.key === " ")) { e.preventDefault(); handleSort(col.key); }}}
                    tabIndex={col.sortable ? 0 : undefined}
                  >
                    <span className="inline-flex items-center gap-1">
                      {col.header}
                      {col.sortable && (
                        isSorted
                          ? (sortDir === "asc" ? <ChevronUp className="w-3 h-3" aria-hidden="true" /> : <ChevronDown className="w-3 h-3" aria-hidden="true" />)
                          : <ChevronsUpDown className="w-3 h-3 opacity-30" aria-hidden="true" />
                      )}
                    </span>
                  </th>
                );
              })}
            </tr>
          </thead>
          <tbody>
            {sorted.length === 0 ? (
              <tr role="row">
                <td colSpan={columns.length} className="text-center py-12" style={{ color: "var(--text-muted)" }}>
                  {loading ? "جاري التحميل..." : emptyMessage}
                </td>
              </tr>
            ) : (
              sorted.map((item, i) => (
                <tr
                  key={i}
                  role="row"
                  onClick={() => onRowClick?.(item)}
                  onKeyDown={(e) => { if (onRowClick && (e.key === "Enter" || e.key === " ")) { e.preventDefault(); onRowClick(item); }}}
                  tabIndex={onRowClick ? 0 : undefined}
                  className={cn(onRowClick && "cursor-pointer")}
                >
                  {columns.map((col) => (
                    <td key={col.key} role="cell" className={cn(alignClass(col.align), compact && "py-2 text-xs")}>
                      {col.render ? col.render(item, i) : String((item as Record<string, unknown>)[col.key] ?? "")}
                    </td>
                  ))}
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
