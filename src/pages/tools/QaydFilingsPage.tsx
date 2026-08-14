import { useState, useEffect, useCallback } from "react";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { Input, Textarea } from "@/components/ui/Input";
import Modal from "@/components/ui/Modal";
import { formatDateTime, cn } from "@/lib/utils";
import { invoke } from "@/lib/tauri";
import { useUIStore } from "@/stores/uiStore";
import {
  FileBarChart2, Play, Eye, Trash2, CheckCircle2, XCircle, Loader2,
  RefreshCw, ShieldCheck, ListOrdered, Hash
} from "lucide-react";

interface QaydFilingResult {
  id: number;
  fiscal_year: number;
  cr_number: string;
  currency: string;
  status: string;
  instance_xml: string;
  validation_report: string[];
  is_valid: boolean;
}

interface QaydFilingRecord {
  id: number;
  fiscal_year: number;
  currency: string;
  cr_number: string | null;
  status: string;
  submitted_at: string | null;
  created_at: string;
}

const statusMeta: Record<string, { label: string; cls: string }> = {
  draft: { label: "مسودة", cls: "badge-info" },
  ready: { label: "جاهز للإيداع", cls: "badge-success" },
  submitted: { label: "مُودَع", cls: "badge-warning" },
  rejected: { label: "مرفوض", cls: "badge-danger" },
};

