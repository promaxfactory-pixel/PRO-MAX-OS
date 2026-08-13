import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import Badge, { StatusBadge } from "@/components/ui/Badge";
import { type BadgeVariant } from "@/components/ui/DataTable";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@/lib/tauri";
import { ArrowRight, Wrench, AlertTriangle, Clock, Calendar } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

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
      .catch((e: unknown) => addNotification({ title: "خطأ", message: String(e), type: "error" }))
      .finally(() => setLoading(false));
  }, [id]);

  const severityMap: Record<string, { label: string; variant: BadgeVariant }> = {
    critical: { label: "حرج", variant: "danger" },
    high: { label: "مرتفع", variant: "warning" },
    medium: { label: "متوسط", variant: "info" },
    low: { label: "منخفض", variant: "success" },
  };

  if (loading) {
    return <div className="flex items-center justify-center h-64"><div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" /></div>;
  }

  const sev = severityMap[sheet.severity] || { label: sheet.severity, variant: "info" as BadgeVariant };

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div className="flex items-center gap-3">
          <button onClick={() => navigate("/maintenance")} className="btn-ghost p-2"><ArrowRight className="w-5 h-5" /></button>
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
              <h4 className="text-sm text-surface-400 mb-3">معلومات العطل</h4>
              <div className="space-y-2">
                <div className="flex items-center gap-2 text-sm"><Wrench className="w-4 h-4 text-surface-500" /> <span>المعدة: {sheet.equipment_name || "—"}</span></div>
                <div className="flex items-center gap-2 text-sm"><AlertTriangle className="w-4 h-4 text-surface-500" /> <span>العطل: {sheet.fault_description || "—"}</span></div>
                <div className="flex justify-between text-sm"><span className="text-surface-400">الوردية</span><span>{sheet.shift || "—"}</span></div>
                <div className="flex justify-between text-sm"><span className="text-surface-400">المشرف</span><span>{sheet.supervisor || "—"}</span></div>
              </div>
            </div>
            <div>
              <h4 className="text-sm text-surface-400 mb-3">تفاصيل الإصلاح</h4>
              <div className="space-y-2">
                <div className="flex items-center gap-2 text-sm"><Clock className="w-4 h-4 text-surface-500" /> <span>وقت التوقف: {sheet.downtime_hours ? `${sheet.downtime_hours} ساعة` : "—"}</span></div>
                <div className="flex justify-between text-sm"><span className="text-surface-400">حالة الإصلاح</span><StatusBadge status={sheet.repair_status || "pending"} /></div>
                <div className="flex justify-between text-sm"><span className="text-surface-400">ملاحظات الإصلاح</span><span>{sheet.repair_notes || "—"}</span></div>
              </div>
            </div>
          </div>
        </Card>

        <Card>
          <h4 className="text-sm text-surface-400 mb-3">التكاليف</h4>
          <div className="space-y-4">
            <div className="text-center py-4">
              <p className="text-3xl font-bold gradient-text">{formatOMR(sheet.total_cost_milli || 0)}</p>
              <p className="text-xs text-surface-400 mt-1">إجمالي التكلفة</p>
            </div>
            <div className="space-y-2">
              <div className="flex justify-between text-sm"><span className="text-surface-400">قطع الغيار</span><span>{formatOMR(sheet.parts_cost_milli || 0)}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">الأجور</span><span>{formatOMR(sheet.labor_cost_milli || 0)}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">أخرى</span><span>{formatOMR(sheet.other_cost_milli || 0)}</span></div>
            </div>
          </div>
        </Card>
      </div>

      <div className="grid grid-cols-2 gap-6">
        <Card>
          <h4 className="text-sm text-surface-400 mb-3">السبب الجذري والوقاية</h4>
          <div className="space-y-2">
            <div className="flex justify-between text-sm"><span className="text-surface-400">السبب الجذري</span><span>{sheet.root_cause || "—"}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">إجراءات وقائية</span><span>{sheet.preventive_action || "—"}</span></div>
          </div>
        </Card>

        <Card>
          <h4 className="text-sm text-surface-400 mb-3">المتابعة</h4>
          <div className="space-y-2">
            <div className="flex items-center gap-2 text-sm"><Calendar className="w-4 h-4 text-surface-500" /> <span>تاريخ المتابعة: {sheet.follow_up_date ? formatDate(sheet.follow_up_date) : "—"}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">أنشأه</span><span>{sheet.created_by || "—"}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">الوقت</span><span>{sheet.created_at || "—"}</span></div>
            {sheet.notes && <div className="p-3 bg-surface-900/50 rounded-xl mt-2"><p className="text-xs text-surface-400">{sheet.notes}</p></div>}
          </div>
        </Card>
      </div>
    </div>
  );
}



