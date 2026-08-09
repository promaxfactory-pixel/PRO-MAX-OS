import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import { StatusBadge } from "@/components/ui/Badge";
import Badge from "@/components/ui/Badge";
import { formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, ClipboardList, User, Clock } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { useTranslation } from "react-i18next";

export default function OperationsSheetDetailPage() {
  const { t } = useTranslation();
  const { id } = useParams();
  const navigate = useNavigate();
  const { addNotification } = useUIStore();
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [sheet, setSheet] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    invoke("get_operations_sheet", { id: Number(id) })
      .then((d) => setSheet(d))
      .catch((e: unknown) => addNotification({ title: t('common.error'), message: String(e), type: 'error' }))
      .finally(() => setLoading(false));
  }, [id]);

  const shiftLabels: Record<string, string> = {
    morning: t("operations.shift.morning"),
    evening: t("operations.shift.evening"),
    night: t("operations.shift.night"),
  };

  if (loading || !sheet) {
    return <div className="flex items-center justify-center h-64"><div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" /></div>;
  }

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div className="flex items-center gap-3">
          <button onClick={() => navigate('/operations')} className="btn-ghost p-2"><ArrowRight className="w-5 h-5" /></button>
          <div>
            <h1 className="page-title flex items-center gap-3">
              <span className="font-mono text-brand-400">{sheet.sheet_no || "—"}</span>
              <StatusBadge status={sheet.status} />
            </h1>
            <p className="page-subtitle">{formatDate(sheet.date)} • {shiftLabels[sheet.shift] || sheet.shift}</p>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-3 gap-6">
        <Card className="col-span-2">
          <div className="grid grid-cols-2 gap-6">
            <div>
              <h4 className="text-sm text-surface-400 mb-3">{t("operations.generalInfo")}</h4>
              <div className="space-y-2">
                <div className="flex items-center gap-2 text-sm"><ClipboardList className="w-4 h-4 text-surface-500" /> <span>{t("operations.sheetNo", { no: sheet.sheet_no })}</span></div>
                <div className="flex items-center gap-2 text-sm"><Clock className="w-4 h-4 text-surface-500" /> <span>{t("operations.dateLabel", { date: formatDate(sheet.date) })}</span></div>
                <div className="flex items-center gap-2 text-sm"><User className="w-4 h-4 text-surface-500" /> <span>{t("operations.supervisorLabel", { name: sheet.supervisor || "—" })}</span></div>
                <div className="flex items-center gap-2 text-sm"><User className="w-4 h-4 text-surface-500" /> <span>{t("operations.workerLabel", { name: sheet.worker || "—" })}</span></div>
              </div>
            </div>
            <div>
              <h4 className="text-sm text-surface-400 mb-3">{t("operations.attendanceTimes")}</h4>
              <div className="space-y-2">
                <div className="flex justify-between text-sm"><span className="text-surface-400">{t("operations.attendance")}</span><span>{sheet.attendance || "—"}</span></div>
                <div className="flex justify-between text-sm"><span className="text-surface-400">{t("operations.startTime")}</span><span>{sheet.start_time || "—"}</span></div>
                <div className="flex justify-between text-sm"><span className="text-surface-400">{t("operations.endTime")}</span><span>{sheet.end_time || "—"}</span></div>
                <div className="flex justify-between text-sm"><span className="text-surface-400">{t("operations.workHours")}</span><span className="font-bold">{sheet.hours_worked || "—"}</span></div>
              </div>
            </div>
          </div>
        </Card>

        <Card>
          <h4 className="text-sm text-surface-400 mb-3">{t("operations.productionSummary")}</h4>
          <div className="space-y-4">
            <div className="text-center py-4">
              <p className="text-3xl font-bold gradient-text">{t("operations.tons", { count: sheet.production_output || 0 })}</p>
              <p className="text-xs text-surface-400 mt-1">{t("operations.totalProduction")}</p>
            </div>
            <div className="text-center py-2 bg-surface-900/50 rounded-xl">
              <p className="text-sm font-medium">{sheet.workers_count || 0}</p>
              <p className="text-xs text-surface-400">{t("operations.workersCount")}</p>
            </div>
          </div>
        </Card>
      </div>

      <div className="grid grid-cols-2 gap-6">
        <Card>
          <h4 className="text-sm text-surface-400 mb-3">{t("operations.qualityInspection")}</h4>
          <div className="space-y-2">
            <div className="flex justify-between text-sm"><span className="text-surface-400">{t("operations.inspectionResult")}</span><Badge variant={sheet.quality_check === "pass" ? "success" : "danger"}>{sheet.quality_check === "pass" ? t("operations.qualityPass") : t("operations.qualityFail")}</Badge></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">{t("operations.qualityNotes")}</span><span>{sheet.quality_notes || "—"}</span></div>
          </div>
        </Card>

        <Card>
          <h4 className="text-sm text-surface-400 mb-3">{t("operations.safetyNotes")}</h4>
          <div className="space-y-2">
            <div className="flex justify-between text-sm"><span className="text-surface-400">{t("operations.safetyActions")}</span><span>{sheet.safety_notes || "—"}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">{t("operations.incidents")}</span><span>{sheet.incidents || "—"}</span></div>
          </div>
        </Card>
      </div>

      <div className="grid grid-cols-2 gap-6">
        <Card>
          <h4 className="text-sm text-surface-400 mb-3">{t("operations.signatures")}</h4>
          <div className="space-y-2">
            <div className="flex justify-between text-sm"><span className="text-surface-400">{t("operations.supervisorSignature")}</span><span className="font-mono text-xs">{sheet.supervisor_signature || "—"}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">{t("operations.managerSignature")}</span><span className="font-mono text-xs">{sheet.manager_signature || "—"}</span></div>
          </div>
        </Card>

        <Card>
          <h4 className="text-sm text-surface-400 mb-3">{t("operations.additionalInfo")}</h4>
          <div className="space-y-2">
            <div className="flex justify-between text-sm"><span className="text-surface-400">{t("maintenance.sheetDetail.createdBy")}</span><span>{sheet.created_by || "—"}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">{t("maintenance.sheetDetail.time")}</span><span>{sheet.created_at || "—"}</span></div>
            {sheet.notes && <div className="p-3 bg-surface-900/50 rounded-xl mt-2"><p className="text-xs text-surface-400">{sheet.notes}</p></div>}
          </div>
        </Card>
      </div>
    </div>
  );
}
