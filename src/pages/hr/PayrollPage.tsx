import { useState, useEffect, useMemo, useCallback } from "react";
import { useTranslation } from "react-i18next";
import DataTable, { Column } from "@/components/ui/DataTable";
import Badge from "@/components/ui/Badge";
import Button from "@/components/ui/Button";
import Card from "@/components/ui/Card";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus, Play, Eye } from "lucide-react";
import ConfirmDialog from "@/components/ui/ConfirmDialog";
import { useUIStore } from "@/stores/uiStore";
import type { PayrollRun } from "@/types";

export default function PayrollPage() {
  const { t } = useTranslation();
  const addNotification = useUIStore((s) => s.addNotification);
  const [runs, setRuns] = useState<PayrollRun[]>([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [saving, setSaving] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);
  const [form, setForm] = useState({ period_start: "", period_end: "" });

  const loadRuns = useCallback(async () => {
    setLoading(true);
    try {
      const d = await invoke("list_payroll_runs");
      setRuns(d as PayrollRun[]);
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("payrollPage.errorLoad") }); }
    finally { setLoading(false); }
  }, [addNotification, t]);

  useEffect(() => { loadRuns(); }, [loadRuns]);

  const handleCreate = async () => {
    setSaving(true);
    try {
      await invoke("create_payroll_run", { input: form });
      setShowForm(false);
      setForm({ period_start: "", period_end: "" });
      loadRuns();
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("payrollPage.errorSave") }); }
    finally { setSaving(false); }
  };

  const totalPaid = runs.filter((r) => r.status === "paid").length;
  const pendingRuns = runs.filter((r) => r.status === "pending" || r.status === "draft").length;

  const columns: Column<PayrollRun>[] = useMemo(() => [
    { key: "run_no", header: t("payrollPage.runNo"), sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.run_no || "—"}</span> },
    { key: "period", header: t("payrollPage.period"), render: (r) => `${formatDate(r.period_start)} — ${formatDate(r.period_end)}` },
    { key: "total_gross_milli", header: t("payrollPage.grossTotal"), sortable: true, align: "left", render: (r) => formatOMR(r.total_gross_milli) },
    { key: "total_deductions_milli", header: t("payrollPage.deductions"), align: "left", render: (r) => formatOMR(r.total_deductions_milli) },
    { key: "total_net_milli", header: t("print.net"), sortable: true, align: "left", render: (r) => <span className="font-bold text-gold-400">{formatOMR(r.total_net_milli)}</span> },
    { key: "status", header: t("common.status"), render: (r) => {
      const map: Record<string, { label: string; variant: any }> = {
        draft: { label: t("badge.draft"), variant: "default" },
        pending: { label: t("badge.pending"), variant: "warning" },
        processing: { label: t("badge.processing"), variant: "info" },
        paid: { label: t("common.paid"), variant: "success" },
      };
      const s = map[r.status] || { label: r.status, variant: "default" };
      return <Badge variant={s.variant}>{s.label}</Badge>;
    }},
    { key: "actions", header: "", render: (r) => (
      <div className="flex items-center gap-1">
        <button className="p-1.5 text-surface-400 hover:text-brand-400 transition-colors rounded-lg hover:bg-surface-700/50" title={t("payrollPage.viewDetails")}>
          <Eye className="w-4 h-4" />
        </button>
        {(r.status === "draft" || r.status === "pending") && (
          <button className="p-1.5 text-surface-400 hover:text-gold-400 transition-colors rounded-lg hover:bg-surface-700/50" title={t("payrollPage.process")}>
            <Play className="w-4 h-4" />
          </button>
        )}
      </div>
    )},
  ], [t]);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">{t("payrollPage.title")}</h1>
          <p className="page-subtitle">{t("payrollPage.subtitle", { count: runs.length })}</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => setShowForm(true)}>{t("payrollPage.newRun")}</Button>
      </div>

      <div className="grid grid-cols-4 gap-4">
        <Card className="text-center">
          <p className="text-2xl font-bold gradient-text">{runs.length}</p>
          <p className="text-xs text-surface-400">{t("payrollPage.totalRuns")}</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-emerald-400">{totalPaid}</p>
          <p className="text-xs text-surface-400">{t("common.paid")}</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-gold-400">{pendingRuns}</p>
          <p className="text-xs text-surface-400">{t("badge.pending")}</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-brand-400">{runs.filter((r) => r.status === "processing").length}</p>
          <p className="text-xs text-surface-400">{t("badge.processing")}</p>
        </Card>
      </div>

      {showForm && (
        <Card>
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-bold text-white">{t("payrollPage.newRun")}</h2>
            <button onClick={() => setShowForm(false)} className="text-surface-400 hover:text-white text-xl">&times;</button>
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div className="input-group">
              <label className="input-label">{t("payrollPage.startDate")}</label>
              <input type="date" value={form.period_start} onChange={(e) => setForm({ ...form, period_start: e.target.value })} className="input-field" aria-label={t("payrollPage.startDate")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("payrollPage.endDate")}</label>
              <input type="date" value={form.period_end} onChange={(e) => setForm({ ...form, period_end: e.target.value })} className="input-field" aria-label={t("payrollPage.endDate")} />
            </div>
          </div>
          <div className="flex justify-end gap-3 mt-4">
            <Button variant="ghost" onClick={() => setShowForm(false)}>{t("common.cancel")}</Button>
            <Button icon={<Play className="w-4 h-4" />} onClick={() => setShowConfirm(true)} loading={saving}>{t("payrollPage.createRun")}</Button>
          </div>
        </Card>
      )}

      <DataTable columns={columns} data={runs} loading={loading} emptyMessage={t("payrollPage.empty")} />

      <ConfirmDialog
        open={showConfirm}
        title={t("payrollPage.confirmTitle")}
        message={t("payrollPage.confirmMessage")}
        variant="warning"
        onConfirm={() => { handleCreate(); setShowConfirm(false); }}
        onCancel={() => setShowConfirm(false)}
      />
    </div>
  );
}
