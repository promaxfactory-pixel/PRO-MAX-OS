import { useState, useEffect, useMemo, useCallback } from "react";
import DataTable, { Column } from "@/components/ui/DataTable";
import Badge from "@/components/ui/Badge";
import Button from "@/components/ui/Button";
import Card from "@/components/ui/Card";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@/lib/tauri";
import { Plus, HandCoins, AlertTriangle } from "lucide-react";
import ConfirmDialog from "@/components/ui/ConfirmDialog";
import { useUIStore } from "@/stores/uiStore";
import type { Employee } from "@/types";

interface EmployeeAdvance {
  id: number;
  employee_id: number;
  employee_name: string | null;
  amount_milli: number;
  date: string;
  reason: string | null;
  remaining_milli: number;
  deduction_per_payroll_milli: number;
  status: string | null;
  source_type: string | null;
  source_id: number | null;
  journal_id: number | null;
}

interface CashbankAccount {
  id: number;
  code: string | null;
  name: string;
  atype: string | null;
  balance_milli: number;
  active: number;
}

interface CustodyAccount {
  id: number;
  name: string;
  code: string | null;
}

const SOURCE_OPTIONS = [
  { value: "company", label: "حساب الشركة / الصندوق أو البنك" },
  { value: "custody", label: "عهدة موظف / صندوق نثري" },
  { value: "owner_saif", label: "سيف محمد دفع أو موّل السلفة" },
  { value: "owner_abu_saif", label: "أبو سيف دفع أو موّل السلفة" },
];

const METHOD_OPTIONS = [
  { value: "cash", label: "نقدي" },
  { value: "bank_transfer", label: "تحويل بنكي" },
  { value: "cheque", label: "شيك" },
];

const emptyForm = () => ({
  employee_id: "",
  amount_omr: "",
  date: new Date().toISOString().split("T")[0],
  reason: "",
  deduction_per_payroll_omr: "",
  source_type: "company",
  method: "cash",
  cashbank_id: "",
  custody_id: "",
});

