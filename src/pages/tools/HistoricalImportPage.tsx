import { useState, useEffect, useCallback } from "react";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { cn } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import {
  Users, Truck, Package, Warehouse, FileText, ShoppingCart,
  Receipt, Database, UserCheck, Upload, CheckCircle2, XCircle,
  AlertTriangle, RotateCcw, ArrowRight, ChevronLeft, FileSpreadsheet,
  Download, Loader2, ChevronDown
} from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

interface FileWithPath {
  name: string;
  path?: string;
}

type EntityType =
  | "customers" | "suppliers" | "products"
  | "inventory" | "invoices" | "purchases"
  | "expenses" | "opening_balances" | "employees";

interface Template {
  name: string;
  entity_type: EntityType;
  columns: { field: string; label: string; required: boolean; description: string }[];
}

interface PreviewData {
  headers: string[];
  rows: Record<string, any>[];
  mappings: { source: string; target: string }[];
  validation: { row: number; column: string; message: string; severity: "error" | "warning" }[];
  total_rows: number;
  valid_rows: number;
  error_count: number;
}

interface ImportResult {
  imported: number;
  skipped: number;
  errors: { row: number; message: string }[];
}

interface HistoryEntry {
  id: number;
  entity_type: string;
  file_name: string;
  rows_imported: number;
  rows_skipped: number;
  created_at: string;
}

const ENTITY_CARDS: { type: EntityType; label: string; description: string; icon: any }[] = [
  { type: "customers", label: "العملاء", description: "استيراد بيانات العملاء والموردين", icon: Users },
  { type: "suppliers", label: "الموردين", description: "استيراد بيانات الموردين والبائعين", icon: Truck },
  { type: "products", label: "المنتجات", description: "استيراد قائمة المنتجات والأصناف", icon: Package },
  { type: "inventory", label: "المخزون", description: "استيراد أرصدة المخزون والمستودعات", icon: Warehouse },
  { type: "invoices", label: "فواتير المبيعات", description: "استيراد فواتير المبيعات السابقة", icon: FileText },
  { type: "purchases", label: "مشتريات", description: "استيراد سجل المشتريات والمشتريات", icon: ShoppingCart },
  { type: "expenses", label: "النفقات", description: "استيراد سجل المصروفات التشغيلية", icon: Receipt },
  { type: "opening_balances", label: "الأرصدة الافتتاحية", description: "استيراد الأرصدة الافتتاحية للحسابات", icon: Database },
  { type: "employees", label: "الموظفين", description: "استيراد بيانات الموظفين والموظفات", icon: UserCheck },
];

const TARGET_FIELDS_BY_TYPE: Record<EntityType, string[]> = {
  customers: ["name", "phone", "email", "address", "vat_number", "credit_limit", "notes"],
  suppliers: ["name", "phone", "email", "address", "vat_number", "payment_terms", "notes"],
  products: ["sku", "name", "barcode", "category", "unit_price", "cost_price", "unit", "min_stock"],
  inventory: ["product_sku", "warehouse", "quantity", "batch_number", "expiry_date", "unit_cost"],
  invoices: ["invoice_no", "date", "customer_name", "subtotal", "vat", "total", "status"],
  purchases: ["purchase_no", "date", "supplier_name", "subtotal", "vat", "total", "status"],
  expenses: ["date", "category", "amount", "description", "payment_method", "reference"],
  opening_balances: ["account_code", "account_name", "debit", "credit", "date", "notes"],
  employees: ["name", "national_id", "phone", "email", "department", "position", "salary", "hire_date"],
};

