import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, Package } from "lucide-react";
import { useUIStore } from "../../stores/uiStore";
import { Product } from "@/types";

export default function ProductDetailPage() {
  const { addNotification } = useUIStore();
  const { id } = useParams();
  const navigate = useNavigate();
  const [product, setProduct] = useState<Product | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    invoke("get_product", { id: Number(id) }).then((d) => setProduct(d as Product)).catch((e: unknown) => addNotification({ title: "ط®ط·ط£", message: String(e), type: "error" })).finally(() => setLoading(false));
  }, [id]);

  if (loading) return <div className="flex items-center justify-center h-64"><div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" /></div>;

  if (!product) return <div className="flex flex-col items-center justify-center h-64 gap-4"><p className="text-surface-400">تعذر تحميل بيانات المنتج</p><button className="btn-outline px-4 py-2 rounded-xl text-sm" onClick={() => window.location.reload()}>إعادة المحاولة</button></div>;

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div className="flex items-center gap-3">
          <button onClick={() => navigate("/products")} className="btn-ghost p-2"><ArrowRight className="w-5 h-5" /></button>
          <div>
            <h1 className="page-title">{product.name_ar || product.name_en}</h1>
            <p className="page-subtitle font-mono">{product.code}</p>
          </div>
        </div>
      </div>
      <div className="grid grid-cols-3 gap-6">
        <Card className="col-span-2">
          <div className="grid grid-cols-2 gap-6">
            <div className="space-y-3">
              <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„ظ…ظ‚ط§ط³</span><span>{product.size || "â€”"}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">ظ†ظˆط¹ ط§ظ„ظƒظˆط¨</span><span>{product.cup_type || "â€”"}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">ظƒظˆط¨ ظپظٹ ط§ظ„ظƒط±طھظˆظ†</span><span>{product.cups_per_carton}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">ظ†ظˆط¹ ط§ظ„ظƒط±طھظˆظ†</span><span>{product.carton_type || "â€”"}</span></div>
            </div>
            <div className="space-y-3">
              <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„ط³ط¹ط± ط§ظ„ط§ظپطھط±ط§ط¶ظٹ</span><span className="font-bold gradient-text">{formatOMR(product.default_price_milli)}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„طھظƒظ„ظپط©</span><span>{formatOMR(product.default_cost_milli)}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„ظ‡ط§ظ…ط´</span><span className={product.default_price_milli > product.default_cost_milli ? "text-emerald-400" : "text-red-400"}>{product.default_price_milli > 0 ? ((1 - product.default_cost_milli / product.default_price_milli) * 100).toFixed(1) : "0"}%</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„ط¨ط§ط±ظƒظˆط¯</span><span className="font-mono text-xs">{product.barcode || "â€”"}</span></div>
            </div>
          </div>
        </Card>
        <Card className="text-center">
          <Package className="w-12 h-12 text-brand-400 mx-auto mb-3" />
          <p className="text-3xl font-bold gradient-text">{product.vat_pct}%</p>
          <p className="text-xs text-surface-400">ط¶ط±ظٹط¨ط© ط§ظ„ظ‚ظٹظ…ط© ط§ظ„ظ…ط¶ط§ظپط©</p>
        </Card>
      </div>
    </div>
  );
}

