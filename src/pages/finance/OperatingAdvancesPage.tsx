import { useState, useEffect, useMemo, useCallback } from "react";
import DataTable, { Column } from "@/components/ui/DataTable";
import Badge from "@/components/ui/Badge";
import Button from "@/components/ui/Button";
import Card from "@/components/ui/Card";
import Modal from "@/components/ui/Modal";
import { formatOMR, formatDate, formatDateTime } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { useAuthStore } from "@/stores/authStore";
import { useUIStore } from "@/stores/uiStore";
import {
  Plus, HandCoins, CheckCircle2, XCircle, Wallet, Receipt,
  ArrowLeftRight, Scale, RefreshCw, Banknote,
  Coins
} from "lucide-react";
import type { Employee, Account } from "@/types";

interface OperatingAdvance {
  id: number;
  advance_no: string;
  date: string;
  employee_id: number;
  employee_name: string | null;
  department: string | null;
  purpose: string;
  description: string | null;
  amount_milli: number;
  currency: string;
  exchange_rate: number;
  status: string;
  approval_status: string;
  approved_by: number | null;
  approved_at: string | null;
  disbursed_by: number | null;
  disbursed_at: string | null;
  source_account_code: string | null;
  advance_gl_account_code: string;
  default_expense_account_code: string | null;
  expected_return_date: string | null;
  actual_return_date: string | null;
  total_spent_milli: number;
  total_returned_milli: number;
  balance_milli: number;
  notes: string | null;
  created_by: string;
  created_at: string | null;
  updated_at: string | null;
}

interface AdvanceTransaction {
  id: number;
  advance_id: number;
  ts: string;
  ttype: string;
  amount_milli: number;
  balance_after_milli: number;
  account_code: string | null;
  category: string | null;
  vendor_name: string | null;
  invoice_no: string | null;
  invoice_date: string | null;
  reference: string | null;
  notes: string | null;
  attachment_ids: string | null;
  journal_id: number | null;
  created_by: string;
}

interface AdvanceReceipt {
  id: number;
  advance_id: number;
  transaction_id: number | null;
  receipt_no: string;
  date: string;
  vendor_name: string | null;
  amount_milli: number;
  vat_milli: number;
  net_milli: number;
  category: string | null;
  account_code: string | null;
  description: string | null;
  attachment_ids: string | null;
  status: string;
  approved_by: number | null;
  approved_at: string | null;
  journal_id: number | null;
  created_by: string;
  created_at: string | null;
}

interface AdvanceSummary {
  total_advances_milli: number;
  total_disbursed_milli: number;
  total_spent_milli: number;
  total_returned_milli: number;
  outstanding_balance_milli: number;
  open_advance_count: number;
  pending_approval_count: number;
  pending_receipt_count: number;
}

const emptyForm = {
  employee_id: "",
  purpose: "",
  amount_omr: "",
  description: "",
  expected_return_date: "",
  advance_gl_account_code: "",
  default_expense_account_code: "",
  source_account_code: "",
  notes: "",
};

