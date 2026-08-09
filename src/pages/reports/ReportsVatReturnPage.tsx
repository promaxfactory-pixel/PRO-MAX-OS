import { useState, useEffect } from "react";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, Printer, Download } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { useUIStore } from "@/stores/uiStore";
import { useTranslation } from "react-i18next";

interface VatReturn {
  taxable_sales: number;
  exempt_sales: number;
  total_sales: number;
  output_vat: number;
  taxable_purchases: number;
  exempt_purchases: number;
  total_purchases: number;
  input_vat: number;
  net_vat: number;
}

export default function ReportsVatReturnPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const addNotification = useUIStore((s) => s.addNotification);
  const [data, setData] = useState<VatReturn | null>(null);
  const [loading, setLoading] = useState(true);
  const [period, setPeriod] = useState(() => {
    const now = new Date();
    return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}`;
  });

  const fetchData = () => {
    setLoading(true);
    invoke("vat_return", { month: period })
      .then((d: unknown) => setData(d as VatReturn))
      .catch((e: unknown) => addNotification({ title: t('common.error'), message: String(e), type: 'error' }))
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
            <h1 className="page-title">{t("reports.vatReturnTitle")}</h1>
            <p className="page-subtitle">{t("reports.vatReturnPeriod", { period })}</p>
          </div>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" icon={<Printer className="w-4 h-4" />}>{t("common.print")}</Button>
          <Button variant="outline" icon={<Download className="w-4 h-4" />}>{t("common.export")}</Button>
        </div>
      </div>

      <Card>
        <div className="flex items-center gap-4 mb-6">
          <label className="form-label">{t("payrollPage.period")}</label>
          <input type="month" value={period} onChange={(e) => setPeriod(e.target.value)} className="input-field w-48" aria-label={t("payrollPage.period")} />
        </div>
      </Card>

      {data && (
        <>
          <div className="grid grid-cols-2 gap-6">
            <Card>
              <h3 className="section-title mb-4">{t("reports.taxableSalesSection")}</h3>
              <div className="space-y-3">
                <div className="flex justify-between py-2 border-b border-surface-700/30">
                  <span className="text-surface-400">{t("reports.taxableSalesValue")}</span>
                  <span className="font-bold">{formatOMR(data.taxable_sales || 0)}</span>
                </div>
                <div className="flex justify-between py-2 border-b border-surface-700/30">
                  <span className="text-surface-400">{t("reports.exemptSalesValue")}</span>
                  <span className="font-bold">{formatOMR(data.exempt_sales || 0)}</span>
                </div>
                <div className="flex justify-between py-2 border-b border-surface-700/30">
                  <span className="text-surface-400">{t("reports.totalSales")}</span>
                  <span className="font-bold text-white">{formatOMR(data.total_sales || 0)}</span>
                </div>
                <div className="flex justify-between py-3">
                  <span className="font-bold text-emerald-400">{t("reports.salesVat")}</span>
                  <span className="text-xl font-bold gradient-text">{formatOMR(data.output_vat || 0)}</span>
                </div>
              </div>
            </Card>

            <Card>
              <h3 className="section-title mb-4">{t("reports.taxablePurchasesSection")}</h3>
              <div className="space-y-3">
                <div className="flex justify-between py-2 border-b border-surface-700/30">
                  <span className="text-surface-400">{t("reports.taxablePurchasesValue")}</span>
                  <span className="font-bold">{formatOMR(data.taxable_purchases || 0)}</span>
                </div>
                <div className="flex justify-between py-2 border-b border-surface-700/30">
                  <span className="text-surface-400">{t("reports.exemptPurchasesValue")}</span>
                  <span className="font-bold">{formatOMR(data.exempt_purchases || 0)}</span>
                </div>
                <div className="flex justify-between py-2 border-b border-surface-700/30">
                  <span className="text-surface-400">{t("reports.totalPurchases")}</span>
                  <span className="font-bold text-white">{formatOMR(data.total_purchases || 0)}</span>
                </div>
                <div className="flex justify-between py-3">
                  <span className="font-bold text-blue-400">{t("reports.purchaseVat")}</span>
                  <span className="text-xl font-bold text-blue-400">{formatOMR(data.input_vat || 0)}</span>
                </div>
              </div>
            </Card>
          </div>

          <Card>
            <h3 className="section-title mb-4">{t("reports.finalResult")}</h3>
            <div className="grid grid-cols-3 gap-6">
              <div className="text-center p-4 bg-surface-800/50 rounded-xl">
                <p className="text-sm text-surface-400 mb-1">{t("reports.outputVat")}</p>
                <p className="text-2xl font-bold text-emerald-400">{formatOMR(data.output_vat || 0)}</p>
              </div>
              <div className="text-center p-4 bg-surface-800/50 rounded-xl">
                <p className="text-sm text-surface-400 mb-1">{t("reports.inputVat")}</p>
                <p className="text-2xl font-bold text-blue-400">{formatOMR(data.input_vat || 0)}</p>
              </div>
              <div className="text-center p-4 bg-surface-800/50 rounded-xl border border-brand-500/30">
                <p className="text-sm text-surface-400 mb-1">{t("reports.amountDue")}</p>
                <p className={`text-3xl font-bold ${(data.net_vat || 0) >= 0 ? 'gradient-text' : 'text-emerald-400'}`}>
                  {formatOMR(data.net_vat || 0)}
                </p>
                <p className="text-xs text-surface-500 mt-1">
                  {(data.net_vat || 0) >= 0 ? t("reports.dueForPayment") : t("reports.refundAmount")}
                </p>
              </div>
            </div>
          </Card>
        </>
      )}
    </div>
  );
}
