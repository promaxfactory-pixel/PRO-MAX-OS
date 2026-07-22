import { useState, useEffect } from "react";
import DataTable, { Column } from "@/components/ui/DataTable";
import Badge from "@/components/ui/Badge";
import Button from "@/components/ui/Button";
import Card from "@/components/ui/Card";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus, HandCoins, Wallet, Lock } from "lucide-react";
import ConfirmDialog from "@/components/ui/ConfirmDialog";

export default function EmployeeAdvancesPage() {
  const [advances, setAdvances] = useState<any[]>([]);
  const [employees, setEmployees] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [saving, setSaving] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);
  const [form, setForm] = useState({
    employee_id: "",
    amount_milli: "",
    date: new Date().toISOString().split("T")[0],
    reason: "",
    deduction_per_payroll_milli: "",
  });

  useEffect(() => { loadData(); }, []);

  const loadData = async () => {
    setLoading(true);
    try {
      const [advData, empData] = await Promise.all([
        invoke("list_employee_advances"),
        invoke("list_employees"),
      ]);
      setAdvances(advData as any[]);
      setEmployees(empData as any[]);
    } catch (err) { console.error(err); }
    finally { setLoading(false); }
  };

  const handleCreate = async () => {
    setSaving(true);
    try {
      await invoke("create_employee_advance", {
        input: {
          employee_id: Number(form.employee_id),
          amount_milli: Number(form.amount_milli),
          date: form.date,
          reason: form.reason,
          deduction_per_payroll_milli: Number(form.deduction_per_payroll_milli),
        },
      });
      setShowForm(false);
      setForm({ employee_id: "", amount_milli: "", date: new Date().toISOString().split("T")[0], reason: "", deduction_per_payroll_milli: "" });
      loadData();
    } catch (err) { console.error(err); }
    finally { setSaving(false); }
  };

  const totalOutstanding = advances.filter((a) => a.status === "open").reduce((s, a) => s + (a.remaining_milli || 0), 0);
  const closedCount = advances.filter((a) => a.status === "closed").length;

  const columns: Column<any>[] = [
    { key: "employee_name", header: "الموظف", sortable: true, render: (r) => <span className="font-medium">{r.employee_name}</span> },
    { key: "amount_milli", header: "المبلغ", sortable: true, align: "left", render: (r) => formatOMR(r.amount_milli) },
    { key: "date", header: "التاريخ", sortable: true, render: (r) => formatDate(r.date) },
    { key: "reason", header: "السبب", render: (r) => r.reason || "—" },
    { key: "remaining_milli", header: "المتبقي", align: "left", render: (r) => <span className="font-bold text-gold-400">{formatOMR(r.remaining_milli)}</span> },
    { key: "status", header: "الحالة", render: (r) => (
      <Badge variant={r.status === "open" ? "warning" : "success"}>
        {r.status === "open" ? "مفتوح" : "مغلق"}
      </Badge>
    )},
  ];

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">سلف الموظفين</h1>
          <p className="page-subtitle">{advances.length} سلفة</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => setShowForm(true)}>سلفة جديدة</Button>
      </div>

      <div className="grid grid-cols-4 gap-4">
        <Card className="text-center">
          <p className="text-2xl font-bold gradient-text">{advances.length}</p>
          <p className="text-xs text-surface-400">إجمالي السلف</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-gold-400">{formatOMR(totalOutstanding)}</p>
          <p className="text-xs text-surface-400">المبلغ المتبقي</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-emerald-400">{closedCount}</p>
          <p className="text-xs text-surface-400">سلف مغلقة</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-yellow-400">{advances.length - closedCount}</p>
          <p className="text-xs text-surface-400">سلف مفتوحة</p>
        </Card>
      </div>

      {showForm && (
        <Card>
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-bold text-white">سلفة جديدة</h2>
            <button onClick={() => setShowForm(false)} className="text-surface-400 hover:text-white text-xl">&times;</button>
          </div>
          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="input-group">
                <label className="input-label">الموظف</label>
                <select value={form.employee_id} onChange={(e) => setForm({ ...form, employee_id: e.target.value })} className="input-field">
                  <option value="">— اختر موظف —</option>
                  {employees.map((emp: any) => (
                    <option key={emp.id} value={emp.id}>{emp.name}</option>
                  ))}
                </select>
              </div>
              <div className="input-group">
                <label className="input-label">المبلغ (مليار)</label>
                <input type="number" value={form.amount_milli} onChange={(e) => setForm({ ...form, amount_milli: e.target.value })} className="input-field" dir="ltr" />
              </div>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="input-group">
                <label className="input-label">التاريخ</label>
                <input type="date" value={form.date} onChange={(e) => setForm({ ...form, date: e.target.value })} className="input-field" />
              </div>
              <div className="input-group">
                <label className="input-label">خصم كل راتب (مليار)</label>
                <input type="number" value={form.deduction_per_payroll_milli} onChange={(e) => setForm({ ...form, deduction_per_payroll_milli: e.target.value })} className="input-field" dir="ltr" />
              </div>
            </div>
            <div className="input-group">
              <label className="input-label">السبب</label>
              <input type="text" value={form.reason} onChange={(e) => setForm({ ...form, reason: e.target.value })} className="input-field" placeholder="سبب السلفة..." />
            </div>
          </div>
          <div className="flex justify-end gap-3 mt-4">
            <Button variant="ghost" onClick={() => setShowForm(false)}>إلغاء</Button>
            <Button icon={<HandCoins className="w-4 h-4" />} onClick={() => setShowConfirm(true)} loading={saving}>إنشاء السلفة</Button>
          </div>
        </Card>
      )}

      <DataTable columns={columns} data={advances} loading={loading} emptyMessage="لا توجد سلف" />

      <ConfirmDialog
        open={showConfirm}
        title="إنشاء سلفة موظف"
        message="هل أنت متأكد من إنشاء سلفة جديدة؟"
        variant="warning"
        onConfirm={() => { handleCreate(); setShowConfirm(false); }}
        onCancel={() => setShowConfirm(false)}
      />
    </div>
  );
}
