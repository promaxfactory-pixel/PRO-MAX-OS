import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, CreditCard } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { useTranslation } from "react-i18next";
import { Customer } from "@/types";

export default function CustomerPaymentPage() {
  const { t } = useTranslation();
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
      .catch((e: unknown) => addNotification({ title: t("common.error"), message: String(e), type: 'error' }))
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
      addNotification({ id: crypto.randomUUID(), type: "success", title: t("common.success"), message: t("customer.paymentSuccess") });
      navigate(`/customers/${id}`);
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("customer.paymentError") });
    } finally {
      setSubmitting(false);
    }
  };

  if (loading || !customer) {
    return <div className="flex items-center justify-center h-64"><div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" /></div>;
  }

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div className="flex items-center gap-3">
          <button onClick={() => navigate(`/customers/${id}`)} className="btn-ghost p-2"><ArrowRight className="w-5 h-5" /></button>
          <div>
            <h1 className="page-title">{t("customer.makePayment")}</h1>
            <p className="page-subtitle">{customer.name}</p>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-3 gap-6">
        <Card className="col-span-2">
          <form onSubmit={handleSubmit} className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm text-surface-400 mb-1">{t("common.date")}</label>
                <input type="date" value={date} onChange={(e) => setDate(e.target.value)} className="w-full input-field" required aria-label={t("common.date")} />
              </div>
              <div>
                <label className="block text-sm text-surface-400 mb-1">{t("customer.amountMilli")}</label>
                <input type="number" value={amountMilli} onChange={(e) => setAmountMilli(e.target.value)} className="w-full input-field" placeholder="0" min="1" required aria-label={t("customer.amountMilliAria")} />
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm text-surface-400 mb-1">{t("customer.paymentMethod")}</label>
                <select value={method} onChange={(e) => setMethod(e.target.value)} className="w-full input-field" aria-label={t("customer.paymentMethod")}>
                  <option value="cash">{t("customer.methodCash")}</option>
                  <option value="bank_transfer">{t("customer.methodBankTransfer")}</option>
                  <option value="cheque">{t("customer.methodCheque")}</option>
                </select>
              </div>
              <div>
                <label className="block text-sm text-surface-400 mb-1">{t("customer.accountNoOptional")}</label>
                <input type="number" value={cashbankId} onChange={(e) => setCashbankId(e.target.value)} className="w-full input-field" placeholder="—" aria-label={t("customer.accountNoAria")} />
              </div>
            </div>

            <div>
              <label className="block text-sm text-surface-400 mb-1">{t("customer.referenceOptional")}</label>
              <input type="text" value={reference} onChange={(e) => setReference(e.target.value)} className="w-full input-field" placeholder="—" aria-label={t("customer.referenceAria")} />
            </div>

            <div>
              <label className="block text-sm text-surface-400 mb-1">{t("customer.notesOptional")}</label>
              <textarea value={notes} onChange={(e) => setNotes(e.target.value)} className="w-full input-field" rows={3} placeholder="—" aria-label={t("common.notes")} />
            </div>

            <div className="flex justify-end gap-2 pt-2">
              <Button type="button" variant="outline" onClick={() => navigate(`/customers/${id}`)}>{t("common.cancel")}</Button>
              <Button type="submit" loading={submitting} icon={<CreditCard className="w-4 h-4" />}>{t("customer.makePayment")}</Button>
            </div>
          </form>
        </Card>

        <Card>
          <h4 className="text-sm text-surface-400 mb-3">{t("customer.customerInfo")}</h4>
          <div className="space-y-4">
            <div className="text-center py-4">
              <p className="text-3xl font-bold gradient-text">{formatOMR(customer.balance_milli)}</p>
              <p className="text-xs text-surface-400 mt-1">{t("customer.currentBalance")}</p>
            </div>
            <div className="space-y-2">
              <div className="flex justify-between text-sm"><span className="text-surface-400">{t("customer.name")}</span><span className="font-medium">{customer.name}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">{t("customer.code")}</span><span className="font-mono text-xs">{customer.code || "—"}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">{t("customer.phone")}</span><span>{customer.phone || "—"}</span></div>
            </div>
          </div>
        </Card>
      </div>
    </div>
  );
}
