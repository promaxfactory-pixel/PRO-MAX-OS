import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, UserCog, Edit } from "lucide-react";

export default function EmployeeDetailPage() {
  const { id } = useParams();
  const navigate = useNavigate();
  const [emp, setEmp] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => { invoke("get_employee", { id: Number(id) }).then((d) => setEmp(d)).catch(console.error).finally(() => setLoading(false)); }, [id]);

  if (loading || !emp) return <div className="flex items-center justify-center h-64"><div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" /></div>;

  const expiryWarning = (date: string) => {
    if (!date) return '';
    const d = new Date(date);
    const now = new Date();
    const days = Math.ceil((d.getTime() - now.getTime()) / (1000 * 60 * 60 * 24));
    if (days < 0) return <span className="text-red-400 text-xs">منتهي</span>;
    if (days < 30) return <span className="text-amber-400 text-xs">{days} يوم</span>;
    return <span className="text-emerald-400 text-xs">{days} يوم</span>;
  };

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div className="flex items-center gap-3">
          <button onClick={() => navigate('/hr/employees')} className="btn-ghost p-2"><ArrowRight className="w-5 h-5" /></button>
          <div><h1 className="page-title">{emp.name}</h1><p className="page-subtitle">{emp.job || "—"}</p></div>
        </div>
        <Button variant="outline" icon={<Edit className="w-4 h-4" />} onClick={() => navigate(`/hr/employees/${id}/edit`)}>تعديل</Button>
      </div>
      <div className="grid grid-cols-3 gap-6">
        <Card className="col-span-2">
          <h3 className="section-title">المعلومات الشخصية</h3>
          <div className="grid grid-cols-2 gap-4 mt-4">
            <div className="space-y-2">
              <div className="flex justify-between text-sm"><span className="text-surface-400">الجنسية</span><span>{emp.nationality || "—"}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">الهاتف</span><span>{emp.phone || "—"}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">جواز السفر</span><span className="font-mono text-xs">{emp.passport_no || "—"}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">تاريخ الانضمام</span><span>{formatDate(emp.joining_date)}</span></div>
            </div>
            <div className="space-y-2">
              <div className="flex justify-between text-sm"><span className="text-surface-400">الراتب</span><span className="font-bold gradient-text">{formatOMR(emp.salary_milli)}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">البدلات</span><span>{formatOMR(emp.allowances_milli)}</span></div>
            </div>
          </div>
        </Card>
        <Card>
          <h3 className="section-title">الوثائق والانتهاء</h3>
          <div className="space-y-3 mt-4">
            <div className="flex justify-between text-sm"><span className="text-surface-400">جواز السفر</span>{expiryWarning(emp.passport_expiry)}</div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">الإقامة</span>{expiryWarning(emp.residence_expiry)}</div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">التأشيرة</span>{expiryWarning(emp.visa_expiry)}</div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">تصريح العمل</span>{expiryWarning(emp.workpermit_expiry)}</div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">التأمين</span>{expiryWarning(emp.insurance_expiry)}</div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">انتهاء العقد</span>{expiryWarning(emp.contract_end)}</div>
          </div>
        </Card>
      </div>
    </div>
  );
}
