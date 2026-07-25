import { useState, useEffect, useMemo, useCallback } from "react";
import DataTable, { Column, type BadgeVariant } from "@/components/ui/DataTable";
import Badge from "@/components/ui/Badge";
import Button from "@/components/ui/Button";
import Card from "@/components/ui/Card";
import { formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus, Trash2, Shield, Edit } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { User } from "@/types";

export default function UsersPage() {
  const addNotification = useUIStore((s) => s.addNotification);
  const [users, setUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(true);
  const [showModal, setShowModal] = useState(false);
  const [saving, setSaving] = useState(false);
  const [form, setForm] = useState({ username: "", full_name: "", email: "", role: "viewer", password: "" });

  const fetchUsers = useCallback(() => {
    invoke("list_users")
      .then((d) => setUsers(d as User[]))
      .catch((e: unknown) => addNotification({ title: 'خطأ', message: String(e), type: 'error' }))
      .finally(() => setLoading(false));
  }, [addNotification]);

  useEffect(() => { fetchUsers(); }, [fetchUsers]);

  const roleMap: Record<string, { label: string; variant: BadgeVariant }> = {
    admin: { label: "مدير", variant: "danger" },
    accountant: { label: "محاسب", variant: "info" },
    hr: { label: "موارد بشرية", variant: "success" },
    operator: { label: "مشغل", variant: "warning" },
    viewer: { label: "مشاهد", variant: "gold" },
  };

  const handleCreate = async () => {
    setSaving(true);
    try {
      await invoke("create_user", { user: form });
      setShowModal(false);
      setForm({ username: "", full_name: "", email: "", role: "viewer", password: "" });
      fetchUsers();
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء الحفظ" });
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = useCallback(async (id: number) => {
    if (!confirm('هل أنت متأكد من حذف هذا المستخدم؟')) return;
    try {
      await invoke('delete_user', { id });
      fetchUsers();
    } catch (err: unknown) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) });
    }
  }, [fetchUsers]);

  const columns: Column<any>[] = useMemo(() => [
    { key: "username", header: "اسم المستخدم", sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.username}</span> },
    { key: "full_name", header: "الاسم الكامل", sortable: true, render: (r) => <span className="font-medium">{r.full_name}</span> },
    { key: "email", header: "البريد الإلكتروني", render: (r) => <span className="text-surface-400" dir="ltr">{r.email || "—"}</span> },
    { key: "role", header: "الصلاحية", render: (r) => { const role = roleMap[r.role] || { label: r.role, variant: "default" as BadgeVariant }; return <Badge variant={role.variant}>{role.label}</Badge>; } },
    { key: "last_login", header: "آخر دخول", render: (r) => r.last_login ? formatDate(r.last_login) : <span className="text-surface-500">لم يسجل دخول</span> },
    { key: "actions", header: "", render: (r) => (
      <div className="flex items-center gap-1">
        <button className="p-1.5 text-surface-400 hover:text-brand-400 transition-colors rounded-lg hover:bg-surface-700/50">
          <Edit className="w-4 h-4" />
        </button>
        <button onClick={() => handleDelete(r.id)} className="p-1.5 text-surface-400 hover:text-red-400 transition-colors rounded-lg hover:bg-surface-700/50">
          <Trash2 className="w-4 h-4" />
        </button>
      </div>
    )},
  ], [handleDelete, roleMap]);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">إدارة المستخدمين</h1>
          <p className="page-subtitle">{users.length} مستخدم</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => setShowModal(true)}>
          مستخدم جديد
        </Button>
      </div>

      <div className="grid grid-cols-4 gap-4">
        <Card className="text-center">
          <p className="text-2xl font-bold gradient-text">{users.length}</p>
          <p className="text-xs text-surface-400">إجمالي المستخدمين</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-red-400">{users.filter((u) => u.role === "admin").length}</p>
          <p className="text-xs text-surface-400">مدير</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-blue-400">{users.filter((u) => u.role === "accountant").length}</p>
          <p className="text-xs text-surface-400">محاسب</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-emerald-400">{users.filter((u) => u.role === "hr").length}</p>
          <p className="text-xs text-surface-400">موارد بشرية</p>
        </Card>
      </div>

      <DataTable columns={columns} data={users} loading={loading} emptyMessage="لا يوجد مستخدمين" />

      {showModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
          <div className="w-full max-w-lg bg-surface-900 border border-surface-700 rounded-2xl p-6 shadow-2xl space-y-4">
            <div className="flex items-center justify-between">
              <h2 className="text-lg font-bold text-white">مستخدم جديد</h2>
              <button onClick={() => setShowModal(false)} className="text-surface-400 hover:text-white text-xl">&times;</button>
            </div>

            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="form-label">اسم المستخدم</label>
                  <input type="text" value={form.username} onChange={(e) => setForm({ ...form, username: e.target.value })} className="input-field" dir="ltr" />
                </div>
                <div>
                  <label className="form-label">الاسم الكامل</label>
                  <input type="text" value={form.full_name} onChange={(e) => setForm({ ...form, full_name: e.target.value })} className="input-field" />
                </div>
              </div>
              <div>
                <label className="form-label">البريد الإلكتروني</label>
                <input type="email" value={form.email} onChange={(e) => setForm({ ...form, email: e.target.value })} className="input-field" dir="ltr" />
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="form-label">الصلاحية</label>
                  <select value={form.role} onChange={(e) => setForm({ ...form, role: e.target.value })} className="input-field">
                    <option value="admin">مدير</option>
                    <option value="accountant">محاسب</option>
                    <option value="hr">موارد بشرية</option>
                    <option value="operator">مشغل</option>
                    <option value="viewer">مشاهد</option>
                  </select>
                </div>
                <div>
                  <label className="form-label">كلمة المرور</label>
                  <input type="password" value={form.password} onChange={(e) => setForm({ ...form, password: e.target.value })} className="input-field" dir="ltr" />
                </div>
              </div>
            </div>

            <div className="flex justify-end gap-3 pt-2">
              <Button variant="ghost" onClick={() => setShowModal(false)}>إلغاء</Button>
              <Button icon={<Shield className="w-4 h-4" />} onClick={handleCreate} loading={saving}>إنشاء</Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
