import { useState, useEffect, useCallback } from "react";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { formatDate, cn } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import {
  Database, Download, Upload, Trash2, CheckCircle2,
  AlertTriangle, RefreshCw
} from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

interface Backup { id: number; file_name: string; size: string; created_at: string; type: string; }

export default function BackupPage() {
  const addNotification = useUIStore((s) => s.addNotification);
  const [backups, setBackups] = useState<Backup[]>([]);
  const [loading, setLoading] = useState(true);
  const [creating, setCreating] = useState(false);
  const [restoring, setRestoring] = useState<number | null>(null);
  const [confirmRestore, setConfirmRestore] = useState<number | null>(null);
  const [message, setMessage] = useState<{ type: "success" | "error"; text: string } | null>(null);

  const loadBackups = useCallback(() => {
    setLoading(true);
    invoke<Backup[]>("backup_list")
      .then(setBackups)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => { loadBackups(); }, [loadBackups]);

  const handleCreate = async () => {
    setCreating(true);
    setMessage(null);
    try {
      await invoke("backup_create");
      setMessage({ type: "success", text: "تم إنشاء النسخة الاحتياطية بنجاح" });
      loadBackups();
    } catch (err) {
      setMessage({ type: "error", text: "فشل إنشاء النسخة الاحتياطية" });
      console.error(err);
    } finally {
      setCreating(false);
      setTimeout(() => setMessage(null), 4000);
    }
  };

  const handleRestore = async (id: number) => {
    setRestoring(id);
    setMessage(null);
    try {
      await invoke("backup_restore", { backupId: id });
      setMessage({ type: "success", text: "تم الاستعادة بنجاح" });
    } catch (err) {
      setMessage({ type: "error", text: "فشل الاستعادة" });
      console.error(err);
    } finally {
      setRestoring(null);
      setConfirmRestore(null);
      setTimeout(() => setMessage(null), 4000);
    }
  };

  const handleExport = async (exportType: string) => {
    try {
      await invoke("backup_export_csv", { exportType });
      setMessage({ type: "success", text: "تم التصدير بنجاح" });
      setTimeout(() => setMessage(null), 4000);
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء التصدير" });
    }
  };

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title flex items-center gap-2">
            <Database className="w-6 h-6 text-gold-400" />
            النسخ الاحتياطي
          </h1>
          <p className="page-subtitle">إنشاء واستعادة النسخ الاحتياطية</p>
        </div>
        <Button icon={<Download className="w-4 h-4" />} onClick={handleCreate} loading={creating}>
          إنشاء نسخة احتياطية
        </Button>
      </div>

      {message && (
        <div className={cn("flex items-center gap-2 p-3 rounded-xl text-sm",
          message.type === "success" ? "bg-emerald-500/10 text-emerald-400 border border-emerald-500/30" : "bg-red-500/10 text-red-400 border border-red-500/30"
        )}>
          {message.type === "success" ? <CheckCircle2 className="w-4 h-4" /> : <AlertTriangle className="w-4 h-4" />}
          {message.text}
        </div>
      )}

      <div className="grid grid-cols-3 gap-6">
        <div className="col-span-2">
          <Card>
            <h3 className="section-title mb-4">النسخ الاحتياطية</h3>
            {loading ? (
              <div className="flex justify-center py-8">
                <div className="w-8 h-8 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" />
              </div>
            ) : backups.length === 0 ? (
              <p className="text-center text-surface-500 py-8">لا توجد نسخ احتياطية</p>
            ) : (
              <div className="space-y-3">
                {backups.map((b) => (
                  <div key={b.id} className="flex items-center justify-between p-4 bg-surface-800/50 rounded-xl">
                    <div className="flex items-center gap-4">
                      <div className="w-10 h-10 rounded-lg bg-brand-800/30 flex items-center justify-center">
                        <Database className="w-5 h-5 text-brand-400" />
                      </div>
                      <div>
                        <p className="text-sm font-medium text-surface-200">{b.file_name}</p>
                        <div className="flex items-center gap-3 text-xs text-surface-500 mt-0.5">
                          <span>{b.size}</span>
                          <span>{b.type}</span>
                          <span>{formatDate(b.created_at)}</span>
                        </div>
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      {confirmRestore === b.id ? (
                        <>
                          <span className="text-xs text-amber-400">تأكيد؟</span>
                          <Button variant="ghost" size="sm" onClick={() => handleRestore(b.id)} loading={restoring === b.id}>
                            نعم
                          </Button>
                          <Button variant="ghost" size="sm" onClick={() => setConfirmRestore(null)}>إلغاء</Button>
                        </>
                      ) : (
                        <Button
                          variant="ghost"
                          size="sm"
                          icon={<Upload className="w-3.5 h-3.5" />}
                          onClick={() => setConfirmRestore(b.id)}
                        >
                          استعادة
                        </Button>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </Card>
        </div>

        <Card>
          <h3 className="section-title mb-4">تصدير CSV</h3>
          <p className="text-sm text-surface-400 mb-4">تصدير البيانات كملفات CSV</p>
          <div className="space-y-3">
            {["العملاء", "المنتجات", "الفواتير", "القيود اليومية"].map((name) => (
              <button
                key={name}
                onClick={() => handleExport(name)}
                className="w-full flex items-center justify-between p-3 bg-surface-800/50 rounded-lg hover:bg-surface-800 transition-colors"
              >
                <span className="text-sm text-surface-300">{name}</span>
                <Download className="w-4 h-4 text-surface-500" />
              </button>
            ))}
          </div>
        </Card>
      </div>
    </div>
  );
}
