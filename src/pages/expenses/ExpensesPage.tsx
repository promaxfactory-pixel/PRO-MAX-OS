import { useState, useEffect, useMemo, useCallback } from "react";
import DataTable, { Column } from "@/components/ui/DataTable";
import Button from "@/components/ui/Button";
import Card, { StatCard } from "@/components/ui/Card";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus, Save, Receipt, ArrowRight, Wallet, UserCheck, RefreshCw, BadgeCheck } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { useTranslation } from "react-i18next";

interface Expense {
  id: number; exp_no: string; date: string; category: string; account_code: string;
  amount_milli: number; vat_milli: number; method: string; vendor: string; reference: string;
  notes: string; approval_status: string;
  paid_by_employee_id: number | null; paid_by_name: string | null;
  paid_from_source: string | null; petty_id: number | null; petty_name: string | null;
  custody_txn_id: number | null; reimbursement_status: string | null;
  reimbursement_date: string | null; reimbursed_by: string | null;
  created_by: string | null; created_at: string | null;
}

interface EmployeeSelect { id: number; name: string; code: string | null; }

const CATEGORIES = ["أجر", "إيجار", "مواصلات", "كهرباء", "مياه", "صيانة", "مكاتب", "اتصالات", "تأمين", "أخرى"];

export default function ExpensesPage() {
  const { t } = useTranslation();
  const addNotification = useUIStore((s) => s.addNotification);

  const catLabel = (value: string) => {
    const map: Record<string, string> = {
      "أجر": t("expenses.cat.salary"), "إيجار": t("expenses.cat.rent"), "مواصلات": t("expenses.cat.transport"),
      "كهرباء": t("expenses.cat.electricity"), "مياه": t("expenses.cat.water"), "صيانة": t("expenses.cat.maintenance"),
      "مكاتب": t("expenses.cat.office"), "اتصالات": t("expenses.cat.communications"), "تأمين": t("expenses.cat.insurance"), "أخرى": t("expenses.cat.other"),
    };
    return map[value] || value;
  };
  const PAYMENT_SOURCES = [
    { value: "company", label: t("expenses.sourceCompany"), icon: "🏢" },
    { value: "custody", label: t("expenses.sourceCustody"), icon: "💼" },
    { value: "personal", label: t("expenses.sourcePersonal"), icon: "👤" },
  ];
  const METHODS = [
    { value: "cash", label: t("expenses.methodCash") },
    { value: "bank_transfer", label: t("expenses.methodBankTransfer") },
    { value: "cheque", label: t("expenses.methodCheque") },
  ];
  const [expenses, setExpenses] = useState<Expense[]>([]);
  const [employees, setEmployees] = useState<EmployeeSelect[]>([]);
  const [custodyAccounts, setCustodyAccounts] = useState<EmployeeSelect[]>([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [submitting, setSubmitting] = useState(false);

  const [date, setDate] = useState(new Date().toISOString().split("T")[0]);
  const [category, setCategory] = useState(CATEGORIES[0]);
  const [accountCode, setAccountCode] = useState("");
  const [amountMilli, setAmountMilli] = useState(0);
  const [vatMilli, setVatMilli] = useState(0);
  const [method, setMethod] = useState("cash");
  const [vendor, setVendor] = useState("");
  const [reference, setReference] = useState("");
  const [notes, setNotes] = useState("");
  const [paidFromSource, setPaidFromSource] = useState("company");
  const [paidByEmployeeId, setPaidByEmployeeId] = useState<number | null>(null);
  const [pettyId, setPettyId] = useState<number | null>(null);

  const loadExpenses = useCallback(async () => {
    setLoading(true);
    try {
      const [expData, empData, custData] = await Promise.all([
        invoke("list_expenses") as Promise<Expense[]>,
        invoke("list_employees_for_select").catch(() => []) as Promise<EmployeeSelect[]>,
        invoke("get_custody_accounts_for_select").catch(() => []) as Promise<EmployeeSelect[]>,
      ]);
      setExpenses(expData);
      setEmployees(empData);
      setCustodyAccounts(custData);
    } catch {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("expenses.loadError") });
    } finally {
      setLoading(false);
    }
  }, [addNotification]);

  useEffect(() => { loadExpenses(); }, [loadExpenses]);

  const resetForm = () => {
    setDate(new Date().toISOString().split("T")[0]);
    setCategory(CATEGORIES[0]); setAccountCode(""); setAmountMilli(0); setVatMilli(0);
    setMethod("cash"); setVendor(""); setReference(""); setNotes("");
    setPaidFromSource("company"); setPaidByEmployeeId(null); setPettyId(null);
  };

  const handleSubmit = async () => {
    if (!date || !category || amountMilli <= 0) return;
    setSubmitting(true);
    try {
      await invoke("create_expense", {
        input: {
          date, category, account_code: accountCode || null,
          amount_milli: amountMilli, vat_milli: vatMilli || 0, method,
          vendor: vendor || null, reference: reference || null, notes: notes || null,
          paid_by_employee_id: paidByEmployeeId, paid_from_source: paidFromSource,
          petty_id: pettyId,
        },
      });
      resetForm(); setShowForm(false); loadExpenses();
      addNotification({ id: crypto.randomUUID(), type: "success", title: t("common.success"), message: t("expenses.saveSuccess") });
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: err instanceof Error ? err.message : t("expenses.saveError") });
    } finally {
      setSubmitting(false);
    }
  };

  const handleApprove = async (id: number) => {
    try {
      await invoke("approve_expense", { expenseId: id });
      loadExpenses();
      addNotification({ id: crypto.randomUUID(), type: "success", title: t("common.success"), message: t("expenses.approveSuccess") });
    } catch {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("expenses.approveFailed") });
    }
  };

  const handleReimburse = async (id: number) => {
    try {
      await invoke("reimburse_expense", { expenseId: id, reimbursedBy: t("common.system") });
      loadExpenses();
      addNotification({ id: crypto.randomUUID(), type: "success", title: t("common.success"), message: t("expenses.reimburseSuccess") });
    } catch {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("expenses.reimburseFailed") });
    }
  };

  const totalExpenses = expenses.reduce((s, e) => s + (e.amount_milli || 0), 0);
  const companyExpenses = expenses.filter(e => !e.paid_from_source || e.paid_from_source === 'company').reduce((s, e) => s + (e.amount_milli || 0), 0);
  const custodyExpenses = expenses.filter(e => e.paid_from_source === 'custody').reduce((s, e) => s + (e.amount_milli || 0), 0);
  const personalExpenses = expenses.filter(e => e.paid_from_source === 'personal').reduce((s, e) => s + (e.amount_milli || 0), 0);
  const pendingReimburse = expenses.filter(e => e.reimbursement_status === 'pending').length;

  const methodLabel = (m: string) => METHODS.find((x) => x.value === m)?.label || m;
  const sourceLabel = (s: string | null) => {
    if (s === 'custody') return t('expenses.sourceCustody');
    if (s === 'personal') return t('expenses.sourcePersonal');
    return t('expenses.sourceCompany');
  };
  const sourceColor = (s: string | null) => {
    if (s === 'custody') return 'bg-blue-500/20 text-blue-400';
    if (s === 'personal') return 'bg-amber-500/20 text-amber-400';
    return 'bg-emerald-500/20 text-emerald-400';
  };

  const columns: Column<Expense>[] = useMemo(() => [
    { key: "exp_no", header: t("expenses.expNo"), sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.exp_no || "—"}</span> },
    { key: "date", header: t("expenses.dateColumn"), sortable: true, render: (r) => formatDate(r.date) },
    { key: "category", header: t("expenses.categoryColumn"), render: (r) => <span className="text-surface-300">{catLabel(r.category)}</span> },
    { key: "paid_from_source", header: t("expenses.source"), render: (r) => <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${sourceColor(r.paid_from_source)}`}>{sourceLabel(r.paid_from_source)}</span> },
    { key: "paid_by_name", header: t("expenses.payerColumn"), render: (r) => <span className="text-surface-300">{r.paid_by_name || r.vendor || "—"}</span> },
    { key: "amount_milli", header: t("expenses.amountColumn"), sortable: true, align: "left", render: (r) => <span className="font-bold text-gold-400 font-mono">{formatOMR(r.amount_milli)}</span> },
    { key: "method", header: t("expenses.method"), render: (r) => <span className="text-surface-300">{methodLabel(r.method)}</span> },
    { key: "approval_status", header: t("expenses.statusColumn"), render: (r) => (
      <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
        r.approval_status === 'approved' ? 'bg-emerald-500/20 text-emerald-400' : 'bg-yellow-500/20 text-yellow-400'
      }`}>{r.approval_status === 'approved' ? t('expenses.approved') : t('expenses.underReview')}</span>
    )},
    { key: "reimbursement_status", header: t("expenses.reimbursement"), render: (r) => {
      if (r.paid_from_source !== 'personal') return <span className="text-surface-600">—</span>;
      if (r.reimbursement_status === 'reimbursed') return <span className="text-emerald-400 text-xs font-medium">{t('expenses.reimbursed')}</span>;
      if (r.reimbursement_status === 'pending') return <span className="text-amber-400 text-xs font-medium">{t('expenses.awaitingReimbursement')}</span>;
      return <span className="text-surface-600">—</span>;
    }},
    { key: "id", header: "", render: (r) => (
      <div className="flex items-center gap-1">
        {r.approval_status !== 'approved' && (
          <button onClick={() => handleApprove(r.id)} className="p-1 rounded-lg hover:bg-emerald-500/10 text-emerald-400 transition-colors" title={t("expenses.approveAction")}>
            <BadgeCheck className="w-4 h-4" />
          </button>
        )}
        {r.paid_from_source === 'personal' && r.reimbursement_status === 'pending' && (
          <button onClick={() => handleReimburse(r.id)} className="p-1 rounded-lg hover:bg-blue-500/10 text-blue-400 transition-colors" title={t("expenses.reimburseAction")}>
            <RefreshCw className="w-4 h-4" />
          </button>
        )}
      </div>
    )},
  ], [t, expenses, catLabel]);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">{t("expenses.title")}</h1>
          <p className="page-subtitle">{t("expenses.subtitle", { count: expenses.length })}</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => setShowForm(!showForm)}>
          {showForm ? t("expenses.hide") : t("expenses.newExpense")}
        </Button>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-4 gap-4">
        <StatCard title={t("expenses.totalExpenses")} value={formatOMR(totalExpenses)} icon={<Receipt className="w-6 h-6" />} />
        <StatCard title={t("expenses.fromCompany")} value={formatOMR(companyExpenses)} icon={<Wallet className="w-6 h-6" />} />
        <StatCard title={t("expenses.fromCustody")} value={formatOMR(custodyExpenses)} icon={<Wallet className="w-6 h-6" />} />
        <StatCard title={t("expenses.personalPending")} value={`${formatOMR(personalExpenses)} — ${t("expenses.pendingCount", { count: pendingReimburse })}`} icon={<UserCheck className="w-6 h-6" />} />
      </div>

      {showForm && (
        <Card className="p-6 relative overflow-hidden">
          <div className="absolute top-0 left-0 w-full h-1 bg-gradient-to-l from-gold-400 via-brand-500 to-gold-400" />
          <h2 className="text-lg font-semibold text-white mb-6">{t("expenses.formTitle")}</h2>
          <div className="grid grid-cols-3 gap-6">
            {/* Basic Info */}
            <div className="input-group">
              <label className="input-label">{t("expenses.date")}</label>
              <input type="date" className="input-field" value={date} onChange={(e) => setDate(e.target.value)} aria-label={t("common.date")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("expenses.category")}</label>
              <select className="input-field" value={category} onChange={(e) => setCategory(e.target.value)} aria-label={t("expenses.categoryColumn")}>
                {CATEGORIES.map((c) => <option key={c} value={c}>{catLabel(c)}</option>)}
              </select>
            </div>
            <div className="input-group">
              <label className="input-label">{t("expenses.paymentMethod")}</label>
              <select className="input-field" value={method} onChange={(e) => setMethod(e.target.value)} aria-label={t("expenses.paymentMethod")}>
                {METHODS.map((m) => <option key={m.value} value={m.value}>{m.label}</option>)}
              </select>
            </div>

            {/* Amount */}
            <div className="input-group">
              <label className="input-label">{t("expenses.amountMilli")}</label>
              <input type="number" className="input-field font-mono text-gold-400 font-bold" min={0} value={amountMilli} onChange={(e) => setAmountMilli(Number(e.target.value))} aria-label={t("expenses.amountAria")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("expenses.vatMilli")}</label>
              <input type="number" className="input-field" min={0} value={vatMilli} onChange={(e) => setVatMilli(Number(e.target.value))} aria-label={t("expenses.vatAria")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("expenses.accountCode")}</label>
              <input type="text" className="input-field" value={accountCode} onChange={(e) => setAccountCode(e.target.value)} placeholder={t("expenses.optional")} aria-label={t("expenses.accountCodeAria")} />
            </div>

            {/* Payment Source - Key Feature */}
            <div className="col-span-3 p-4 bg-surface-800/50 rounded-2xl border border-surface-700/30">
              <label className="input-label mb-3 block flex items-center gap-2">
                <Wallet className="w-4 h-4 text-gold-400" />
                {t("expenses.paymentSource")}
              </label>
              <div className="grid grid-cols-3 gap-3">
                {PAYMENT_SOURCES.map((src) => (
                  <button
                    key={src.value}
                    type="button"
                    onClick={() => { setPaidFromSource(src.value); if (src.value !== 'custody') setPettyId(null); if (src.value === 'company') setPaidByEmployeeId(null); }}
                    className={`p-4 rounded-xl border-2 text-center transition-all ${
                      paidFromSource === src.value
                        ? 'border-gold-400/50 bg-gold-400/5 text-white shadow-[0_0_15px_rgba(212,175,55,0.1)]'
                        : 'border-surface-700/50 bg-surface-800/30 text-surface-400 hover:border-surface-600'
                    }`}
                  >
                    <span className="text-2xl block mb-2">{src.icon}</span>
                    <span className="text-sm font-medium block">{src.label}</span>
                  </button>
                ))}
              </div>

              {/* Custody selector */}
              {paidFromSource === 'custody' && (
                <div className="mt-4 grid grid-cols-2 gap-4">
                  <div className="input-group">
                    <label className="input-label">{t("expenses.custody")}</label>
                    <select className="input-field" value={pettyId || ''} onChange={(e) => setPettyId(Number(e.target.value) || null)} aria-label={t("expenses.selectCustodyAria")}>
                      <option value="">{t("expenses.selectCustody")}</option>
                      {custodyAccounts.map((c) => <option key={c.id} value={c.id}>{c.name} ({c.code})</option>)}
                    </select>
                  </div>
                  <div className="input-group">
                    <label className="input-label">{t("expenses.payer")}</label>
                    <select className="input-field" value={paidByEmployeeId || ''} onChange={(e) => setPaidByEmployeeId(Number(e.target.value) || null)} aria-label={t("expenses.selectPayerAria")}>
                      <option value="">{t("expenses.selectPayer")}</option>
                      {employees.map((e) => <option key={e.id} value={e.id}>{e.name} ({e.code})</option>)}
                    </select>
                  </div>
                </div>
              )}

              {/* Personal payment - employee selector */}
              {paidFromSource === 'personal' && (
                <div className="mt-4">
                  <div className="input-group">
                    <label className="input-label">{t("expenses.personalPayer")}</label>
                    <select className="input-field" value={paidByEmployeeId || ''} onChange={(e) => setPaidByEmployeeId(Number(e.target.value) || null)} aria-label={t("expenses.selectEmployeeAria")}>
                      <option value="">{t("expenses.selectEmployee")}</option>
                      {employees.map((e) => <option key={e.id} value={e.id}>{e.name} ({e.code})</option>)}
                    </select>
                  </div>
                  <p className="text-xs text-amber-400 mt-2">{t("expenses.personalNotice")}</p>
                </div>
              )}
            </div>

            {/* Vendor & Reference */}
            <div className="input-group">
              <label className="input-label">{t("expenses.vendor")}</label>
              <input type="text" className="input-field" value={vendor} onChange={(e) => setVendor(e.target.value)} placeholder={t("expenses.optional")} aria-label={t("expenses.vendorAria")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("expenses.reference")}</label>
              <input type="text" className="input-field" value={reference} onChange={(e) => setReference(e.target.value)} placeholder={t("expenses.optional")} aria-label={t("expenses.referenceAria")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("expenses.notes")}</label>
              <input type="text" className="input-field" value={notes} onChange={(e) => setNotes(e.target.value)} placeholder={t("expenses.optional")} aria-label={t("expenses.notesAria")} />
            </div>
          </div>

          <div className="flex justify-end gap-3 mt-6 pt-4 border-t border-surface-700/30">
            <Button variant="ghost" icon={<ArrowRight className="w-4 h-4" />} onClick={() => { resetForm(); setShowForm(false); }}>{t("expenses.cancel")}</Button>
            <Button icon={<Save className="w-4 h-4" />} onClick={handleSubmit} disabled={submitting}>
              {submitting ? t("expenses.saving") : t("expenses.saveExpense")}
            </Button>
          </div>
        </Card>
      )}

      <DataTable columns={columns} data={expenses} loading={loading} emptyMessage={t("expenses.empty")} />
    </div>
  );
}
