import { useState, useEffect, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import DataTable, { Column } from "@/components/ui/DataTable";
import Button from "@/components/ui/Button";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus } from "lucide-react";
import { useUIStore } from "../../stores/uiStore";
import { Product } from "@/types";

export default function ProductListPage() {
  const { t } = useTranslation();
  const { addNotification } = useUIStore();
  const navigate = useNavigate();
  const [products, setProducts] = useState<Product[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => { invoke("list_products").then((d) => setProducts(d as Product[])).catch((e: unknown) => addNotification({ title: t('common.error'), message: String(e), type: 'error' })).finally(() => setLoading(false)); }, [t]);

  const columns: Column<any>[] = useMemo(() => [
    { key: "code", header: t("inventory.code"), sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.code || "—"}</span> },
    { key: "name_ar", header: t("inventory.name"), sortable: true, render: (r) => r.name_ar || r.name_en || "—" },
    { key: "size", header: t("productList.size"), render: (r) => r.size || "—" },
    { key: "cup_type", header: t("inventory.kind"), render: (r) => r.cup_type || "—" },
    { key: "cups_per_carton", header: t("print.cupsPerCarton"), align: "center" },
    { key: "default_price_milli", header: t("productList.price"), sortable: true, align: "left", render: (r) => formatOMR(r.default_price_milli) },
    { key: "default_cost_milli", header: t("productList.cost"), align: "left", render: (r) => formatOMR(r.default_cost_milli) },
    { key: "vat_pct", header: t("common.vat"), align: "center", render: (r) => r.vat_pct + "%" },
  ], [t]);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div><h1 className="page-title">{t("nav.products")}</h1><p className="page-subtitle">{t("productList.activeProducts", { count: products.length })}</p></div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => navigate('/products/new')}>{t("productList.newProduct")}</Button>
      </div>
      <DataTable columns={columns} data={products} loading={loading} onRowClick={(r) => navigate(`/products/${r.id}`)} emptyMessage={t("productList.empty")} />
    </div>
  );
}
