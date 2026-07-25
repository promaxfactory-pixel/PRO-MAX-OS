import { useState, useEffect, useMemo, useCallback } from "react";
import DataTable, { Column } from "@/components/ui/DataTable";
import Button from "@/components/ui/Button";
import Card, { StatCard } from "@/components/ui/Card";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus, Save, Coins, ArrowRight } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

interface PettyCashAccount {
  code: string;
  name: string;
  responsible: string;
  role: string;
  spending_limit_milli: number;
  balance_milli: number;
  status: string;
  notes: string;
}

export default function PettyCashPage() {
  const addNotification = useUIStore((s) => s.addNotification);
  const [accounts, setAccounts] = useState<PettyCashAccount[]>([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [submitting, setSubmitting] = useState(false);

  const [name, setName] = useState("");
  const [code, setCode] = useState("");
  const [responsible, setResponsible] = useState("");
  const [role, setRole] = useState("");
  const [spendingLimitMilli, setSpendingLimitMilli] = useState(0);
  const [notes, setNotes] = useState("");

  const loadAccounts = useCallback(async () => {
    setLoading(true);
    try { const d = await invoke("list_petty_cash_accounts"); setAccounts(d as PettyCashAccount[]); }
    catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء تحميل البيانات" }); }
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
            <Button variant="ghost" icon={<ArrowRight className="w-4 h-4" />} onClick={() => { resetForm(); setShowForm(false); }}>إلغاء</Button>
            <Button icon={<Save className="w-4 h-4" />} onClick={handleSubmit} disabled={submitting}>
              {submitting ? "جاري الحفظ..." : "حفظ الحساب"}
            </Button>
          </div>
        </Card>
      )}

      <DataTable columns={columns} data={accounts} loading={loading} emptyMessage="لا توجد حسابات صرف نثري" />
    </div>
  );
}
