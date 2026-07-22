import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import DataTable, { Column } from "@/components/ui/DataTable";
import Button from "@/components/ui/Button";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus, Users } from "lucide-react";

export default function CustomerListPage() {
  const navigate = useNavigate();
  const [customers, setCustomers] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => { loadCustomers(); }, []);

  const loadCustomers = async () => {
    setLoading(true);
    try { const d = await invoke("list_customers"); setCustomers(d as any[]); }
    catch (err) { console.error(err); }
    finally { setLoading(false); }
  };

  const columns: Column<any>[] = [
    { key: "code", header: "الكود", sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.code || "—"}</span> },
    { key: "name", header: "الاسم", sortable: true, render: (r) => <span className="font-medium">{r.name}</span> },
    { key: "phone", header: "الهاتف", render: (r) => r.phone || "—" },
    { key: "email", header: "البريد", render: (r) => r.email || "—" },
    { key: "balance_milli", header: "الرصيد", sortable: true, align: "left", render: (r) => <span className={`font-bold ${r.balance_milli > 0 ? 'text-gold-400' : r.balance_milli < 0 ? 'text-red-400' : ''}`}>{formatOMR(r.balance_milli)}</span> },
    { key: "credit_limit_milli", header: "الحد الائتماني", align: "left", render: (r) => formatOMR(r.credit_limit_milli) },
  ];

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">العملاء</h1>
          <p className="page-subtitle">{customers.length} عميل نشط</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => navigate('/customers/new')}>عميل جديد</Button>
      </div>
      <DataTable columns={columns} data={customers} loading={loading}
        onRowClick={(r) => navigate(`/customers/${r.id}`)} emptyMessage="لا يوجد عملاء" />
    </div>
  );
}
