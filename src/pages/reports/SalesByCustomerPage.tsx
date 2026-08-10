import { useState, useEffect, useMemo, useCallback } from "react";
import Card from "@/components/ui/Card";
import DataTable, { Column } from "@/components/ui/DataTable";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Calendar, Users } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

interface SalesByCustomer {
  customer_name: string;
  invoice_count: number;
  net_milli: number;
  vat_milli: number;
  total_milli: number;
}

export default function SalesByCustomerPage() {
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
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء تحميل البيانات" }); }
    finally { setLoading(false); }
  }, [addNotification, fromDate, toDate]);

  useEffect(() => { loadData(); }, [loadData]);

  const totalNet = data.reduce((s, r) => s + r.net_milli, 0);
  const totalVat = data.reduce((s, r) => s + r.vat_milli, 0);
  const totalTotal = data.reduce((s, r) => s + r.total_milli, 0);

  const columns: Column<SalesByCustomer>[] = useMemo(() => [
    { key: "customer_name", header: "العميل", render: (r) => <span className="font-bold">{r.customer_name}</span> },
    { key: "invoice_count", header: "عدد الفواتير", align: "center", render: (r) => <span className="badge-info">{r.invoice_count}</span> },
    { key: "net_milli", header: "الصافي", align: "left", render: (r) => formatOMR(r.net_milli) },
    { key: "vat_milli", header: "الضريبة", align: "left", render: (r) => formatOMR(r.vat_milli) },
    { key: "total_milli", header: "الإجمالي", align: "left", render: (r) => <span className="font-bold gradient-text">{formatOMR(r.total_milli)}</span> },
  ], []);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">المبيعات حسب العميل</h1>
          <p className="page-subtitle">تحليل المبيعات لكل عميل</p>
        </div>
        <div className="flex items-center gap-2">
          <Calendar className="w-4 h-4 text-surface-400" />
          <input type="date" value={fromDate} onChange={(e) => setFromDate(e.target.value)} className="input-field text-sm" aria-label="من تاريخ" />
          <span className="text-surface-500">إلى</span>
          <input type="date" value={toDate} onChange={(e) => setToDate(e.target.value)} className="input-field text-sm" aria-label="إلى تاريخ" />
        </div>
      </div>

      <div className="grid grid-cols-4 gap-4">
        <Card>
          <div className="flex items-center gap-3 mb-2">
            <div className="p-2 rounded-lg bg-brand-500/10 text-brand-400"><Users className="w-5 h-5" /></div>
            <p className="text-sm text-surface-400">عدد العملاء</p>
          </div>
          <p className="text-2xl font-bold gradient-text">{data.length}</p>
        </Card>
        <Card><p className="text-sm text-surface-400 mb-1">صافي المبيعات</p><p className="text-xl font-bold gradient-text">{formatOMR(totalNet)}</p></Card>
        <Card><p className="text-sm text-surface-400 mb-1">الضريبة</p><p className="text-xl font-bold">{formatOMR(totalVat)}</p></Card>
        <Card><p className="text-sm text-surface-400 mb-1">الإجمالي</p><p className="text-xl font-bold gradient-text">{formatOMR(totalTotal)}</p></Card>
      </div>

      <Card>
        <DataTable columns={columns} data={data} compact />
      </Card>
    </div>
  );
}
