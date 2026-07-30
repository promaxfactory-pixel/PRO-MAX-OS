import { useState, useEffect, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import LoadingSpinner from "@/components/ui/LoadingSpinner";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus, Search, Truck } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { Supplier } from "@/types";

export default function SupplierListPage() {
  const navigate = useNavigate();
  const addNotification = useUIStore((s) => s.addNotification);
  const [suppliers, setSuppliers] = useState<Supplier[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [search, setSearch] = useState("");

  const loadSuppliers = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await invoke("list_suppliers");
      setSuppliers(data as Supplier[]);
    } catch (err) { setError(err instanceof Error ? err.message : String(err)); addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء تحميل البيانات" }); }
    finally { setLoading(false); }
  }, [addNotification]);

  useEffect(() => { loadSuppliers(); }, [loadSuppliers]);

  if (error) return <div className="flex flex-col items-center py-16"><div className="text-6xl mb-4 text-red-400">⚠️</div><h3 className="text-lg font-semibold text-[var(--text-primary)] mb-2">حدث خطأ</h3><p className="text-[var(--text-secondary)] mb-4">{error}</p><button onClick={loadSuppliers} className="px-6 py-2.5 bg-brand-500 text-pure-white rounded-xl">إعادة المحاولة</button></div>;

  const filtered = suppliers.filter((s) =>
    !search || s.name.includes(search) || (s.code && s.code.includes(search)) || (s.phone && s.phone.includes(search))
  );

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">الموردين</h1>
          <p className="page-subtitle">{suppliers.length} مورد مسجل</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => navigate('/suppliers/new')}>إضافة مورد</Button>
      </div>

      <div className="relative">
        <Search className="absolute right-3 top-1/2 -translate-y-1/2 w-4 h-4 text-surface-500" />
        <input type="text" placeholder="بحث بالاسم أو الكود..." value={search} onChange={(e) => setSearch(e.target.value)} className="w-full pr-10 input-field" aria-label="بحث" />
      </div>

      {loading ? (
        <LoadingSpinner size="lg" />
      ) : (
        <div className="grid grid-cols-3 gap-4">
          {filtered.map((supplier) => (
            <Card key={supplier.id} className="cursor-pointer hover:border-brand-500/50 transition-all group" onClick={() => navigate(`/suppliers/${supplier.id}`)}>
              <div className="flex items-start gap-3">
                <div className="w-10 h-10 rounded-xl bg-surface-800/80 flex items-center justify-center text-orange-400 group-hover:scale-110 transition-transform">
                  <Truck className="w-5 h-5" />
                </div>
                <div className="flex-1 min-w-0">
                  <h3 className="font-bold text-white group-hover:text-brand-300 transition-colors truncate">{supplier.name}</h3>
                  <p className="text-xs text-surface-400 font-mono">{supplier.code || "بدون كود"}</p>
                  <p className="text-xs text-surface-500 mt-1">{supplier.phone || "—"}</p>
                </div>
                <div className="text-left">
                  <p className="text-sm font-bold gradient-text">{formatOMR(supplier.balance_milli)}</p>
                  <p className="text-[10px] text-surface-500">الرصيد</p>
                </div>
              </div>
            </Card>
          ))}
          {filtered.length === 0 && (
            <div className="col-span-3 text-center py-12 text-surface-500">لا يوجد موردين</div>
          )}
        </div>
      )}
    </div>
  );
}
