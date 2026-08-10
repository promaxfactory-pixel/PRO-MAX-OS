import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import { StatusBadge } from "@/components/ui/Badge";
import Badge from "@/components/ui/Badge";
import { formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, ClipboardList, User, Clock } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

export default function OperationsSheetDetailPage() {
  const { id } = useParams();
  const navigate = useNavigate();
  const { addNotification } = useUIStore();
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [sheet, setSheet] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    invoke("get_operations_sheet", { id: Number(id) })
      .then((d) => setSheet(d))
      .catch((e: unknown) => addNotification({ title: "ط®ط·ط£", message: String(e), type: "error" }))
      .finally(() => setLoading(false));
  }, [id]);

  const shiftLabels: Record<string, string> = {
    morning: "طµط¨ط§ط­ظٹ",
    evening: "ظ…ط³ط§ط¦ظٹ",
    night: "ظ„ظٹظ„ظٹ",
  };

  if (loading) {
    return <div className="flex items-center justify-center h-64"><div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" /></div>;
  }

  if (!sheet) {
    return <div className="flex flex-col items-center justify-center h-64 gap-4"><p className="text-surface-400">تعذر تحميل ورقة التشغيل</p><button className="btn-outline px-4 py-2 rounded-xl text-sm" onClick={() => window.location.reload()}>إعادة المحاولة</button></div>;
  }

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div className="flex items-center gap-3">
          <button onClick={() => navigate("/operations")} className="btn-ghost p-2"><ArrowRight className="w-5 h-5" /></button>
          <div>
            <h1 className="page-title flex items-center gap-3">
              <span className="font-mono text-brand-400">{sheet.sheet_no || "â€”"}</span>
              <StatusBadge status={sheet.status} />
            </h1>
            <p className="page-subtitle">{formatDate(sheet.date)} â€¢ {shiftLabels[sheet.shift] || sheet.shift}</p>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-3 gap-6">
        <Card className="col-span-2">
          <div className="grid grid-cols-2 gap-6">
            <div>
              <h4 className="text-sm text-surface-400 mb-3">ظ…ط¹ظ„ظˆظ…ط§طھ ط¹ط§ظ…ط©</h4>
              <div className="space-y-2">
                <div className="flex items-center gap-2 text-sm"><ClipboardList className="w-4 h-4 text-surface-500" /> <span>ط±ظ‚ظ… ط§ظ„ظˆط±ظ‚ط©: <span className="font-mono">{sheet.sheet_no}</span></span></div>
                <div className="flex items-center gap-2 text-sm"><Clock className="w-4 h-4 text-surface-500" /> <span>ط§ظ„طھط§ط±ظٹط®: {formatDate(sheet.date)}</span></div>
                <div className="flex items-center gap-2 text-sm"><User className="w-4 h-4 text-surface-500" /> <span>ط§ظ„ظ…ط´ط±ظپ: {sheet.supervisor || "â€”"}</span></div>
                <div className="flex items-center gap-2 text-sm"><User className="w-4 h-4 text-surface-500" /> <span>ط§ظ„ط¹ط§ظ…ظ„: {sheet.worker || "â€”"}</span></div>
              </div>
            </div>
            <div>
              <h4 className="text-sm text-surface-400 mb-3">ط§ظ„ط­ط¶ظˆط± ظˆط§ظ„ط£ظˆظ‚ط§طھ</h4>
              <div className="space-y-2">
                <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„ط­ط¶ظˆط±</span><span>{sheet.attendance || "â€”"}</span></div>
                <div className="flex justify-between text-sm"><span className="text-surface-400">ظˆظ‚طھ ط§ظ„ط¨ط¯ط§ظٹط©</span><span>{sheet.start_time || "â€”"}</span></div>
                <div className="flex justify-between text-sm"><span className="text-surface-400">ظˆظ‚طھ ط§ظ„ظ†ظ‡ط§ظٹط©</span><span>{sheet.end_time || "â€”"}</span></div>
                <div className="flex justify-between text-sm"><span className="text-surface-400">ط³ط§ط¹ط§طھ ط§ظ„ط¹ظ…ظ„</span><span className="font-bold">{sheet.hours_worked || "â€”"}</span></div>
              </div>
            </div>
          </div>
        </Card>

        <Card>
          <h4 className="text-sm text-surface-400 mb-3">ظ…ظ„ط®طµ ط§ظ„ط¥ظ†طھط§ط¬</h4>
          <div className="space-y-4">
            <div className="text-center py-4">
              <p className="text-3xl font-bold gradient-text">{sheet.production_output || 0} ط·ظ†</p>
              <p className="text-xs text-surface-400 mt-1">ط¥ط¬ظ…ط§ظ„ظٹ ط§ظ„ط¥ظ†طھط§ط¬</p>
            </div>
            <div className="text-center py-2 bg-surface-900/50 rounded-xl">
              <p className="text-sm font-medium">{sheet.workers_count || 0}</p>
              <p className="text-xs text-surface-400">ط¹ط¯ط¯ ط§ظ„ط¹ظ…ط§ظ„</p>
            </div>
          </div>
        </Card>
      </div>

      <div className="grid grid-cols-2 gap-6">
        <Card>
          <h4 className="text-sm text-surface-400 mb-3">ظپط­طµ ط§ظ„ط¬ظˆط¯ط©</h4>
          <div className="space-y-2">
            <div className="flex justify-between text-sm"><span className="text-surface-400">ظ†طھظٹط¬ط© ط§ظ„ظپط­طµ</span><Badge variant={sheet.quality_check === "pass" ? "success" : "danger"}>{sheet.quality_check === "pass" ? "ظ†ط§ط¬ط­" : "ط؛ظٹط± ظ†ط§ط¬ط­"}</Badge></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">ظ…ظ„ط§ط­ط¸ط§طھ ط§ظ„ط¬ظˆط¯ط©</span><span>{sheet.quality_notes || "â€”"}</span></div>
          </div>
        </Card>

        <Card>
          <h4 className="text-sm text-surface-400 mb-3">ظ…ظ„ط§ط­ط¸ط§طھ ط§ظ„ط³ظ„ط§ظ…ط©</h4>
          <div className="space-y-2">
            <div className="flex justify-between text-sm"><span className="text-surface-400">ط¥ط¬ط±ط§ط،ط§طھ ط§ظ„ط³ظ„ط§ظ…ط©</span><span>{sheet.safety_notes || "â€”"}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„ط­ظˆط§ط¯ط«</span><span>{sheet.incidents || "â€”"}</span></div>
          </div>
        </Card>
      </div>

      <div className="grid grid-cols-2 gap-6">
        <Card>
          <h4 className="text-sm text-surface-400 mb-3">ط§ظ„طھظˆظ‚ظٹط¹ط§طھ</h4>
          <div className="space-y-2">
            <div className="flex justify-between text-sm"><span className="text-surface-400">طھظˆظ‚ظٹط¹ ط§ظ„ظ…ط´ط±ظپ</span><span className="font-mono text-xs">{sheet.supervisor_signature || "â€”"}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">طھظˆظ‚ظٹط¹ ط§ظ„ظ…ط¯ظٹط±</span><span className="font-mono text-xs">{sheet.manager_signature || "â€”"}</span></div>
          </div>
        </Card>

        <Card>
          <h4 className="text-sm text-surface-400 mb-3">ظ…ط¹ظ„ظˆظ…ط§طھ ط¥ط¶ط§ظپظٹط©</h4>
          <div className="space-y-2">
            <div className="flex justify-between text-sm"><span className="text-surface-400">ط£ظ†ط´ط£ظ‡</span><span>{sheet.created_by || "â€”"}</span></div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„ظˆظ‚طھ</span><span>{sheet.created_at || "â€”"}</span></div>
            {sheet.notes && <div className="p-3 bg-surface-900/50 rounded-xl mt-2"><p className="text-xs text-surface-400">{sheet.notes}</p></div>}
          </div>
        </Card>
      </div>
    </div>
  );
}



