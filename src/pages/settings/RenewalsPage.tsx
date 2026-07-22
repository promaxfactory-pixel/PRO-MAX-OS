import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import DataTable, { Column } from "@/components/ui/DataTable";
import Badge from "@/components/ui/Badge";
import Button from "@/components/ui/Button";
import Card from "@/components/ui/Card";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus, FileCheck, AlertTriangle, Clock, ShieldAlert } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

export default function RenewalsPage() {
  const navigate = useNavigate();
  const addNotification = useUIStore((s) => s.addNotification);
  const [renewals, setRenewals] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [saving, setSaving] = useState(false);
  const [form, setForm] = useState({
    name: "",
    category: "",
    authority: "",
    issue_date: "",
    expiry_date: "",
    cost_milli: "",
    responsible: "",
    alert_days: "30",
    notes: "",
  });

  useEffect(() => { loadRenewals(); }, []);

  const loadRenewals = async () => {
    setLoading(true);
    try {
      const d = await invoke("list_renewals");
      setRenewals(d as any[]);
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء تحميل البيانات" }); }
    finally { setLoading(false); }
  };

  const handleCreate = async () => {
    setSaving(true);
    try {
      await invoke("create_renewal", {
        input: {
          name: form.name,
          category: form.category,
          authority: form.authority,
          issue_date: form.issue_date,
          expiry_date: form.expiry_date,
          cost_milli: Number(form.cost_milli),
          responsible: form.responsible,
          alert_days: Number(form.alert_days),
          notes: form.notes,
        },
      });
      setShowForm(false);
      setForm({ name: "", category: "", authority: "", issue_date: "", expiry_date: "", cost_milli: "", responsible: "", alert_days: "30", notes: "" });
      loadRenewals();
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء الحفظ" }); }
    finally { setSaving(false); }
  };

  const getDaysUntilExpiry = (expiryDate: string): number => {
    if (!expiryDate) return Infinity;
    const now = new Date();
    now.setHours(0, 0, 0, 0);
    const exp = new Date(expiryDate);
    exp.setHours(0, 0, 0, 0);
    return Math.ceil((exp.getTime() - now.getTime()) / (1000 * 60 * 60 * 24));
  };

  const getRowBg = (r: any): string => {
    const days = getDaysUntilExpiry(r.expiry_date);
    if (days < 0) return "bg-red-500/10 border-r-2 border-red-500";
    if (days <= (r.alert_days || 30)) return "bg-yellow-500/10 border-r-2 border-yellow-500";
    return "";
  };

  const getStatus = (r: any): string => {
    const days = getDaysUntilExpiry(r.expiry_date);
    if (r.status === "cancelled") return "cancelled";
    if (days < 0) return "expired";
    if (days <= (r.alert_days || 30)) return "expiring";
    return "active";
  };

  const statusMap: Record<string, { label: string; variant: any }> = {
    active: { label: "نشط", variant: "success" },
    expiring: { label: "قرب الانتهاء", variant: "warning" },
    expired: { label: "منتهي", variant: "danger" },
    cancelled: { label: "ملغى", variant: "default" },
  };

  const totalRenewals = renewals.length;
  const activeCount = renewals.filter((r) => getStatus(r) === "active").length;
  const expiringSoon = renewals.filter((r) => getStatus(r) === "expiring").length;
  const expiredCount = renewals.filter((r) => getStatus(r) === "expired").length;

  const columns: Column<any>[] = [
    { key: "name", header: "الاسم", sortable: true, render: (r) => <span className="font-medium">{r.name}</span> },
    { key: "category", header: "الفئة", sortable: true, render: (r) => r.category || "—" },
    { key: "authority", header: "الجهة", sortable: true, render: (r) => r.authority || "—" },
    { key: "expiry_date", header: "تاريخ الانتهاء", sortable: true, render: (r) => {
      const days = getDaysUntilExpiry(r.expiry_date);
      return (
        <div className="flex items-center gap-2">
          <span>{formatDate(r.expiry_date)}</span>
          {days >= 0 && days <= (r.alert_days || 30) && (
            <span className={`text-xs font-medium ${days < 0 ? "text-red-400" : days <= 7 ? "text-red-400" : "text-yellow-400"}`}>
              {days < 0 ? `منتهي منذ ${Math.abs(days)} يوم` : `${days} يوم`}
            </span>
          )}
          {days < 0 && <ShieldAlert className="w-3.5 h-3.5 text-red-400" />}
        </div>
      );
    }},
    { key: "responsible", header: "المسؤول", render: (r) => r.responsible || "—" },
    { key: "status", header: "الحالة", render: (r) => {
      const s = statusMap[getStatus(r)] || { label: r.status, variant: "default" };
      return <Badge variant={s.variant}>{s.label}</Badge>;
    }},
  ];

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">التجديدات والترخيص</h1>
          <p className="page-subtitle">{totalRenewals} تجديد</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => setShowForm(true)}>تجديد جديد</Button>
      </div>

      <div className="grid grid-cols-4 gap-4">
        <Card className="text-center">
          <p className="text-2xl font-bold gradient-text">{totalRenewals}</p>
          <p className="text-xs text-surface-400">إجمالي التجديدات</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-emerald-400">{activeCount}</p>
          <p className="text-xs text-surface-400">نشط</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-yellow-400">{expiringSoon}</p>
          <p className="text-xs text-surface-400">قرب الانتهاء</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-red-400">{expiredCount}</p>
          <p className="text-xs text-surface-400">منتهي</p>
        </Card>
      </div>

      {showForm && (
        <Card>
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-bold text-white">تجديد جديد</h2>
            <button onClick={() => setShowForm(false)} className="text-surface-400 hover:text-white text-xl">&times;</button>
          </div>
          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="input-group">
                <label className="input-label">الاسم</label>
                <input type="text" value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} className="input-field" />
              </div>
              <div className="input-group">
                <label className="input-label">الفئة</label>
                <input type="text" value={form.category} onChange={(e) => setForm({ ...form, category: e.target.value })} className="input-field" placeholder="مثال: ترخيص، تأمين..." />
              </div>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="input-group">
                <label className="input-label">الجهة</label>
                <input type="text" value={form.authority} onChange={(e) => setForm({ ...form, authority: e.target.value })} className="input-field" />
              </div>
              <div className="input-group">
                <label className="input-label">المسؤول</label>
                <input type="text" value={form.responsible} onChange={(e) => setForm({ ...form, responsible: e.target.value })} className="input-field" />
              </div>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="input-group">
                <label className="input-label">تاريخ الإصدار</label>
                <input type="date" value={form.issue_date} onChange={(e) => setForm({ ...form, issue_date: e.target.value })} className="input-field" />
              </div>
              <div className="input-group">
                <label className="input-label">تاريخ الانتهاء</label>
                <input type="date" value={form.expiry_date} onChange={(e) => setForm({ ...form, expiry_date: e.target.value })} className="input-field" />
              </div>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="input-group">
                <label className="input-label">التكلفة (مليار)</label>
                <input type="number" value={form.cost_milli} onChange={(e) => setForm({ ...form, cost_milli: e.target.value })} className="input-field" dir="ltr" />
              </div>
              <div className="input-group">
                <label className="input-label">أيام التنبيه قبل الانتهاء</label>
                <input type="number" value={form.alert_days} onChange={(e) => setForm({ ...form, alert_days: e.target.value })} className="input-field" dir="ltr" />
              </div>
            </div>
            <div className="input-group">
              <label className="input-label">ملاحظات</label>
              <input type="text" value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} className="input-field" />
            </div>
          </div>
          <div className="flex justify-end gap-3 mt-4">
            <Button variant="ghost" onClick={() => setShowForm(false)}>إلغاء</Button>
            <Button icon={<FileCheck className="w-4 h-4" />} onClick={handleCreate} loading={saving}>إنشاء التجديد</Button>
          </div>
        </Card>
      )}

      <DataTable
        columns={columns}
        data={renewals}
        loading={loading}
        emptyMessage="لا توجد تجديدات"
        onRowClick={(r) => navigate(`/renewals/${r.id}`)}
      />
    </div>
  );
}
