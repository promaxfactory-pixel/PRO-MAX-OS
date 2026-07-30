import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { StatusBadge } from "@/components/ui/Badge";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, Phone, Mail, MapPin, Edit, FileText, Banknote } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { Customer } from "@/types";

export default function CustomerDetailPage() {
  const { id } = useParams();
  const navigate = useNavigate();
  const addNotification = useUIStore((s) => s.addNotification);
  const [customer, setCustomer] = useState<Customer | null>(null);
  
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);
  useEffect(() => {
    invoke("get_customer", { id: Number(id) }).then((d) => setCustomer(d as Customer)).catch((e: unknown) => { const msg = String(e); setLoadError(msg); addNotification({ title: 'ط®ط·ط£', message: String(e), type: 'error' }); }).finally(() => setLoading(false));
  }, [id]);

  if (loading) {
    return <div className="flex items-center justify-center h-64"><div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" /></div>;
  }

  if (loadError || !customer) {
    return <div className="flex flex-col items-center justify-center h-64 gap-4"><p className="text-surface-400">تعذر تحميل بيانات العميل</p><button className="btn-outline px-4 py-2 rounded-xl text-sm" onClick={() => window.location.reload()}>إعادة المحاولة</button></div>;
  }
  return (
    <div className="space-y-6">
      <div className="page-header">
        <div className="flex items-center gap-3">
          <button onClick={() => navigate('/customers')} className="btn-ghost p-2"><ArrowRight className="w-5 h-5" /></button>
          <div>
            <h1 className="page-title">{customer.name}</h1>
            <p className="page-subtitle font-mono">{customer.code || "ط¨ط¯ظˆظ† ظƒظˆط¯"}</p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" icon={<FileText className="w-4 h-4" />} onClick={() => navigate(`/customers/${id}/statement`)}>ظƒط´ظپ ط­ط³ط§ط¨</Button>
          <Button variant="gold" icon={<Banknote className="w-4 h-4" />} onClick={() => navigate(`/customers/${id}/pay`)}>طھط³ط¬ظٹظ„ ط¯ظپط¹ط©</Button>
          <Button variant="outline" icon={<Edit className="w-4 h-4" />} onClick={() => navigate(`/customers/${id}/edit`)}>طھط¹ط¯ظٹظ„</Button>
        </div>
      </div>

      <div className="grid grid-cols-3 gap-6">
        <Card className="col-span-2">
          <div className="grid grid-cols-2 gap-6">
            <div>
              <h4 className="text-sm text-surface-400 mb-3">ظ…ط¹ظ„ظˆظ…ط§طھ ط§ظ„ط§طھطµط§ظ„</h4>
              <div className="space-y-2">
                <div className="flex items-center gap-2 text-sm"><Phone className="w-4 h-4 text-surface-500" /> {customer.phone || "â€”"}</div>
                <div className="flex items-center gap-2 text-sm"><Mail className="w-4 h-4 text-surface-500" /> {customer.email || "â€”"}</div>
                <div className="flex items-center gap-2 text-sm"><MapPin className="w-4 h-4 text-surface-500" /> {customer.address || "â€”"}</div>
              </div>
            </div>
            <div>
              <h4 className="text-sm text-surface-400 mb-3">ظ…ط¹ظ„ظˆظ…ط§طھ ظ…ط§ظ„ظٹط©</h4>
              <div className="space-y-2">
                <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„ط±طµظٹط¯</span><span className="font-bold gradient-text">{formatOMR(customer.balance_milli)}</span></div>
                <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„ط­ط¯ ط§ظ„ط§ط¦طھظ…ط§ظ†ظٹ</span><span>{formatOMR(customer.credit_limit_milli)}</span></div>
                <div className="flex justify-between text-sm"><span className="text-surface-400">ط§ظ„ظ†ظˆط¹</span><StatusBadge status={customer.ctype || "credit"} /></div>
                <div className="flex justify-between text-sm"><span className="text-surface-400">ط±ظ‚ظ… ط¶ط±ظٹط¨ط© ط§ظ„ظ‚ظٹظ…ط© ط§ظ„ظ…ط¶ط§ظپط©</span><span className="font-mono text-xs">{customer.vat_number || "â€”"}</span></div>
              </div>
            </div>
          </div>
        </Card>
        <Card>
          <h4 className="text-sm text-surface-400 mb-3">ظ…ظ„ط®طµ</h4>
          <div className="space-y-4">
            <div className="text-center py-4">
              <p className="text-3xl font-bold gradient-text">{formatOMR(customer.balance_milli)}</p>
              <p className="text-xs text-surface-400 mt-1">ط§ظ„ط±طµظٹط¯ ط§ظ„ط­ط§ظ„ظٹ</p>
            </div>
            <div className="text-center py-2 bg-surface-900/50 rounded-xl">
              <p className="text-sm font-medium">{formatOMR(customer.credit_limit_milli)}</p>
              <p className="text-xs text-surface-400">ط§ظ„ط­ط¯ ط§ظ„ط§ط¦طھظ…ط§ظ†ظٹ</p>
            </div>
          </div>
        </Card>
      </div>
    </div>
  );
}




