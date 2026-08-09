import { useState, useEffect } from "react";
import Card from "@/components/ui/Card";
import Tabs from "@/components/ui/Tabs";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { TrendingUp, BarChart3 } from "lucide-react";
import { useUIStore } from "../../stores/uiStore";
import { useTranslation } from "react-i18next";

export default function FinancialStatementsPage() {
  const { t } = useTranslation();
  const { addNotification } = useUIStore();
  const [activeTab, setActiveTab] = useState("income");
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [income, setIncome] = useState<any>(null);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [balance, setBalance] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    Promise.all([invoke("get_income_statement"), invoke("get_balance_sheet")])
      .then(([i, b]) => { setIncome(i); setBalance(b); })
      .catch((e: unknown) => addNotification({ title: t("common.error"), message: String(e), type: 'error' }))
      .finally(() => setLoading(false));
  }, []);

  if (loading) return <div className="flex items-center justify-center h-64"><div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" /></div>;

  return (
    <div className="space-y-6">
      <div className="page-header"><div><h1 className="page-title">{t("financialStatements.title")}</h1></div></div>
      <Tabs tabs={[{key:"income",label:t("accounting.incomeStatement"),icon:<TrendingUp className="w-4 h-4"/>},{key:"balance",label:t("accounting.balanceSheet"),icon:<BarChart3 className="w-4 h-4"/>}]} onChange={setActiveTab} />
      {activeTab === "income" && income && (
        <Card>
          <h3 className="section-title mb-4">{t("accounting.incomeStatement")}</h3>
          <div className="space-y-3">
            <div className="flex justify-between py-2 border-b border-surface-700/30"><span className="text-surface-400">{t("financialStatements.totalRevenue")}</span><span className="text-emerald-400 font-bold">{formatOMR(income.total_revenue || 0)}</span></div>
            <div className="flex justify-between py-2 border-b border-surface-700/30"><span className="text-surface-400">{t("expenses.totalExpenses")}</span><span className="text-red-400 font-bold">{formatOMR(income.total_expenses || 0)}</span></div>
            <div className="flex justify-between py-3"><span className="font-bold text-white">{t("financialStatements.netProfit")}</span><span className={`text-xl font-bold ${(income.net_income || 0) >= 0 ? 'gradient-text' : 'text-red-400'}`}>{formatOMR(income.net_income || 0)}</span></div>
          </div>
        </Card>
      )}
      {activeTab === "balance" && balance && (
        <div className="grid grid-cols-2 gap-6">
          <Card>
            <h3 className="section-title mb-4">{t("financialStatements.assets")}</h3>
            {(balance.assets || []).map((a: any, i: number) => (
              <div key={i} className="flex justify-between py-1.5 text-sm border-b border-surface-700/20"><span className="text-surface-300">{a.account_name}</span><span>{formatOMR(a.total || 0)}</span></div>
            ))}
            <div className="flex justify-between py-2 mt-2 border-t border-surface-600 font-bold"><span>{t("cashBank.total")}</span><span className="gradient-text">{formatOMR(balance.total_assets || 0)}</span></div>
          </Card>
          <Card>
            <h3 className="section-title mb-4">{t("financialStatements.liabilitiesEquity")}</h3>
            {[...(balance.liabilities || []), ...(balance.equity || [])].map((a: any, i: number) => (
              <div key={i} className="flex justify-between py-1.5 text-sm border-b border-surface-700/20"><span className="text-surface-300">{a.account_name}</span><span>{formatOMR(a.total || 0)}</span></div>
            ))}
            <div className="flex justify-between py-2 mt-2 border-t border-surface-600 font-bold"><span>{t("cashBank.total")}</span><span className="gradient-text">{formatOMR(balance.total_liabilities_equity || 0)}</span></div>
          </Card>
        </div>
      )}
    </div>
  );
}
