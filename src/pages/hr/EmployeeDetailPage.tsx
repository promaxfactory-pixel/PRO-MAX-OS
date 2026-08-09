import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, Edit } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { Employee } from "@/types";

export default function EmployeeDetailPage() {
  const { t } = useTranslation();
  const { id } = useParams();
  const navigate = useNavigate();
  const { addNotification } = useUIStore();
  const [emp, setEmp] = useState<Employee | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => { invoke("get_employee", { id: Number(id) }).then((d) => setEmp(d as Employee)).catch((e: unknown) => addNotification({ title: t('common.error'), message: String(e), type: 'error' })).finally(() => setLoading(false)); }, [id, t]);

  if (loading || !emp) return <div className="flex items-center justify-center h-64"><div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" /></div>;

  const expiryWarning = (date: string) => {
    if (!date) return '';
    const d = new Date(date);
    const now = new Date();
    const days = Math.ceil((d.getTime() - now.getTime()) / (1000 * 60 * 60 * 24));
    if (days < 0) return <span className="text-red-400 text-xs">{t("employeeDetail.expired")}</span>;
    if (days < 30) return <span className="text-amber-400 text-xs">{t("employeeDetail.daysLeft", { days })}</span>;
    return <span className="text-emerald-400 text-xs">{t("employeeDetail.daysLeft", { days })}</span>;
  };

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div className="flex items-center gap-3">
          <button onClick={() => navigate('/hr/employees')} className="btn-ghost p-2"><ArrowRight className="w-5 h-5" /></button>
          <div><h1 className="page-title">{emp.name}</h1><p className="page-subtitle">{emp.job || "—"}</p></div>
        </div>
        <Button variant="outline" icon={<Edit className="w-4 h-4" />} onClick={() => navigate(`/hr/employees/${id}/edit`)}>{t("common.edit")}</Button>
      </div>
      <div className="grid grid-cols-3 gap-6">
        <Card className="col-span-2">
          <h3 className="section-title">{t("employeeDetail.personalInfo")}</h3>
          <div className="grid grid-cols-2 gap-4 mt-4">
            <div className="space-y-2">
              <div className="flex justify-between text-sm"><span className="text-surface-400">{t("hr.nationality")}</span><span>{emp.nationality || "—"}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">{t("hr.phone")}</span><span>{emp.phone || "—"}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">{t("employeeDetail.passport")}</span><span className="font-mono text-xs">{emp.passport_no || "—"}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">{t("employeeDetail.joiningDate")}</span><span>{formatDate(emp.joining_date)}</span></div>
            </div>
            <div className="space-y-2">
              <div className="flex justify-between text-sm"><span className="text-surface-400">{t("hr.salary")}</span><span className="font-bold gradient-text">{formatOMR(emp.salary_milli)}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">{t("hr.allowances")}</span><span>{formatOMR(emp.allowances_milli)}</span></div>
            </div>
          </div>
        </Card>
        <Card>
          <h3 className="section-title">{t("employeeDetail.documentsExpiry")}</h3>
          <div className="space-y-3 mt-4">
            <div className="flex justify-between text-sm"><span className="text-surface-400">{t("employeeDetail.passport")}</span>{expiryWarning(emp.passport_expiry)}</div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">{t("employeeDetail.residence")}</span>{expiryWarning(emp.residence_expiry)}</div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">{t("employeeDetail.visa")}</span>{expiryWarning(emp.visa_expiry)}</div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">{t("employeeDetail.workPermit")}</span>{expiryWarning(emp.workpermit_expiry)}</div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">{t("employeeDetail.insurance")}</span>{expiryWarning(emp.insurance_expiry)}</div>
            <div className="flex justify-between text-sm"><span className="text-surface-400">{t("hr.contractEnd")}</span>{expiryWarning(emp.contract_end)}</div>
          </div>
        </Card>
      </div>
    </div>
  );
}
