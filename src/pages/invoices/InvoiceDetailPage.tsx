import { useState, useEffect, useMemo, useCallback } from "react";
import { useParams, useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { StatusBadge } from "@/components/ui/Badge";
import DataTable, { Column } from "@/components/ui/DataTable";
import ConfirmDialog from "@/components/ui/ConfirmDialog";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, Send, Ban, Copy, Printer, FileText, Truck, RotateCcw } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import type { SalesInvoice, InvoiceLine } from "@/types";
import InvoicePrintTemplate from "@/components/print/InvoicePrintTemplate";
import ReceiptPrintTemplate from "@/components/print/ReceiptPrintTemplate";
import DeliveryNotePrintTemplate from "@/components/print/DeliveryNotePrintTemplate";
import { printComponent } from "@/utils/printUtils";

export default function InvoiceDetailPage() {
  const { id } = useParams();
  const navigate = useNavigate();
  const { addNotification } = useUIStore();
  const [invoice, setInvoice] = useState<SalesInvoice | null>(null);
  const [lines, setLines] = useState<InvoiceLine[]>([]);
  const [loading, setLoading] = useState(true);
  const [actionLoading, setActionLoading] = useState(false);
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [confirmAction, setConfirmAction] = useState<"post" | "void">("post");
  const [printData, setPrintData] = useState<SalesInvoice | null>(null);
  const [printType, setPrintType] = useState<string>("");
  const [showPrintModal, setShowPrintModal] = useState(false);

  const loadInvoice = useCallback(async () => {
    setLoading(true);
    try {
      const inv = await invoke<SalesInvoice>("get_invoice", { id: Number(id) });
      const ln = await invoke<InvoiceLine[]>("get_invoice_lines", { invoiceId: Number(id) });
      setInvoice(inv);
      setLines(ln);
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: "ط®ط·ط£", message: "ط­ط¯ط« ط®ط·ط£ ط£ط«ظ†ط§ط، طھط­ظ…ظٹظ„ ط§ظ„ط¨ظٹط§ظ†ط§طھ" }); }
    finally { setLoading(false); }
  }, [id, addNotification]);

  useEffect(() => { loadInvoice(); }, [loadInvoice]);

  const handlePost = async () => {
    setActionLoading(true);
    try {
      await invoke("post_invoice", { id: Number(id) });
      await loadInvoice();
    } catch (err: unknown) { addNotification({ id: crypto.randomUUID(), type: "error", title: "ط®ط·ط£", message: String(err) }); }
    finally { setActionLoading(false); setConfirmOpen(false); }
  };

  const handleVoid = async () => {
    setActionLoading(true);
    try {
      await invoke("void_invoice", { id: Number(id), reason: "ط¥ظ„ط؛ط§ط،" });
      await loadInvoice();
    } catch (err: unknown) { addNotification({ id: crypto.randomUUID(), type: "error", title: "ط®ط·ط£", message: String(err) }); }
    finally { setActionLoading(false); setConfirmOpen(false); }
  };

  const handleDuplicate = async () => {
    try {
      const newId = await invoke("duplicate_invoice", { id: Number(id) });
      navigate(`/invoices/${newId}`);
    } catch (err: unknown) { addNotification({ id: crypto.randomUUID(), type: "error", title: "ط®ط·ط£", message: String(err) }); }
  };

  const handlePrint = async (type: string) => {
    try {
      const result = await invoke<SalesInvoice>("get_invoice_for_print", { invoiceId: Number(id) });
      setPrintData(result);
      setPrintType(type);
      setShowPrintModal(false);
      setTimeout(() => {
        printComponent("print-area");
        setPrintData(null);
      }, 200);
    } catch (err: unknown) { addNotification({ id: crypto.randomUUID(), type: "error", title: "ط®ط·ط£", message: String(err) }); }
  };

  const handlePrintDeliveryNote = async () => {
    try {
      const result = await invoke<SalesInvoice>("get_delivery_note_for_print", { invoiceId: Number(id) });
      setPrintData(result);
      setPrintType("delivery_note");
      setShowPrintModal(false);
      setTimeout(() => {
        printComponent("print-area");
        setPrintData(null);
      }, 200);
    } catch (err: unknown) { addNotification({ id: crypto.randomUUID(), type: "error", title: "ط®ط·ط£", message: String(err) }); }
  };

  const handlePrintCustoms = async () => {
    try {
      const result = await invoke<SalesInvoice>("get_invoice_for_print_customs", { invoiceId: Number(id) });
      setPrintData(result);
      setPrintType("invoice_customs");
      setShowPrintModal(false);
      setTimeout(() => {
        printComponent("print-area");
        setPrintData(null);
      }, 200);
    } catch (err: unknown) { addNotification({ id: crypto.randomUUID(), type: "error", title: "ط®ط·ط£", message: String(err) }); }
  };

  if (loading) {
    return <div className="flex items-center justify-center h-64"><div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" /></div>;
  }

  const lineColumns: Column<InvoiceLine>[] = useMemo(() => [
    { key: "product_name", header: "ط§ظ„ظ…ظ†طھط¬", render: (r) => r.product_name || "â€”" },
    { key: "cartons", header: "ط§ظ„ظƒط±ط§طھظٹظ†", align: "center" },
    { key: "cups_per_carton", header: "ظƒظˆط¨/ظƒط±طھظˆظ†", align: "center", render: (r) => r.cups_per_carton || "â€”" },
    { key: "qty_cups", header: "ط§ظ„ط¥ط¬ظ…ط§ظ„ظٹ", align: "center", render: (r) => r.qty_cups?.toLocaleString() || "â€”" },
    { key: "unit_price_milli", header: "ط³ط¹ط± ط§ظ„ظˆط­ط¯ط©", align: "left", render: (r) => formatOMR(r.unit_price_milli) },
    { key: "line_net_milli", header: "ط§ظ„طµط§ظپظٹ", align: "left", render: (r) => <span className="font-bold">{formatOMR(r.line_net_milli)}</span> },
    { key: "vat_milli", header: "ط§ظ„ط¶ط±ظٹط¨ط©", align: "left", render: (r) => formatOMR(r.vat_milli) },
  ], []);

  if (!invoice) {
    return <div className="flex flex-col items-center justify-center h-64 gap-4"><p className="text-surface-400">تعذر تحميل بيانات الفاتورة</p><button className="btn-outline px-4 py-2 rounded-xl text-sm" onClick={() => window.location.reload()}>إعادة المحاولة</button></div>;
  }

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div className="flex items-center gap-3">
          <button onClick={() => navigate('/invoices')} className="btn-ghost p-2"><ArrowRight className="w-5 h-5" /></button>
          <div>
            <h1 className="page-title flex items-center gap-3">
              <span className="font-mono text-brand-400">{invoice.inv_no}</span>
              <StatusBadge status={invoice.status} />
            </h1>
            <p className="page-subtitle">{invoice.customer_name} â€¢ {formatDate(invoice.date)}</p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" onClick={handleDuplicate} icon={<Copy className="w-4 h-4" />}>ظ†ط³ط®</Button>
          {invoice.status === "Draft" && (
            <>
              <Button onClick={() => { setConfirmAction("post"); setConfirmOpen(true); }} icon={<Send className="w-4 h-4" />}>طھط±ط­ظٹظ„</Button>
              <Button variant="danger" onClick={() => { setConfirmAction("void"); setConfirmOpen(true); }} icon={<Ban className="w-4 h-4" />}>ط¥ظ„ط؛ط§ط،</Button>
            </>
          )}
          <div className="relative">
            <Button variant="outline" icon={<Printer className="w-4 h-4" />} onClick={() => setShowPrintModal(!showPrintModal)}>ط·ط¨ط§ط¹ط©</Button>
            {showPrintModal && (
              <div className="absolute top-full mt-2 left-0 bg-surface-800 border border-surface-700 rounded-xl shadow-xl z-50 py-2 min-w-[200px]">
                <button onClick={() => handlePrint("invoice")} className="w-full px-4 py-2 text-right text-sm hover:bg-surface-700 flex items-center gap-2">
                  <FileText className="w-4 h-4 text-brand-400" /> ظپط§طھظˆط±ط© ظ…ط¨ظٹط¹ط§طھ
                </button>
                <button onClick={handlePrintDeliveryNote} className="w-full px-4 py-2 text-right text-sm hover:bg-surface-700 flex items-center gap-2">
                  <Truck className="w-4 h-4 text-orange-400" /> ط¥ظٹطµط§ظ„ طھظˆطµظٹظ„
                </button>
                <button onClick={handlePrintCustoms} className="w-full px-4 py-2 text-right text-sm hover:bg-surface-700 flex items-center gap-2">
                  <FileText className="w-4 h-4 text-amber-400" /> ظپط§طھظˆط±ط© ط§ظ„ط¬ظ…ط§ط±ظƒ
                </button>
              </div>
            )}
          </div>
        </div>
      </div>

      <div className="grid grid-cols-3 gap-6">
        <div className="col-span-2">
          <DataTable columns={lineColumns} data={lines} compact />
        </div>
        <div className="space-y-4">
          <Card>
            <h3 className="section-title">ط§ظ„ظ…ظ„ط®طµ ط§ظ„ظ…ط§ظ„ظٹ</h3>
            <div className="space-y-3">
              <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„طµط§ظپظٹ</span><span>{formatOMR(invoice.net_milli)}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„ط¶ط±ظٹط¨ط©</span><span>{formatOMR(invoice.vat_milli)}</span></div>
              {invoice.discount_milli > 0 && <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„ط®طµظ…</span><span className="text-red-400">- {formatOMR(invoice.discount_milli)}</span></div>}
              <div className="flex justify-between text-sm border-t border-surface-700 pt-2"><span className="text-surface-400">ط§ظ„ط¥ط¬ظ…ط§ظ„ظٹ</span><span className="font-bold text-lg gradient-text">{formatOMR(invoice.total_milli)}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„ظ…ط¯ظپظˆط¹</span><span>{formatOMR(invoice.paid_milli)}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„ظ…طھط¨ظ‚ظٹ</span><span className="text-gold-400 font-bold">{formatOMR(invoice.total_milli - invoice.paid_milli)}</span></div>
            </div>
          </Card>
          <Card>
            <p className="text-xs text-surface-500">ط£ظ†ط´ط£: {invoice.created_by || "â€”"}</p>
            <p className="text-xs text-surface-500">ط§ظ„ظˆظ‚طھ: {invoice.created_at || "â€”"}</p>
          </Card>
        </div>
      </div>

      <ConfirmDialog
        open={confirmOpen}
        onCancel={() => setConfirmOpen(false)}
        onConfirm={confirmAction === "post" ? handlePost : handleVoid}
        title={confirmAction === "post" ? "طھط±ط­ظٹظ„ ط§ظ„ظپط§طھظˆط±ط©" : "ط¥ظ„ط؛ط§ط، ط§ظ„ظپط§طھظˆط±ط©"}
        message={confirmAction === "post" ? "ظ‡ظ„ طھط±ظٹط¯ طھط±ط­ظٹظ„ ظ‡ط°ظ‡ ط§ظ„ظپط§طھظˆط±ط©طں ظ„ظ† ظٹظ…ظƒظ† ط§ظ„طھط±ط§ط¬ط¹ ط¹ظ† ظ‡ط°ط§ ط§ظ„ط¥ط¬ط±ط§ط،." : "ظ‡ظ„ طھط±ظٹط¯ ط¥ظ„ط؛ط§ط، ظ‡ط°ظ‡ ط§ظ„ظپط§طھظˆط±ط©طں"}
        confirmLabel={confirmAction === "post" ? "طھط±ط­ظٹظ„" : "ط¥ظ„ط؛ط§ط،"}
        loading={actionLoading}
      />

      {printData && printType === "invoice" && (
        <div style={{ position: "absolute", left: "-9999px" }}>
          <InvoicePrintTemplate data={printData} />
        </div>
      )}
      {printData && printType === "delivery_note" && (
        <div style={{ position: "absolute", left: "-9999px" }}>
          <DeliveryNotePrintTemplate data={printData} />
        </div>
      )}
      {printData && printType === "invoice_customs" && (
        <div style={{ position: "absolute", left: "-9999px" }}>
          <div style={{ textAlign: "center", fontSize: "18px", fontWeight: "bold", color: "#dc2626", marginBottom: "8px" }}>
            â”€â”€â”€ ظپط§طھظˆط±ط© ط§ظ„ط¬ظ…ط§ط±ظƒ (ظ„ظ„ط£ط؛ط±ط§ط¶ ط§ظ„ط¬ظ…ط±ظƒظٹط© ظپظ‚ط·) â”€â”€â”€
          </div>
          <InvoicePrintTemplate data={printData} />
        </div>
      )}
    </div>
  );
}

