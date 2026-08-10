import { useState, useEffect, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import DataTable, { Column, type BadgeVariant } from "@/components/ui/DataTable";
import Badge from "@/components/ui/Badge";
import Button from "@/components/ui/Button";
import { formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import type { MaintenanceSheet } from "@/types";

export default function MaintenanceSheetListPage() {
  const navigate = useNavigate();
  const { addNotification } = useUIStore();
  const [sheets, setSheets] = useState<MaintenanceSheet[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    invoke("list_maintenance_sheets")
      .then((d) => setSheets(d as MaintenanceSheet[]))
      .catch((e: unknown) => addNotification({ title: "خطأ", message: String(e), type: "error" }))
      .finally(() => setLoading(false));
  }, []);

  const severityMap: Record<string, { label: string; variant: BadgeVariant }> = {
    critical: { label: "حرج", variant: "danger" },
    high: { label: "مرتفع", variant: "warning" },
    medium: { label: "متوسط", variant: "info" },
    low: { label: "منخفض", variant: "success" },
  };

  const statusMap: Record<string, { label: string; variant: BadgeVariant }> = {
    open: { label: "مفتوح", variant: "warning" },
    in_progress: { label: "قيد التنفيذ", variant: "info" },
    completed: { label: "مكتمل", variant: "success" },
    cancelled: { label: "ملغي", variant: "danger" },
  };

  const columns: Column<MaintenanceSheet>[] = useMemo(() => [
    { key: "ticket_no", header: "التذكرة", sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.ticket_no || "—"}</span> },
    { key: "date", header: "التاريخ", sortable: true, render: (r) => formatDate(r.date) },
    { key: "equipment", header: "المعدة", sortable: true, render: (r) => <span className="font-medium">{r.equipment_name || "—"}</span> },
    { key: "fault", header: "العطل", sortable: true, render: (r) => <span className="line-clamp-1">{r.fault_description || "—"}</span> },
    { key: "severity", header: "الخطورة", render: (r) => { const s = severityMap[r.severity] || { label: r.severity, variant: "default" as BadgeVariant }; return <Badge variant={s.variant}>{s.label}</Badge>; } },
    { key: "downtime_hours", header: "وقت التوقف", align: "left", render: (r) => r.downtime_hours ? `${r.downtime_hours} س` : "—" },
    { key: "status", header: "الحالة", render: (r) => { const s = statusMap[r.status] || { label: r.status, variant: "default" as BadgeVariant }; return <Badge variant={s.variant}>{s.label}</Badge>; } },
    { key: "assigned_to", header: "المسؤول", render: (r) => r.assigned_to || "—" },
  ], []);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">أوراق الصيانة</h1>
          <p className="page-subtitle">{sheets.length} تذكرة صيانة</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => navigate("/maintenance/new")}>
          تذكرة جديدة
        </Button>
      </div>
      <DataTable
        columns={columns}
        data={sheets}
        loading={loading}
        onRowClick={(r) => navigate(`/maintenance/sheets/${r.id}`)}
        emptyMessage="لا توجد تذاكر صيانة"
      />
    </div>
  );
}
