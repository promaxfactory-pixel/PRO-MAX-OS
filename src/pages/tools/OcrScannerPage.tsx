import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { formatOMR, formatDate, cn } from "@/lib/utils";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { ScanLine, Upload, Save, Clock, FileText, CheckCircle2 } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

interface FileWithPath {
  name: string;
  path?: string;
}

interface OcrResult {
  file_path: string;
  file_size: number;
  raw_text: string;
  confidence: number;
  suggested_fields: {
    invoice_number?: string;
    date?: string;
    customer_name?: string;
    total_amount?: number;
    vat_amount?: number;
    net_amount?: number;
    items: { description: string; quantity?: number; unit_price?: number; total?: number }[];
    raw_text: string;
  };
}

interface HistoryEntry {
  id: number;
  file_name: string;
  confidence: number;
  status: string;
  created_at: string;
}

export default function OcrScannerPage() {
  const { t } = useTranslation();
  const addNotification = useUIStore((s) => s.addNotification);
  const [file, setFile] = useState<FileWithPath | null>(null);
  const [preview, setPreview] = useState<string | null>(null);
  const [scanning, setScanning] = useState(false);
  const [result, setResult] = useState<OcrResult | null>(null);
  const [saving, setSaving] = useState(false);
  const [history, setHistory] = useState<HistoryEntry[]>([]);
  const [historyLoading, setHistoryLoading] = useState(true);

  const loadHistory = useCallback(() => {
    setHistoryLoading(true);
    invoke("ocr_get_history")
      .then((d) => setHistory((d as HistoryEntry[]) || []))
      .catch((e: unknown) => addNotification({ title: t('common.error'), message: String(e), type: 'error' }))
      .finally(() => setHistoryLoading(false));
  }, []);

  useEffect(() => { loadHistory(); }, [loadHistory]);

  const handleFileChange = async () => {
    const selected = await open({
      multiple: false,
      filters: [{ name: "Images & PDF", extensions: ["png", "jpg", "jpeg", "pdf"] }],
    });
    if (!selected) return;
    const filePath = typeof selected === "string" ? selected : selected;
    setFile({ name: filePath.split(/[\\/]/).pop() || "file", path: filePath } as FileWithPath);
    setPreview(convertFileSrc(filePath));
    setResult(null);
  };

  const handleScan = async () => {
    if (!file) return;
    setScanning(true);
    try {
      const filePath = (file as FileWithPath).path || file.name;
      const data = await invoke<OcrResult>("ocr_extract_from_file", { path: filePath });
      setResult(data);
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: String(err) });
    } finally {
      setScanning(false);
    }
  };

  const handleSave = async () => {
    if (!result) return;
    setSaving(true);
    try {
      await invoke("ocr_save_scan", {
        input: {
          file_path: result.file_path,
          raw_text: result.raw_text,
          parsed_data: JSON.stringify(result.suggested_fields),
          confidence: result.confidence,
        },
      });
      loadHistory();
      setResult(null);
      setFile(null);
      setPreview(null);
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: String(err) });
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">{t("tools.ocrScanner.title")}</h1>
          <p className="page-subtitle">{t("tools.ocrScanner.subtitle")}</p>
        </div>
      </div>

      <div className="grid grid-cols-3 gap-6">
        <div className="col-span-2 space-y-6">
          <Card>
            <div
              className={cn(
                "border-2 border-dashed rounded-xl p-8 text-center cursor-pointer transition-all",
                file ? "border-brand-500 bg-brand-900/20" : "border-surface-600 hover:border-brand-500/50"
              )}
              onClick={handleFileChange}
            >
              {preview ? (
                <img src={preview} alt="Preview" className="max-h-48 mx-auto rounded-lg" />
              ) : (
                <>
                  <Upload className="w-12 h-12 mx-auto mb-3 text-surface-500" />
                  <p className="text-surface-400">{t("tools.dragDrop")}</p>
                  <p className="text-xs text-surface-600 mt-1">PNG, JPG, PDF</p>
                </>
              )}
            </div>
            {file && (
              <div className="mt-4 flex items-center justify-between">
                <span className="text-sm text-surface-400">{file.name}</span>
                <Button icon={<ScanLine className="w-4 h-4" />} onClick={handleScan} loading={scanning}>
                  {t("tools.ocrScanner.scanDocument")}
                </Button>
              </div>
            )}
          </Card>

          {result && (
            <Card>
              <div className="flex items-center justify-between mb-4">
                <h3 className="section-title flex items-center gap-2">
                  <FileText className="w-5 h-5 text-gold-400" />
                  {t("tools.ocrScanner.scanResults")}
                  <span className="text-xs font-normal text-surface-400">
                    {t("tools.confidence", { confidence: Math.round(result.confidence) })}
                  </span>
                </h3>
                <Button icon={<Save className="w-4 h-4" />} onClick={handleSave} loading={saving}>
                  {t("tools.ocrScanner.saveToSystem")}
                </Button>
              </div>
              <div className="grid grid-cols-2 gap-4 mb-4">
                <div className="space-y-2">
                  <div className="flex justify-between py-1.5 border-b border-surface-700/30">
                    <span className="text-surface-400 text-sm">{t("invoice.invoiceNo")}</span>
                    <span className="font-mono text-brand-400">{result.suggested_fields.invoice_number || "—"}</span>
                  </div>
                  <div className="flex justify-between py-1.5 border-b border-surface-700/30">
                    <span className="text-surface-400 text-sm">{t("common.date")}</span>
                    <span>{result.suggested_fields.date || "—"}</span>
                  </div>
                  <div className="flex justify-between py-1.5 border-b border-surface-700/30">
                    <span className="text-surface-400 text-sm">{t("invoice.customer")}</span>
                    <span>{result.suggested_fields.customer_name || "—"}</span>
                  </div>
                </div>
                <div className="space-y-2">
                  <div className="flex justify-between py-1.5 border-b border-surface-700/30">
                    <span className="text-surface-400 text-sm">{t("tools.ocrScanner.netAmount")}</span>
                    <span>{formatOMR(result.suggested_fields.net_amount || 0)}</span>
                  </div>
                  <div className="flex justify-between py-1.5 border-b border-surface-700/30">
                    <span className="text-surface-400 text-sm">{t("common.vat")}</span>
                    <span>{formatOMR(result.suggested_fields.vat_amount || 0)}</span>
                  </div>
                  <div className="flex justify-between py-1.5">
                    <span className="text-surface-400 text-sm font-bold">{t("tools.total")}</span>
                    <span className="text-lg font-bold gradient-text">{formatOMR(result.suggested_fields.total_amount || 0)}</span>
                  </div>
                </div>
              </div>
              {result.suggested_fields.items.length > 0 && (
                <div className="mt-4">
                  <h4 className="text-sm font-medium text-surface-300 mb-2">{t("tools.ocrScanner.items")}</h4>
                  <div className="bg-surface-800/50 rounded-lg overflow-hidden">
                    <table className="w-full text-sm">
                      <thead>
                        <tr className="border-b border-surface-700/50 text-surface-400">
                          <th className="p-2 text-right">{t("tools.description")}</th>
                          <th className="p-2 text-center">{t("invoice.qty")}</th>
                          <th className="p-2 text-center">{t("print.price")}</th>
                          <th className="p-2 text-left">{t("tools.total")}</th>
                        </tr>
                      </thead>
                      <tbody>
                        {result.suggested_fields.items.map((item, i) => (
                          <tr key={i} className="border-b border-surface-700/20">
                            <td className="p-2">{item.description}</td>
                            <td className="p-2 text-center">{item.quantity ?? "—"}</td>
                            <td className="p-2 text-center">{formatOMR(item.unit_price || 0)}</td>
                            <td className="p-2 text-left">{formatOMR(item.total || 0)}</td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </div>
              )}
            </Card>
          )}
        </div>

        <Card>
          <h3 className="section-title flex items-center gap-2 mb-4">
            <Clock className="w-5 h-5 text-brand-400" />
            {t("tools.scanHistory")}
          </h3>
          {historyLoading ? (
            <div className="flex justify-center py-8">
              <div className="w-8 h-8 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" />
            </div>
          ) : history.length === 0 ? (
            <p className="text-center text-surface-500 py-8 text-sm">{t("tools.ocrScanner.noPreviousScans")}</p>
          ) : (
            <div className="space-y-3 max-h-96 overflow-y-auto">
              {history.map((h) => (
                <div key={h.id} className="p-3 bg-surface-800/50 rounded-lg hover:bg-surface-800 transition-colors">
                  <div className="flex items-center justify-between mb-1">
                    <span className="text-sm font-medium text-surface-200 truncate">{h.file_name}</span>
                    <CheckCircle2 className="w-4 h-4 text-emerald-400 flex-shrink-0" />
                  </div>
                  <div className="flex justify-between mt-1">
                    <span className="text-xs text-surface-500">{t("tools.confidence", { confidence: Math.round(h.confidence) })}</span>
                    <span className="text-xs text-surface-600">{formatDate(h.created_at)}</span>
                  </div>
                </div>
              ))}
            </div>
          )}
        </Card>
      </div>
    </div>
  );
}
