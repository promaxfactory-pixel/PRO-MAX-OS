import { useState, useEffect, useCallback } from "react";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { useUIStore } from "@/stores/uiStore";
import {
  AlertTriangle,
  Bell,
  Clock,
  FileWarning,
  Shield,
  RefreshCw,
  CheckCircle,
  Info,
} from "lucide-react";

interface AlertsData {
  expiry: { product_name: string; expiry_date: string; batch?: string }[];
  overdue_orders: { order_id: number; product_name: string; due_date: string }[];
  low_stock: { product_name: string; current_stock: number; min_stock: number }[];
  overdue_invoices: { invoice_id: number; invoice_number: string; due_date: string; amount: number }[];
  quality_pending: { batch_id: number; product_name: string; created_at: string }[];
}

const sectionConfig = [
  {
    key: "expiry" as const,
    title: "ط§ظ†طھظ‡ط§ط، طµظ„ط§ط­ظٹط©",
    icon: Clock,
    borderColor: "border-l-red-500",
    iconColor: "text-red-400",
    badgeColor: "bg-red-500/20 text-red-400",
  },
  {
    key: "overdue_orders" as const,
    title: "ط£ظˆط§ظ…ط± ط¥ظ†طھط§ط¬ ظ…طھط£ط®ط±ط©",
    icon: FileWarning,
    borderColor: "border-l-yellow-500",
    iconColor: "text-yellow-400",
    badgeColor: "bg-yellow-500/20 text-yellow-400",
  },
  {
    key: "low_stock" as const,
    title: "ظ…ط®ط²ظˆظ† ظ…ظ†ط®ظپط¶",
    icon: AlertTriangle,
    borderColor: "border-l-amber-500",
    iconColor: "text-amber-400",
    badgeColor: "bg-amber-500/20 text-amber-400",
  },
  {
    key: "overdue_invoices" as const,
    title: "ظپظˆط§طھظٹط± ظ…طھط£ط®ط±ط©",
    icon: FileWarning,
    borderColor: "border-l-red-500",
    iconColor: "text-red-400",
    badgeColor: "bg-red-500/20 text-red-400",
  },
  {
    key: "quality_pending" as const,
    title: "ط¬ظˆط¯ط© ظ…ط¹ظ„ظ‚ط©",
    icon: Shield,
    borderColor: "border-l-blue-500",
    iconColor: "text-blue-400",
    badgeColor: "bg-blue-500/20 text-blue-400",
  },
] as const;

