import { useState, useEffect, useCallback, useMemo } from "react";
import DataTable, { Column } from "@/components/ui/DataTable";
import Card, { StatCard } from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { useUIStore } from "@/stores/uiStore";
import { Plus, CheckCircle2, AlertTriangle, Clock } from "lucide-react";
import { useTranslation } from "react-i18next";

interface InstallmentPayment {
  id: number;
  installment_name: string;
  installment_number: number;
  due_date: string;
  amount_milli: number;
  paid_milli: number;
  status: string;
  penalty_milli: number;
  paid_date: string;
  notes: string;
  created_at: string;
}

interface Installment {
  id: number;
  name: string;
  total_amount_milli: number;
  number_of_installments: number;
  supplier_name: string;
}

const EMPTY_PAYMENT_FORM = {
  installment_id: 0,
  installment_name: "",
  installment_number: 0,
  due_date: "",
  amount_milli: 0,
  notes: "",
};

export default function InstallmentTrackingPage() {
  const { t } = useTranslation();
  const { addNotification } = useUIStore();
  const [payments, setPayments] = useState<InstallmentPayment[]>([]);
  const [, setInstallments] = useState<Installment[]>([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [form, setForm] = useState(EMPTY_PAYMENT_FORM);
  const [submitting, setSubmitting] = useState(false);
  const [payingId, setPayingId] = useState<number | null>(null);

  const loadPayments = useCallback(async () => {
    setLoading(true);
    try {
      const d = await invoke<InstallmentPayment[]>("list_installment_payments");
      setPayments(d);
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("installments.loadError", { error: String(err) }) });
    } finally {
      setLoading(false);
    }
  }, [addNotification]);

  const loadInstallments = useCallback(async () => {
    try {
      const d = await invoke<Installment[]>("list_installments");
      setInstallments(d);
    } catch { /* optional command */ }
  }, []);

  useEffect(() => {
    loadPayments();
    loadInstallments();
  }, [loadPayments, loadInstallments]);

  const handleCreate = async () => {
    if (!form.installment_name || !form.due_date || form.amount_milli <= 0) return;
    setSubmitting(true);
    try {
      await invoke("create_installment_payment", { input: form });
      setShowForm(false);
      setForm(EMPTY_PAYMENT_FORM);
      await loadPayments();
      addNotification({ id: crypto.randomUUID(), type: "success", title: t("installments.done"), message: t("installments.paymentCreated") });
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("installments.createFailed", { error: String(err) }) });
    }
    setSubmitting(false);
  };

  const handleMarkPaid = async (paymentId: number) => {
    setPayingId(paymentId);
    try {
      await invoke("mark_installment_paid", { paymentId });
      await loadPayments();
      addNotification({ id: crypto.randomUUID(), type: "success", title: t("installments.done"), message: t("installments.markedPaid") });
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("installments.markPaidFailed", { error: String(err) }) });
    }
    setPayingId(null);
  };

  const totalPaid = payments.filter((p) => p.status === "Paid").reduce((s, p) => s + p.paid_milli, 0);
  const totalPending = payments.filter((p) => p.status !== "Paid").reduce((s, p) => s + (p.amount_milli - p.paid_milli), 0);
  const overdueCount = payments.filter((p) => p.status !== "Paid" && new Date(p.due_date) < new Date()).length;

  const columns: Column<InstallmentPayment>[] = useMemo(() => [
    { key: "installment_name", header: t("installments.loanName"), sortable: true, render: (r) => <span className="font-medium text-white">{r.installment_name || "—"}</span> },
    { key: "installment_number", header: t("installments.installmentNumber"), sortable: true, align: "center", render: (r) => <span className="font-mono text-brand-400">{r.installment_number}</span> },
    { key: "due_date", header: t("installments.dueDate"), sortable: true, render: (r) => {
      const isOverdue = r.status !== "Paid" && new Date(r.due_date) < new Date();
      return <span className={isOverdue ? "text-red-400 font-medium" : ""}>{formatDate(r.due_date)}</span>;
    }},
    { key: "amount_milli", header: t("installments.amount"), sortable: true, align: "left", render: (r) => <span className="font-bold text-gold-400">{formatOMR(r.amount_milli)}</span> },
    { key: "paid_milli", header: t("installments.paid"), align: "left", render: (r) => formatOMR(r.paid_milli) },
    { key: "status", header: t("common.status"), render: (r) => {
      const isOverdue = r.status !== "Paid" && new Date(r.due_date) < new Date();
      return (
        <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
          r.status === "Paid" ? "bg-emerald-500/20 text-emerald-400" :
          isOverdue ? "bg-red-500/20 text-red-400" :
          "bg-amber-500/20 text-amber-400"
        }`}>{r.status === "Paid" ? t("installments.paidStatus") : isOverdue ? t("installments.overdue") : t("installments.pending")}</span>
      );
    }},
    { key: "penalty_milli", header: t("installments.penalty"), align: "left", render: (r) => r.penalty_milli > 0 ? <span className="text-red-400 font-medium">{formatOMR(r.penalty_milli)}</span> : "—" },
    { key: "id", header: "", align: "center", width: "80px", render: (r) => r.status !== "Paid" ? (
      <Button variant="success" size="sm" loading={payingId === r.id} onClick={(e) => { e.stopPropagation(); handleMarkPaid(r.id); }}>
        <CheckCircle2 className="w-3 h-3" /> {t("installments.markPaid")}
      </Button>
    ) : null },
  ], [t, payingId]);

  return (
    <div className="space-y-6" dir="rtl">
      <div className="page-header">
        <div>
          <h1 className="page-title">{t("installments.title")}</h1>
          <p className="page-subtitle">{t("installments.subtitle", { count: payments.length })}</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => setShowForm(!showForm)}>{t("installments.newSchedule")}</Button>
      </div>

      <div className="grid grid-cols-3 gap-4">
        <StatCard title={t("installments.totalPaid")} value={formatOMR(totalPaid)} icon={<CheckCircle2 className="w-6 h-6" />} />
        <StatCard title={t("common.remaining")} value={formatOMR(totalPending)} icon={<Clock className="w-6 h-6" />} />
        <StatCard title={t("installments.overdueLoans")} value={overdueCount} icon={<AlertTriangle className="w-6 h-6" />} className={overdueCount > 0 ? "border-red-500/30" : ""} />
      </div>

      {showForm && (
        <Card className="border-brand-500/30">
          <h3 className="section-title mb-4"><Plus className="w-4 h-4" /> {t("installments.newSchedule")}</h3>
          <div className="grid grid-cols-3 gap-4">
            <div>
              <label className="form-label">{t("installments.loanNameRequired")}</label>
              <input className="input-field" value={form.installment_name} onChange={(e) => setForm({ ...form, installment_name: e.target.value })} placeholder={t("installments.loanNamePlaceholder")} aria-label={t("installments.loanName")} />
            </div>
            <div>
              <label className="form-label">{t("installments.installmentNumber")}</label>
              <input type="number" className="input-field" value={form.installment_number || ""} onChange={(e) => setForm({ ...form, installment_number: Number(e.target.value) || 0 })} aria-label={t("installments.installmentNumber")} />
            </div>
            <div>
              <label className="form-label">{t("installments.dueDateRequired")}</label>
              <input type="date" className="input-field" value={form.due_date} onChange={(e) => setForm({ ...form, due_date: e.target.value })} aria-label={t("installments.dueDate")} />
            </div>
            <div>
              <label className="form-label">{t("installments.amountMilliRequired")}</label>
              <input type="number" className="input-field" value={form.amount_milli || ""} onChange={(e) => setForm({ ...form, amount_milli: Number(e.target.value) || 0 })} aria-label={t("installments.amount")} />
            </div>
            <div className="col-span-2">
              <label className="form-label">{t("common.notes")}</label>
              <textarea className="input-field" rows={2} value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} aria-label={t("common.notes")} />
            </div>
          </div>
          <div className="flex justify-end gap-2 mt-4">
            <Button variant="ghost" onClick={() => setShowForm(false)}>{t("common.cancel")}</Button>
            <Button variant="gold" loading={submitting} onClick={handleCreate} disabled={!form.installment_name || !form.due_date || form.amount_milli <= 0}>{t("installments.addPayment")}</Button>
          </div>
        </Card>
      )}

      <DataTable columns={columns} data={payments} loading={loading} emptyMessage={t("installments.empty")} />
    </div>
  );
}
