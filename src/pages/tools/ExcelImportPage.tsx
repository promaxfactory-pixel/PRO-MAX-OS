import { useState, useEffect, useCallback } from "react";
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
interface ImportRecord { id: number; type: string; file_name: string; rows_imported: number; created_at: string; }

const IMPORT_TYPES: { value: ImportType; label: string }[] = [
  { value: "journal", label: "القيود اليومية" },
  { value: "customers", label: "العملاء" },
  { value: "products", label: "المنتجات" },
  { value: "inventory", label: "المخزون" },
];

const TARGET_FIELDS: Record<ImportType, string[]> = {
  journal: ["date", "account_code", "debit", "credit", "memo"],
  customers: ["name", "phone", "email", "address", "vat_number"],
  products: ["sku", "name", "category", "unit_price", "unit"],
  inventory: ["product_sku", "warehouse", "quantity", "batch_number", "expiry_date"],
};

export default function ExcelImportPage() {
  const addNotification = useUIStore((s) => s.addNotification);
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
      const data = await invoke<{ headers: string[]; rows: PreviewRow[] }>("excel_read_preview", { filePath: path, importType });
      setHeaders(data.headers);
      setRows(data.rows.slice(0, 5));
      setMappings(data.headers.map((h) => ({ source: h, target: "" })));
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء تحميل البيانات" });
    }
  };

  const handleMappingChange = (index: number, target: string) => {
    setMappings((prev) => prev.map((m, i) => (i === index ? { ...m, target } : m)));
  };

  const handleAnalyze = async () => {
    if (!file) return;
    try {
      const path = (file as FileWithPath).path || file.name;
      const mapped = mappings.filter((m) => m.target);
      const data = await invoke<ValidationResult[]>("excel_analyze_data", {
        filePath: path, importType, mappings: mapped,
      });
      setErrors(data);
      setStep("review");
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء تحميل البيانات" });
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
        filePath: path, mappings: mapped, dryRun,
      });
      setStep("done");
      loadHistory();
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء الحفظ" });
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
            استيراد Excel
          </h1>
          <p className="page-subtitle">استيراد البيانات من ملفات Excel</p>
        </div>
        {step !== "upload" && (
          <Button variant="ghost" icon={<RotateCcw className="w-4 h-4" />} onClick={reset}>بدء من جديد</Button>
        )}
      </div>

      <div className="grid grid-cols-3 gap-6">
        <div className="col-span-2 space-y-6">
          {step === "upload" && (
            <Card>
              <div className="space-y-4">
                <div>
                  <label className="form-label">نوع البيانات</label>
                  <select value={importType} onChange={(e) => setImportType(e.target.value as ImportType)} className="input-field w-64" aria-label="نوع البيانات">
                    {IMPORT_TYPES.map((t) => <option key={t.value} value={t.value}>{t.label}</option>)}
                  </select>
                </div>
                <div
                  className="border-2 border-dashed rounded-xl p-8 text-center cursor-pointer border-surface-600 hover:border-brand-500/50 transition-all"
                  onClick={() => document.getElementById("excel-input")?.click()}
                >
                  <input id="excel-input" type="file" accept=".xlsx,.xls,.csv" className="hidden" onChange={handleFileChange} aria-label="اختر ملف Excel" />
                  <Upload className="w-12 h-12 mx-auto mb-3 text-surface-500" />
                  <p className="text-surface-400">اختر ملف Excel أو CSV</p>
                </div>
              </div>
            </Card>
          )}

          {step === "map" && (
            <>
              <Card>
                <h3 className="section-title mb-4">تعيين الأعمدة</h3>
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
                        <option value="">— تخطي —</option>
                        {targetFields.map((f) => <option key={f} value={f}>{f}</option>)}
                      </select>
                    </div>
                  ))}
                </div>
                <div className="mt-4 flex justify-end">
                  <Button icon={<CheckCircle2 className="w-4 h-4" />} onClick={handleAnalyze}>تحليل البيانات</Button>
                </div>
              </Card>

              <Card>
                <h3 className="section-title mb-3">معاينة البيانات ({rows.length} صف)</h3>
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
              <h3 className="section-title mb-4">نتائج التحليل</h3>
              {errors.length === 0 ? (
                <div className="text-center py-6">
                  <CheckCircle2 className="w-12 h-12 mx-auto mb-3 text-emerald-400" />
                  <p className="text-surface-300">البيانات صالحة للاستيراد</p>
                </div>
              ) : (
                <div className="space-y-2 max-h-64 overflow-y-auto">
                  {errors.map((e, i) => (
                    <div key={i} className={cn("flex items-center gap-2 p-2 rounded-lg text-sm", e.severity === "error" ? "bg-red-500/10 text-red-400" : "bg-amber-500/10 text-amber-400")}>
                      {e.severity === "error" ? <XCircle className="w-4 h-4 flex-shrink-0" /> : <AlertTriangle className="w-4 h-4 flex-shrink-0" />}
                      <span>صف {e.row}: {e.column} - {e.message}</span>
                    </div>
                  ))}
                </div>
              )}
              <div className="mt-4 flex items-center justify-between">
                <label className="flex items-center gap-2 text-sm text-surface-400 cursor-pointer">
                  <input type="checkbox" checked={dryRun} onChange={(e) => setDryRun(e.target.checked)} className="rounded" />
                  تشغيل تجريبي فقط
                </label>
                <Button
                  icon={<Download className="w-4 h-4" />}
                  onClick={handleImport}
                  loading={importing}
                  disabled={errorCount > 0}
                >
                  {dryRun ? "تجربة الاستيراد" : "استيراد البيانات"}
                </Button>
              </div>
            </Card>
          )}

          {step === "done" && (
            <Card>
              <div className="text-center py-8">
                <CheckCircle2 className="w-16 h-16 mx-auto mb-4 text-emerald-400" />
                <h3 className="text-lg font-bold text-white mb-2">تم الاستيراد بنجاح</h3>
                <Button variant="outline" onClick={reset} icon={<RotateCcw className="w-4 h-4" />}>استيراد ملف آخر</Button>
              </div>
            </Card>
          )}
        </div>

        <Card>
          <h3 className="section-title mb-4">سجل الاستيراد</h3>
          {history.length === 0 ? (
            <p className="text-center text-surface-500 py-4 text-sm">لا توجد سجلات</p>
          ) : (
            <div className="space-y-3">
              {history.map((h) => (
                <div key={h.id} className="p-3 bg-surface-800/50 rounded-lg">
                  <div className="flex justify-between mb-1">
                    <span className="text-sm font-medium text-surface-200">{h.file_name}</span>
                    <CheckCircle2 className="w-4 h-4 text-emerald-400" />
                  </div>
                  <div className="flex justify-between text-xs text-surface-500">
                    <span>{IMPORT_TYPES.find((t) => t.value === h.type)?.label || h.type}</span>
                    <span>{h.rows_imported} صف</span>
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
