import { useState, useEffect, useMemo, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import DataTable, { Column } from "@/components/ui/DataTable";
import Button from "@/components/ui/Button";
import LoadingSpinner from "@/components/ui/LoadingSpinner";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { Customer } from "@/types";

export default function CustomerListPage() {
  const navigate = useNavigate();
  const addNotification = useUIStore((s) => s.addNotification);
  const [customers, setCustomers] = useState<Customer[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadCustomers = useCallback(async () => {
    setLoading(true);
    setError(null);
    try { const d = await invoke("list_customers"); setCustomers(d as Customer[]); }
    catch (err) { setError(err instanceof Error ? err.message : String(err)); addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء تحميل البيانات" }); }
    finally { setLoading(false); }
  }, [addNotification]);

  useEffect(() => { loadCustomers(); }, [loadCustomers]);

  if (error) return <div className="flex flex-col items-center py-16"><div className="text-6xl mb-4 text-red-400">⚠️</div><h3 className="text-lg font-semibold text-[var(--text-primary)] mb-2">حدث خطأ</h3><p className="text-[var(--text-secondary)] mb-4">{error}</p><button onClick={loadCustomers} className="px-6 py-2.5 bg-brand-500 text-pure-white rounded-xl">إعادة المحاولة</button></div>;

  if (loading) return <div className="flex items-center justify-center py-16"><LoadingSpinner /></div>;

  const columns: Column<any>[] = useMemo(() => [
    { key: "code", header: "الكود", sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.code || "—"}</span> },
    { key: "name", header: "الاسم", sortable: true, render: (r) => <span className="font-medium">{r.name}</span> },
    { key: "phone", header: "الهاتف", render: (r) => r.phone || "—" },
    { key: "email", header: "البريد", render: (r) => r.email || "—" },
    { key: "balance_milli", header: "الرصيد", sortable: true, align: "left", render: (r) => <span className={`font-bold ${r.balance_milli > 0 ? "text-gold-400" : r.balance_milli < 0 ? "text-red-400" : ""}`}>{formatOMR(r.balance_milli)}</span> },
    { key: "credit_limit_milli", header: "الحد الائتماني", align: "left", render: (r) => formatOMR(r.credit_limit_milli) },
  ], []);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">العملاء</h1>
          <p className="page-subtitle">{customers.length} عميل نشط</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => navigate("/customers/new")}>عميل جديد</Button>
      </div>
      <DataTable columns={columns} data={customers} loading={loading}
        onRowClick={(r) => navigate(`/customers/${r.id}`)} emptyMessage="لا يوجد عملاء" />
    </div>
  );
}