export default function HistoricalImportPage() {
  const navigate = useNavigate();

  const [step, setStep] = useState<1 | 2 | 3>(1);
  const [entityType, setEntityType] = useState<EntityType | null>(null);
  const [file, setFile] = useState<File | null>(null);
  const [preview, setPreview] = useState<PreviewData | null>(null);
  const [mappings, setMappings] = useState<{ source: string; target: string }[]>([]);
  const [skipFirstRow, setSkipFirstRow] = useState(true);
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<ImportResult | null>(null);
  const [templates, setTemplates] = useState<Template[]>([]);
  const [history, setHistory] = useState<HistoryEntry[]>([]);
  const [error, setError] = useState<string | null>(null);

  const loadTemplates = useCallback(async () => {
    try {
      const data = await invoke<Template[]>("get_import_templates");
      setTemplates(data);
    } catch { /* ignore */ }
  }, []);

  const loadHistory = useCallback(async () => {
    try {
      const data = await invoke<HistoryEntry[]>("import_get_history");
      setHistory(data);
    } catch { /* ignore */ }
  }, []);

  useEffect(() => {
    loadTemplates();
    loadHistory();
  }, [loadTemplates, loadHistory]);

  const handleSelectEntity = (type: EntityType) => {
    setEntityType(type);
    setFile(null);
    setPreview(null);
    setMappings([]);
    setResult(null);
    setError(null);
    setStep(2);
  };

  const handleFileUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const selected = e.target.files?.[0];
    if (!selected || !entityType) return;
    setFile(selected);
    setError(null);
    setLoading(true);
    try {
      const path = (selected as FileWithPath).path || selected.name;
      const data = await invoke<PreviewData>("preview_import", {
        filePath: path,
        entityType,
      });
      setPreview(data);
      setMappings(data.mappings.length > 0 ? data.mappings : data.headers.map((h: string) => ({ source: h, target: "" })));
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err) || "فشل تحميل الملف");
    } finally {
      setLoading(false);
    }
  };

  const handleMappingChange = (index: number, target: string) => {
    setMappings((prev) => prev.map((m, i) => (i === index ? { ...m, target } : m)));
  };

  const handleExecuteImport = async () => {
    if (!file || !entityType) return;
    setLoading(true);
    setError(null);
    try {
      const path = (file as FileWithPath).path || file.name;
      const activeMappings = mappings.filter((m) => m.target);
      const data = await invoke<ImportResult>("execute_import", {
        entityType,
        data: { filePath: path },
        mappings: activeMappings,
        skipFirstRow,
      });
      setResult(data);
      setStep(3);
      loadHistory();
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err) || "فشل الاستيراد");
    } finally {
      setLoading(false);
    }
  };

  const resetAll = () => {
    setStep(1);
    setEntityType(null);
    setFile(null);
    setPreview(null);
    setMappings([]);
    setResult(null);
    setError(null);
  };

  const targetFields = entityType ? TARGET_FIELDS_BY_TYPE[entityType] : [];

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title flex items-center gap-2">
            <FileSpreadsheet className="w-6 h-6 text-gold-400" />
            استيراد البيانات التاريخية
          </h1>
          <p className="page-subtitle">استيراد البيانات السابقة من ملفات Excel و CSV</p>
        </div>
        {step > 1 && (
          <Button variant="ghost" icon={<RotateCcw className="w-4 h-4" />} onClick={resetAll}>
            بدء من جديد
          </Button>
        )}
      </div>

      {error && (
        <div className="flex items-center gap-2 p-3 rounded-xl text-sm bg-red-500/10 text-red-400 border border-red-500/30">
          <XCircle className="w-4 h-4 flex-shrink-0" />
          {error}
        </div>
      )}

      {templates.length > 0 && step === 1 && (
        <Card>
          <h3 className="section-title mb-4">القوالب المتاحة</h3>
          <div className="grid grid-cols-3 gap-4">
            {templates.map((tpl) => (
              <div key={tpl.name} className="p-4 bg-surface-800/50 rounded-xl border border-surface-700/50">
                <h4 className="text-sm font-medium text-surface-200 mb-2">{tpl.name}</h4>
                <div className="space-y-1">
                  {tpl.columns.slice(0, 4).map((col) => (
                    <div key={col.field} className="flex items-center gap-1 text-xs">
                      <span className="text-surface-500">{col.label}</span>
                      {col.required && <span className="text-red-400">*</span>}
                    </div>
                  ))}
                  {tpl.columns.length > 4 && (
                    <p className="text-xs text-surface-600">+{tpl.columns.length - 4} حقول أخرى</p>
                  )}
                </div>
              </div>
            ))}
          </div>
        </Card>
      )}

      <div className="flex items-center gap-3 mb-2">
        <div className={cn("w-8 h-8 rounded-full flex items-center justify-center text-sm font-bold", step >= 1 ? "bg-gold-500/20 text-gold-400" : "bg-surface-800 text-surface-500")}>1</div>
        <span className={cn("text-sm", step >= 1 ? "text-white" : "text-surface-500")}>اختيار نوع البيانات</span>
        <div className="h-px w-12 bg-surface-700" />
        <div className={cn("w-8 h-8 rounded-full flex items-center justify-center text-sm font-bold", step >= 2 ? "bg-gold-500/20 text-gold-400" : "bg-surface-800 text-surface-500")}>2</div>
        <span className={cn("text-sm", step >= 2 ? "text-white" : "text-surface-500")}>رفع ومعاينة</span>
        <div className="h-px w-12 bg-surface-700" />
        <div className={cn("w-8 h-8 rounded-full flex items-center justify-center text-sm font-bold", step >= 3 ? "bg-gold-500/20 text-gold-400" : "bg-surface-800 text-surface-500")}>3</div>
        <span className={cn("text-sm", step >= 3 ? "text-white" : "text-surface-500")}>نتائج الاستيراد</span>
      </div>

      {step === 1 && (
        <div className="grid grid-cols-3 gap-4">
          {ENTITY_CARDS.map((item) => {
            const Icon = item.icon;
            return (
              <div
                key={item.type}
                onClick={() => handleSelectEntity(item.type)}
                className="p-5 bg-surface-800/50 rounded-xl border border-surface-700/50 cursor-pointer hover:border-brand-500/40 hover:bg-surface-800 transition-all"
              >
                <div className="w-12 h-12 rounded-xl bg-brand-800/30 flex items-center justify-center mb-3">
                  <Icon className="w-6 h-6 text-brand-400" />
                </div>
                <h3 className="text-sm font-bold text-white mb-1">{item.label}</h3>
                <p className="text-xs text-surface-500">{item.description}</p>
              </div>
            );
          })}
        </div>
      )}

      {step === 2 && (
        <div className="grid grid-cols-3 gap-6">
          <div className="col-span-2 space-y-6">
            <Card>
              <div className="space-y-4">
                <h3 className="section-title">رفع الملف</h3>
                <div
                  className="border-2 border-dashed rounded-xl p-8 text-center cursor-pointer border-surface-600 hover:border-brand-500/50 transition-all"
                  onClick={() => document.getElementById("hist-file-input")?.click()}
                >
                  <input
                    id="hist-file-input"
                    type="file"
                    accept=".xlsx,.xls,.csv"
                    className="hidden"
                    onChange={handleFileUpload}
                    aria-label="اختر ملف"
                  />
                  {loading ? (
                    <Loader2 className="w-12 h-12 mx-auto mb-3 text-gold-400 animate-spin" />
                  ) : (
                    <Upload className="w-12 h-12 mx-auto mb-3 text-surface-500" />
                  )}
                  <p className="text-surface-400">اسحب الملف هنا أو انقر للاختيار</p>
                  <p className="text-xs text-surface-600 mt-1">Excel (.xlsx, .xls) أو CSV</p>
                </div>
                {file && (
                  <div className="flex items-center gap-2 text-sm text-surface-400">
                    <FileSpreadsheet className="w-4 h-4 text-brand-400" />
                    {file.name}
                  </div>
                )}
              </div>
            </Card>

            {preview && (
              <>
                <div className="flex items-center gap-4">
                  <div className="stat-card flex-1 text-center">
                    <p className="text-xs text-surface-500">إجمالي الصفوف</p>
                    <p className="text-lg font-bold text-white">{preview.total_rows}</p>
                  </div>
                  <div className="stat-card flex-1 text-center">
                    <p className="text-xs text-surface-500">صحيحة</p>
                    <p className="text-lg font-bold text-emerald-400">{preview.valid_rows}</p>
                  </div>
                  <div className="stat-card flex-1 text-center">
                    <p className="text-xs text-surface-500">أخطاء</p>
                    <p className="text-lg font-bold text-red-400">{preview.error_count}</p>
                  </div>
                </div>

                <Card>
                  <h3 className="section-title mb-3">معاينة البيانات</h3>
                  <div className="overflow-x-auto">
                    <table className="w-full text-sm">
                      <thead>
                        <tr className="border-b border-surface-700/50 text-surface-400">
                          {preview.headers.map((h) => <th key={h} className="p-2 text-right whitespace-nowrap">{h}</th>)}
                        </tr>
                      </thead>
                      <tbody>
                        {preview.rows.slice(0, 5).map((r, i) => (
                          <tr key={i} className="border-b border-surface-700/20">
                            {preview.headers.map((h) => <td key={h} className="p-2 whitespace-nowrap">{String(r[h] ?? "")}</td>)}
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </Card>

                <Card>
                  <h3 className="section-title mb-4">تعيين الأعمدة</h3>
                  <div className="space-y-3">
                    {mappings.map((m, i) => (
                      <div key={i} className="flex items-center gap-4">
                        <span className="text-sm text-surface-300 w-40 truncate">{m.source}</span>
                        <ArrowRight className="w-4 h-4 text-surface-500 flex-shrink-0" />
                        <select
                          value={m.target}
                          onChange={(e) => handleMappingChange(i, e.target.value)}
                          className="input-field flex-1"
                        >
                          <option value="">— تخطي —</option>
                          {targetFields.map((f) => {
                            const autoMapped = preview.mappings.find((pm) => pm.source === m.source)?.target;
                            return (
                              <option key={f} value={f}>
                                {f} {autoMapped === f ? "(مقترح)" : ""}
                              </option>
                            );
                          })}
                        </select>
                        {preview.validation.filter((v) => v.column === m.source).length > 0 && (
                          <AlertTriangle className="w-4 h-4 text-amber-400 flex-shrink-0" />
                        )}
                      </div>
                    ))}
                  </div>
                  <div className="mt-4 flex items-center justify-between">
                    <label className="flex items-center gap-2 text-sm text-surface-400 cursor-pointer">
                      <input
                        type="checkbox"
                        checked={skipFirstRow}
                        onChange={(e) => setSkipFirstRow(e.target.checked)}
                        className="rounded"
                      />
                      تخطي الصف الأول (رؤوس الأعمدة)
                    </label>
                    <Button onClick={handleExecuteImport} loading={loading} icon={<Download className="w-4 h-4" />}>
                      الاستيراد
                    </Button>
                  </div>
                </Card>

                {preview.validation.filter((v) => v.severity === "error" || v.severity === "warning").length > 0 && (
                  <Card>
                    <h3 className="section-title mb-3">نتائج التحقق</h3>
                    <div className="space-y-2 max-h-48 overflow-y-auto">
                      {preview.validation.map((v, i) => (
                        <div
                          key={i}
                          className={cn(
                            "flex items-center gap-2 p-2 rounded-lg text-sm",
                            v.severity === "error"
                              ? "bg-red-500/10 text-red-400"
                              : "bg-amber-500/10 text-amber-400"
                          )}
                        >
                          {v.severity === "error" ? (
                            <XCircle className="w-4 h-4 flex-shrink-0" />
                          ) : (
                            <AlertTriangle className="w-4 h-4 flex-shrink-0" />
                          )}
                          <span>صف {v.row}: {v.column} - {v.message}</span>
                        </div>
                      ))}
                    </div>
                  </Card>
                )}
              </>
            )}
          </div>

          <Card>
            <h3 className="section-title mb-4">سجل الاستيراد</h3>
            {history.length === 0 ? (
              <p className="text-center text-surface-500 py-4 text-sm">لا توجد سجلات</p>
            ) : (
              <div className="space-y-3 max-h-96 overflow-y-auto">
                {history.map((h) => {
                  const card = ENTITY_CARDS.find((c) => c.type === h.entity_type);
                  const Icon = card?.icon || Database;
                  return (
                    <div key={h.id} className="p-3 bg-surface-800/50 rounded-lg">
                      <div className="flex items-center gap-2 mb-1">
                        <Icon className="w-4 h-4 text-surface-500" />
                        <span className="text-sm font-medium text-surface-200">{h.file_name}</span>
                      </div>
                      <div className="flex items-center justify-between text-xs text-surface-500">
                        <span>{card?.label || h.entity_type}</span>
                        <span>{h.rows_imported} صف</span>
                      </div>
                    </div>
                  );
                })}
              </div>
            )}
          </Card>
        </div>
      )}

      {step === 3 && result && (
        <Card>
          <div className="text-center py-8">
            <div className="w-20 h-20 mx-auto mb-4 rounded-full bg-emerald-500/10 flex items-center justify-center">
              <CheckCircle2 className="w-10 h-10 text-emerald-400" />
            </div>
            <h3 className="text-lg font-bold text-white mb-2">تم الاستيراد بنجاح</h3>
            <div className="flex items-center justify-center gap-6 mt-4 mb-6">
              <div className="text-center">
                <p className="text-2xl font-bold text-emerald-400">{result.imported}</p>
                <p className="text-xs text-surface-500">تم استيرادها</p>
              </div>
              {result.skipped > 0 && (
                <div className="text-center">
                  <p className="text-2xl font-bold text-amber-400">{result.skipped}</p>
                  <p className="text-xs text-surface-500">تم تخطيها</p>
                </div>
              )}
            </div>
            {result.errors.length > 0 && (
              <div className="max-w-md mx-auto mb-6 space-y-1">
                {result.errors.map((e, i) => (
                  <div key={i} className="flex items-center gap-2 text-sm text-red-400 bg-red-500/10 p-2 rounded-lg">
                    <XCircle className="w-4 h-4 flex-shrink-0" />
                    <span>صف {e.row}: {e.message}</span>
                  </div>
                ))}
              </div>
            )}
            <div className="flex items-center justify-center gap-3">
              <Button variant="outline" onClick={resetAll} icon={<RotateCcw className="w-4 h-4" />}>
                استيراد المزيد
              </Button>
              <Button variant="ghost" onClick={() => navigate(-1)} icon={<ChevronLeft className="w-4 h-4" />}>
                العودة
              </Button>
            </div>
          </div>
        </Card>
      )}
    </div>
  );
}
