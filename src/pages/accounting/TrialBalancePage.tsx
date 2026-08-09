import { useState, useEffect, useMemo } from "react";
import DataTable, { Column } from "@/components/ui/DataTable";
import Card from "@/components/ui/Card";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { useUIStore } from "../../stores/uiStore";
import { useTranslation } from "react-i18next";

interface TrialBalanceRow {
  account_code: string;
  account_name: string;
  debit_milli: number;
  credit_milli: number;
}

export default function TrialBalancePage() {
  const { t } = useTranslation();
  const { addNotification } = useUIStore();
  const [data, setData] = useState<TrialBalanceRow[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => { invoke("get_trial_balance").then((d: unknown) => setData(d as TrialBalanceRow[])).catch((e: unknown) => addNotification({ title: t("common.error"), message: String(e), type: 'error' })).finally(() => setLoading(false)); }, []);

  const totalDebit = data.reduce((s, r) => s + (r.debit_milli || 0), 0);
  const totalCredit = data.reduce((s, r) => s + (r.credit_milli || 0), 0);

  const columns: Column<TrialBalanceRow>[] = useMemo(() => [
    { key: "account_code", header: t("customer.code"), render: (r) => <span className="font-mono text-brand-400">{r.account_code}</span> },
    { key: "account_name", header: t("common.name") },
    { key: "debit_milli", header: t("trialBalance.debit"), align: "left", render: (r) => r.debit_milli > 0 ? formatOMR(r.debit_milli) : "—" },
    { key: "credit_milli", header: t("trialBalance.credit"), align: "left", render: (r) => r.credit_milli > 0 ? formatOMR(r.credit_milli) : "—" },
  ], [t]);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div><h1 className="page-title">{t("accounting.trialBalance")}</h1></div>
      </div>
      <div className="grid grid-cols-2 gap-4 mb-6">
        <Card><div className="text-center"><p className="text-3xl font-bold gradient-text">{formatOMR(totalDebit)}</p><p className="text-xs text-surface-400">{t("print.totalDebit")}</p></div></Card>
        <Card><div className="text-center"><p className="text-3xl font-bold gradient-text">{formatOMR(totalCredit)}</p><p className="text-xs text-surface-400">{t("print.totalCredit")}</p></div></Card>
      </div>
      <DataTable columns={columns} data={data} loading={loading} emptyMessage={t("common.noData")} />
      {totalDebit !== totalCredit && (
        <div className="p-4 bg-red-500/10 border border-red-500/30 rounded-xl text-red-400 text-sm text-center">
          {t("trialBalance.unbalanced", { diff: formatOMR(Math.abs(totalDebit - totalCredit)) })}
        </div>
      )}
    </div>
  );
}
