import { useState, useEffect, useCallback } from "react";
import Card, { StatCard } from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { formatOMR, formatDate, cn } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import {
  FileCheck, Send, Eye, CheckCircle2, XCircle, Clock, RefreshCw,
  Settings2, ListOrdered, AlertTriangle, Hash, Ban,
  Play, Globe, Server, Key,
} from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

interface DashboardData {
  total_generated: number;
  total_submitted: number;
  total_accepted: number;
  total_rejected: number;
  total_pending_submission: number;
  total_amount_milli: number;
  total_vat_milli: number;
  queue_pending: number;
  queue_failed: number;
  settings_configured: boolean;
}

interface EinvoiceRecord {
  id: number;
  invoice_id: number;
  invoice_no: string;
  customer_name: string;
  total_milli: number;
  status: string;
  compliance_score: number;
  created_at: string;
  submitted_at: string | null;
}

interface EInvResult {
  invoice_id: number;
  invoice_no: string;
  xml_content: string;
  hash: string;
  qr_code_data: string;
  generated_at: string;
}

interface ValidationResult {
  is_valid: boolean;
  errors: ValidationIssue[];
  warnings: ValidationIssue[];
  compliance_score: number;
}

interface ValidationIssue {
  field: string;
  code: string;
  message: string;
  severity: string;
}

interface QueueItem {
  id: number;
  invoice_id: number;
  invoice_no: string;
  customer_name: string;
  total_milli: number;
  action: string;
  retry_count: number;
  max_retries: number;
  last_error: string | null;
  next_retry_at: string | null;
  status: string;
  created_at: string;
}

interface EinvoiceSettings {
  id: number;
  company_id: number;
  environment: string;
  auto_submit: boolean;
  submit_on_post: boolean;
  tax_authority_endpoint: string | null;
  active: boolean;
}

function statusBadge(status: string) {
  const map: Record<string, { label: string; cls: string }> = {
    generated: { label: "تم الإنشاء", cls: "bg-blue-500/20 text-blue-400" },
    submitted: { label: "تم الإرسال", cls: "bg-amber-500/20 text-amber-400" },
    accepted: { label: "مقبول", cls: "bg-emerald-500/20 text-emerald-400" },
    rejected: { label: "مرفوض", cls: "bg-red-500/20 text-red-400" },
    cancelled: { label: "ملغي", cls: "bg-surface-500/20 text-surface-400" },
    pending: { label: "قيد الانتظار", cls: "bg-amber-500/20 text-amber-400" },
    completed: { label: "مكتمل", cls: "bg-emerald-500/20 text-emerald-400" },
    failed: { label: "فشل", cls: "bg-red-500/20 text-red-400" },
  };
  const m = map[status] || { label: status, cls: "bg-surface-500/20 text-surface-400" };
  return <span className={cn("inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium", m.cls)}>{m.label}</span>;
}

