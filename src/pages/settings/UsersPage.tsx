import { useState, useEffect, useMemo, useCallback } from "react";
import { useTranslation } from "react-i18next";
import DataTable, { Column, type BadgeVariant } from "@/components/ui/DataTable";
import Badge from "@/components/ui/Badge";
import Button from "@/components/ui/Button";
import Card from "@/components/ui/Card";
import { formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus, Trash2, Shield, Edit, KeyRound, Power, CheckCircle2, XCircle } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { useAuthStore } from "@/stores/authStore";

interface ManagedUser {
  id: number;
  username: string;
  full_name: string | null;
  role: string;
  active: number;
  created_at: string | null;
}

interface UserForm {
  username: string;
  full_name: string;
  role: string;
  password: string;
  confirmPassword: string;
}

const emptyForm: UserForm = { username: "", full_name: "", role: "viewer", password: "", confirmPassword: "" };

export default function UsersPage() {
  const { t } = useTranslation();
  const addNotification = useUIStore((s) => s.addNotification);
  const currentUser = useAuthStore((s) => s.user);
  const [users, setUsers] = useState<ManagedUser[]>([]);
  const [loading, setLoading] = useState(true);
  const [showModal, setShowModal] = useState(false);
  const [editingUser, setEditingUser] = useState<ManagedUser | null>(null);
  const [saving, setSaving] = useState(false);
  const [form, setForm] = useState<UserForm>(emptyForm);
  const [resetUser, setResetUser] = useState<ManagedUser | null>(null);
  const [resetPassword, setResetPassword] = useState("");
  const [savingReset, setSavingReset] = useState(false);

  const ROLE_OPTIONS = [
    { value: "admin", label: t("users.role.admin") },
    { value: "accountant", label: t("users.role.accountant") },
    { value: "hr", label: t("users.role.hr") },
    { value: "operator", label: t("users.role.operator") },
    { value: "viewer", label: t("users.role.viewer") },
  ];

  const roleMap: Record<string, { label: string; variant: BadgeVariant }> = {
    admin: { label: t("users.role.admin"), variant: "danger" },
    accountant: { label: t("users.role.accountant"), variant: "info" },
    hr: { label: t("users.role.hr"), variant: "success" },
    operator: { label: t("users.role.operator"), variant: "warning" },
    viewer: { label: t("users.role.viewer"), variant: "gold" },
  };

  const fetchUsers = useCallback(() => {
    invoke("list_users")
      .then((d) => setUsers((d as ManagedUser[]) || []))
      .catch((e: unknown) => addNotification({ title: t("common.error"), message: String(e), type: "error" }))
      .finally(() => setLoading(false));
  }, [t, addNotification]);

  useEffect(() => { fetchUsers(); }, [fetchUsers]);

  const notify = (type: "success" | "error", message: string) =>
    addNotification({ id: crypto.randomUUID(), type, title: type === "success" ? t("users.page.success") : t("common.error"), message });

  const openCreate = () => {
    setEditingUser(null);
    setForm(emptyForm);
    setShowModal(true);
  };

  const openEdit = (u: ManagedUser) => {
    setEditingUser(u);
    setForm({ username: u.username, full_name: u.full_name || "", role: u.role, password: "", confirmPassword: "" });
    setShowModal(true);
  };

  const handleSave = async () => {
    if (!form.username.trim()) return notify("error", t("users.page.usernameRequired"));
    if (form.full_name.trim() && form.full_name.trim().length < 3) return notify("error", t("users.page.fullNameMin"));

    setSaving(true);
    try {
      if (editingUser) {
        await invoke("update_user", {
          id: editingUser.id,
          input: { full_name: form.full_name || null, role: form.role },
        });
        notify("success", t("users.page.updateSuccess"));
      } else {
        if (form.password.length < 8) {
          setSaving(false);
          return notify("error", t("users.page.passwordMin"));
        }
        if (form.password !== form.confirmPassword) {
          setSaving(false);
          return notify("error", t("users.page.passwordMismatch"));
        }
        await invoke("create_user", {
          callerId: currentUser?.id,
          input: { username: form.username.trim(), password: form.password, full_name: form.full_name || null, role: form.role },
        });
        notify("success", t("users.page.createSuccess"));
      }
      setShowModal(false);
      fetchUsers();
    } catch (err: unknown) {
      notify("error", err instanceof Error ? err.message : String(err));
    } finally {
      setSaving(false);
    }
  };

  const handleToggleActive = useCallback(async (u: ManagedUser) => {
    if (u.id === currentUser?.id) return notify("error", t("users.page.cannotDisableSelf"));
    try {
      await invoke("update_user", { id: u.id, input: { active: u.active ? 0 : 1 } });
      notify("success", u.active ? t("users.page.userDisabled") : t("users.page.userEnabled"));
      fetchUsers();
    } catch (err: unknown) {
      notify("error", err instanceof Error ? err.message : String(err));
    }
  }, [currentUser?.id, fetchUsers]);

  const handleDelete = useCallback(async (u: ManagedUser) => {
    if (u.id === currentUser?.id) return notify("error", t("users.page.cannotDeleteSelf"));
    if (!confirm(t("users.page.deleteConfirm", { username: u.username }))) return;
    try {
      await invoke("delete_user", { callerId: currentUser?.id, id: u.id });
      notify("success", t("users.page.deleteSuccess"));
      fetchUsers();
    } catch (err: unknown) {
      notify("error", err instanceof Error ? err.message : String(err));
    }
  }, [currentUser?.id, fetchUsers]);

  const handleResetPassword = async () => {
    if (!resetUser) return;
    if (resetPassword.length < 8) return notify("error", t("users.page.passwordMin"));
    setSavingReset(true);
    try {
      await invoke("reset_user_password", { callerId: currentUser?.id, id: resetUser.id, newPassword: resetPassword });
      notify("success", t("users.page.resetSuccess"));
      setResetUser(null);
      setResetPassword("");
    } catch (err: unknown) {
      notify("error", err instanceof Error ? err.message : String(err));
    } finally {
      setSavingReset(false);
    }
  };

  const columns: Column<ManagedUser>[] = useMemo(() => [
    { key: "username", header: t("auth.username"), sortable: true, render: (r) => (
      <span className="font-mono text-brand-400">{r.username}{r.id === currentUser?.id && <span className="mr-2 text-[10px] text-gold-400">({t("users.page.you")})</span>}</span>
    ) },
    { key: "full_name", header: t("users.page.fullName"), sortable: true, render: (r) => <span className="font-medium">{r.full_name || "—"}</span> },
    { key: "role", header: t("users.page.role"), render: (r) => { const role = roleMap[r.role] || { label: r.role, variant: "default" as BadgeVariant }; return <Badge variant={role.variant}>{role.label}</Badge>; } },
    { key: "active", header: t("common.status"), render: (r) => r.active ? <Badge variant="success"><CheckCircle2 className="w-3 h-3 inline-block ml-1" />{t("users.page.active")}</Badge> : <Badge variant="danger"><XCircle className="w-3 h-3 inline-block ml-1" />{t("users.page.disabled")}</Badge> },
    { key: "created_at", header: t("users.page.createdAt"), render: (r) => r.created_at ? formatDate(r.created_at) : <span className="text-surface-500">—</span> },
    { key: "actions", header: "", render: (r) => (
      <div className="flex items-center gap-1">
        <button onClick={() => openEdit(r)} className="p-1.5 text-surface-400 hover:text-brand-400 transition-colors rounded-lg hover:bg-surface-700/50" title={t("common.edit")}>
          <Edit className="w-4 h-4" />
        </button>
        <button onClick={() => handleToggleActive(r)} className="p-1.5 text-surface-400 hover:text-gold-400 transition-colors rounded-lg hover:bg-surface-700/50" title={r.active ? t("users.page.deactivate") : t("users.page.activate")}>
          <Power className="w-4 h-4" />
        </button>
        <button onClick={() => setResetUser(r)} className="p-1.5 text-surface-400 hover:text-blue-400 transition-colors rounded-lg hover:bg-surface-700/50" title={t("users.page.resetPassword")}>
          <KeyRound className="w-4 h-4" />
        </button>
        <button onClick={() => handleDelete(r)} className="p-1.5 text-surface-400 hover:text-red-400 transition-colors rounded-lg hover:bg-surface-700/50" title={t("common.delete")}>
          <Trash2 className="w-4 h-4" />
        </button>
      </div>
    )},
  ], [t, currentUser?.id, handleDelete, handleToggleActive]);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">{t("users.page.title")}</h1>
          <p className="page-subtitle">{t("users.page.subtitle", { count: users.length })}</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={openCreate}>
          {t("users.page.newUser")}
        </Button>
      </div>

      <div className="grid grid-cols-4 gap-4">
        <Card className="text-center">
          <p className="text-2xl font-bold gradient-text">{users.length}</p>
          <p className="text-xs text-surface-400">{t("users.page.totalUsers")}</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-red-400">{users.filter((u) => u.role === "admin").length}</p>
          <p className="text-xs text-surface-400">{t("users.role.admin")}</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-emerald-400">{users.filter((u) => u.active === 1).length}</p>
          <p className="text-xs text-surface-400">{t("users.page.activeUsers")}</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-red-400">{users.filter((u) => u.active === 0).length}</p>
          <p className="text-xs text-surface-400">{t("users.page.disabledUsers")}</p>
        </Card>
      </div>

      <DataTable columns={columns} data={users} loading={loading} emptyMessage={t("users.page.empty")} />

      {showModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
          <div className="w-full max-w-lg bg-surface-900 border border-surface-700 rounded-2xl p-6 shadow-2xl space-y-4">
            <div className="flex items-center justify-between">
              <h2 className="text-lg font-bold text-white">{editingUser ? t("users.page.editUser") : t("users.page.newUser")}</h2>
              <button onClick={() => setShowModal(false)} className="text-surface-400 hover:text-white text-xl">&times;</button>
            </div>

            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="form-label">{t("auth.username")}</label>
                  <input
                    type="text"
                    value={form.username}
                    onChange={(e) => setForm({ ...form, username: e.target.value })}
                    className="input-field"
                    dir="ltr"
                    disabled={!!editingUser}
                    aria-label={t("auth.username")}
                  />
                </div>
                <div>
                  <label className="form-label">{t("users.page.fullName")}</label>
                  <input
                    type="text"
                    value={form.full_name}
                    onChange={(e) => setForm({ ...form, full_name: e.target.value })}
                    className="input-field"
                    aria-label={t("users.page.fullName")}
                  />
                </div>
              </div>
              <div>
                <label className="form-label">{t("users.page.role")}</label>
                <select value={form.role} onChange={(e) => setForm({ ...form, role: e.target.value })} className="input-field" aria-label={t("users.page.role")}>
                  {ROLE_OPTIONS.map((r) => <option key={r.value} value={r.value}>{r.label}</option>)}
                </select>
              </div>
              {!editingUser && (
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="form-label">{t("auth.password")}</label>
                    <input
                      type="password"
                      value={form.password}
                      onChange={(e) => setForm({ ...form, password: e.target.value })}
                      className="input-field"
                      dir="ltr"
                      placeholder={t("users.page.passwordMinHint")}
                      aria-label={t("auth.password")}
                    />
                  </div>
                  <div>
                    <label className="form-label">{t("auth.confirmPassword")}</label>
                    <input
                      type="password"
                      value={form.confirmPassword}
                      onChange={(e) => setForm({ ...form, confirmPassword: e.target.value })}
                      className="input-field"
                      dir="ltr"
                      aria-label={t("auth.confirmPassword")}
                    />
                  </div>
                </div>
              )}
            </div>

            <div className="flex justify-end gap-3 pt-2">
              <Button variant="ghost" onClick={() => setShowModal(false)}>{t("common.cancel")}</Button>
              <Button icon={<Shield className="w-4 h-4" />} onClick={handleSave} loading={saving}>
                {editingUser ? t("users.page.saveChanges") : t("common.create")}
              </Button>
            </div>
          </div>
        </div>
      )}

      {resetUser && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
          <div className="w-full max-w-md bg-surface-900 border border-surface-700 rounded-2xl p-6 shadow-2xl space-y-4">
            <div className="flex items-center justify-between">
              <h2 className="text-lg font-bold text-white">{t("users.page.resetPassword")}</h2>
              <button onClick={() => { setResetUser(null); setResetPassword(""); }} className="text-surface-400 hover:text-white text-xl">&times;</button>
            </div>
            <div>
              <p className="text-sm text-surface-400 mb-4">
                {t("users.page.userLabel")}: <span className="font-bold text-white" dir="ltr">{resetUser.username}</span>
              </p>
              <label className="form-label">{t("auth.newPassword")}</label>
              <input
                type="password"
                value={resetPassword}
                onChange={(e) => setResetPassword(e.target.value)}
                className="input-field"
                dir="ltr"
                placeholder={t("users.page.passwordMinHint")}
                aria-label={t("auth.newPassword")}
              />
              <p className="text-xs text-surface-500 mt-2">{t("users.page.mustChangePassword")}</p>
            </div>
            <div className="flex justify-end gap-3 pt-2">
              <Button variant="ghost" onClick={() => { setResetUser(null); setResetPassword(""); }}>{t("common.cancel")}</Button>
              <Button icon={<KeyRound className="w-4 h-4" />} onClick={handleResetPassword} loading={savingReset}>{t("users.page.reset")}</Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
