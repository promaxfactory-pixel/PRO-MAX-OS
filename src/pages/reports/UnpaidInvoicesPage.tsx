import { useState, useEffect, useMemo, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import DataTable, { Column } from "@/components/ui/DataTable";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Calendar, AlertTriangle } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

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
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء تحميل البيانات" }); }
    finally { setLoading(false); }
  }, [addNotification, asOf]);

  useEffect(() => { loadData(); }, [loadData]);

  const totalRemaining = data.reduce((s, r) => s + r.remaining_milli, 0);

  const columns: Column<UnpaidInvoice>[] = useMemo(() => [
    { key: "inv_no", header: "رقم الفاتورة", render: (r) => <span className="font-mono text-brand-400 cursor-pointer hover:underline" onClick={() => navigate(`/invoices/${r.id}`)}>{r.inv_no || `#${r.id}`}</span> },
    { key: "date", header: "التاريخ", render: (r) => formatDate(r.date) },
    { key: "customer_name", header: "العميل", render: (r) => r.customer_name || "—" },
    { key: "total_milli", header: "الإجمالي", align: "left", render: (r) => formatOMR(r.total_milli) },
    { key: "paid_milli", header: "المدفوع", align: "left", render: (r) => formatOMR(r.paid_milli) },
    { key: "remaining_milli", header: "المتبقي", align: "left", render: (r) => <span className="font-bold text-gold-400">{formatOMR(r.remaining_milli)}</span> },
  ], []);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">الفواتير غير المحصلة</h1>
          <p className="page-subtitle">جميع الفواتير التي لم تُسدد بالكامل</p>
        </div>
        <div className="flex items-center gap-2">
          <Calendar className="w-4 h-4 text-surface-400" />
          <span className="text-sm text-surface-400">حتى</span>
          <input type="date" value={asOf} onChange={(e) => setAsOf(e.target.value)} className="input-field text-sm" aria-label="حتى تاريخ" />
        </div>
      </div>

      <Card>
        <div className="flex items-center gap-3">
          <div className="p-3 rounded-xl bg-gold-500/10">
            <AlertTriangle className="w-6 h-6 text-gold-400" />
          </div>
          <div>
            <p className="text-sm text-surface-400">إجمالي المبالغ المحصلة</p>
            <p className="text-2xl font-bold gradient-text">{formatOMR(totalRemaining)}</p>
          </div>
          <div className="mr-auto text-left">
            <p className="text-sm text-surface-400">عدد الفواتير</p>
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
