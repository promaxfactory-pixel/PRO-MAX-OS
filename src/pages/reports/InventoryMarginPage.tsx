import { useState, useEffect, useMemo, useCallback } from "react";
import Card from "@/components/ui/Card";
import DataTable, { Column } from "@/components/ui/DataTable";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@/lib/tauri";
import { Package, TrendingUp } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

interface MarginRow {
  code: string;
  name: string;
  qty_on_hand: number;
  avg_cost_milli: number;
  selling_price_milli: number;
  margin_milli: number;
  margin_pct: number;
  stock_value_milli: number;
  stock_revenue_milli: number;
}

export default function InventoryMarginPage() {
  const addNotification = useUIStore((s) => s.addNotification);
  const [data, setData] = useState<MarginRow[]>([]);
  const [, setLoading] = useState(true);

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const result = await invoke("inventory_margin_report");
      setData(result as MarginRow[]);
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء تحميل البيانات" }); }
    finally { setLoading(false); }
  }, [addNotification]);

  useEffect(() => { loadData(); }, [loadData]);

  const totalStockValue = data.reduce((s, r) => s + r.stock_value_milli, 0);
  const totalRevenue = data.reduce((s, r) => s + r.stock_revenue_milli, 0);

  const columns: Column<MarginRow>[] = useMemo(() => [
    { key: "code", header: "الكود", render: (r) => r.code || "—" },
    { key: "name", header: "المنتج", render: (r) => r.name || "—" },
    { key: "qty_on_hand", header: "الكمية", align: "center", render: (r) => r.qty_on_hand.toLocaleString() },
    { key: "avg_cost_milli", header: "متوسط التكلفة", align: "left", render: (r) => formatOMR(r.avg_cost_milli) },
    { key: "selling_price_milli", header: "سعر البيع", align: "left", render: (r) => formatOMR(r.selling_price_milli) },
    { key: "margin_milli", header: "الهامش", align: "left", render: (r) => <span className="font-bold text-emerald-400">{formatOMR(r.margin_milli)}</span> },
    { key: "margin_pct", header: "النسبة %", align: "center", render: (r) => <span className="font-bold">{r.margin_pct.toFixed(1)}%</span> },
    { key: "stock_value_milli", header: "قيمة المخزون", align: "left", render: (r) => formatOMR(r.stock_value_milli) },
    { key: "stock_revenue_milli", header: "الإيراد المتوقع", align: "left", render: (r) => formatOMR(r.stock_revenue_milli) },
  ], []);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">تقرير هامش الربح</h1>
          <p className="page-subtitle">تحليل هامش الربح لكل منتج في المخزون</p>
        </div>
      </div>

      <div className="grid grid-cols-3 gap-4">
        <Card>
          <div className="flex items-center gap-3 mb-2">
            <div className="p-2 rounded-lg bg-brand-500/10 text-brand-400"><Package className="w-5 h-5" /></div>
            <p className="text-sm text-surface-400">عدد المنتجات</p>
          </div>
          <p className="text-2xl font-bold gradient-text">{data.length}</p>
        </Card>
        <Card>
          <div className="flex items-center gap-3 mb-2">
            <div className="p-2 rounded-lg bg-blue-500/10 text-blue-400"><Package className="w-5 h-5" /></div>
            <p className="text-sm text-surface-400">إجمالي قيمة المخزون</p>
          </div>
          <p className="text-2xl font-bold gradient-text">{formatOMR(totalStockValue)}</p>
        </Card>
        <Card>
          <div className="flex items-center gap-3 mb-2">
            <div className="p-2 rounded-lg bg-emerald-500/10 text-emerald-400"><TrendingUp className="w-5 h-5" /></div>
            <p className="text-sm text-surface-400">الإيراد المتوقع</p>
          </div>
          <p className="text-2xl font-bold text-emerald-400">{formatOMR(totalRevenue)}</p>
        </Card>
      </div>

      <Card>
        <DataTable columns={columns} data={data} compact />
      </Card>
    </div>
  );
}
