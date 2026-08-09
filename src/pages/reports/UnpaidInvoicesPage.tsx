import { useState, useEffect, useMemo, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import DataTable, { Column } from "@/components/ui/DataTable";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Calendar, AlertTriangle } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { useTranslation } from "react-i18next";

interface UnpaidInvoice {
  id: number;
  inv_no: string;
  date: string;
  customer_name: string;
  total_milli: number;
  paid_milli: number;
  remaining_milli: number;
}

export default function UnpaidInvoicesPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const addNotification = useUIStore((s) => s.addNotification);
  const [data, setData] = useState<UnpaidInvoice[]>([]);
  const [, setLoading] = useState(true);
  const [asOf, setAsOf] = useState(new Date().toISOString().split("T")[0]);

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const result = await invoke("unpaid_invoices_report", { asOf: asOf || null });
      setData(result as UnpaidInvoice[]);
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("reports.loadError") }); }
    finally { setLoading(false); }
  }, [addNotification, asOf]);

  useEffect(() => { loadData(); }, [loadData]);

  const totalRemaining = data.reduce((s, r) => s + r.remaining_milli, 0);

  const columns: Column<UnpaidInvoice>[] = useMemo(() => [
    { key: "inv_no", header: t("invoice.invoiceNo"), render: (r) => <span className="font-mono text-brand-400 cursor-pointer hover:underline" onClick={() => navigate(`/invoices/${r.id}`)}>{r.inv_no || `#${r.id}`}</span> },
    { key: "date", header: t("common.date"), render: (r) => formatDate(r.date) },
    { key: "customer_name", header: t("invoice.customer"), render: (r) => r.customer_name || "—" },
    { key: "total_milli", header: t("reports.total"), align: "left", render: (r) => formatOMR(r.total_milli) },
    { key: "paid_milli", header: t("reports.paidAmount"), align: "left", render: (r) => formatOMR(r.paid_milli) },
    { key: "remaining_milli", header: t("common.remaining"), align: "left", render: (r) => <span className="font-bold text-gold-400">{formatOMR(r.remaining_milli)}</span> },
  ], [t, navigate]);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">{t("reports.unpaidInvoicesTitle")}</h1>
          <p className="page-subtitle">{t("reports.unpaidInvoicesDesc")}</p>
        </div>
        <div className="flex items-center gap-2">
          <Calendar className="w-4 h-4 text-surface-400" />
          <span className="text-sm text-surface-400">{t("reports.asOf")}</span>
          <input type="date" value={asOf} onChange={(e) => setAsOf(e.target.value)} className="input-field text-sm" aria-label={t("reports.asOfDateAria")} />
        </div>
      </div>

      <Card>
        <div className="flex items-center gap-3">
          <div className="p-3 rounded-xl bg-gold-500/10">
            <AlertTriangle className="w-6 h-6 text-gold-400" />
          </div>
          <div>
            <p className="text-sm text-surface-400">{t("reports.totalOutstanding")}</p>
            <p className="text-2xl font-bold gradient-text">{formatOMR(totalRemaining)}</p>
          </div>
          <div className="mr-auto text-left">
            <p className="text-sm text-surface-400">{t("reports.invoiceCount")}</p>
            <p className="text-2xl font-bold">{data.length}</p>
          </div>
        </div>
      </Card>

      <Card>
        <DataTable columns={columns} data={data} compact />
      </Card>
    </div>
  );
}
