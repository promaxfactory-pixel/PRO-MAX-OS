import { useState, useEffect, useMemo, useCallback } from "react";
import { useTranslation } from "react-i18next";
import DataTable, { Column } from "@/components/ui/DataTable";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import OvertimeBadge from "@/components/ui/OvertimeBadge";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus, Clock } from "lucide-react";
import ConfirmDialog from "@/components/ui/ConfirmDialog";
import { useUIStore } from "@/stores/uiStore";
import type { Employee } from "@/types";

interface OvertimeRecord {
  id: number;
  employee_id: number;
  employee_name: string;
  date: string;
  hours: number;
  rate_multiplier: number;
  reason: string;
  notes: string;
  status: string;
}

export default function OvertimePage() {
  const { t } = useTranslation();
  const addNotification = useUIStore((s) => s.addNotification);
  const [records, setRecords] = useState<OvertimeRecord[]>([]);
  const [employees, setEmployees] = useState<Employee[]>([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [saving, setSaving] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);
  const [pendingAction, setPendingAction] = useState<() => void>(() => {});
  const [confirmTitle, setConfirmTitle] = useState("");
  const [confirmMessage, setConfirmMessage] = useState("");
  const [confirmVariant, setConfirmVariant] = useState<'danger' | 'warning' | 'info'>('danger');
  const [form, setForm] = useState({
    employee_id: "",
    date: new Date().toISOString().split("T")[0],
    hours: "",
    rate_multiplier: "1.5",
    reason: "",
    notes: "",
  });

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const [otData, empData] = await Promise.all([
        invoke("list_overtime_records"),
        invoke("list_employees"),
      ]);
      setRecords(otData as OvertimeRecord[]);
      setEmployees(empData as Employee[]);
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("overtime.errorLoad") }); }
    finally { setLoading(false); }
  }, [addNotification, t]);

  useEffect(() => { loadData(); }, [loadData]);

  const handleCreate = async () => {
    setSaving(true);
    try {
      await invoke("create_overtime_record", {
        input: {
          employee_id: Number(form.employee_id),
          date: form.date,
          hours: Number(form.hours),
          rate_multiplier: Number(form.rate_multiplier),
          reason: form.reason,
          notes: form.notes,
        },
      });
      setShowForm(false);
      setForm({ employee_id: "", date: new Date().toISOString().split("T")[0], hours: "", rate_multiplier: "1.5", reason: "", notes: "" });
      loadData();
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("overtime.errorSave") }); }
    finally { setSaving(false); }
  };

  const handleApprove = async (id: number) => {
    try {
      await invoke("approve_overtime", { id, input: { approved_by: "admin" } });
      loadData();
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("overtime.errorSave") }); }
  };

  const handleReject = async (id: number) => {
    try {
      await invoke("reject_overtime", { id });
      loadData();
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("overtime.errorSave") }); }
  };

  const totalHoursThisMonth = records
    .filter((r) => {
      const d = new Date(r.date);
      const now = new Date();
      return d.getMonth() === now.getMonth() && d.getFullYear() === now.getFullYear();
    })
    .reduce((s, r) => s + (r.hours || 0), 0);

  const totalCost = records
    .filter((r) => {
      const d = new Date(r.date);
      const now = new Date();
      return d.getMonth() === now.getMonth() && d.getFullYear() === now.getFullYear();
    })
    .reduce((s, r) => s + (r.hours || 0) * (r.rate_multiplier || 1), 0);

  const pendingCount = records.filter((r) => r.status === "pending").length;
  const approvedCount = records.filter((r) => r.status === "approved").length;

  const columns: Column<OvertimeRecord>[] = useMemo(() => [
    { key: "employee_name", header: t("overtime.employee"), sortable: true, render: (r) => <span className="font-medium">{r.employee_name}</span> },
    { key: "date", header: t("common.date"), sortable: true, render: (r) => formatDate(r.date) },
    { key: "hours", header: t("overtime.hours"), sortable: true, align: "left", render: (r) => <span className="font-bold">{r.hours}</span> },
    { key: "rate_multiplier", header: t("overtime.multiplier"), sortable: true, align: "left", render: (r) => <span className="text-gold-400">{r.rate_multiplier}x</span> },
    { key: "reason", header: t("overtime.reason"), render: (r) => r.reason || "—" },
    { key: "status", header: t("common.status"), render: (r) => <OvertimeBadge status={r.status} /> },
    { key: "actions", header: "", render: (r) => r.status === "Pending" ? (
      <div className="flex gap-2">
        <Button size="sm" variant="success" onClick={(e) => { e.stopPropagation(); setConfirmTitle(t("overtime.approveConfirmTitle")); setConfirmMessage(t("overtime.approveConfirmMessage")); setConfirmVariant("warning"); setPendingAction(() => () => handleApprove(r.id)); setShowConfirm(true); }}>{t("overtime.approve")}</Button>
        <Button size="sm" variant="danger" onClick={(e) => { e.stopPropagation(); setConfirmTitle(t("overtime.rejectConfirmTitle")); setConfirmMessage(t("overtime.rejectConfirmMessage")); setConfirmVariant("danger"); setPendingAction(() => () => handleReject(r.id)); setShowConfirm(true); }}>{t("overtime.reject")}</Button>
      </div>
    ) : null },
  ], [t]);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">{t("overtime.title")}</h1>
          <p className="page-subtitle">{t("overtime.subtitle", { count: records.length })}</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => setShowForm(true)}>{t("overtime.newRecord")}</Button>
      </div>

      <div className="grid grid-cols-4 gap-4">
        <Card className="text-center">
          <p className="text-2xl font-bold gradient-text">{totalHoursThisMonth}</p>
          <p className="text-xs text-surface-400">{t("overtime.hoursThisMonth")}</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-gold-400">{formatOMR(Math.round(totalCost * 1000))}</p>
          <p className="text-xs text-surface-400">{t("overtime.estimatedCost")}</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-yellow-400">{pendingCount}</p>
          <p className="text-xs text-surface-400">{t("overtime.pendingApproval")}</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-emerald-400">{approvedCount}</p>
          <p className="text-xs text-surface-400">{t("overtime.approved")}</p>
        </Card>
      </div>

      {showForm && (
        <Card>
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-bold text-white">{t("overtime.newRecord")}</h2>
            <button onClick={() => setShowForm(false)} className="text-surface-400 hover:text-white text-xl">&times;</button>
          </div>
          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="input-group">
                <label className="input-label">{t("overtime.employee")}</label>
                <select value={form.employee_id} onChange={(e) => setForm({ ...form, employee_id: e.target.value })} className="input-field" aria-label={t("overtime.employee")}>
                  <option value="">{t("overtime.selectEmployee")}</option>
                  {employees.map((emp: any) => (
                    <option key={emp.id} value={emp.id}>{emp.name}</option>
                  ))}
                </select>
              </div>
              <div className="input-group">
                <label className="input-label">{t("common.date")}</label>
                <input type="date" value={form.date} onChange={(e) => setForm({ ...form, date: e.target.value })} className="input-field" aria-label={t("common.date")} />
              </div>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="input-group">
                <label className="input-label">{t("overtime.hoursCount")}</label>
                <input type="number" min="0.5" step="0.5" value={form.hours} onChange={(e) => setForm({ ...form, hours: e.target.value })} className="input-field" dir="ltr" aria-label={t("overtime.hoursCount")} />
              </div>
              <div className="input-group">
                <label className="input-label">{t("overtime.rateMultiplier")}</label>
                <select value={form.rate_multiplier} onChange={(e) => setForm({ ...form, rate_multiplier: e.target.value })} className="input-field" aria-label={t("overtime.rateMultiplier")}>
                  <option value="1.5">1.5x</option>
                  <option value="2">2x</option>
                  <option value="2.5">2.5x</option>
                  <option value="3">3x</option>
                </select>
              </div>
            </div>
            <div className="input-group">
              <label className="input-label">{t("overtime.reason")}</label>
              <input type="text" value={form.reason} onChange={(e) => setForm({ ...form, reason: e.target.value })} className="input-field" placeholder={t("overtime.reasonPlaceholder")} aria-label={t("overtime.reason")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("common.notes")}</label>
              <textarea value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} className="input-field" rows={3} placeholder={t("overtime.notesPlaceholder")} aria-label={t("common.notes")} />
            </div>
          </div>
          <div className="flex justify-end gap-3 mt-4">
            <Button variant="ghost" onClick={() => setShowForm(false)}>{t("common.cancel")}</Button>
            <Button icon={<Clock className="w-4 h-4" />} onClick={handleCreate} loading={saving}>{t("overtime.createRecord")}</Button>
          </div>
        </Card>
      )}

      <DataTable columns={columns} data={records} loading={loading} emptyMessage={t("overtime.empty")} />

      <ConfirmDialog
        open={showConfirm}
        title={confirmTitle}
        message={confirmMessage}
        variant={confirmVariant}
        onConfirm={() => { pendingAction(); setShowConfirm(false); }}
        onCancel={() => setShowConfirm(false)}
      />
    </div>
  );
}
