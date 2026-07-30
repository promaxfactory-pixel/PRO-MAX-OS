import { useState, useEffect, useMemo } from "react";
import { useParams, useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { StatusBadge } from "@/components/ui/Badge";
import DataTable, { Column } from "@/components/ui/DataTable";
import { formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, Check, Ban } from "lucide-react";
import ConfirmDialog from "@/components/ui/ConfirmDialog";
import { useUIStore } from "../../stores/uiStore";
import type { ProductionLine, ProductionOrder } from "@/types";

export default function ProductionOrderDetailPage() {
  const { addNotification } = useUIStore();
  const { id } = useParams();
  const navigate = useNavigate();
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [order, setOrder] = useState<any>(null);
  const [lines, setLines] = useState<ProductionLine[]>([]);
  const [loading, setLoading] = useState(true);
  const [showConfirm, setShowConfirm] = useState(false);

  useEffect(() => {
    Promise.all([
      invoke("get_production_order", { id: Number(id) }),
      invoke("get_production_lines", { orderId: Number(id) }),
    ]).then(([o, l]) => { setOrder(o); setLines(l as ProductionLine[]); }).catch((e: unknown) => addNotification({ title: 'ط®ط·ط£', message: String(e), type: 'error' })).finally(() => setLoading(false));
  }, [id]);

  const handleApprove = async () => {
    await invoke("approve_production_order", { id: Number(id) });
    invoke("get_production_order", { id: Number(id) }).then((o) => setOrder(o));
  };

  if (loading) return <div className="flex items-center justify-center h-64"><div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" /></div>;

  const cols: Column<ProductionLine>[] = useMemo(() => [
    { key: "product_name", header: "ط§ظ„ظ…ظ†طھط¬" },
    { key: "cartons_good", header: "ظƒط±طھظˆظ† طµط§ظ„ط­", align: "center" },
    { key: "cups_good", header: "ظƒظˆط¨ طµط§ظ„ط­", align: "center" },
    { key: "cartons_waste", header: "ظ‡ط§ظ„ظƒ ظƒط±طھظˆظ†", align: "center", render: (r) => <span className="text-red-400">{r.cartons_waste}</span> },
    { key: "worker", header: "ط§ظ„ط¹ط§ظ…ظ„" },
    { key: "brand_type", header: "ط§ظ„ط¹ظ„ط§ظ…ط©" },
    { key: "quality_status", header: "ط§ظ„ط¬ظˆط¯ط©", render: (r) => r.quality_status || "â€”" },
  ], []);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div className="flex items-center gap-3">
          <button onClick={() => navigate('/production')} className="btn-ghost p-2"><ArrowRight className="w-5 h-5" /></button>
          <div>
            <h1 className="page-title flex items-center gap-3">
              <span className="font-mono text-brand-400">{order.prod_no}</span>
              <StatusBadge status={order.status} />
            </h1>
            <p className="page-subtitle">{formatDate(order.date)} â€¢ {order.shift} â€¢ {order.operator}</p>
          </div>
        </div>
        {order.status === "Draft" && (
          <div className="flex gap-2">
            <Button onClick={() => setShowConfirm(true)} icon={<Check className="w-4 h-4" />}>ط§ط¹طھظ…ط§ط¯</Button>
            <Button variant="danger" icon={<Ban className="w-4 h-4" />}>ط¥ظ„ط؛ط§ط،</Button>
          </div>
        )}
      </div>
      <div className="grid grid-cols-4 gap-4">
        <Card className="text-center"><p className="text-2xl font-bold text-white">{order.run_minutes}</p><p className="text-xs text-surface-400">ط¯ظ‚ظٹظ‚ط© طھط´ط؛ظٹظ„</p></Card>
        <Card className="text-center"><p className="text-2xl font-bold text-amber-400">{order.downtime_minutes}</p><p className="text-xs text-surface-400">ط¯ظ‚ط§ط¦ظ‚ طھظˆظ‚ظپ</p></Card>
        <Card className="text-center"><p className="text-2xl font-bold text-emerald-400">{lines.reduce((s, l) => s + (l.cartons_good || 0), 0)}</p><p className="text-xs text-surface-400">ظƒط±طھظˆظ† طµط§ظ„ط­</p></Card>
        <Card className="text-center"><p className="text-2xl font-bold text-red-400">{lines.reduce((s, l) => s + (l.cartons_waste || 0), 0)}</p><p className="text-xs text-surface-400">ظ‡ط§ظ„ظƒ</p></Card>
      </div>
      <DataTable columns={cols} data={lines} compact emptyMessage="ظ„ط§ طھظˆط¬ط¯ ط®ط·ظˆط·" />

      <ConfirmDialog
        open={showConfirm}
        title="ط§ط¹طھظ…ط§ط¯ ط£ظ…ط± ط§ظ„ط¥ظ†طھط§ط¬"
        message="ظ‡ظ„ ط£ظ†طھ ظ…طھط£ظƒط¯ ظ…ظ† ط§ط¹طھظ…ط§ط¯ ط£ظ…ط± ط§ظ„ط¥ظ†طھط§ط¬طں ظ„ط§ ظٹظ…ظƒظ† ط§ظ„طھط±ط§ط¬ط¹ ط¨ط¹ط¯ ط§ظ„ط§ط¹طھظ…ط§ط¯."
        variant="warning"
        onConfirm={() => { handleApprove(); setShowConfirm(false); }}
        onCancel={() => setShowConfirm(false)}
      />
    </div>
  );
}



