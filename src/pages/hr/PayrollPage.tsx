import { useState, useEffect, useMemo, useCallback } from "react";
import DataTable, { Column } from "@/components/ui/DataTable";
import Badge from "@/components/ui/Badge";
import Button from "@/components/ui/Button";
import Card from "@/components/ui/Card";
import { formatOMR, formatDate, omrToMilli } from "@/lib/utils";
import { invoke } from "@/lib/tauri";
import {
  Plus, Play, Eye, RefreshCw, CheckCircle2, CreditCard, X,
  SlidersHorizontal, Check, XCircle, AlertTriangle
} from "lucide-react";
import ConfirmDialog from "@/components/ui/ConfirmDialog";
import { useUIStore } from "@/stores/uiStore";
import type { PayrollRun } from "@/types";

interface PayrollLine {
  id: number;
  run_id: number;
  employee_id: number;
  employee_name: string;
  basic_milli: number;
  allowance_milli: number;
  overtime_milli: number;
  bonus_milli: number;
  deduction_milli: number;
  advance_deduction_milli: number;
  insurance_deduction_milli: number;
  tax_deduction_milli: number;
  net_milli: number;
  paid_milli: number;
  notes?: string | null;
}

interface PayrollPayment {
  id: number;
  employee_name?: string | null;
  payment_date: string;
  source_type: string;
  amount_milli: number;
  method?: string | null;
  reference?: string | null;
  wps_status?: string | null;
  wps_reference?: string | null;
}

interface PayrollAdjustment {
  id: number;
  employee_id: number;
  employee_name?: string | null;
  adjustment_type: string;
  amount_milli: number;
  effective_date: string;
  reason?: string | null;
  reference?: string | null;
  notes?: string | null;
  status: string;
  applied_run_id?: number | null;
}

interface AccountChoice { id: number; name: string; code?: string | null; }
interface EmployeeChoice { id: number; name: string; code?: string | null; }

const statusLower = (value?: string | null) => (value || "").toLowerCase();
const today = () => new Date().toISOString().split("T")[0];

const adjustmentTypeLabels: Record<string, string> = {
  absence: "خصم غياب",
  disciplinary_penalty: "جزاء تأديبي",
  other_deduction: "خصم آخر",
  bonus: "مكافأة",
  allowance: "بدل إضافي",
};

