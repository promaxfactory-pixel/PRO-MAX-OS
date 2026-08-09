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
import type { ProductionLine } from "@/types";
import { useTranslation } from "react-i18next";

export default function ProductionOrderDetailPage() {
  const { t } = useTranslation();
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
    ]).then(([o, l]) => { setOrder(o); setLines(l as ProductionLine[]); }).catch((e: unknown) => addNotification({ title: t('common.error'), message: String(e), type: 'error' })).finally(() => setLoading(false));
  }, [id]);

  const handleApprove = async () => {
    await invoke("approve_production_order", { id: Number(id) });
    invoke("get_production_order", { id: Number(id) }).then((o) => setOrder(o));
  };

  if (loading || !order) return <div className="flex items-center justify-center h-64"><div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" /></div>;

  const cols: Column<ProductionLine>[] = useMemo(() => [
    { key: "product_name", header: t("production.product") },
    { key: "cartons_good", header: t("production.goodCartons"), align: "center" },
    { key: "cups_good", header: t("production.goodCups"), align: "center" },
    { key: "cartons_waste", header: t("production.wasteCartonsDetail"), align: "center", render: (r) => <span className="text-red-400">{r.cartons_waste}</span> },
    { key: "worker", header: t("production.worker") },
    { key: "brand_type", header: t("production.brandType") },
    { key: "quality_status", header: t("production.quality"), render: (r) => r.quality_status || "—" },
  ], [t]);

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
            <p className="page-subtitle">{formatDate(order.date)} • {order.shift} • {order.operator}</p>
          </div>
        </div>
        {order.status === "Draft" && (
          <div className="flex gap-2">
            <Button onClick={() => setShowConfirm(true)} icon={<Check className="w-4 h-4" />}>{t("production.approve")}</Button>
            <Button variant="danger" icon={<Ban className="w-4 h-4" />}>{t("common.cancel")}</Button>
          </div>
        )}
      </div>
      <div className="grid grid-cols-4 gap-4">
        <Card className="text-center"><p className="text-2xl font-bold text-white">{order.run_minutes}</p><p className="text-xs text-surface-400">{t("production.runMinutesLabel")}</p></Card>
        <Card className="text-center"><p className="text-2xl font-bold text-amber-400">{order.downtime_minutes}</p><p className="text-xs text-surface-400">{t("production.downtimeMinutesLabel")}</p></Card>
        <Card className="text-center"><p className="text-2xl font-bold text-emerald-400">{lines.reduce((s, l) => s + (l.cartons_good || 0), 0)}</p><p className="text-xs text-surface-400">{t("production.goodCartons")}</p></Card>
        <Card className="text-center"><p className="text-2xl font-bold text-red-400">{lines.reduce((s, l) => s + (l.cartons_waste || 0), 0)}</p><p className="text-xs text-surface-400">{t("production.waste")}</p></Card>
      </div>
      <DataTable columns={cols} data={lines} compact emptyMessage={t("production.noLines")} />

      <ConfirmDialog
        open={showConfirm}
        title={t("production.approveDialogTitle")}
        message={t("production.approveDialogMessage")}
        variant="warning"
        onConfirm={() => { handleApprove(); setShowConfirm(false); }}
        onCancel={() => setShowConfirm(false)}
      />
    </div>
  );
}
