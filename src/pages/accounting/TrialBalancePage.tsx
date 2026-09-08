import { useState, useEffect, useMemo } from "react";
import DataTable, { Column } from "@/components/ui/DataTable";
import Card from "@/components/ui/Card";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@/lib/tauri";
import { useUIStore } from "../../stores/uiStore";

interface TrialBalanceRow {
  account_code: string;
  account_name: string;
  debit_milli: number;
  credit_milli: number;
}

export default function TrialBalancePage() {
  const { addNotification } = useUIStore();
  const [data, setData] = useState<TrialBalanceRow[]>([]);
  const [loading, setLoading] = useState(true);
  const [dateTo, setDateTo] = useState(() => new Date().toISOString().slice(0, 10));

  useEffect(() => { setLoading(true); invoke("get_trial_balance_as_of", { dateTo }).then((d: unknown) => setData(d as TrialBalanceRow[])).catch((e: unknown) => addNotification({ title: "خطأ", message: String(e), type: "error" })).finally(() => setLoading(false)); }, [addNotification, dateTo]);

  const totalDebit = data.reduce((s, r) => s + (r.debit_milli || 0), 0);
  const totalCredit = data.reduce((s, r) => s + (r.credit_milli || 0), 0);

  const columns: Column<TrialBalanceRow>[] = useMemo(() => [
    { key: "account_code", header: "الكود", render: (r) => <span className="font-mono text-brand-400">{r.account_code}</span> },
    { key: "account_name", header: "الاسم" },
    { key: "debit_milli", header: "المدين", align: "left", render: (r) => r.debit_milli > 0 ? formatOMR(r.debit_milli) : "—" },
    { key: "credit_milli", header: "الدائن", align: "left", render: (r) => r.credit_milli > 0 ? formatOMR(r.credit_milli) : "—" },
  ], []);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div><h1 className="page-title">ميزان المراجعة</h1><p className="mt-1 text-sm text-surface-400">رصيد القيود حتى تاريخ التقرير المحدد</p></div>
        <label className="text-sm text-surface-300">حتى تاريخ <input aria-label="تاريخ ميزان المراجعة" className="ms-2 rounded-lg border border-surface-600 bg-surface-800 px-3 py-2 text-white" type="date" value={dateTo} onChange={(event) => setDateTo(event.target.value)} /></label>
      </div>
      <div className="grid grid-cols-2 gap-4 mb-6">
        <Card><div className="text-center"><p className="text-3xl font-bold gradient-text">{formatOMR(totalDebit)}</p><p className="text-xs text-surface-400">إجمالي المدين</p></div></Card>
        <Card><div className="text-center"><p className="text-3xl font-bold gradient-text">{formatOMR(totalCredit)}</p><p className="text-xs text-surface-400">إجمالي الدائن</p></div></Card>
      </div>
      <DataTable columns={columns} data={data} loading={loading} emptyMessage="لا توجد بيانات" />
      {totalDebit !== totalCredit && (
        <div className="p-4 bg-red-500/10 border border-red-500/30 rounded-xl text-red-400 text-sm text-center">
          ⚠ الميزان غير متوازن! الفرق: {formatOMR(Math.abs(totalDebit - totalCredit))}
        </div>
      )}
    </div>
  );
}
