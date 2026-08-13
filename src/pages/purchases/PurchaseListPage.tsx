import { useState, useEffect, useMemo, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import DataTable, { Column } from "@/components/ui/DataTable";
import Button from "@/components/ui/Button";
import { StatCard } from "@/components/ui/Card";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@/lib/tauri";
import { ShoppingCart } from "lucide-react";
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
  const navigate = useNavigate();
  const addNotification = useUIStore((s) => s.addNotification);
  const [purchases, setPurchases] = useState<PurchaseOrder[]>([]);
  const [loading, setLoading] = useState(true);

  const loadPurchases = useCallback(async () => {
    setLoading(true);
    try { const d = await invoke("list_purchases"); setPurchases(d as PurchaseOrder[]); }
    catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء تحميل البيانات" }); }
    finally { setLoading(false); }
  }, [addNotification]);

  useEffect(() => { loadPurchases(); }, [loadPurchases]);

  const postPurchase = useCallback(async (id: number) => {
    if (!window.confirm("ترحيل هذه المشتريات إلى دفتر الأستاذ؟")) return;
    try {
      await invoke("post_purchase", { id });
      addNotification({ id: crypto.randomUUID(), type: "success", title: "تم الترحيل", message: "تم ترحيل المشتريات إلى دفتر الأستاذ" });
      loadPurchases();
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) });
    }
  }, [addNotification, loadPurchases]);

  const voidPurchase = useCallback(async (id: number) => {
    const reason = window.prompt("سبب الإلغاء (اختياري):");
    if (reason === null) return; // cancelled
    if (!window.confirm("إلغاء هذه المشتريات؟ سيتم عكس قيد الأستاذ ورد البضاعة للمخزون.")) return;
    try {
      await invoke("void_purchase", { id, reason: reason || null });
      addNotification({ id: crypto.randomUUID(), type: "success", title: "تم الإلغاء", message: "تم إلغاء المشتريات وعكس القيد" });
      loadPurchases();
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) });
    }
  }, [addNotification, loadPurchases]);

  const activePurchases = purchases.filter((p) => p.status?.toLowerCase() !== "void");
  const totalAmount = activePurchases.reduce((s: number, p: any) => s + (p.total_milli || 0), 0);
  const totalPaid = activePurchases.reduce((s: number, p: any) => s + (p.paid_milli || 0), 0);

  const columns: Column<PurchaseOrder>[] = useMemo(() => [
    { key: "pur_no", header: "رقم المشتريات", sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.pur_no || "—"}</span> },
    { key: "date", header: "التاريخ", sortable: true, render: (r) => formatDate(r.date) },
    { key: "supplier_name", header: "المورد", render: (r) => r.supplier_name || "—" },
    { key: "total_milli", header: "المبلغ", sortable: true, align: "left", render: (r) => <span className="font-bold text-gold-400">{formatOMR(r.total_milli)}</span> },
    { key: "paid_milli", header: "المدفوع", align: "left", render: (r) => formatOMR(r.paid_milli) },
    { key: "status", header: "الحالة", render: (r) => {
      const posted = r.status?.toLowerCase() === "posted";
      const draft = r.status?.toLowerCase() === "draft";
      const voided = r.status?.toLowerCase() === "void";
      return (
        <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
          voided ? "bg-red-500/20 text-red-400" :
            posted ? "bg-emerald-500/20 text-emerald-400" :
              draft ? "bg-yellow-500/20 text-yellow-400" :
                "bg-surface-600 text-surface-300"
        }`}>{voided ? "ملغي" : posted ? "مرحّل" : draft ? "مسودة" : r.status}</span>
      );
    }},
    { key: "actions", header: "الإجراءات", render: (r) => {
      const voided = r.status?.toLowerCase() === "void";
      if (voided) return <span className="text-xs text-red-400">ملغي</span>;
      const posted = r.status?.toLowerCase() === "posted";
      return (
        <div className="flex items-center gap-2">
          {!posted ? (
            <Button size="sm" variant="success" onClick={() => postPurchase(r.id)}>ترحيل</Button>
          ) : (
            <span className="text-xs text-emerald-400">تم الترحيل</span>
          )}
          <Button size="sm" variant="outline" className="!text-red-400 !border-red-500/40" onClick={() => voidPurchase(r.id)}>إلغاء</Button>
        </div>
      );
    }},
  ], [postPurchase, voidPurchase]);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">المشتريات</h1>
          <p className="page-subtitle">{purchases.length} مشتريات</p>
        </div>
        <Button icon={<ShoppingCart className="w-4 h-4" />} onClick={() => navigate("/purchases/new")}>مشتريات جديدة</Button>
      </div>
      <div className="grid grid-cols-3 gap-4">
        <StatCard title="إجمالي المشتريات" value={formatOMR(totalAmount)} icon={<ShoppingCart className="w-6 h-6" />} />
        <StatCard title="المدفوع" value={formatOMR(totalPaid)} icon={<ShoppingCart className="w-6 h-6" />} />
        <StatCard title="المتبقي" value={formatOMR(totalAmount - totalPaid)} icon={<ShoppingCart className="w-6 h-6" />} />
      </div>
      <DataTable columns={columns} data={purchases} loading={loading}
        emptyMessage="لا توجد مشتريات" />
    </div>
  );
}
