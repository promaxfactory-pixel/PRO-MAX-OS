import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import Badge, { StatusBadge } from "@/components/ui/Badge";
import { type BadgeVariant } from "@/components/ui/DataTable";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, Wrench, AlertTriangle, Clock, DollarSign, Calendar } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { MaintenanceSheet } from "@/types";

export default function MaintenanceSheetDetailPage() {
  const { id } = useParams();
  const navigate = useNavigate();
  const { addNotification } = useUIStore();
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [sheet, setSheet] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    invoke("get_maintenance_sheet", { id: Number(id) })
      .then((d) => setSheet(d))
      .catch((e: unknown) => addNotification({ title: 'ط®ط·ط£', message: String(e), type: 'error' }))
      .finally(() => setLoading(false));
  }, [id]);

  const severityMap: Record<string, { label: string; variant: BadgeVariant }> = {
    critical: { label: "ط­ط±ط¬", variant: "danger" },
    high: { label: "ظ…ط±طھظپط¹", variant: "warning" },
    medium: { label: "ظ…طھظˆط³ط·", variant: "info" },
    low: { label: "ظ…ظ†ط®ظپط¶", variant: "success" },
  };

  if (loading) {
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
              <span className="font-mono text-brand-400">{sheet.ticket_no || "â€”"}</span>
              <StatusBadge status={sheet.status} />
              <Badge variant={sev.variant}>{sev.label}</Badge>
            </h1>
            <p className="page-subtitle">{formatDate(sheet.date)} â€¢ {sheet.equipment_name || "â€”"}</p>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-3 gap-6">
        <Card className="col-span-2">
          <div className="grid grid-cols-2 gap-6">
            <div>
              <h4 className="text-sm text-surface-400 mb-3">ظ…ط¹ظ„ظˆظ…ط§طھ ط§ظ„ط¹ط·ظ„</h4>
              <div className="space-y-2">
                <div className="flex items-center gap-2 text-sm"><Wrench className="w-4 h-4 text-surface-500" /> <span>ط§ظ„ظ…ط¹ط¯ط©: {sheet.equipment_name || "â€”"}</span></div>
                <div className="flex items-center gap-2 text-sm"><AlertTriangle className="w-4 h-4 text-surface-500" /> <span>ط§ظ„ط¹ط·ظ„: {sheet.fault_description || "â€”"}</span></div>
                <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„ظˆط±ط¯ظٹط©</span><span>{sheet.shift || "â€”"}</span></div>
                <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„ظ…ط´ط±ظپ</span><span>{sheet.supervisor || "â€”"}</span></div>
              </div>
            </div>
            <div>
              <h4 className="text-sm text-surface-400 mb-3">طھظپط§طµظٹظ„ ط§ظ„ط¥طµظ„ط§ط­</h4>
              <div className="space-y-2">
                <div className="flex items-center gap-2 text-sm"><Clock className="w-4 h-4 text-surface-500" /> <span>ظˆظ‚طھ ط§ظ„طھظˆظ‚ظپ: {sheet.downtime_hours ? `${sheet.downtime_hours} ط³ط§ط¹ط©` : "â€”"}</span></div>
                <div className="flex justify-between text-sm"><span className="text-surface-400">ط­ط§ظ„ط© ط§ظ„ط¥طµظ„ط§ط­</span><StatusBadge status={sheet.repair_status || "pending"} /></div>
                <div className="flex justify-between text-sm"><span className="text-surface-400">ظ…ظ„ط§ط­ط¸ط§طھ ط§ظ„ط¥طµظ„ط§ط­</span><span>{sheet.repair_notes || "â€”"}</span></div>
              </div>
            </div>
          </div>
        </Card>

        <Card>
          <h4 className="text-sm text-surface-400 mb-3">ط§ظ„طھظƒط§ظ„ظٹظپ</h4>
          <div className="space-y-4">
            <div className="text-center py-4">
              <p className="text-3xl font-bold gradient-text">{formatOMR(sheet.total_cost_milli || 0)}</p>
              <p className="text-xs text-surface-400 mt-1">ط¥ط¬ظ…ط§ظ„ظٹ ط§ظ„طھظƒظ„ظپط©</p>
            </div>
            <div className="space-y-2">
              <div className="flex justify-between text-sm"><span className="text-surface-400">ظ‚ط·ط¹ ط§ظ„ط؛ظٹط§ط±</span><span>{formatOMR(sheet.parts_cost_milli || 0)}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„ط£ط¬ظˆط±</span><span>{formatOMR(sheet.labor_cost_milli || 0)}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">ط£ط®ط±ظ‰</span><span>{formatOMR(sheet.other_cost_milli || 0)}</span></div>
            </div>
          </div>
        </Card>
      </div>

      <div className="grid grid-cols-2 gap-6">
        <Card>
          <h4 className="text-sm text-surface-400 mb-3">ط§ظ„ط³ط¨ط¨ ط§ظ„ط¬ط°ط±ظٹ ظˆط§ظ„ظˆظ‚ط§ظٹط©</h4>
          <div className="space-y-2">
            <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„ط³ط¨ط¨ ط§ظ„ط¬ط°ط±ظٹ</span><span>{sheet.root_cause || "â€”"}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">ط¥ط¬ط±ط§ط،ط§طھ ظˆظ‚ط§ط¦ظٹط©</span><span>{sheet.preventive_action || "â€”"}</span></div>
          </div>
        </Card>

        <Card>
          <h4 className="text-sm text-surface-400 mb-3">ط§ظ„ظ…طھط§ط¨ط¹ط©</h4>
          <div className="space-y-2">
            <div className="flex items-center gap-2 text-sm"><Calendar className="w-4 h-4 text-surface-500" /> <span>طھط§ط±ظٹط® ط§ظ„ظ…طھط§ط¨ط¹ط©: {sheet.follow_up_date ? formatDate(sheet.follow_up_date) : "â€”"}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">ط£ظ†ط´ط£ظ‡</span><span>{sheet.created_by || "â€”"}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„ظˆظ‚طھ</span><span>{sheet.created_at || "â€”"}</span></div>
            {sheet.notes && <div className="p-3 bg-surface-900/50 rounded-xl mt-2"><p className="text-xs text-surface-400">{sheet.notes}</p></div>}
          </div>
        </Card>
      </div>
    </div>
  );
}



