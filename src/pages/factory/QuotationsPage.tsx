import { useState, useEffect, useCallback, useMemo } from "react";
import DataTable, { Column } from "@/components/ui/DataTable";
import Button from "@/components/ui/Button";
import Modal from "@/components/ui/Modal";
import { formatOMR, formatDate, omrToMilli } from "@/lib/utils";
import { invoke } from "@/lib/tauri";
import { useUIStore } from "@/stores/uiStore";
import QuotationPrintTemplate from "@/components/print/QuotationPrintTemplate";
import { printComponent } from "@/utils/printUtils";
import { Plus, Printer, Trash2, Pencil, Save } from "lucide-react";
import type { Quotation, QuotationLine, QuotationPrintData } from "@/types";

interface ProductOption { id: number; name_ar: string | null; code: string | null; cups_per_carton: number; default_price_milli: number; }
interface CustomerOption { id: number; name: string; }

interface LineForm {
  key: number;
  product_id: number | null;
  item_name: string;
  cup_size: string;
  cups_per_carton: string;
  cartons: string;
  unit_price_milli: number;
}

const emptyLine = (key: number): LineForm => ({
  key, product_id: null, item_name: "", cup_size: "", cups_per_carton: "", cartons: "100", unit_price_milli: 0,
});

const STATUSES = ["Draft", "Sent", "Accepted", "Rejected"];

