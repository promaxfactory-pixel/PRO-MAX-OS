import { useState, useEffect } from "react";
import DataTable, { Column } from "@/components/ui/DataTable";
import Button from "@/components/ui/Button";
import Card, { StatCard } from "@/components/ui/Card";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus, Save, Wallet, ArrowRight } from "lucide-react";

export default function CashBankPage() {
  const [accounts, setAccounts] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [submitting, setSubmitting] = useState(false);

  const [name, setName] = useState("");
  const [code, setCode] = useState("");
  const [atype, setAtype] = useState("cash");

  useEffect(() => { loadAccounts(); }, []);

  const loadAccounts = async () => {
    setLoading(true);
    try { const d = await invoke("list_cashbank_accounts"); setAccounts(d as any[]); }
    catch (err) { console.error(err); }
    finally { setLoading(false); }
  };

  const resetForm = () => {
    setName("");
    setCode("");
    setAtype("cash");
  };

  const handleSubmit = async () => {
    if (!name || !code) return;
    setSubmitting(true);
    try {
      await invoke("create_cashbank_account", {
        input: { name, code, atype },
      });
      resetForm();
      setShowForm(false);
      loadAccounts();
    } catch (err) {
      console.error(err);
    } finally {
      setSubmitting(false);
    }
  };

  const totalCash = accounts
    .filter((a) => a.atype === "cash")
    .reduce((s: number, a: any) => s + (a.balance_milli || 0), 0);
  const totalBank = accounts
    .filter((a) => a.atype === "bank")
    .reduce((s: number, a: any) => s + (a.balance_milli || 0), 0);
  const combined = totalCash + totalBank;

  const columns: Column<any>[] = [
    { key: "code", header: "الرمز", sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.code}</span> },
    { key: "name", header: "الاسم", sortable: true, render: (r) => <span className="text-white font-medium">{r.name}</span> },
    { key: "atype", header: "النوع", render: (r) => (
      <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
        r.atype === "cash" ? "bg-emerald-500/20 text-emerald-400" : "bg-blue-500/20 text-blue-400"
      }`}>{r.atype === "cash" ? "نقدي" : "بنكي"}</span>
    )},
    { key: "balance_milli", header: "الرصيد", sortable: true, align: "left", render: (r) => (
      <span className="font-bold text-gold-400 font-mono">{formatOMR(r.balance_milli)}</span>
    )},
    { key: "active", header: "الحالة", render: (r) => (
      <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
        r.active ? "bg-emerald-500/20 text-emerald-400" : "bg-surface-600 text-surface-400"
      }`}>{r.active ? "نشط" : "غير نشط"}</span>
    )},
  ];

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">الحسابات النقدية والبنكية</h1>
          <p className="page-subtitle">{accounts.length} حسابات</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => setShowForm(!showForm)}>
          {showForm ? "إخفاء" : "حساب جديد"}
        </Button>
      </div>

      <div className="grid grid-cols-3 gap-4">
        <StatCard title="إجمالي النقدية" value={formatOMR(totalCash)} icon={<Wallet className="w-6 h-6" />} />
        <StatCard title="إجمالي البنكية" value={formatOMR(totalBank)} icon={<Wallet className="w-6 h-6" />} />
        <StatCard title="الإجمالي" value={formatOMR(combined)} icon={<Wallet className="w-6 h-6" />} />
      </div>

      {showForm && (
        <Card className="p-6">
          <h2 className="text-lg font-semibold text-white mb-4">حساب جديد</h2>
          <div className="grid grid-cols-3 gap-6">
            <div className="input-group">
              <label className="input-label">اسم الحساب *</label>
              <input type="text" className="input-field" value={name} onChange={(e) => setName(e.target.value)} placeholder="مثال: الصندوق الرئيسي" />
            </div>

            <div className="input-group">
              <label className="input-label">الرمز *</label>
              <input type="text" className="input-field" value={code} onChange={(e) => setCode(e.target.value)} placeholder="مثال: CB-001" />
            </div>

            <div className="input-group">
              <label className="input-label">النوع *</label>
              <select className="input-field" value={atype} onChange={(e) => setAtype(e.target.value)}>
                <option value="cash">نقدي</option>
                <option value="bank">بنكي</option>
              </select>
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

      <DataTable columns={columns} data={accounts} loading={loading} emptyMessage="لا توجد حسابات" />
    </div>
  );
}
