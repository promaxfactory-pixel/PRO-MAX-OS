import { useState, useEffect, useMemo, useCallback } from "react";
import DataTable, { Column } from "@/components/ui/DataTable";
import Button from "@/components/ui/Button";
import Card, { StatCard } from "@/components/ui/Card";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus, Save, Coins, ArrowRight } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { useTranslation } from "react-i18next";

interface PettyCashAccount {
  code: string;
  name: string;
  responsible: string;
  role: string;
  spending_limit_milli: number;
  balance_milli: number;
  status: string;
  notes: string;
}

export default function PettyCashPage() {
  const { t } = useTranslation();
  const addNotification = useUIStore((s) => s.addNotification);
  const [accounts, setAccounts] = useState<PettyCashAccount[]>([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [submitting, setSubmitting] = useState(false);

  const [name, setName] = useState("");
  const [code, setCode] = useState("");
  const [responsible, setResponsible] = useState("");
  const [role, setRole] = useState("");
  const [spendingLimitMilli, setSpendingLimitMilli] = useState(0);
  const [notes, setNotes] = useState("");

  const loadAccounts = useCallback(async () => {
    setLoading(true);
    try { const d = await invoke("list_petty_cash_accounts"); setAccounts(d as PettyCashAccount[]); }
    catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("pettyCash.loadError") }); }
    finally { setLoading(false); }
  }, [addNotification]);

  useEffect(() => { loadAccounts(); }, [loadAccounts]);

  const resetForm = () => {
    setName("");
    setCode("");
    setResponsible("");
    setRole("");
    setSpendingLimitMilli(0);
    setNotes("");
  };

  const handleSubmit = async () => {
    if (!name || !code || !responsible) return;
    setSubmitting(true);
    try {
      await invoke("create_petty_cash_account", {
        input: {
          name,
          code,
          responsible,
          role: role || null,
          spending_limit_milli: spendingLimitMilli || 0,
          notes: notes || null,
        },
      });
      resetForm();
      setShowForm(false);
      loadAccounts();
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("pettyCash.saveError") });
    } finally {
      setSubmitting(false);
    }
  };

  const totalAccounts = accounts.length;
  const totalBalance = accounts.reduce((s: number, a: PettyCashAccount) => s + (a.balance_milli || 0), 0);
  const activeCount = accounts.filter((a) => a.status === "active").length;

  const columns: Column<PettyCashAccount>[] = useMemo(() => [
    { key: "code", header: t("pettyCash.code"), sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.code}</span> },
    { key: "name", header: t("common.name"), sortable: true, render: (r) => <span className="text-white font-medium">{r.name}</span> },
    { key: "responsible", header: t("pettyCash.responsible"), render: (r) => <span className="text-surface-300">{r.responsible}</span> },
    { key: "spending_limit_milli", header: t("pettyCash.spendingLimit"), align: "left", render: (r) => (
      <span className="text-surface-300 font-mono">{formatOMR(r.spending_limit_milli)}</span>
    )},
    { key: "balance_milli", header: t("common.balance"), sortable: true, align: "left", render: (r) => (
      <span className="font-bold text-gold-400 font-mono">{formatOMR(r.balance_milli)}</span>
    )},
    { key: "status", header: t("common.status"), render: (r) => (
      <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
        r.status === "active" ? "bg-emerald-500/20 text-emerald-400" :
        r.status === "closed" ? "bg-red-500/20 text-red-400" :
        "bg-surface-600 text-surface-300"
      }`}>{r.status === "active" ? t("pettyCash.active") : r.status === "closed" ? t("pettyCash.closed") : r.status || "—"}</span>
    )},
  ], [t]);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">{t("pettyCash.title")}</h1>
          <p className="page-subtitle">{t("pettyCash.subtitle", { count: totalAccounts })}</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => setShowForm(!showForm)}>
          {showForm ? t("pettyCash.hide") : t("pettyCash.newAccount")}
        </Button>
      </div>

      <div className="grid grid-cols-3 gap-4">
        <StatCard title={t("pettyCash.accountCount")} value={String(totalAccounts)} icon={<Coins className="w-6 h-6" />} />
        <StatCard title={t("pettyCash.totalBalances")} value={formatOMR(totalBalance)} icon={<Coins className="w-6 h-6" />} />
        <StatCard title={t("pettyCash.activeAccounts")} value={String(activeCount)} icon={<Coins className="w-6 h-6" />} />
      </div>

      {showForm && (
        <Card className="p-6">
          <h2 className="text-lg font-semibold text-white mb-4">{t("pettyCash.formTitle")}</h2>
          <div className="grid grid-cols-2 gap-6">
            <div className="input-group">
              <label className="input-label">{t("pettyCash.accountNameLabel")}</label>
              <input type="text" className="input-field" value={name} onChange={(e) => setName(e.target.value)} placeholder={t("pettyCash.accountNamePlaceholder")} aria-label={t("pettyCash.accountNameAria")} />
            </div>

            <div className="input-group">
              <label className="input-label">{t("pettyCash.codeLabel")}</label>
              <input type="text" className="input-field" value={code} onChange={(e) => setCode(e.target.value)} placeholder={t("pettyCash.codePlaceholder")} aria-label={t("pettyCash.codeAria")} />
            </div>

            <div className="input-group">
              <label className="input-label">{t("pettyCash.responsibleLabel")}</label>
              <input type="text" className="input-field" value={responsible} onChange={(e) => setResponsible(e.target.value)} placeholder={t("pettyCash.responsiblePlaceholder")} aria-label={t("pettyCash.responsibleAria")} />
            </div>

            <div className="input-group">
              <label className="input-label">{t("pettyCash.role")}</label>
              <input type="text" className="input-field" value={role} onChange={(e) => setRole(e.target.value)} placeholder={t("pettyCash.rolePlaceholder")} aria-label={t("pettyCash.roleAria")} />
            </div>

            <div className="input-group">
              <label className="input-label">{t("pettyCash.spendingLimitMilli")}</label>
              <input type="number" className="input-field" min={0} value={spendingLimitMilli} onChange={(e) => setSpendingLimitMilli(Number(e.target.value))} aria-label={t("pettyCash.spendingLimit")} />
            </div>

            <div className="input-group">
              <label className="input-label">{t("pettyCash.notes")}</label>
              <input type="text" className="input-field" value={notes} onChange={(e) => setNotes(e.target.value)} placeholder={t("pettyCash.notesPlaceholder")} aria-label={t("pettyCash.notesAria")} />
            </div>
          </div>

          <div className="flex justify-end gap-3 mt-6">
            <Button variant="ghost" icon={<ArrowRight className="w-4 h-4" />} onClick={() => { resetForm(); setShowForm(false); }}>{t("pettyCash.cancel")}</Button>
            <Button icon={<Save className="w-4 h-4" />} onClick={handleSubmit} disabled={submitting}>
              {submitting ? t("pettyCash.saving") : t("pettyCash.saveAccount")}
            </Button>
          </div>
        </Card>
      )}

      <DataTable columns={columns} data={accounts} loading={loading} emptyMessage={t("pettyCash.empty")} />
    </div>
  );
}
