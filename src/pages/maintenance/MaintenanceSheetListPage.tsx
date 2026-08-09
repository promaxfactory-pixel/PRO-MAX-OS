import { useState, useEffect, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import DataTable, { Column, type BadgeVariant } from "@/components/ui/DataTable";
import Badge from "@/components/ui/Badge";
import Button from "@/components/ui/Button";
import { formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import type { MaintenanceSheet } from "@/types";

export default function MaintenanceSheetListPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { addNotification } = useUIStore();
  const [sheets, setSheets] = useState<MaintenanceSheet[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    invoke("list_maintenance_sheets")
      .then((d) => setSheets(d as MaintenanceSheet[]))
      .catch((e: unknown) => addNotification({ title: t('common.error'), message: String(e), type: 'error' }))
      .finally(() => setLoading(false));
  }, [t]);

  const severityMap: Record<string, { label: string; variant: BadgeVariant }> = {
    critical: { label: t("maintenance.severity.critical"), variant: "danger" },
    high: { label: t("maintenance.severity.high"), variant: "warning" },
    medium: { label: t("maintenance.severity.medium"), variant: "info" },
    low: { label: t("maintenance.severity.low"), variant: "success" },
  };

  const statusMap: Record<string, { label: string; variant: BadgeVariant }> = {
    open: { label: t("maintenance.status.open"), variant: "warning" },
    in_progress: { label: t("maintenance.status.in_progress"), variant: "info" },
    completed: { label: t("maintenance.status.completed"), variant: "success" },
    cancelled: { label: t("maintenance.status.cancelled"), variant: "danger" },
  };

  const columns: Column<MaintenanceSheet>[] = useMemo(() => [
    { key: "ticket_no", header: t("maintenance.sheetList.ticket"), sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.ticket_no || "—"}</span> },
    { key: "date", header: t("common.date"), sortable: true, render: (r) => formatDate(r.date) },
    { key: "equipment", header: t("maintenance.sheetList.equipment"), sortable: true, render: (r) => <span className="font-medium">{r.equipment_name || "—"}</span> },
    { key: "fault", header: t("maintenance.sheetList.fault"), sortable: true, render: (r) => <span className="line-clamp-1">{r.fault_description || "—"}</span> },
    { key: "severity", header: t("maintenance.sheetList.severity"), render: (r) => { const s = severityMap[r.severity] || { label: r.severity, variant: "default" as BadgeVariant }; return <Badge variant={s.variant}>{s.label}</Badge>; } },
    { key: "downtime_hours", header: t("maintenance.sheetList.downtime"), align: "left", render: (r) => r.downtime_hours ? t("maintenance.hoursShort", { hours: r.downtime_hours }) : "—" },
    { key: "status", header: t("common.status"), render: (r) => { const s = statusMap[r.status] || { label: r.status, variant: "default" as BadgeVariant }; return <Badge variant={s.variant}>{s.label}</Badge>; } },
    { key: "assigned_to", header: t("maintenance.sheetList.assignedTo"), render: (r) => r.assigned_to || "—" },
  ], [t]);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">{t("maintenance.sheetList.title")}</h1>
          <p className="page-subtitle">{t("maintenance.sheetList.subtitle", { count: sheets.length })}</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => navigate("/maintenance/new")}>
          {t("maintenance.sheetList.newTicket")}
        </Button>
      </div>
      <DataTable
        columns={columns}
        data={sheets}
        loading={loading}
        onRowClick={(r) => navigate(`/maintenance/sheets/${r.id}`)}
        emptyMessage={t("maintenance.sheetList.empty")}
      />
    </div>
  );
}
