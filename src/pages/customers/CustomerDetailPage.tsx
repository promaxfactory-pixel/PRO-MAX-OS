import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { StatusBadge } from "@/components/ui/Badge";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, Phone, Mail, MapPin, Edit, FileText, Banknote } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

export default function CustomerDetailPage() {
  const { id } = useParams();
  const navigate = useNavigate();
  const addNotification = useUIStore((s) => s.addNotification);
  const [customer, setCustomer] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    invoke("get_customer", { id: Number(id) }).then((d) => setCustomer(d)).catch((e: any) => addNotification({ title: 'خطأ', message: String(e), type: 'error' })).finally(() => setLoading(false));
  }, [id]);

  if (loading || !customer) {
    return <div className="flex items-center justify-center h-64"><div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" /></div>;
  }

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div className="flex items-center gap-3">
          <button onClick={() => navigate('/customers')} className="btn-ghost p-2"><ArrowRight className="w-5 h-5" /></button>
          <div>
            <h1 className="page-title">{customer.name}</h1>
            <p className="page-subtitle font-mono">{customer.code || "بدون كود"}</p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" icon={<FileText className="w-4 h-4" />} onClick={() => navigate(`/customers/${id}/statement`)}>كشف حساب</Button>
          <Button variant="gold" icon={<Banknote className="w-4 h-4" />} onClick={() => navigate(`/customers/${id}/pay`)}>تسجيل دفعة</Button>
          <Button variant="outline" icon={<Edit className="w-4 h-4" />} onClick={() => navigate(`/customers/${id}/edit`)}>تعديل</Button>
        </div>
      </div>

      <div className="grid grid-cols-3 gap-6">
        <Card className="col-span-2">
          <div className="grid grid-cols-2 gap-6">
            <div>
              <h4 className="text-sm text-surface-400 mb-3">معلومات الاتصال</h4>
              <div className="space-y-2">
                <div className="flex items-center gap-2 text-sm"><Phone className="w-4 h-4 text-surface-500" /> {customer.phone || "—"}</div>
                <div className="flex items-center gap-2 text-sm"><Mail className="w-4 h-4 text-surface-500" /> {customer.email || "—"}</div>
                <div className="flex items-center gap-2 text-sm"><MapPin className="w-4 h-4 text-surface-500" /> {customer.address || "—"}</div>
              </div>
            </div>
            <div>
              <h4 className="text-sm text-surface-400 mb-3">معلومات مالية</h4>
              <div className="space-y-2">
                <div className="flex justify-between text-sm"><span className="text-surface-400">الرصيد</span><span className="font-bold gradient-text">{formatOMR(customer.balance_milli)}</span></div>
                <div className="flex justify-between text-sm"><span className="text-surface-400">الحد الائتماني</span><span>{formatOMR(customer.credit_limit_milli)}</span></div>
                <div className="flex justify-between text-sm"><span className="text-surface-400">النوع</span><StatusBadge status={customer.ctype || "credit"} /></div>
                <div className="flex justify-between text-sm"><span className="text-surface-400">رقم ضريبة القيمة المضافة</span><span className="font-mono text-xs">{customer.vat_number || "—"}</span></div>
              </div>
            </div>
          </div>
        </Card>
        <Card>
          <h4 className="text-sm text-surface-400 mb-3">ملخص</h4>
          <div className="space-y-4">
            <div className="text-center py-4">
              <p className="text-3xl font-bold gradient-text">{formatOMR(customer.balance_milli)}</p>
              <p className="text-xs text-surface-400 mt-1">الرصيد الحالي</p>
            </div>
            <div className="text-center py-2 bg-surface-900/50 rounded-xl">
              <p className="text-sm font-medium">{formatOMR(customer.credit_limit_milli)}</p>
              <p className="text-xs text-surface-400">الحد الائتماني</p>
            </div>
          </div>
        </Card>
      </div>
    </div>
  );
}
