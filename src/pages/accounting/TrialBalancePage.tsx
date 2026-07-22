import { useState, useEffect } from "react";
import DataTable, { Column } from "@/components/ui/DataTable";
import Card from "@/components/ui/Card";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Calculator } from "lucide-react";
import { useUIStore } from "../../stores/uiStore";

export default function TrialBalancePage() {
  const { addNotification } = useUIStore();
  const [data, setData] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => { invoke("get_trial_balance").then((d: any) => setData(d)).catch((e: any) => addNotification({ title: 'خطأ', message: String(e), type: 'error' })).finally(() => setLoading(false)); }, []);

  const totalDebit = data.reduce((s, r) => s + (r.debit_milli || 0), 0);
  const totalCredit = data.reduce((s, r) => s + (r.credit_milli || 0), 0);

  const columns: Column<any>[] = [
    { key: "account_code", header: "الكود", render: (r) => <span className="font-mono text-brand-400">{r.account_code}</span> },
    { key: "account_name", header: "الاسم" },
    { key: "debit_milli", header: "المدين", align: "left", render: (r) => r.debit_milli > 0 ? formatOMR(r.debit_milli) : "—" },
    { key: "credit_milli", header: "الدائن", align: "left", render: (r) => r.credit_milli > 0 ? formatOMR(r.credit_milli) : "—" },
  ];

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div><h1 className="page-title">ميزان المراجعة</h1></div>
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
