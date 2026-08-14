import { useState, useEffect, useCallback } from "react";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { Input, Select, Textarea } from "@/components/ui/Input";
import { formatDateTime, cn } from "@/lib/utils";
import { invoke } from "@/lib/tauri";
import { useUIStore } from "@/stores/uiStore";
import {
  ShieldCheck, FileKey2, Send, CheckCircle2, XCircle, Save,
  RefreshCw, Globe, Server, Loader2,
  FileCheck, Eye, AlertTriangle, BadgeCheck
} from "lucide-react";

interface Zatca2Settings {
  id: number;
  environment: string;
  vat_number: string | null;
  organization_name: string | null;
  csid_stage: string;
  icv_counter: number;
  last_invoice_hash: string | null;
  onboarded: boolean;
}

interface Zatca2Record {
  id: number;
  invoice_id: number;
  invoice_no: string;
  status: string;
  zatca_stage: string | null;
  invoice_hash: string | null;
  icv: number | null;
  submitted_at: string | null;
  created_at: string;
}

interface Zatca2Generated {
  e_invoice_id: number;
  invoice_no: string;
  invoice_hash: string;
  qr_payload: string;
  signature_value: string;
  icv: number;
  pih: string | null;
  xml: string;
  status: string;
}

interface Zatca2Validation {
  is_valid: boolean;
  errors: string[];
  warnings: string[];
  compliance_score: number;
}

interface Zatca2SubmitResult {
  e_invoice_id: number;
  invoice_no: string;
  status: string;
  zatca_uuid: string | null;
  message: string;
}

interface Zatca2OnboardResult {
  stage: string;
  request_id: string | null;
  certificate_der: string | null;
  message: string;
}

interface SalesInvoiceLite {
  id: number;
  inv_no: string | null;
  date: string;
  customer_name: string | null;
  net_milli: number;
  vat_milli: number;
  total_milli: number;
  status: string;
}

const stageMeta: Record<string, { label: string; cls: string }> = {
  none: { label: "غير مسجل", cls: "badge-info" },
  compliance: { label: "شهادة امتثال", cls: "badge-warning" },
  production: { label: "شهادة إنتاج", cls: "badge-success" },
  simplified: { label: "مبسطة", cls: "badge-info" },
};

const statusMeta: Record<string, { label: string; cls: string }> = {
  Draft: { label: "مسودة", cls: "badge-info" },
  Generated: { label: "مولدة", cls: "badge-warning" },
  Validated: { label: "متحققة", cls: "badge-success" },
  Cleared: { label: "مجدازة", cls: "badge-success" },
  Reported: { label: "مبلغة", cls: "badge-success" },
  Rejected: { label: "مرفوضة", cls: "badge-danger" },
};

