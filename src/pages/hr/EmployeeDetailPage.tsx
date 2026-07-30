import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, UserCog, Edit } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { Employee } from "@/types";

export default function EmployeeDetailPage() {
  const { id } = useParams();
  const navigate = useNavigate();
  const { addNotification } = useUIStore();
  const [emp, setEmp] = useState<Employee | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => { invoke("get_employee", { id: Number(id) }).then((d) => setEmp(d as Employee)).catch((e: unknown) => addNotification({ title: 'ط®ط·ط£', message: String(e), type: 'error' })).finally(() => setLoading(false)); }, [id]);

  if (loading) return <div className="flex items-center justify-center h-64"><div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" /></div>;

  if (!emp) return <div className="flex flex-col items-center justify-center h-64 gap-4"><p className="text-surface-400">تعذر تحميل بيانات الموظف</p><button className="btn-outline px-4 py-2 rounded-xl text-sm" onClick={() => window.location.reload()}>إعادة المحاولة</button></div>;

  const expiryWarning = (date: string) => {
    if (!date) return '';
    const d = new Date(date);
    const now = new Date();
    const days = Math.ceil((d.getTime() - now.getTime()) / (1000 * 60 * 60 * 24));
    if (days < 0) return <span className="text-red-400 text-xs">ظ…ظ†طھظ‡ظٹ</span>;
    if (days < 30) return <span className="text-amber-400 text-xs">{days} ظٹظˆظ…</span>;
    return <span className="text-emerald-400 text-xs">{days} ظٹظˆظ…</span>;
  };

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div className="flex items-center gap-3">
          <button onClick={() => navigate('/hr/employees')} className="btn-ghost p-2"><ArrowRight className="w-5 h-5" /></button>
          <div><h1 className="page-title">{emp.name}</h1><p className="page-subtitle">{emp.job || "â€”"}</p></div>
        </div>
        <Button variant="outline" icon={<Edit className="w-4 h-4" />} onClick={() => navigate(`/hr/employees/${id}/edit`)}>طھط¹ط¯ظٹظ„</Button>
      </div>
      <div className="grid grid-cols-3 gap-6">
        <Card className="col-span-2">
          <h3 className="section-title">ط§ظ„ظ…ط¹ظ„ظˆظ…ط§طھ ط§ظ„ط´ط®طµظٹط©</h3>
          <div className="grid grid-cols-2 gap-4 mt-4">
            <div className="space-y-2">
              <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„ط¬ظ†ط³ظٹط©</span><span>{emp.nationality || "â€”"}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„ظ‡ط§طھظپ</span><span>{emp.phone || "â€”"}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">ط¬ظˆط§ط² ط§ظ„ط³ظپط±</span><span className="font-mono text-xs">{emp.passport_no || "â€”"}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">طھط§ط±ظٹط® ط§ظ„ط§ظ†ط¶ظ…ط§ظ…</span><span>{formatDate(emp.joining_date)}</span></div>
            </div>
            <div className="space-y-2">
              <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„ط±ط§طھط¨</span><span className="font-bold gradient-text">{formatOMR(emp.salary_milli)}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„ط¨ط¯ظ„ط§طھ</span><span>{formatOMR(emp.allowances_milli)}</span></div>
            </div>
          </div>
        </Card>
        <Card>
          <h3 className="section-title">ط§ظ„ظˆط«ط§ط¦ظ‚ ظˆط§ظ„ط§ظ†طھظ‡ط§ط،</h3>
          <div className="space-y-3 mt-4">
            <div className="flex justify-between text-sm"><span className="text-surface-400">ط¬ظˆط§ط² ط§ظ„ط³ظپط±</span>{expiryWarning(emp.passport_expiry)}</div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„ط¥ظ‚ط§ظ…ط©</span>{expiryWarning(emp.residence_expiry)}</div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„طھط£ط´ظٹط±ط©</span>{expiryWarning(emp.visa_expiry)}</div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">طھطµط±ظٹط­ ط§ظ„ط¹ظ…ظ„</span>{expiryWarning(emp.workpermit_expiry)}</div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„طھط£ظ…ظٹظ†</span>{expiryWarning(emp.insurance_expiry)}</div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ†طھظ‡ط§ط، ط§ظ„ط¹ظ‚ط¯</span>{expiryWarning(emp.contract_end)}</div>
          </div>
        </Card>
      </div>
    </div>
  );
}

