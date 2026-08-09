import { useState, useEffect, useMemo, useCallback } from "react";
import Card from "@/components/ui/Card";
import DataTable, { Column } from "@/components/ui/DataTable";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Calendar, Users } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { useTranslation } from "react-i18next";

interface SalesByCustomer {
  customer_name: string;
  invoice_count: number;
  net_milli: number;
  vat_milli: number;
  total_milli: number;
}

export default function SalesByCustomerPage() {
  const { t } = useTranslation();
  const addNotification = useUIStore((s) => s.addNotification);
  const [data, setData] = useState<SalesByCustomer[]>([]);
  const [, setLoading] = useState(true);
  const [fromDate, setFromDate] = useState("");
  const [toDate, setToDate] = useState("");

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const result = await invoke("sales_by_customer_report", {
        fromDate: fromDate || null,
        toDate: toDate || null,
      });
      setData(result as SalesByCustomer[]);
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("reports.loadError") }); }
    finally { setLoading(false); }
  }, [addNotification, fromDate, toDate]);

  useEffect(() => { loadData(); }, [loadData]);

  const totalNet = data.reduce((s, r) => s + r.net_milli, 0);
  const totalVat = data.reduce((s, r) => s + r.vat_milli, 0);
  const totalTotal = data.reduce((s, r) => s + r.total_milli, 0);

  const columns: Column<SalesByCustomer>[] = useMemo(() => [
    { key: "customer_name", header: t("invoice.customer"), render: (r) => <span className="font-bold">{r.customer_name}</span> },
    { key: "invoice_count", header: t("reports.invoiceCount"), align: "center", render: (r) => <span className="badge-info">{r.invoice_count}</span> },
    { key: "net_milli", header: t("reports.netColumn"), align: "left", render: (r) => formatOMR(r.net_milli) },
    { key: "vat_milli", header: t("common.vat"), align: "left", render: (r) => formatOMR(r.vat_milli) },
    { key: "total_milli", header: t("reports.total"), align: "left", render: (r) => <span className="font-bold gradient-text">{formatOMR(r.total_milli)}</span> },
  ], [t]);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">{t("reports.salesByCustomer")}</h1>
          <p className="page-subtitle">{t("reports.salesByCustomerSubtitle")}</p>
        </div>
        <div className="flex items-center gap-2">
          <Calendar className="w-4 h-4 text-surface-400" />
          <input type="date" value={fromDate} onChange={(e) => setFromDate(e.target.value)} className="input-field text-sm" aria-label={t("common.fromDate")} />
          <span className="text-surface-500">{t("stockTransfers.to")}</span>
          <input type="date" value={toDate} onChange={(e) => setToDate(e.target.value)} className="input-field text-sm" aria-label={t("common.toDate")} />
        </div>
      </div>

      <div className="grid grid-cols-4 gap-4">
        <Card>
          <div className="flex items-center gap-3 mb-2">
            <div className="p-2 rounded-lg bg-brand-500/10 text-brand-400"><Users className="w-5 h-5" /></div>
            <p className="text-sm text-surface-400">{t("reports.customerCount")}</p>
          </div>
          <p className="text-2xl font-bold gradient-text">{data.length}</p>
        </Card>
        <Card><p className="text-sm text-surface-400 mb-1">{t("reports.netSales")}</p><p className="text-xl font-bold gradient-text">{formatOMR(totalNet)}</p></Card>
        <Card><p className="text-sm text-surface-400 mb-1">{t("common.vat")}</p><p className="text-xl font-bold">{formatOMR(totalVat)}</p></Card>
        <Card><p className="text-sm text-surface-400 mb-1">{t("reports.total")}</p><p className="text-xl font-bold gradient-text">{formatOMR(totalTotal)}</p></Card>
      </div>

      <Card>
        <DataTable columns={columns} data={data} compact />
      </Card>
    </div>
  );
}
