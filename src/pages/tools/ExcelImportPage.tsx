import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { cn } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import {
  FileSpreadsheet, Upload, CheckCircle2, XCircle, ArrowRight,
  RotateCcw, Download, AlertTriangle
} from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

interface FileWithPath {
  name: string;
  path?: string;
}

type ImportType = "journal" | "customers" | "products" | "inventory";

interface PreviewRow { [key: string]: any; }
interface ColumnMapping { source: string; target: string; }
interface ValidationResult { row: number; column: string; message: string; severity: "error" | "warning"; }
interface ImportRecord { id: number; import_type: string; file_name: string; imported: number; created_at: string; }
interface ExcelAnalysisResponse {
  total_rows: number;
  headers: string[];
  validation_errors: { row: number; column: string; value: string; error_type: string; message: string; suggestion: string }[];
  warnings: string[];
}

const TARGET_FIELDS: Record<ImportType, string[]> = {
  journal: ["date", "account_code", "debit", "credit", "memo"],
  customers: ["name", "phone", "email", "address", "vat_number"],
  products: ["sku", "name", "category", "unit_price", "unit"],
  inventory: ["product_sku", "warehouse", "quantity", "batch_number", "expiry_date"],
};

export default function ExcelImportPage() {
  const { t } = useTranslation();
  const addNotification = useUIStore((s) => s.addNotification);

  const IMPORT_TYPES: { value: ImportType; label: string }[] = [
    { value: "journal", label: t("tools.excelImport.typeJournal") },
    { value: "customers", label: t("tools.excelImport.typeCustomers") },
    { value: "products", label: t("tools.excelImport.typeProducts") },
    { value: "inventory", label: t("tools.excelImport.typeInventory") },
  ];
  const [file, setFile] = useState<File | null>(null);
  const [importType, setImportType] = useState<ImportType>("journal");
  const [headers, setHeaders] = useState<string[]>([]);
  const [rows, setRows] = useState<PreviewRow[]>([]);
  const [mappings, setMappings] = useState<ColumnMapping[]>([]);
  const [errors, setErrors] = useState<ValidationResult[]>([]);
  const [importing, setImporting] = useState(false);
  const [dryRun, setDryRun] = useState(false);
  const [history, setHistory] = useState<ImportRecord[]>([]);
  const [step, setStep] = useState<"upload" | "map" | "review" | "done">("upload");

  const loadHistory = useCallback(() => {
    invoke<ImportRecord[]>("excel_get_import_history").then(setHistory).catch(() => setHistory([]));
  }, []);

  useEffect(() => { loadHistory(); }, [loadHistory]);

  const handleFileChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const selected = e.target.files?.[0];
    if (!selected) return;
    setFile(selected);
    setStep("map");
    setErrors([]);
    try {
      const path = (selected as FileWithPath).path || selected.name;
      const data = await invoke<{ headers: string[]; rows: (string | number | null)[][]; total_rows: number }>("excel_read_preview", { file_path: path });
      setHeaders(data.headers);
      setRows(
        data.rows.slice(0, 5).map((row) => {
          const obj: PreviewRow = {};
          data.headers.forEach((h, i) => { obj[h] = row[i] ?? ""; });
          return obj;
        })
      );
      setMappings(data.headers.map((h) => ({ source: h, target: "" })));
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("tools.loadError") });
    }
  };

  const handleMappingChange = (index: number, target: string) => {
    setMappings((prev) => prev.map((m, i) => (i === index ? { ...m, target } : m)));
  };

  const handleAnalyze = async () => {
    if (!file) return;
    try {
      const path = (file as FileWithPath).path || file.name;
      const data = await invoke<ExcelAnalysisResponse>("excel_analyze_data", {
        input: {
          file_path: path,
          sheet_name: "",
          import_type: importType,
        },
      });
      setErrors(
        data.validation_errors.map((v) => ({
          row: v.row,
          column: v.column,
          message: v.message,
          severity: v.error_type === "error" ? "error" : "warning",
        }))
      );
      setStep("review");
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("tools.loadError") });
    }
  };

  const handleImport = async () => {
    if (!file) return;
    setImporting(true);
    try {
      const path = (file as FileWithPath).path || file.name;
      const mapped = mappings.filter((m) => m.target);
      const commandMap: Record<ImportType, string> = {
        journal: "excel_import_journal",
        customers: "excel_import_customers",
        products: "excel_import_products",
        inventory: "excel_import_inventory",
      };
      await invoke(commandMap[importType], {
        input: {
          file_path: path,
          sheet_name: "",
          import_type: importType,
          column_mapping: {
            mappings: mapped.map((m) => ({ excel_column: m.source, system_field: m.target })),
          },
          skip_first_row: true,
          dry_run: dryRun,
        },
      });
      setStep("done");
      loadHistory();
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("tools.saveError") });
    } finally {
      setImporting(false);
    }
  };

  const reset = () => {
    setFile(null); setHeaders([]); setRows([]); setMappings([]); setErrors([]); setStep("upload");
  };

  const targetFields = TARGET_FIELDS[importType];
  const errorCount = errors.filter((e) => e.severity === "error").length;

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title flex items-center gap-2">
            <FileSpreadsheet className="w-6 h-6 text-gold-400" />
            {t("tools.excelImport.title")}
          </h1>
          <p className="page-subtitle">{t("tools.excelImport.subtitle")}</p>
        </div>
        {step !== "upload" && (
          <Button variant="ghost" icon={<RotateCcw className="w-4 h-4" />} onClick={reset}>{t("tools.restart")}</Button>
        )}
      </div>

      <div className="grid grid-cols-3 gap-6">
        <div className="col-span-2 space-y-6">
          {step === "upload" && (
            <Card>
              <div className="space-y-4">
                <div>
                  <label className="form-label">{t("tools.excelImport.dataType")}</label>
                  <select value={importType} onChange={(e) => setImportType(e.target.value as ImportType)} className="input-field w-64" aria-label={t("tools.excelImport.dataType")}>
                    {IMPORT_TYPES.map((it) => <option key={it.value} value={it.value}>{it.label}</option>)}
                  </select>
                </div>
                <div
                  className="border-2 border-dashed rounded-xl p-8 text-center cursor-pointer border-surface-600 hover:border-brand-500/50 transition-all"
                  onClick={() => document.getElementById("excel-input")?.click()}
                >
                  <input id="excel-input" type="file" accept=".xlsx,.xls,.csv" className="hidden" onChange={handleFileChange} aria-label={t("tools.excelImport.chooseFileAria")} />
                  <Upload className="w-12 h-12 mx-auto mb-3 text-surface-500" />
                  <p className="text-surface-400">{t("tools.excelImport.chooseExcelCsv")}</p>
                </div>
              </div>
            </Card>
          )}

          {step === "map" && (
            <>
              <Card>
                <h3 className="section-title mb-4">{t("tools.columnMapping")}</h3>
                <div className="space-y-3">
                  {mappings.map((m, i) => (
                    <div key={i} className="flex items-center gap-4">
                      <span className="text-sm text-surface-300 w-40 truncate">{m.source}</span>
                      <ArrowRight className="w-4 h-4 text-surface-500" />
                      <select
                        value={m.target}
                        onChange={(e) => handleMappingChange(i, e.target.value)}
                        className="input-field flex-1"
                      >
                        <option value="">{t("tools.skipOption")}</option>
                        {targetFields.map((f) => <option key={f} value={f}>{f}</option>)}
                      </select>
                    </div>
                  ))}
                </div>
                <div className="mt-4 flex justify-end">
                  <Button icon={<CheckCircle2 className="w-4 h-4" />} onClick={handleAnalyze}>{t("tools.excelImport.analyzeData")}</Button>
                </div>
              </Card>

              <Card>
                <h3 className="section-title mb-3">{t("tools.excelImport.dataPreviewCount", { count: rows.length })}</h3>
                <div className="overflow-x-auto">
                  <table className="w-full text-sm">
                    <thead>
                      <tr className="border-b border-surface-700/50 text-surface-400">
                        {headers.map((h) => <th key={h} className="p-2 text-right">{h}</th>)}
                      </tr>
                    </thead>
                    <tbody>
                      {rows.map((r, i) => (
                        <tr key={i} className="border-b border-surface-700/20">
                          {headers.map((h) => <td key={h} className="p-2">{String(r[h] ?? "")}</td>)}
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </Card>
            </>
          )}

          {step === "review" && (
            <Card>
              <h3 className="section-title mb-4">{t("tools.excelImport.analysisResults")}</h3>
              {errors.length === 0 ? (
                <div className="text-center py-6">
                  <CheckCircle2 className="w-12 h-12 mx-auto mb-3 text-emerald-400" />
                  <p className="text-surface-300">{t("tools.excelImport.dataValid")}</p>
                </div>
              ) : (
                <div className="space-y-2 max-h-64 overflow-y-auto">
                  {errors.map((e, i) => (
                    <div key={i} className={cn("flex items-center gap-2 p-2 rounded-lg text-sm", e.severity === "error" ? "bg-red-500/10 text-red-400" : "bg-amber-500/10 text-amber-400")}>
                      {e.severity === "error" ? <XCircle className="w-4 h-4 flex-shrink-0" /> : <AlertTriangle className="w-4 h-4 flex-shrink-0" />}
                      <span>{t("tools.excelImport.rowColumnError", { row: e.row, column: e.column, message: e.message })}</span>
                    </div>
                  ))}
                </div>
              )}
              <div className="mt-4 flex items-center justify-between">
                <label className="flex items-center gap-2 text-sm text-surface-400 cursor-pointer">
                  <input type="checkbox" checked={dryRun} onChange={(e) => setDryRun(e.target.checked)} className="rounded" />
                  {t("tools.excelImport.dryRunOnly")}
                </label>
                <Button
                  icon={<Download className="w-4 h-4" />}
                  onClick={handleImport}
                  loading={importing}
                  disabled={errorCount > 0}
                >
                  {dryRun ? t("tools.excelImport.tryImport") : t("tools.excelImport.importData")}
                </Button>
              </div>
            </Card>
          )}

          {step === "done" && (
            <Card>
              <div className="text-center py-8">
                <CheckCircle2 className="w-16 h-16 mx-auto mb-4 text-emerald-400" />
                <h3 className="text-lg font-bold text-white mb-2">{t("tools.importSuccess")}</h3>
                <Button variant="outline" onClick={reset} icon={<RotateCcw className="w-4 h-4" />}>{t("tools.excelImport.importAnotherFile")}</Button>
              </div>
            </Card>
          )}
        </div>

        <Card>
          <h3 className="section-title mb-4">{t("tools.importHistory")}</h3>
          {history.length === 0 ? (
            <p className="text-center text-surface-500 py-4 text-sm">{t("tools.noRecords")}</p>
          ) : (
            <div className="space-y-3">
              {history.map((h) => (
                <div key={h.id} className="p-3 bg-surface-800/50 rounded-lg">
                  <div className="flex justify-between mb-1">
                    <span className="text-sm font-medium text-surface-200">{h.file_name}</span>
                    <CheckCircle2 className="w-4 h-4 text-emerald-400" />
                  </div>
                  <div className="flex justify-between text-xs text-surface-500">
                    <span>{IMPORT_TYPES.find((it) => it.value === h.import_type)?.label || h.import_type}</span>
                    <span>{t("tools.rowsCount", { count: h.imported })}</span>
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
