import { useState, useEffect } from "react";
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

  useEffect(() => { loadInvoice(); }, [id]);

  const loadInvoice = async () => {
    setLoading(true);
    try {
      const inv = await invoke<SalesInvoice>("get_invoice", { id: Number(id) });
      const ln = await invoke<InvoiceLine[]>("get_invoice_lines", { invoiceId: Number(id) });
      setInvoice(inv);
      setLines(ln);
    } catch (err) { console.error(err); }
    finally { setLoading(false); }
  };

  const handlePost = async () => {
    setActionLoading(true);
    try {
      await invoke("post_invoice", { id: Number(id) });
      await loadInvoice();
    } catch (err: unknown) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) }); }
    finally { setActionLoading(false); setConfirmOpen(false); }
  };

  const handleVoid = async () => {
    setActionLoading(true);
    try {
      await invoke("void_invoice", { id: Number(id), reason: "إلغاء" });
      await loadInvoice();
    } catch (err: unknown) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) }); }
    finally { setActionLoading(false); setConfirmOpen(false); }
  };

  const handleDuplicate = async () => {
    try {
      const newId = await invoke("duplicate_invoice", { id: Number(id) });
      navigate(`/invoices/${newId}`);
    } catch (err: unknown) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) }); }
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
    } catch (err: unknown) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) }); }
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
    } catch (err: unknown) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) }); }
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
    } catch (err: unknown) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) }); }
  };

  if (loading || !invoice) {
    return <div className="flex items-center justify-center h-64"><div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" /></div>;
  }

  const lineColumns: Column<InvoiceLine>[] = [
    { key: "product_name", header: "المنتج", render: (r) => r.product_name || "—" },
    { key: "cartons", header: "الكراتين", align: "center" },
    { key: "cups_per_carton", header: "كوب/كرتون", align: "center", render: (r) => r.cups_per_carton || "—" },
    { key: "qty_cups", header: "الإجمالي", align: "center", render: (r) => r.qty_cups?.toLocaleString() || "—" },
    { key: "unit_price_milli", header: "سعر الوحدة", align: "left", render: (r) => formatOMR(r.unit_price_milli) },
    { key: "line_net_milli", header: "الصافي", align: "left", render: (r) => <span className="font-bold">{formatOMR(r.line_net_milli)}</span> },
    { key: "vat_milli", header: "الضريبة", align: "left", render: (r) => formatOMR(r.vat_milli) },
  ];

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
            <p className="page-subtitle">{invoice.customer_name} • {formatDate(invoice.date)}</p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" onClick={handleDuplicate} icon={<Copy className="w-4 h-4" />}>نسخ</Button>
          {invoice.status === "Draft" && (
            <>
              <Button onClick={() => { setConfirmAction("post"); setConfirmOpen(true); }} icon={<Send className="w-4 h-4" />}>ترحيل</Button>
              <Button variant="danger" onClick={() => { setConfirmAction("void"); setConfirmOpen(true); }} icon={<Ban className="w-4 h-4" />}>إلغاء</Button>
            </>
          )}
          <div className="relative">
            <Button variant="outline" icon={<Printer className="w-4 h-4" />} onClick={() => setShowPrintModal(!showPrintModal)}>طباعة</Button>
            {showPrintModal && (
              <div className="absolute top-full mt-2 left-0 bg-surface-800 border border-surface-700 rounded-xl shadow-xl z-50 py-2 min-w-[200px]">
                <button onClick={() => handlePrint("invoice")} className="w-full px-4 py-2 text-right text-sm hover:bg-surface-700 flex items-center gap-2">
                  <FileText className="w-4 h-4 text-brand-400" /> فاتورة مبيعات
                </button>
                <button onClick={handlePrintDeliveryNote} className="w-full px-4 py-2 text-right text-sm hover:bg-surface-700 flex items-center gap-2">
                  <Truck className="w-4 h-4 text-orange-400" /> إيصال توصيل
                </button>
                <button onClick={handlePrintCustoms} className="w-full px-4 py-2 text-right text-sm hover:bg-surface-700 flex items-center gap-2">
                  <FileText className="w-4 h-4 text-amber-400" /> فاتورة الجمارك
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
            <h3 className="section-title">الملخص المالي</h3>
            <div className="space-y-3">
              <div className="flex justify-between text-sm"><span className="text-surface-400">الصافي</span><span>{formatOMR(invoice.net_milli)}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">الضريبة</span><span>{formatOMR(invoice.vat_milli)}</span></div>
              {invoice.discount_milli > 0 && <div className="flex justify-between text-sm"><span className="text-surface-400">الخصم</span><span className="text-red-400">- {formatOMR(invoice.discount_milli)}</span></div>}
              <div className="flex justify-between text-sm border-t border-surface-700 pt-2"><span className="text-surface-400">الإجمالي</span><span className="font-bold text-lg gradient-text">{formatOMR(invoice.total_milli)}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">المدفوع</span><span>{formatOMR(invoice.paid_milli)}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">المتبقي</span><span className="text-gold-400 font-bold">{formatOMR(invoice.total_milli - invoice.paid_milli)}</span></div>
            </div>
          </Card>
          <Card>
            <p className="text-xs text-surface-500">أنشأ: {invoice.created_by || "—"}</p>
            <p className="text-xs text-surface-500">الوقت: {invoice.created_at || "—"}</p>
          </Card>
        </div>
      </div>

      <ConfirmDialog
        open={confirmOpen}
        onCancel={() => setConfirmOpen(false)}
        onConfirm={confirmAction === "post" ? handlePost : handleVoid}
        title={confirmAction === "post" ? "ترحيل الفاتورة" : "إلغاء الفاتورة"}
        message={confirmAction === "post" ? "هل تريد ترحيل هذه الفاتورة؟ لن يمكن التراجع عن هذا الإجراء." : "هل تريد إلغاء هذه الفاتورة؟"}
        confirmLabel={confirmAction === "post" ? "ترحيل" : "إلغاء"}
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
            ─── فاتورة الجمارك (للأغراض الجمركية فقط) ───
          </div>
          <InvoicePrintTemplate data={printData} />
        </div>
      )}
    </div>
  );
}