export default function EmployeeAdvancesPage() {
  const addNotification = useUIStore((s) => s.addNotification);
  const [advances, setAdvances] = useState<EmployeeAdvance[]>([]);
  const [employees, setEmployees] = useState<Employee[]>([]);
  const [cashbankAccounts, setCashbankAccounts] = useState<CashbankAccount[]>([]);
  const [custodyAccounts, setCustodyAccounts] = useState<CustodyAccount[]>([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [saving, setSaving] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);
  const [form, setForm] = useState(emptyForm);

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const [advData, empData, bankData, custodyData] = await Promise.all([
        invoke<EmployeeAdvance[]>("list_employee_advances"),
        invoke<Employee[]>("list_employees"),
        invoke<CashbankAccount[]>("list_cashbank_accounts").catch(() => []),
        invoke<CustodyAccount[]>("get_custody_accounts_for_select").catch(() => []),
      ]);
      setAdvances(advData);
      setEmployees(empData);
      setCashbankAccounts(bankData);
      setCustodyAccounts(custodyData);
    } catch (err) {
      addNotification({
        id: crypto.randomUUID(),
        type: "error",
        title: "خطأ",
        message: err instanceof Error ? err.message : "حدث خطأ أثناء تحميل بيانات السلف",
      });
    } finally {
      setLoading(false);
    }
  }, [addNotification]);

  useEffect(() => { loadData(); }, [loadData]);

  const resetForm = () => setForm(emptyForm());

  const handleCreate = async () => {
    const amountOmr = Number(form.amount_omr);
    const deductionOmr = form.deduction_per_payroll_omr.trim()
      ? Number(form.deduction_per_payroll_omr)
      : 0;

    if (!form.employee_id || !Number.isFinite(amountOmr) || amountOmr <= 0) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "بيانات غير مكتملة", message: "حدد العامل وأدخل قيمة سلفة صحيحة." });
      return;
    }
    if (!Number.isFinite(deductionOmr) || deductionOmr < 0 || deductionOmr > amountOmr) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "قيمة غير صحيحة", message: "الخصم المقترح لا يمكن أن يكون سالبًا أو أكبر من قيمة السلفة." });
      return;
    }
    if (form.source_type === "custody" && !form.custody_id) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "حدد العهدة", message: "يجب تحديد العهدة التي ستصرف منها السلفة." });
      return;
    }

    setSaving(true);
    try {
      await invoke("create_employee_advance", {
        input: {
          employee_id: Number(form.employee_id),
          amount_milli: Math.round(amountOmr * 1000),
          date: form.date,
          reason: form.reason.trim() || null,
          deduction_per_payroll_milli: Math.round(deductionOmr * 1000),
          source_type: form.source_type,
          method: form.method,
          cashbank_id: form.cashbank_id ? Number(form.cashbank_id) : null,
          custody_id: form.custody_id ? Number(form.custody_id) : null,
        },
      });
      setShowForm(false);
      resetForm();
      await loadData();
      addNotification({ id: crypto.randomUUID(), type: "success", title: "تم", message: "تم تسجيل السلفة وترحيلها محاسبيًا." });
    } catch (err) {
      addNotification({
        id: crypto.randomUUID(),
        type: "error",
        title: "تعذر إنشاء السلفة",
        message: err instanceof Error ? err.message : "فشل إنشاء السلفة أو ترحيلها المحاسبي.",
      });
    } finally {
      setSaving(false);
    }
  };

  const openAdvances = advances.filter((a) => (a.status || "").toLowerCase() === "open");
  const totalOutstanding = openAdvances.reduce((sum, item) => sum + (item.remaining_milli || 0), 0);
  const closedCount = advances.filter((a) => (a.status || "").toLowerCase() === "closed").length;
  const unpostedCount = advances.filter((a) => !a.journal_id).length;

  const sourceLabel = (source: string | null) => {
    if (source === "custody") return "عهدة";
    if (source === "owner_saif") return "سيف محمد";
    if (source === "owner_abu_saif") return "أبو سيف";
    if (source === "company") return "الشركة";
    return "قديم / غير محدد";
  };

  const columns: Column<EmployeeAdvance>[] = useMemo(() => [
    { key: "employee_name", header: "الموظف", sortable: true, render: (r) => <span className="font-medium">{r.employee_name || `#${r.employee_id}`}</span> },
    { key: "amount_milli", header: "قيمة السلفة", sortable: true, align: "left", render: (r) => <span className="font-mono">{formatOMR(r.amount_milli)}</span> },
    { key: "date", header: "التاريخ", sortable: true, render: (r) => formatDate(r.date) },
    { key: "source_type", header: "مصدر الصرف", render: (r) => <span className="text-surface-300">{sourceLabel(r.source_type)}</span> },
    { key: "reason", header: "السبب", render: (r) => r.reason || "—" },
    { key: "deduction_per_payroll_milli", header: "خصم مقترح/راتب", align: "left", render: (r) => r.deduction_per_payroll_milli > 0 ? formatOMR(r.deduction_per_payroll_milli) : "—" },
    { key: "remaining_milli", header: "المتبقي", align: "left", render: (r) => <span className="font-bold text-gold-400">{formatOMR(r.remaining_milli)}</span> },
    { key: "journal_id", header: "الترحيل", render: (r) => r.journal_id ? (
      <span className="inline-flex px-2 py-1 rounded-lg text-xs bg-emerald-500/15 text-emerald-400">مرحّل · JE #{r.journal_id}</span>
    ) : (
      <span className="inline-flex px-2 py-1 rounded-lg text-xs bg-red-500/15 text-red-300">غير مرحّل</span>
    ) },
    { key: "status", header: "الحالة", render: (r) => (
      <Badge variant={(r.status || "open").toLowerCase() === "open" ? "warning" : "success"}>
        {(r.status || "open").toLowerCase() === "open" ? "مفتوح" : "مغلق"}
      </Badge>
    ) },
  ], []);

  return (
    <div className="space-y-6" dir="rtl">
      <div className="page-header">
        <div>
          <h1 className="page-title">سلف الموظفين</h1>
          <p className="page-subtitle">سلف الرواتب الشخصية — منفصلة عن عهد التشغيل والمشتريات</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => setShowForm(true)}>سلفة جديدة</Button>
      </div>

      <div className="grid grid-cols-2 xl:grid-cols-5 gap-4">
        <Card className="text-center"><p className="text-2xl font-bold gradient-text">{advances.length}</p><p className="text-xs text-surface-400">إجمالي السلف</p></Card>
        <Card className="text-center"><p className="text-2xl font-bold text-gold-400">{formatOMR(totalOutstanding)}</p><p className="text-xs text-surface-400">الرصيد على الموظفين</p></Card>
        <Card className="text-center"><p className="text-2xl font-bold text-yellow-400">{openAdvances.length}</p><p className="text-xs text-surface-400">سلف مفتوحة</p></Card>
        <Card className="text-center"><p className="text-2xl font-bold text-emerald-400">{closedCount}</p><p className="text-xs text-surface-400">سلف مغلقة</p></Card>
        <Card className="text-center"><p className={`text-2xl font-bold ${unpostedCount ? "text-red-400" : "text-emerald-400"}`}>{unpostedCount}</p><p className="text-xs text-surface-400">غير مرحلة للمراجعة</p></Card>
      </div>

      {unpostedCount > 0 && (
        <Card className="border border-amber-500/30">
          <div className="flex items-start gap-3">
            <AlertTriangle className="w-5 h-5 text-amber-400 mt-0.5" />
            <div>
              <p className="font-bold text-amber-300">توجد سلف تاريخية بلا قيد مرتبط</p>
              <p className="text-sm text-surface-400 mt-1">لا تُرحّل تلقائيًا حتى لا نكرر أثرًا ماليًا قد يكون مسجلًا قديمًا يدويًا. يجب مطابقتها مع الأستاذ العام قبل إنشاء قيود تسوية.</p>
            </div>
          </div>
        </Card>
      )}

      {showForm && (
        <Card>
          <div className="flex items-center justify-between mb-5">
            <div>
              <h2 className="text-lg font-bold text-white">سلفة موظف جديدة</h2>
              <p className="text-xs text-surface-500 mt-1">القيد الناتج: مدين 1320 سلف الموظفين / دائن مصدر الصرف الفعلي</p>
            </div>
            <button onClick={() => { setShowForm(false); resetForm(); }} className="text-surface-400 hover:text-white text-xl">&times;</button>
          </div>

          <div className="space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div className="input-group">
                <label className="input-label">الموظف</label>
                <select value={form.employee_id} onChange={(e) => setForm({ ...form, employee_id: e.target.value })} className="input-field" aria-label="الموظف">
                  <option value="">— اختر موظف —</option>
                  {employees.map((employee: Employee) => <option key={employee.id} value={employee.id}>{employee.name}</option>)}
                </select>
              </div>
              <div className="input-group">
                <label className="input-label">قيمة السلفة (ر.ع)</label>
                <input type="number" min="0.001" step="0.001" value={form.amount_omr} onChange={(e) => setForm({ ...form, amount_omr: e.target.value })} className="input-field" dir="ltr" placeholder="0.000" aria-label="قيمة السلفة بالريال العماني" />
              </div>
              <div className="input-group">
                <label className="input-label">التاريخ</label>
                <input type="date" value={form.date} onChange={(e) => setForm({ ...form, date: e.target.value })} className="input-field" aria-label="التاريخ" />
              </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="input-group">
                <label className="input-label">مصدر صرف السلفة</label>
                <select value={form.source_type} onChange={(e) => setForm({ ...form, source_type: e.target.value, cashbank_id: "", custody_id: "" })} className="input-field" aria-label="مصدر صرف السلفة">
                  {SOURCE_OPTIONS.map((item) => <option key={item.value} value={item.value}>{item.label}</option>)}
                </select>
              </div>
              <div className="input-group">
                <label className="input-label">الخصم المقترح من كل راتب (ر.ع)</label>
                <input type="number" min="0" step="0.001" value={form.deduction_per_payroll_omr} onChange={(e) => setForm({ ...form, deduction_per_payroll_omr: e.target.value })} className="input-field" dir="ltr" placeholder="0.000" aria-label="الخصم المقترح من كل راتب بالريال العماني" />
                <p className="text-[11px] text-surface-500 mt-1">قيمة تخطيطية فقط؛ لا تُخصم من المسير تلقائيًا قبل اعتماد تسوية الرواتب.</p>
              </div>
            </div>

            {form.source_type === "company" && (
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="input-group">
                  <label className="input-label">طريقة الصرف</label>
                  <select value={form.method} onChange={(e) => setForm({ ...form, method: e.target.value })} className="input-field" aria-label="طريقة الصرف">
                    {METHOD_OPTIONS.map((item) => <option key={item.value} value={item.value}>{item.label}</option>)}
                  </select>
                </div>
                <div className="input-group">
                  <label className="input-label">حساب النقدية / البنك</label>
                  <select value={form.cashbank_id} onChange={(e) => setForm({ ...form, cashbank_id: e.target.value })} className="input-field" aria-label="حساب النقدية أو البنك">
                    <option value="">الحساب العام حسب طريقة الصرف</option>
                    {cashbankAccounts.map((account) => <option key={account.id} value={account.id}>{account.name}</option>)}
                  </select>
                </div>
              </div>
            )}

            {form.source_type === "custody" && (
              <div className="input-group">
                <label className="input-label">العهدة الدافعة</label>
                <select value={form.custody_id} onChange={(e) => setForm({ ...form, custody_id: e.target.value })} className="input-field" aria-label="العهدة الدافعة">
                  <option value="">— اختر العهدة —</option>
                  {custodyAccounts.map((account) => <option key={account.id} value={account.id}>{account.name}</option>)}
                </select>
              </div>
            )}

            <div className="input-group">
              <label className="input-label">السبب / البيان</label>
              <input type="text" value={form.reason} onChange={(e) => setForm({ ...form, reason: e.target.value })} className="input-field" placeholder="مثال: سلفة من راتب سبتمبر" aria-label="سبب السلفة" />
            </div>
          </div>

          <div className="flex justify-end gap-3 mt-5">
            <Button variant="ghost" onClick={() => { setShowForm(false); resetForm(); }}>إلغاء</Button>
            <Button icon={<HandCoins className="w-4 h-4" />} onClick={() => setShowConfirm(true)} loading={saving}>إنشاء وترحيل السلفة</Button>
          </div>
        </Card>
      )}

      <DataTable columns={columns} data={advances} loading={loading} emptyMessage="لا توجد سلف موظفين" />

      <ConfirmDialog
        open={showConfirm}
        title="إنشاء وترحيل سلفة موظف"
        message="سيتم تسجيل السلفة كذمة على الموظف وترحيل قيد محاسبي مقابل مصدر الصرف المحدد. هل تريد المتابعة؟"
        variant="warning"
        onConfirm={() => { setShowConfirm(false); handleCreate(); }}
        onCancel={() => setShowConfirm(false)}
      />
    </div>
  );
}