export default function AlertCenterPage() {
  const addNotification = useUIStore((s) => s.addNotification);
  const [alerts, setAlerts] = useState<AlertsData | null>(null);
  const [loading, setLoading] = useState(true);

  const fetchAlerts = useCallback(async () => {
    setLoading(true);
    try {
      const data = await invoke<AlertsData>("get_all_alerts");
      setAlerts(data);
    } catch {
      setAlerts(null);
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "فشل تحميل التنبيهات" });
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchAlerts();
  }, [fetchAlerts]);

  const counts = alerts
    ? {
      total:
          alerts.expiry.length +
          alerts.overdue_orders.length +
          alerts.low_stock.length +
          alerts.overdue_invoices.length +
          alerts.quality_pending.length,
      critical: alerts.expiry.length + alerts.overdue_invoices.length,
      info: alerts.overdue_orders.length + alerts.low_stock.length + alerts.quality_pending.length,
    }
    : { total: 0, critical: 0, info: 0 };

  const isEmpty = alerts && counts.total === 0;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="rounded-xl bg-amber-500/10 p-2.5">
            <Bell className="h-6 w-6 text-amber-400" />
          </div>
          <div>
            <h1 className="text-xl font-bold text-white">ظ…ط±ظƒط² ط§ظ„طھظ†ط¨ظٹظ‡ط§طھ</h1>
            <p className="text-sm text-surface-400">ظ…ط±ط§ظ‚ط¨ط© ط¬ظ…ظٹط¹ ط§ظ„طھظ†ط¨ظٹظ‡ط§طھ ظˆط§ظ„ط¥ط´ط¹ط§ط±ط§طھ</p>
          </div>
        </div>
        <Button
          variant="outline"
          onClick={fetchAlerts}
          disabled={loading}
          className="border-surface-700 text-surface-400 hover:text-white"
        >
          <RefreshCw className={`h-4 w-4 ml-2 ${loading ? "animate-spin" : ""}`} />
          طھط­ط¯ظٹط«
        </Button>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <div className="bg-surface-800 border border-surface-700 rounded-xl p-4 flex items-center gap-3">
          <div className="rounded-lg bg-brand-500/10 p-2">
            <Bell className="h-5 w-5 text-brand-400" />
          </div>
          <div>
            <p className="text-2xl font-bold text-white">{counts.total}</p>
            <p className="text-xs text-surface-400">ط¥ط¬ظ…ط§ظ„ظٹ ط§ظ„طھظ†ط¨ظٹظ‡ط§طھ</p>
          </div>
        </div>
        <div className="bg-surface-800 border border-surface-700 rounded-xl p-4 flex items-center gap-3">
          <div className="rounded-lg bg-red-500/10 p-2">
            <AlertTriangle className="h-5 w-5 text-red-400" />
          </div>
          <div>
            <p className="text-2xl font-bold text-white">{counts.critical}</p>
            <p className="text-xs text-surface-400">ط­ط±ط¬ط©</p>
          </div>
        </div>
        <div className="bg-surface-800 border border-surface-700 rounded-xl p-4 flex items-center gap-3">
          <div className="rounded-lg bg-blue-500/10 p-2">
            <Info className="h-5 w-5 text-blue-400" />
          </div>
          <div>
            <p className="text-2xl font-bold text-white">{counts.info}</p>
            <p className="text-xs text-surface-400">ظ…ط¹ظ„ظˆظ…ط§طھظٹط©</p>
          </div>
        </div>
      </div>

      {loading && (
        <div className="flex items-center justify-center py-20">
          <RefreshCw className="h-8 w-8 text-surface-400 animate-spin" />
        </div>
      )}

      {isEmpty && (
        <div className="flex flex-col items-center justify-center py-20 text-surface-400">
          <CheckCircle className="h-16 w-16 mb-4 text-emerald-500 opacity-50" />
          <p className="text-lg font-semibold text-white">ظ„ط§ طھظˆط¬ط¯ طھظ†ط¨ظٹظ‡ط§طھ ط­ط§ظ„ظٹط§ظ‹</p>
          <p className="text-sm mt-1">ظƒظ„ ط´ظٹط، ظٹط¹ظ…ظ„ ط¨ط´ظƒظ„ ط·ط¨ظٹط¹ظٹ</p>
        </div>
      )}

      {!loading && alerts && !isEmpty && (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
          {sectionConfig.map(({ key, title, icon: Icon, borderColor, iconColor, badgeColor }) => {
            const items = alerts[key];
            if (!items || items.length === 0) return null;

            return (
              <Card key={key} className={`border-l-4 ${borderColor}`}>
                <div className="flex items-center justify-between mb-4">
                  <div className="flex items-center gap-2">
                    <Icon className={`h-5 w-5 ${iconColor}`} />
                    <h2 className="font-bold text-white">{title}</h2>
                  </div>
                  <span className={`text-xs font-bold px-2.5 py-1 rounded-full ${badgeColor}`}>
                    {items.length}
                  </span>
                </div>

                <div className="space-y-2">
                  {key === "expiry" &&
                    (alerts.expiry as AlertsData["expiry"]).map((item, i) => (
                      <div
                        key={i}
                        className="flex items-center justify-between bg-surface-900 rounded-lg px-3 py-2 text-sm"
                      >
                        <span className="text-white">{item.product_name}</span>
                        <span className="text-red-400 text-xs">{formatDate(item.expiry_date)}</span>
                      </div>
                    ))}

                  {key === "overdue_orders" &&
                    (alerts.overdue_orders as AlertsData["overdue_orders"]).map((item, i) => (
                      <div
                        key={i}
                        className="flex items-center justify-between bg-surface-900 rounded-lg px-3 py-2 text-sm"
                      >
                        <span className="text-white">{item.product_name}</span>
                        <span className="text-yellow-400 text-xs">{formatDate(item.due_date)}</span>
                      </div>
                    ))}

                  {key === "low_stock" &&
                    (alerts.low_stock as AlertsData["low_stock"]).map((item, i) => (
                      <div
                        key={i}
                        className="flex items-center justify-between bg-surface-900 rounded-lg px-3 py-2 text-sm"
                      >
                        <span className="text-white">{item.product_name}</span>
                        <span className="text-amber-400 text-xs">
                          {item.current_stock} / {item.min_stock}
                        </span>
                      </div>
                    ))}

                  {key === "overdue_invoices" &&
                    (alerts.overdue_invoices as AlertsData["overdue_invoices"]).map((item, i) => (
                      <div
                        key={i}
                        className="flex items-center justify-between bg-surface-900 rounded-lg px-3 py-2 text-sm"
                      >
                        <span className="text-white">{item.invoice_number}</span>
                        <div className="flex items-center gap-3">
                          <span className="text-red-400 text-xs">{formatOMR(item.amount)}</span>
                          <span className="text-surface-400 text-xs">{formatDate(item.due_date)}</span>
                        </div>
                      </div>
                    ))}

                  {key === "quality_pending" &&
                    (alerts.quality_pending as AlertsData["quality_pending"]).map((item, i) => (
                      <div
                        key={i}
                        className="flex items-center justify-between bg-surface-900 rounded-lg px-3 py-2 text-sm"
                      >
                        <span className="text-white">{item.product_name}</span>
                        <span className="text-blue-400 text-xs">{formatDate(item.created_at)}</span>
                      </div>
                    ))}
                </div>
              </Card>
            );
          })}
        </div>
      )}
    </div>
  );
}


