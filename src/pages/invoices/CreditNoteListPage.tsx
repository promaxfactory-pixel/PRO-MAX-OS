import { useState, useEffect, useMemo, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import DataTable, { Column } from "@/components/ui/DataTable";
import { StatusBadge } from "@/components/ui/Badge";
import LoadingSpinner from "@/components/ui/LoadingSpinner";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@/lib/tauri";
import { Printer } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import type { CreditNoteSummary, CreditNotePrintData } from "@/types";
import CreditNotePrintTemplate from "@/components/print/CreditNotePrintTemplate";
import { printComponent } from "@/utils/printUtils";

export default function CreditNoteListPage() {
  const navigate = useNavigate();
  const addNotification = useUIStore((s) => s.addNotification);
  const [notes, setNotes] = useState<CreditNoteSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [printData, setPrintData] = useState<CreditNotePrintData | null>(null);
  const [printLoading, setPrintLoading] = useState<number | null>(null);

  const loadNotes = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await invoke("list_credit_notes");
      setNotes(data as CreditNoteSummary[]);
    } catch (err) { setError(err instanceof Error ? err.message : String(err)); addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء تحميل البيانات" }); }
    finally { setLoading(false); }
  }, [addNotification]);

  useEffect(() => { loadNotes(); }, [loadNotes]);

  const handlePrint = async (id: number) => {
    setPrintLoading(id);
    try {
      const result = await invoke<CreditNotePrintData>("get_credit_note_for_print", { creditNoteId: id });
      setPrintData(result);
      setTimeout(() => {
        printComponent("print-area");
        setPrintData(null);
      }, 200);
    } catch (err: unknown) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) }); }
    finally { setPrintLoading(null); }
  };

  if (error) return <div className="flex flex-col items-center py-16"><div className="text-6xl mb-4 text-red-400">⚠️</div><h3 className="text-lg font-semibold text-[var(--text-primary)] mb-2">حدث خطأ</h3><p className="text-[var(--text-secondary)] mb-4">{error}</p><button onClick={loadNotes} className="px-6 py-2.5 bg-brand-500 text-pure-white rounded-xl">إعادة المحاولة</button></div>;

  if (loading) return <div className="flex items-center justify-center py-16"><LoadingSpinner /></div>;

  const columns: Column<CreditNoteSummary>[] = useMemo(() => [
    { key: "cn_no", header: "رقم الإشعار", sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.cn_no || `#${r.id}`}</span> },
    { key: "date", header: "التاريخ", sortable: true, render: (r) => formatDate(r.date) },
    { key: "invoice_no", header: "الفاتورة المرجعية", sortable: true, render: (r) => <button className="font-mono text-surface-300 hover:text-gold-400" onClick={(e) => { e.stopPropagation(); navigate(`/invoices/${r.invoice_id}`); }}>{r.invoice_no || "—"}</button> },
    { key: "customer_name", header: "العميل", sortable: true, render: (r) => r.customer_name || "—" },
    { key: "total_milli", header: "الإجمالي", sortable: true, align: "left", render: (r) => <span className="font-bold text-red-400">- {formatOMR(r.total_milli)}</span> },
    { key: "reason", header: "السبب", sortable: true, render: (r) => r.reason || "—" },
    { key: "status", header: "الحالة", sortable: true, render: (r) => <StatusBadge status={r.status} /> },
    { key: "actions", header: "إجراءات", align: "center", render: (r) => <button onClick={(e) => { e.stopPropagation(); handlePrint(r.id); }} disabled={printLoading === r.id} className="btn-outline px-3 py-1.5 rounded-lg text-xs disabled:opacity-50"><Printer className="w-3.5 h-3.5" /> طباعة</button> },
  ], [navigate, printLoading]);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">الإشعارات الدائنة</h1>
          <p className="page-subtitle">{notes.length} إشعار</p>
        </div>
      </div>

      <DataTable columns={columns} data={notes} loading={loading}
        onRowClick={(r) => navigate(`/invoices/${r.invoice_id}`)}
        emptyMessage="لا توجد إشعارات دائنة" />

      {printData && (
        <div style={{ position: "absolute", left: "-9999px" }}>
          <CreditNotePrintTemplate data={printData} />
        </div>
      )}
    </div>
  );
}
