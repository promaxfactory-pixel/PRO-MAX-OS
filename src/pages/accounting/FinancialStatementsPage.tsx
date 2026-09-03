import { useEffect, useState } from "react";
import Card from "@/components/ui/Card";
import EmptyState from "@/components/ui/EmptyState";
import Tabs from "@/components/ui/Tabs";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@/lib/tauri";
import {
  AlertTriangle,
  BarChart3,
  CircleDollarSign,
  RefreshCw,
  Scale,
  TrendingUp,
} from "lucide-react";
import { useUIStore } from "../../stores/uiStore";

interface StatementAccount {
  type: "Asset" | "Liability" | "Equity" | "Revenue" | "Expense";
  code: string;
  name: string;
  balance_milli: number;
}

interface IncomeStatement {
  revenue: StatementAccount[];
  expenses: StatementAccount[];
  total_revenue_milli: number;
  total_expenses_milli: number;
  net_income_milli: number;
}

interface BalanceSheet {
  assets: StatementAccount[];
  liabilities: StatementAccount[];
  equity: StatementAccount[];
  current_period_earnings_milli: number;
  total_assets_milli: number;
  total_liabilities_milli: number;
  total_equity_milli: number;
  total_liabilities_equity_milli: number;
  out_of_balance_milli: number;
}

function AccountRows({ rows }: { rows: StatementAccount[] }) {
  const materialRows = rows.filter((row) => row.balance_milli !== 0);
  if (materialRows.length === 0) {
    return <p className="py-5 text-center text-sm text-surface-400">لا توجد أرصدة مسجلة</p>;
  }

  return (
    <div className="divide-y" style={{ borderColor: "var(--border-light)" }}>
      {materialRows.map((row) => (
        <div key={row.code} className="flex items-center justify-between gap-4 py-3 text-sm">
          <div className="min-w-0">
            <p className="truncate font-medium text-white">{row.name}</p>
            <p className="mt-0.5 font-mono text-xs text-surface-400" dir="ltr">
              {row.code}
            </p>
          </div>
          <span className="shrink-0 font-mono font-semibold tabular-nums" dir="ltr">
            {formatOMR(row.balance_milli)}
          </span>
        </div>
      ))}
    </div>
  );
}