export default function Zatca2Page() {
  const addNotification = useUIStore((s) => s.addNotification);
  const [settings, setSettings] = useState<Zatca2Settings | null>(null);
  const [env, setEnv] = useState("sandbox");
  const [vat, setVat] = useState("");
  const [org, setOrg] = useState("");
  const [records, setRecords] = useState<Zatca2Record[]>([]);
  const [invoices, setInvoices] = useState<SalesInvoiceLite[]>([]);
  const [invoiceId, setInvoiceId] = useState<number | "">("");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [buildingCsr, setBuildingCsr] = useState(false);
  const [onboarding, setOnboarding] = useState<"sandbox" | "production" | null>(null);
  const [onboardResult, setOnboardResult] = useState<Zatca2OnboardResult | null>(null);
  const [generated, setGenerated] = useState<Zatca2Generated | null>(null);
  const [validation, setValidation] = useState<Zatca2Validation | null>(null);
  const [submitResult, setSubmitResult] = useState<Zatca2SubmitResult | null>(null);
  const [busyAction, setBusyAction] = useState<string | null>(null);

  const load = useCallback(() => {
    Promise.all([
      invoke<Zatca2Settings | null>("zatca2_get_settings"),
      invoke<Zatca2Record[]>("zatca2_list"),
      invoke<SalesInvoiceLite[]>("list_invoices"),
    ])
      .then(([s, r, inv]) => {
        setSettings(s);
        if (s) {
          setEnv(s.environment);
          setVat(s.vat_number || "");
          setOrg(s.organization_name || "");
        }
        setRecords(r);
        setInvoices(inv.filter((i) => i.status !== "Void" && i.status !== "Cancelled"));
      })
      .catch((e: unknown) => addNotification({ title: "خطأ", message: String(e), type: "error" }))
      .finally(() => setLoading(false));
  }, [addNotification]);

  useEffect(() => { load(); }, [load]);

  const handleSaveSettings = async () => {
    setSaving(true);
    try {
      const msg = await invoke<string>("zatca2_save_settings", {
        environment: env,
        vatNumber: vat.trim() || null,
        organizationName: org.trim() || null,
      });
      addNotification({ title: "تم الحفظ", message: msg, type: "success" });
      load();
    } catch (e) {
      addNotification({ title: "خطأ", message: String(e), type: "error" });
    } finally {
      setSaving(false);
    }
  };

  const handleBuildCsr = async () => {
    setBuildingCsr(true);
    try {
      const csr = await invoke<string>("zatca2_build_csr");
      await navigator.clipboard.writeText(csr);
      addNotification({ title: "CSR", message: "تم إنشاء ونسخ ملف CSR إلى الحافظة", type: "success" });
    } catch (e) {
      addNotification({ title: "خطأ", message: String(e), type: "error" });
    } finally {
      setBuildingCsr(false);
    }
  };

  const handleOnboard = async (sandbox: boolean) => {
    setOnboarding(sandbox ? "sandbox" : "production");
    setOnboardResult(null);
    try {
      const res = await invoke<Zatca2OnboardResult>("zatca2_onboard", { sandbox });
      setOnboardResult(res);
      addNotification({ title: "التسجيل", message: res.message, type: res.stage === "production" ? "success" : "warning" });
      load();
    } catch (e) {
      addNotification({ title: "خطأ", message: String(e), type: "error" });
    } finally {
      setOnboarding(null);
    }
  };

  const runAction = async (action: string, fn: () => Promise<void>) => {
    setBusyAction(action);
    setGenerated(null);
    setValidation(null);
    setSubmitResult(null);
    try {
      await fn();
    } catch (e) {
      addNotification({ title: "خطأ", message: String(e), type: "error" });
    } finally {
      setBusyAction(null);
    }
  };

  const handleGenerate = () =>
    runAction("generate", async () => {
      if (invoiceId === "") throw new Error("اختر فاتورة أولاً");
      const res = await invoke<Zatca2Generated>("zatca2_generate", { invoiceId });
      setGenerated(res);
      addNotification({ title: "توليد", message: `تم توليد XML للفاتورة ${res.invoice_no}`, type: "success" });
      load();
    });

  const handleValidate = () =>
    runAction("validate", async () => {
      if (invoiceId === "") throw new Error("اختر فاتورة أولاً");
      const res = await invoke<Zatca2Validation>("zatca2_validate", { invoiceId });
      setValidation(res);
      addNotification({
        title: "التحقق",
        message: res.is_valid ? `مطابقة (${res.compliance_score}%)` : `فشل: ${res.errors.join("، ") || "أخطاء تحقق"}`,
        type: res.is_valid ? "success" : "error",
      });
    });

  const handleSubmit = (sandbox: boolean) =>
    runAction("submit", async () => {
      if (!generated) throw new Error("قم بتوليد الفاتورة أولاً");
      const res = await invoke<Zatca2SubmitResult>("zatca2_submit", { eInvoiceId: generated.e_invoice_id, sandbox });
      setSubmitResult(res);
      addNotification({ title: "الإرسال", message: res.message, type: "success" });
      load();
    });

  const stage = settings ? stageMeta[settings.csid_stage] || stageMeta.none : stageMeta.none;
  const badge = (s: string) => {
    const m = statusMeta[s] || { label: s, cls: "badge-info" };
    return <span className={m.cls}>{m.label}</span>;
  };

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title flex items-center gap-2">
            <ShieldCheck className="w-6 h-6 text-gold-400" />
            الفوترة الإلكترونية السعودية - المرحلة الثانية
          </h1>
          <p className="page-subtitle">ZATCA Phase 2 — فواتير UBL 2.1، توقيع ECDSA، QR بمرحلتين، إجازة وإبلاغ فاتورة</p>
        </div>
        {settings && (
          <div className="flex items-center gap-2">
            <span className={cn("text-xs px-3 py-1.5 rounded-full border", stage.cls)}>
              {stage.label}
            </span>
            <span className="text-xs text-surface-500">العدّاد ICV: {settings.icv_counter}</span>
          </div>
        )}
      </div>

      <div className="grid grid-cols-2 gap-6">
        <Card>
          <h3 className="section-title mb-4">إعدادات المنشأة</h3>
          <div className="space-y-4">
            <Select
              label="البيئة"
              options={[
                { value: "sandbox", label: "تجريبية (Sandbox)" },
                { value: "production", label: "إنتاجية (Production)" },
                { value: "simplified", label: "مبسطة (Simplified)" },
              ]}
              value={env}
              onChange={(e) => setEnv(e.target.value)}
            />
            <Input
              label="الرقم الضريبي (VAT Number)"
              placeholder="3xxxxxxxxxxxx3"
              value={vat}
              onChange={(e) => setVat(e.target.value)}
              icon={<Globe className="w-4 h-4" />}
            />
            <Input
              label="اسم المنشأة"
              placeholder="الاسم التجاري"
              value={org}
              onChange={(e) => setOrg(e.target.value)}
            />
            <div className="flex gap-2">
              <Button onClick={handleSaveSettings} loading={saving} icon={<Save className="w-4 h-4" />}>
                حفظ الإعدادات
              </Button>
              <Button variant="outline" onClick={handleBuildCsr} loading={buildingCsr} icon={<FileKey2 className="w-4 h-4" />}>
                إنشاء ونسخ CSR
              </Button>
            </div>
            <p className="text-xs text-surface-500 leading-relaxed">
              عند الحفظ يتم توليد مفتاح ECDSA (secp256k1) وتشفيره محلياً. ملف CSR يُرسل إلى هيئة ZATCA للتحقق ثم تحصل على
              شهادة CSID للسماح بالإجازة الآلية.
            </p>
          </div>
        </Card>

        <Card>
          <h3 className="section-title mb-4">التسجيل في فاتورة (CSID)</h3>
          <div className="space-y-4">
            <div className="grid grid-cols-3 gap-3">
              <div className="p-3 bg-surface-800/50 rounded-xl text-center">
                <p className="text-2xl font-bold text-gold-400">{settings?.onboarded ? "نشط" : "غير مفعل"}</p>
                <p className="text-xs text-surface-500 mt-1">حالة التسجيل</p>
              </div>
              <div className="p-3 bg-surface-800/50 rounded-xl text-center">
                <p className="text-2xl font-bold text-white">{settings?.icv_counter ?? 0}</p>
                <p className="text-xs text-surface-500 mt-1">عداد الإصدار (ICV)</p>
              </div>
              <div className="p-3 bg-surface-800/50 rounded-xl text-center">
                <p className="text-2xl font-bold text-white">{settings?.csid_stage ?? "none"}</p>
                <p className="text-xs text-surface-500 mt-1">مرحلة CSID</p>
              </div>
            </div>
            <div className="flex gap-2">
              <Button
                variant="success"
                onClick={() => handleOnboard(true)}
                loading={onboarding === "sandbox"}
                disabled={onboarding !== null}
                icon={<Server className="w-4 h-4" />}
              >
                تسجيل تلقائي (Sandbox)
              </Button>
              <Button
                variant="gold"
                onClick={() => handleOnboard(false)}
                loading={onboarding === "production"}
                disabled={onboarding !== null}
                icon={<Globe className="w-4 h-4" />}
              >
                تسجيل تلقائي (Production)
              </Button>
            </div>
            {onboardResult && (
              <div className={cn("p-3 rounded-xl text-sm border",
                onboardResult.stage === "production"
                  ? "bg-emerald-500/10 text-emerald-400 border-emerald-500/30"
                  : "bg-amber-500/10 text-amber-400 border-amber-500/30")}>
                <div className="flex items-start gap-2">
                  {onboardResult.stage === "production"
                    ? <BadgeCheck className="w-4 h-4 mt-0.5 flex-shrink-0" />
                    : <AlertTriangle className="w-4 h-4 mt-0.5 flex-shrink-0" />}
                  <div className="space-y-1">
                    <p className="font-medium">{onboardResult.message}</p>
                    {onboardResult.request_id && <p className="text-xs opacity-80">RequestID: {onboardResult.request_id}</p>}
                    {onboardResult.certificate_der && (
                      <p className="text-xs opacity-60 break-all">Cert: {onboardResult.certificate_der.slice(0, 60)}…</p>
                    )}
                  </div>
                </div>
              </div>
            )}
            <p className="text-xs text-surface-500 leading-relaxed">
              التسجيل التلقائي يطلب شهادة الامتثال ثم شهادة الإنتاج تلقائياً عبر واجهات فاتورة. للتسجيل اليدوي استخدم زر
              نسخ CSR وأرسله إلى منصة فاتورة ثم عدّل مرحلة الشهادة يدوياً.
            </p>
          </div>
        </Card>
      </div>

      <Card>
        <div className="flex items-center justify-between mb-4">
          <h3 className="section-title">توليد وتحقق وإرسال الفواتير</h3>
          <Button variant="ghost" size="sm" onClick={load} icon={<RefreshCw className="w-4 h-4" />}>
            تحديث
          </Button>
        </div>
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
          <Select
            label="الفاتورة"
            placeholder="اختر فاتورة بيع…"
            options={invoices.map((i) => ({
              value: i.id,
              label: `${i.inv_no || `#${i.id}`} — ${i.customer_name || "زبون"} (${(i.total_milli / 1000).toFixed(3)})`,
            }))}
            value={invoiceId}
            onChange={(e) => { setInvoiceId(e.target.value === "" ? "" : Number(e.target.value)); }}
          />
          <div className="flex items-end gap-2">
            <Button
              onClick={handleGenerate}
              loading={busyAction === "generate"}
              disabled={invoiceId === ""}
              icon={<FileCheck className="w-4 h-4" />}
            >
              توليد XML
            </Button>
            <Button
              variant="outline"
              onClick={handleValidate}
              loading={busyAction === "validate"}
              disabled={invoiceId === ""}
              icon={<Eye className="w-4 h-4" />}
            >
              تحقق
            </Button>
            <Button
              variant="success"
              onClick={() => handleSubmit(true)}
              loading={busyAction === "submit"}
              disabled={!generated}
              icon={<Send className="w-4 h-4" />}
            >
              إرسال Sandbox
            </Button>
            <Button
              variant="gold"
              onClick={() => handleSubmit(false)}
              loading={busyAction === "submit"}
              disabled={!generated}
              icon={<Send className="w-4 h-4" />}
            >
              إرسال Production
            </Button>
          </div>
        </div>

        {generated && (
          <div className="mt-5 grid grid-cols-4 gap-3">
            <div className="p-3 bg-surface-800/50 rounded-xl">
              <p className="text-[10px] text-surface-500">رقم الفاتورة</p>
              <p className="text-sm font-bold text-white mt-1">{generated.invoice_no}</p>
            </div>
            <div className="p-3 bg-surface-800/50 rounded-xl">
              <p className="text-[10px] text-surface-500">ICV</p>
              <p className="text-sm font-bold text-white mt-1" dir="ltr">{generated.icv}</p>
            </div>
            <div className="p-3 bg-surface-800/50 rounded-xl">
              <p className="text-[10px] text-surface-500">PIH (التجزئة السابقة)</p>
              <p className="text-xs font-mono text-surface-300 mt-1 break-all" dir="ltr">{generated.pih || "—"}</p>
            </div>
            <div className="p-3 bg-surface-800/50 rounded-xl">
              <p className="text-[10px] text-surface-500">تجزئة الفاتورة (Hash)</p>
              <p className="text-xs font-mono text-surface-300 mt-1 break-all" dir="ltr">{generated.invoice_hash}</p>
            </div>
          </div>
        )}

        {validation && (
          <div className={cn("mt-4 p-4 rounded-xl border",
            validation.is_valid ? "bg-emerald-500/10 border-emerald-500/30" : "bg-red-500/10 border-red-500/30")}>
            <div className="flex items-center gap-2">
              {validation.is_valid ? <CheckCircle2 className="w-5 h-5 text-emerald-400" /> : <XCircle className="w-5 h-5 text-red-400" />}
              <p className="font-semibold">
                {validation.is_valid ? `مطابقة للمتطلبات — درجة ${validation.compliance_score}%` : "توجد أخطاء يجب معالجتها"}
              </p>
            </div>
            {validation.errors.length > 0 && (
              <ul className="mt-2 space-y-1">
                {validation.errors.map((er, i) => <li key={i} className="text-sm text-red-300 list-disc mr-5">• {er}</li>)}
              </ul>
            )}
            {validation.warnings.length > 0 && (
              <ul className="mt-2 space-y-1">
                {validation.warnings.map((w, i) => <li key={i} className="text-sm text-amber-300 list-disc mr-5">• {w}</li>)}
              </ul>
            )}
          </div>
        )}

        {submitResult && (
          <div className="mt-4 p-4 bg-emerald-500/10 border border-emerald-500/30 rounded-xl">
            <div className="flex items-center gap-2">
              <BadgeCheck className="w-5 h-5 text-emerald-400" />
              <p className="font-semibold text-emerald-300">{submitResult.message} — {submitResult.status}</p>
            </div>
            {submitResult.zatca_uuid && <p className="text-xs text-surface-400 mt-1 font-mono" dir="ltr">UUID: {submitResult.zatca_uuid}</p>}
          </div>
        )}

        {generated && (
          <div className="mt-4">
            <p className="text-xs text-surface-500 mb-2">معاينة XML مولدة</p>
            <Textarea readOnly value={generated.xml} className="min-h-[220px] font-mono text-xs" />
          </div>
        )}
      </Card>

      <Card>
        <h3 className="section-title mb-4">سجل فواتير المرحلة الثانية</h3>
        {loading ? (
          <div className="flex justify-center py-10">
            <Loader2 className="w-8 h-8 animate-spin text-gold-400" />
          </div>
        ) : records.length === 0 ? (
          <p className="text-center text-surface-500 py-8">لا توجد فواتير مسجلة بعد. قم بتوليد أول فاتورة.</p>
        ) : (
          <div className="overflow-x-auto">
            <table className="table w-full text-sm">
              <thead>
                <tr className="text-xs text-surface-500">
                  <th className="text-right">رقم الفاتورة</th>
                  <th className="text-right">الحالة</th>
                  <th className="text-right">المرحلة</th>
                  <th className="text-right">ICV</th>
                  <th className="text-right">التجزئة</th>
                  <th className="text-right">تاريخ الإرسال</th>
                </tr>
              </thead>
              <tbody>
                {records.map((r) => (
                  <tr key={r.id} className="border-t border-surface-800">
                    <td className="py-2.5 font-medium text-white">{r.invoice_no}</td>
                    <td>{badge(r.status)}</td>
                    <td className="text-surface-400">{r.zatca_stage || "—"}</td>
                    <td className="font-mono" dir="ltr">{r.icv ?? "—"}</td>
                    <td className="font-mono text-xs text-surface-400 max-w-[180px] truncate" dir="ltr">{r.invoice_hash || "—"}</td>
                    <td className="text-xs text-surface-400">{r.submitted_at ? formatDateTime(r.submitted_at) : "—"}</td>
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
