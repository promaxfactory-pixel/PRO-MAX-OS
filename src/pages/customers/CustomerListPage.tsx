import { useState, useEffect, useMemo, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import DataTable, { Column } from "@/components/ui/DataTable";
import Button from "@/components/ui/Button";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { useTranslation } from "react-i18next";
import { Customer } from "@/types";

export default function CustomerListPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const addNotification = useUIStore((s) => s.addNotification);
  const [customers, setCustomers] = useState<Customer[]>([]);
  const [loading, setLoading] = useState(true);

  const loadCustomers = useCallback(async () => {
    setLoading(true);
    try { const d = await invoke("list_customers"); setCustomers(d as Customer[]); }
    catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("customer.loadError") }); }
    finally { setLoading(false); }
  }, [addNotification]);

  useEffect(() => { loadCustomers(); }, [loadCustomers]);

  const columns: Column<any>[] = useMemo(() => [
    { key: "code", header: t("customer.code"), sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.code || "—"}</span> },
    { key: "name", header: t("customer.name"), sortable: true, render: (r) => <span className="font-medium">{r.name}</span> },
    { key: "phone", header: t("customer.phone"), render: (r) => r.phone || "—" },
    { key: "email", header: t("customer.email"), render: (r) => r.email || "—" },
    { key: "balance_milli", header: t("customer.balance"), sortable: true, align: "left", render: (r) => <span className={`font-bold ${r.balance_milli > 0 ? 'text-gold-400' : r.balance_milli < 0 ? 'text-red-400' : ''}`}>{formatOMR(r.balance_milli)}</span> },
    { key: "credit_limit_milli", header: t("customer.creditLimitLabel"), align: "left", render: (r) => formatOMR(r.credit_limit_milli) },
  ], [t]);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">{t("customer.title")}</h1>
          <p className="page-subtitle">{t("customer.subtitle", { count: customers.length })}</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => navigate('/customers/new')}>{t("customer.newCustomer")}</Button>
      </div>
      <DataTable columns={columns} data={customers} loading={loading}
        onRowClick={(r) => navigate(`/customers/${r.id}`)} emptyMessage={t("customer.empty")} />
    </div>
  );
}
