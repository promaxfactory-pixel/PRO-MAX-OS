import { useState, useEffect, useMemo, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import DataTable, { Column } from "@/components/ui/DataTable";
import Button from "@/components/ui/Button";
import LoadingSpinner from "@/components/ui/LoadingSpinner";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus } from "lucide-react";
import { useUIStore } from "../../stores/uiStore";
import { Product } from "@/types";

export default function ProductListPage() {
  const { addNotification } = useUIStore();
  const navigate = useNavigate();
  const [products, setProducts] = useState<Product[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadProducts = useCallback(async () => {
    setLoading(true);
    setError(null);
    try { const d = await invoke("list_products"); setProducts(d as Product[]); }
    catch (err) { setError(err instanceof Error ? err.message : String(err)); addNotification({ title: "خطأ", message: String(err), type: "error" }); }
    finally { setLoading(false); }
  }, [addNotification]);

  useEffect(() => { loadProducts(); }, [loadProducts]);

  if (error) return <div className="flex flex-col items-center py-16"><div className="text-6xl mb-4 text-red-400">⚠️</div><h3 className="text-lg font-semibold text-[var(--text-primary)] mb-2">حدث خطأ</h3><p className="text-[var(--text-secondary)] mb-4">{error}</p><button onClick={loadProducts} className="px-6 py-2.5 bg-brand-500 text-pure-white rounded-xl">إعادة المحاولة</button></div>;

  if (loading) return <div className="flex items-center justify-center py-16"><LoadingSpinner /></div>;

  const columns: Column<any>[] = useMemo(() => [
    { key: "code", header: "الكود", sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.code || "—"}</span> },
    { key: "name_ar", header: "الاسم", sortable: true, render: (r) => r.name_ar || r.name_en || "—" },
    { key: "size", header: "المقاس", render: (r) => r.size || "—" },
    { key: "cup_type", header: "النوع", render: (r) => r.cup_type || "—" },
    { key: "cups_per_carton", header: "كوب/كرتون", align: "center" },
    { key: "default_price_milli", header: "السعر", sortable: true, align: "left", render: (r) => formatOMR(r.default_price_milli) },
    { key: "default_cost_milli", header: "التكلفة", align: "left", render: (r) => formatOMR(r.default_cost_milli) },
    { key: "vat_pct", header: "الضريبة", align: "center", render: (r) => r.vat_pct + "%" },
  ], []);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div><h1 className="page-title">المنتجات</h1><p className="page-subtitle">{products.length} منتج نشط</p></div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => navigate("/products/new")}>منتج جديد</Button>
      </div>
      <DataTable columns={columns} data={products} loading={loading} onRowClick={(r) => navigate(`/products/${r.id}`)} emptyMessage="لا توجد منتجات" />
    </div>
  );
}
