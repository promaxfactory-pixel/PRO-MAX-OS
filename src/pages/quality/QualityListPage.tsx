import { useState, useEffect, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import DataTable, { Column, type BadgeVariant } from "@/components/ui/DataTable";
import Badge from "@/components/ui/Badge";
import Button from "@/components/ui/Button";
import { formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus, AlertCircle } from "lucide-react";

interface QualityInspection {
  id: number;
  inspection_no: string;
  date: string;
  inspector: string;
  production_line_id: number;
  result: string;
  defect_type: string;
  defect_qty: number;
  status: string;
}

export default function QualityListPage() {
  const navigate = useNavigate();
  const [inspections, setInspections] = useState<QualityInspection[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);

  useEffect(() => {
    invoke("list_quality_inspections")
      .then((d) => setInspections(d as QualityInspection[]))
      .catch(() => setError(true))
      .finally(() => setLoading(false));
  }, []);

  const resultMap: Record<string, { label: string; variant: BadgeVariant }> = {
    pass: { label: "ناجح", variant: "success" },
    fail: { label: "غير ناجح", variant: "danger" },
    conditional: { label: "مشروط", variant: "warning" },
  };


  const statusMap: Record<string, { label: string; variant: BadgeVariant }> = {
    open: { label: "مفتوح", variant: "info" },
    in_progress: { label: "قيد المراجعة", variant: "warning" },
    closed: { label: "مغلق", variant: "success" },
    rejected: { label: "مرفوض", variant: "danger" },
  };

  const columns: Column<QualityInspection>[] = useMemo(() => [
    { key: "inspection_no", header: "رقم الفحص", sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.inspection_no || "—"}</span> },
    { key: "date", header: "التاريخ", sortable: true, render: (r) => formatDate(r.date) },
    { key: "inspector", header: "الفاحص", sortable: true, render: (r) => <span className="font-medium">{r.inspector || "—"}</span> },
    { key: "production_line_id", header: "خط الإنتاج", align: "center", render: (r) => r.production_line_id || "—" },
    { key: "result", header: "النتيجة", render: (r) => { const s = resultMap[r.result] || { label: r.result, variant: "default" as BadgeVariant }; return <Badge variant={s.variant}>{s.label}</Badge>; } },
    { key: "defect_type", header: "نوع العيب", render: (r) => r.defect_type || "—" },
    { key: "defect_qty", header: "كمية العيب", align: "left", render: (r) => r.defect_qty ? <span className="font-bold text-red-400">{r.defect_qty}</span> : "0" },
    { key: "status", header: "الحالة", render: (r) => { const s = statusMap[r.status] || { label: r.status, variant: "default" as BadgeVariant }; return <Badge variant={s.variant}>{s.label}</Badge>; } },
  ], []);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">فحوصات الجودة</h1>
          <p className="page-subtitle">{inspections.length} فحص مسجل</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />}>فحص جديد</Button>
      </div>

      {error ? (
        <div className="flex flex-col items-center justify-center h-64 text-surface-400">
          <AlertCircle className="w-12 h-12 mb-4 text-surface-500" />
          <p className="text-lg font-medium">فحوصات الجودة قيد التطوير</p>
          <p className="text-sm text-surface-500 mt-1">سيتم إضافة هذه الميزة قريباً</p>
        </div>
      ) : (
        <DataTable
          columns={columns}
          data={inspections}
          loading={loading}
          onRowClick={(r) => navigate(`/quality/${r.id}`)}
          emptyMessage="لا توجد فحوصات جودة"
        />
      )}
    </div>
  );
}
