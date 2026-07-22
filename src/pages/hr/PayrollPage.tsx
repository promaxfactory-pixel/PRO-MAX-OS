import { useState, useEffect } from "react";
import DataTable, { Column } from "@/components/ui/DataTable";
import Badge from "@/components/ui/Badge";
import Button from "@/components/ui/Button";
import Card from "@/components/ui/Card";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus, DollarSign, Clock, CheckCircle, Play, Eye } from "lucide-react";
import ConfirmDialog from "@/components/ui/ConfirmDialog";
import { useUIStore } from "@/stores/uiStore";

export default function PayrollPage() {
  const addNotification = useUIStore((s) => s.addNotification);
  const [runs, setRuns] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [saving, setSaving] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);
  const [form, setForm] = useState({ period_start: "", period_end: "" });

  useEffect(() => { loadRuns(); }, []);

  const loadRuns = async () => {
    setLoading(true);
    try {
      const d = await invoke("list_payroll_runs");
      setRuns(d as any[]);
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء تحميل البيانات" }); }
    finally { setLoading(false); }
  };

  const handleCreate = async () => {
    setSaving(true);
    try {
      await invoke("create_payroll_run", { input: form });
      setShowForm(false);
      setForm({ period_start: "", period_end: "" });
      loadRuns();
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء الحفظ" }); }
    finally { setSaving(false); }
  };

  const totalPaid = runs.filter((r) => r.status === "paid").length;
  const pendingRuns = runs.filter((r) => r.status === "pending" || r.status === "draft").length;

  const columns: Column<any>[] = [
    { key: "run_no", header: "رقم التشغيلة", sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.run_no || "—"}</span> },
    { key: "period", header: "الفترة", render: (r) => `${formatDate(r.period_start)} — ${formatDate(r.period_end)}` },
    { key: "total_gross_milli", header: "الإجمالي", sortable: true, align: "left", render: (r) => formatOMR(r.total_gross_milli) },
    { key: "total_deductions_milli", header: "الخصومات", align: "left", render: (r) => formatOMR(r.total_deductions_milli) },
    { key: "total_net_milli", header: "الصافي", sortable: true, align: "left", render: (r) => <span className="font-bold text-gold-400">{formatOMR(r.total_net_milli)}</span> },
    { key: "status", header: "الحالة", render: (r) => {
      const map: Record<string, { label: string; variant: any }> = {
        draft: { label: "مسودة", variant: "default" },
        pending: { label: "قيد الانتظار", variant: "warning" },
        processing: { label: "قيد المعالجة", variant: "info" },
        paid: { label: "مدفوع", variant: "success" },
      };
      const s = map[r.status] || { label: r.status, variant: "default" };
      return <Badge variant={s.variant}>{s.label}</Badge>;
    }},
    { key: "actions", header: "", render: (r) => (
      <div className="flex items-center gap-1">
        <button className="p-1.5 text-surface-400 hover:text-brand-400 transition-colors rounded-lg hover:bg-surface-700/50" title="عرض التفاصيل">
          <Eye className="w-4 h-4" />
        </button>
        {(r.status === "draft" || r.status === "pending") && (
          <button className="p-1.5 text-surface-400 hover:text-gold-400 transition-colors rounded-lg hover:bg-surface-700/50" title="معالجة">
            <Play className="w-4 h-4" />
          </button>
        )}
      </div>
    )},
  ];

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">تشغيل الرواتب</h1>
          <p className="page-subtitle">{runs.length} تشغيلة</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => setShowForm(true)}>تشغيل راتب جديد</Button>
      </div>

      <div className="grid grid-cols-4 gap-4">
        <Card className="text-center">
          <p className="text-2xl font-bold gradient-text">{runs.length}</p>
          <p className="text-xs text-surface-400">إجمالي التشغيلات</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-emerald-400">{totalPaid}</p>
          <p className="text-xs text-surface-400">مدفوعة</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-gold-400">{pendingRuns}</p>
          <p className="text-xs text-surface-400">قيد الانتظار</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-brand-400">{runs.filter((r) => r.status === "processing").length}</p>
          <p className="text-xs text-surface-400">قيد المعالجة</p>
        </Card>
      </div>

      {showForm && (
        <Card>
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-bold text-white">تشغيل راتب جديد</h2>
            <button onClick={() => setShowForm(false)} className="text-surface-400 hover:text-white text-xl">&times;</button>
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div className="input-group">
              <label className="input-label">تاريخ البداية</label>
              <input type="date" value={form.period_start} onChange={(e) => setForm({ ...form, period_start: e.target.value })} className="input-field" />
            </div>
            <div className="input-group">
              <label className="input-label">تاريخ النهاية</label>
              <input type="date" value={form.period_end} onChange={(e) => setForm({ ...form, period_end: e.target.value })} className="input-field" />
            </div>
          </div>
          <div className="flex justify-end gap-3 mt-4">
            <Button variant="ghost" onClick={() => setShowForm(false)}>إلغاء</Button>
            <Button icon={<Play className="w-4 h-4" />} onClick={() => setShowConfirm(true)} loading={saving}>إنشاء التشغيلة</Button>
          </div>
        </Card>
      )}

      <DataTable columns={columns} data={runs} loading={loading} emptyMessage="لا توجد تشغيلات رواتب" />

      <ConfirmDialog
        open={showConfirm}
        title="إنشاء تشغيلة رواتب"
        message="هل أنت متأكد من إنشاء تشغيلة رواتب جديدة؟"
        variant="warning"
        onConfirm={() => { handleCreate(); setShowConfirm(false); }}
        onCancel={() => setShowConfirm(false)}
      />
    </div>
  );
}
