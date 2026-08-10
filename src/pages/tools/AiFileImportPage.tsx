import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { cn } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { useUIStore } from "@/stores/uiStore";
import {
  FileUp, Sparkles, FileText, ShoppingCart, Users, Package,
  Receipt, Truck, ScanLine, CheckCircle2, XCircle, Loader2,
  Database, Trash2, RefreshCw, AlertTriangle, Wand2
} from "lucide-react";

interface FileWithPath {
  name: string;
  path?: string;
}

interface ExtractionRecord {
  id: number;
  file_path: string;
  file_name: string;
  file_type: string;
  doc_type: string;
  provider: string;
  model: string;
  raw_text: string;
  extracted_json: string;
  fields_json: string;
  confidence: number;
  status: string;
  target_table: string | null;
  target_id: number | null;
  created_at: string;
  updated_at: string | null;
}

interface ExtractionSummary {
  id: number;
  file_name: string;
  doc_type: string;
  provider: string;
  confidence: number;
  status: string;
  created_at: string;
  summary: string;
}

interface CommitResult {
  success: boolean;
  target_table: string;
  target_id: number;
  ref_no: string;
  created: string[];
  resolved: string[];
  warnings: string[];
  message: string;
}

interface ProviderStatus {
  id: string;
  label: string;
  model: string;
  configured: boolean;
  enabled: boolean;
  requires_key: boolean;
  free_tier: boolean;
  message: string;
}

const DOC_TYPES = [
  { value: "", labelKey: "auto" },
  { value: "invoice", labelKey: "invoice" },
  { value: "purchase", labelKey: "purchase" },
  { value: "customer", labelKey: "customer" },
  { value: "product", labelKey: "product" },
  { value: "supplier", labelKey: "supplier" },
  { value: "expense", labelKey: "expense" },
  { value: "inventory", labelKey: "inventory" },
];

const STATUS_STYLES: Record<string, string> = {
  draft: "bg-amber-500/10 text-amber-400 border border-amber-500/30",
  committed: "bg-emerald-500/10 text-emerald-400 border border-emerald-500/30",
  failed: "bg-red-500/10 text-red-400 border border-red-500/30",
};

