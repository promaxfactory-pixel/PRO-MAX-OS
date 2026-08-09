import { useState, useEffect, useMemo, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import DataTable, { Column } from "@/components/ui/DataTable";
import { StatusBadge } from "@/components/ui/Badge";
import Button from "@/components/ui/Button";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useUIStore } from "@/stores/uiStore";
import { SalesInvoice } from "@/types";

export default function InvoiceListPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const addNotification = useUIStore((s) => s.addNotification);
  const [invoices, setInvoices] = useState<SalesInvoice[]>([]);
  const [loading, setLoading] = useState(true);
  const [statusFilter, setStatusFilter] = useState("all");

  const loadInvoices = useCallback(async () => {
    setLoading(true);
    try {
      const data = await invoke("list_invoices");
      setInvoices(data as SalesInvoice[]);
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("invoice.loadError") }); }
    finally { setLoading(false); }
  }, [addNotification, t]);

  useEffect(() => { loadInvoices(); }, [loadInvoices]);

  const filtered = statusFilter === "all" ? invoices : invoices.filter(i => i.status?.toLowerCase() === statusFilter);

  const columns: Column<any>[] = useMemo(() => [
    { key: "inv_no", header: t("invoice.invoiceNo"), sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.inv_no}</span> },
    { key: "date", header: t("common.date"), sortable: true, render: (r) => formatDate(r.date) },
    { key: "customer_name", header: t("invoice.customer"), sortable: true, render: (r) => r.customer_name || "—" },
    { key: "total_milli", header: t("invoice.total"), sortable: true, align: "left", render: (r) => <span className="font-bold">{formatOMR(r.total_milli)}</span> },
    { key: "paid_milli", header: t("common.paid"), sortable: true, align: "left", render: (r) => formatOMR(r.paid_milli) },
    { key: "status", header: t("invoice.status"), sortable: true, render: (r) => <StatusBadge status={r.status} /> },
  ], [t]);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">{t("invoice.title")}</h1>
          <p className="page-subtitle">{t("invoice.subtitle", { count: invoices.length })}</p>
        </div>
        <div className="flex items-center gap-3">
          <Button onClick={() => navigate('/invoices/new')} icon={<Plus className="w-4 h-4" />}>{t("invoice.newInvoice")}</Button>
        </div>
      </div>

      <div className="flex items-center gap-2 mb-4">
        {["all", "draft", "posted", "void"].map((s) => (
          <button key={s} onClick={() => setStatusFilter(s)}
            className={`px-4 py-1.5 rounded-full text-xs font-medium transition-all ${statusFilter === s ? 'bg-brand-800 text-gold-400 border border-brand-500/30' : 'bg-surface-800 text-surface-400 border border-surface-700 hover:text-white'}`}>
            {s === "all" ? t("common.all") : s === "draft" ? t("invoice.draft") : s === "posted" ? t("invoice.posted") : t("invoice.void")}
          </button>
        ))}
      </div>

      <DataTable columns={columns} data={filtered} loading={loading}
        onRowClick={(r) => navigate(`/invoices/${r.id}`)}
        emptyMessage={t("invoice.empty")} />
    </div>
  );
}
