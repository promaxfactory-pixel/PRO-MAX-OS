import { useState, useEffect } from "react";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { cn } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import {
  MessageSquare, Mail, Printer, Save, CheckCircle2,
  Send, Wifi, WifiOff
} from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

interface Settings {
  whatsapp_api_key: string;
  whatsapp_phone: string;
  smtp_server: string;
  smtp_port: number;
  smtp_user: string;
  smtp_pass: string;
  smtp_from: string;
  printer_name: string;
  printer_auto: boolean;
}

export default function IntegrationsPage() {
  const addNotification = useUIStore((s) => s.addNotification);
  const [form, setForm] = useState<Settings>({
    whatsapp_api_key: "", whatsapp_phone: "",
    smtp_server: "", smtp_port: 587, smtp_user: "", smtp_pass: "", smtp_from: "",
    printer_name: "", printer_auto: false,
  });
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);
  const [testing, setTesting] = useState<string | null>(null);
  const [testResult, setTestResult] = useState<{ type: string; ok: boolean; msg: string } | null>(null);

  useEffect(() => {
    invoke<Partial<Settings>>("integrations_get_settings")
      .then((d) => setForm((prev) => ({ ...prev, ...d })))
      .catch(console.error);
  }, []);

  const handleChange = (field: keyof Settings, value: any) => {
    setForm((prev) => ({ ...prev, [field]: value }));
    setSaved(false);
  };

  const handleSave = async () => {
    setSaving(true);
    try {
      await invoke("integrations_save_settings", { settings: form });
      setSaved(true);
      setTimeout(() => setSaved(false), 3000);
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء الحفظ" }); }
    finally { setSaving(false); }
  };

  const handleTest = async (type: string) => {
    setTesting(type);
    setTestResult(null);
    try {
      const result = await invoke<{ ok: boolean; message: string }>(`integrations_test_${type}`);
      setTestResult({ type, ok: result.ok, msg: result.message });
    } catch (err: any) {
      setTestResult({ type, ok: false, msg: err?.toString() || "فشل الاتصال" });
    } finally {
      setTesting(null);
      setTimeout(() => setTestResult(null), 5000);
    }
  };

  const field = (label: string, value: string | number, onChange: (v: string) => void, opts?: { type?: string; dir?: string }) => (
    <div>
      <label className="form-label">{label}</label>
      <input
        type={opts?.type || "text"}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="input-field"
        dir={opts?.dir || "ltr"}
      />
    </div>
  );

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">التكاملات</h1>
          <p className="page-subtitle">إعدادات واتساب والبريد والطابعة</p>
        </div>
        <Button icon={<Save className="w-4 h-4" />} onClick={handleSave} loading={saving}>
          {saved ? "تم الحفظ ✓" : "حفظ"}
        </Button>
      </div>

      {testResult && (
        <div className={cn("flex items-center gap-2 p-3 rounded-xl text-sm",
          testResult.ok ? "bg-emerald-500/10 text-emerald-400 border border-emerald-500/30" : "bg-red-500/10 text-red-400 border border-red-500/30"
        )}>
          {testResult.ok ? <CheckCircle2 className="w-4 h-4" /> : <WifiOff className="w-4 h-4" />}
          {testResult.msg}
        </div>
      )}

      <div className="grid grid-cols-3 gap-6">
        <Card>
          <div className="flex items-center justify-between mb-4">
            <h3 className="section-title flex items-center gap-2">
              <MessageSquare className="w-5 h-5 text-emerald-400" />
              واتساب
            </h3>
            <Button variant="ghost" size="sm" icon={<Send className="w-3.5 h-3.5" />} onClick={() => handleTest("whatsapp")} loading={testing === "whatsapp"}>
              اختبار
            </Button>
          </div>
          <div className="space-y-4">
            {field("API Key", form.whatsapp_api_key, (v) => handleChange("whatsapp_api_key", v))}
            {field("رقم الهاتف", form.whatsapp_phone, (v) => handleChange("whatsapp_phone", v))}
          </div>
        </Card>

        <Card>
          <div className="flex items-center justify-between mb-4">
            <h3 className="section-title flex items-center gap-2">
              <Mail className="w-5 h-5 text-blue-400" />
              البريد الإلكتروني (SMTP)
            </h3>
            <Button variant="ghost" size="sm" icon={<Send className="w-3.5 h-3.5" />} onClick={() => handleTest("email")} loading={testing === "email"}>
              اختبار
            </Button>
          </div>
          <div className="space-y-4">
            <div className="grid grid-cols-3 gap-4">
              {field("الخادم", form.smtp_server, (v) => handleChange("smtp_server", v))}
              {field("المنفذ", form.smtp_port, (v) => handleChange("smtp_port", Number(v)), { type: "number" })}
              <div />
            </div>
            {field("المستخدم", form.smtp_user, (v) => handleChange("smtp_user", v))}
            {field("كلمة المرور", form.smtp_pass, (v) => handleChange("smtp_pass", v), { type: "password" })}
            {field("من", form.smtp_from, (v) => handleChange("smtp_from", v))}
          </div>
        </Card>

        <Card>
          <div className="flex items-center justify-between mb-4">
            <h3 className="section-title flex items-center gap-2">
              <Printer className="w-5 h-5 text-gold-400" />
              الطابعة
            </h3>
            <Button variant="ghost" size="sm" icon={<Send className="w-3.5 h-3.5" />} onClick={() => handleTest("printer")} loading={testing === "printer"}>
              اختبار
            </Button>
          </div>
          <div className="space-y-4">
            <div>
              <label className="form-label">اسم الطابعة</label>
              <input type="text" value={form.printer_name} onChange={(e) => handleChange("printer_name", e.target.value)} className="input-field" />
            </div>
            <label className="flex items-center gap-3 cursor-pointer">
              <input
                type="checkbox"
                checked={form.printer_auto}
                onChange={(e) => handleChange("printer_auto", e.target.checked)}
                className="rounded"
              />
              <span className="text-sm text-surface-300">طباعة تلقائية</span>
            </label>
          </div>
        </Card>
      </div>
    </div>
  );
}
