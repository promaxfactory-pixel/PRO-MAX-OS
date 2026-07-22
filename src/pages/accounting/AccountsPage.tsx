import { useState, useEffect } from "react";
import DataTable, { Column } from "@/components/ui/DataTable";
import Badge from "@/components/ui/Badge";
import { invoke } from "@tauri-apps/api/core";
import { BookOpen } from "lucide-react";

export default function AccountsPage() {
  const [accounts, setAccounts] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => { invoke("list_accounts").then((d: any) => setAccounts(d)).catch(console.error).finally(() => setLoading(false)); }, []);

  const typeColors: Record<string, string> = { asset: 'badge-success', liability: 'badge-danger', equity: 'badge-info', revenue: 'badge-gold', expense: 'badge-warning' };
  const typeLabels: Record<string, string> = { asset: 'أصول', liability: 'التزامات', equity: 'حقوق ملكية', revenue: 'إيرادات', expense: 'مصروفات' };

  const columns: Column<any>[] = [
    { key: "code", header: "الكود", sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.code}</span> },
    { key: "name_ar", header: "الاسم بالعربية", sortable: true },
    { key: "name_en", header: "الاسم بالإنجليزية", sortable: true },
    { key: "type", header: "النوع", sortable: true, render: (r) => <Badge variant={typeColors[r.type] as any}>{typeLabels[r.type] || r.type}</Badge> },
    { key: "parent", header: "الأب", render: (r) => r.parent || "—" },
  ];

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div><h1 className="page-title">دليل الحسابات</h1><p className="page-subtitle">{accounts.length} حساب</p></div>
      </div>
      <DataTable columns={columns} data={accounts} loading={loading} emptyMessage="لا توجد حسابات" />
    </div>
  );
}
