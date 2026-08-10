import { useState, useEffect, useMemo, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import DataTable, { Column } from "@/components/ui/DataTable";
import { StatusBadge } from "@/components/ui/Badge";
import Button from "@/components/ui/Button";
import LoadingSpinner from "@/components/ui/LoadingSpinner";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { SalesInvoice } from "@/types";

export default function InvoiceListPage() {
  const navigate = useNavigate();
  const addNotification = useUIStore((s) => s.addNotification);
  const [invoices, setInvoices] = useState<SalesInvoice[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [statusFilter, setStatusFilter] = useState("all");

  const loadInvoices = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await invoke("list_invoices");
      setInvoices(data as SalesInvoice[]);
    } catch (err) { setError(err instanceof Error ? err.message : String(err)); addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء تحميل البيانات" }); }
    finally { setLoading(false); }
  }, [addNotification]);

  useEffect(() => { loadInvoices(); }, [loadInvoices]);

  if (error) return <div className="flex flex-col items-center py-16"><div className="text-6xl mb-4 text-red-400">⚠️</div><h3 className="text-lg font-semibold text-[var(--text-primary)] mb-2">حدث خطأ</h3><p className="text-[var(--text-secondary)] mb-4">{error}</p><button onClick={loadInvoices} className="px-6 py-2.5 bg-brand-500 text-pure-white rounded-xl">إعادة المحاولة</button></div>;

  if (loading) return <div className="flex items-center justify-center py-16"><LoadingSpinner /></div>;

  const filtered = statusFilter === "all" ? invoices : invoices.filter(i => i.status?.toLowerCase() === statusFilter);

  const columns: Column<any>[] = useMemo(() => [
    { key: "inv_no", header: "رقم الفاتورة", sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.inv_no}</span> },
    { key: "date", header: "التاريخ", sortable: true, render: (r) => formatDate(r.date) },
    { key: "customer_name", header: "العميل", sortable: true, render: (r) => r.customer_name || "—" },
    { key: "total_milli", header: "الإجمالي", sortable: true, align: "left", render: (r) => <span className="font-bold">{formatOMR(r.total_milli)}</span> },
    { key: "paid_milli", header: "المدفوع", sortable: true, align: "left", render: (r) => formatOMR(r.paid_milli) },
    { key: "status", header: "الحالة", sortable: true, render: (r) => <StatusBadge status={r.status} /> },
  ], []);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">الفواتير</h1>
          <p className="page-subtitle">{invoices.length} فاتورة</p>
        </div>
        <div className="flex items-center gap-3">
          <Button onClick={() => navigate("/invoices/new")} icon={<Plus className="w-4 h-4" />}>فاتورة جديدة</Button>
        </div>
      </div>

      <div className="flex items-center gap-2 mb-4">
        {["all", "draft", "posted", "void"].map((s) => (
          <button key={s} onClick={() => setStatusFilter(s)}
            className={`px-4 py-1.5 rounded-full text-xs font-medium transition-all ${statusFilter === s ? "bg-brand-800 text-gold-400 border border-brand-500/30" : "bg-surface-800 text-surface-400 border border-surface-700 hover:text-white"}`}>
            {s === "all" ? "الكل" : s === "draft" ? "مسودة" : s === "posted" ? "مرحل" : "ملغى"}
          </button>
        ))}
      </div>

      <DataTable columns={columns} data={filtered} loading={loading}
        onRowClick={(r) => navigate(`/invoices/${r.id}`)}
        emptyMessage="لا توجد فواتير" />
    </div>
  );
}
