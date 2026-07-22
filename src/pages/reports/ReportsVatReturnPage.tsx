import { useState, useEffect } from "react";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, Printer, Download } from "lucide-react";
import { useNavigate } from "react-router-dom";

export default function ReportsVatReturnPage() {
  const navigate = useNavigate();
  const [data, setData] = useState<any>(null);
  const [loading, setLoading] = useState(true);
  const [period, setPeriod] = useState(() => {
    const now = new Date();
    return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}`;
  });

  const fetchData = () => {
    setLoading(true);
    invoke("vat_return", { month: period })
      .then((d: any) => setData(d))
      .catch(console.error)
      .finally(() => setLoading(false));
  };

  useEffect(() => { fetchData(); }, [period]);

  if (loading) return (
    <div className="flex items-center justify-center h-64">
      <div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" />
    </div>
  );

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div className="flex items-center gap-3">
          <button onClick={() => navigate("/reports")} className="btn-ghost p-2">
            <ArrowRight className="w-5 h-5" />
          </button>
          <div>
            <h1 className="page-title">إقرار ضريبة القيمة المضافة</h1>
            <p className="page-subtitle">لفترة {period}</p>
          </div>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" icon={<Printer className="w-4 h-4" />}>طباعة</Button>
          <Button variant="outline" icon={<Download className="w-4 h-4" />}>تصدير</Button>
        </div>
      </div>

      <Card>
        <div className="flex items-center gap-4 mb-6">
          <label className="form-label">الفترة</label>
          <input type="month" value={period} onChange={(e) => setPeriod(e.target.value)} className="input-field w-48" />
        </div>
      </Card>

      {data && (
        <>
          <div className="grid grid-cols-2 gap-6">
            <Card>
              <h3 className="section-title mb-4">المبيعات الخاضعة للضريبة</h3>
              <div className="space-y-3">
                <div className="flex justify-between py-2 border-b border-surface-700/30">
                  <span className="text-surface-400">قيمة المبيعات الخاضعة (100%)</span>
                  <span className="font-bold">{formatOMR(data.taxable_sales || 0)}</span>
                </div>
                <div className="flex justify-between py-2 border-b border-surface-700/30">
                  <span className="text-surface-400">المبيعات المعفاة (0%)</span>
                  <span className="font-bold">{formatOMR(data.exempt_sales || 0)}</span>
                </div>
                <div className="flex justify-between py-2 border-b border-surface-700/30">
                  <span className="text-surface-400">إجمالي المبيعات</span>
                  <span className="font-bold text-white">{formatOMR(data.total_sales || 0)}</span>
                </div>
                <div className="flex justify-between py-3">
                  <span className="font-bold text-emerald-400">ضريبة المبيعات (5%)</span>
                  <span className="text-xl font-bold gradient-text">{formatOMR(data.output_vat || 0)}</span>
                </div>
              </div>
            </Card>

            <Card>
              <h3 className="section-title mb-4">المشتريات الخاضعة للضريبة</h3>
              <div className="space-y-3">
                <div className="flex justify-between py-2 border-b border-surface-700/30">
                  <span className="text-surface-400">قيمة المشتريات الخاضعة</span>
                  <span className="font-bold">{formatOMR(data.taxable_purchases || 0)}</span>
                </div>
                <div className="flex justify-between py-2 border-b border-surface-700/30">
                  <span className="text-surface-400">المشتريات المعفاة</span>
                  <span className="font-bold">{formatOMR(data.exempt_purchases || 0)}</span>
                </div>
                <div className="flex justify-between py-2 border-b border-surface-700/30">
                  <span className="text-surface-400">إجمالي المشتريات</span>
                  <span className="font-bold text-white">{formatOMR(data.total_purchases || 0)}</span>
                </div>
                <div className="flex justify-between py-3">
                  <span className="font-bold text-blue-400">ضريبة المشتريات (5%)</span>
                  <span className="text-xl font-bold text-blue-400">{formatOMR(data.input_vat || 0)}</span>
                </div>
              </div>
            </Card>
          </div>

          <Card>
            <h3 className="section-title mb-4">النتيجة النهائية</h3>
            <div className="grid grid-cols-3 gap-6">
              <div className="text-center p-4 bg-surface-800/50 rounded-xl">
                <p className="text-sm text-surface-400 mb-1">ضريبة المبيعات</p>
                <p className="text-2xl font-bold text-emerald-400">{formatOMR(data.output_vat || 0)}</p>
              </div>
              <div className="text-center p-4 bg-surface-800/50 rounded-xl">
                <p className="text-sm text-surface-400 mb-1">ضريبة المشتريات</p>
                <p className="text-2xl font-bold text-blue-400">{formatOMR(data.input_vat || 0)}</p>
              </div>
              <div className="text-center p-4 bg-surface-800/50 rounded-xl border border-brand-500/30">
                <p className="text-sm text-surface-400 mb-1">المبلغ المستحق للسداد</p>
                <p className={`text-3xl font-bold ${(data.net_vat || 0) >= 0 ? 'gradient-text' : 'text-emerald-400'}`}>
                  {formatOMR(data.net_vat || 0)}
                </p>
                <p className="text-xs text-surface-500 mt-1">
                  {(data.net_vat || 0) >= 0 ? "مبلغ مستحق للسداد" : "مبلغ مسترد"}
                </p>
              </div>
            </div>
          </Card>
        </>
      )}
    </div>
  );
}
