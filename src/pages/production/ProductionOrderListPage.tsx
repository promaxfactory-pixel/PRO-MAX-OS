import { useState, useEffect, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import DataTable, { Column } from "@/components/ui/DataTable";
import { StatusBadge } from "@/components/ui/Badge";
import Button from "@/components/ui/Button";
import { formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus, Factory } from "lucide-react";
import { useUIStore } from "../../stores/uiStore";
import type { ProductionOrder } from "@/types";

export default function ProductionOrderListPage() {
  const { addNotification } = useUIStore();
  const navigate = useNavigate();
  const [orders, setOrders] = useState<ProductionOrder[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => { invoke("list_production_orders").then((d) => setOrders(d as ProductionOrder[])).catch((e: unknown) => addNotification({ title: 'خطأ', message: String(e), type: 'error' })).finally(() => setLoading(false)); }, []);

  const columns: Column<ProductionOrder>[] = useMemo(() => [
    { key: "prod_no", header: "رقم الأمر", sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.prod_no || "—"}</span> },
    { key: "date", header: "التاريخ", sortable: true, render: (r) => formatDate(r.date) },
    { key: "shift", header: "الوردية", render: (r) => r.shift || "—" },
    { key: "machine_name", header: "الماكينة", render: (r) => r.machine_name || "—" },
    { key: "operator", header: "المشغل", render: (r) => r.operator || "—" },
    { key: "supervisor", header: "المشرف", render: (r) => r.supervisor || "—" },
    { key: "run_minutes", header: "دقائق التشغيل", align: "center", render: (r) => r.run_minutes + " دقيقة" },
    { key: "status", header: "الحالة", sortable: true, render: (r) => <StatusBadge status={r.status} /> },
  ], []);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div><h1 className="page-title">أوامر الإنتاج</h1><p className="page-subtitle">{orders.length} أمر</p></div>
        <Button onClick={() => navigate('/production/new')} icon={<Plus className="w-4 h-4" />}>أمر جديد</Button>
      </div>
      <DataTable columns={columns} data={orders} loading={loading} onRowClick={(r) => navigate(`/production/${r.id}`)} emptyMessage="لا توجد أوامر إنتاج" />
    </div>
  );
}
