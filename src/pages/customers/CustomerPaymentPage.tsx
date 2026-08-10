import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, CreditCard } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { Customer } from "@/types";

export default function CustomerPaymentPage() {
  const { id } = useParams();
  const navigate = useNavigate();
  const addNotification = useUIStore((s) => s.addNotification);
  const [customer, setCustomer] = useState<Customer | null>(null);
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);

  const [date, setDate] = useState(new Date().toISOString().split("T")[0]);
  const [amountMilli, setAmountMilli] = useState("");
  const [method, setMethod] = useState("cash");
  const [cashbankId, setCashbankId] = useState("");
  const [reference, setReference] = useState("");
  const [notes, setNotes] = useState("");

  useEffect(() => {
    invoke("get_customer", { id: Number(id) })
      .then((d) => setCustomer(d as Customer))
      .catch((e: unknown) => addNotification({ title: "ط®ط·ط£", message: String(e), type: "error" }))
      .finally(() => setLoading(false));
  }, [id]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!amountMilli || Number(amountMilli) <= 0) return;
    setSubmitting(true);
    try {
      await invoke("create_customer_payment", {
        customerId: Number(id),
        input: {
          date,
          amount_milli: Number(amountMilli),
          method,
          cashbank_id: cashbankId ? Number(cashbankId) : null,
          reference: reference || null,
          notes: notes || null,
        },
      });
      addNotification({ id: crypto.randomUUID(), type: "success", title: "طھظ… ط¨ظ†ط¬ط§ط­", message: "طھظ… طھط³ط¬ظٹظ„ ط§ظ„ط¯ظپط¹ط© ط¨ظ†ط¬ط§ط­" });
      navigate(`/customers/${id}`);
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "ط®ط·ط£", message: "ظپط´ظ„ ظپظٹ طھط³ط¬ظٹظ„ ط§ظ„ط¯ظپط¹ط©" });
    } finally {
      setSubmitting(false);
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
            <h1 className="page-title">طھط³ط¬ظٹظ„ ط¯ظپط¹ط©</h1>
            <p className="page-subtitle">{customer.name}</p>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-3 gap-6">
        <Card className="col-span-2">
          <form onSubmit={handleSubmit} className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm text-surface-400 mb-1">ط§ظ„طھط§ط±ظٹط®</label>
                <input type="date" value={date} onChange={(e) => setDate(e.target.value)} className="w-full input-field" required aria-label="ط§ظ„طھط§ط±ظٹط®" />
              </div>
              <div>
                <label className="block text-sm text-surface-400 mb-1">ط§ظ„ظ…ط¨ظ„ط؛ (ط¨ط§ظ„ظ…ظٹظ„ظٹ)</label>
                <input type="number" value={amountMilli} onChange={(e) => setAmountMilli(e.target.value)} className="w-full input-field" placeholder="0" min="1" required aria-label="ط§ظ„ظ…ط¨ظ„ط؛ ط¨ط§ظ„ظ…ظٹظ„ظٹ" />
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm text-surface-400 mb-1">ط·ط±ظٹظ‚ط© ط§ظ„ط¯ظپط¹</label>
                <select value={method} onChange={(e) => setMethod(e.target.value)} className="w-full input-field" aria-label="ط·ط±ظٹظ‚ط© ط§ظ„ط¯ظپط¹">
                  <option value="cash">ظ†ظ‚ط¯ظٹ</option>
                  <option value="bank_transfer">طھط­ظˆظٹظ„ ط¨ظ†ظƒظٹ</option>
                  <option value="cheque">ط´ظٹظƒ</option>
                </select>
              </div>
              <div>
                <label className="block text-sm text-surface-400 mb-1">ط±ظ‚ظ… ط§ظ„ط­ط³ط§ط¨ (ط§ط®طھظٹط§ط±ظٹ)</label>
                <input type="number" value={cashbankId} onChange={(e) => setCashbankId(e.target.value)} className="w-full input-field" placeholder="â€”" aria-label="ط±ظ‚ظ… ط§ظ„ط­ط³ط§ط¨" />
              </div>
            </div>

            <div>
              <label className="block text-sm text-surface-400 mb-1">ط§ظ„ظ…ط±ط¬ط¹ (ط§ط®طھظٹط§ط±ظٹ)</label>
              <input type="text" value={reference} onChange={(e) => setReference(e.target.value)} className="w-full input-field" placeholder="â€”" aria-label="ط§ظ„ظ…ط±ط¬ط¹" />
            </div>

            <div>
              <label className="block text-sm text-surface-400 mb-1">ظ…ظ„ط§ط­ط¸ط§طھ (ط§ط®طھظٹط§ط±ظٹ)</label>
              <textarea value={notes} onChange={(e) => setNotes(e.target.value)} className="w-full input-field" rows={3} placeholder="â€”" aria-label="ظ…ظ„ط§ط­ط¸ط§طھ" />
            </div>

            <div className="flex justify-end gap-2 pt-2">
              <Button type="button" variant="outline" onClick={() => navigate(`/customers/${id}`)}>ط¥ظ„ط؛ط§ط،</Button>
              <Button type="submit" loading={submitting} icon={<CreditCard className="w-4 h-4" />}>طھط³ط¬ظٹظ„ ط§ظ„ط¯ظپط¹ط©</Button>
            </div>
          </form>
        </Card>

        <Card>
          <h4 className="text-sm text-surface-400 mb-3">ظ…ط¹ظ„ظˆظ…ط§طھ ط§ظ„ط¹ظ…ظٹظ„</h4>
          <div className="space-y-4">
            <div className="text-center py-4">
              <p className="text-3xl font-bold gradient-text">{formatOMR(customer.balance_milli)}</p>
              <p className="text-xs text-surface-400 mt-1">ط§ظ„ط±طµظٹط¯ ط§ظ„ط­ط§ظ„ظٹ</p>
            </div>
            <div className="space-y-2">
              <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„ط§ط³ظ…</span><span className="font-medium">{customer.name}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„ظƒظˆط¯</span><span className="font-mono text-xs">{customer.code || "â€”"}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„ظ‡ط§طھظپ</span><span>{customer.phone || "â€”"}</span></div>
            </div>
          </div>
        </Card>
      </div>
    </div>
  );
}



