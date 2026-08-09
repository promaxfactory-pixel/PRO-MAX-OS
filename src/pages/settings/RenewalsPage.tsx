import { useState, useEffect, useMemo, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import DataTable, { Column } from "@/components/ui/DataTable";
import Badge from "@/components/ui/Badge";
import Button from "@/components/ui/Button";
import Card from "@/components/ui/Card";
import { formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus, FileCheck, ShieldAlert } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

interface Renewal {
  id: number;
  name: string;
  category: string;
  authority: string;
  issue_date: string;
  expiry_date: string;
  cost_milli: number;
  responsible: string;
  alert_days: number;
  notes: string;
  status: string;
}

export default function RenewalsPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const addNotification = useUIStore((s) => s.addNotification);
  const [renewals, setRenewals] = useState<Renewal[]>([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [saving, setSaving] = useState(false);
  const [form, setForm] = useState({
    name: "",
    category: "",
    authority: "",
    issue_date: "",
    expiry_date: "",
    cost_milli: "",
    responsible: "",
    alert_days: "30",
    notes: "",
  });

  const loadRenewals = useCallback(async () => {
    setLoading(true);
    try {
      const d = await invoke("list_renewals");
      setRenewals(d as Renewal[]);
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("settings.renewals.loadError") }); }
    finally { setLoading(false); }
  }, [t, addNotification]);

  useEffect(() => { loadRenewals(); }, [loadRenewals]);

  const handleCreate = async () => {
    setSaving(true);
    try {
      await invoke("create_renewal", {
        input: {
          name: form.name,
          category: form.category,
          authority: form.authority,
          issue_date: form.issue_date,
          expiry_date: form.expiry_date,
          cost_milli: Number(form.cost_milli),
          responsible: form.responsible,
          alert_days: Number(form.alert_days),
          notes: form.notes,
        },
      });
      setShowForm(false);
      setForm({ name: "", category: "", authority: "", issue_date: "", expiry_date: "", cost_milli: "", responsible: "", alert_days: "30", notes: "" });
      loadRenewals();
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("settings.renewals.saveError") }); }
    finally { setSaving(false); }
  };

  const getDaysUntilExpiry = (expiryDate: string): number => {
    if (!expiryDate) return Infinity;
    const now = new Date();
    now.setHours(0, 0, 0, 0);
    const exp = new Date(expiryDate);
    exp.setHours(0, 0, 0, 0);
    return Math.ceil((exp.getTime() - now.getTime()) / (1000 * 60 * 60 * 24));
  };

  const getStatus = (r: Renewal): string => {
    const days = getDaysUntilExpiry(r.expiry_date);
    if (r.status === "cancelled") return "cancelled";
    if (days < 0) return "expired";
    if (days <= (r.alert_days || 30)) return "expiring";
    return "active";
  };

  const statusMap: Record<string, { label: string; variant: any }> = {
    active: { label: t("settings.renewals.status.active"), variant: "success" },
    expiring: { label: t("settings.renewals.status.expiring"), variant: "warning" },
    expired: { label: t("settings.renewals.status.expired"), variant: "danger" },
    cancelled: { label: t("settings.renewals.status.cancelled"), variant: "default" },
  };

  const totalRenewals = renewals.length;
  const activeCount = renewals.filter((r) => getStatus(r) === "active").length;
  const expiringSoon = renewals.filter((r) => getStatus(r) === "expiring").length;
  const expiredCount = renewals.filter((r) => getStatus(r) === "expired").length;

  const columns: Column<Renewal>[] = useMemo(() => [
    { key: "name", header: t("settings.renewals.name"), sortable: true, render: (r) => <span className="font-medium">{r.name}</span> },
    { key: "category", header: t("settings.renewals.category"), sortable: true, render: (r) => r.category || "—" },
    { key: "authority", header: t("settings.renewals.authority"), sortable: true, render: (r) => r.authority || "—" },
    { key: "expiry_date", header: t("license.expiry"), sortable: true, render: (r) => {
      const days = getDaysUntilExpiry(r.expiry_date);
      return (
        <div className="flex items-center gap-2">
          <span>{formatDate(r.expiry_date)}</span>
          {days >= 0 && days <= (r.alert_days || 30) && (
            <span className={`text-xs font-medium ${days < 0 ? "text-red-400" : days <= 7 ? "text-red-400" : "text-yellow-400"}`}>
              {days < 0 ? t("settings.renewals.expiredSince", { days: Math.abs(days) }) : t("settings.renewals.daysRemaining", { days })}
            </span>
          )}
          {days < 0 && <ShieldAlert className="w-3.5 h-3.5 text-red-400" />}
        </div>
      );
    }},
    { key: "responsible", header: t("settings.renewals.responsible"), render: (r) => r.responsible || "—" },
    { key: "status", header: t("common.status"), render: (r) => {
      const s = statusMap[getStatus(r)] || { label: r.status, variant: "default" };
      return <Badge variant={s.variant}>{s.label}</Badge>;
    }},
  ], [t]);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">{t("settings.renewals.title")}</h1>
          <p className="page-subtitle">{t("settings.renewals.subtitle", { count: totalRenewals })}</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => setShowForm(true)}>{t("settings.renewals.newRenewal")}</Button>
      </div>

      <div className="grid grid-cols-4 gap-4">
        <Card className="text-center">
          <p className="text-2xl font-bold gradient-text">{totalRenewals}</p>
          <p className="text-xs text-surface-400">{t("settings.renewals.totalRenewals")}</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-emerald-400">{activeCount}</p>
          <p className="text-xs text-surface-400">{t("settings.renewals.status.active")}</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-yellow-400">{expiringSoon}</p>
          <p className="text-xs text-surface-400">{t("settings.renewals.status.expiring")}</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-red-400">{expiredCount}</p>
          <p className="text-xs text-surface-400">{t("settings.renewals.status.expired")}</p>
        </Card>
      </div>

      {showForm && (
        <Card>
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-bold text-white">{t("settings.renewals.newRenewal")}</h2>
            <button onClick={() => setShowForm(false)} className="text-surface-400 hover:text-white text-xl">&times;</button>
          </div>
          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="input-group">
                <label className="input-label">{t("settings.renewals.name")}</label>
                <input type="text" value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} className="input-field" aria-label={t("settings.renewals.name")} />
              </div>
              <div className="input-group">
                <label className="input-label">{t("settings.renewals.category")}</label>
                <input type="text" value={form.category} onChange={(e) => setForm({ ...form, category: e.target.value })} className="input-field" placeholder={t("settings.renewals.categoryPlaceholder")} aria-label={t("settings.renewals.category")} />
              </div>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="input-group">
                <label className="input-label">{t("settings.renewals.authority")}</label>
                <input type="text" value={form.authority} onChange={(e) => setForm({ ...form, authority: e.target.value })} className="input-field" aria-label={t("settings.renewals.authority")} />
              </div>
              <div className="input-group">
                <label className="input-label">{t("settings.renewals.responsible")}</label>
                <input type="text" value={form.responsible} onChange={(e) => setForm({ ...form, responsible: e.target.value })} className="input-field" aria-label={t("settings.renewals.responsible")} />
              </div>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="input-group">
                <label className="input-label">{t("settings.renewals.issueDate")}</label>
                <input type="date" value={form.issue_date} onChange={(e) => setForm({ ...form, issue_date: e.target.value })} className="input-field" aria-label={t("settings.renewals.issueDate")} />
              </div>
              <div className="input-group">
                <label className="input-label">{t("license.expiry")}</label>
                <input type="date" value={form.expiry_date} onChange={(e) => setForm({ ...form, expiry_date: e.target.value })} className="input-field" aria-label={t("license.expiry")} />
              </div>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="input-group">
                <label className="input-label">{t("settings.renewals.cost")}</label>
                <input type="number" value={form.cost_milli} onChange={(e) => setForm({ ...form, cost_milli: e.target.value })} className="input-field" dir="ltr" aria-label={t("settings.renewals.cost")} />
              </div>
              <div className="input-group">
                <label className="input-label">{t("settings.renewals.alertDays")}</label>
                <input type="number" value={form.alert_days} onChange={(e) => setForm({ ...form, alert_days: e.target.value })} className="input-field" dir="ltr" aria-label={t("settings.renewals.alertDays")} />
              </div>
            </div>
            <div className="input-group">
              <label className="input-label">{t("common.notes")}</label>
              <input type="text" value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} className="input-field" aria-label={t("common.notes")} />
            </div>
          </div>
          <div className="flex justify-end gap-3 mt-4">
            <Button variant="ghost" onClick={() => setShowForm(false)}>{t("common.cancel")}</Button>
            <Button icon={<FileCheck className="w-4 h-4" />} onClick={handleCreate} loading={saving}>{t("settings.renewals.createRenewal")}</Button>
          </div>
        </Card>
      )}

      <DataTable
        columns={columns}
        data={renewals}
        loading={loading}
        emptyMessage={t("settings.renewals.empty")}
        onRowClick={(r) => navigate(`/renewals/${r.id}`)}
      />
    </div>
  );
}
