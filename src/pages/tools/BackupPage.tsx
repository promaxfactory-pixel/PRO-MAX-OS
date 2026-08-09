import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { formatDate, cn } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import {
  Database, Download, Upload, CheckCircle2,
  AlertTriangle
} from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

interface Backup { file_path: string; file_name: string; file_size: number; created_at: string; description?: string | null; }

const formatFileSize = (bytes: number) => {
  if (!bytes) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
};

export default function BackupPage() {
  const { t } = useTranslation();
  const addNotification = useUIStore((s) => s.addNotification);
  const [backups, setBackups] = useState<Backup[]>([]);
  const [loading, setLoading] = useState(true);
  const [creating, setCreating] = useState(false);
  const [restoring, setRestoring] = useState<string | null>(null);
  const [confirmRestore, setConfirmRestore] = useState<string | null>(null);
  const [message, setMessage] = useState<{ type: "success" | "error"; text: string } | null>(null);

  const loadBackups = useCallback(() => {
    setLoading(true);
    invoke<Backup[]>("backup_list")
      .then(setBackups)
      .catch((e: unknown) => addNotification({ title: t('common.error'), message: String(e), type: 'error' }))
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => { loadBackups(); }, [loadBackups]);

  const handleCreate = async () => {
    setCreating(true);
    setMessage(null);
    try {
      await invoke("backup_create");
      setMessage({ type: "success", text: t("tools.backup.createdSuccess") });
      loadBackups();
    } catch (err) {
      setMessage({ type: "error", text: t("tools.backup.createFailed") });
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: String(err) });
    } finally {
      setCreating(false);
      setTimeout(() => setMessage(null), 4000);
    }
  };

  const handleRestore = async (b: Backup) => {
    setRestoring(b.file_path);
    setMessage(null);
    try {
      await invoke("backup_restore", { backupPath: b.file_path });
      setMessage({ type: "success", text: t("tools.backup.restoreSuccess") });
    } catch (err) {
      setMessage({ type: "error", text: t("tools.backup.restoreFailed") });
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: String(err) });
    } finally {
      setRestoring(null);
      setConfirmRestore(null);
      setTimeout(() => setMessage(null), 4000);
    }
  };

  const TABLE_BY_KEY: Record<string, string> = {
    customers: "customers",
    products: "products",
    invoices: "sales_invoices",
    journal: "journal_entries",
  };

  const handleExport = async (exportKey: string) => {
    try {
      await invoke("backup_export_csv", { tableName: TABLE_BY_KEY[exportKey] || exportKey, outputPath: null });
      setMessage({ type: "success", text: t("tools.backup.exportSuccess") });
      setTimeout(() => setMessage(null), 4000);
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("tools.backup.exportError") });
    }
  };

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title flex items-center gap-2">
            <Database className="w-6 h-6 text-gold-400" />
            {t("tools.backup.title")}
          </h1>
          <p className="page-subtitle">{t("tools.backup.subtitle")}</p>
        </div>
        <Button icon={<Download className="w-4 h-4" />} onClick={handleCreate} loading={creating}>
          {t("tools.backup.createBackup")}
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
            <h3 className="section-title mb-4">{t("tools.backup.backups")}</h3>
            {loading ? (
              <div className="flex justify-center py-8">
                <div className="w-8 h-8 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" />
              </div>
            ) : backups.length === 0 ? (
              <p className="text-center text-surface-500 py-8">{t("tools.backup.noBackups")}</p>
            ) : (
              <div className="space-y-3">
                {backups.map((b) => (
                  <div key={b.file_path} className="flex items-center justify-between p-4 bg-surface-800/50 rounded-xl">
                    <div className="flex items-center gap-4">
                      <div className="w-10 h-10 rounded-lg bg-brand-800/30 flex items-center justify-center">
                        <Database className="w-5 h-5 text-brand-400" />
                      </div>
                      <div>
                        <p className="text-sm font-medium text-surface-200">{b.file_name}</p>
                        <div className="flex items-center gap-3 text-xs text-surface-500 mt-0.5">
                          <span>{formatFileSize(b.file_size)}</span>
                          <span>{b.description || t("tools.backup.backup")}</span>
                          <span>{formatDate(b.created_at)}</span>
                        </div>
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      {confirmRestore === b.file_path ? (
                        <>
                          <span className="text-xs text-amber-400">{t("tools.backup.confirmQuestion")}</span>
                          <Button variant="ghost" size="sm" onClick={() => handleRestore(b)} loading={restoring === b.file_path}>
                            {t("tools.backup.yes")}
                          </Button>
                          <Button variant="ghost" size="sm" onClick={() => setConfirmRestore(null)}>{t("tools.backup.cancel")}</Button>
                        </>
                      ) : (
                        <Button
                          variant="ghost"
                          size="sm"
                          icon={<Upload className="w-3.5 h-3.5" />}
                          onClick={() => setConfirmRestore(b.file_path)}
                        >
                          {t("tools.backup.restore")}
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
          <h3 className="section-title mb-4">{t("tools.backup.exportCsv")}</h3>
          <p className="text-sm text-surface-400 mb-4">{t("tools.backup.exportCsvDesc")}</p>
          <div className="space-y-3">
            {[
              { key: "customers", label: t("tools.backup.exportCustomers") },
              { key: "products", label: t("tools.backup.exportProducts") },
              { key: "invoices", label: t("tools.backup.exportInvoices") },
              { key: "journal", label: t("tools.backup.exportJournalEntries") },
            ].map((item) => (
              <button
                key={item.key}
                onClick={() => handleExport(item.key)}
                className="w-full flex items-center justify-between p-3 bg-surface-800/50 rounded-lg hover:bg-surface-800 transition-colors"
              >
                <span className="text-sm text-surface-300">{item.label}</span>
                <Download className="w-4 h-4 text-surface-500" />
              </button>
            ))}
          </div>
        </Card>
      </div>
    </div>
  );
}
