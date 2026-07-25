import { useState, useEffect, useMemo } from "react";
import DataTable, { Column, type BadgeVariant } from "@/components/ui/DataTable";
import Badge from "@/components/ui/Badge";
import { invoke } from "@tauri-apps/api/core";
import { BookOpen } from "lucide-react";
import { useUIStore } from "../../stores/uiStore";
import { Account } from "@/types";

export default function AccountsPage() {
  const { addNotification } = useUIStore();
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => { invoke("list_accounts").then((d) => setAccounts(d as Account[])).catch((e: unknown) => addNotification({ title: 'خطأ', message: String(e), type: 'error' })).finally(() => setLoading(false)); }, []);

  const typeColors: Record<string, BadgeVariant> = { asset: 'success', liability: 'danger', equity: 'info', revenue: 'gold', expense: 'warning' };
  const typeLabels: Record<string, string> = { asset: 'أصول', liability: 'التزامات', equity: 'حقوق ملكية', revenue: 'إيرادات', expense: 'مصروفات' };

  const columns: Column<any>[] = useMemo(() => [
    { key: "code", header: "الكود", sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.code}</span> },
    { key: "name_ar", header: "الاسم بالعربية", sortable: true },
    { key: "name_en", header: "الاسم بالإنجليزية", sortable: true },
    { key: "type", header: "النوع", sortable: true, render: (r) => <Badge variant={typeColors[r.type] || "default"}>{typeLabels[r.type] || r.type}</Badge> },
    { key: "parent", header: "الأب", render: (r) => r.parent || "—" },
  ], []);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div><h1 className="page-title">دليل الحسابات</h1><p className="page-subtitle">{accounts.length} حساب</p></div>
      </div>
      <DataTable columns={columns} data={accounts} loading={loading} emptyMessage="لا توجد حسابات" />
    </div>
  );
}
