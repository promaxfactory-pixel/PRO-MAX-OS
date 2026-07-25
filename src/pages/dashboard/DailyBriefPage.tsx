import { useState, useEffect, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import Badge from "@/components/ui/Badge";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import {
  AlertTriangle, Package, Users, FileText, Clock,
  TrendingDown, Shield, RefreshCw
} from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

interface DailyBrief {
  date: string; total_sales_milli: number; total_orders: number;
  production_summary: { product: string; qty: number }[];
  alerts: { type: string; message: string; severity: string }[];
  low_stock_items: { name: string; qty: number; reorder: number }[];
}

export default function DailyBriefPage() {
  const navigate = useNavigate();
  const addNotification = useUIStore((s) => s.addNotification);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [brief, setBrief] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  const loadBrief = useCallback(async () => {
    setLoading(true);
    try {
      const data = await invoke("get_daily_brief");
      setBrief(data);
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء تحميل البيانات" }); }
    finally { setLoading(false); }
  }, [addNotification]);

  useEffect(() => { loadBrief(); }, [loadBrief]);

  if (loading) {
    return <div className="flex items-center justify-center h-64"><div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" /></div>;
  }

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">الموجز اليومي</h1>
          <p className="page-subtitle">نظرة سريعة على أهم المؤشرات اليومية</p>
        </div>
        <button onClick={loadBrief} className="btn-ghost flex items-center gap-2"><RefreshCw className="w-4 h-4" /> تحديث</button>
      </div>

      {/* KPI Cards */}
      <div className="grid grid-cols-4 gap-4">
        <Card className="text-center">
          <FileText className="w-8 h-8 text-brand-400 mx-auto mb-2" />
          <p className="text-3xl font-bold text-white">{brief?.unpaid_count || 0}</p>
          <p className="text-sm text-surface-400">فواتير غير مدفوعة</p>
          <p className="text-lg font-bold text-gold-400 mt-1">{formatOMR(brief?.unpaid_total || 0)}</p>
        </Card>
        <Card className="text-center">
          <AlertTriangle className="w-8 h-8 text-amber-400 mx-auto mb-2" />
          <p className="text-3xl font-bold text-white">{formatOMR(brief?.overdue_total || 0)}</p>
          <p className="text-sm text-surface-400">مبالغ متأخرة</p>
        </Card>
        <Card className="text-center">
          <Package className="w-8 h-8 text-red-400 mx-auto mb-2" />
          <p className="text-3xl font-bold text-white">{brief?.waste_yesterday || 0}</p>
          <p className="text-sm text-surface-400">هالك كرتون (أمس)</p>
        </Card>
        <Card className="text-center">
          <Shield className={`w-8 h-8 mx-auto mb-2 ${brief?.backup_status === 'green' ? 'text-emerald-400' : brief?.backup_status === 'amber' ? 'text-amber-400' : 'text-red-400'}`} />
          <p className="text-3xl font-bold text-white">{brief?.last_backup_days ?? '—'}</p>
          <p className="text-sm text-surface-400">أيام من آخر نسخة احتياطية</p>
          <Badge variant={brief?.backup_status === 'green' ? 'success' : brief?.backup_status === 'amber' ? 'warning' : 'danger'}>
            {brief?.backup_status === 'green' ? 'محدث' : brief?.backup_status === 'amber' ? 'يحتاج تحديث' : 'متأخر'}
          </Badge>
        </Card>
      </div>

      {/* Detail sections */}
      <div className="grid grid-cols-2 gap-6">
        <Card>
          <h3 className="section-title"><Users className="w-5 h-5 text-gold-400" /> أكبر 5 عملاء متأخرين</h3>
          <div className="space-y-3">
            {(brief?.overdue_customers || []).length === 0 ? (
              <p className="text-sm text-surface-500 text-center py-4">لا توجد مبالغ متأخرة</p>
            ) : (brief?.overdue_customers || []).map((c: any, i: number) => (
              <div key={i} className="flex items-center justify-between py-2 border-b border-surface-700/30 last:border-0">
                <span className="text-sm text-white">{c.name}</span>
                <div className="text-left">
                  <span className="text-sm font-bold text-gold-400">{formatOMR(c.amount)}</span>
                  <span className="text-xs text-surface-400 mr-2">{c.days} يوم</span>
                </div>
              </div>
            ))}
          </div>
        </Card>

        <Card>
          <h3 className="section-title"><Package className="w-5 h-5 text-gold-400" /> أكبر 5 عناصر منخفضة</h3>
          <div className="space-y-3">
            {(brief?.low_stock || []).length === 0 ? (
              <p className="text-sm text-surface-500 text-center py-4">المخزون ممتلئ</p>
            ) : (brief?.low_stock || []).map((s: any, i: number) => (
              <div key={i} className="flex items-center justify-between py-2 border-b border-surface-700/30 last:border-0">
                <span className="text-sm text-white">{s.name}</span>
                <div className="text-left">
                  <span className="text-sm text-surface-300">{s.on_hand}</span>
                  <span className="text-xs text-surface-500 mx-1">/</span>
                  <span className="text-sm text-red-400">{s.reorder}</span>
                </div>
              </div>
            ))}
          </div>
        </Card>
      </div>
    </div>
  );
}