export default function QaydFilingsPage() {
  const addNotification = useUIStore((s) => s.addNotification);
  const [filings, setFilings] = useState<QaydFilingRecord[]>([]);
  const [fiscalYear, setFiscalYear] = useState(String(new Date().getFullYear() - 1));
  const [loading, setLoading] = useState(true);
  const [generating, setGenerating] = useState(false);
  const [validatingId, setValidatingId] = useState<number | null>(null);
  const [deletingId, setDeletingId] = useState<number | null>(null);
  const [viewing, setViewing] = useState<QaydFilingResult | null>(null);
  const [viewingTotals, setViewingTotals] = useState<Record<string, number | null> | null>(null);

  const load = useCallback(() => {
    setLoading(true);
    invoke<QaydFilingRecord[]>("qayd_list_filings")
      .then(setFilings)
      .catch((e: unknown) => addNotification({ title: "خطأ", message: String(e), type: "error" }))
      .finally(() => setLoading(false));
  }, [addNotification]);

  useEffect(() => { load(); }, [load]);

  const handleGenerate = async () => {
    const year = Number(fiscalYear);
    if (!Number.isInteger(year) || year < 2000 || year > 2100) {
      addNotification({ title: "خطأ", message: "سنة مالية غير صالحة", type: "error" });
      return;
    }
    setGenerating(true);
    try {
      const res = await invoke<QaydFilingResult>("qayd_generate_filing", { fiscalYear: year });
      addNotification({
        title: "تم توليد الإيداع",
        message: `إيداع XBRL للسنة ${res.fiscal_year} (${res.currency}) — ${res.validation_report.length} ملاحظة تحقق`,
        type: res.is_valid ? "success" : "warning",
      });
      load();
    } catch (e) {
      addNotification({ title: "خطأ", message: String(e), type: "error" });
    } finally {
      setGenerating(false);
    }
  };

  const handleValidate = async (id: number) => {
    setValidatingId(id);
    try {
      const res = await invoke<QaydFilingResult>("qayd_validate_filing", { filingId: id });
      addNotification({
        title: "التحقق",
        message: res.is_valid ? "الإيداع مطابق للمتطلبات" : `ملاحظات: ${res.validation_report.join("، ") || "توجد أخطاء"}`,
        type: res.is_valid ? "success" : "error",
      });
      load();
    } catch (e) {
      addNotification({ title: "خطأ", message: String(e), type: "error" });
    } finally {
      setValidatingId(null);
    }
  };

  const handleView = async (id: number) => {
    try {
      const res = await invoke<QaydFilingResult>("qayd_get_filing", { filingId: id });
      setViewing(res);
      const totals = await invoke<Record<string, number | null>>("qayd_filing_totals", { filingId: id });
      setViewingTotals(totals);
    } catch (e) {
      addNotification({ title: "خطأ", message: String(e), type: "error" });
    }
  };

  const handleDelete = async (id: number) => {
    setDeletingId(id);
    try {
      await invoke<string>("qayd_delete_filing", { filingId: id });
      addNotification({ title: "حذف", message: "تم حذف الإيداع", type: "success" });
      load();
    } catch (e) {
      addNotification({ title: "خطأ", message: String(e), type: "error" });
    } finally {
      setDeletingId(null);
    }
  };

  const badge = (s: string) => {
    const m = statusMeta[s] || { label: s, cls: "badge-info" };
    return <span className={m.cls}>{m.label}</span>;
  };

  const totalsRows: { key: string; label: string }[] = [
    { key: "Assets", label: "الأصول" },
    { key: "Liabilities", label: "الخصوم" },
    { key: "Equity", label: "حقوق الملكية" },
    { key: "Revenue", label: "الإيرادات" },
    { key: "ProfitLoss", label: "الأرباح والخسائر" },
  ];

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title flex items-center gap-2">
            <FileBarChart2 className="w-6 h-6 text-gold-400" />
            قيد — إيداع القوائم المالية (XBRL)
          </h1>
          <p className="page-subtitle">MOICI Qayd — بيانات قوائم مالية سنوية (IFRS-Full) بالدينار الكويتي</p>
        </div>
        <Button variant="ghost" size="sm" onClick={load} icon={<RefreshCw className="w-4 h-4" />}>
          تحديث
        </Button>
      </div>

      <Card>
        <h3 className="section-title mb-4">توليد إيداع جديد</h3>
        <div className="flex items-end gap-3">
          <div className="w-56">
            <Input
              label="السنة المالية"
              type="number"
              min={2000}
              max={2100}
              value={fiscalYear}
              onChange={(e) => setFiscalYear(e.target.value)}
              icon={<ListOrdered className="w-4 h-4" />}
            />
          </div>
          <Button onClick={handleGenerate} loading={generating} icon={<Play className="w-4 h-4" />}>
            توليد إيداع XBRL
          </Button>
        </div>
        <p className="text-xs text-surface-500 mt-3 leading-relaxed">
          يتم قراءة الأرصدة الختامية وبيان الدخل من دفتر الأستاذ، وبناء نموذج XBRL متوافق مع تصنيف قيد (moici-ifrs) بتنسيق
          كامل، مع تسوية الميزانية. الاستخدام إلزامي للشركات الكويتية اعتباراً من 2027-01-01.
        </p>
      </Card>

      <Card>
        <h3 className="section-title mb-4">الإيداعات</h3>
        {loading ? (
          <div className="flex justify-center py-10">
            <Loader2 className="w-8 h-8 animate-spin text-gold-400" />
          </div>
        ) : filings.length === 0 ? (
          <p className="text-center text-surface-500 py-8">لا توجد إيداعات بعد.</p>
        ) : (
          <div className="overflow-x-auto">
            <table className="table w-full text-sm">
              <thead>
                <tr className="text-xs text-surface-500">
                  <th className="text-right">السنة</th>
                  <th className="text-right">العملة</th>
                  <th className="text-right">رقم السجل التجاري</th>
                  <th className="text-right">الحالة</th>
                  <th className="text-right">تاريخ الإنشاء</th>
                  <th className="text-right">تاريخ الإيداع</th>
                  <th className="text-right">إجراءات</th>
                </tr>
              </thead>
              <tbody>
                {filings.map((f) => (
                  <tr key={f.id} className="border-t border-surface-800">
                    <td className="py-2.5 font-bold text-white">{f.fiscal_year}</td>
                    <td className="font-mono text-xs" dir="ltr">{f.currency}</td>
                    <td className="font-mono text-xs" dir="ltr">{f.cr_number || "—"}</td>
                    <td>{badge(f.status)}</td>
                    <td className="text-xs text-surface-400">{formatDateTime(f.created_at)}</td>
                    <td className="text-xs text-surface-400">{f.submitted_at ? formatDateTime(f.submitted_at) : "—"}</td>
                    <td>
                      <div className="flex items-center gap-1.5">
                        <Button variant="ghost" size="sm" onClick={() => handleView(f.id)} icon={<Eye className="w-4 h-4" />}>
                          عرض
                        </Button>
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => handleValidate(f.id)}
                          loading={validatingId === f.id}
                          icon={<CheckCircle2 className="w-4 h-4" />}
                        >
                          تحقق
                        </Button>
                        <Button
                          variant="danger"
                          size="sm"
                          onClick={() => handleDelete(f.id)}
                          loading={deletingId === f.id}
                          icon={<Trash2 className="w-4 h-4" />}
                        >
                          حذف
                        </Button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </Card>

      <Modal
        open={!!viewing}
        onClose={() => setViewing(null)}
        title={viewing ? `إيداع ${viewing.fiscal_year} — عرض` : ""}
      >
        {viewing && (
          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-3">
              {totalsRows.map((tr) => (
                <div key={tr.key} className="p-3 bg-surface-800/50 rounded-xl">
                  <p className="text-[10px] text-surface-500">{tr.label}</p>
                  <p className="text-lg font-bold text-white mt-1" dir="ltr">
                    {viewingTotals?.[tr.key] != null ? Number(viewingTotals[tr.key]).toFixed(3) : "—"}
                  </p>
                </div>
              ))}
            </div>

            <div className="flex items-center gap-2">
              <ShieldCheck className={cn("w-5 h-5", viewing.is_valid ? "text-emerald-400" : "text-red-400")} />
              <span className={cn("font-semibold", viewing.is_valid ? "text-emerald-400" : "text-red-400")}>
                {viewing.is_valid ? "متوافق مع التصنيف" : "توجد ملاحظات تحقق"}
              </span>
            </div>
            {viewing.validation_report.length > 0 && (
              <ul className="space-y-1">
                {viewing.validation_report.map((v, i) => (
                  <li key={i} className={cn("text-sm flex items-start gap-1.5", v.startsWith("ERROR") ? "text-red-300" : "text-amber-300")}>
                    {v.startsWith("ERROR") ? <XCircle className="w-4 h-4 mt-0.5 flex-shrink-0" /> : <CheckCircle2 className="w-4 h-4 mt-0.5 flex-shrink-0" />}
                    {v}
                  </li>
                ))}
              </ul>
            )}

            <div>
              <p className="text-xs text-surface-500 mb-2 flex items-center gap-1.5">
                <Hash className="w-3.5 h-3.5" /> نموذج XBRL (instance.xml)
              </p>
              <Textarea readOnly value={viewing.instance_xml} className="min-h-[300px] font-mono text-xs" />
            </div>
          </div>
        )}
      </Modal>
    </div>
  );
}
