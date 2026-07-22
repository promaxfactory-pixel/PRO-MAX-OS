import { useState, useEffect } from "react";
import DataTable, { Column } from "@/components/ui/DataTable";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Receipt } from "lucide-react";
import { useUIStore } from "../../stores/uiStore";

export default function JournalPage() {
  const { addNotification } = useUIStore();
  const [entries, setEntries] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => { invoke("list_journal_entries").then((d: any) => setEntries(d)).catch((e: any) => addNotification({ title: 'خطأ', message: String(e), type: 'error' })).finally(() => setLoading(false)); }, []);

  const columns: Column<any>[] = [
    { key: "entry_no", header: "الرقم", sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.entry_no || "—"}</span> },
    { key: "date", header: "التاريخ", sortable: true, render: (r) => formatDate(r.date) },
    { key: "memo", header: "البيان", sortable: true },
    { key: "ref_type", header: "المرجع", render: (r) => r.ref_type ? `${r.ref_type}#${r.ref_id}` : "—" },
    { key: "created_by", header: "أنشأه" },
  ];

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div><h1 className="page-title">القيود اليومية</h1><p className="page-subtitle">{entries.length} قيد</p></div>
      </div>
      <DataTable columns={columns} data={entries} loading={loading} emptyMessage="لا توجد قيود" />
    </div>
  );
}
