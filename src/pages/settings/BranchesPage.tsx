import { useState, useEffect, useCallback } from "react";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { Input, Textarea, Select } from "@/components/ui/Input";
import Modal from "@/components/ui/Modal";
import { formatDateTime, cn } from "@/lib/utils";
import { invoke } from "@/lib/tauri";
import { useUIStore } from "@/stores/uiStore";
import {
  Building2, Plus, Pencil, Trash2, RefreshCw, Loader2,
  Database, Send, RotateCcw, CheckCircle2, AlertTriangle,
  Landmark
} from "lucide-react";

interface Branch {
  id: number;
  name: string;
  code: string | null;
  address: string | null;
  is_head_office: boolean;
  is_active: boolean;
  created_at: string;
}

interface OfflineQueueItem {
  id: number;
  branch_id: number | null;
  branch_name: string | null;
  operation: string;
  entity: string;
  entity_id: number | null;
  payload: string;
  status: string;
  created_at: string;
  synced_at: string | null;
}

interface QueueStats {
  pending: number;
  synced: number;
  failed: number;
}

const queueStatusMeta: Record<string, { label: string; cls: string }> = {
  pending: { label: "بانتظار المزامنة", cls: "badge-warning" },
  synced: { label: "تمت المزامنة", cls: "badge-success" },
  failed: { label: "فشل", cls: "badge-danger" },
};

const emptyForm = { id: 0, name: "", code: "", address: "" };

