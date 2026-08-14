import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { formatOMR, omrToMilli } from "@/lib/utils";
import { invoke } from "@/lib/tauri";
import { ArrowRight, CreditCard, Printer } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { Customer, ReceiptPrintData } from "@/types";
import ReceiptPrintTemplate from "@/components/print/ReceiptPrintTemplate";
import { printComponent } from "@/utils/printUtils";

export default function CustomerPaymentPage() {
  const { id } = useParams();
  const navigate = useNavigate();
  const addNotification = useUIStore((s) => s.addNotification);
  const [customer, setCustomer] = useState<Customer | null>(null);
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);

  const [date, setDate] = useState(new Date().toISOString().split("T")[0]);
  const [amount, setAmount] = useState("");
  const [method, setMethod] = useState("cash");
  const [cashbankId, setCashbankId] = useState("");
  const [reference, setReference] = useState("");
  const [notes, setNotes] = useState("");

  const [savedPaymentId, setSavedPaymentId] = useState<number | null>(null);
  const [printData, setPrintData] = useState<ReceiptPrintData | null>(null);

  useEffect(() => {
    invoke("get_customer", { id: Number(id) })
      .then((d) => setCustomer(d as Customer))
      .catch((e: unknown) => addNotification({ title: "خطأ", message: String(e), type: "error" }))
      .finally(() => setLoading(false));
  }, [id]);

  const amountMilli = omrToMilli(Number(amount) || 0);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!amount || Number(amount) <= 0) return;
    setSubmitting(true);
    try {
      const paymentId = await invoke<number>("create_customer_payment", {
        customerId: Number(id),
        input: {
          date,
          amount_milli: amountMilli,
          method,
          cashbank_id: cashbankId ? Number(cashbankId) : null,
          reference: reference || null,
          notes: notes || null,
        },
      });
      addNotification({ id: crypto.randomUUID(), type: "success", title: "تم بنجاح", message: "تم تسجيل الدفعة بنجاح" });
      setSavedPaymentId(paymentId);
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "فشل في تسجيل الدفعة" });
    } finally {
      setSubmitting(false);
    }
  };

  const handlePrintReceipt = async () => {
    if (savedPaymentId == null) return;
    try {
      const result = await invoke<ReceiptPrintData>("get_receipt_for_print", { paymentId: savedPaymentId });
      setPrintData(result);
      setTimeout(() => {
        printComponent("print-area");
        setPrintData(null);
      }, 200);
    } catch (err: unknown) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) });
    }
  };

  if (loading) {
    return <div className="flex items-center justify-center h-64"><div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" /></div>;
  }

  if (!customer) {
    return <div className="flex flex-col items-center justify-center h-64 gap-4"><p className="text-surface-400">تعذر تحميل بيانات العميل</p><button className="btn-outline px-4 py-2 rounded-xl text-sm" onClick={() => window.location.reload()}>إعادة المحاولة</button></div>;
  }

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div className="flex items-center gap-3">
          <button onClick={() => navigate(`/customers/${id}`)} className="btn-ghost p-2"><ArrowRight className="w-5 h-5" /></button>
          <div>
            <h1 className="page-title">تسجيل دفعة</h1>
            <p className="page-subtitle">{customer.name}</p>
          </div>
        </div>
      </div>

      {savedPaymentId != null && (
        <Card className="border-emerald-700/40">
          <div className="flex flex-col items-center gap-4 py-6 text-center">
            <div className="w-14 h-14 rounded-full bg-emerald-500/10 flex items-center justify-center">
              <svg className="w-7 h-7 text-emerald-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
              </svg>
            </div>
            <div>
              <p className="text-lg font-bold">تم تسجيل الدفعة بنجاح</p>
              <p className="text-sm text-surface-400 mt-1">المبلغ: <span className="text-emerald-400 font-semibold">{formatOMR(amountMilli)}</span></p>
            </div>
            <div className="flex gap-3">
              <Button variant="gold" icon={<Printer className="w-4 h-4" />} onClick={handlePrintReceipt}>طباعة الإيصال</Button>
              <Button variant="outline" onClick={() => navigate(`/customers/${id}`)}>العودة إلى العميل</Button>
            </div>
          </div>
        </Card>
      )}

      {savedPaymentId == null && (
        <div className="grid grid-cols-3 gap-6">
          <Card className="col-span-2">
            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm text-surface-400 mb-1">التاريخ</label>
                  <input type="date" value={date} onChange={(e) => setDate(e.target.value)} className="w-full input-field" required aria-label="التاريخ" />
                </div>
                <div>
                  <label className="block text-sm text-surface-400 mb-1">المبلغ (بالريال)</label>
                  <input type="number" value={amount} onChange={(e) => setAmount(e.target.value)} className="w-full input-field" placeholder="مثال: 125.500" min="0.001" step="0.001" required aria-label="المبلغ بالريال" />
                  {Number(amount) > 0 && <p className="text-xs text-surface-500 mt-1">= {amountMilli.toLocaleString("en-US")} بيسة/ملي</p>}
                </div>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm text-surface-400 mb-1">طريقة الدفع</label>
                  <select value={method} onChange={(e) => setMethod(e.target.value)} className="w-full input-field" aria-label="طريقة الدفع">
                    <option value="cash">نقدي</option>
                    <option value="bank_transfer">تحويل بنكي</option>
                    <option value="cheque">شيك</option>
                  </select>
                </div>
                <div>
                  <label className="block text-sm text-surface-400 mb-1">رقم الحساب (اختياري)</label>
                  <input type="number" value={cashbankId} onChange={(e) => setCashbankId(e.target.value)} className="w-full input-field" placeholder="—" aria-label="رقم الحساب" />
                </div>
              </div>

              <div>
                <label className="block text-sm text-surface-400 mb-1">المرجع (اختياري)</label>
                <input type="text" value={reference} onChange={(e) => setReference(e.target.value)} className="w-full input-field" placeholder="—" aria-label="المرجع" />
              </div>

              <div>
                <label className="block text-sm text-surface-400 mb-1">ملاحظات (اختياري)</label>
                <textarea value={notes} onChange={(e) => setNotes(e.target.value)} className="w-full input-field" rows={3} placeholder="—" aria-label="ملاحظات" />
              </div>

              <div className="flex justify-end gap-2 pt-2">
                <Button type="button" variant="outline" onClick={() => navigate(`/customers/${id}`)}>إلغاء</Button>
                <Button type="submit" loading={submitting} icon={<CreditCard className="w-4 h-4" />}>تسجيل الدفعة</Button>
              </div>
            </form>
          </Card>

          <Card>
            <h4 className="text-sm text-surface-400 mb-3">معلومات العميل</h4>
            <div className="space-y-4">
              <div className="text-center py-4">
                <p className="text-3xl font-bold gradient-text">{formatOMR(customer.balance_milli)}</p>
                <p className="text-xs text-surface-400 mt-1">الرصيد الحالي</p>
              </div>
              <div className="space-y-2">
                <div className="flex justify-between text-sm"><span className="text-surface-400">الاسم</span><span className="font-medium">{customer.name}</span></div>
                <div className="flex justify-between text-sm"><span className="text-surface-400">الكود</span><span className="font-mono text-xs">{customer.code || "—"}</span></div>
                <div className="flex justify-between text-sm"><span className="text-surface-400">الهاتف</span><span>{customer.phone || "—"}</span></div>
              </div>
            </div>
          </Card>
        </div>
      )}

      {printData && (
        <div style={{ position: "absolute", left: "-9999px" }}>
          <ReceiptPrintTemplate data={printData} />
        </div>
      )}
    </div>
  );
}
