import { useState, useEffect, useMemo, useCallback } from "react";
import DataTable, { Column } from "@/components/ui/DataTable";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import OvertimeBadge from "@/components/ui/OvertimeBadge";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus, Clock, DollarSign, CheckCircle, XCircle } from "lucide-react";
import ConfirmDialog from "@/components/ui/ConfirmDialog";
import { useUIStore } from "@/stores/uiStore";
import type { Employee } from "@/types";

interface OvertimeRecord {
  id: number;
  employee_id: number;
  employee_name: string;
  date: string;
  hours: number;
  rate_multiplier: number;
  reason: string;
  notes: string;
  status: string;
}

export default function OvertimePage() {
  const addNotification = useUIStore((s) => s.addNotification);
  const [records, setRecords] = useState<OvertimeRecord[]>([]);
  const [employees, setEmployees] = useState<Employee[]>([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [saving, setSaving] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);
  const [pendingAction, setPendingAction] = useState<() => void>(() => {});
  const [confirmTitle, setConfirmTitle] = useState("");
  const [confirmMessage, setConfirmMessage] = useState("");
  const [confirmVariant, setConfirmVariant] = useState<'danger' | 'warning' | 'info'>('danger');
  const [form, setForm] = useState({
    employee_id: "",
    date: new Date().toISOString().split("T")[0],
    hours: "",
    rate_multiplier: "1.5",
    reason: "",
    notes: "",
  });

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const [otData, empData] = await Promise.all([
        invoke("list_overtime_records"),
        invoke("list_employees"),
      ]);
      setRecords(otData as OvertimeRecord[]);
      setEmployees(empData as Employee[]);
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء تحميل البيانات" }); }
    finally { setLoading(false); }
  }, [addNotification]);

  useEffect(() => { loadData(); }, [loadData]);

  const handleCreate = async () => {
    setSaving(true);
    try {
      await invoke("create_overtime_record", {
        input: {
          employee_id: Number(form.employee_id),
          date: form.date,
          hours: Number(form.hours),
          rate_multiplier: Number(form.rate_multiplier),
          reason: form.reason,
          notes: form.notes,
        },
      });
      setShowForm(false);
      setForm({ employee_id: "", date: new Date().toISOString().split("T")[0], hours: "", rate_multiplier: "1.5", reason: "", notes: "" });
      loadData();
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء الحفظ" }); }
    finally { setSaving(false); }
  };

  const handleApprove = async (id: number) => {
    try {
      await invoke("approve_overtime", { id, input: { approved_by: "admin" } });
      loadData();
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء الحفظ" }); }
  };

  const handleReject = async (id: number) => {
    try {
      await invoke("reject_overtime", { id });
      loadData();
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء الحفظ" }); }
  };

  const totalHoursThisMonth = records
    .filter((r) => {
      const d = new Date(r.date);
      const now = new Date();
      return d.getMonth() === now.getMonth() && d.getFullYear() === now.getFullYear();
    })
    .reduce((s, r) => s + (r.hours || 0), 0);

  const totalCost = records
    .filter((r) => {
      const d = new Date(r.date);
      const now = new Date();
      return d.getMonth() === now.getMonth() && d.getFullYear() === now.getFullYear();
    })
    .reduce((s, r) => s + (r.hours || 0) * (r.rate_multiplier || 1), 0);

  const pendingCount = records.filter((r) => r.status === "pending").length;
  const approvedCount = records.filter((r) => r.status === "approved").length;

  const columns: Column<OvertimeRecord>[] = useMemo(() => [
    { key: "employee_name", header: "الموظف", sortable: true, render: (r) => <span className="font-medium">{r.employee_name}</span> },
    { key: "date", header: "التاريخ", sortable: true, render: (r) => formatDate(r.date) },
    { key: "hours", header: "الساعات", sortable: true, align: "left", render: (r) => <span className="font-bold">{r.hours}</span> },
    { key: "rate_multiplier", header: "المضاعف", sortable: true, align: "left", render: (r) => <span className="text-gold-400">{r.rate_multiplier}x</span> },
    { key: "reason", header: "السبب", render: (r) => r.reason || "—" },
    { key: "status", header: "الحالة", render: (r) => <OvertimeBadge status={r.status} /> },
    { key: "actions", header: "", render: (r) => r.status === "Pending" ? (
      <div className="flex gap-2">
        <Button size="sm" variant="success" onClick={(e) => { e.stopPropagation(); setConfirmTitle("موافقة على Hours إضافية"); setConfirmMessage("هل أنت متأكد من الموافقة على هذه الساعة الإضافية؟"); setConfirmVariant("warning"); setPendingAction(() => () => handleApprove(r.id)); setShowConfirm(true); }}>موافقة</Button>
        <Button size="sm" variant="danger" onClick={(e) => { e.stopPropagation(); setConfirmTitle("رفض Hours إضافية"); setConfirmMessage("هل أنت متأكد من رفض هذه الساعة الإضافية؟"); setConfirmVariant("danger"); setPendingAction(() => () => handleReject(r.id)); setShowConfirm(true); }}>رفض</Button>
      </div>
    ) : null },
  ], []);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">ساعات العمل الإضافية</h1>
          <p className="page-subtitle">{records.length} سجل</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => setShowForm(true)}>ساعة إضافية جديدة</Button>
      </div>

      <div className="grid grid-cols-4 gap-4">
        <Card className="text-center">
          <p className="text-2xl font-bold gradient-text">{totalHoursThisMonth}</p>
          <p className="text-xs text-surface-400">ساعات إضافية هذا الشهر</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-gold-400">{formatOMR(Math.round(totalCost * 1000))}</p>
          <p className="text-xs text-surface-400">التكلفة التقديرية</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-yellow-400">{pendingCount}</p>
          <p className="text-xs text-surface-400">بانتظار الموافقة</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-emerald-400">{approvedCount}</p>
          <p className="text-xs text-surface-400">تمت الموافقة</p>
        </Card>
      </div>

      {showForm && (
        <Card>
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-bold text-white">ساعة إضافية جديدة</h2>
            <button onClick={() => setShowForm(false)} className="text-surface-400 hover:text-white text-xl">&times;</button>
          </div>
          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="input-group">
                <label className="input-label">الموظف</label>
                <select value={form.employee_id} onChange={(e) => setForm({ ...form, employee_id: e.target.value })} className="input-field" aria-label="الموظف">
                  <option value="">— اختر موظف —</option>
                  {employees.map((emp: any) => (
                    <option key={emp.id} value={emp.id}>{emp.name}</option>
                  ))}
                </select>
              </div>
              <div className="input-group">
                <label className="input-label">التاريخ</label>
                <input type="date" value={form.date} onChange={(e) => setForm({ ...form, date: e.target.value })} className="input-field" aria-label="التاريخ" />
              </div>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="input-group">
                <label className="input-label">عدد الساعات</label>
                <input type="number" min="0.5" step="0.5" value={form.hours} onChange={(e) => setForm({ ...form, hours: e.target.value })} className="input-field" dir="ltr" aria-label="عدد الساعات" />
              </div>
              <div className="input-group">
                <label className="input-label">مضاعف الأجر</label>
                <select value={form.rate_multiplier} onChange={(e) => setForm({ ...form, rate_multiplier: e.target.value })} className="input-field" aria-label="مضاعف الأجر">
                  <option value="1.5">1.5x</option>
                  <option value="2">2x</option>
                  <option value="2.5">2.5x</option>
                  <option value="3">3x</option>
                </select>
              </div>
            </div>
            <div className="input-group">
              <label className="input-label">السبب</label>
              <input type="text" value={form.reason} onChange={(e) => setForm({ ...form, reason: e.target.value })} className="input-field" placeholder="سبب العمل الإضافي..." aria-label="السبب" />
            </div>
            <div className="input-group">
              <label className="input-label">ملاحظات</label>
              <textarea value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} className="input-field" rows={3} placeholder="ملاحظات إضافية..." aria-label="ملاحظات" />
            </div>
          </div>
          <div className="flex justify-end gap-3 mt-4">
            <Button variant="ghost" onClick={() => setShowForm(false)}>إلغاء</Button>
            <Button icon={<Clock className="w-4 h-4" />} onClick={handleCreate} loading={saving}>إنشاء السجل</Button>
          </div>
        </Card>
      )}

      <DataTable columns={columns} data={records} loading={loading} emptyMessage="لا توجد سجلات ساعات إضافية" />

      <ConfirmDialog
        open={showConfirm}
        title={confirmTitle}
        message={confirmMessage}
        variant={confirmVariant}
        onConfirm={() => { pendingAction(); setShowConfirm(false); }}
        onCancel={() => setShowConfirm(false)}
      />
    </div>
  );
}
