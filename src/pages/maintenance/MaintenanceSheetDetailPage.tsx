import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import Card from "@/components/ui/Card";
import Badge, { StatusBadge } from "@/components/ui/Badge";
import { type BadgeVariant } from "@/components/ui/DataTable";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, Wrench, AlertTriangle, Clock, Calendar } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

export default function MaintenanceSheetDetailPage() {
  const { t } = useTranslation();
  const { id } = useParams();
  const navigate = useNavigate();
  const { addNotification } = useUIStore();
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [sheet, setSheet] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    invoke("get_maintenance_sheet", { id: Number(id) })
      .then((d) => setSheet(d))
      .catch((e: unknown) => addNotification({ title: t('common.error'), message: String(e), type: 'error' }))
      .finally(() => setLoading(false));
  }, [id, t]);

  const severityMap: Record<string, { label: string; variant: BadgeVariant }> = {
    critical: { label: t("maintenance.severity.critical"), variant: "danger" },
    high: { label: t("maintenance.severity.high"), variant: "warning" },
    medium: { label: t("maintenance.severity.medium"), variant: "info" },
    low: { label: t("maintenance.severity.low"), variant: "success" },
  };

  if (loading || !sheet) {
    return <div className="flex items-center justify-center h-64"><div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" /></div>;
  }

  const sev = severityMap[sheet.severity] || { label: sheet.severity, variant: "info" as BadgeVariant };

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div className="flex items-center gap-3">
          <button onClick={() => navigate('/maintenance')} className="btn-ghost p-2"><ArrowRight className="w-5 h-5" /></button>
          <div>
            <h1 className="page-title flex items-center gap-3">
              <span className="font-mono text-brand-400">{sheet.ticket_no || "—"}</span>
              <StatusBadge status={sheet.status} />
              <Badge variant={sev.variant}>{sev.label}</Badge>
            </h1>
            <p className="page-subtitle">{formatDate(sheet.date)} • {sheet.equipment_name || "—"}</p>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-3 gap-6">
        <Card className="col-span-2">
          <div className="grid grid-cols-2 gap-6">
            <div>
              <h4 className="text-sm text-surface-400 mb-3">{t("maintenance.sheetDetail.faultInfo")}</h4>
              <div className="space-y-2">
                <div className="flex items-center gap-2 text-sm"><Wrench className="w-4 h-4 text-surface-500" /> <span>{t("maintenance.sheetDetail.equipment")}: {sheet.equipment_name || "—"}</span></div>
                <div className="flex items-center gap-2 text-sm"><AlertTriangle className="w-4 h-4 text-surface-500" /> <span>{t("maintenance.sheetDetail.fault")}: {sheet.fault_description || "—"}</span></div>
                <div className="flex justify-between text-sm"><span className="text-surface-400">{t("maintenance.sheetDetail.shift")}</span><span>{sheet.shift || "—"}</span></div>
                <div className="flex justify-between text-sm"><span className="text-surface-400">{t("maintenance.sheetDetail.supervisor")}</span><span>{sheet.supervisor || "—"}</span></div>
              </div>
            </div>
            <div>
              <h4 className="text-sm text-surface-400 mb-3">{t("maintenance.sheetDetail.repairDetails")}</h4>
              <div className="space-y-2">
                <div className="flex items-center gap-2 text-sm"><Clock className="w-4 h-4 text-surface-500" /> <span>{t("maintenance.sheetDetail.downtime")}: {sheet.downtime_hours ? t("maintenance.hoursLong", { hours: sheet.downtime_hours }) : "—"}</span></div>
                <div className="flex justify-between text-sm"><span className="text-surface-400">{t("maintenance.sheetDetail.repairStatus")}</span><StatusBadge status={sheet.repair_status || "pending"} /></div>
                <div className="flex justify-between text-sm"><span className="text-surface-400">{t("maintenance.sheetDetail.repairNotes")}</span><span>{sheet.repair_notes || "—"}</span></div>
              </div>
            </div>
          </div>
        </Card>

        <Card>
          <h4 className="text-sm text-surface-400 mb-3">{t("maintenance.sheetDetail.costs")}</h4>
          <div className="space-y-4">
            <div className="text-center py-4">
              <p className="text-3xl font-bold gradient-text">{formatOMR(sheet.total_cost_milli || 0)}</p>
              <p className="text-xs text-surface-400 mt-1">{t("maintenance.sheetDetail.totalCost")}</p>
            </div>
            <div className="space-y-2">
              <div className="flex justify-between text-sm"><span className="text-surface-400">{t("maintenance.sheetDetail.partsCost")}</span><span>{formatOMR(sheet.parts_cost_milli || 0)}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">{t("maintenance.sheetDetail.laborCost")}</span><span>{formatOMR(sheet.labor_cost_milli || 0)}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">{t("maintenance.sheetDetail.otherCost")}</span><span>{formatOMR(sheet.other_cost_milli || 0)}</span></div>
            </div>
          </div>
        </Card>
      </div>

      <div className="grid grid-cols-2 gap-6">
        <Card>
          <h4 className="text-sm text-surface-400 mb-3">{t("maintenance.sheetDetail.rootCausePrevention")}</h4>
          <div className="space-y-2">
            <div className="flex justify-between text-sm"><span className="text-surface-400">{t("maintenance.sheetDetail.rootCause")}</span><span>{sheet.root_cause || "—"}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">{t("maintenance.sheetDetail.preventiveAction")}</span><span>{sheet.preventive_action || "—"}</span></div>
          </div>
        </Card>

        <Card>
          <h4 className="text-sm text-surface-400 mb-3">{t("maintenance.sheetDetail.followUp")}</h4>
          <div className="space-y-2">
            <div className="flex items-center gap-2 text-sm"><Calendar className="w-4 h-4 text-surface-500" /> <span>{t("maintenance.sheetDetail.followUpDate")}: {sheet.follow_up_date ? formatDate(sheet.follow_up_date) : "—"}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">{t("maintenance.sheetDetail.createdBy")}</span><span>{sheet.created_by || "—"}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">{t("maintenance.sheetDetail.time")}</span><span>{sheet.created_at || "—"}</span></div>
            {sheet.notes && <div className="p-3 bg-surface-900/50 rounded-xl mt-2"><p className="text-xs text-surface-400">{sheet.notes}</p></div>}
          </div>
        </Card>
      </div>
    </div>
  );
}