export default function BranchesPage() {
  const addNotification = useUIStore((s) => s.addNotification);
  const [branches, setBranches] = useState<Branch[]>([]);
  const [queue, setQueue] = useState<OfflineQueueItem[]>([]);
  const [stats, setStats] = useState<QueueStats>({ pending: 0, synced: 0, failed: 0 });
  const [queueFilter, setQueueFilter] = useState("");
  const [loading, setLoading] = useState(true);
  const [form, setForm] = useState(emptyForm);
  const [modalOpen, setModalOpen] = useState(false);
  const [saving, setSaving] = useState(false);
  const [deletingId, setDeletingId] = useState<number | null>(null);
  const [busyItemId, setBusyItemId] = useState<number | null>(null);
  const [expandedPayload, setExpandedPayload] = useState<OfflineQueueItem | null>(null);

  const load = useCallback(() => {
    setLoading(true);
    Promise.all([
      invoke<Branch[]>("branches_list"),
      invoke<OfflineQueueItem[]>("offline_queue_list", { status: queueFilter || null }),
      invoke<QueueStats>("offline_queue_stats"),
    ])
      .then(([b, q, s]) => {
        setBranches(b);
        setQueue(q);
        setStats(s);
      })
      .catch((e: unknown) => addNotification({ title: "خطأ", message: String(e), type: "error" }))
      .finally(() => setLoading(false));
  }, [addNotification, queueFilter]);

  useEffect(() => { load(); }, [load]);

  const openCreate = () => {
    setForm(emptyForm);
    setModalOpen(true);
  };

  const openEdit = (b: Branch) => {
    setForm({ id: b.id, name: b.name, code: b.code || "", address: b.address || "" });
    setModalOpen(true);
  };

  const handleSave = async () => {
    if (!form.name.trim()) {
      addNotification({ title: "خطأ", message: "اسم الفرع مطلوب", type: "error" });
      return;
    }
    setSaving(true);
    try {
      if (form.id === 0) {
        await invoke<number>("branches_create", { name: form.name, code: form.code || null, address: form.address || null });
        addNotification({ title: "تم", message: "تم إضافة الفرع", type: "success" });
      } else {
        await invoke<string>("branches_update", {
          id: form.id,
          name: form.name,
          code: form.code || null,
          address: form.address || null,
          isActive: true,
        });
        addNotification({ title: "تم", message: "تم تحديث الفرع", type: "success" });
      }
      setModalOpen(false);
      load();
    } catch (e) {
      addNotification({ title: "خطأ", message: String(e), type: "error" });
    } finally {
      setSaving(false);
    }
  };

  const handleToggleActive = async (b: Branch) => {
    try {
      await invoke<string>("branches_update", {
        id: b.id,
        name: b.name,
        code: b.code,
        address: b.address,
        isActive: !b.is_active,
      });
      load();
    } catch (e) {
      addNotification({ title: "خطأ", message: String(e), type: "error" });
    }
  };

  const handleDelete = async (b: Branch) => {
    setDeletingId(b.id);
    try {
      await invoke<string>("branches_delete", { id: b.id });
      addNotification({ title: "تم", message: "تم حذف الفرع", type: "success" });
      load();
    } catch (e) {
      addNotification({ title: "خطأ", message: String(e), type: "error" });
    } finally {
      setDeletingId(null);
    }
  };

  const handleMarkSynced = async (id: number) => {
    setBusyItemId(id);
    try {
      await invoke<string>("offline_queue_mark_synced", { id });
      addNotification({ title: "تم", message: "تم تأكيد المزامنة", type: "success" });
      load();
    } catch (e) {
      addNotification({ title: "خطأ", message: String(e), type: "error" });
    } finally {
      setBusyItemId(null);
    }
  };

  const handleRetry = async (id: number) => {
    setBusyItemId(id);
    try {
      await invoke<string>("offline_queue_retry", { id });
      addNotification({ title: "تم", message: "أعيدت العملية لقائمة الانتظار", type: "success" });
      load();
    } catch (e) {
      addNotification({ title: "خطأ", message: String(e), type: "error" });
    } finally {
      setBusyItemId(null);
    }
  };

  const qbadge = (s: string) => {
    const m = queueStatusMeta[s] || { label: s, cls: "badge-info" };
    return <span className={m.cls}>{m.label}</span>;
  };

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title flex items-center gap-2">
            <Building2 className="w-6 h-6 text-gold-400" />
            الفروع والمزامنة دون اتصال
          </h1>
          <p className="page-subtitle">إدارة الفروع المتعددة وقائمة المزامنة للمعاملات التي تمت أثناء انقطاع الاتصال</p>
        </div>
        <Button onClick={openCreate} icon={<Plus className="w-4 h-4" />}>
          إضافة فرع
        </Button>
      </div>

      <div className="grid grid-cols-3 gap-4">
        <div className="p-4 bg-surface-800/50 rounded-xl flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-amber-500/15 flex items-center justify-center">
            <Database className="w-5 h-5 text-amber-400" />
          </div>
          <div>
            <p className="text-2xl font-bold text-white">{stats.pending}</p>
            <p className="text-xs text-surface-500">بانتظار المزامنة</p>
          </div>
        </div>
        <div className="p-4 bg-surface-800/50 rounded-xl flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-emerald-500/15 flex items-center justify-center">
            <CheckCircle2 className="w-5 h-5 text-emerald-400" />
          </div>
          <div>
            <p className="text-2xl font-bold text-white">{stats.synced}</p>
            <p className="text-xs text-surface-500">تمت المزامنة</p>
          </div>
        </div>
        <div className="p-4 bg-surface-800/50 rounded-xl flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-red-500/15 flex items-center justify-center">
            <AlertTriangle className="w-5 h-5 text-red-400" />
          </div>
          <div>
            <p className="text-2xl font-bold text-white">{stats.failed}</p>
            <p className="text-xs text-surface-500">فشلت</p>
          </div>
        </div>
      </div>

      <Card>
        <h3 className="section-title mb-4">الفروع</h3>
        {loading ? (
          <div className="flex justify-center py-10">
            <Loader2 className="w-8 h-8 animate-spin text-gold-400" />
          </div>
        ) : branches.length === 0 ? (
          <p className="text-center text-surface-500 py-8">لا توجد فروع. أضف فرعاً أولاً.</p>
        ) : (
          <div className="overflow-x-auto">
            <table className="table w-full text-sm">
              <thead>
                <tr className="text-xs text-surface-500">
                  <th className="text-right">الفرع</th>
                  <th className="text-right">الرمز</th>
                  <th className="text-right">العنوان</th>
                  <th className="text-right">النوع</th>
                  <th className="text-right">الحالة</th>
                  <th className="text-right">إجراءات</th>
                </tr>
              </thead>
              <tbody>
                {branches.map((b) => (
                  <tr key={b.id} className="border-t border-surface-800">
                    <td className="py-2.5 font-medium text-white">
                      <div className="flex items-center gap-2">
                        {b.is_head_office && <Landmark className="w-4 h-4 text-gold-400" />}
                        {b.name}
                      </div>
                    </td>
                    <td className="font-mono text-xs" dir="ltr">{b.code || "—"}</td>
                    <td className="text-surface-400 text-xs max-w-[220px] truncate">{b.address || "—"}</td>
                    <td>
                      {b.is_head_office
                        ? <span className="badge-success">الفرع الرئيسي</span>
                        : <span className="badge-info">فرع</span>}
                    </td>
                    <td>
                      <button
                        onClick={() => handleToggleActive(b)}
                        disabled={b.is_head_office}
                        className={cn(
                          "text-xs px-3 py-1 rounded-full border transition-colors",
                          b.is_active
                            ? "bg-emerald-500/10 text-emerald-400 border-emerald-500/30 hover:bg-emerald-500/20"
                            : "bg-surface-800 text-surface-500 border-surface-700 hover:bg-surface-700",
                          b.is_head_office && "opacity-60 cursor-not-allowed"
                        )}
                      >
                        {b.is_active ? "نشط" : "معطل"}
                      </button>
                    </td>
                    <td>
                      <div className="flex items-center gap-1.5">
                        <Button variant="ghost" size="sm" onClick={() => openEdit(b)} icon={<Pencil className="w-4 h-4" />}>
                          تعديل
                        </Button>
                        <Button
                          variant="danger"
                          size="sm"
                          onClick={() => handleDelete(b)}
                          loading={deletingId === b.id}
                          disabled={b.is_head_office}
                          icon={<Trash2 className="w-4 h-4" />}
                        >
                          حذف
                        </Button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </Card>

      <Card>
        <div className="flex items-center justify-between mb-4">
          <h3 className="section-title">قائمة المزامنة (Offline Sync Queue)</h3>
          <div className="flex items-center gap-2">
            <Select
              className="w-48"
              placeholder="كل الحالات"
              options={[
                { value: "pending", label: "بانتظار المزامنة" },
                { value: "synced", label: "تمت المزامنة" },
                { value: "failed", label: "فشل" },
              ]}
              value={queueFilter}
              onChange={(e) => setQueueFilter(e.target.value)}
            />
            <Button variant="ghost" size="sm" onClick={load} icon={<RefreshCw className="w-4 h-4" />}>
              تحديث
            </Button>
          </div>
        </div>
        {queue.length === 0 ? (
          <p className="text-center text-surface-500 py-8">قائمة الانتظار فارغة.</p>
        ) : (
          <div className="overflow-x-auto">
            <table className="table w-full text-sm">
              <thead>
                <tr className="text-xs text-surface-500">
                  <th className="text-right">#</th>
                  <th className="text-right">الفرع</th>
                  <th className="text-right">العملية</th>
                  <th className="text-right">الكيان</th>
                  <th className="text-right">المعرف</th>
                  <th className="text-right">الحالة</th>
                  <th className="text-right">تاريخ الإنشاء</th>
                  <th className="text-right">إجراءات</th>
                </tr>
              </thead>
              <tbody>
                {queue.map((q) => (
                  <tr key={q.id} className="border-t border-surface-800">
                    <td className="font-mono text-xs">{q.id}</td>
                    <td className="text-xs">{q.branch_name || "—"}</td>
                    <td className="font-mono text-xs" dir="ltr">{q.operation}</td>
                    <td className="text-xs">{q.entity}</td>
                    <td className="font-mono text-xs" dir="ltr">{q.entity_id ?? "—"}</td>
                    <td>{qbadge(q.status)}</td>
                    <td className="text-xs text-surface-400">{formatDateTime(q.created_at)}</td>
                    <td>
                      <div className="flex items-center gap-1.5">
                        <Button variant="ghost" size="sm" onClick={() => setExpandedPayload(q)}>
                          بيانات
                        </Button>
                        {q.status === "failed" && (
                          <Button variant="outline" size="sm" onClick={() => handleRetry(q.id)} loading={busyItemId === q.id} icon={<RotateCcw className="w-4 h-4" />}>
                            إعادة
                          </Button>
                        )}
                        {q.status !== "synced" && (
                          <Button variant="success" size="sm" onClick={() => handleMarkSynced(q.id)} loading={busyItemId === q.id} icon={<Send className="w-4 h-4" />}>
                            مزامنة
                          </Button>
                        )}
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </Card>

      <Modal
        open={modalOpen}
        onClose={() => setModalOpen(false)}
        title={form.id === 0 ? "إضافة فرع" : "تعديل الفرع"}
        footer={
          <>
            <Button variant="ghost" onClick={() => setModalOpen(false)}>إلغاء</Button>
            <Button onClick={handleSave} loading={saving} icon={<Plus className="w-4 h-4" />}>
              {form.id === 0 ? "إضافة" : "حفظ"}
            </Button>
          </>
        }
      >
        <div className="space-y-4">
          <Input label="اسم الفرع *" value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} />
          <Input label="الرمز" value={form.code} onChange={(e) => setForm({ ...form, code: e.target.value })} />
          <Textarea label="العنوان" value={form.address} onChange={(e) => setForm({ ...form, address: e.target.value })} />
        </div>
      </Modal>

      <Modal
        open={!!expandedPayload}
        onClose={() => setExpandedPayload(null)}
        title="تفاصيل عنصر المزامنة"
        footer={<Button variant="ghost" onClick={() => setExpandedPayload(null)}>إغلاق</Button>}
      >
        {expandedPayload && (
          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-3">
              <div className="p-3 bg-surface-800/50 rounded-xl">
                <p className="text-[10px] text-surface-500">العملية</p>
                <p className="text-sm font-mono text-white mt-1" dir="ltr">{expandedPayload.operation}</p>
              </div>
              <div className="p-3 bg-surface-800/50 rounded-xl">
                <p className="text-[10px] text-surface-500">الكيان</p>
                <p className="text-sm text-white mt-1">{expandedPayload.entity} #{expandedPayload.entity_id ?? ""}</p>
              </div>
            </div>
            <div>
              <p className="text-xs text-surface-500 mb-2">الحمولة (JSON)</p>
              <Textarea readOnly value={expandedPayload.payload} className="min-h-[240px] font-mono text-xs" />
            </div>
          </div>
        )}
      </Modal>
    </div>
  );
}
