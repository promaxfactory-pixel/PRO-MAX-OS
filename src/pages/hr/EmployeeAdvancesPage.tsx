import { useState, useEffect, useMemo, useCallback } from "react";
import { useTranslation } from "react-i18next";
import DataTable, { Column } from "@/components/ui/DataTable";
import Badge from "@/components/ui/Badge";
import Button from "@/components/ui/Button";
import Card from "@/components/ui/Card";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus, HandCoins } from "lucide-react";
import ConfirmDialog from "@/components/ui/ConfirmDialog";
import { useUIStore } from "@/stores/uiStore";
import type { Employee } from "@/types";

interface EmployeeAdvance {
  id: number;
  employee_id: number;
  employee_name: string;
  amount_milli: number;
  date: string;
  reason: string;
  remaining_milli: number;
  deduction_per_payroll_milli: number;
  status: string;
}

export default function EmployeeAdvancesPage() {
  const { t } = useTranslation();
  const addNotification = useUIStore((s) => s.addNotification);
  const [advances, setAdvances] = useState<EmployeeAdvance[]>([]);
  const [employees, setEmployees] = useState<Employee[]>([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [saving, setSaving] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);
  const [form, setForm] = useState({
    employee_id: "",
    amount_milli: "",
    date: new Date().toISOString().split("T")[0],
    reason: "",
    deduction_per_payroll_milli: "",
  });

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const [advData, empData] = await Promise.all([
        invoke("list_employee_advances"),
        invoke("list_employees"),
      ]);
      setAdvances(advData as EmployeeAdvance[]);
      setEmployees(empData as Employee[]);
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("employeeAdvances.errorLoad") }); }
    finally { setLoading(false); }
  }, [addNotification, t]);

  useEffect(() => { loadData(); }, [loadData]);

  const handleCreate = async () => {
    setSaving(true);
    try {
      await invoke("create_employee_advance", {
        input: {
          employee_id: Number(form.employee_id),
          amount_milli: Number(form.amount_milli),
          date: form.date,
          reason: form.reason,
          deduction_per_payroll_milli: Number(form.deduction_per_payroll_milli),
        },
      });
      setShowForm(false);
      setForm({ employee_id: "", amount_milli: "", date: new Date().toISOString().split("T")[0], reason: "", deduction_per_payroll_milli: "" });
      loadData();
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("employeeAdvances.errorSave") }); }
    finally { setSaving(false); }
  };

  const totalOutstanding = advances.filter((a) => a.status === "open").reduce((s, a) => s + (a.remaining_milli || 0), 0);
  const closedCount = advances.filter((a) => a.status === "closed").length;

  const columns: Column<EmployeeAdvance>[] = useMemo(() => [
    { key: "employee_name", header: t("employeeAdvances.employee"), sortable: true, render: (r) => <span className="font-medium">{r.employee_name}</span> },
    { key: "amount_milli", header: t("employeeAdvances.amount"), sortable: true, align: "left", render: (r) => formatOMR(r.amount_milli) },
    { key: "date", header: t("common.date"), sortable: true, render: (r) => formatDate(r.date) },
    { key: "reason", header: t("employeeAdvances.reason"), render: (r) => r.reason || "—" },
    { key: "remaining_milli", header: t("common.remaining"), align: "left", render: (r) => <span className="font-bold text-gold-400">{formatOMR(r.remaining_milli)}</span> },
    { key: "status", header: t("common.status"), render: (r) => (
      <Badge variant={r.status === "open" ? "warning" : "success"}>
        {r.status === "open" ? t("badge.open") : t("badge.closed")}
      </Badge>
    )},
  ], [t]);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">{t("employeeAdvances.title")}</h1>
          <p className="page-subtitle">{t("employeeAdvances.subtitle", { count: advances.length })}</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => setShowForm(true)}>{t("employeeAdvances.newAdvance")}</Button>
      </div>

      <div className="grid grid-cols-4 gap-4">
        <Card className="text-center">
          <p className="text-2xl font-bold gradient-text">{advances.length}</p>
          <p className="text-xs text-surface-400">{t("employeeAdvances.totalAdvances")}</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-gold-400">{formatOMR(totalOutstanding)}</p>
          <p className="text-xs text-surface-400">{t("employeeAdvances.remainingAmount")}</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-emerald-400">{closedCount}</p>
          <p className="text-xs text-surface-400">{t("employeeAdvances.closedAdvances")}</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-yellow-400">{advances.length - closedCount}</p>
          <p className="text-xs text-surface-400">{t("employeeAdvances.openAdvances")}</p>
        </Card>
      </div>

      {showForm && (
        <Card>
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-bold text-white">{t("employeeAdvances.newAdvance")}</h2>
            <button onClick={() => setShowForm(false)} className="text-surface-400 hover:text-white text-xl">&times;</button>
          </div>
          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="input-group">
                <label className="input-label">{t("employeeAdvances.employee")}</label>
                <select value={form.employee_id} onChange={(e) => setForm({ ...form, employee_id: e.target.value })} className="input-field" aria-label={t("employeeAdvances.employee")}>
                  <option value="">{t("employeeAdvances.selectEmployee")}</option>
                  {employees.map((emp: any) => (
                    <option key={emp.id} value={emp.id}>{emp.name}</option>
                  ))}
                </select>
              </div>
              <div className="input-group">
                <label className="input-label">{t("employeeAdvances.amount")}</label>
                <input type="number" value={form.amount_milli} onChange={(e) => setForm({ ...form, amount_milli: e.target.value })} className="input-field" dir="ltr" aria-label={t("employeeAdvances.amountAria")} />
              </div>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="input-group">
                <label className="input-label">{t("common.date")}</label>
                <input type="date" value={form.date} onChange={(e) => setForm({ ...form, date: e.target.value })} className="input-field" aria-label={t("common.date")} />
              </div>
              <div className="input-group">
                <label className="input-label">{t("employeeAdvances.deductionPerPayroll")}</label>
                <input type="number" value={form.deduction_per_payroll_milli} onChange={(e) => setForm({ ...form, deduction_per_payroll_milli: e.target.value })} className="input-field" dir="ltr" aria-label={t("employeeAdvances.deductionPerPayrollAria")} />
              </div>
            </div>
            <div className="input-group">
              <label className="input-label">{t("employeeAdvances.reason")}</label>
              <input type="text" value={form.reason} onChange={(e) => setForm({ ...form, reason: e.target.value })} className="input-field" placeholder={t("employeeAdvances.reasonPlaceholder")} aria-label={t("employeeAdvances.reason")} />
            </div>
          </div>
          <div className="flex justify-end gap-3 mt-4">
            <Button variant="ghost" onClick={() => setShowForm(false)}>{t("common.cancel")}</Button>
            <Button icon={<HandCoins className="w-4 h-4" />} onClick={() => setShowConfirm(true)} loading={saving}>{t("employeeAdvances.createAdvance")}</Button>
          </div>
        </Card>
      )}

      <DataTable columns={columns} data={advances} loading={loading} emptyMessage={t("employeeAdvances.empty")} />

      <ConfirmDialog
        open={showConfirm}
        title={t("employeeAdvances.confirmTitle")}
        message={t("employeeAdvances.confirmMessage")}
        variant="warning"
        onConfirm={() => { handleCreate(); setShowConfirm(false); }}
        onCancel={() => setShowConfirm(false)}
      />
    </div>
  );
}