export default function QuotationsPage() {
  const addNotification = useUIStore((s) => s.addNotification);
  const [items, setItems] = useState<Quotation[]>([]);
  const [loading, setLoading] = useState(true);
  const [statusFilter, setStatusFilter] = useState("");
  const [products, setProducts] = useState<ProductOption[]>([]);
  const [customers, setCustomers] = useState<CustomerOption[]>([]);

  const [modalOpen, setModalOpen] = useState(false);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [submitting, setSubmitting] = useState(false);

  const [clientName, setClientName] = useState("");
  const [customerId, setCustomerId] = useState<number | null>(null);
  const [clientContact, setClientContact] = useState("");
  const [clientPhone, setClientPhone] = useState("");
  const [clientEmail, setClientEmail] = useState("");
  const [clientAddress, setClientAddress] = useState("");
  const [title, setTitle] = useState("");
  const [notes, setNotes] = useState("");
  const [terms, setTerms] = useState("");
  const [validity, setValidity] = useState("7");
  const [discountOmr, setDiscountOmr] = useState("");
  const [quoteDate, setQuoteDate] = useState("");
  const [status, setStatus] = useState("Draft");
  const [lines, setLines] = useState<LineForm[]>([emptyLine(1)]);

  const [printData, setPrintData] = useState<QuotationPrintData | null>(null);
  const [printLoading, setPrintLoading] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState<Quotation | null>(null);
  const [deleteLoading, setDeleteLoading] = useState(false);

  const loadList = useCallback(async () => {
    setLoading(true);
    try {
      const d = await invoke<Quotation[]>("list_quotations", { status: statusFilter || null });
      setItems(d);
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) });
    } finally { setLoading(false); }
  }, [statusFilter, addNotification]);

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

  const resetForm = () => {
    setEditingId(null);
    setClientName(""); setCustomerId(null); setClientContact(""); setClientPhone(""); setClientEmail(""); setClientAddress("");
    setTitle(""); setNotes(""); setTerms(""); setValidity("7"); setDiscountOmr(""); setQuoteDate(""); setStatus("Draft");
    setLines([emptyLine(1)]);
  };

  const openCreate = () => { resetForm(); setModalOpen(true); };
  const openEdit = async (q: Quotation) => {
    try {
      const res = await invoke<{ quotation: Quotation; lines: QuotationLine[] }>("get_quotation", { id: q.id });
      setEditingId(q.id);
      setClientName(res.quotation.client_name || "");
      setCustomerId(res.quotation.customer_id);
      setClientContact(res.quotation.client_contact || "");
      setClientPhone(res.quotation.client_phone || "");
      setClientEmail(res.quotation.client_email || "");
      setClientAddress(res.quotation.client_address || "");
      setTitle(res.quotation.title || "");
      setNotes(res.quotation.notes || "");
      setTerms(res.quotation.terms || "");
      setValidity(String(res.quotation.validity_days ?? 7));
      setDiscountOmr(String((res.quotation.discount_milli || 0) / 1000));
      setQuoteDate(res.quotation.date || "");
      setStatus(res.quotation.status || "Draft");
      setLines(res.lines.length > 0
        ? res.lines.map((l, i) => ({
          key: i + 1,
          product_id: l.product_id,
          item_name: l.item_name || "",
          cup_size: l.cup_size || "",
          cups_per_carton: String(l.cups_per_carton ?? ""),
          cartons: String(l.cartons ?? ""),
          unit_price_milli: l.unit_price_milli,
        }))
        : [emptyLine(1)]);
      setModalOpen(true);
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) });
    }
  };

  const onProductChange = (key: number, pid: number | null) => {
    setLines(prev => prev.map(l => {
      if (l.key !== key) return l;
      if (pid == null) return { ...l, product_id: null };
      const p = products.find(pp => pp.id === pid);
      return {
        ...l,
        product_id: pid,
        item_name: p?.name_ar || "",
        cups_per_carton: p?.cups_per_carton ? String(p.cups_per_carton) : "",
        unit_price_milli: p?.default_price_milli || 0,
      };
    }));
  };

  const updateLine = (key: number, patch: Partial<LineForm>) => {
    setLines(prev => prev.map(l => (l.key === key ? { ...l, ...patch } : l)));
  };

  const addLine = () => setLines(prev => [...prev, emptyLine(Math.max(0, ...prev.map(l => l.key)) + 1)]);
  const removeLine = (key: number) => setLines(prev => (prev.length > 1 ? prev.filter(l => l.key !== key) : prev));

  const netMilli = useMemo(() => lines.reduce((s, l) => s + (Number(l.cartons) || 0) * (l.unit_price_milli || 0), 0), [lines]);
  const discountMilli = Math.max(0, omrToMilli(Number(discountOmr) || 0));
  const totalMilli = Math.max(0, netMilli - discountMilli);

  const handleSave = async () => {
    if (lines.length === 0 || lines.every(l => !(l.product_id || l.item_name.trim()) || !l.cartons)) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "أضف سطراً واحداً على الأقل باسم الصنف وعدد الكراتين" });
      return;
    }
    setSubmitting(true);
    const input = {
      date: quoteDate || null,
      customer_id: customerId,
      client_name: clientName.trim() || null,
      client_contact: clientContact.trim() || null,
      client_phone: clientPhone.trim() || null,
      client_email: clientEmail.trim() || null,
      client_address: clientAddress.trim() || null,
      title: title.trim() || null,
      notes: notes.trim() || null,
      terms: terms.trim() || null,
      validity_days: Number(validity) || 7,
      discount_milli: discountMilli,
      currency: "OMR",
      status,
      lines: lines.map(l => ({
        product_id: l.product_id,
        item_name: l.item_name.trim() || null,
        cup_size: l.cup_size.trim() || null,
        cups_per_carton: l.cups_per_carton ? Number(l.cups_per_carton) : null,
        cartons: Number(l.cartons) || 0,
        unit_price_milli: l.unit_price_milli || 0,
        notes: null,
      })),
    };
    try {
      if (editingId) {
        await invoke("update_quotation", { id: editingId, input });
        addNotification({ id: crypto.randomUUID(), type: "success", title: "تم بنجاح", message: "تم تحديث الكوتيشن" });
      } else {
        await invoke("create_quotation", { input });
        addNotification({ id: crypto.randomUUID(), type: "success", title: "تم بنجاح", message: "تم إنشاء الكوتيشن" });
      }
      setModalOpen(false);
      await loadList();
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) });
    } finally { setSubmitting(false); }
  };

  const handlePrint = async (q: Quotation) => {
    setPrintLoading(true);
    try {
      const res = await invoke<QuotationPrintData>("get_quotation_for_print", { id: q.id });
      setPrintData(res);
      setTimeout(() => {
        printComponent("print-area");
        setPrintData(null);
      }, 250);
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) });
    } finally { setPrintLoading(false); }
  };

  const handleDelete = async () => {
    if (!confirmDelete) return;
    setDeleteLoading(true);
    try {
      await invoke("delete_quotation", { id: confirmDelete.id });
      addNotification({ id: crypto.randomUUID(), type: "success", title: "تم الحذف", message: "تم حذف الكوتيشن" });
      setConfirmDelete(null);
      await loadList();
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) });
    } finally { setDeleteLoading(false); }
  };

  const columns: Column<Quotation>[] = useMemo(() => [
    { key: "quote_no", header: "الرقم", render: (r) => <span className="font-mono text-brand-400">{r.quote_no || `#${r.id}`}</span> },
    { key: "date", header: "التاريخ", render: (r) => formatDate(r.date) },
    { key: "client_name", header: "العميل", render: (r) => <span className="font-medium text-white">{r.client_name || "—"}</span> },
    { key: "total_milli", header: "الإجمالي", align: "left", render: (r) => <span className="font-bold text-gold-400 font-mono">{formatOMR(r.total_milli)}</span> },
    { key: "status", header: "الحالة", render: (r) => <span className="inline-flex px-2 py-0.5 rounded text-xs font-medium bg-surface-800 text-surface-300">{r.status}</span> },
    { key: "created_by", header: "المعد", render: (r) => r.created_by || "—" },
    { key: "actions", header: "إجراءات", align: "center", render: (r) => (
      <div className="flex items-center justify-center gap-1">
        <button onClick={(e) => { e.stopPropagation(); handlePrint(r); }} disabled={printLoading} className="p-1.5 rounded-lg text-surface-400 hover:text-brand-300 hover:bg-surface-800/50 transition-all" title="طباعة"><Printer className="w-4 h-4" /></button>
        <button onClick={(e) => { e.stopPropagation(); openEdit(r); }} className="p-1.5 rounded-lg text-surface-400 hover:text-amber-300 hover:bg-surface-800/50 transition-all" title="تعديل"><Pencil className="w-4 h-4" /></button>
        <button onClick={(e) => { e.stopPropagation(); setConfirmDelete(r); }} className="p-1.5 rounded-lg text-surface-400 hover:text-red-400 hover:bg-surface-800/50 transition-all" title="حذف"><Trash2 className="w-4 h-4" /></button>
      </div>
    )},
  ], [printLoading]);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">عروض الأسعار (كوتيشن)</h1>
          <p className="page-subtitle">عروض احترافية لأي عميل وبأي بيانات — الصنف، سعر الكرتون، عدد الأكواب، المواصفات</p>
        </div>
        <div className="flex items-center gap-2">
          <select className="input-field text-sm w-44" value={statusFilter} onChange={(e) => setStatusFilter(e.target.value)} aria-label="تصفية بالحالة">
            <option value="">كل الحالات</option>
            {STATUSES.map(s => <option key={s} value={s}>{s}</option>)}
          </select>
          <Button icon={<Plus className="w-4 h-4" />} onClick={openCreate}>كوتيشن جديد</Button>
        </div>
      </div>

      <DataTable columns={columns} data={items} loading={loading} emptyMessage="لا توجد عروض أسعار — أنشئ أول كوتيشن" />

      <Modal open={modalOpen} onClose={() => setModalOpen(false)} title={editingId ? "تعديل الكوتيشن" : "كوتيشن جديد"} size="xl" footer={
        <>
          <Button variant="outline" onClick={() => setModalOpen(false)}>إلغاء</Button>
          <Button onClick={handleSave} loading={submitting} icon={<Save className="w-4 h-4" />}>{editingId ? "حفظ التعديلات" : "إنشاء الكوتيشن"}</Button>
        </>
      }>
        <div className="space-y-4 max-h-[70vh] overflow-y-auto px-0.5">
          <div className="grid grid-cols-2 lg:grid-cols-3 gap-3">
            <div>
              <label className="block text-sm text-surface-400 mb-1">اسم العميل *</label>
              <input type="text" className="w-full input-field" value={clientName} onChange={(e) => setClientName(e.target.value)} placeholder="مثال: هايبر السيب" aria-label="اسم العميل" />
            </div>
            <div>
              <label className="block text-sm text-surface-400 mb-1">عميل مسجل (اختياري)</label>
              <select className="w-full input-field" value={customerId ?? ""} onChange={(e) => { const v = e.target.value; setCustomerId(v ? Number(v) : null); if (v && !clientName) { const c = customers.find(cc => cc.id === Number(v)); setClientName(c?.name || ""); } }} aria-label="عميل مسجل">
                <option value="">— بدون ربط —</option>
                {customers.map(c => <option key={c.id} value={c.id}>{c.name}</option>)}
              </select>
            </div>
            <div>
              <label className="block text-sm text-surface-400 mb-1">المسؤول / جهة الاتصال</label>
              <input type="text" className="w-full input-field" value={clientContact} onChange={(e) => setClientContact(e.target.value)} placeholder="اختياري" aria-label="جهة الاتصال" />
            </div>
            <div>
              <label className="block text-sm text-surface-400 mb-1">الهاتف</label>
              <input type="text" className="w-full input-field" value={clientPhone} onChange={(e) => setClientPhone(e.target.value)} placeholder="اختياري" aria-label="الهاتف" />
            </div>
            <div>
              <label className="block text-sm text-surface-400 mb-1">البريد</label>
              <input type="email" className="w-full input-field" value={clientEmail} onChange={(e) => setClientEmail(e.target.value)} placeholder="اختياري" aria-label="البريد" />
            </div>
            <div>
              <label className="block text-sm text-surface-400 mb-1">العنوان</label>
              <input type="text" className="w-full input-field" value={clientAddress} onChange={(e) => setClientAddress(e.target.value)} placeholder="اختياري" aria-label="العنوان" />
            </div>
            <div>
              <label className="block text-sm text-surface-400 mb-1">التاريخ</label>
              <input type="date" className="w-full input-field" value={quoteDate} onChange={(e) => setQuoteDate(e.target.value)} aria-label="التاريخ" />
            </div>
            <div>
              <label className="block text-sm text-surface-400 mb-1">صلاحية العرض (يوم)</label>
              <input type="number" min={1} className="w-full input-field" value={validity} onChange={(e) => setValidity(e.target.value)} aria-label="صلاحية العرض" />
            </div>
            <div>
              <label className="block text-sm text-surface-400 mb-1">الحالة</label>
              <select className="w-full input-field" value={status} onChange={(e) => setStatus(e.target.value)} aria-label="الحالة">
                {STATUSES.map(s => <option key={s} value={s}>{s}</option>)}
              </select>
            </div>
            <div className="lg:col-span-2">
              <label className="block text-sm text-surface-400 mb-1">الموضوع</label>
              <input type="text" className="w-full input-field" value={title} onChange={(e) => setTitle(e.target.value)} placeholder="مثال: عرض سعر كؤوس 200 مل — 500 كرتون" aria-label="الموضوع" />
            </div>
          </div>

          <div>
            <div className="flex items-center justify-between mb-2">
              <label className="block text-sm text-surface-400">بنود العرض</label>
              <Button size="sm" variant="outline" onClick={addLine} icon={<Plus className="w-4 h-4" />}>إضافة بند</Button>
            </div>
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead><tr className="bg-surface-800 text-surface-400">
                  <th className="px-2 py-2 text-right font-medium">الصنف</th>
                  <th className="px-2 py-2 text-right font-medium">المقاس</th>
                  <th className="px-2 py-2 text-center font-medium">كوب/كرتون</th>
                  <th className="px-2 py-2 text-center font-medium">الكراتين</th>
                  <th className="px-2 py-2 text-center font-medium">سعر الكرتون (ريال)</th>
                  <th className="px-2 py-2 text-center font-medium">الإجمالي</th>
                  <th className="px-2 py-2" />
                </tr></thead>
                <tbody>
                  {lines.map(l => (
                    <tr key={l.key} className="border-t border-surface-700">
                      <td className="px-2 py-2">
                        <select className="input-field text-sm w-48" value={l.product_id ?? ""} onChange={(e) => onProductChange(l.key, e.target.value ? Number(e.target.value) : null)} aria-label="اختيار منتج">
                          <option value="">نص حر…</option>
                          {products.map(p => <option key={p.id} value={p.id}>{p.name_ar}</option>)}
                        </select>
                        <input type="text" className="w-full input-field text-sm mt-1" value={l.item_name} onChange={(e) => updateLine(l.key, { item_name: e.target.value })} placeholder="اسم الصنف (يدوي أو تلقائي)" aria-label="اسم الصنف" />
                      </td>
                      <td className="px-2 py-2"><input type="text" className="input-field text-sm w-24" value={l.cup_size} onChange={(e) => updateLine(l.key, { cup_size: e.target.value })} placeholder="200 مل" aria-label="المقاس" /></td>
                      <td className="px-2 py-2 text-center"><input type="number" min={1} className="input-field text-sm w-24 text-center" value={l.cups_per_carton} onChange={(e) => updateLine(l.key, { cups_per_carton: e.target.value })} placeholder="1000" aria-label="كوب/كرتون" /></td>
                      <td className="px-2 py-2 text-center"><input type="number" min={0} step={0.5} className="input-field text-sm w-24 text-center" value={l.cartons} onChange={(e) => updateLine(l.key, { cartons: e.target.value })} placeholder="0" aria-label="الكراتين" /></td>
                      <td className="px-2 py-2 text-center"><input type="number" min={0} step={0.001} className="input-field text-sm w-28 text-center" value={l.unit_price_milli ? (l.unit_price_milli / 1000) : ""} onChange={(e) => updateLine(l.key, { unit_price_milli: omrToMilli(Number(e.target.value) || 0) })} placeholder="0.000" aria-label="سعر الكرتون بالريال" /></td>
                      <td className="px-2 py-2 text-center font-mono font-bold text-gold-400">{formatOMR((Number(l.cartons) || 0) * (l.unit_price_milli || 0))}</td>
                      <td className="px-2 py-2 text-center">
                        <button onClick={() => removeLine(l.key)} className="p-1.5 rounded-lg text-surface-400 hover:text-red-400 transition-all" title="حذف البند"><Trash2 className="w-4 h-4" /></button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-2 gap-3">
            <div>
              <label className="block text-sm text-surface-400 mb-1">الخصم (بالريال)</label>
              <input type="number" min={0} step={0.001} className="w-full input-field" value={discountOmr} onChange={(e) => setDiscountOmr(e.target.value)} placeholder="0.000" aria-label="الخصم بالريال" />
            </div>
            <div className="flex items-end justify-end gap-4 rounded-xl bg-surface-900/50 px-4 py-3">
              <div className="text-sm text-surface-400">المجموع الفرعي <span className="font-mono text-white font-bold">{formatOMR(netMilli)}</span></div>
              <div className="text-sm text-surface-400">الإجمالي <span className="font-mono text-gold-400 font-bold text-base">{formatOMR(totalMilli)}</span></div>
            </div>
            <div>
              <label className="block text-sm text-surface-400 mb-1">شروط العرض</label>
              <textarea className="w-full input-field" rows={2} value={terms} onChange={(e) => setTerms(e.target.value)} placeholder="مثال: الدفع عند الاستلام، الأسعار شاملة التغليف" aria-label="شروط العرض" />
            </div>
            <div>
              <label className="block text-sm text-surface-400 mb-1">ملاحظات</label>
              <textarea className="w-full input-field" rows={2} value={notes} onChange={(e) => setNotes(e.target.value)} placeholder="اختياري" aria-label="ملاحظات" />
            </div>
          </div>
        </div>
      </Modal>

      <Modal open={!!confirmDelete} onClose={() => setConfirmDelete(null)} title="حذف الكوتيشن" footer={
        <>
          <Button variant="outline" onClick={() => setConfirmDelete(null)}>إلغاء</Button>
          <Button onClick={handleDelete} loading={deleteLoading} variant="danger" icon={<Trash2 className="w-4 h-4" />}>حذف</Button>
        </>
      }>
        <p className="text-sm text-surface-300">هل تريد حذف الكوتيشن <span className="font-mono text-white">{confirmDelete?.quote_no}</span>؟ لا يمكن التراجع عن هذا الإجراء.</p>
      </Modal>

      {printData && <div style={{ display: "none" }}><QuotationPrintTemplate data={printData} /></div>}
    </div>
  );
}
