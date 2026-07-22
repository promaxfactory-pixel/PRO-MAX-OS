import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus, Search, Truck } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

export default function SupplierListPage() {
  const navigate = useNavigate();
  const addNotification = useUIStore((s) => s.addNotification);
  const [suppliers, setSuppliers] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState("");

  useEffect(() => { loadSuppliers(); }, []);

  const loadSuppliers = async () => {
    setLoading(true);
    try {
      const data = await invoke("list_suppliers");
      setSuppliers(data as any[]);
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء تحميل البيانات" }); }
    finally { setLoading(false); }
  };

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
        <input type="text" placeholder="بحث بالاسم أو الكود..." value={search} onChange={(e) => setSearch(e.target.value)} className="w-full pr-10 input-field" />
      </div>

      {loading ? (
        <div className="flex items-center justify-center h-64"><div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" /></div>
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
