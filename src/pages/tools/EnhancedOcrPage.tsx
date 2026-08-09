import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { formatOMR, formatDate, cn } from "@/lib/utils";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import {
  ScanLine, Upload, FileText, Image, FileSpreadsheet,
  CheckCircle2, XCircle,
  History, Zap, RotateCcw, Loader2,
  BadgeCheck
} from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

interface FileWithPath {
  name: string;
  path?: string;
}

type ConfidenceLevel = "high" | "medium" | "low";

interface ExtractedField {
  key: string;
  label: string;
  value: string;
  confidence: ConfidenceLevel;
  source: string;
}

interface LineItem {
  description: string;
  qty: number;
  unit_price: number;
  total: number;
  confidence: ConfidenceLevel;
}

interface ExtractionResult {
  fields: ExtractedField[];
  items: LineItem[];
  raw_text: string;
  vendor_name: string;
  invoice_number: string;
  date: string;
  subtotal: number;
  vat: number;
  total: number;
  confidence_score: number;
}

interface Suggestion {
  id: string;
  label: string;
  description: string;
  action_type: "create_invoice" | "add_supplier" | "register_expense" | "update_prices";
  data: Record<string, any>;
}

interface HistoryEntry {
  id: number;
  file_name: string;
  parsed_data: string;
  confidence: number;
  status: string;
  created_at: string;
}

function parseOcrData(raw: string): { invoice_number?: string; total_amount?: number } | null {
  if (!raw) return null;
  try {
    const parsed = JSON.parse(raw);
    return {
      invoice_number: parsed?.invoice_number ?? "",
      total_amount: parsed?.total_amount ?? 0,
    };
  } catch {
    return null;
  }
}

const CONFIDENCE_COLORS: Record<ConfidenceLevel, { color: string; bg: string }> = {
  high: { color: "text-emerald-400", bg: "bg-emerald-500/10" },
  medium: { color: "text-amber-400", bg: "bg-amber-500/10" },
  low: { color: "text-red-400", bg: "bg-red-500/10" },
};

function FieldBadge({ level }: { level: ConfidenceLevel }) {
  const { t } = useTranslation();
  const cfg = CONFIDENCE_COLORS[level];
  const labelMap: Record<ConfidenceLevel, string> = {
    high: t("tools.confidenceHigh"),
    medium: t("tools.confidenceMedium"),
    low: t("tools.confidenceLow"),
  };
  return (
    <span className={cn("px-2 py-0.5 rounded-full text-xs font-medium", cfg.bg, cfg.color)}>
      {labelMap[level]}
    </span>
  );
}

