import { useState, useEffect, useCallback, useMemo } from "react";
import DataTable, { Column } from "@/components/ui/DataTable";
import Button from "@/components/ui/Button";
import Modal from "@/components/ui/Modal";
import LoadingSpinner from "@/components/ui/LoadingSpinner";
import { formatOMR, formatDate, omrToMilli } from "@/lib/utils";
import { invoke } from "@/lib/tauri";
import { useUIStore } from "@/stores/uiStore";
import CommercialInvoicePrintTemplate from "@/components/print/CommercialInvoicePrintTemplate";
import { printComponent } from "@/utils/printUtils";
import { Plus, Printer, Save, Send, Receipt, Trash2 } from "lucide-react";
import type { SalesInvoice, InvoiceLine, InvoicePrintData } from "@/types";

interface ProductOption { id: number; name_ar: string | null; code: string | null; cups_per_carton: number; default_price_milli: number; }
interface CustomerOption { id: number; name: string; balance_milli: number; }

interface LineForm { key: number; product_id: number | null; cartons: string; unit_price_milli: number; }

const emptyLine = (key: number): LineForm => ({ key, product_id: null, cartons: "", unit_price_milli: 0 });

export default function CommercialInvoicesPage() {
  const addNotification = useUIStore((s) => s.addNotification);
  const [items, setItems] = useState<SalesInvoice[]>([]);
  const [loading, setLoading] = useState(true);
  const [products, setProducts] = useState<ProductOption[]>([]);
  const [customers, setCustomers] = useState<CustomerOption[]>([]);

  const [modalOpen, setModalOpen] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [customerId, setCustomerId] = useState<number | null>(null);
  const [paymentType, setPaymentType] = useState("credit");
  const [invDate, setInvDate] = useState("");
  const [notes, setNotes] = useState("");
  const [lines, setLines] = useState<LineForm[]>([emptyLine(1)]);

  const [printData, setPrintData] = useState<InvoicePrintData | null>(null);
  const [postingId, setPostingId] = useState<number | null>(null);
  const [detail, setDetail] = useState<SalesInvoice | null>(null);
  const [detailLines, setDetailLines] = useState<InvoiceLine[]>([]);
  const [detailLoading, setDetailLoading] = useState(false);

  const loadList = useCallback(async () => {
    setLoading(true);
    try {
      const d = await invoke<SalesInvoice[]>("list_commercial_invoices");
      setItems(d);
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) });
    } finally { setLoading(false); }
  }, [addNotification]);

  useEffect(() => { loadList(); }, [loadList]);

  const loadOptions = useCallback(async () => {
    try {
      const [p, c] = await Promise.all([
        invoke<ProductOption[]>("list_products_for_select"),
        invoke<CustomerOption[]>("list_customers"),
      ]);
      setProducts(p);
      setCustomers(c);
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) });
    }
  }, [addNotification]);

  useEffect(() => { loadOptions(); }, [loadOptions]);

  const openCreate = () => {
    setCustomerId(null); setPaymentType("credit"); setInvDate(""); setNotes(""); setLines([emptyLine(1)]);
    setModalOpen(true);
  };

  const onProductChange = (key: number, pid: number | null) => {
    setLines(prev => prev.map(l => {
      if (l.key !== key) return l;
      if (pid == null) return { ...l, product_id: null, unit_price_milli: 0 };
      const p = products.find(pp => pp.id === pid);
      return { ...l, product_id: pid, unit_price_milli: p?.default_price_milli || 0 };
    }));
  };

  const updateLine = (key: number, patch: Partial<LineForm>) => {
    setLines(prev => prev.map(l => (l.key === key ? { ...l, ...patch } : l)));
  };

  const addLine = () => setLines(prev => [...prev, emptyLine(Math.max(0, ...prev.map(l => l.key)) + 1)]);
  const removeLine = (key: number) => setLines(prev => (prev.length > 1 ? prev.filter(l => l.key !== key) : prev));

  const netMilli = useMemo(() => lines.reduce((s, l) => s + (Number(l.cartons) || 0) * (l.unit_price_milli || 0), 0), [lines]);

  const handleCreate = async () => {
    if (!customerId) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "اختر العميل" }); return; }
    if (lines.every(l => !l.product_id || !l.cartons)) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "أضف بنداً واحداً على الأقل" }); return; }
    setSubmitting(true);
    try {
      await invoke("create_commercial_invoice", {
        input: {
          customer_id: customerId,
          payment_type: paymentType,
          date: invDate || null,
          notes: notes.trim() || null,
          lines: lines.map(l => ({ product_id: l.product_id!, cartons: Number(l.cartons) || 0, unit_price_milli: l.unit_price_milli || 0 })),
        },
      });
      setModalOpen(false);
      addNotification({ id: crypto.randomUUID(), type: "success", title: "تم بنجاح", message: "تم إنشاء الفاتورة التجارية (غير ضريبية)" });
      await loadList();
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) });
    } finally { setSubmitting(false); }
  };

  const handlePost = async (inv: SalesInvoice) => {
    setPostingId(inv.id);
    try {
      await invoke("post_invoice", { id: inv.id });
      addNotification({ id: crypto.randomUUID(), type: "success", title: "تم الترحيل", message: "تم ترحيل الفاتورة للمخزون والحسابات" });
      await loadList();
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) });
    } finally { setPostingId(null); }
  };

  const handlePrint = async (inv: SalesInvoice) => {
    try {
      const res = await invoke<InvoicePrintData>("get_commercial_invoice_for_print", { invoiceId: inv.id });
      setPrintData(res);
      setTimeout(() => {
        printComponent("print-area");
        setPrintData(null);
      }, 250);
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) });
    }
  };

  const openDetail = async (inv: SalesInvoice) => {
    setDetail(inv);
    setDetailLoading(true);
    setDetailLines([]);
    try {
      const res = await invoke<InvoiceLine[]>("get_invoice_lines", { invoiceId: inv.id });
      setDetailLines(res);
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) });
    } finally { setDetailLoading(false); }
  };

  const columns: Column<SalesInvoice>[] = useMemo(() => [
    { key: "inv_no", header: "الرقم", render: (r) => <span className="font-mono text-brand-400">{r.inv_no}</span> },
    { key: "date", header: "التاريخ", render: (r) => formatDate(r.date) },
    { key: "customer_name", header: "العميل", render: (r) => <span className="font-medium text-white">{r.customer_name || "—"}</span> },
    { key: "total_milli", header: "الإجمالي", align: "left", render: (r) => <span className="font-bold text-gold-400 font-mono">{formatOMR(r.total_milli)}</span> },
    { key: "status", header: "الحالة", render: (r) => <span className={`inline-flex px-2 py-0.5 rounded text-xs font-medium ${r.status === "Posted" ? "bg-emerald-500/15 text-emerald-400" : "bg-amber-500/15 text-amber-400"}`}>{r.status}</span> },
    { key: "actions", header: "إجراءات", align: "center", render: (r) => (
      <div className="flex items-center justify-center gap-1">
        <button onClick={(e) => { e.stopPropagation(); openDetail(r); }} aria-label="تفاصيل" className="p-1.5 rounded-lg text-surface-400 hover:text-brand-300 hover:bg-surface-800/50 transition-all" title="تفاصيل"><Receipt className="w-4 h-4" /></button>
        <button onClick={(e) => { e.stopPropagation(); handlePrint(r); }} aria-label="طباعة" className="p-1.5 rounded-lg text-surface-400 hover:text-brand-300 hover:bg-surface-800/50 transition-all" title="طباعة"><Printer className="w-4 h-4" /></button>
        {r.status !== "Posted" && (
          <button onClick={(e) => { e.stopPropagation(); handlePost(r); }} disabled={postingId === r.id} aria-label="ترحيل" className="p-1.5 rounded-lg text-surface-400 hover:text-emerald-300 hover:bg-surface-800/50 transition-all" title="ترحيل"><Send className="w-4 h-4" /></button>
        )}
      </div>
    )},
  ], [postingId]);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">الفواتير التجارية (غير ضريبية)</h1>
          <p className="page-subtitle">تُطبع باسم المصنع فقط — بدون شركة، بدون ضريبة، وبدون فواتير إلكترونية</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={openCreate}>فاتورة تجارية</Button>
      </div>

      <DataTable columns={columns} data={items} loading={loading} emptyMessage="لا توجد فواتير تجارية — أنشئ أول فاتورة" />

      <Modal open={modalOpen} onClose={() => setModalOpen(false)} title="فاتورة تجارية جديدة" size="xl" footer={
        <>
          <Button variant="outline" onClick={() => setModalOpen(false)}>إلغاء</Button>
          <Button onClick={handleCreate} loading={submitting} icon={<Save className="w-4 h-4" />}>إنشاء</Button>
        </>
      }>
        <div className="space-y-4 max-h-[70vh] overflow-y-auto px-0.5">
          <div className="grid grid-cols-2 lg:grid-cols-4 gap-3">
            <div>
              <label className="block text-sm text-surface-400 mb-1">العميل *</label>
              <select className="w-full input-field" value={customerId ?? ""} onChange={(e) => setCustomerId(e.target.value ? Number(e.target.value) : null)} aria-label="العميل">
                <option value="" disabled>اختر العميل</option>
                {customers.map(c => <option key={c.id} value={c.id}>{c.name}</option>)}
              </select>
            </div>
            <div>
              <label className="block text-sm text-surface-400 mb-1">طريقة الدفع</label>
              <select className="w-full input-field" value={paymentType} onChange={(e) => setPaymentType(e.target.value)} aria-label="طريقة الدفع">
                <option value="credit">آجل</option>
                <option value="cash">نقداً</option>
              </select>
            </div>
            <div>
              <label className="block text-sm text-surface-400 mb-1">التاريخ</label>
              <input type="date" className="w-full input-field" value={invDate} onChange={(e) => setInvDate(e.target.value)} aria-label="التاريخ" />
            </div>
            <div>
              <label className="block text-sm text-surface-400 mb-1">ملاحظات</label>
              <input type="text" className="w-full input-field" value={notes} onChange={(e) => setNotes(e.target.value)} aria-label="ملاحظات" />
            </div>
          </div>

          <div>
            <div className="flex items-center justify-between mb-2">
              <label className="block text-sm text-surface-400">بنود الفاتورة</label>
              <Button size="sm" variant="outline" onClick={addLine} icon={<Plus className="w-4 h-4" />}>إضافة بند</Button>
            </div>
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead><tr className="bg-surface-800 text-surface-400">
                  <th className="px-2 py-2 text-right font-medium">الصنف</th>
                  <th className="px-2 py-2 text-center font-medium">الكراتين</th>
                  <th className="px-2 py-2 text-center font-medium">سعر الكرتون (ريال)</th>
                  <th className="px-2 py-2 text-center font-medium">الإجمالي</th>
                  <th className="px-2 py-2" />
                </tr></thead>
                <tbody>
                  {lines.map(l => (
                    <tr key={l.key} className="border-t border-surface-700">
                      <td className="px-2 py-2">
                        <select className="input-field text-sm w-72" value={l.product_id ?? ""} onChange={(e) => onProductChange(l.key, e.target.value ? Number(e.target.value) : null)} aria-label="الصنف">
                          <option value="" disabled>اختر الصنف</option>
                          {products.map(p => <option key={p.id} value={p.id}>{p.name_ar}</option>)}
                        </select>
                      </td>
                      <td className="px-2 py-2 text-center"><input type="number" min={0} step={0.5} className="input-field text-sm w-24 text-center" value={l.cartons} onChange={(e) => updateLine(l.key, { cartons: e.target.value })} aria-label="الكراتين" /></td>
                      <td className="px-2 py-2 text-center"><input type="number" min={0} step={0.001} className="input-field text-sm w-28 text-center" value={l.unit_price_milli ? (l.unit_price_milli / 1000) : ""} onChange={(e) => updateLine(l.key, { unit_price_milli: omrToMilli(Number(e.target.value) || 0) })} aria-label="سعر الكرتون بالريال" /></td>
                      <td className="px-2 py-2 text-center font-mono font-bold text-gold-400">{formatOMR((Number(l.cartons) || 0) * (l.unit_price_milli || 0))}</td>
                      <td className="px-2 py-2 text-center">
                        <button onClick={() => removeLine(l.key)} aria-label="حذف البند" className="p-1.5 rounded-lg text-surface-400 hover:text-red-400 transition-all" title="حذف البند"><Trash2 className="w-4 h-4" /></button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>

          <div className="flex items-center justify-end gap-4 rounded-xl bg-surface-900/50 px-4 py-3">
            <div className="text-sm text-surface-400">الإجمالي (بدون ضريبة) <span className="font-mono text-gold-400 font-bold text-base">{formatOMR(netMilli)}</span></div>
          </div>
        </div>
      </Modal>

      <Modal open={!!detail} onClose={() => setDetail(null)} title={detail ? `تفاصيل الفاتورة ${detail.inv_no}` : ""} size="xl">
        {detailLoading ? (
          <div className="flex items-center justify-center py-12"><LoadingSpinner /></div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead><tr className="bg-surface-800 text-surface-400">
                <th className="px-4 py-2 text-right font-medium">الصنف</th>
                <th className="px-4 py-2 text-center font-medium">الكراتين</th>
                <th className="px-4 py-2 text-center font-medium">كوب/كرتون</th>
                <th className="px-4 py-2 text-center font-medium">إجمالي الأكواب</th>
                <th className="px-4 py-2 text-center font-medium">سعر الكرتون</th>
                <th className="px-4 py-2 text-center font-medium">الإجمالي</th>
              </tr></thead>
              <tbody>
                {detailLines.map(l => (
                  <tr key={l.id} className="border-t border-surface-700">
                    <td className="px-4 py-2 text-white font-medium">{l.product_name || "—"}</td>
                    <td className="px-4 py-2 text-center font-mono">{l.cartons.toLocaleString()}</td>
                    <td className="px-4 py-2 text-center font-mono">{l.cups_per_carton.toLocaleString()}</td>
                    <td className="px-4 py-2 text-center font-mono">{l.qty_cups.toLocaleString()}</td>
                    <td className="px-4 py-2 text-center font-mono">{formatOMR(l.unit_price_milli)}</td>
                    <td className="px-4 py-2 text-center font-mono font-bold text-gold-400">{formatOMR(l.line_net_milli)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
            {detail && (
              <div className="flex items-center justify-between mt-4 rounded-xl bg-surface-900/50 px-4 py-3">
                <span className="text-sm text-surface-400">الإجمالي (غير ضريبي)</span>
                <span className="font-mono font-bold text-gold-400 text-base">{formatOMR(detail.total_milli)}</span>
              </div>
            )}
          </div>
        )}
      </Modal>

      {printData && <div style={{ display: "none" }}><CommercialInvoicePrintTemplate data={printData} /></div>}
    </div>
  );
}
