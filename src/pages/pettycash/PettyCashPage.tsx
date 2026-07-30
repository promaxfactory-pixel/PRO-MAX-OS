import { useState, useEffect, useMemo, useCallback } from "react";
import DataTable, { Column } from "@/components/ui/DataTable";
import Button from "@/components/ui/Button";
import Card, { StatCard } from "@/components/ui/Card";
import Modal from "@/components/ui/Modal";
import LoadingSpinner from "@/components/ui/LoadingSpinner";
import { formatOMR, formatDateTime } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus, Save, Coins, FileText, X, Check, Edit2, Search } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { useAuthStore } from "@/stores/authStore";

interface PettyCashAccount {
  id: number;
  code: string;
  name: string;
  responsible: string;
  role: string;
  spending_limit_milli: number;
  balance_milli: number;
  status: string;
  notes: string;
}

interface StatementTxn {
  id: number;
  ts: string;
  petty_id: number;
  ttype: string;
  debit_milli: number;
  credit_milli: number;
  balance_milli: number;
  category: string;
  reference: string;
  notes: string;
  journal_id: number | null;
}

export default function PettyCashPage() {
  const addNotification = useUIStore((s) => s.addNotification);
  const user = useAuthStore((s) => s.user);
  const [accounts, setAccounts] = useState<PettyCashAccount[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [submitting, setSubmitting] = useState(false);

  const [name, setName] = useState("");
  const [code, setCode] = useState("");
  const [responsible, setResponsible] = useState("");
  const [role, setRole] = useState("");
  const [spendingLimitMilli, setSpendingLimitMilli] = useState(0);
  const [notes, setNotes] = useState("");

  const [statementOpen, setStatementOpen] = useState(false);
  const [statementAccount, setStatementAccount] = useState<PettyCashAccount | null>(null);
  const [transactions, setTransactions] = useState<StatementTxn[]>([]);
  const [statementLoading, setStatementLoading] = useState(false);
  const [statementError, setStatementError] = useState<string | null>(null);
  const [fromDate, setFromDate] = useState("");
  const [toDate, setToDate] = useState("");

  const [editingId, setEditingId] = useState<number | null>(null);
  const [editNotes, setEditNotes] = useState("");
  const [editCategory, setEditCategory] = useState("");
  const [editSaving, setEditSaving] = useState(false);

  const loadAccounts = useCallback(async () => {
    setLoading(true);
    setError(null);
    try { const d = await invoke("list_petty_cash_accounts"); setAccounts(d as PettyCashAccount[]); }
    catch (err) { setError(err instanceof Error ? err.message : String(err)); addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء تحميل البيانات" }); }
    finally { setLoading(false); }
  }, [addNotification]);

  useEffect(() => { loadAccounts(); }, [loadAccounts]);

  const resetForm = () => {
    setName("");
    setCode("");
    setResponsible("");
    setRole("");
    setSpendingLimitMilli(0);
    setNotes("");
  };

  const handleSubmit = async () => {
    if (!name || !code || !responsible) return;
    setSubmitting(true);
    try {
      await invoke("create_petty_cash_account", {
        input: {
          name,
          code,
          responsible,
          role: role || null,
          spending_limit_milli: spendingLimitMilli || 0,
          notes: notes || null,
        },
      });
      resetForm();
      setShowForm(false);
      loadAccounts();
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء الحفظ" });
    } finally {
      setSubmitting(false);
    }
  };

  const openStatement = async (account: PettyCashAccount) => {
    setStatementAccount(account);
    setStatementOpen(true);
    setStatementLoading(true);
    setStatementError(null);
    setFromDate("");
    setToDate("");
    setEditingId(null);
    try {
      const d = await invoke("get_custody_statement", { pettyId: account.id, dateFrom: null, dateTo: null });
      setTransactions(d as StatementTxn[]);
    } catch (err) {
      setStatementError(err instanceof Error ? err.message : String(err));
    } finally {
      setStatementLoading(false);
    }
  };

  const filterStatement = async () => {
    if (!statementAccount) return;
    setStatementLoading(true);
    setStatementError(null);
    setEditingId(null);
    try {
      const d = await invoke("get_custody_statement", {
        pettyId: statementAccount.id,
        dateFrom: fromDate || null,
        dateTo: toDate || null,
      });
      setTransactions(d as StatementTxn[]);
    } catch (err) {
      setStatementError(err instanceof Error ? err.message : String(err));
    } finally {
      setStatementLoading(false);
    }
  };

  const startEdit = (txn: StatementTxn) => {
    setEditingId(txn.id);
    setEditNotes(txn.notes || "");
    setEditCategory(txn.category || "");
  };

  const cancelEdit = () => {
    setEditingId(null);
    setEditNotes("");
    setEditCategory("");
  };

  const saveEdit = async () => {
    if (editingId === null) return;
    setEditSaving(true);
    try {
      await invoke("update_custody_spend", {
        userId: user?.id || 0,
        input: { txn_id: editingId, category: editCategory || null, notes: editNotes || null },
      });
      setTransactions(prev => prev.map(t =>
        t.id === editingId ? { ...t, category: editCategory, notes: editNotes } : t
      ));
      setEditingId(null);
      addNotification({ id: crypto.randomUUID(), type: "success", title: "تم", message: "تم تحديث المعاملة" });
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء التحديث" });
    } finally {
      setEditSaving(false);
    }
  };

  if (loading) return <div className="flex items-center justify-center py-16"><LoadingSpinner size="lg" /></div>;

  if (error) return <div className="flex flex-col items-center py-16"><div className="text-6xl mb-4 text-red-400">⚠️</div><h3 className="text-lg font-semibold text-[var(--text-primary)] mb-2">حدث خطأ</h3><p className="text-[var(--text-secondary)] mb-4">{error}</p><button onClick={loadAccounts} className="px-6 py-2.5 bg-brand-500 text-pure-white rounded-xl">إعادة المحاولة</button></div>;

  const totalAccounts = accounts.length;
  const totalBalance = accounts.reduce((s: number, a: PettyCashAccount) => s + (a.balance_milli || 0), 0);
  const activeCount = accounts.filter((a) => a.status === "active").length;

  const columns: Column<PettyCashAccount>[] = useMemo(() => [
    { key: "code", header: "الرمز", sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.code}</span> },
    { key: "name", header: "الاسم", sortable: true, render: (r) => <span className="text-white font-medium">{r.name}</span> },
    { key: "responsible", header: "المسؤول", render: (r) => <span className="text-surface-300">{r.responsible}</span> },
    { key: "spending_limit_milli", header: "حد المصروفات", align: "left", render: (r) => (
      <span className="text-surface-300 font-mono">{formatOMR(r.spending_limit_milli)}</span>
    )},
    { key: "balance_milli", header: "الرصيد", sortable: true, align: "left", render: (r) => (
      <span className="font-bold text-gold-400 font-mono">{formatOMR(r.balance_milli)}</span>
    )},
    { key: "status", header: "الحالة", render: (r) => (
      <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
        r.status === "active" ? "bg-emerald-500/20 text-emerald-400" :
        r.status === "closed" ? "bg-red-500/20 text-red-400" :
        "bg-surface-600 text-surface-300"
      }`}>{r.status === "active" ? "نشط" : r.status === "closed" ? "مغلق" : r.status || "—"}</span>
    )},
    { key: "actions", header: "", render: (r) => (
      <button onClick={(e) => { e.stopPropagation(); openStatement(r); }} className="p-1.5 rounded-lg text-surface-400 hover:text-brand-300 hover:bg-surface-800/50 transition-all" title="كشف حساب">
        <FileText className="w-4 h-4" />
      </button>
    )},
  ], []);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">الصرفات النثرية</h1>
          <p className="page-subtitle">{totalAccounts} حسابات</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => setShowForm(!showForm)}>
          {showForm ? "إخفاء" : "حساب جديد"}
        </Button>
      </div>

      <div className="grid grid-cols-3 gap-4">
        <StatCard title="عدد الحسابات" value={String(totalAccounts)} icon={<Coins className="w-6 h-6" />} />
        <StatCard title="إجمالي الأرصدة" value={formatOMR(totalBalance)} icon={<Coins className="w-6 h-6" />} />
        <StatCard title="الحسابات النشطة" value={String(activeCount)} icon={<Coins className="w-6 h-6" />} />
      </div>

      {showForm && (
        <Card className="p-6">
          <h2 className="text-lg font-semibold text-white mb-4">حساب صرف نثري جديد</h2>
          <div className="grid grid-cols-2 gap-6">
            <div className="input-group">
              <label className="input-label">اسم الحساب *</label>
              <input type="text" className="input-field" value={name} onChange={(e) => setName(e.target.value)} placeholder="مثال: صرف نثري المكتب" aria-label="اسم الحساب" />
            </div>

            <div className="input-group">
              <label className="input-label">الرمز *</label>
              <input type="text" className="input-field" value={code} onChange={(e) => setCode(e.target.value)} placeholder="مثال: PC-001" aria-label="الرمز" />
            </div>

            <div className="input-group">
              <label className="input-label">المسؤول *</label>
              <input type="text" className="input-field" value={responsible} onChange={(e) => setResponsible(e.target.value)} placeholder="اسم المسؤول" aria-label="المسؤول" />
            </div>

            <div className="input-group">
              <label className="input-label">الدور</label>
              <input type="text" className="input-field" value={role} onChange={(e) => setRole(e.target.value)} placeholder="اختياري" aria-label="الدور" />
            </div>

            <div className="input-group">
              <label className="input-label">حد المصروفات (ملي)</label>
              <input type="number" className="input-field" min={0} value={spendingLimitMilli} onChange={(e) => setSpendingLimitMilli(Number(e.target.value))} aria-label="حد المصروفات" />
            </div>

            <div className="input-group">
              <label className="input-label">ملاحظات</label>
              <input type="text" className="input-field" value={notes} onChange={(e) => setNotes(e.target.value)} placeholder="اختياري" aria-label="ملاحظات" />
            </div>
          </div>

          <div className="flex justify-end gap-3 mt-6">
            <Button variant="ghost" onClick={() => { resetForm(); setShowForm(false); }}>إلغاء</Button>
            <Button icon={<Save className="w-4 h-4" />} onClick={handleSubmit} disabled={submitting}>
              {submitting ? "جاري الحفظ..." : "حفظ الحساب"}
            </Button>
          </div>
        </Card>
      )}

      <DataTable columns={columns} data={accounts} loading={loading} emptyMessage="لا توجد حسابات صرف نثري" />

      <Modal open={statementOpen} onClose={() => { setStatementOpen(false); setEditingId(null); }} title={statementAccount ? `كشف حساب: ${statementAccount.name}` : ''} size="xl">
        <div className="flex items-center gap-3 mb-4">
          <div className="flex items-center gap-2">
            <span className="text-sm text-[var(--text-secondary)]">من</span>
            <input type="date" value={fromDate} onChange={(e) => setFromDate(e.target.value)} className="input-field text-sm" aria-label="من تاريخ" />
            <span className="text-sm text-[var(--text-secondary)]">إلى</span>
            <input type="date" value={toDate} onChange={(e) => setToDate(e.target.value)} className="input-field text-sm" aria-label="إلى تاريخ" />
          </div>
          <Button size="sm" onClick={filterStatement} loading={statementLoading} icon={<Search className="w-3 h-3" />}>بحث</Button>
          {(fromDate || toDate) && (
            <Button size="sm" variant="ghost" onClick={() => { setFromDate(""); setToDate(""); filterStatement(); }}>إعادة تعيين</Button>
          )}
        </div>

        {statementLoading ? (
          <div className="flex items-center justify-center py-16"><LoadingSpinner /></div>
        ) : statementError ? (
          <div className="flex flex-col items-center py-12"><div className="text-4xl mb-3 text-red-400">⚠️</div><p className="text-red-400 mb-3">{statementError}</p><Button size="sm" onClick={filterStatement}>إعادة المحاولة</Button></div>
        ) : transactions.length === 0 ? (
          <div className="text-center py-12 text-[var(--text-secondary)]">لا توجد معاملات</div>
        ) : (
          <div className="overflow-x-auto">
            <table className="data-table w-full" role="grid" aria-label="معاملات الحساب">
              <thead>
                <tr role="row">
                  <th role="columnheader" className="text-right">التاريخ</th>
                  <th role="columnheader" className="text-right">النوع</th>
                  <th role="columnheader" className="text-left">مدين</th>
                  <th role="columnheader" className="text-left">دائن</th>
                  <th role="columnheader" className="text-left">الرصيد</th>
                  <th role="columnheader" className="text-right">التصنيف</th>
                  <th role="columnheader" className="text-right">ملاحظات</th>
                  <th role="columnheader" className="text-center"></th>
                </tr>
              </thead>
              <tbody>
                {transactions.map((txn) => (
                  <tr key={txn.id} role="row" className="hover:bg-surface-800/30 transition-colors">
                    <td role="cell" className="text-sm font-mono text-surface-300">{formatDateTime(txn.ts)}</td>
                    <td role="cell"><span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-surface-800 text-surface-300">{txn.ttype}</span></td>
                    <td role="cell" className="text-left font-mono text-sm">{txn.debit_milli > 0 ? <span className="text-emerald-400">{formatOMR(txn.debit_milli)}</span> : "—"}</td>
                    <td role="cell" className="text-left font-mono text-sm">{txn.credit_milli > 0 ? <span className="text-red-400">{formatOMR(txn.credit_milli)}</span> : "—"}</td>
                    <td role="cell" className="text-left font-mono text-sm font-bold text-gold-400">{formatOMR(txn.balance_milli)}</td>
                    <td role="cell">
                      {editingId === txn.id ? (
                        <input type="text" value={editCategory} onChange={(e) => setEditCategory(e.target.value)} className="input-field text-xs py-1 px-2 w-28" aria-label="تعديل التصنيف" />
                      ) : (
                        <span className="text-sm">{txn.category || "—"}</span>
                      )}
                    </td>
                    <td role="cell">
                      {editingId === txn.id ? (
                        <input type="text" value={editNotes} onChange={(e) => setEditNotes(e.target.value)} className="input-field text-xs py-1 px-2 w-36" aria-label="تعديل الملاحظات" />
                      ) : (
                        <span className="text-sm text-surface-400">{txn.notes || "—"}</span>
                      )}
                    </td>
                    <td role="cell" className="text-center">
                      {editingId === txn.id ? (
                        <div className="flex items-center justify-center gap-1">
                          <button onClick={saveEdit} disabled={editSaving} className="p-1 rounded hover:bg-surface-700/50 transition-colors" aria-label="حفظ"><Check className="w-4 h-4 text-emerald-400" /></button>
                          <button onClick={cancelEdit} className="p-1 rounded hover:bg-surface-700/50 transition-colors" aria-label="إلغاء"><X className="w-4 h-4 text-red-400" /></button>
                        </div>
                      ) : (
                        <button onClick={() => startEdit(txn)} className="p-1 rounded hover:bg-surface-700/50 transition-colors" aria-label="تعديل"><Edit2 className="w-4 h-4 text-surface-500 hover:text-brand-300" /></button>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </Modal>
    </div>
  );
}