export default function EnhancedOcrPage() {
  const { t } = useTranslation();
  const addNotification = useUIStore((s) => s.addNotification);

  const [activeTab, setActiveTab] = useState<"scan" | "history">("scan");
  const [file, setFile] = useState<File | null>(null);
  const [previewUrl, setPreviewUrl] = useState<string | null>(null);
  const [scanning, setScanning] = useState(false);
  const [result, setResult] = useState<ExtractionResult | null>(null);
  const [suggestions, setSuggestions] = useState<Suggestion[]>([]);
  const [acceptedSuggestions, setAcceptedSuggestions] = useState<Set<string>>(new Set());
  const [rejectedSuggestions, setRejectedSuggestions] = useState<Set<string>>(new Set());
  const [executing, setExecuting] = useState<string | null>(null);
  const [history, setHistory] = useState<HistoryEntry[]>([]);
  const [historyLoading, setHistoryLoading] = useState(true);
  const [selectedHistoryId, setSelectedHistoryId] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [successMsg, setSuccessMsg] = useState<string | null>(null);

  const loadHistory = useCallback(() => {
    setHistoryLoading(true);
    invoke("ocr_get_history")
      .then((d: any) => setHistory(d || []))
      .catch((e: unknown) => addNotification({ title: t('common.error'), message: String(e), type: 'error' }))
      .finally(() => setHistoryLoading(false));
  }, []);

  useEffect(() => { loadHistory(); }, [loadHistory]);

  const handleFileSelect = async () => {
    const picked = await open({
      multiple: false,
      filters: [{ name: "Documents", extensions: ["jpg", "jpeg", "png", "pdf", "xlsx", "xls", "csv"] }],
    });
    if (!picked) return;
    const filePath = Array.isArray(picked) ? picked[0] : picked;
    if (!filePath) return;
    const selectedName = filePath.split(/[\\/]/).pop() || "file";
    const selected = { name: selectedName, path: filePath } as unknown as File;
    setFile(selected);
    setResult(null);
    setSuggestions([]);
    setAcceptedSuggestions(new Set());
    setRejectedSuggestions(new Set());
    setError(null);

    const path = (selected as FileWithPath).path || selected.name;
    const ext = path.split(".").pop()?.toLowerCase() || "";

    if (["jpg", "jpeg", "png"].includes(ext)) {
      setPreviewUrl(convertFileSrc(path));
    } else if (ext === "pdf") {
      setPreviewUrl(null);
    } else if (["xlsx", "xls", "csv"].includes(ext)) {
      setPreviewUrl("excel");
    }

    setScanning(true);
    try {
      let data: ExtractionResult;
      if (["jpg", "jpeg", "png", "pdf"].includes(ext)) {
        data = await invoke<ExtractionResult>("ocr_parse_invoice", { path });
      } else {
        const excelData = await invoke<{ headers: string[]; rows: (string | number | null)[][]; total_rows: number }>("excel_read_preview", {
          file_path: path,
        });
        const firstRow: Record<string, any> = {};
        excelData.headers.forEach((h, i) => { firstRow[h] = excelData.rows[0]?.[i] ?? ""; });
        data = {
          fields: excelData.headers.map((h) => ({
            key: h,
            label: h,
            value: firstRow[h]?.toString() || "",
            confidence: "medium" as ConfidenceLevel,
            source: h,
          })),
          items: [],
          raw_text: "",
          vendor_name: "",
          invoice_number: "",
          date: "",
          subtotal: 0,
          vat: 0,
          total: 0,
          confidence_score: 0.7,
        };
      }
      setResult(data);

      const sugs = await invoke<Suggestion[]>("ocr_get_suggestions", { result: data }).catch(() => []);
      setSuggestions(sugs);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err) || t("tools.enhancedOcr.documentParseFailed"));
    } finally {
      setScanning(false);
    }
  };

  const handleFieldChange = (key: string, value: string) => {
    if (!result) return;
    setResult({
      ...result,
      fields: result.fields.map((f) => (f.key === key ? { ...f, value } : f)),
    });
  };

  const handleAcceptSuggestion = async (suggestion: Suggestion) => {
    setExecuting(suggestion.id);
    setError(null);
    const commandMap: Record<string, string> = {
      create_invoice: "ocr_create_invoice",
      add_supplier: "ocr_add_supplier",
      register_expense: "ocr_register_expense",
      update_prices: "ocr_update_prices",
    };
    const command = commandMap[suggestion.action_type];
    if (!command) return;
    try {
      await invoke(command, { data: suggestion.data });
      setAcceptedSuggestions((prev) => new Set(prev).add(suggestion.id));
      setSuccessMsg(t("tools.enhancedOcr.executedLabel", { label: suggestion.label }));
      setTimeout(() => setSuccessMsg(null), 3000);
      loadHistory();
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err) || t("tools.enhancedOcr.actionExecuteFailed"));
    } finally {
      setExecuting(null);
    }
  };

  const handleRejectSuggestion = (id: string) => {
    setRejectedSuggestions((prev) => new Set(prev).add(id));
  };

  const resetScan = () => {
    setFile(null);
    setPreviewUrl(null);
    setResult(null);
    setSuggestions([]);
    setAcceptedSuggestions(new Set());
    setRejectedSuggestions(new Set());
    setError(null);
    setSuccessMsg(null);
  };

  const overallConfidence = result
    ? result.fields.reduce((acc, f) => {
        const scores: Record<ConfidenceLevel, number> = { high: 1, medium: 0.6, low: 0.3 };
        return acc + scores[f.confidence];
      }, 0) / Math.max(result.fields.length, 1)
    : 0;

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title flex items-center gap-2">
            <ScanLine className="w-6 h-6 text-gold-400" />
            {t("tools.enhancedOcr.title")}
          </h1>
          <p className="page-subtitle">{t("tools.enhancedOcr.subtitle")}</p>
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant={activeTab === "scan" ? "primary" : "ghost"}
            size="sm"
            icon={<ScanLine className="w-4 h-4" />}
            onClick={() => setActiveTab("scan")}
          >
            {t("tools.enhancedOcr.scanTab")}
          </Button>
          <Button
            variant={activeTab === "history" ? "primary" : "ghost"}
            size="sm"
            icon={<History className="w-4 h-4" />}
            onClick={() => setActiveTab("history")}
          >
            {t("tools.enhancedOcr.historyTab")}
          </Button>
        </div>
      </div>

      {successMsg && (
        <div className="flex items-center gap-2 p-3 rounded-xl text-sm bg-emerald-500/10 text-emerald-400 border border-emerald-500/30">
          <CheckCircle2 className="w-4 h-4 flex-shrink-0" />
          {successMsg}
        </div>
      )}

      {error && (
        <div className="flex items-center gap-2 p-3 rounded-xl text-sm bg-red-500/10 text-red-400 border border-red-500/30">
          <XCircle className="w-4 h-4 flex-shrink-0" />
          {error}
        </div>
      )}

      {activeTab === "scan" && (
        <div className="grid grid-cols-5 gap-6">
          <div className="col-span-2 space-y-6">
            <Card>
              <div
                className={cn(
                  "border-2 border-dashed rounded-xl p-6 text-center cursor-pointer transition-all",
                  file ? "border-brand-500/50 bg-brand-900/20" : "border-surface-600 hover:border-brand-500/50"
                )}
                onClick={handleFileSelect}
              >
                {scanning ? (
                  <Loader2 className="w-12 h-12 mx-auto mb-3 text-gold-400 animate-spin" />
                ) : previewUrl === "excel" ? (
                  <FileSpreadsheet className="w-12 h-12 mx-auto mb-3 text-brand-400" />
                ) : previewUrl ? (
                  <img src={previewUrl} alt="Preview" className="max-h-40 mx-auto rounded-lg" />
                ) : (
                  <>
                    <Upload className="w-12 h-12 mx-auto mb-3 text-surface-500" />
                    <p className="text-surface-400">{t("tools.dragDrop")}</p>
                  </>
                )}
              </div>
              {file && (
                <div className="mt-3 flex items-center justify-between">
                  <div className="flex items-center gap-2 text-sm text-surface-400">
                    {previewUrl && previewUrl !== "excel" ? (
                      <Image className="w-4 h-4 text-brand-400" />
                    ) : previewUrl === "excel" ? (
                      <FileSpreadsheet className="w-4 h-4 text-brand-400" />
                    ) : (
                      <FileText className="w-4 h-4 text-brand-400" />
                    )}
                    {file.name}
                  </div>
                  {result && (
                    <Button variant="ghost" size="sm" onClick={resetScan} icon={<RotateCcw className="w-3.5 h-3.5" />}>
                      {t("tools.enhancedOcr.newScan")}
                    </Button>
                  )}
                </div>
              )}
              <div className="flex items-center gap-2 mt-3">
                <span className="px-2 py-0.5 text-xs bg-surface-800 rounded-lg text-surface-500">JPG</span>
                <span className="px-2 py-0.5 text-xs bg-surface-800 rounded-lg text-surface-500">PNG</span>
                <span className="px-2 py-0.5 text-xs bg-surface-800 rounded-lg text-surface-500">PDF</span>
                <span className="px-2 py-0.5 text-xs bg-surface-800 rounded-lg text-surface-500">XLSX</span>
                <span className="px-2 py-0.5 text-xs bg-surface-800 rounded-lg text-surface-500">CSV</span>
              </div>
            </Card>

            {result && result.confidence_score > 0 && (
              <Card>
                <h3 className="section-title flex items-center gap-2 mb-3">
                  <Zap className="w-4 h-4 text-gold-400" />
                  {t("tools.enhancedOcr.overallConfidence")}
                </h3>
                <div className="text-center">
                  <div className={cn(
                    "inline-flex items-center justify-center w-20 h-20 rounded-full text-2xl font-bold",
                    overallConfidence >= 0.8 ? "bg-emerald-500/10 text-emerald-400" :
                    overallConfidence >= 0.5 ? "bg-amber-500/10 text-amber-400" :
                    "bg-red-500/10 text-red-400"
                  )}>
                    {Math.round(overallConfidence * 100)}%
                  </div>
                </div>
              </Card>
            )}

            {result && suggestions.length > 0 && (
              <Card>
                <h3 className="section-title flex items-center gap-2 mb-3">
                  <Zap className="w-4 h-4 text-gold-400" />
                  {t("tools.suggestedActions")}
                </h3>
                <div className="space-y-3">
                  {suggestions.map((s) => {
                    const isAccepted = acceptedSuggestions.has(s.id);
                    const isRejected = rejectedSuggestions.has(s.id);
                    if (isRejected) return null;
                    return (
                      <div
                        key={s.id}
                        className={cn(
                          "p-3 rounded-xl border transition-all",
                          isAccepted
                            ? "bg-emerald-500/10 border-emerald-500/30"
                            : "bg-surface-800/50 border-surface-700/50"
                        )}
                      >
                        <div className="flex items-start justify-between gap-2">
                          <div className="flex-1">
                            <h4 className="text-sm font-medium text-white">{s.label}</h4>
                            <p className="text-xs text-surface-500 mt-0.5">{s.description}</p>
                          </div>
                          {isAccepted ? (
                            <CheckCircle2 className="w-5 h-5 text-emerald-400 flex-shrink-0 mt-0.5" />
                          ) : (
                            <div className="flex items-center gap-1 flex-shrink-0">
                              <button
                                onClick={() => handleAcceptSuggestion(s)}
                                disabled={executing === s.id}
                                className="w-7 h-7 rounded-lg bg-emerald-500/20 flex items-center justify-center hover:bg-emerald-500/30 transition-all"
                              >
                                {executing === s.id ? (
                                  <Loader2 className="w-3.5 h-3.5 text-emerald-400 animate-spin" />
                                ) : (
                                  <CheckCircle2 className="w-3.5 h-3.5 text-emerald-400" />
                                )}
                              </button>
                              <button
                                onClick={() => handleRejectSuggestion(s.id)}
                                className="w-7 h-7 rounded-lg bg-surface-700 flex items-center justify-center hover:bg-red-500/20 transition-all"
                              >
                                <XCircle className="w-3.5 h-3.5 text-surface-500 hover:text-red-400" />
                              </button>
                            </div>
                          )}
                        </div>
                      </div>
                    );
                  })}
                </div>
              </Card>
            )}
          </div>

          <div className="col-span-3 space-y-6">
            {!result && !scanning && (
              <Card>
                <div className="text-center py-12">
                  <ScanLine className="w-16 h-16 mx-auto mb-4 text-surface-600" />
                  <h3 className="text-lg font-bold text-white mb-2">{t("tools.enhancedOcr.smartScanTitle")}</h3>
                  <p className="text-surface-400 text-sm">
                    {t("tools.enhancedOcr.smartScanDesc")}
                  </p>
                </div>
              </Card>
            )}

            {scanning && (
              <Card>
                <div className="text-center py-12">
                  <Loader2 className="w-12 h-12 mx-auto mb-4 text-gold-400 animate-spin" />
                  <p className="text-surface-400">{t("tools.enhancedOcr.analyzingDocument")}</p>
                </div>
              </Card>
            )}

            {result && (
              <>
                <Card>
                  <div className="flex items-center justify-between mb-4">
                    <h3 className="section-title flex items-center gap-2">
                      <FileText className="w-5 h-5 text-gold-400" />
                      {t("tools.enhancedOcr.extractedData")}
                    </h3>
                    <BadgeCheck className={cn("w-5 h-5", result.confidence_score >= 0.8 ? "text-emerald-400" : "text-amber-400")} />
                  </div>
                  <div className="space-y-3">
                    {result.fields.map((field) => {
                      const isEditable = field.confidence !== "high";
                      return (
                        <div key={field.key} className="flex items-center gap-3 py-2 border-b border-surface-700/20 last:border-0">
                          <div className="w-36 flex-shrink-0">
                            <span className="text-xs text-surface-500">{field.label}</span>
                          </div>
                          <div className="flex-1 min-w-0">
                            {isEditable ? (
                              <input
                                type="text"
                                value={field.value}
                                onChange={(e) => handleFieldChange(field.key, e.target.value)}
                                className="input-field w-full text-sm"
                              />
                            ) : (
                              <span className="text-sm text-white">{field.value}</span>
                            )}
                          </div>
                          <FieldBadge level={field.confidence} />
                          <span className="text-xs text-surface-600 w-20 truncate text-left" dir="ltr">{field.source}</span>
                        </div>
                      );
                    })}
                  </div>
                  {result.items.length > 0 && (
                    <div className="mt-4">
                      <h4 className="text-sm font-medium text-surface-300 mb-2">{t("tools.enhancedOcr.invoiceItems")}</h4>
                      <div className="overflow-x-auto">
                        <table className="w-full text-sm">
                          <thead>
                            <tr className="border-b border-surface-700/50 text-surface-400">
                              <th className="p-2 text-right">{t("tools.description")}</th>
                              <th className="p-2 text-center">{t("invoice.qty")}</th>
                              <th className="p-2 text-center">{t("tools.enhancedOcr.unitPrice")}</th>
                              <th className="p-2 text-left">{t("tools.total")}</th>
                              <th className="p-2 text-center">{t("tools.enhancedOcr.confidenceColumn")}</th>
                            </tr>
                          </thead>
                          <tbody>
                            {result.items.map((item, i) => (
                              <tr key={i} className="border-b border-surface-700/20">
                                <td className="p-2">{item.description}</td>
                                <td className="p-2 text-center">{item.qty}</td>
                                <td className="p-2 text-center">{formatOMR(item.unit_price)}</td>
                                <td className="p-2 text-left">{formatOMR(item.total)}</td>
                                <td className="p-2 text-center"><FieldBadge level={item.confidence} /></td>
                              </tr>
                            ))}
                          </tbody>
                        </table>
                      </div>
                    </div>
                  )}
                  <div className="mt-4 grid grid-cols-3 gap-4">
                    <div className="p-3 bg-surface-800/50 rounded-lg text-center">
                      <p className="text-xs text-surface-500 mb-1">{t("tools.enhancedOcr.subtotal")}</p>
                      <p className="text-sm font-bold text-white">{formatOMR(result.subtotal)}</p>
                    </div>
                    <div className="p-3 bg-surface-800/50 rounded-lg text-center">
                      <p className="text-xs text-surface-500 mb-1">{t("common.vat")}</p>
                      <p className="text-sm font-bold text-amber-400">{formatOMR(result.vat)}</p>
                    </div>
                    <div className="p-3 bg-surface-800/50 rounded-lg text-center">
                      <p className="text-xs text-surface-500 mb-1">{t("tools.total")}</p>
                      <p className="text-sm font-bold text-gold-400">{formatOMR(result.total)}</p>
                    </div>
                  </div>
                </Card>

                {result.raw_text && (
                  <Card>
                    <h3 className="section-title mb-3">{t("tools.enhancedOcr.extractedText")}</h3>
                    <div className="bg-surface-900 rounded-lg p-4 max-h-40 overflow-y-auto">
                      <pre className="text-xs text-surface-400 whitespace-pre-wrap font-mono" dir="ltr">
                        {result.raw_text}
                      </pre>
                    </div>
                  </Card>
                )}
              </>
            )}
          </div>
        </div>
      )}

      {activeTab === "history" && (
        <Card>
          <h3 className="section-title flex items-center gap-2 mb-4">
            <History className="w-5 h-5 text-gold-400" />
            {t("tools.scanHistory")}
          </h3>
          {historyLoading ? (
            <div className="flex justify-center py-8">
              <div className="w-8 h-8 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" />
            </div>
          ) : history.length === 0 ? (
            <p className="text-center text-surface-500 py-8 text-sm">{t("tools.enhancedOcr.noHistory")}</p>
          ) : (
            <div className="space-y-3">
              {history.map((h) => {
                const isSelected = selectedHistoryId === h.id;
                const parsed = parseOcrData(h.parsed_data);
                return (
                  <div
                    key={h.id}
                    onClick={() => setSelectedHistoryId(isSelected ? null : h.id)}
                    className={cn(
                      "flex items-center justify-between p-4 rounded-xl cursor-pointer transition-all",
                      isSelected ? "bg-brand-800/30 border border-brand-500/40" : "bg-surface-800/50 hover:bg-surface-800"
                    )}
                  >
                    <div className="flex items-center gap-4">
                      <FileText className="w-5 h-5 text-surface-500" />
                      <div>
                        <span className="text-sm font-medium text-surface-200">{parsed?.invoice_number || h.file_name}</span>
                        <p className="text-xs text-surface-500 mt-0.5">{h.file_name}</p>
                      </div>
                    </div>
                    <div className="text-left">
                      <p className="text-sm font-medium">{formatOMR(parsed?.total_amount || 0)}</p>
                      <p className="text-xs text-surface-600">{formatDate(h.created_at)} • {t("tools.confidence", { confidence: Math.round(h.confidence) })}</p>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </Card>
      )}
    </div>
  );
}
