import { useState, useEffect, useMemo } from "react";
import DataTable, { Column, type BadgeVariant } from "@/components/ui/DataTable";
import Badge from "@/components/ui/Badge";
import { invoke } from "@tauri-apps/api/core";
import { useUIStore } from "../../stores/uiStore";
import { Account } from "@/types";
import { useTranslation } from "react-i18next";

export default function AccountsPage() {
  const { t } = useTranslation();
  const { addNotification } = useUIStore();
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => { invoke("list_accounts").then((d) => setAccounts(d as Account[]))      .catch((e: unknown) => addNotification({ title: t("common.error"), message: String(e), type: 'error' })).finally(() => setLoading(false)); }, []);

  const typeColors: Record<string, BadgeVariant> = { asset: 'success', liability: 'danger', equity: 'info', revenue: 'gold', expense: 'warning' };
  const typeLabels: Record<string, string> = { asset: t("accounts.typeAsset"), liability: t("accounts.typeLiability"), equity: t("accounts.typeEquity"), revenue: t("accounts.typeRevenue"), expense: t("accounts.typeExpense") };

  const columns: Column<any>[] = useMemo(() => [
    { key: "code", header: t("customer.code"), sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.code}</span> },
    { key: "name_ar", header: t("accounts.nameArabic"), sortable: true },
    { key: "name_en", header: t("accounts.nameEnglish"), sortable: true },
    { key: "type", header: t("common.type"), sortable: true, render: (r) => <Badge variant={typeColors[r.type] || "default"}>{typeLabels[r.type] || r.type}</Badge> },
    { key: "parent", header: t("accounts.parent"), render: (r) => r.parent || "—" },
  ], [t]);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div><h1 className="page-title">{t("accounting.accounts")}</h1><p className="page-subtitle">{t("accounts.count", { count: accounts.length })}</p></div>
      </div>
      <DataTable columns={columns} data={accounts} loading={loading} emptyMessage={t("cashBank.empty")} />
    </div>
  );
}
