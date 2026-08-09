import { useState, useEffect, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import DataTable, { Column } from "@/components/ui/DataTable";
import Button from "@/components/ui/Button";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { Employee } from "@/types";

export default function EmployeeListPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { addNotification } = useUIStore();
  const [employees, setEmployees] = useState<Employee[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => { invoke("list_employees").then((d) => setEmployees(d as Employee[])).catch((e: unknown) => addNotification({ title: t('common.error'), message: String(e), type: 'error' })).finally(() => setLoading(false)); }, [t]);

  const columns: Column<any>[] = useMemo(() => [
    { key: "code", header: t("hr.code"), render: (r) => <span className="font-mono text-brand-400">{r.code || "—"}</span> },
    { key: "name", header: t("hr.name"), sortable: true, render: (r) => <span className="font-medium">{r.name}</span> },
    { key: "job", header: t("hr.job"), render: (r) => r.job || "—" },
    { key: "nationality", header: t("hr.nationality"), render: (r) => r.nationality || "—" },
    { key: "salary_milli", header: t("hr.salary"), align: "left", render: (r) => formatOMR(r.salary_milli) },
    { key: "phone", header: t("hr.phone"), render: (r) => r.phone || "—" },
  ], [t]);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div><h1 className="page-title">{t("hr.employees")}</h1><p className="page-subtitle">{t("employeeList.count", { count: employees.length })}</p></div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => navigate('/hr/employees/new')}>{t("hr.newEmployee")}</Button>
      </div>
      <DataTable columns={columns} data={employees} loading={loading} onRowClick={(r) => navigate(`/hr/employees/${r.id}`)} emptyMessage={t("employeeList.empty")} />
    </div>
  );
}
