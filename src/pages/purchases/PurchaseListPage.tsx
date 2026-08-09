import { useState, useEffect, useMemo, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import DataTable, { Column } from "@/components/ui/DataTable";
import Button from "@/components/ui/Button";
import { StatCard } from "@/components/ui/Card";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { ShoppingCart } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useUIStore } from "@/stores/uiStore";

interface PurchaseOrder {
  id: number;
  pur_no: string;
  date: string;
  supplier_name: string;
  total_milli: number;
  paid_milli: number;
  status: string;
}

export default function PurchaseListPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const addNotification = useUIStore((s) => s.addNotification);
  const [purchases, setPurchases] = useState<PurchaseOrder[]>([]);
  const [loading, setLoading] = useState(true);

  const loadPurchases = useCallback(async () => {
    setLoading(true);
    try { const d = await invoke("list_purchases"); setPurchases(d as PurchaseOrder[]); }
    catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("purchase.loadError") }); }
    finally { setLoading(false); }
  }, [addNotification, t]);

  useEffect(() => { loadPurchases(); }, [loadPurchases]);

  const totalAmount = purchases.reduce((s: number, p: any) => s + (p.total_milli || 0), 0);
  const totalPaid = purchases.reduce((s: number, p: any) => s + (p.paid_milli || 0), 0);

  const columns: Column<PurchaseOrder>[] = useMemo(() => [
    { key: "pur_no", header: t("purchase.purchaseNo"), sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.pur_no || "—"}</span> },
    { key: "date", header: t("common.date"), sortable: true, render: (r) => formatDate(r.date) },
    { key: "supplier_name", header: t("purchase.supplier"), render: (r) => r.supplier_name || "—" },
    { key: "total_milli", header: t("purchase.amount"), sortable: true, align: "left", render: (r) => <span className="font-bold text-gold-400">{formatOMR(r.total_milli)}</span> },
    { key: "paid_milli", header: t("common.paid"), align: "left", render: (r) => formatOMR(r.paid_milli) },
    { key: "status", header: t("common.status"), render: (r) => (
      <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
        r.status === 'Posted' ? 'bg-emerald-500/20 text-emerald-400' :
        r.status === 'Draft' ? 'bg-yellow-500/20 text-yellow-400' :
        'bg-surface-600 text-surface-300'
      }`}>{r.status === 'Posted' ? t("purchase.posted") : r.status === 'Draft' ? t("purchase.draft") : r.status}</span>
    )},
  ], [t]);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">{t("purchase.title")}</h1>
          <p className="page-subtitle">{t("purchase.subtitle", { count: purchases.length })}</p>
        </div>
        <Button icon={<ShoppingCart className="w-4 h-4" />} onClick={() => navigate('/purchases/new')}>{t("purchase.newPurchase")}</Button>
      </div>
      <div className="grid grid-cols-3 gap-4">
        <StatCard title={t("purchase.totalPurchases")} value={formatOMR(totalAmount)} icon={<ShoppingCart className="w-6 h-6" />} />
        <StatCard title={t("common.paid")} value={formatOMR(totalPaid)} icon={<ShoppingCart className="w-6 h-6" />} />
        <StatCard title={t("common.remaining")} value={formatOMR(totalAmount - totalPaid)} icon={<ShoppingCart className="w-6 h-6" />} />
      </div>
      <DataTable columns={columns} data={purchases} loading={loading}
        onRowClick={(r) => navigate(`/purchases/${r.id}`)} emptyMessage={t("purchase.empty")} />
    </div>
  );
}
