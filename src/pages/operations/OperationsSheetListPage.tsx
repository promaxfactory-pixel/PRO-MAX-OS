import { useState, useEffect, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import DataTable, { Column, type BadgeVariant } from "@/components/ui/DataTable";
import Badge from "@/components/ui/Badge";
import Button from "@/components/ui/Button";
import { formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { OperationsSheet } from "@/types";
import { useTranslation } from "react-i18next";

export default function OperationsSheetListPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { addNotification } = useUIStore();
  const [sheets, setSheets] = useState<OperationsSheet[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    invoke("list_operations_sheets")
      .then((d) => setSheets(d as OperationsSheet[]))
      .catch((e: unknown) => addNotification({ title: t('common.error'), message: String(e), type: 'error' }))
      .finally(() => setLoading(false));
  }, []);

  const statusMap: Record<string, { label: string; variant: BadgeVariant }> = {
    draft: { label: t("badge.draft"), variant: "warning" },
    submitted: { label: t("operations.statusSubmitted"), variant: "info" },
    approved: { label: t("badge.approved"), variant: "success" },
    rejected: { label: t("badge.rejected"), variant: "danger" },
  };

  const shiftLabels: Record<string, string> = {
    morning: t("operations.shift.morning"),
    evening: t("operations.shift.evening"),
    night: t("operations.shift.night"),
  };

  const columns: Column<any>[] = useMemo(() => [
    { key: "sheet_no", header: t("operations.sheetNoHeader"), sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.sheet_no || "—"}</span> },
    { key: "date", header: t("common.date"), sortable: true, render: (r) => formatDate(r.date) },
    { key: "shift", header: t("production.shift"), render: (r) => <Badge variant="info">{shiftLabels[r.shift] || r.shift}</Badge> },
    { key: "workers_count", header: t("operations.workersCount"), align: "left", render: (r) => <span className="font-bold">{r.workers_count || 0}</span> },
    { key: "production_output", header: t("production.title"), align: "left", render: (r) => t("operations.tons", { count: r.production_output || 0 }) },
    { key: "status", header: t("common.status"), render: (r) => { const s = statusMap[r.status] || { label: r.status, variant: "default" as BadgeVariant }; return <Badge variant={s.variant}>{s.label}</Badge>; } },
    { key: "created_by", header: t("maintenance.sheetDetail.createdBy") },
  ], [t]);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">{t("operations.listTitle")}</h1>
          <p className="page-subtitle">{t("operations.sheetCount", { count: sheets.length })}</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => navigate("/operations/new")}>
          {t("operations.newSheet")}
        </Button>
      </div>
      <DataTable
        columns={columns}
        data={sheets}
        loading={loading}
        onRowClick={(r) => navigate(`/operations/sheets/${r.id}`)}
        emptyMessage={t("operations.empty")}
      />
    </div>
  );
}