export default function PayrollPage() {
  const addNotification = useUIStore((s) => s.addNotification);
  const [runs, setRuns] = useState<PayrollRun[]>([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [saving, setSaving] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);
  const [confirmAction, setConfirmAction] = useState<() => Promise<void>>(async () => {});
  const [confirmTitle, setConfirmTitle] = useState("");
  const [confirmMessage, setConfirmMessage] = useState("");
  const [form, setForm] = useState({ period_start: "", period_end: "" });

  const [selectedRun, setSelectedRun] = useState<PayrollRun | null>(null);
  const [lines, setLines] = useState<PayrollLine[]>([]);
  const [payments, setPayments] = useState<PayrollPayment[]>([]);
  const [detailLoading, setDetailLoading] = useState(false);
  const [cashAccounts, setCashAccounts] = useState<AccountChoice[]>([]);
  const [custodyAccounts, setCustodyAccounts] = useState<AccountChoice[]>([]);

  const [adjustments, setAdjustments] = useState<PayrollAdjustment[]>([]);
  const [employees, setEmployees] = useState<EmployeeChoice[]>([]);
  const [showAdjustmentForm, setShowAdjustmentForm] = useState(false);
  const [adjustmentForm, setAdjustmentForm] = useState({
    employee_id: "",
    adjustment_type: "absence",
    amount: "",
    effective_date: today(),
    reason: "",
    reference: "",
    notes: "",
  });

  const [payLine, setPayLine] = useState<PayrollLine | null>(null);
  const [payForm, setPayForm] = useState({
    payment_date: today(),
    amount: "",
    source_type: "company",
    method: "bank_transfer",
    cashbank_id: "",
    custody_id: "",
    reference: "",
    notes: "",
    wps_reference: "",
  });

  const loadRuns = useCallback(async () => {
    setLoading(true);
    try {
      const d = await invoke<PayrollRun[]>("list_payroll_runs");
      setRuns(d);
      if (selectedRun) {
        const refreshed = d.find((r) => r.id === selectedRun.id);
        if (refreshed) setSelectedRun(refreshed);
      }
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) });
    } finally {
      setLoading(false);
    }
  }, [addNotification, selectedRun?.id]);

  const loadReferenceAccounts = useCallback(async () => {
    const [cash, custody, employeeData] = await Promise.all([
      invoke<AccountChoice[]>("list_cashbank_accounts").catch(() => []),
      invoke<AccountChoice[]>("get_custody_accounts_for_select").catch(() => []),
      invoke<EmployeeChoice[]>("list_employees_for_select").catch(() => []),
    ]);
    setCashAccounts(cash);
    setCustodyAccounts(custody);
    setEmployees(employeeData);
  }, []);

  const loadAdjustments = useCallback(async () => {
    try {
      const data = await invoke<PayrollAdjustment[]>("list_payroll_adjustments", { employeeId: null });
      setAdjustments(data);
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "تعذر تحميل تسويات الرواتب", message: String(err) });
    }
  }, [addNotification]);

  const loadDetails = useCallback(async (run: PayrollRun) => {
    setSelectedRun(run);
    setDetailLoading(true);
    try {
      const [l, p] = await Promise.all([
        invoke<PayrollLine[]>("list_payroll_run_lines", { runId: run.id }),
        invoke<PayrollPayment[]>("list_payroll_payments", { runId: run.id }),
      ]);
      setLines(l);
      setPayments(p);
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) });
    } finally {
      setDetailLoading(false);
    }
  }, [addNotification]);

  useEffect(() => {
    loadRuns();
    loadReferenceAccounts();
    loadAdjustments();
  }, []);

  const handleCreate = async () => {
    if (!form.period_start || !form.period_end) return;
    setSaving(true);
    try {
      const runId = await invoke<number>("create_payroll_run", { input: form });
      setShowForm(false);
      setForm({ period_start: "", period_end: "" });
      await loadRuns();
      addNotification({ id: crypto.randomUUID(), type: "success", title: "تم", message: `تم إنشاء تشغيلة الرواتب #${runId}` });
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "تعذر إنشاء المسير", message: String(err) });
    } finally {
      setSaving(false);
    }
  };

  const prepareRun = async (run: PayrollRun) => {
    setSaving(true);
    try {
      await invoke("prepare_payroll_run", { runId: run.id });
      await loadRuns();
      const latest = { ...run, status: "Prepared" };
      await loadDetails(latest);
      addNotification({
        id: crypto.randomUUID(),
        type: "success",
        title: statusLower(run.status) === "prepared" ? "تمت إعادة التحضير" : "تم التحضير",
        message: "تم احتساب الراتب والأوفر تايم والتسويات المعتمدة وخصم السلف المجدول.",
      });
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "فشل التحضير", message: String(err) });
    } finally {
      setSaving(false);
    }
  };

  const approveRun = async (run: PayrollRun) => {
    setSaving(true);
    try {
      await invoke("approve_payroll_run", { runId: run.id });
      await loadRuns();
      await loadAdjustments();
      const latest = { ...run, status: "Approved" };
      await loadDetails(latest);
      addNotification({ id: crypto.randomUUID(), type: "success", title: "تم الاعتماد", message: "تم إثبات الرواتب والتسويات وسداد السلف في الأستاذ العام." });
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "فشل الاعتماد", message: String(err) });
    } finally {
      setSaving(false);
    }
  };

  const createAdjustment = async () => {
    if (!adjustmentForm.employee_id || Number(adjustmentForm.amount) <= 0 || !adjustmentForm.effective_date) return;
    setSaving(true);
    try {
      await invoke("create_payroll_adjustment", {
        input: {
          employee_id: Number(adjustmentForm.employee_id),
          adjustment_type: adjustmentForm.adjustment_type,
          amount_milli: omrToMilli(Number(adjustmentForm.amount)),
          effective_date: adjustmentForm.effective_date,
          reason: adjustmentForm.reason || null,
          reference: adjustmentForm.reference || null,
          notes: adjustmentForm.notes || null,
        },
      });
      setShowAdjustmentForm(false);
      setAdjustmentForm({ employee_id: "", adjustment_type: "absence", amount: "", effective_date: today(), reason: "", reference: "", notes: "" });
      await loadAdjustments();
      addNotification({ id: crypto.randomUUID(), type: "success", title: "تم تسجيل التسوية", message: "التسوية ما زالت معلقة وتحتاج اعتمادًا قبل أن تدخل في مسير الرواتب." });
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "تعذر تسجيل التسوية", message: String(err) });
    } finally {
      setSaving(false);
    }
  };

  const decideAdjustment = async (adjustment: PayrollAdjustment, approve: boolean) => {
    setSaving(true);
    try {
      await invoke(approve ? "approve_payroll_adjustment" : "reject_payroll_adjustment", { id: adjustment.id });
      await loadAdjustments();
      addNotification({
        id: crypto.randomUUID(),
        type: approve ? "success" : "info",
        title: approve ? "تم اعتماد التسوية" : "تم رفض التسوية",
        message: approve
          ? "إذا كان مسير هذه الفترة محضرًا بالفعل، أعد تحضيره قبل الاعتماد النهائي."
          : "لن تدخل هذه التسوية في مسير الرواتب.",
      });
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "تعذر تحديث التسوية", message: String(err) });
    } finally {
      setSaving(false);
    }
  };

  const openPay = (line: PayrollLine) => {
    const remaining = Math.max(0, line.net_milli - line.paid_milli);
    setPayLine(line);
    setPayForm({
      payment_date: today(),
      amount: (remaining / 1000).toFixed(3),
      source_type: "company",
      method: "bank_transfer",
      cashbank_id: "",
      custody_id: "",
      reference: "",
      notes: "",
      wps_reference: "",
    });
  };

  const recordPayment = async () => {
    if (!selectedRun || !payLine || Number(payForm.amount) <= 0) return;
    setSaving(true);
    try {
      await invoke("record_payroll_payment", {
        input: {
          run_id: selectedRun.id,
          employee_id: payLine.employee_id,
          payment_date: payForm.payment_date,
          amount_milli: omrToMilli(Number(payForm.amount)),
          source_type: payForm.source_type,
          cashbank_id: payForm.source_type === "company" && payForm.cashbank_id ? Number(payForm.cashbank_id) : null,
          custody_id: payForm.source_type === "custody" && payForm.custody_id ? Number(payForm.custody_id) : null,
          method: payForm.method,
          reference: payForm.reference || null,
          notes: payForm.notes || null,
          wps_reference: payForm.wps_reference || null,
        },
      });
      setPayLine(null);
      await loadRuns();
      await loadDetails(selectedRun);
      addNotification({ id: crypto.randomUUID(), type: "success", title: "تم الدفع", message: "تم تسجيل سداد الراتب وربطه بمصدر المال." });
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "فشل دفع الراتب", message: String(err) });
    } finally {
      setSaving(false);
    }
  };

  const totalPaidRuns = runs.filter((r) => statusLower(r.status) === "paid").length;
  const pendingRuns = runs.filter((r) => ["draft", "prepared", "approved", "partially paid"].includes(statusLower(r.status))).length;
  const pendingAdjustments = adjustments.filter((a) => statusLower(a.status) === "pending").length;
  const approvedAdjustments = adjustments.filter((a) => statusLower(a.status) === "approved").length;

  const statusBadge = (status?: string | null) => {
    const s = statusLower(status);
    const map: Record<string, { label: string; variant: "default" | "info" | "warning" | "success" | "danger" }> = {
      draft: { label: "مسودة", variant: "default" },
      prepared: { label: "محضّر", variant: "info" },
      approved: { label: "معتمد / مستحق", variant: "warning" },
      "partially paid": { label: "مدفوع جزئيًا", variant: "warning" },
      paid: { label: "مدفوع", variant: "success" },
    };
    const item = map[s] || { label: status || "—", variant: "default" as const };
    return <Badge variant={item.variant}>{item.label}</Badge>;
  };

  const adjustmentStatusBadge = (status: string) => {
    const s = statusLower(status);
    if (s === "applied") return <Badge variant="success">مطبق</Badge>;
    if (s === "approved") return <Badge variant="info">معتمد</Badge>;
    if (s === "rejected") return <Badge variant="danger">مرفوض</Badge>;
    return <Badge variant="warning">بانتظار الاعتماد</Badge>;
  };

  const columns: Column<PayrollRun>[] = useMemo(() => [
    { key: "run_no", header: "رقم التشغيلة", sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.run_no || "—"}</span> },
    { key: "period", header: "الفترة", render: (r) => `${formatDate(r.period_start)} — ${formatDate(r.period_end)}` },
    { key: "total_gross_milli", header: "الإجمالي", align: "left", render: (r) => formatOMR(r.total_gross_milli) },
    { key: "total_deductions_milli", header: "الخصومات المعتمدة", align: "left", render: (r) => formatOMR(r.total_deductions_milli) },
    { key: "total_net_milli", header: "الصافي", align: "left", render: (r) => <span className="font-bold text-gold-400">{formatOMR(r.total_net_milli)}</span> },
    { key: "status", header: "الحالة", render: (r) => statusBadge(r.status) },
    { key: "actions", header: "", render: (r) => (
      <div className="flex items-center gap-1">
        <button onClick={() => loadDetails(r)} className="p-1.5 text-surface-400 hover:text-brand-400 rounded-lg" title="التفاصيل"><Eye className="w-4 h-4" /></button>
        {["draft", "prepared"].includes(statusLower(r.status)) && (
          <button onClick={() => prepareRun(r)} className="p-1.5 text-surface-400 hover:text-gold-400 rounded-lg" title={statusLower(r.status) === "prepared" ? "إعادة التحضير" : "تحضير"}><RefreshCw className="w-4 h-4" /></button>
        )}
        {statusLower(r.status) === "prepared" && (
          <button onClick={() => { setConfirmTitle("اعتماد مسير الرواتب"); setConfirmMessage("سيتم تطبيق التسويات المعتمدة وتخفيض أرصدة السلف وإنشاء قيد الاستحقاق. تأكد أن المسير أعيد تحضيره بعد آخر تعديل. هل تريد الاعتماد؟"); setConfirmAction(() => async () => approveRun(r)); setShowConfirm(true); }} className="p-1.5 text-emerald-400 rounded-lg" title="اعتماد">
            <CheckCircle2 className="w-4 h-4" />
          </button>
        )}
      </div>
    )},
  ], [loadDetails]);

  return (
    <div className="space-y-6" dir="rtl">
      <div className="page-header">
        <div>
          <h1 className="page-title">الرواتب والأجور</h1>
          <p className="page-subtitle">رواتب، أوفر تايم، سلف، غياب، جزاءات، دفع ومطابقة WPS مع أثر تدقيق كامل</p>
        </div>
        <div className="flex gap-2 flex-wrap">
          <Button variant="outline" icon={<RefreshCw className="w-4 h-4" />} onClick={() => { loadRuns(); loadAdjustments(); }}>تحديث</Button>
          <Button variant="outline" icon={<SlidersHorizontal className="w-4 h-4" />} onClick={() => setShowAdjustmentForm(true)}>تسوية راتب</Button>
          <Button icon={<Plus className="w-4 h-4" />} onClick={() => setShowForm(true)}>تشغيلة جديدة</Button>
        </div>
      </div>

      <div className="grid grid-cols-2 lg:grid-cols-5 gap-4">
        <Card className="text-center"><p className="text-2xl font-bold gradient-text">{runs.length}</p><p className="text-xs text-surface-400">التشغيلات</p></Card>
        <Card className="text-center"><p className="text-2xl font-bold text-emerald-400">{totalPaidRuns}</p><p className="text-xs text-surface-400">مدفوعة بالكامل</p></Card>
        <Card className="text-center"><p className="text-2xl font-bold text-gold-400">{pendingRuns}</p><p className="text-xs text-surface-400">مسيرات تحت الإجراء</p></Card>
        <Card className="text-center"><p className="text-2xl font-bold text-amber-400">{pendingAdjustments}</p><p className="text-xs text-surface-400">تسويات بانتظار الاعتماد</p></Card>
        <Card className="text-center"><p className="text-2xl font-bold text-brand-400">{formatOMR(runs.reduce((s,r) => s + (r.total_net_milli || 0),0))}</p><p className="text-xs text-surface-400">إجمالي صافي المسيرات</p></Card>
      </div>

      {showAdjustmentForm && (
        <Card className="border border-brand-500/20">
          <div className="flex items-center justify-between mb-4">
            <div><h2 className="text-lg font-bold">تسوية راتب جديدة</h2><p className="text-xs text-surface-400 mt-1">لا تدخل في الراتب إلا بعد الاعتماد ثم تحضير/إعادة تحضير المسير.</p></div>
            <button onClick={() => setShowAdjustmentForm(false)}><X className="w-5 h-5" /></button>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="input-group"><label className="input-label">العامل *</label><select className="input-field" value={adjustmentForm.employee_id} onChange={(e) => setAdjustmentForm({...adjustmentForm,employee_id:e.target.value})}><option value="">— اختر العامل —</option>{employees.map((e)=><option key={e.id} value={e.id}>{e.name}</option>)}</select></div>
            <div className="input-group"><label className="input-label">النوع *</label><select className="input-field" value={adjustmentForm.adjustment_type} onChange={(e) => setAdjustmentForm({...adjustmentForm,adjustment_type:e.target.value})}><option value="absence">خصم غياب</option><option value="disciplinary_penalty">جزاء تأديبي</option><option value="other_deduction">خصم آخر</option><option value="bonus">مكافأة</option><option value="allowance">بدل إضافي</option></select></div>
            <div className="input-group"><label className="input-label">القيمة (ر.ع) *</label><input type="number" min="0.001" step="0.001" className="input-field" value={adjustmentForm.amount} onChange={(e) => setAdjustmentForm({...adjustmentForm,amount:e.target.value})} /></div>
            <div className="input-group"><label className="input-label">تاريخ الاستحقاق *</label><input type="date" className="input-field" value={adjustmentForm.effective_date} onChange={(e) => setAdjustmentForm({...adjustmentForm,effective_date:e.target.value})} /></div>
            <div className="input-group"><label className="input-label">السبب / البيان</label><input className="input-field" value={adjustmentForm.reason} onChange={(e) => setAdjustmentForm({...adjustmentForm,reason:e.target.value})} placeholder="غياب 24 أغسطس / جزاء رقم..." /></div>
            <div className="input-group"><label className="input-label">مرجع المستند</label><input className="input-field" value={adjustmentForm.reference} onChange={(e) => setAdjustmentForm({...adjustmentForm,reference:e.target.value})} placeholder="إنذار / قرار / طلب معتمد" /></div>
            <div className="input-group md:col-span-3"><label className="input-label">ملاحظات</label><input className="input-field" value={adjustmentForm.notes} onChange={(e) => setAdjustmentForm({...adjustmentForm,notes:e.target.value})} /></div>
          </div>
          <div className="flex justify-end gap-3 mt-4"><Button variant="ghost" onClick={() => setShowAdjustmentForm(false)}>إلغاء</Button><Button icon={<Plus className="w-4 h-4" />} onClick={createAdjustment} loading={saving}>تسجيل كمعلق</Button></div>
        </Card>
      )}

      <Card>
        <div className="flex items-center justify-between gap-4 mb-4">
          <div><h2 className="font-bold">تسويات الرواتب</h2><p className="text-xs text-surface-500 mt-1">الغياب والجزاءات والإضافات موثقة بشكل مستقل عن مسير الراتب.</p></div>
          <div className="flex gap-2 text-xs"><span className="px-2 py-1 rounded-lg bg-amber-500/10 text-amber-300">معلق {pendingAdjustments}</span><span className="px-2 py-1 rounded-lg bg-blue-500/10 text-blue-300">معتمد {approvedAdjustments}</span></div>
        </div>
        {adjustments.length === 0 ? <div className="py-6 text-center text-surface-500">لا توجد تسويات رواتب مسجلة.</div> : (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead><tr className="text-surface-400 border-b border-surface-700"><th className="p-2 text-right">العامل</th><th className="p-2">التاريخ</th><th className="p-2">النوع</th><th className="p-2">القيمة</th><th className="p-2 text-right">السبب / المرجع</th><th className="p-2">الحالة</th><th className="p-2"></th></tr></thead>
              <tbody>{adjustments.slice(0, 20).map((a) => <tr key={a.id} className="border-b border-surface-800"><td className="p-2 font-medium">{a.employee_name || `#${a.employee_id}`}</td><td className="p-2 text-center">{formatDate(a.effective_date)}</td><td className="p-2 text-center">{adjustmentTypeLabels[a.adjustment_type] || a.adjustment_type}</td><td className={`p-2 text-center font-bold ${["bonus","allowance"].includes(a.adjustment_type) ? "text-emerald-400" : "text-amber-400"}`}>{formatOMR(a.amount_milli)}</td><td className="p-2"><div>{a.reason || "—"}</div>{a.reference && <div className="text-xs text-surface-500 mt-1">{a.reference}</div>}</td><td className="p-2 text-center">{adjustmentStatusBadge(a.status)}</td><td className="p-2">{statusLower(a.status) === "pending" && <div className="flex gap-1 justify-end"><button onClick={() => decideAdjustment(a,true)} className="p-1.5 rounded-lg text-emerald-400 hover:bg-emerald-500/10" title="اعتماد"><Check className="w-4 h-4" /></button><button onClick={() => decideAdjustment(a,false)} className="p-1.5 rounded-lg text-red-400 hover:bg-red-500/10" title="رفض"><XCircle className="w-4 h-4" /></button></div>}</td></tr>)}</tbody>
            </table>
          </div>
        )}
      </Card>

      {approvedAdjustments > 0 && runs.some((r) => statusLower(r.status) === "prepared") && (
        <div className="flex gap-3 items-start p-4 rounded-xl border border-amber-500/30 bg-amber-500/5 text-sm text-amber-200">
          <AlertTriangle className="w-5 h-5 mt-0.5 shrink-0" />
          <div><p className="font-bold">يوجد مسير محضّر وتسويات معتمدة غير مطبقة.</p><p className="text-xs mt-1 text-amber-300/80">أعد تحضير المسير للفترة المناسبة قبل الاعتماد حتى تدخل التسويات والسلف بأرصدة محدثة.</p></div>
        </div>
      )}

      {showForm && (
        <Card>
          <div className="flex items-center justify-between mb-4"><h2 className="text-lg font-bold">تشغيلة رواتب جديدة</h2><button onClick={() => setShowForm(false)}><X className="w-5 h-5" /></button></div>
          <div className="grid grid-cols-2 gap-4">
            <div className="input-group"><label className="input-label">بداية الفترة</label><input type="date" value={form.period_start} onChange={(e) => setForm({ ...form, period_start: e.target.value })} className="input-field" /></div>
            <div className="input-group"><label className="input-label">نهاية الفترة</label><input type="date" value={form.period_end} onChange={(e) => setForm({ ...form, period_end: e.target.value })} className="input-field" /></div>
          </div>
          <p className="text-xs text-surface-500 mt-3">المسير يسحب فقط التسويات المعتمدة والسلف التي لها قيمة خصم دوري محددة.</p>
          <div className="flex justify-end gap-3 mt-4"><Button variant="ghost" onClick={() => setShowForm(false)}>إلغاء</Button><Button icon={<Play className="w-4 h-4" />} onClick={handleCreate} loading={saving}>إنشاء</Button></div>
        </Card>
      )}

      <DataTable columns={columns} data={runs} loading={loading} emptyMessage="لا توجد تشغيلات رواتب" />

      {selectedRun && (
        <Card>
          <div className="flex items-center justify-between mb-5">
            <div><h2 className="text-lg font-bold">تفاصيل {selectedRun.run_no}</h2><div className="mt-1">{statusBadge(selectedRun.status)}</div></div>
            <div className="flex gap-2 flex-wrap">
              {["draft","prepared"].includes(statusLower(selectedRun.status)) && <Button variant={statusLower(selectedRun.status) === "prepared" ? "outline" : "primary"} icon={<RefreshCw className="w-4 h-4" />} onClick={() => prepareRun(selectedRun)} loading={saving}>{statusLower(selectedRun.status) === "prepared" ? "إعادة التحضير" : "تحضير المسير"}</Button>}
              {statusLower(selectedRun.status) === "prepared" && <Button variant="gold" onClick={() => { setConfirmTitle("اعتماد مسير الرواتب"); setConfirmMessage("سيتم تطبيق التسويات وسداد السلف وإنشاء قيد الاستحقاق. هل تريد المتابعة؟"); setConfirmAction(() => async () => approveRun(selectedRun)); setShowConfirm(true); }}>اعتماد المسير</Button>}
              <Button variant="ghost" onClick={() => { setSelectedRun(null); setLines([]); setPayments([]); }}>إغلاق</Button>
            </div>
          </div>

          {detailLoading ? <p className="text-surface-400">جاري تحميل التفاصيل...</p> : lines.length === 0 ? <div className="py-8 text-center text-surface-500">المسير لم يُحضّر بعد.</div> : (
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead><tr className="text-surface-400 border-b border-surface-700"><th className="text-right p-2">العامل</th><th className="p-2">أساسي</th><th className="p-2">بدلات</th><th className="p-2">أوفر تايم</th><th className="p-2">إضافات</th><th className="p-2">خصم إداري</th><th className="p-2">سلفة</th><th className="p-2">صافي</th><th className="p-2">متبقي</th><th></th></tr></thead>
                <tbody>{lines.map((line) => {
                  const remaining = Math.max(0, line.net_milli - line.paid_milli);
                  const canPay = ["approved","partially paid"].includes(statusLower(selectedRun.status)) && remaining > 0;
                  return <tr key={line.id} className="border-b border-surface-800"><td className="p-2 font-medium"><div>{line.employee_name}</div>{line.notes && <div className="text-[10px] text-surface-500 mt-1">{line.notes}</div>}</td><td className="p-2 text-center">{formatOMR(line.basic_milli)}</td><td className="p-2 text-center">{formatOMR(line.allowance_milli)}</td><td className="p-2 text-center text-gold-400">{formatOMR(line.overtime_milli)}</td><td className="p-2 text-center text-emerald-400">{formatOMR(line.bonus_milli)}</td><td className="p-2 text-center text-amber-400">{formatOMR(line.deduction_milli)}</td><td className="p-2 text-center text-amber-400">{formatOMR(line.advance_deduction_milli)}</td><td className="p-2 text-center font-bold">{formatOMR(line.net_milli)}</td><td className="p-2 text-center">{formatOMR(remaining)}</td><td className="p-2">{canPay && <Button size="sm" icon={<CreditCard className="w-3 h-3" />} onClick={() => openPay(line)}>دفع</Button>}</td></tr>;
                })}</tbody>
              </table>
            </div>
          )}

          {payments.length > 0 && <div className="mt-6"><h3 className="font-bold mb-3">سجل دفعات الرواتب</h3><div className="space-y-2">{payments.map((p) => <div key={p.id} className="grid grid-cols-2 md:grid-cols-6 gap-2 p-3 rounded-xl bg-surface-800/40 text-sm"><span>{p.employee_name || "—"}</span><span>{formatDate(p.payment_date)}</span><span>{formatOMR(p.amount_milli)}</span><span>{p.source_type}</span><span>{p.method || "—"}</span><span className={p.wps_status === "recorded_for_reconciliation" ? "text-blue-400" : "text-amber-400"}>{p.wps_status || "—"}</span></div>)}</div></div>}
        </Card>
      )}

      {payLine && selectedRun && (
        <Card className="border border-brand-500/30">
          <div className="flex items-center justify-between mb-4"><div><h2 className="font-bold">دفع راتب — {payLine.employee_name}</h2><p className="text-xs text-surface-400">المتبقي {formatOMR(Math.max(0, payLine.net_milli - payLine.paid_milli))}</p></div><button onClick={() => setPayLine(null)}><X className="w-5 h-5" /></button></div>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="input-group"><label className="input-label">تاريخ الدفع</label><input type="date" className="input-field" value={payForm.payment_date} onChange={(e) => setPayForm({...payForm,payment_date:e.target.value})} /></div>
            <div className="input-group"><label className="input-label">المبلغ (ر.ع)</label><input type="number" min="0.001" step="0.001" className="input-field" value={payForm.amount} onChange={(e) => setPayForm({...payForm,amount:e.target.value})} /></div>
            <div className="input-group"><label className="input-label">مصدر الدفع</label><select className="input-field" value={payForm.source_type} onChange={(e) => setPayForm({...payForm,source_type:e.target.value,cashbank_id:"",custody_id:""})}><option value="company">الشركة / المصنع</option><option value="custody">العهدة</option><option value="owner_saif">سيف محمد</option><option value="owner_abu_saif">أبو سيف</option></select></div>
            <div className="input-group"><label className="input-label">طريقة الدفع</label><select className="input-field" value={payForm.method} onChange={(e) => setPayForm({...payForm,method:e.target.value})}><option value="bank_transfer">تحويل بنكي</option><option value="cash">نقدي</option><option value="cheque">شيك</option></select></div>
            {payForm.source_type === "company" && <div className="input-group"><label className="input-label">حساب الشركة</label><select className="input-field" value={payForm.cashbank_id} onChange={(e) => setPayForm({...payForm,cashbank_id:e.target.value})}><option value="">تلقائي</option>{cashAccounts.map(a=><option key={a.id} value={a.id}>{a.name}</option>)}</select></div>}
            {payForm.source_type === "custody" && <div className="input-group"><label className="input-label">العهدة *</label><select className="input-field" value={payForm.custody_id} onChange={(e) => setPayForm({...payForm,custody_id:e.target.value})}><option value="">— اختر —</option>{custodyAccounts.map(a=><option key={a.id} value={a.id}>{a.name}</option>)}</select></div>}
            <div className="input-group"><label className="input-label">مرجع الدفع</label><input className="input-field" value={payForm.reference} onChange={(e) => setPayForm({...payForm,reference:e.target.value})} /></div>
            <div className="input-group"><label className="input-label">مرجع WPS / SIF</label><input className="input-field" value={payForm.wps_reference} onChange={(e) => setPayForm({...payForm,wps_reference:e.target.value})} placeholder="إن وجد" /></div>
            <div className="input-group"><label className="input-label">ملاحظات</label><input className="input-field" value={payForm.notes} onChange={(e) => setPayForm({...payForm,notes:e.target.value})} /></div>
          </div>
          <p className="text-xs text-amber-400 mt-3">الدفع النقدي أو عبر العهدة/المالك يُسجل ماليًا، لكنه لا يثبت وحده مطابقة WPS؛ تبقى حالة WPS للمراجعة أو الاستثناء النظامي.</p>
          <div className="flex justify-end mt-4"><Button icon={<CreditCard className="w-4 h-4" />} onClick={recordPayment} loading={saving}>تسجيل دفع الراتب</Button></div>
        </Card>
      )}

      <ConfirmDialog open={showConfirm} title={confirmTitle} message={confirmMessage} variant="warning" onConfirm={() => { const action = confirmAction; setShowConfirm(false); action(); }} onCancel={() => setShowConfirm(false)} />
    </div>
  );
}
