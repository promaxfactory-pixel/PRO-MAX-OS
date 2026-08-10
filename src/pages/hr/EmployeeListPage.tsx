import { useState, useEffect, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import DataTable, { Column } from "@/components/ui/DataTable";
import Button from "@/components/ui/Button";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { Employee } from "@/types";

export default function EmployeeListPage() {
  const navigate = useNavigate();
  const { addNotification } = useUIStore();
  const [employees, setEmployees] = useState<Employee[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => { invoke("list_employees").then((d) => setEmployees(d as Employee[])).catch((e: unknown) => addNotification({ title: "خطأ", message: String(e), type: "error" })).finally(() => setLoading(false)); }, []);

  const columns: Column<any>[] = useMemo(() => [
    { key: "code", header: "الكود", render: (r) => <span className="font-mono text-brand-400">{r.code || "—"}</span> },
    { key: "name", header: "الاسم", sortable: true, render: (r) => <span className="font-medium">{r.name}</span> },
    { key: "job", header: "الوظيفة", render: (r) => r.job || "—" },
    { key: "nationality", header: "الجنسية", render: (r) => r.nationality || "—" },
    { key: "salary_milli", header: "الراتب", align: "left", render: (r) => formatOMR(r.salary_milli) },
    { key: "phone", header: "الهاتف", render: (r) => r.phone || "—" },
  ], []);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div><h1 className="page-title">الموظفين</h1><p className="page-subtitle">{employees.length} موظف</p></div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => navigate("/hr/employees/new")}>موظف جديد</Button>
      </div>
      <DataTable columns={columns} data={employees} loading={loading} onRowClick={(r) => navigate(`/hr/employees/${r.id}`)} emptyMessage="لا يوجد موظفين" />
    </div>
  );
}
