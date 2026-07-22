import { useState, useEffect } from "react";
import DataTable, { Column } from "@/components/ui/DataTable";
import Button from "@/components/ui/Button";
import Card, { StatCard } from "@/components/ui/Card";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus, Save, Receipt, ArrowRight } from "lucide-react";

const CATEGORIES = [
  "أجر", "إيجار", "مواصلات", "كهرباء", "مياه",
  "صيانة", "مكاتب", "اتصالات", "تأمين", "أخرى",
];

const METHODS = [
  { value: "cash", label: "نقدي" },
  { value: "bank_transfer", label: "تحويل بنكي" },
  { value: "cheque", label: "شيك" },
];

export default function ExpensesPage() {
  const [expenses, setExpenses] = useState<any[]>([]);
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

  useEffect(() => { loadExpenses(); }, []);

  const loadExpenses = async () => {
    setLoading(true);
    try { const d = await invoke("list_expenses"); setExpenses(d as any[]); }
    catch (err) { console.error(err); }
    finally { setLoading(false); }
  };

  const resetForm = () => {
    setDate(new Date().toISOString().split("T")[0]);
    setCategory(CATEGORIES[0]);
    setAccountCode("");
    setAmountMilli(0);
    setVatMilli(0);
    setMethod("cash");
    setVendor("");
    setReference("");
    setNotes("");
  };

  const handleSubmit = async () => {
    if (!date || !category || amountMilli <= 0) return;
    setSubmitting(true);
    try {
      await invoke("create_expense", {
        input: {
          date,
          category,
          account_code: accountCode || null,
          amount_milli: amountMilli,
          vat_milli: vatMilli || 0,
          method,
          vendor: vendor || null,
          reference: reference || null,
          notes: notes || null,
        },
      });
      resetForm();
      setShowForm(false);
      loadExpenses();
    } catch (err) {
      console.error(err);
    } finally {
      setSubmitting(false);
    }
  };

  const totalExpenses = expenses.reduce((s: number, e: any) => s + (e.amount_milli || 0), 0);
  const count = expenses.length;
  const average = count > 0 ? Math.round(totalExpenses / count) : 0;

  const methodLabel = (m: string) => METHODS.find((x) => x.value === m)?.label || m;

  const columns: Column<any>[] = [
    { key: "exp_no", header: "رقم المصروف", sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.exp_no || "—"}</span> },
    { key: "date", header: "التاريخ", sortable: true, render: (r) => formatDate(r.date) },
    { key: "category", header: "التصنيف", render: (r) => <span className="text-surface-300">{r.category}</span> },
    { key: "vendor", header: "المورد / الجهة", render: (r) => r.vendor || "—" },
    { key: "amount_milli", header: "المبلغ", sortable: true, align: "left", render: (r) => <span className="font-bold text-gold-400 font-mono">{formatOMR(r.amount_milli)}</span> },
    { key: "method", header: "الطريقة", render: (r) => <span className="text-surface-300">{methodLabel(r.method)}</span> },
    { key: "approval_status", header: "الحالة", render: (r) => (
      <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
        r.approval_status === 'approved' ? 'bg-emerald-500/20 text-emerald-400' :
        r.approval_status === 'pending' ? 'bg-yellow-500/20 text-yellow-400' :
        'bg-surface-600 text-surface-300'
      }`}>{r.approval_status === 'approved' ? ' معتمد' : r.approval_status === 'pending' ? 'قيد المراجعة' : r.approval_status || "—"}</span>
    )},
  ];

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">المصروفات</h1>
          <p className="page-subtitle">{count} مصروفات</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => setShowForm(!showForm)}>
          {showForm ? "إخفاء" : "مصروف جديد"}
        </Button>
      </div>

      <div className="grid grid-cols-3 gap-4">
        <StatCard title="إجمالي المصروفات" value={formatOMR(totalExpenses)} icon={<Receipt className="w-6 h-6" />} />
        <StatCard title="عدد المصروفات" value={String(count)} icon={<Receipt className="w-6 h-6" />} />
        <StatCard title="المتوسط" value={formatOMR(average)} icon={<Receipt className="w-6 h-6" />} />
      </div>

      {showForm && (
        <Card className="p-6">
          <h2 className="text-lg font-semibold text-white mb-4">مصروف جديد</h2>
          <div className="grid grid-cols-2 gap-6">
            <div className="input-group">
              <label className="input-label">التاريخ *</label>
              <input type="date" className="input-field" value={date} onChange={(e) => setDate(e.target.value)} />
            </div>

            <div className="input-group">
              <label className="input-label">التصنيف *</label>
              <select className="input-field" value={category} onChange={(e) => setCategory(e.target.value)}>
                {CATEGORIES.map((c) => (
                  <option key={c} value={c}>{c}</option>
                ))}
              </select>
            </div>

            <div className="input-group">
              <label className="input-label">رمز الحساب</label>
              <input type="text" className="input-field" value={accountCode} onChange={(e) => setAccountCode(e.target.value)} placeholder="اختياري" />
            </div>

            <div className="input-group">
              <label className="input-label">المبلغ (ملي) *</label>
              <input type="number" className="input-field" min={0} value={amountMilli} onChange={(e) => setAmountMilli(Number(e.target.value))} />
            </div>

            <div className="input-group">
              <label className="input-label">الضريبة (ملي)</label>
              <input type="number" className="input-field" min={0} value={vatMilli} onChange={(e) => setVatMilli(Number(e.target.value))} />
            </div>

            <div className="input-group">
              <label className="input-label">طريقة الدفع *</label>
              <select className="input-field" value={method} onChange={(e) => setMethod(e.target.value)}>
                {METHODS.map((m) => (
                  <option key={m.value} value={m.value}>{m.label}</option>
                ))}
              </select>
            </div>

            <div className="input-group">
              <label className="input-label">المورد / الجهة</label>
              <input type="text" className="input-field" value={vendor} onChange={(e) => setVendor(e.target.value)} placeholder="اختياري" />
            </div>

            <div className="input-group">
              <label className="input-label">المرجع</label>
              <input type="text" className="input-field" value={reference} onChange={(e) => setReference(e.target.value)} placeholder="اختياري" />
            </div>

            <div className="input-group col-span-2">
              <label className="input-label">ملاحظات</label>
              <textarea className="input-field min-h-[80px]" value={notes} onChange={(e) => setNotes(e.target.value)} placeholder="اختياري" />
            </div>
          </div>

          <div className="flex justify-end gap-3 mt-6">
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
