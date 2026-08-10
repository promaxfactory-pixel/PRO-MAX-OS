import { useState, useEffect, useMemo } from "react";
import DataTable, { Column } from "@/components/ui/DataTable";
import Card from "@/components/ui/Card";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { useUIStore } from "@/stores/uiStore";

interface AgingRow {
  customer_code: string;
  customer_name: string;
  current: number;
  days_30: number;
  days_60: number;
  days_90: number;
  over_90: number;
  total: number;
}

interface AgingSummary {
  total: number;
  current: number;
  days_30: number;
  days_60: number;
  days_90: number;
  over_90: number;
}

export default function ReportsAgingPage() {
  const navigate = useNavigate();
  const addNotification = useUIStore((s) => s.addNotification);
  const [data, setData] = useState<AgingRow[]>([]);
  const [loading, setLoading] = useState(true);
  const [summary, setSummary] = useState<AgingSummary>({ total: 0, current: 0, days_30: 0, days_60: 0, days_90: 0, over_90: 0 });

  useEffect(() => {
    invoke("customers_aging")
      .then((d: unknown) => {
        const arr = (d || []) as AgingRow[];
        setData(arr);
        setSummary({
          total: arr.reduce((s, c) => s + (c.total || 0), 0),
          current: arr.reduce((s, c) => s + (c.current || 0), 0),
          days_30: arr.reduce((s, c) => s + (c.days_30 || 0), 0),
          days_60: arr.reduce((s, c) => s + (c.days_60 || 0), 0),
          days_90: arr.reduce((s, c) => s + (c.days_90 || 0), 0),
          over_90: arr.reduce((s, c) => s + (c.over_90 || 0), 0),
        });
      })
      .catch((e: unknown) => addNotification({ title: "خطأ", message: String(e), type: "error" }))
      .finally(() => setLoading(false));
  }, []);

  const columns: Column<AgingRow>[] = useMemo(() => [
    { key: "customer_code", header: "الكود", render: (r) => <span className="font-mono text-brand-400">{r.customer_code}</span> },
    { key: "customer_name", header: "اسم العميل", sortable: true, render: (r) => <span className="font-medium">{r.customer_name}</span> },
    { key: "current", header: "حتى 30 يوم", align: "left", render: (r) => <span className="text-emerald-400">{formatOMR(r.current)}</span> },
    { key: "days_30", header: "31-60 يوم", align: "left", render: (r) => <span className="text-amber-400">{formatOMR(r.days_30)}</span> },
    { key: "days_60", header: "61-90 يوم", align: "left", render: (r) => <span className="text-orange-400">{formatOMR(r.days_60)}</span> },
    { key: "days_90", header: "91-120 يوم", align: "left", render: (r) => <span className="text-red-400">{formatOMR(r.days_90)}</span> },
    { key: "over_90", header: "أكثر من 120 يوم", align: "left", render: (r) => <span className="text-red-500 font-bold">{formatOMR(r.over_90)}</span> },
    { key: "total", header: "الإجمالي", align: "left", render: (r) => <span className="font-bold text-white">{formatOMR(r.total)}</span> },
  ], []);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div className="flex items-center gap-3">
          <button onClick={() => navigate("/reports")} className="btn-ghost p-2">
            <ArrowRight className="w-5 h-5" />
          </button>
          <div>
            <h1 className="page-title">أعمار الذمم</h1>
            <p className="page-subtitle">تحليل مستحقات العملاء</p>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-5 gap-4">
        <Card>
          <div className="text-center">
            <p className="text-2xl font-bold gradient-text">{formatOMR(summary.current)}</p>
            <p className="text-xs text-surface-400 mt-1">حتى 30 يوم</p>
          </div>
        </Card>
        <Card>
          <div className="text-center">
            <p className="text-2xl font-bold text-amber-400">{formatOMR(summary.days_30)}</p>
            <p className="text-xs text-surface-400 mt-1">31-60 يوم</p>
          </div>
        </Card>
        <Card>
          <div className="text-center">
            <p className="text-2xl font-bold text-orange-400">{formatOMR(summary.days_60)}</p>
            <p className="text-xs text-surface-400 mt-1">61-90 يوم</p>
          </div>
        </Card>
        <Card>
          <div className="text-center">
            <p className="text-2xl font-bold text-red-400">{formatOMR(summary.days_90)}</p>
            <p className="text-xs text-surface-400 mt-1">91-120 يوم</p>
          </div>
        </Card>
        <Card>
          <div className="text-center">
            <p className="text-2xl font-bold text-red-500">{formatOMR(summary.over_90)}</p>
            <p className="text-xs text-surface-400 mt-1">أكثر من 120 يوم</p>
          </div>
        </Card>
      </div>

      <Card>
        <div className="flex justify-between items-center mb-4">
          <h3 className="section-title">التفاصيل حسب العميل</h3>
          <span className="text-sm text-surface-400">الإجمالي: <span className="font-bold text-white">{formatOMR(summary.total)}</span></span>
        </div>
        <DataTable columns={columns} data={data} loading={loading} emptyMessage="لا توجد مستحقات" />
      </Card>
    </div>
  );
}