export default function AiFileImportPage() {
  const { t } = useTranslation();
  const addNotification = useUIStore((s) => s.addNotification);

  const [file, setFile] = useState<File | null>(null);
  const [docType, setDocType] = useState("");
  const [provider, setProvider] = useState("auto");
  const [providers, setProviders] = useState<ProviderStatus[]>([]);
  const [analyzing, setAnalyzing] = useState(false);
  const [committing, setCommitting] = useState(false);
  const [current, setCurrent] = useState<ExtractionRecord | null>(null);
  const [jsonEdit, setJsonEdit] = useState("");
  const [editing, setEditing] = useState(false);
  const [history, setHistory] = useState<ExtractionSummary[]>([]);
  const [commitResult, setCommitResult] = useState<CommitResult | null>(null);
  const [duplicates, setDuplicates] = useState<{ table: string; ref: string; date: string; total: number }[]>([]);
  const [checkingDup, setCheckingDup] = useState(false);

  const loadProviders = useCallback(() => {
    invoke<ProviderStatus[]>("ai_provider_statuses").then(setProviders).catch(() => setProviders([]));
  }, []);

  const loadHistory = useCallback(() => {
    invoke<ExtractionSummary[]>("ai_list_extractions").then(setHistory).catch(() => setHistory([]));
  }, []);

  useEffect(() => { loadProviders(); loadHistory(); }, [loadProviders, loadHistory]);

  const handleFileChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const selected = e.target.files?.[0];
    if (!selected) return;
    setFile(selected);
    setCurrent(null);
    setCommitResult(null);
    setDuplicates([]);
  };

  const handleAnalyze = async () => {
    if (!file) return;
    setAnalyzing(true);
    setCommitResult(null);
    setDuplicates([]);
    try {
      const path = (file as FileWithPath).path || file.name;
      const record = await invoke<ExtractionRecord>("ai_analyze_document", {
        input: {
          path,
          doc_type: docType || null,
          provider: provider === "auto" ? null : provider,
        },
      });
      setCurrent(record);
      setJsonEdit(record.extracted_json);
      setEditing(false);
      try {
        const dup = await invoke<{ duplicates: any[] }>("ai_duplicate_check", { extractedJson: record.extracted_json });
        setDuplicates(dup.duplicates || []);
      } catch { setDuplicates([]); }
      loadHistory();
    } catch (err: unknown) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: err instanceof Error ? err.message : String(err) || t("tools.aiFileImport.analysisFailed") });
    } finally {
      setAnalyzing(false);
    }
  };

  const handleEdit = async () => {
    if (!current) return;
    setCommitting(true);
    try {
      const updated = await invoke<ExtractionRecord>("ai_update_extraction", {
        id: current.id,
        docType,
        extractedJson: jsonEdit,
      });
      setCurrent(updated);
      setEditing(false);
      addNotification({ id: crypto.randomUUID(), type: "success", title: t("common.success"), message: t("tools.aiFileImport.saved") });
    } catch (err: unknown) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: err instanceof Error ? err.message : String(err) });
    } finally {
      setCommitting(false);
    }
  };

  const handleCommit = async () => {
    if (!current) return;
    setCommitting(true);
    try {
      const result = await invoke<CommitResult>("ai_commit_extraction", { id: current.id });
      setCommitResult(result);
      loadHistory();
      if (current) setCurrent({ ...current, status: "committed" });
      addNotification({ id: crypto.randomUUID(), type: "success", title: t("common.success"), message: result.message });
    } catch (err: unknown) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: err instanceof Error ? err.message : String(err) || t("tools.aiFileImport.commitFailed") });
    } finally {
      setCommitting(false);
    }
  };

  const handleDelete = async (id: number) => {
    try {
      await invoke("ai_delete_extraction", { id });
      if (current?.id === id) { setCurrent(null); setCommitResult(null); }
      loadHistory();
    } catch (err: unknown) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: err instanceof Error ? err.message : String(err) });
    }
  };

  const checkDuplicate = async () => {
    if (!jsonEdit) return;
    setCheckingDup(true);
    try {
      const dup = await invoke<{ duplicates: any[] }>("ai_duplicate_check", { extractedJson: jsonEdit });
      setDuplicates(dup.duplicates || []);
    } catch (err: unknown) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: err instanceof Error ? err.message : String(err) });
    } finally {
      setCheckingDup(false);
    }
  };

  const parsed = (() => {
    try { return JSON.parse(jsonEdit || "null") as Record<string, any>; }
    catch { return null; }
  })();

  const renderFields = () => {
    if (!parsed) return null;
    const fields = parsed.fields;
    if (fields && typeof fields === "object") {
      return (
        <div className="grid grid-cols-2 md:grid-cols-3 gap-3">
          {Object.entries(fields).map(([k, v]) => (
            <div key={k} className="bg-surface-800/60 rounded-xl p-3 border border-surface-700/50">
              <p className="text-[11px] uppercase tracking-wide text-surface-500 mb-1">{k}</p>
              <p className="text-sm font-medium text-surface-100 truncate" title={String(v)}>{String(v ?? "—")}</p>
            </div>
          ))}
        </div>
      );
    }
    return null;
  };

  const renderItems = () => {
    if (!parsed?.items || !Array.isArray(parsed.items) || parsed.items.length === 0) return null;
    return (
      <div className="mt-4 overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="text-surface-500 text-xs uppercase tracking-wide">
              {Object.keys(parsed.items[0]).map((k) => (
                <th key={k} className="text-right p-2 border-b border-surface-700">{k}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {parsed.items.map((it: any, i: number) => (
              <tr key={i} className="border-b border-surface-800">
                {Object.values(it).map((v: any, j: number) => (
                  <td key={j} className="p-2 text-surface-200">{String(v ?? "—")}</td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    );
  };

  const providerLabel = (id: string) => providers.find((p) => p.id === id)?.label || id;

  return (
    <div className="max-w-6xl mx-auto space-y-6">
      <div>
        <h1 className="page-title">{t("tools.aiFileImport.title")}</h1>
        <p className="page-subtitle">{t("tools.aiFileImport.subtitle")}</p>
      </div>

      {/* Upload + settings */}
      <Card className="space-y-4">
        <div className="flex flex-col md:flex-row md:items-end gap-4">
          <div className="flex-1">
            <label className="form-label">{t("tools.aiFileImport.fileLabel")}</label>
            <label
              className={cn(
                "flex flex-col items-center justify-center gap-2 border-2 border-dashed rounded-xl p-6 cursor-pointer transition-all",
                file ? "border-emerald-500/50 bg-emerald-500/5" : "border-surface-700 hover:border-surface-500 bg-surface-800/30"
              )}
            >
              <FileUp className={cn("w-8 h-8", file ? "text-emerald-400" : "text-surface-500")} />
              <span className="text-sm text-surface-300">{file ? file.name : t("tools.dragDrop")}</span>
              <span className="text-xs text-surface-500">{t("tools.aiFileImport.fileHint")}</span>
              <input type="file" className="hidden" onChange={handleFileChange}
                accept=".pdf,.png,.jpg,.jpeg,.bmp,.tiff,.webp,.xlsx,.xls,.csv,.docx,.txt,.json,.xml" />
            </label>
          </div>

          <div className="grid grid-cols-2 md:grid-cols-1 gap-3">
            <div>
              <label className="form-label">{t("tools.aiFileImport.documentType")}</label>
              <select value={docType} onChange={(e) => setDocType(e.target.value)} className="input-field w-full" aria-label={t("tools.aiFileImport.documentType")}>
                {DOC_TYPES.map((d) => (
                  <option key={d.value || "auto"} value={d.value}>{t(`tools.aiFileImport.docTypes.${d.labelKey}`)}</option>
                ))}
              </select>
            </div>
            <div>
              <label className="form-label">{t("tools.aiFileImport.provider")}</label>
              <select value={provider} onChange={(e) => setProvider(e.target.value)} className="input-field w-full" aria-label={t("tools.aiFileImport.provider")}>
                <option value="auto">{t("tools.aiFileImport.autoProvider")}</option>
                {providers.map((p) => (
                  <option key={p.id} value={p.id} disabled={!p.configured}>
                    {p.label}{p.configured ? "" : ` (${t("tools.aiFileImport.notConfigured")})`}
                  </option>
                ))}
              </select>
            </div>
            <Button onClick={handleAnalyze} disabled={!file || analyzing} className="md:mt-auto"
              icon={analyzing ? <Loader2 className="w-4 h-4 animate-spin" /> : <Wand2 className="w-4 h-4" />}>
              {analyzing ? t("tools.aiFileImport.analyzing") : t("tools.aiFileImport.analyze")}
            </Button>
          </div>
        </div>

        {/* Provider status */}
        <div className="flex flex-wrap gap-2 pt-2">
          {providers.map((p) => (
            <span key={p.id} className={cn(
              "inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-[11px] border",
              p.configured ? "bg-emerald-500/10 text-emerald-400 border-emerald-500/30" : "bg-surface-800 text-surface-500 border-surface-700"
            )}>
              {p.configured ? <CheckCircle2 className="w-3 h-3" /> : <XCircle className="w-3 h-3" />}
              {p.label}
              {p.free_tier && <span className="opacity-70">{t("tools.aiFileImport.freeTier")}</span>}
            </span>
          ))}
        </div>
      </Card>

      {/* Current extraction */}
      {current && (
        <Card className="space-y-4">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 rounded-xl bg-gold-500/15 flex items-center justify-center">
                <Sparkles className="w-5 h-5 text-gold-400" />
              </div>
              <div>
                <h3 className="font-bold text-white">{current.file_name}</h3>
                <p className="text-xs text-surface-500">
                  {t("tools.aiFileImport.byProvider", { provider: providerLabel(current.provider) })}
                  {current.model ? ` · ${current.model}` : ""} ·{" "}
                  {t("tools.confidence", { confidence: Math.round(current.confidence) })}
                </p>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <span className={cn("px-2.5 py-1 rounded-lg text-xs font-medium", STATUS_STYLES[current.status] || "bg-surface-800 text-surface-400")}>
                {t(`tools.aiFileImport.status.${current.status}`)}
              </span>
              {current.status === "draft" && (
                <>
                  <Button variant="outline" size="sm" onClick={() => setEditing(!editing)}
                    icon={<RefreshCw className="w-4 h-4" />}>
                    {t("tools.aiFileImport.edit")}
                  </Button>
                  <Button variant="danger" size="sm" onClick={() => handleDelete(current.id)}
                    icon={<Trash2 className="w-4 h-4" />}>
                    {t("tools.aiFileImport.delete")}
                  </Button>
                </>
              )}
            </div>
          </div>

          {duplicates.length > 0 && (
            <div className="flex items-start gap-2 p-3 rounded-xl bg-amber-500/10 border border-amber-500/30 text-sm text-amber-400">
              <AlertTriangle className="w-4 h-4 mt-0.5 flex-shrink-0" />
              <div>
                <p className="font-medium">{t("tools.aiFileImport.duplicatesFound")}</p>
                {duplicates.map((d, i) => (
                  <p key={i} className="text-xs mt-1">{d.table} · {d.ref} · {d.date} · {d.total}</p>
                ))}
              </div>
            </div>
          )}

          {!editing ? (
            <>
              <div className="flex items-center gap-2 text-sm text-surface-400">
                <ScanLine className="w-4 h-4" />
                {t("tools.aiFileImport.extractedFields")}
                <button onClick={checkDuplicate} className="ml-auto text-xs text-gold-400 hover:underline flex items-center gap-1">
                  <RefreshCw className={cn("w-3 h-3", checkingDup && "animate-spin")} />
                  {t("tools.aiFileImport.checkDuplicate")}
                </button>
              </div>
              {renderFields()}
              {renderItems()}
            </>
          ) : (
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <label className="text-sm text-surface-400">{t("tools.aiFileImport.jsonEditor")}</label>
                <Button variant="ghost" size="sm" onClick={() => setJsonEdit(current.extracted_json)} icon={<RefreshCw className="w-4 h-4" />}>
                  {t("tools.aiFileImport.reset")}
                </Button>
              </div>
              <textarea
                value={jsonEdit}
                onChange={(e) => setJsonEdit(e.target.value)}
                className="input-field w-full font-mono text-xs h-64"
                dir="ltr"
                aria-label={t("tools.aiFileImport.jsonEditor")}
              />
              <Button onClick={handleEdit} disabled={committing}
                icon={committing ? <Loader2 className="w-4 h-4 animate-spin" /> : <CheckCircle2 className="w-4 h-4" />}>
                {t("tools.aiFileImport.save")}
              </Button>
            </div>
          )}

          {current.status === "draft" && !editing && (
            <div className="flex items-center gap-3 pt-2 border-t border-surface-700/50">
              <Button onClick={handleCommit} disabled={committing}
                icon={committing ? <Loader2 className="w-4 h-4 animate-spin" /> : <Database className="w-4 h-4" />}>
                {t("tools.aiFileImport.commitToSystem")}
              </Button>
              <p className="text-xs text-surface-500">{t("tools.aiFileImport.commitHint")}</p>
            </div>
          )}
        </Card>
      )}

      {/* Commit result */}
      {commitResult && (
        <Card className={cn("border", commitResult.success ? "border-emerald-500/40" : "border-red-500/40")}>
          <div className="flex items-start gap-3">
            {commitResult.success
              ? <CheckCircle2 className="w-6 h-6 text-emerald-400 flex-shrink-0" />
              : <XCircle className="w-6 h-6 text-red-400 flex-shrink-0" />}
            <div className="flex-1">
              <h3 className="font-bold text-white">{commitResult.message}</h3>
              <p className="text-xs text-surface-500 mt-1">
                {t("tools.aiFileImport.targetTable")}: {commitResult.target_table}
                {commitResult.ref_no ? ` · ${commitResult.ref_no}` : ""}
                {commitResult.target_id ? ` · #${commitResult.target_id}` : ""}
              </p>
              {(commitResult.created.length > 0 || commitResult.resolved.length > 0) && (
                <div className="mt-2 text-xs">
                  {commitResult.created.length > 0 && (
                    <p className="text-emerald-400">{t("tools.aiFileImport.createdItems", { count: commitResult.created.length })}</p>
                  )}
                  {commitResult.resolved.length > 0 && (
                    <p className="text-gold-400">{t("tools.aiFileImport.resolvedItems", { count: commitResult.resolved.length })}</p>
                  )}
                </div>
              )}
            </div>
          </div>
        </Card>
      )}

      {/* History */}
      <Card>
        <div className="flex items-center justify-between mb-4">
          <h3 className="section-title">{t("tools.aiFileImport.history")}</h3>
          <button onClick={loadHistory} className="text-surface-500 hover:text-surface-300">
            <RefreshCw className="w-4 h-4" />
          </button>
        </div>
        {history.length === 0 ? (
          <p className="text-center text-surface-500 py-6 text-sm">{t("tools.aiFileImport.noHistory")}</p>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="text-surface-500 text-xs uppercase tracking-wide">
                  <th className="text-right p-2">{t("tools.aiFileImport.colFile")}</th>
                  <th className="text-right p-2">{t("tools.aiFileImport.colType")}</th>
                  <th className="text-right p-2">{t("tools.aiFileImport.colProvider")}</th>
                  <th className="text-right p-2">{t("tools.aiFileImport.colConfidence")}</th>
                  <th className="text-right p-2">{t("tools.aiFileImport.colStatus")}</th>
                  <th className="text-right p-2">{t("tools.aiFileImport.colDate")}</th>
                  <th className="text-right p-2"></th>
                </tr>
              </thead>
              <tbody>
                {history.map((h) => (
                  <tr key={h.id} className="border-b border-surface-800 hover:bg-surface-800/30 cursor-pointer" onClick={() => invoke<ExtractionRecord>("ai_get_extraction", { id: h.id }).then(setCurrent).catch(() => {})}>
                    <td className="p-2 text-surface-200">{h.file_name}</td>
                    <td className="p-2">
                      <span className="inline-flex items-center gap-1 text-xs text-surface-300">
                        {h.doc_type === "purchase" ? <ShoppingCart className="w-3 h-3" />
                          : h.doc_type === "customer" ? <Users className="w-3 h-3" />
                            : h.doc_type === "product" ? <Package className="w-3 h-3" />
                              : h.doc_type === "supplier" ? <Truck className="w-3 h-3" />
                                : h.doc_type === "expense" ? <Receipt className="w-3 h-3" />
                                  : <FileText className="w-3 h-3" />}
                        {t(`tools.aiFileImport.docTypes.${h.doc_type}`)}
                      </span>
                    </td>
                    <td className="p-2 text-xs text-surface-400">{providerLabel(h.provider)}</td>
                    <td className="p-2 text-surface-300">{Math.round(h.confidence)}%</td>
                    <td className="p-2">
                      <span className={cn("px-2 py-0.5 rounded-md text-[11px]", STATUS_STYLES[h.status] || "bg-surface-800 text-surface-400")}>
                        {t(`tools.aiFileImport.status.${h.status}`)}
                      </span>
                    </td>
                    <td className="p-2 text-xs text-surface-500">{h.created_at}</td>
                    <td className="p-2 text-right">
                      <button
                        onClick={(e) => { e.stopPropagation(); handleDelete(h.id); }}
                        className="text-surface-600 hover:text-red-400"
                        aria-label={t("tools.aiFileImport.delete")}
                      >
                        <Trash2 className="w-3.5 h-3.5" />
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </Card>
    </div>
  );
}