export default function EInvoicePage() {
  const addNotification = useUIStore((s) => s.addNotification);
  const [dashboard, setDashboard] = useState<DashboardData | null>(null);
  const [records, setRecords] = useState<EinvoiceRecord[]>([]);
  const [queue, setQueue] = useState<QueueItem[]>([]);
  const [settings, setSettings] = useState<EinvoiceSettings | null>(null);

  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [generating, setGenerating] = useState(false);
  const [result, setResult] = useState<EInvResult | null>(null);
  const [validation, setValidation] = useState<ValidationResult | null>(null);
  const [loading, setLoading] = useState(true);
  const [showSettings, setShowSettings] = useState(false);
  const [processing, setProcessing] = useState(false);
  const [statusFilter, setStatusFilter] = useState<string | null>(null);

  // Settings form
  const [sEnv, setSEnv] = useState("sandbox");
  const [sAuto, setSAuto] = useState(false);
  const [sOnPost, setSOnPost] = useState(false);
  const [sEndpoint, setSEndpoint] = useState("");
  const [sApiKey, setSApiKey] = useState("");
  const [sApiSecret, setSApiSecret] = useState("");
  const [saving, setSaving] = useState(false);

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const [dash, recs, q, set] = await Promise.all([
        invoke<DashboardData>("einvoice_get_dashboard"),
        invoke<EinvoiceRecord[]>("einvoice_list", { status: statusFilter }).catch(() => []),
        invoke<QueueItem[]>("einvoice_get_queue").catch(() => []),
        invoke<EinvoiceSettings | null>("einvoice_get_settings").catch(() => null),
      ]);
      setDashboard(dash);
      setRecords(recs);
      setQueue(q);
      setSettings(set);
      if (set) {
        setSEnv(set.environment);
        setSAuto(set.auto_submit);
        setSOnPost(set.submit_on_post);
        setSEndpoint(set.tax_authority_endpoint || "");
      }
    } catch {
      addNotification({ type: "error", title: "خطأ", message: "حدث خطأ أثناء تحميل البيانات" });
    }
    finally { setLoading(false); }
  }, [statusFilter]);

  useEffect(() => { loadData(); }, [loadData]);

  const handleGenerate = async () => {
    if (!selectedId) return;
    setGenerating(true);
    setResult(null);
    setValidation(null);
    try {
      const data = await invoke<EInvResult>("einvoice_generate", { invoiceId: selectedId });
      setResult(data);
      const valid = await invoke<ValidationResult>("einvoice_validate", { invoiceId: selectedId });
      setValidation(valid);
      loadData();
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء تحميل البيانات" }); }
    finally { setGenerating(false); }
  };

  const handleSubmit = async (id: number) => {
    setProcessing(true);
    try {
      await invoke("einvoice_submit", { invoiceId: id });
      loadData();
    } catch (err) { addNotification({ type: "error", title: "خطأ", message: String(err) }); }
    finally { setProcessing(false); }
  };

  const handleCancel = async (id: number) => {
    const reason = prompt("سبب الإلغاء:");
    if (!reason) return;
    try {
      await invoke("einvoice_cancel", { invoiceId: id, reason });
      loadData();
    } catch (err) { addNotification({ type: "error", title: "خطأ في الإلغاء", message: String(err) }); }
  };

  const handleQueueAdd = async (id: number) => {
    try {
      await invoke("einvoice_add_to_queue", { invoiceId: id });
      loadData();
    } catch (err) { addNotification({ type: "error", title: "خطأ", message: String(err) }); }
  };

  const handleProcessQueue = async () => {
    setProcessing(true);
    try {
      await invoke("einvoice_process_queue");
      loadData();
    } catch (err) { addNotification({ type: "error", title: "خطأ", message: String(err) }); }
    finally { setProcessing(false); }
  };

  const handleRetryQueue = async (qid: number) => {
    try {
      await invoke("einvoice_retry_queue_item", { queueId: qid });
      loadData();
    } catch (err) { addNotification({ type: "error", title: "خطأ", message: String(err) }); }
  };

  const handleSaveSettings = async () => {
    setSaving(true);
    try {
      await invoke("einvoice_save_settings", {
        environment: sEnv,
        autoSubmit: sAuto,
        submitOnPost: sOnPost,
        taxAuthorityEndpoint: sEndpoint || null,
        apiKey: sApiKey || null,
        apiSecret: sApiSecret || null,
        portalUsername: null,
        portalPassword: null,
      });
      loadData();
      setShowSettings(false);
    } catch (err) { addNotification({ type: "error", title: "خطأ في الحفظ", message: String(err) }); }
    finally { setSaving(false); }
  };

  const selected = records.find((r) => r.invoice_id === selectedId);

  if (loading) return (
    <div className="flex items-center justify-center h-64">
      <div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" />
    </div>
  );

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title flex items-center gap-2">
            <FileCheck className="w-6 h-6 text-gold-400" />
            الفوترة الإلكترونية
          </h1>
          <p className="page-subtitle">إنشاء وإدارة وإرسال الفواتير الإلكترونية لهيئة الزكاة والضريبة</p>
        </div>
        <div className="flex items-center gap-3">
          <Button variant="outline" icon={<RefreshCw className="w-4 h-4" />} onClick={loadData}>تحديث</Button>
          <Button variant={showSettings ? "primary" : "outline"} icon={<Settings2 className="w-4 h-4" />}
            onClick={() => setShowSettings(!showSettings)}>الإعدادات</Button>
        </div>
      </div>

      {/* Dashboard Stats */}
      {dashboard && (
        <div className="grid grid-cols-6 gap-4">
          <StatCard title="تم الإنشاء" value={dashboard.total_generated} icon={<FileCheck className="w-6 h-6" />} className="col-span-1" />
          <StatCard title="قيد الانتظار" value={dashboard.total_pending_submission} icon={<Clock className="w-6 h-6" />} className="col-span-1" />
          <StatCard title="تم الإرسال" value={dashboard.total_submitted} icon={<Send className="w-6 h-6" />} className="col-span-1" />
          <StatCard title="مقبول" value={dashboard.total_accepted} icon={<CheckCircle2 className="w-6 h-6" />} className="col-span-1" />
          <StatCard title="مرفوض" value={dashboard.total_rejected} icon={<XCircle className="w-6 h-6" />} className="col-span-1" />
          <StatCard title="المجموع" value={formatOMR(dashboard.total_amount_milli)} icon={<Hash className="w-6 h-6" />} className="col-span-1" />
        </div>
      )}

      {/* Settings Panel */}
      {showSettings && (
        <Card>
          <h3 className="section-title flex items-center gap-2 mb-4">
            <Settings2 className="w-5 h-5 text-gold-400" />
            إعدادات التكامل مع هيئة الزكاة والضريبة
          </h3>
          <div className="grid grid-cols-2 gap-4 mb-4">
            <div>
              <label className="form-label">البيئة</label>
              <select value={sEnv} onChange={(e) => setSEnv(e.target.value)} className="input-field" aria-label="البيئة">
                <option value="sandbox">Sandbox (اختبار)</option>
                <option value="production">إنتاج</option>
              </select>
            </div>
            <div>
              <label className="form-label">رابط API</label>
              <input type="text" value={sEndpoint} onChange={(e) => setSEndpoint(e.target.value)}
                className="input-field" placeholder="https://api.tax.gov.om/einvoicing" aria-label="رابط API" />
            </div>
            <div>
              <label className="form-label">مفتاح API</label>
              <input type="password" value={sApiKey} onChange={(e) => setSApiKey(e.target.value)}
                className="input-field" placeholder="أدخل مفتاح API" aria-label="مفتاح API" />
            </div>
            <div>
              <label className="form-label">Secret Key</label>
              <input type="password" value={sApiSecret} onChange={(e) => setSApiSecret(e.target.value)}
                className="input-field" placeholder="أدخل Secret Key" aria-label="المفتاح السري" />
            </div>
          </div>
          <div className="flex items-center gap-4 mb-4">
            <label className="flex items-center gap-2 text-sm text-surface-300 cursor-pointer">
              <input type="checkbox" checked={sAuto} onChange={(e) => setSAuto(e.target.checked)}
                className="w-4 h-4 rounded border-surface-600 bg-surface-800" aria-label="إرسال تلقائي للفواتير المعلقة" />
              إرسال تلقائي للفواتير المعلقة
            </label>
            <label className="flex items-center gap-2 text-sm text-surface-300 cursor-pointer">
              <input type="checkbox" checked={sOnPost} onChange={(e) => setSOnPost(e.target.checked)}
                className="w-4 h-4 rounded border-surface-600 bg-surface-800" aria-label="إرسال عند ترحيل الفاتورة" />
              إرسال عند ترحيل الفاتورة
            </label>
          </div>
          <div className="flex justify-end gap-3">
            <Button variant="outline" onClick={() => setShowSettings(false)}>إلغاء</Button>
            <Button icon={<Globe className="w-4 h-4" />} loading={saving} onClick={handleSaveSettings}>حفظ الإعدادات</Button>
          </div>
        </Card>
      )}

      <div className="grid grid-cols-3 gap-6">
        {/* Left: Invoice List */}
        <div className="col-span-2 space-y-6">
          <Card>
            <div className="flex items-center justify-between mb-4">
              <h3 className="section-title mb-0">الفواتير</h3>
              <div className="flex items-center gap-2">
                <select value={statusFilter || ""} onChange={(e) => setStatusFilter(e.target.value || null)}
                  className="input-field text-sm py-1.5 w-36" aria-label="تصفية الحالة">
                  <option value="">الكل</option>
                  <option value="generated">تم الإنشاء</option>
                  <option value="submitted">تم الإرسال</option>
                  <option value="accepted">مقبول</option>
                  <option value="rejected">مرفوض</option>
                  <option value="cancelled">ملغي</option>
                </select>
              </div>
            </div>
            <div className="max-h-80 overflow-y-auto space-y-2">
              {records.length === 0 ? (
                <p className="text-center text-surface-500 py-6 text-sm">لا توجد فواتير إلكترونية</p>
              ) : records.map((rec) => (
                <div key={rec.id}
                  onClick={() => { setSelectedId(rec.invoice_id); setResult(null); setValidation(null); }}
                  className={cn(
                    "flex items-center justify-between p-3 rounded-xl cursor-pointer transition-all",
                    selectedId === rec.invoice_id ? "bg-brand-800/30 border border-brand-500/40" : "bg-surface-800/50 hover:bg-surface-800 border border-transparent"
                  )}
                >
                  <div className="flex items-center gap-3 min-w-0">
                    <span className="font-mono text-brand-400 text-sm whitespace-nowrap">{rec.invoice_no}</span>
                    <span className="text-sm text-surface-300 truncate">{rec.customer_name}</span>
                  </div>
                  <div className="flex items-center gap-3 flex-shrink-0">
                    <span className="text-xs text-surface-500">{formatDate(rec.created_at)}</span>
                    <span className="text-sm font-medium">{formatOMR(rec.total_milli)}</span>
                    {statusBadge(rec.status)}
                  </div>
                </div>
              ))}
            </div>
            {selected && (
              <div className="mt-4 pt-4 border-t border-surface-700/50 flex items-center justify-between">
                <div className="flex items-center gap-2 text-sm text-surface-400">
                  <span className="font-mono text-brand-400">{selected.invoice_no}</span>
                  <span>- {selected.customer_name}</span>
                </div>
                <div className="flex items-center gap-2">
                  <Button size="sm" icon={<FileCheck className="w-3.5 h-3.5" />} onClick={handleGenerate} loading={generating}>
                    إنشاء
                  </Button>
                  {selected.status === "generated" && (
                    <>
                      <Button size="sm" variant="gold" icon={<Send className="w-3.5 h-3.5" />}
                        onClick={() => handleSubmit(selected.invoice_id)} loading={processing}>
                        إرسال
                      </Button>
                      <Button size="sm" variant="outline" icon={<ListOrdered className="w-3.5 h-3.5" />}
                        onClick={() => handleQueueAdd(selected.invoice_id)}>
                        أضف للطابور
                      </Button>
                    </>
                  )}
                  {(selected.status === "generated" || selected.status === "submitted" || selected.status === "pending") && (
                    <Button size="sm" variant="danger" icon={<Ban className="w-3.5 h-3.5" />}
                      onClick={() => handleCancel(selected.invoice_id)}>
                      إلغاء
                    </Button>
                  )}
                </div>
              </div>
            )}
          </Card>

          {/* Generated XML + Validation */}
          {result && (
            <div className="grid grid-cols-2 gap-6">
              <Card>
                <h3 className="section-title flex items-center gap-2 mb-4">
                  <Eye className="w-5 h-5 text-brand-400" />
                  معاينة XML
                </h3>
                <div className="bg-surface-900 rounded-xl p-4 max-h-72 overflow-auto">
                  <pre className="text-xs text-surface-300 font-mono whitespace-pre-wrap dir-ltr text-left" dir="ltr">{result.xml_content}</pre>
                </div>
                <div className="mt-3 flex items-center gap-2 text-xs text-surface-500">
                  <Hash className="w-3 h-3" />
                  {result.hash}
                </div>
              </Card>
              <Card>
                <h3 className="section-title mb-4">نتائج التحقق</h3>
                {validation && (
                  <>
                    <div className="text-center mb-4">
                      <div className={cn(
                        "inline-flex items-center justify-center w-20 h-20 rounded-full text-2xl font-bold",
                        validation.is_valid ? "bg-emerald-500/10 text-emerald-400" : "bg-red-500/10 text-red-400"
                      )}>
                        {validation.compliance_score}%
                      </div>
                      <p className="text-sm text-surface-400 mt-2">
                        {validation.is_valid ? "متوافق مع معايير الهيئة" : "يحتاج تصحيح"}
                      </p>
                    </div>
                    {[...validation.errors, ...validation.warnings].map((issue, i) => (
                      <div key={i} className={cn(
                        "flex items-center gap-2 p-2 rounded-lg text-sm mb-2",
                        issue.severity === "error" ? "bg-red-500/10 text-red-400" : "bg-amber-500/10 text-amber-400"
                      )}>
                        {issue.severity === "error" ? <XCircle className="w-4 h-4 flex-shrink-0" /> : <AlertTriangle className="w-4 h-4 flex-shrink-0" />}
                        {issue.message}
                      </div>
                    ))}
                    {validation.errors.length === 0 && validation.warnings.length === 0 && (
                      <p className="text-emerald-400 text-sm text-center">جميع الفحوصات ناجحة ✓</p>
                    )}
                  </>
                )}
              </Card>
            </div>
          )}
        </div>

        {/* Right: Queue + Info */}
        <div className="space-y-6">
          {/* Queue */}
          <Card>
            <div className="flex items-center justify-between mb-4">
              <h3 className="section-title mb-0 flex items-center gap-2">
                <ListOrdered className="w-5 h-5 text-gold-400" />
                طابور الإرسال
              </h3>
              <Button size="sm" icon={<Play className="w-3.5 h-3.5" />} onClick={handleProcessQueue} loading={processing}>
                معالجة
              </Button>
            </div>
            {dashboard && dashboard.queue_pending > 0 && (
              <div className="flex items-center gap-2 p-3 bg-amber-500/10 rounded-xl mb-3 text-sm text-amber-400">
                <Clock className="w-4 h-4" />
                {dashboard.queue_pending} فاتورة في انتظار الإرسال
              </div>
            )}
            {dashboard && dashboard.queue_failed > 0 && (
              <div className="flex items-center gap-2 p-3 bg-red-500/10 rounded-xl mb-3 text-sm text-red-400">
                <AlertTriangle className="w-4 h-4" />
                {dashboard.queue_failed} فشل في الإرسال
              </div>
            )}
            <div className="max-h-64 overflow-y-auto space-y-2">
              {queue.length === 0 ? (
                <p className="text-center text-surface-500 py-4 text-sm">الطابور فارغ</p>
              ) : queue.map((q) => (
                <div key={q.id} className="p-3 bg-surface-800/50 rounded-xl">
                  <div className="flex items-center justify-between mb-1">
                    <span className="text-sm font-mono text-brand-400">{q.invoice_no}</span>
                    {statusBadge(q.status)}
                  </div>
                  <p className="text-xs text-surface-500">{q.customer_name}</p>
                  {q.last_error && (
                    <p className="text-xs text-red-400 mt-1 truncate">{q.last_error}</p>
                  )}
                  <div className="flex items-center justify-between mt-2">
                    <span className="text-xs text-surface-500">محاولة {q.retry_count}/{q.max_retries}</span>
                    {q.status === "failed" && (
                      <button onClick={() => handleRetryQueue(q.id)} className="text-xs text-brand-400 hover:text-brand-300">
                        إعادة المحاولة
                      </button>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </Card>

          {/* Info */}
          <Card>
            <h3 className="section-title mb-3 flex items-center gap-2">
              <Server className="w-5 h-5 text-gold-400" />
              حالة التكامل
            </h3>
            <div className="space-y-3">
              <div className="flex items-center justify-between p-3 bg-surface-800/50 rounded-xl">
                <div className="flex items-center gap-2">
                  <Globe className="w-4 h-4 text-surface-400" />
                  <span className="text-sm text-surface-300">البيئة</span>
                </div>
                <span className={cn("text-sm", settings?.environment === "production" ? "text-emerald-400" : "text-amber-400")}>
                  {settings?.environment === "production" ? "إنتاج" : "اختبار"}
                </span>
              </div>
              <div className="flex items-center justify-between p-3 bg-surface-800/50 rounded-xl">
                <div className="flex items-center gap-2">
                  <Key className="w-4 h-4 text-surface-400" />
                  <span className="text-sm text-surface-300">API Key</span>
                </div>
                <span className={cn("text-sm", settings ? "text-emerald-400" : "text-red-400")}>
                  {settings ? "مكون" : "غير مكون"}
                </span>
              </div>
              <div className="flex items-center justify-between p-3 bg-surface-800/50 rounded-xl">
                <div className="flex items-center gap-2">
                  <Send className="w-4 h-4 text-surface-400" />
                  <span className="text-sm text-surface-300">الإرسال التلقائي</span>
                </div>
                <span className={cn("text-sm", settings?.auto_submit ? "text-emerald-400" : "text-surface-500")}>
                  {settings?.auto_submit ? "مفعل" : "معطل"}
                </span>
              </div>
            </div>
          </Card>
        </div>
      </div>
    </div>
  );
}
