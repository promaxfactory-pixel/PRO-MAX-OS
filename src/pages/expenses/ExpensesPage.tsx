import { useState, useEffect, useMemo, useCallback } from "react";
import DataTable, { Column } from "@/components/ui/DataTable";
import Button from "@/components/ui/Button";
import Card, { StatCard } from "@/components/ui/Card";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus, Save, Receipt, ArrowRight, Wallet, UserCheck, RefreshCw, CheckCircle, BadgeCheck } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

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

const PAYMENT_SOURCES = [
  { value: "company", label: "company (من حساب الشركة)", icon: "🏢" },
  { value: "custody", label: "عهدة (من رصيد العهدة)", icon: "💼" },
  { value: "personal", label: "شخصي (من جيب الموظف)", icon: "👤" },
];

const METHODS = [
  { value: "cash", label: "نقدي" },
  { value: "bank_transfer", label: "تحويل بنكي" },
  { value: "cheque", label: "شيك" },
];

export default function ExpensesPage() {
  const addNotification = useUIStore((s) => s.addNotification);
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
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء تحميل البيانات" });
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
      addNotification({ id: crypto.randomUUID(), type: "success", title: "نجاح", message: "تم تسجيل المصروف بنجاح" });
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: err instanceof Error ? err.message : "حدث خطأ أثناء الحفظ" });
    } finally {
      setSubmitting(false);
    }
  };

  const handleApprove = async (id: number) => {
    try {
      await invoke("approve_expense", { expenseId: id });
      loadExpenses();
      addNotification({ id: crypto.randomUUID(), type: "success", title: "تم", message: "تم اعتماد المصروف" });
    } catch {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "فشل الاعتماد" });
    }
  };

  const handleReimburse = async (id: number) => {
    try {
      await invoke("reimburse_expense", { expenseId: id, reimbursedBy: "النظام" });
      loadExpenses();
      addNotification({ id: crypto.randomUUID(), type: "success", title: "تم", message: "تم رد المبلغ بنجاح" });
    } catch {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "فشل رد المبلغ" });
    }
  };

  const totalExpenses = expenses.reduce((s, e) => s + (e.amount_milli || 0), 0);
  const companyExpenses = expenses.filter(e => !e.paid_from_source || e.paid_from_source === 'company').reduce((s, e) => s + (e.amount_milli || 0), 0);
  const custodyExpenses = expenses.filter(e => e.paid_from_source === 'custody').reduce((s, e) => s + (e.amount_milli || 0), 0);
  const personalExpenses = expenses.filter(e => e.paid_from_source === 'personal').reduce((s, e) => s + (e.amount_milli || 0), 0);
  const pendingReimburse = expenses.filter(e => e.reimbursement_status === 'pending').length;

  const methodLabel = (m: string) => METHODS.find((x) => x.value === m)?.label || m;
  const sourceLabel = (s: string | null) => {
    if (s === 'custody') return 'عهدة';
    if (s === 'personal') return 'شخصي';
    return 'الشركة';
  };
  const sourceColor = (s: string | null) => {
    if (s === 'custody') return 'bg-blue-500/20 text-blue-400';
    if (s === 'personal') return 'bg-amber-500/20 text-amber-400';
    return 'bg-emerald-500/20 text-emerald-400';
  };

  const columns: Column<Expense>[] = useMemo(() => [
    { key: "exp_no", header: "رقم المصروف", sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.exp_no || "—"}</span> },
    { key: "date", header: "التاريخ", sortable: true, render: (r) => formatDate(r.date) },
    { key: "category", header: "التصنيف", render: (r) => <span className="text-surface-300">{r.category}</span> },
    { key: "paid_from_source", header: "المصدر", render: (r) => <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${sourceColor(r.paid_from_source)}`}>{sourceLabel(r.paid_from_source)}</span> },
    { key: "paid_by_name", header: "الدافع", render: (r) => <span className="text-surface-300">{r.paid_by_name || r.vendor || "—"}</span> },
    { key: "amount_milli", header: "المبلغ", sortable: true, align: "left", render: (r) => <span className="font-bold text-gold-400 font-mono">{formatOMR(r.amount_milli)}</span> },
    { key: "method", header: "الطريقة", render: (r) => <span className="text-surface-300">{methodLabel(r.method)}</span> },
    { key: "approval_status", header: "الحالة", render: (r) => (
      <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
        r.approval_status === 'approved' ? 'bg-emerald-500/20 text-emerald-400' : 'bg-yellow-500/20 text-yellow-400'
      }`}>{r.approval_status === 'approved' ? '✓ معتمد' : '⏳ قيد المراجعة'}</span>
    )},
    { key: "reimbursement_status", header: "الرد", render: (r) => {
      if (r.paid_from_source !== 'personal') return <span className="text-surface-600">—</span>;
      if (r.reimbursement_status === 'reimbursed') return <span className="text-emerald-400 text-xs font-medium">✓ تم الرد</span>;
      if (r.reimbursement_status === 'pending') return <span className="text-amber-400 text-xs font-medium">⏳ ينتظر الرد</span>;
      return <span className="text-surface-600">—</span>;
    }},
    { key: "id", header: "", render: (r) => (
      <div className="flex items-center gap-1">
        {r.approval_status !== 'approved' && (
          <button onClick={() => handleApprove(r.id)} className="p-1 rounded-lg hover:bg-emerald-500/10 text-emerald-400 transition-colors" title="اعتماد">
            <BadgeCheck className="w-4 h-4" />
          </button>
        )}
        {r.paid_from_source === 'personal' && r.reimbursement_status === 'pending' && (
          <button onClick={() => handleReimburse(r.id)} className="p-1 rounded-lg hover:bg-blue-500/10 text-blue-400 transition-colors" title="رد المبلغ">
            <RefreshCw className="w-4 h-4" />
          </button>
        )}
      </div>
    )},
  ], [expenses]);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">المصروفات</h1>
          <p className="page-subtitle">{expenses.length} مصروف — تتبع شامل للمصروفات والمصادر</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => setShowForm(!showForm)}>
          {showForm ? "إخفاء" : "مصروف جديد"}
        </Button>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-4 gap-4">
        <StatCard title="إجمالي المصروفات" value={formatOMR(totalExpenses)} icon={<Receipt className="w-6 h-6" />} />
        <StatCard title="من حساب الشركة" value={formatOMR(companyExpenses)} icon={<Wallet className="w-6 h-6" />} />
        <StatCard title="من العهد" value={formatOMR(custodyExpenses)} icon={<Wallet className="w-6 h-6" />} />
        <StatCard title="شخصي (ينتظر رد)" value={`${formatOMR(personalExpenses)} — ${pendingReimburse} قيد`} icon={<UserCheck className="w-6 h-6" />} />
      </div>

      {showForm && (
        <Card className="p-6 relative overflow-hidden">
          <div className="absolute top-0 left-0 w-full h-1 bg-gradient-to-l from-gold-400 via-brand-500 to-gold-400" />
          <h2 className="text-lg font-semibold text-white mb-6">مصروف جديد</h2>
          <div className="grid grid-cols-3 gap-6">
            {/* Basic Info */}
            <div className="input-group">
              <label className="input-label">التاريخ *</label>
              <input type="date" className="input-field" value={date} onChange={(e) => setDate(e.target.value)} aria-label="التاريخ" />
            </div>
            <div className="input-group">
              <label className="input-label">التصنيف *</label>
              <select className="input-field" value={category} onChange={(e) => setCategory(e.target.value)} aria-label="التصنيف">
                {CATEGORIES.map((c) => <option key={c} value={c}>{c}</option>)}
              </select>
            </div>
            <div className="input-group">
              <label className="input-label">طريقة الدفع *</label>
              <select className="input-field" value={method} onChange={(e) => setMethod(e.target.value)} aria-label="طريقة الدفع">
                {METHODS.map((m) => <option key={m.value} value={m.value}>{m.label}</option>)}
              </select>
            </div>

            {/* Amount */}
            <div className="input-group">
              <label className="input-label">المبلغ (ملي) *</label>
              <input type="number" className="input-field font-mono text-gold-400 font-bold" min={0} value={amountMilli} onChange={(e) => setAmountMilli(Number(e.target.value))} aria-label="المبلغ" />
            </div>
            <div className="input-group">
              <label className="input-label">الضريبة (ملي)</label>
              <input type="number" className="input-field" min={0} value={vatMilli} onChange={(e) => setVatMilli(Number(e.target.value))} aria-label="الضريبة" />
            </div>
            <div className="input-group">
              <label className="input-label">رمز الحساب</label>
              <input type="text" className="input-field" value={accountCode} onChange={(e) => setAccountCode(e.target.value)} placeholder="اختياري" aria-label="رمز الحساب" />
            </div>

            {/* Payment Source - Key Feature */}
            <div className="col-span-3 p-4 bg-surface-800/50 rounded-2xl border border-surface-700/30">
              <label className="input-label mb-3 block flex items-center gap-2">
                <Wallet className="w-4 h-4 text-gold-400" />
                مصدر الدفع *
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
                    <span className="text-sm font-medium block">{src.label.split('(')[0].trim()}</span>
                  </button>
                ))}
              </div>

              {/* Custody selector */}
              {paidFromSource === 'custody' && (
                <div className="mt-4 grid grid-cols-2 gap-4">
                  <div className="input-group">
                    <label className="input-label">العهدة *</label>
                    <select className="input-field" value={pettyId || ''} onChange={(e) => setPettyId(Number(e.target.value) || null)} aria-label="اختر العهدة">
                      <option value="">اختر العهدة...</option>
                      {custodyAccounts.map((c) => <option key={c.id} value={c.id}>{c.name} ({c.code})</option>)}
                    </select>
                  </div>
                  <div className="input-group">
                    <label className="input-label">الدافع *</label>
                    <select className="input-field" value={paidByEmployeeId || ''} onChange={(e) => setPaidByEmployeeId(Number(e.target.value) || null)} aria-label="اختر الدافع">
                      <option value="">اختر الشخص...</option>
                      {employees.map((e) => <option key={e.id} value={e.id}>{e.name} ({e.code})</option>)}
                    </select>
                  </div>
                </div>
              )}

              {/* Personal payment - employee selector */}
              {paidFromSource === 'personal' && (
                <div className="mt-4">
                  <div className="input-group">
                    <label className="input-label">الموظف الذي دفع من جيبه *</label>
                    <select className="input-field" value={paidByEmployeeId || ''} onChange={(e) => setPaidByEmployeeId(Number(e.target.value) || null)} aria-label="اختر الموظف">
                      <option value="">اختر الموظف...</option>
                      {employees.map((e) => <option key={e.id} value={e.id}>{e.name} ({e.code})</option>)}
                    </select>
                  </div>
                  <p className="text-xs text-amber-400 mt-2">⚠ سيتم تسجيل هذا كمبلغ ينتظر الرد من الشركة للموظف</p>
                </div>
              )}
            </div>

            {/* Vendor & Reference */}
            <div className="input-group">
              <label className="input-label">المورد / الجهة</label>
              <input type="text" className="input-field" value={vendor} onChange={(e) => setVendor(e.target.value)} placeholder="اختياري" aria-label="المورد" />
            </div>
            <div className="input-group">
              <label className="input-label">المرجع</label>
              <input type="text" className="input-field" value={reference} onChange={(e) => setReference(e.target.value)} placeholder="اختياري" aria-label="المرجع" />
            </div>
            <div className="input-group">
              <label className="input-label">ملاحظات</label>
              <input type="text" className="input-field" value={notes} onChange={(e) => setNotes(e.target.value)} placeholder="اختياري" aria-label="ملاحظات" />
            </div>
          </div>

          <div className="flex justify-end gap-3 mt-6 pt-4 border-t border-surface-700/30">
            <Button variant="ghost" icon={<ArrowRight className="w-4 h-4" />} onClick={() => { resetForm(); setShowForm(false); }}>إلغاء</Button>
            <Button icon={<Save className="w-4 h-4" />} onClick={handleSubmit} disabled={submitting}>
              {submitting ? "جاري الحفظ..." : "حفظ المصروف"}
            </Button>
          </div>
        </Card>
      )}

      <DataTable columns={columns} data={expenses} loading={loading} emptyMessage="لا توجد مصروفات" />
    </div>
  );
}
