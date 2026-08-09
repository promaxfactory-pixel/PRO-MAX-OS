import { useState, useEffect, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import DataTable, { Column } from "@/components/ui/DataTable";
import { StatusBadge } from "@/components/ui/Badge";
import Button from "@/components/ui/Button";
import { formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus } from "lucide-react";
import { useUIStore } from "../../stores/uiStore";
import type { ProductionOrder } from "@/types";
import { useTranslation } from "react-i18next";

export default function ProductionOrderListPage() {
  const { t } = useTranslation();
  const { addNotification } = useUIStore();
  const navigate = useNavigate();
  const [orders, setOrders] = useState<ProductionOrder[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => { invoke("list_production_orders").then((d) => setOrders(d as ProductionOrder[])).catch((e: unknown) => addNotification({ title: t('common.error'), message: String(e), type: 'error' })).finally(() => setLoading(false)); }, []);

  const columns: Column<ProductionOrder>[] = useMemo(() => [
    { key: "prod_no", header: t("production.prodNo"), sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.prod_no || "—"}</span> },
    { key: "date", header: t("common.date"), sortable: true, render: (r) => formatDate(r.date) },
    { key: "shift", header: t("production.shift"), render: (r) => r.shift || "—" },
    { key: "machine_name", header: t("maintenance.sheetCreate.machine"), render: (r) => r.machine_name || "—" },
    { key: "operator", header: t("production.operator"), render: (r) => r.operator || "—" },
    { key: "supervisor", header: t("production.supervisor"), render: (r) => r.supervisor || "—" },
    { key: "run_minutes", header: t("production.runMinutes"), align: "center", render: (r) => t("production.minutesUnit", { count: r.run_minutes }) },
    { key: "status", header: t("common.status"), sortable: true, render: (r) => <StatusBadge status={r.status} /> },
  ], [t]);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div><h1 className="page-title">{t("production.orders")}</h1><p className="page-subtitle">{t("production.orderCount", { count: orders.length })}</p></div>
        <Button onClick={() => navigate('/production/new')} icon={<Plus className="w-4 h-4" />}>{t("production.newOrderButton")}</Button>
      </div>
      <DataTable columns={columns} data={orders} loading={loading} onRowClick={(r) => navigate(`/production/${r.id}`)} emptyMessage={t("production.ordersEmpty")} />
    </div>
  );
}
