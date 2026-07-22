import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import DataTable, { Column } from "@/components/ui/DataTable";
import Button from "@/components/ui/Button";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus, Package } from "lucide-react";
import { useUIStore } from "../../stores/uiStore";

export default function ProductListPage() {
  const { addNotification } = useUIStore();
  const navigate = useNavigate();
  const [products, setProducts] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => { invoke("list_products").then((d: any) => setProducts(d)).catch((e: any) => addNotification({ title: 'خطأ', message: String(e), type: 'error' })).finally(() => setLoading(false)); }, []);

  const columns: Column<any>[] = [
    { key: "code", header: "الكود", sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.code || "—"}</span> },
    { key: "name_ar", header: "الاسم", sortable: true, render: (r) => r.name_ar || r.name_en || "—" },
    { key: "size", header: "المقاس", render: (r) => r.size || "—" },
    { key: "cup_type", header: "النوع", render: (r) => r.cup_type || "—" },
    { key: "cups_per_carton", header: "كوب/كرتون", align: "center" },
    { key: "default_price_milli", header: "السعر", sortable: true, align: "left", render: (r) => formatOMR(r.default_price_milli) },
    { key: "default_cost_milli", header: "التكلفة", align: "left", render: (r) => formatOMR(r.default_cost_milli) },
    { key: "vat_pct", header: "الضريبة", align: "center", render: (r) => r.vat_pct + "%" },
  ];

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div><h1 className="page-title">المنتجات</h1><p className="page-subtitle">{products.length} منتج نشط</p></div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => navigate('/products/new')}>منتج جديد</Button>
      </div>
      <DataTable columns={columns} data={products} loading={loading} onRowClick={(r) => navigate(`/products/${r.id}`)} emptyMessage="لا توجد منتجات" />
    </div>
  );
}
