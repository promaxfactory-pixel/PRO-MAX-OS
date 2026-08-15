import { useState, useEffect, useMemo, useCallback } from "react";
import DataTable, { Column } from "@/components/ui/DataTable";
import Button from "@/components/ui/Button";
import { StatCard } from "@/components/ui/Card";
import Modal from "@/components/ui/Modal";
import LoadingSpinner from "@/components/ui/LoadingSpinner";
import { formatOMR, formatDateTime, omrToMilli } from "@/lib/utils";import { invoke } from "@/lib/tauri";
import { Plus, Coins, FileText, Banknote, ArrowLeftRight, Wallet, Save } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import type { CustodyAccount, CustodyTransaction } from "@/types";

type ModalKind = "fund" | "spend" | "transfer" | "statement" | null;

export default function CustodyPage() {
  const addNotification = useUIStore((s) => s.addNotification);
  const [accounts, setAccounts] = useState<CustodyAccount[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [modal, setModal] = useState<ModalKind>(null);
  const [submitting, setSubmitting] = useState(false);

  const [active, setActive] = useState<CustodyAccount | null>(null);
  const [fundName, setFundName] = useState("");
  const [fundResponsible, setFundResponsible] = useState("");
  const [fundLimitOmr, setFundLimitOmr] = useState("");
  const [fundOpeningOmr, setFundOpeningOmr] = useState("");
  const [fundNotes, setFundNotes] = useState("");

  const [amountOmr, setAmountOmr] = useState("");
  const [category, setCategory] = useState("");
  const [reference, setReference] = useState("");
  const [txnDate, setTxnDate] = useState("");
  const [txnNotes, setTxnNotes] = useState("");
  const [toAccountId, setToAccountId] = useState<number | null>(null);

  const [transactions, setTransactions] = useState<CustodyTransaction[]>([]);
  const [statementLoading, setStatementLoading] = useState(false);
  const [fromDate, setFromDate] = useState("");
  const [toDate, setToDate] = useState("");

  const loadAccounts = useCallback(async () => {
    setLoading(true);
    setError(null);
    try { const d = await invoke("list_custody_accounts"); setAccounts(d as CustodyAccount[]); }
    catch (err) { setError(err instanceof Error ? err.message : String(err)); addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء تحميل البيانات" }); }
    finally { setLoading(false); }
  }, [addNotification]);

  useEffect(() => { loadAccounts(); }, [loadAccounts]);

  const resetForm = () => {
    setFundName(""); setFundResponsible(""); setFundLimitOmr(""); setFundOpeningOmr(""); setFundNotes("");
    setAmountOmr(""); setCategory(""); setReference(""); setTxnDate(""); setTxnNotes(""); setToAccountId(null);
  };

  const openFund = () => { resetForm(); setModal("fund"); };
  const openSpend = (acc: CustodyAccount) => { resetForm(); setActive(acc); setModal("spend"); };
  const openTransfer = (acc: CustodyAccount) => { resetForm(); setActive(acc); setToAccountId(accounts.find(a => a.id !== acc.id)?.id ?? null); setModal("transfer"); };

  const openStatement = async (acc: CustodyAccount) => {
    setActive(acc);
    setModal("statement");
    setStatementLoading(true);
    setFromDate(""); setToDate("");
    try {
      const d = await invoke("get_custody_statement", { pettyId: acc.id, dateFrom: null, dateTo: null });
      setTransactions(d as CustodyTransaction[]);
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) });
      setTransactions([]);
    } finally { setStatementLoading(false); }
  };

  const filterStatement = async () => {
    if (!active) return;
    setStatementLoading(true);
    try {
      const d = await invoke("get_custody_statement", {
        pettyId: active.id,
        dateFrom: fromDate || null,
        dateTo: toDate || null,
      });
      setTransactions(d as CustodyTransaction[]);
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) });
    } finally { setStatementLoading(false); }
  };

  const handleCreateFund = async () => {
    if (!fundName.trim()) return;
    setSubmitting(true);
    try {
      await invoke("create_custody_fund", {
        input: {
          name: fundName.trim(),
          responsible: fundResponsible.trim() || null,
          employee_id: null,
          spending_limit_milli: omrToMilli(Number(fundLimitOmr) || 0),
          opening_balance_milli: omrToMilli(Number(fundOpeningOmr) || 0),
          notes: fundNotes.trim() || null,
        },
      });
      setModal(null);
      addNotification({ id: crypto.randomUUID(), type: "success", title: "تم بنجاح", message: "تم إنشاء الصندوق بنجاح" });
      await loadAccounts();
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) });
    } finally { setSubmitting(false); }
  };

  const handleSpend = async () => {
    if (!active || !amountOmr || Number(amountOmr) <= 0) return;
    setSubmitting(true);
    try {
      await invoke("create_custody_spend", {
        input: {
          petty_id: active.id,
          amount_milli: omrToMilli(Number(amountOmr)),
          category: category.trim() || null,
          reference: reference.trim() || null,
          notes: txnNotes.trim() || null,
          date: txnDate || null,
        },
      });
      setModal(null);
      addNotification({ id: crypto.randomUUID(), type: "success", title: "تم بنجاح", message: "تم تسجيل الصرف" });
      await loadAccounts();
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) });
    } finally { setSubmitting(false); }
  };

  const handleTransfer = async () => {
    if (!active || !toAccountId || !amountOmr || Number(amountOmr) <= 0) return;
    setSubmitting(true);
    try {
      await invoke("create_custody_transfer", {
        input: {
          from_petty_id: active.id,
          to_petty_id: toAccountId,
          amount_milli: omrToMilli(Number(amountOmr)),
          notes: txnNotes.trim() || null,
        },
      });
      setModal(null);
      addNotification({ id: crypto.randomUUID(), type: "success", title: "تم بنجاح", message: "تم التحويل بنجاح" });
      await loadAccounts();
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) });
    } finally { setSubmitting(false); }
  };

  if (loading) return <div className="flex items-center justify-center py-16"><LoadingSpinner size="lg" /></div>;

  if (error) return <div className="flex flex-col items-center py-16"><div className="text-6xl mb-4 text-red-400">⚠️</div><h3 className="text-lg font-semibold text-[var(--text-primary)] mb-2">حدث خطأ</h3><p className="text-[var(--text-secondary)] mb-4">{error}</p><button onClick={loadAccounts} className="px-6 py-2.5 bg-brand-500 text-pure-white rounded-xl">إعادة المحاولة</button></div>;

  const totalBalance = accounts.reduce((s, a) => s + (a.balance_milli || 0), 0);
  const totalLimit = accounts.reduce((s, a) => s + (a.spending_limit_milli || 0), 0);

  const columns: Column<CustodyAccount>[] = useMemo(() => [
    { key: "code", header: "الرمز", sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.code || `#${r.id}`}</span> },
    { key: "name", header: "الاسم", sortable: true, render: (r) => <span className="text-white font-medium">{r.name}</span> },
    { key: "responsible", header: "المسؤول", render: (r) => <span className="text-surface-300">{r.responsible || "—"}</span> },
    { key: "spending_limit_milli", header: "حد الصرف", align: "left", render: (r) => <span className="text-surface-300 font-mono">{formatOMR(r.spending_limit_milli)}</span> },
    { key: "balance_milli", header: "الرصيد", sortable: true, align: "left", render: (r) => <span className="font-bold text-gold-400 font-mono">{formatOMR(r.balance_milli)}</span> },
    { key: "actions", header: "إجراءات", align: "center", render: (r) => (
      <div className="flex items-center justify-center gap-1">
        <button onClick={(e) => { e.stopPropagation(); openStatement(r); }} className="p-1.5 rounded-lg text-surface-400 hover:text-brand-300 hover:bg-surface-800/50 transition-all" title="كشف حساب"><FileText className="w-4 h-4" /></button>
        <button onClick={(e) => { e.stopPropagation(); openSpend(r); }} className="p-1.5 rounded-lg text-surface-400 hover:text-amber-300 hover:bg-surface-800/50 transition-all" title="صرف"><Wallet className="w-4 h-4" /></button>
        <button onClick={(e) => { e.stopPropagation(); openTransfer(r); }} className="p-1.5 rounded-lg text-surface-400 hover:text-emerald-300 hover:bg-surface-800/50 transition-all" title="تحويل"><ArrowLeftRight className="w-4 h-4" /></button>
      </div>
    )},
  ], []);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">العهد والصرف النثري</h1>
          <p className="page-subtitle">{accounts.length} صندوق • إجمالي الأرصدة {formatOMR(totalBalance)}</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={openFund}>إنشاء صندوق</Button>
      </div>

      <div className="grid grid-cols-3 gap-4">
        <StatCard title="عدد الصناديق" value={String(accounts.length)} icon={<Coins className="w-6 h-6" />} />
        <StatCard title="إجمالي الأرصدة" value={formatOMR(totalBalance)} icon={<Coins className="w-6 h-6" />} />
        <StatCard title="إجمالي حدود الصرف" value={formatOMR(totalLimit)} icon={<Banknote className="w-6 h-6" />} />
      </div>

      <DataTable columns={columns} data={accounts} loading={loading} emptyMessage="لا توجد صناديق — أنشئ أول صندوق عهدة" />

      <Modal open={modal === "fund"} onClose={() => setModal(null)} title="إنشاء صندوق عهدة" footer={
        <>
          <Button variant="outline" onClick={() => setModal(null)}>إلغاء</Button>
          <Button onClick={handleCreateFund} loading={submitting} icon={<Save className="w-4 h-4" />}>إنشاء</Button>
        </>
      }>
        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="block text-sm text-surface-400 mb-1">اسم الصندوق *</label>
            <input type="text" className="w-full input-field" value={fundName} onChange={(e) => setFundName(e.target.value)} placeholder="مثال: عهدة مصنع الكؤوس" aria-label="اسم الصندوق" />
          </div>
          <div>
            <label className="block text-sm text-surface-400 mb-1">المسؤول</label>
            <input type="text" className="w-full input-field" value={fundResponsible} onChange={(e) => setFundResponsible(e.target.value)} placeholder="اختياري" aria-label="المسؤول" />
          </div>
          <div>
            <label className="block text-sm text-surface-400 mb-1">حد الصرف (بالريال)</label>
            <input type="number" min={0} step={0.001} className="w-full input-field" value={fundLimitOmr} onChange={(e) => setFundLimitOmr(e.target.value)} placeholder="0.000" aria-label="حد الصرف بالريال" />
          </div>
          <div>
            <label className="block text-sm text-surface-400 mb-1">الرصيد الافتتاحي (بالريال)</label>
            <input type="number" min={0} step={0.001} className="w-full input-field" value={fundOpeningOmr} onChange={(e) => setFundOpeningOmr(e.target.value)} placeholder="0.000" aria-label="الرصيد الافتتاحي بالريال" />
          </div>
          <div className="col-span-2">
            <label className="block text-sm text-surface-400 mb-1">ملاحظات</label>
            <textarea className="w-full input-field" rows={2} value={fundNotes} onChange={(e) => setFundNotes(e.target.value)} aria-label="ملاحظات" />
          </div>
        </div>
      </Modal>

      <Modal open={modal === "spend"} onClose={() => setModal(null)} title={active ? `صرف من: ${active.name}` : "صرف"} footer={
        <>
          <Button variant="outline" onClick={() => setModal(null)}>إلغاء</Button>
          <Button onClick={handleSpend} loading={submitting} icon={<Wallet className="w-4 h-4" />}>تسجيل الصرف</Button>
        </>
      }>
        {active && (
          <div className="space-y-4">
            <div className="flex justify-between text-sm rounded-xl bg-surface-900/50 px-4 py-3">
              <span className="text-surface-400">الرصيد الحالي</span>
              <span className="font-bold text-gold-400">{formatOMR(active.balance_milli)}</span>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm text-surface-400 mb-1">المبلغ (بالريال) *</label>
                <input type="number" min={0.001} step={0.001} className="w-full input-field" value={amountOmr} onChange={(e) => setAmountOmr(e.target.value)} placeholder="0.000" aria-label="المبلغ بالريال" />
              </div>
              <div>
                <label className="block text-sm text-surface-400 mb-1">التاريخ</label>
                <input type="date" className="w-full input-field" value={txnDate} onChange={(e) => setTxnDate(e.target.value)} aria-label="التاريخ" />
              </div>
              <div>
                <label className="block text-sm text-surface-400 mb-1">التصنيف</label>
                <input type="text" className="w-full input-field" value={category} onChange={(e) => setCategory(e.target.value)} placeholder="مثال: قرطاسية" aria-label="التصنيف" />
              </div>
              <div>
                <label className="block text-sm text-surface-400 mb-1">المرجع</label>
                <input type="text" className="w-full input-field" value={reference} onChange={(e) => setReference(e.target.value)} placeholder="اختياري" aria-label="المرجع" />
              </div>
              <div className="col-span-2">
                <label className="block text-sm text-surface-400 mb-1">ملاحظات</label>
                <textarea className="w-full input-field" rows={2} value={txnNotes} onChange={(e) => setTxnNotes(e.target.value)} aria-label="ملاحظات" />
              </div>
            </div>
            {Number(amountOmr) > 0 && <p className="text-xs text-surface-500">= {omrToMilli(Number(amountOmr)).toLocaleString("en-US")} بيسة/ملي</p>}
          </div>
        )}
      </Modal>

      <Modal open={modal === "transfer"} onClose={() => setModal(null)} title={active ? `تحويل من: ${active.name}` : "تحويل"} footer={
        <>
          <Button variant="outline" onClick={() => setModal(null)}>إلغاء</Button>
          <Button onClick={handleTransfer} loading={submitting} icon={<ArrowLeftRight className="w-4 h-4" />}>تحويل</Button>
        </>
      }>
        {active && (
          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm text-surface-400 mb-1">إلى الصندوق *</label>
                <select className="w-full input-field" value={toAccountId ?? ""} onChange={(e) => setToAccountId(Number(e.target.value))} aria-label="الصندوق المستهدف">
                  <option value="" disabled>اختر الصندوق</option>
                  {accounts.filter(a => a.id !== active.id).map(a => <option key={a.id} value={a.id}>{a.name} — {formatOMR(a.balance_milli)}</option>)}
                </select>
              </div>
              <div>
                <label className="block text-sm text-surface-400 mb-1">المبلغ (بالريال) *</label>
                <input type="number" min={0.001} step={0.001} className="w-full input-field" value={amountOmr} onChange={(e) => setAmountOmr(e.target.value)} placeholder="0.000" aria-label="المبلغ بالريال" />
              </div>
              <div className="col-span-2">
                <label className="block text-sm text-surface-400 mb-1">ملاحظات</label>
                <textarea className="w-full input-field" rows={2} value={txnNotes} onChange={(e) => setTxnNotes(e.target.value)} aria-label="ملاحظات" />
              </div>
            </div>
            {Number(amountOmr) > 0 && <p className="text-xs text-surface-500">= {omrToMilli(Number(amountOmr)).toLocaleString("en-US")} بيسة/ملي</p>}
          </div>
        )}
      </Modal>

      <Modal open={modal === "statement"} onClose={() => setModal(null)} title={active ? `كشف حساب: ${active.name}` : "كشف حساب"} size="xl">
        <div className="flex items-center gap-3 mb-4">
          <span className="text-sm text-surface-400">من</span>
          <input type="date" value={fromDate} onChange={(e) => setFromDate(e.target.value)} className="input-field text-sm" aria-label="من تاريخ" />
          <span className="text-sm text-surface-400">إلى</span>
          <input type="date" value={toDate} onChange={(e) => setToDate(e.target.value)} className="input-field text-sm" aria-label="إلى تاريخ" />
          <Button size="sm" onClick={filterStatement} loading={statementLoading}>بحث</Button>
          {(fromDate || toDate) && <Button size="sm" variant="ghost" onClick={() => { setFromDate(""); setToDate(""); filterStatement(); }}>إعادة تعيين</Button>}
        </div>

        {statementLoading ? (
          <div className="flex items-center justify-center py-16"><LoadingSpinner /></div>
        ) : transactions.length === 0 ? (
          <div className="text-center py-12 text-surface-400">لا توجد معاملات</div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="bg-surface-800 text-surface-400">
                  <th className="px-4 py-2 text-right font-medium">التاريخ</th>
                  <th className="px-4 py-2 text-right font-medium">النوع</th>
                  <th className="px-4 py-2 text-left font-medium">مدين</th>
                  <th className="px-4 py-2 text-left font-medium">دائن</th>
                  <th className="px-4 py-2 text-left font-medium">الرصيد</th>
                  <th className="px-4 py-2 text-right font-medium">التصنيف</th>
                  <th className="px-4 py-2 text-right font-medium">ملاحظات</th>
                </tr>
              </thead>
              <tbody>
                {transactions.map((txn) => (
                  <tr key={txn.id} className="border-t border-surface-700 hover:bg-surface-800/30 transition-colors">
                    <td className="px-4 py-2 font-mono text-xs text-surface-300">{formatDateTime(txn.ts)}</td>
                    <td className="px-4 py-2"><span className="inline-flex px-2 py-0.5 rounded text-xs font-medium bg-surface-800 text-surface-300">{txn.ttype}</span></td>
                    <td className="px-4 py-2 text-left font-mono">{txn.debit_milli > 0 ? <span className="text-emerald-400">{formatOMR(txn.debit_milli)}</span> : "—"}</td>
                    <td className="px-4 py-2 text-left font-mono">{txn.credit_milli > 0 ? <span className="text-red-400">{formatOMR(txn.credit_milli)}</span> : "—"}</td>
                    <td className="px-4 py-2 text-left font-mono font-bold text-gold-400">{formatOMR(txn.balance_milli)}</td>
                    <td className="px-4 py-2">{txn.category || "—"}</td>
                    <td className="px-4 py-2 text-surface-400">{txn.notes || "—"}</td>
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