export default function OperatingAdvancesPage() {
  const { t } = useTranslation();
  const addNotification = useUIStore((s) => s.addNotification);
  const currentUser = useAuthStore((s) => s.user);

  const STATUS_STYLES: Record<string, { label: string; variant: "success" | "warning" | "danger" | "info" | "gold" | "default" }> = {
    draft: { label: t("advances.status.draft"), variant: "default" },
    approved: { label: t("advances.status.approved"), variant: "info" },
    disbursed: { label: t("advances.status.disbursed"), variant: "gold" },
    partially_spent: { label: t("advances.status.partiallySpent"), variant: "warning" },
    reconciled: { label: t("advances.status.reconciled"), variant: "success" },
    closed: { label: t("advances.status.closed"), variant: "success" },
    cancelled: { label: t("advances.status.cancelled"), variant: "danger" },
  };

  const APPROVAL_STYLES: Record<string, { label: string; variant: "success" | "warning" | "danger" | "info" | "gold" | "default" }> = {
    pending: { label: t("advances.approval.pending"), variant: "warning" },
    approved: { label: t("advances.approval.approved"), variant: "success" },
    rejected: { label: t("advances.approval.rejected"), variant: "danger" },
  };

  const TX_TYPE_LABELS: Record<string, string> = {
    disburse: t("advances.tx.disburse"),
    spend: t("advances.tx.spend"),
    return: t("advances.tx.return"),
  };

  const [advances, setAdvances] = useState<OperatingAdvance[]>([]);
  const [employees, setEmployees] = useState<Employee[]>([]);
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [summary, setSummary] = useState<AdvanceSummary | null>(null);
  const [loading, setLoading] = useState(true);
  const [statusFilter, setStatusFilter] = useState("");

  const [showCreate, setShowCreate] = useState(false);
  const [saving, setSaving] = useState(false);
  const [form, setForm] = useState(emptyForm);

  const [selected, setSelected] = useState<OperatingAdvance | null>(null);
  const [transactions, setTransactions] = useState<AdvanceTransaction[]>([]);
  const [receipts, setReceipts] = useState<AdvanceReceipt[]>([]);
  const [detailLoading, setDetailLoading] = useState(false);
  const [actionBusy, setActionBusy] = useState<string | null>(null);

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const [advData, empData, accData, sumData] = await Promise.all([
        invoke("list_operating_advances", { statusFilter: statusFilter || null, employeeId: null, fromDate: null, toDate: null }),
        invoke("list_employees"),
        invoke("list_accounts"),
        invoke("get_advance_summary"),
      ]);
      setAdvances(advData as OperatingAdvance[]);
      setEmployees(empData as Employee[]);
      setAccounts(accData as Account[]);
      setSummary(sumData as AdvanceSummary);
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: String(err) });
    } finally {
      setLoading(false);
    }
  }, [statusFilter, addNotification, t]);

  useEffect(() => { loadData(); }, [loadData]);

  const loadDetail = useCallback(async (adv: OperatingAdvance) => {
    setSelected(adv);
    setDetailLoading(true);
    try {
      const [txns, rcts] = await Promise.all([
        invoke("get_advance_transactions", { advanceId: adv.id }),
        invoke("get_advance_receipts", { advanceId: adv.id }),
      ]);
      setTransactions(txns as AdvanceTransaction[]);
      setReceipts(rcts as AdvanceReceipt[]);
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: String(err) });
    } finally {
      setDetailLoading(false);
    }
  }, [addNotification, t]);

  const refreshAfterAction = useCallback(async (advanceId: number) => {
    await Promise.all([loadData(), invoke("get_operating_advance", { id: advanceId })])
      .then(([_, adv]) => {
        setSelected(adv as OperatingAdvance);
        return invoke("get_advance_transactions", { advanceId }).then((t) => setTransactions(t as AdvanceTransaction[]));
      })
      .catch((err) => addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: String(err) }));
  }, [loadData, addNotification, t]);

  const handleCreate = async () => {
    setSaving(true);
    try {
      await invoke("create_operating_advance", {
        input: {
          employee_id: Number(form.employee_id),
          employee_name: employees.find((e) => e.id === Number(form.employee_id))?.name ?? null,
          purpose: form.purpose,
          description: form.description || null,
          amount_milli: Math.round(Number(form.amount_omr) * 1000),
          source_account_code: form.source_account_code || null,
          advance_gl_account_code: form.advance_gl_account_code || null,
          default_expense_account_code: form.default_expense_account_code || null,
          expected_return_date: form.expected_return_date || null,
          notes: form.notes || null,
          created_by: currentUser?.username ?? "",
        },
      });
      setShowCreate(false);
      setForm(emptyForm);
      addNotification({ id: crypto.randomUUID(), type: "success", title: t("common.success"), message: t("advances.created") });
      loadData();
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: String(err) });
    } finally {
      setSaving(false);
    }
  };

  const runAction = async (key: string, fn: () => Promise<any>) => {
    setActionBusy(key);
    try {
      await fn();
      if (selected) await refreshAfterAction(selected.id);
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: String(err) });
    } finally {
      setActionBusy(null);
    }
  };

  const canApprove = selected && selected.status === "draft";
  const canReject = selected && selected.status === "draft";
  const canDisburse = selected && selected.status === "approved";
  const canSpend = selected && (selected.status === "disbursed" || selected.status === "partially_spent") && selected.balance_milli > 0;
  const canReturn = selected && (selected.status === "disbursed" || selected.status === "partially_spent") && selected.balance_milli > 0;
  const canReconcile = selected && (selected.status === "disbursed" || selected.status === "partially_spent");
  const canCancel = selected && ["draft", "approved", "disbursed"].includes(selected.status);

  const columns: Column<OperatingAdvance>[] = useMemo(() => [
    { key: "advance_no", header: t("advances.advanceNo"), sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.advance_no}</span> },
    { key: "date", header: t("advances.date"), sortable: true, render: (r) => formatDate(r.date) },
    { key: "employee_name", header: t("advances.employee"), sortable: true, render: (r) => <span className="font-medium">{r.employee_name || "—"}</span> },
    { key: "purpose", header: t("advances.purpose"), render: (r) => <span className="text-surface-300">{r.purpose}</span> },
    { key: "amount_milli", header: t("advances.amount"), sortable: true, align: "left", render: (r) => <span className="font-bold">{formatOMR(r.amount_milli)}</span> },
    { key: "total_spent_milli", header: t("advances.spent"), align: "left", render: (r) => <span className="text-amber-400">{formatOMR(r.total_spent_milli)}</span> },
    { key: "balance_milli", header: t("advances.balance"), align: "left", render: (r) => <span className="font-bold text-gold-400">{formatOMR(r.balance_milli)}</span> },
    { key: "approval_status", header: t("advances.approvalStatus"), render: (r) => <Badge variant={APPROVAL_STYLES[r.approval_status]?.variant || "default"}>{APPROVAL_STYLES[r.approval_status]?.label || r.approval_status}</Badge> },
    { key: "status", header: t("advances.statusColumn"), render: (r) => <Badge variant={STATUS_STYLES[r.status]?.variant || "default"}>{STATUS_STYLES[r.status]?.label || r.status}</Badge> },
  ], [t, STATUS_STYLES, APPROVAL_STYLES]);

  const txnColumns: Column<AdvanceTransaction>[] = [
    { key: "ts", header: t("advances.tx.time"), render: (r) => <span className="text-xs">{formatDateTime(r.ts)}</span> },
    { key: "ttype", header: t("advances.tx.type"), render: (r) => (
      <Badge variant={r.ttype === "disburse" ? "gold" : r.ttype === "spend" ? "warning" : "info"}>
        {TX_TYPE_LABELS[r.ttype] || r.ttype}
      </Badge>
    )},
    { key: "amount_milli", header: t("advances.tx.amount"), align: "left", render: (r) => <span className="font-medium">{formatOMR(r.amount_milli)}</span> },
    { key: "balance_after_milli", header: t("advances.tx.balanceAfter"), align: "left", render: (r) => <span className="text-gold-400">{formatOMR(r.balance_after_milli)}</span> },
    { key: "vendor_name", header: t("advances.tx.vendor"), render: (r) => r.vendor_name || "—" },
    { key: "account_code", header: t("advances.tx.account"), render: (r) => <span className="font-mono text-xs">{r.account_code || "—"}</span> },
  ];

  const receiptColumns: Column<AdvanceReceipt>[] = [
    { key: "receipt_no", header: t("advances.receiptNo"), render: (r) => <span className="font-mono text-brand-400">{r.receipt_no}</span> },
    { key: "date", header: t("advances.date"), render: (r) => formatDate(r.date) },
    { key: "vendor_name", header: t("advances.vendor"), render: (r) => r.vendor_name || "—" },
    { key: "amount_milli", header: t("advances.amount"), align: "left", render: (r) => formatOMR(r.amount_milli) },
    { key: "vat_milli", header: t("advances.vat"), align: "left", render: (r) => formatOMR(r.vat_milli) },
    { key: "net_milli", header: t("advances.net"), align: "left", render: (r) => <span className="font-bold text-gold-400">{formatOMR(r.net_milli)}</span> },
    { key: "status", header: t("advances.statusColumn"), render: (r) => (
      <Badge variant={r.status === "approved" ? "success" : "warning"}>
        {r.status === "approved" ? t("advances.approved") : t("advances.pendingApproval")}
      </Badge>
    )},
    { key: "actions", header: t("advances.actions"), render: (r) => (
      r.status === "submitted" ? (
        <Button size="sm" variant="outline" icon={<CheckCircle2 className="w-3.5 h-3.5" />}
          onClick={(e) => { e.stopPropagation(); runAction("receipt", () => invoke("approve_receipt", { receiptId: r.id, approvedBy: currentUser?.id ?? 0 })); }}>
          {t("advances.approve")}
        </Button>
      ) : null
    )},
  ];

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title flex items-center gap-2">
            <HandCoins className="w-6 h-6 text-gold-400" />
            {t("advances.title")}
          </h1>
          <p className="page-subtitle">{t("advances.subtitle")}</p>
        </div>
        <div className="flex items-center gap-2">
          <select value={statusFilter} onChange={(e) => setStatusFilter(e.target.value)} className="input-field text-sm w-44" aria-label={t("advances.filterAria")}>
            <option value="">{t("advances.allStatuses")}</option>
            {Object.entries(STATUS_STYLES).map(([k, v]) => <option key={k} value={k}>{v.label}</option>)}
          </select>
          <Button icon={<Plus className="w-4 h-4" />} onClick={() => setShowCreate(true)}>{t("advances.newAdvance")}</Button>
        </div>
      </div>

      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <Card className="text-center">
          <div className="flex items-center justify-center gap-2 mb-1">
            <Coins className="w-4 h-4 text-brand-400" />
            <p className="text-xs text-surface-400">{t("advances.totalAdvances")}</p>
          </div>
          <p className="text-2xl font-bold gradient-text">{formatOMR(summary?.total_advances_milli || 0)}</p>
          <p className="text-[10px] text-surface-500 mt-1">{t("advances.openCount", { count: summary?.open_advance_count || 0 })}</p>
        </Card>
        <Card className="text-center">
          <div className="flex items-center justify-center gap-2 mb-1">
            <Banknote className="w-4 h-4 text-gold-400" />
            <p className="text-xs text-surface-400">{t("advances.totalDisbursed")}</p>
          </div>
          <p className="text-2xl font-bold text-gold-400">{formatOMR(summary?.total_disbursed_milli || 0)}</p>
          <p className="text-[10px] text-surface-500 mt-1">{t("advances.approvalCount", { count: summary?.pending_approval_count || 0 })}</p>
        </Card>
        <Card className="text-center">
          <div className="flex items-center justify-center gap-2 mb-1">
            <Wallet className="w-4 h-4 text-amber-400" />
            <p className="text-xs text-surface-400">{t("advances.totalSpent")}</p>
          </div>
          <p className="text-2xl font-bold text-amber-400">{formatOMR(summary?.total_spent_milli || 0)}</p>
          <p className="text-[10px] text-surface-500 mt-1">{t("advances.receiptCount", { count: summary?.pending_receipt_count || 0 })}</p>
        </Card>
        <Card className="text-center">
          <div className="flex items-center justify-center gap-2 mb-1">
            <ArrowLeftRight className="w-4 h-4 text-emerald-400" />
            <p className="text-xs text-surface-400">{t("advances.outstanding")}</p>
          </div>
          <p className="text-2xl font-bold text-emerald-400">{formatOMR(summary?.outstanding_balance_milli || 0)}</p>
          <p className="text-[10px] text-surface-500 mt-1">{t("advances.returnedTotal", { amount: formatOMR(summary?.total_returned_milli || 0) })}</p>
        </Card>
      </div>

      <DataTable
        columns={columns}
        data={advances}
        loading={loading}
        onRowClick={(r) => loadDetail(r)}
        emptyMessage={t("advances.empty")}
      />

      <Modal open={showCreate} onClose={() => setShowCreate(false)} title={t("advances.newTitle")} size="lg"
        footer={
          <>
            <Button variant="ghost" onClick={() => setShowCreate(false)}>{t("advances.cancel")}</Button>
            <Button icon={<HandCoins className="w-4 h-4" />} onClick={handleCreate} loading={saving}>{t("advances.create")}</Button>
          </>
        }>
        <div className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div className="input-group">
              <label className="input-label">{t("advances.employeeReq")}</label>
              <select value={form.employee_id} onChange={(e) => setForm({ ...form, employee_id: e.target.value })} className="input-field" aria-label={t("advances.employee")}>
                <option value="">{t("advances.selectEmployee")}</option>
                {employees.map((emp) => <option key={emp.id} value={emp.id}>{emp.name}</option>)}
              </select>
            </div>
            <div className="input-group">
              <label className="input-label">{t("advances.amountOmr")}</label>
              <input type="number" min="0" step="0.001" value={form.amount_omr}
                onChange={(e) => setForm({ ...form, amount_omr: e.target.value })} className="input-field" dir="ltr" placeholder="0.000" aria-label={t("advances.amountOmrAria")} />
            </div>
          </div>
          <div className="input-group">
            <label className="input-label">{t("advances.purposeReq")}</label>
            <input type="text" value={form.purpose} onChange={(e) => setForm({ ...form, purpose: e.target.value })} className="input-field" placeholder={t("advances.purposePlaceholder")} aria-label={t("advances.purpose")} />
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div className="input-group">
              <label className="input-label">{t("advances.advanceAccount")}</label>
              <select value={form.advance_gl_account_code} onChange={(e) => setForm({ ...form, advance_gl_account_code: e.target.value })} className="input-field" aria-label={t("advances.advanceAccountAria")}>
                <option value="">{t("advances.default")}</option>
                {accounts.filter((a) => a.type === "asset").map((a) => <option key={a.code} value={a.code}>{a.code} - {a.name_ar}</option>)}
              </select>
            </div>
            <div className="input-group">
              <label className="input-label">{t("advances.expenseAccount")}</label>
              <select value={form.default_expense_account_code} onChange={(e) => setForm({ ...form, default_expense_account_code: e.target.value })} className="input-field" aria-label={t("advances.expenseAccountAria")}>
                <option value="">{t("advances.default")}</option>
                {accounts.filter((a) => a.type === "expense").map((a) => <option key={a.code} value={a.code}>{a.code} - {a.name_ar}</option>)}
              </select>
            </div>
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div className="input-group">
              <label className="input-label">{t("advances.sourceAccount")}</label>
              <select value={form.source_account_code} onChange={(e) => setForm({ ...form, source_account_code: e.target.value })} className="input-field" aria-label={t("advances.sourceAccountAria")}>
                <option value="">{t("advances.select")}</option>
                {accounts.filter((a) => a.type === "asset").map((a) => <option key={a.code} value={a.code}>{a.code} - {a.name_ar}</option>)}
              </select>
            </div>
            <div className="input-group">
              <label className="input-label">{t("advances.expectedReturn")}</label>
              <input type="date" value={form.expected_return_date} onChange={(e) => setForm({ ...form, expected_return_date: e.target.value })} className="input-field" aria-label={t("advances.expectedReturnAria")} />
            </div>
          </div>
          <div className="input-group">
            <label className="input-label">{t("advances.notes")}</label>
            <input type="text" value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} className="input-field" placeholder={t("advances.notesPlaceholder")} aria-label={t("advances.notes")} />
          </div>
        </div>
      </Modal>

      <Modal open={!!selected} onClose={() => setSelected(null)} title={selected ? t("advances.detailTitle", { no: selected.advance_no }) : ""} size="xl">
        {selected && (
          <div className="space-y-6">
            <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
              <div className="p-3 bg-surface-800/50 rounded-xl">
                <p className="text-xs text-surface-500 mb-1">{t("advances.detailEmployee")}</p>
                <p className="text-sm font-bold text-white">{selected.employee_name || "—"}</p>
                {selected.department && <p className="text-xs text-surface-500">{selected.department}</p>}
              </div>
              <div className="p-3 bg-surface-800/50 rounded-xl">
                <p className="text-xs text-surface-500 mb-1">{t("advances.detailPurpose")}</p>
                <p className="text-sm font-medium text-white">{selected.purpose}</p>
              </div>
              <div className="p-3 bg-surface-800/50 rounded-xl">
                <p className="text-xs text-surface-500 mb-1">{t("advances.detailAmount")}</p>
                <p className="text-sm font-bold text-gold-400">{formatOMR(selected.amount_milli)}</p>
                <p className="text-xs text-surface-500">{t("advances.detailBalance", { amount: formatOMR(selected.balance_milli) })}</p>
              </div>
              <div className="p-3 bg-surface-800/50 rounded-xl">
                <p className="text-xs text-surface-500 mb-1">{t("advances.detailStatus")}</p>
                <div className="space-y-1">
                  <Badge variant={STATUS_STYLES[selected.status]?.variant || "default"}>{STATUS_STYLES[selected.status]?.label || selected.status}</Badge>
                  <Badge variant={APPROVAL_STYLES[selected.approval_status]?.variant || "default"}>{APPROVAL_STYLES[selected.approval_status]?.label || selected.approval_status}</Badge>
                </div>
              </div>
            </div>

            <div className="flex flex-wrap items-center gap-2">
              {canApprove && (
                <Button size="sm" icon={<CheckCircle2 className="w-4 h-4" />} loading={actionBusy === "approve"}
                  onClick={() => runAction("approve", () => invoke("approve_advance", { input: { advance_id: selected.id, approved_by: currentUser?.id ?? 0 } }))}>
                  {t("advances.approveAdvance")}
                </Button>
              )}
              {canReject && (
                <Button size="sm" variant="danger" icon={<XCircle className="w-4 h-4" />} loading={actionBusy === "reject"}
                  onClick={() => runAction("reject", () => invoke("reject_advance", { input: { advance_id: selected.id, rejected_by: currentUser?.id ?? 0, reason: t("advances.rejectReason") } }))}>
                  {t("advances.reject")}
                </Button>
              )}
              {canDisburse && (
                <Button size="sm" icon={<Banknote className="w-4 h-4" />} loading={actionBusy === "disburse"}
                  onClick={() => runAction("disburse", () => invoke("disburse_advance", {
                    input: {
                      advance_id: selected.id,
                      source_account_code: selected.source_account_code || "1100",
                      disbursed_by: currentUser?.id ?? 0,
                      notes: null,
                    },
                  }))}>
                  {t("advances.disburseAdvance")}
                </Button>
              )}
              {canSpend && (
                <Button size="sm" variant="outline" icon={<Wallet className="w-4 h-4" />} loading={actionBusy === "spend"}
                  onClick={() => {
                    const amountOmr = window.prompt(t("advances.spendPrompt"));
                    if (!amountOmr) return;
                    const milli = Math.round(Number(amountOmr) * 1000);
                    if (milli <= 0 || milli > selected.balance_milli) { addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("advances.invalidAmount") }); return; }
                    runAction("spend", () => invoke("record_advance_spend", {
                      input: { advance_id: selected.id, amount_milli: milli, created_by: currentUser?.username ?? "" },
                    }));
                  }}>
                  {t("advances.recordSpend")}
                </Button>
              )}
              {canReturn && (
                <Button size="sm" variant="outline" icon={<ArrowLeftRight className="w-4 h-4" />} loading={actionBusy === "return"}
                  onClick={() => {
                    const amountOmr = window.prompt(t("advances.returnPrompt"));
                    if (!amountOmr) return;
                    const milli = Math.round(Number(amountOmr) * 1000);
                    if (milli <= 0 || milli > selected.balance_milli) { addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("advances.invalidAmount") }); return; }
                    runAction("return", () => invoke("return_advance", {
                      input: {
                        advance_id: selected.id,
                        amount_milli: milli,
                        source_account_code: selected.source_account_code || "1100",
                        created_by: currentUser?.username ?? "",
                      },
                    }));
                  }}>
                  {t("advances.returnAmount")}
                </Button>
              )}
              {canReconcile && (
                <Button size="sm" variant="outline" icon={<Scale className="w-4 h-4" />} loading={actionBusy === "reconcile"}
                  onClick={() => {
                    const physOmr = window.prompt(t("advances.reconcilePrompt"));
                    if (!physOmr) return;
                    runAction("reconcile", () => invoke("reconcile_advance", {
                      input: {
                        advance_id: selected.id,
                        physical_amount_milli: Math.round(Number(physOmr) * 1000),
                        created_by: currentUser?.username ?? "",
                      },
                    }));
                  }}>
                  {t("advances.finalReconcile")}
                </Button>
              )}
              {canCancel && (
                <Button size="sm" variant="danger" icon={<XCircle className="w-4 h-4" />} loading={actionBusy === "cancel"}
                  onClick={() => runAction("cancel", () => invoke("cancel_advance", { advanceId: selected.id, cancelledBy: currentUser?.username ?? "" }))}>
                  {t("advances.cancelAdvance")}
                </Button>
              )}
              {!canApprove && !canReject && !canDisburse && !canSpend && !canReturn && !canReconcile && !canCancel && (
                <p className="text-xs text-surface-500">{t("advances.noActions")}</p>
              )}
            </div>

            {selected.notes && (
              <div className="p-3 bg-surface-800/50 rounded-xl text-sm text-surface-300">
                <span className="text-surface-500 font-medium">{t("advances.notesLabel")} </span>{selected.notes}
              </div>
            )}

            <div>
              <h3 className="section-title flex items-center gap-2">
                <RefreshCw className="w-4 h-4 text-gold-400" />
                {t("advances.movement")}
              </h3>
              <DataTable columns={txnColumns} data={transactions} loading={detailLoading} emptyMessage={t("advances.tx.empty")} compact />
            </div>

            <div>
              <h3 className="section-title flex items-center gap-2">
                <Receipt className="w-4 h-4 text-gold-400" />
                {t("advances.receiptsTitle")}
              </h3>
              <DataTable columns={receiptColumns} data={receipts} loading={detailLoading} emptyMessage={t("advances.receiptsEmpty")} compact />
            </div>
          </div>
        )}
      </Modal>
    </div>
  );
}