export default function FinancialStatementsPage() {
  const { addNotification } = useUIStore();
  const [activeTab, setActiveTab] = useState("income");
  const [income, setIncome] = useState<IncomeStatement | null>(null);
  const [balance, setBalance] = useState<BalanceSheet | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [reloadKey, setReloadKey] = useState(0);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);
    Promise.all([
      invoke<IncomeStatement>("get_income_statement"),
      invoke<BalanceSheet>("get_balance_sheet"),
    ])
      .then(([incomeResult, balanceResult]) => {
        if (!cancelled) {
          setIncome(incomeResult);
          setBalance(balanceResult);
        }
      })
      .catch((requestError: unknown) => {
        if (cancelled) return;
        const message = String(requestError);
        setError(message);
        addNotification({ title: "تعذر تحميل القوائم المالية", message, type: "error" });
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [addNotification, reloadKey]);

  if (loading) {
    return (
      <div
        className="flex h-64 items-center justify-center"
        role="status"
        aria-label="جاري تحميل القوائم المالية"
      >
        <div className="h-11 w-11 animate-spin rounded-full border-2 border-brand-200 border-t-brand-600" />
      </div>
    );
  }

  if (error) {
    return (
      <Card className="mx-auto max-w-2xl text-center">
        <AlertTriangle className="mx-auto h-10 w-10 text-red-400" />
        <h2 className="mt-3 text-lg font-bold text-white">تعذر تحميل القوائم المالية</h2>
        <p className="mt-2 text-sm text-surface-400">{error}</p>
        <button className="btn-outline mt-5" onClick={() => setReloadKey((key) => key + 1)}>
          <RefreshCw className="h-4 w-4" /> إعادة المحاولة
        </button>
      </Card>
    );
  }

  if (!income || !balance) {
    return (
      <EmptyState
        title="لا توجد بيانات مالية"
        description="لم يرجع النظام بيانات كافية لإعداد القوائم المالية."
      />
    );
  }

  const balanced = balance.out_of_balance_milli === 0;

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">القوائم المالية</h1>
          <p className="mt-1 text-sm text-surface-400">
            ملخص تراكمي مبني على القيود المسجلة في دفتر الأستاذ
          </p>
        </div>
        <div
          className={`flex items-center gap-2 rounded-xl border px-3 py-2 text-sm font-semibold ${balanced ? "text-emerald-500" : "text-red-500"}`}
          style={{
            borderColor: balanced
              ? "color-mix(in srgb, var(--success) 30%, transparent)"
              : "color-mix(in srgb, var(--danger) 30%, transparent)",
          }}
          role="status"
        >
          {balanced ? <Scale className="h-4 w-4" /> : <AlertTriangle className="h-4 w-4" />}
          {balanced
            ? "الميزانية متوازنة"
            : `فرق غير مسوّى: ${formatOMR(balance.out_of_balance_milli)}`}
        </div>
      </div>

      <Tabs
        tabs={[
          { key: "income", label: "قائمة الدخل", icon: <TrendingUp className="h-4 w-4" /> },
          { key: "balance", label: "الميزانية العمومية", icon: <BarChart3 className="h-4 w-4" /> },
        ]}
        activeKey={activeTab}
        onChange={setActiveTab}
      />

      {activeTab === "income" && (
        <div id="tabpanel-income" role="tabpanel" className="space-y-5">
          <div className="grid gap-4 md:grid-cols-3">
            <Card className="border-s-4 border-s-emerald-400">
              <p className="text-sm font-medium text-surface-400">إجمالي الإيرادات</p>
              <p className="mt-2 font-mono text-2xl font-bold text-emerald-500" dir="ltr">
                {formatOMR(income.total_revenue_milli)}
              </p>
            </Card>
            <Card className="border-s-4 border-s-amber-400">
              <p className="text-sm font-medium text-surface-400">إجمالي المصروفات</p>
              <p className="mt-2 font-mono text-2xl font-bold text-amber-500" dir="ltr">
                {formatOMR(income.total_expenses_milli)}
              </p>
            </Card>
            <Card
              className={`border-s-4 ${income.net_income_milli >= 0 ? "border-s-blue-400" : "border-s-red-400"}`}
            >
              <p className="text-sm font-medium text-surface-400">صافي الربح / الخسارة</p>
              <p
                className={`mt-2 font-mono text-2xl font-bold ${income.net_income_milli >= 0 ? "text-blue-500" : "text-red-500"}`}
                dir="ltr"
              >
                {formatOMR(income.net_income_milli)}
              </p>
            </Card>
          </div>
          <div className="grid gap-5 lg:grid-cols-2">
            <Card>
              <h2 className="section-title flex items-center gap-2">
                <TrendingUp className="h-5 w-5 text-emerald-500" />
                الإيرادات
              </h2>
              <AccountRows rows={income.revenue} />
            </Card>
            <Card>
              <h2 className="section-title flex items-center gap-2">
                <CircleDollarSign className="h-5 w-5 text-amber-500" />
                المصروفات
              </h2>
              <AccountRows rows={income.expenses} />
            </Card>
          </div>
        </div>
      )}

      {activeTab === "balance" && (
        <div id="tabpanel-balance" role="tabpanel" className="grid gap-5 lg:grid-cols-2">
          <Card>
            <h2 className="section-title">الأصول</h2>
            <AccountRows rows={balance.assets} />
            <div
              className="mt-3 flex justify-between border-t pt-4 font-bold"
              style={{ borderColor: "var(--border)" }}
            >
              <span>إجمالي الأصول</span>
              <span className="font-mono text-blue-500" dir="ltr">
                {formatOMR(balance.total_assets_milli)}
              </span>
            </div>
          </Card>
          <Card>
            <h2 className="section-title">الالتزامات وحقوق الملكية</h2>
            <p className="mb-2 mt-4 text-xs font-bold uppercase tracking-wide text-surface-400">
              الالتزامات
            </p>
            <AccountRows rows={balance.liabilities} />
            <p className="mb-2 mt-5 text-xs font-bold uppercase tracking-wide text-surface-400">
              حقوق الملكية
            </p>
            <AccountRows rows={balance.equity} />
            <div className="flex items-center justify-between gap-4 py-3 text-sm">
              <span className="font-medium text-white">نتيجة الفترة الحالية</span>
              <span className="font-mono font-semibold" dir="ltr">
                {formatOMR(balance.current_period_earnings_milli)}
              </span>
            </div>
            <div
              className="mt-3 flex justify-between border-t pt-4 font-bold"
              style={{ borderColor: "var(--border)" }}
            >
              <span>إجمالي الالتزامات وحقوق الملكية</span>
              <span className="font-mono text-blue-500" dir="ltr">
                {formatOMR(balance.total_liabilities_equity_milli)}
              </span>
            </div>
          </Card>
        </div>
      )}
    </div>
  );
}
