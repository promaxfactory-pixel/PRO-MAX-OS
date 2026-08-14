import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { formatOMR, omrToMilli } from "@/lib/utils";
import { invoke } from "@/lib/tauri";
import { ArrowRight, CreditCard } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { Supplier } from "@/types";

export default function SupplierPaymentPage() {
  const { id } = useParams();
  const navigate = useNavigate();
  const addNotification = useUIStore((s) => s.addNotification);
  const [supplier, setSupplier] = useState<Supplier | null>(null);
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);

  const [date, setDate] = useState(new Date().toISOString().split("T")[0]);
  const [amount, setAmount] = useState("");
  const [method, setMethod] = useState("cash");
  const [reference, setReference] = useState("");
  const [notes, setNotes] = useState("");

  useEffect(() => {
    invoke("get_supplier", { id: Number(id) })
      .then((d) => setSupplier(d as Supplier))
      .catch((e: unknown) => addNotification({ title: "خطأ", message: String(e), type: "error" }))
      .finally(() => setLoading(false));
  }, [id]);

  const amountMilli = omrToMilli(Number(amount) || 0);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!amount || Number(amount) <= 0) return;
    setSubmitting(true);
    try {
      await invoke("create_supplier_payment", {
        supplierId: Number(id),
        input: {
          date,
          amount_milli: amountMilli,
          method,
          reference: reference || null,
          notes: notes || null,
        },
      });
      addNotification({ id: crypto.randomUUID(), type: "success", title: "تم بنجاح", message: "تم تسجيل الدفعة بنجاح" });
      navigate(`/suppliers/${id}`);
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "فشل في تسجيل الدفعة" });
    } finally {
      setSubmitting(false);
    }
  };

  if (loading) {
    return <div className="flex items-center justify-center h-64"><div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" /></div>;
  }

  if (!supplier) {
    return <div className="flex flex-col items-center justify-center h-64 gap-4"><p className="text-surface-400">تعذر تحميل بيانات المورد</p><button className="btn-outline px-4 py-2 rounded-xl text-sm" onClick={() => window.location.reload()}>إعادة المحاولة</button></div>;
  }

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div className="flex items-center gap-3">
          <button onClick={() => navigate(`/suppliers/${id}`)} className="btn-ghost p-2"><ArrowRight className="w-5 h-5" /></button>
          <div>
            <h1 className="page-title">تسجيل دفعة</h1>
            <p className="page-subtitle">{supplier.name}</p>
          </div>
        </div>
      </div>

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
                <label className="block text-sm text-surface-400 mb-1">المرجع (اختياري)</label>
                <input type="text" value={reference} onChange={(e) => setReference(e.target.value)} className="w-full input-field" placeholder="—" aria-label="المرجع" />
              </div>
            </div>

            <div>
              <label className="block text-sm text-surface-400 mb-1">ملاحظات (اختياري)</label>
              <textarea value={notes} onChange={(e) => setNotes(e.target.value)} className="w-full input-field" rows={3} placeholder="—" aria-label="ملاحظات" />
            </div>

            <div className="flex justify-end gap-2 pt-2">
              <Button type="button" variant="outline" onClick={() => navigate(`/suppliers/${id}`)}>إلغاء</Button>
              <Button type="submit" loading={submitting} icon={<CreditCard className="w-4 h-4" />}>تسجيل الدفعة</Button>
            </div>
          </form>
        </Card>

        <Card>
          <h4 className="text-sm text-surface-400 mb-3">معلومات المورد</h4>
          <div className="space-y-4">
            <div className="text-center py-4">
              <p className="text-3xl font-bold gradient-text">{formatOMR(supplier.balance_milli)}</p>
              <p className="text-xs text-surface-400 mt-1">الرصيد الحالي</p>
            </div>
            <div className="space-y-2">
              <div className="flex justify-between text-sm"><span className="text-surface-400">الاسم</span><span className="font-medium">{supplier.name}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">الكود</span><span className="font-mono text-xs">{supplier.code || "—"}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">الهاتف</span><span>{supplier.phone || "—"}</span></div>
            </div>
          </div>
        </Card>
      </div>
    </div>
  );
}



