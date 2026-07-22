import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import DataTable, { Column } from "@/components/ui/DataTable";
import Badge from "@/components/ui/Badge";
import Button from "@/components/ui/Button";
import { formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus, ClipboardList } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

export default function OperationsSheetListPage() {
  const navigate = useNavigate();
  const { addNotification } = useUIStore();
  const [sheets, setSheets] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    invoke("list_operations_sheets")
      .then((d: any) => setSheets(d))
      .catch((e: any) => addNotification({ title: 'خطأ', message: String(e), type: 'error' }))
      .finally(() => setLoading(false));
  }, []);

  const statusMap: Record<string, { label: string; variant: string }> = {
    draft: { label: "مسودة", variant: "warning" },
    submitted: { label: "مرسل", variant: "info" },
    approved: { label: "معتمد", variant: "success" },
    rejected: { label: "مرفوض", variant: "danger" },
  };

  const shiftLabels: Record<string, string> = {
    morning: "صباحي",
    evening: "مسائي",
    night: "ليلي",
  };

  const columns: Column<any>[] = [
    { key: "sheet_no", header: "الرقم", sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.sheet_no || "—"}</span> },
    { key: "date", header: "التاريخ", sortable: true, render: (r) => formatDate(r.date) },
    { key: "shift", header: "الوردية", render: (r) => <Badge variant="info">{shiftLabels[r.shift] || r.shift}</Badge> },
    { key: "workers_count", header: "عدد العمال", align: "left", render: (r) => <span className="font-bold">{r.workers_count || 0}</span> },
    { key: "production_output", header: "الإنتاج", align: "left", render: (r) => `${r.production_output || 0} طن` },
    { key: "status", header: "الحالة", render: (r) => { const s = statusMap[r.status] || { label: r.status, variant: "" }; return <Badge variant={s.variant as any}>{s.label}</Badge>; } },
    { key: "created_by", header: "أنشأه" },
  ];

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title"> الورش اليومية</h1>
          <p className="page-subtitle">{sheets.length} ورقة</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => navigate("/operations/new")}>
          ورقة جديدة
        </Button>
      </div>
      <DataTable
        columns={columns}
        data={sheets}
        loading={loading}
        onRowClick={(r) => navigate(`/operations/sheets/${r.id}`)}
        emptyMessage="لا توجد أوراق عمليات"
      />
    </div>
  );
}
