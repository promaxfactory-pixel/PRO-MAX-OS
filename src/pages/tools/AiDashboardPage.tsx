import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { formatOMR, cn } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { useUIStore } from "@/stores/uiStore";
import {
  Brain, AlertTriangle, AlertCircle, Info, TrendingUp,
  Users, Factory, DollarSign, Warehouse, RefreshCw
} from "lucide-react";
import {
  AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer
} from "recharts";

interface Insight {
  id: number;
  title: string;
  description: string;
  severity: "info" | "warning" | "critical";
  category: string;
}

interface Forecast { month: string; actual: number; predicted: number; }
interface RiskCustomer { name: string; overdue_amount: number; days_overdue: number; risk: string; }
interface CostItem { category: string; current: number; previous: number; change: number; }
interface InventoryAlert { product: string; current_stock: number; min_stock: number; suggestion: string; }

const SEVERITY_CONFIG = {
  info: { icon: Info, color: "text-blue-400", bg: "bg-blue-500/10", border: "border-blue-500/30" },
  warning: { icon: AlertTriangle, color: "text-amber-400", bg: "bg-amber-500/10", border: "border-amber-500/30" },
  critical: { icon: AlertCircle, color: "text-red-400", bg: "bg-red-500/10", border: "border-red-500/30" },
};

export default function AiDashboardPage() {
  const { t } = useTranslation();
  const addNotification = useUIStore((s) => s.addNotification);
  const [insights, setInsights] = useState<Insight[]>([]);
  const [forecast, setForecast] = useState<Forecast[]>([]);
  const [risks, setRisks] = useState<RiskCustomer[]>([]);
  const [costs, setCosts] = useState<CostItem[]>([]);
  const [invAlerts, setInvAlerts] = useState<InventoryAlert[]>([]);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [production, setProduction] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const [i, f, r, c, inv, p] = await Promise.all([
        invoke<Insight[]>("ai_dashboard_insights").catch(() => []),
        invoke<Forecast[]>("ai_sales_forecast", { days: 30 }).catch(() => []),
        invoke<RiskCustomer[]>("ai_customer_risk").catch(() => []),
        invoke<CostItem[]>("ai_cost_analysis").catch(() => []),
        invoke<InventoryAlert[]>("ai_inventory_optimization").catch(() => []),
        invoke("ai_production_analysis").catch(() => null),
      ]);
      setInsights(i); setForecast(f); setRisks(r); setCosts(c); setInvAlerts(inv);         setProduction(p);
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: String(err) });
    } finally {
      setLoading(false);
    }
  }, [addNotification, t]);

  useEffect(() => { loadData(); }, [loadData]);

  if (loading) return (
    <div className="flex items-center justify-center h-64">
      <div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" />
    </div>
  );

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title flex items-center gap-2">
            <Brain className="w-6 h-6 text-gold-400" />
            {t("tools.aiDashboard.title")}
          </h1>
          <p className="page-subtitle">{t("tools.aiDashboard.subtitle")}</p>
        </div>
        <Button variant="outline" icon={<RefreshCw className="w-4 h-4" />} onClick={loadData}>{t("tools.refresh")}</Button>
      </div>

      {insights.length > 0 && (
        <div className="grid grid-cols-3 gap-4">
          {insights.slice(0, 6).map((insight) => {
            const cfg = SEVERITY_CONFIG[insight.severity];
            const Icon = cfg.icon;
            return (
              <div key={insight.id} className={cn("p-4 rounded-xl border", cfg.bg, cfg.border)}>
                <div className="flex items-center gap-2 mb-2">
                  <Icon className={cn("w-5 h-5", cfg.color)} />
                  <span className="text-xs text-surface-500">{insight.category}</span>
                </div>
                <h4 className="font-medium text-white text-sm mb-1">{insight.title}</h4>
                <p className="text-xs text-surface-400 line-clamp-2">{insight.description}</p>
              </div>
            );
          })}
        </div>
      )}

      <div className="grid grid-cols-2 gap-6">
        <Card>
          <h3 className="section-title flex items-center gap-2 mb-4">
            <TrendingUp className="w-5 h-5 text-gold-400" />
            {t("tools.aiDashboard.salesForecast")}
          </h3>
          <div className="h-64">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={forecast}>
                <defs>
                  <linearGradient id="predGrad" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#4c1d95" stopOpacity={0.3} />
                    <stop offset="95%" stopColor="#4c1d95" stopOpacity={0} />
                  </linearGradient>
                </defs>
                <CartesianGrid strokeDasharray="3 3" stroke="#334155" />
                <XAxis dataKey="month" stroke="#64748b" fontSize={12} />
                <YAxis stroke="#64748b" fontSize={12} />
                <Tooltip
                  contentStyle={{ backgroundColor: "#1e293b", border: "1px solid #334155", borderRadius: "12px", color: "#f8fafc" }}
                  formatter={(v: number) => [formatOMR(v), ""]}
                />
                <Area type="monotone" dataKey="actual" stroke="#d4af37" strokeWidth={2} fill="url(#predGrad)" name={t("tools.aiDashboard.actual")} />
                <Area type="monotone" dataKey="predicted" stroke="#8b5cf6" strokeWidth={2} strokeDasharray="5 5" fill="none" name={t("tools.aiDashboard.predicted")} />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </Card>

        <Card>
          <h3 className="section-title flex items-center gap-2 mb-4">
            <DollarSign className="w-5 h-5 text-gold-400" />
            {t("tools.aiDashboard.costAnalysis")}
          </h3>
          <div className="space-y-3">
            {costs.length === 0 ? (
              <p className="text-center text-surface-500 py-4 text-sm">{t("tools.noData")}</p>
            ) : costs.map((c, i) => (
              <div key={i} className="flex items-center justify-between py-2 border-b border-surface-700/30 last:border-0">
                <div>
                  <span className="text-sm text-surface-200">{c.category}</span>
                  <span className={cn("text-xs mr-2", c.change > 0 ? "text-red-400" : "text-emerald-400")}>
                    {c.change > 0 ? "+" : ""}{c.change}%
                  </span>
                </div>
                <span className="text-sm font-medium">{formatOMR(c.current)}</span>
              </div>
            ))}
          </div>
        </Card>
      </div>

      <div className="grid grid-cols-2 gap-6">
        <Card>
          <h3 className="section-title flex items-center gap-2 mb-4">
            <Users className="w-5 h-5 text-brand-400" />
            {t("tools.aiDashboard.customerRisk")}
          </h3>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-surface-700/50 text-surface-400">
                  <th className="p-2 text-right">{t("tools.aiDashboard.customer")}</th>
                  <th className="p-2 text-center">{t("tools.aiDashboard.overdueAmount")}</th>
                  <th className="p-2 text-center">{t("tools.aiDashboard.days")}</th>
                  <th className="p-2 text-center">{t("tools.aiDashboard.level")}</th>
                </tr>
              </thead>
              <tbody>
                {risks.length === 0 ? (
                  <tr><td colSpan={4} className="p-4 text-center text-surface-500">{t("tools.aiDashboard.noRisks")}</td></tr>
                ) : risks.map((r, i) => (
                  <tr key={i} className="border-b border-surface-700/20">
                    <td className="p-2">{r.name}</td>
                    <td className="p-2 text-center">{formatOMR(r.overdue_amount)}</td>
                    <td className="p-2 text-center">{r.days_overdue}</td>
                    <td className="p-2 text-center">
                      <span className={cn("px-2 py-0.5 rounded-full text-xs", r.risk === "high" ? "bg-red-500/20 text-red-400" : r.risk === "medium" ? "bg-amber-500/20 text-amber-400" : "bg-emerald-500/20 text-emerald-400")}>
                        {r.risk === "high" ? t("tools.aiDashboard.riskHigh") : r.risk === "medium" ? t("tools.aiDashboard.riskMedium") : t("tools.aiDashboard.riskLow")}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Card>

        <Card>
          <h3 className="section-title flex items-center gap-2 mb-4">
            <Factory className="w-5 h-5 text-brand-400" />
            {t("tools.aiDashboard.productionAnalysis")}
          </h3>
          {production ? (
            <div className="grid grid-cols-2 gap-4">
              <div className="p-3 bg-surface-800/50 rounded-lg text-center">
                <p className="text-xs text-surface-500 mb-1">{t("tools.aiDashboard.productionRate")}</p>
                <p className="text-xl font-bold gradient-text">{production.efficiency || 0}%</p>
              </div>
              <div className="p-3 bg-surface-800/50 rounded-lg text-center">
                <p className="text-xs text-surface-500 mb-1">{t("tools.aiDashboard.wasteRate")}</p>
                <p className="text-xl font-bold text-red-400">{production.waste_rate || 0}%</p>
              </div>
              <div className="p-3 bg-surface-800/50 rounded-lg text-center">
                <p className="text-xs text-surface-500 mb-1">{t("tools.aiDashboard.downtime")}</p>
                <p className="text-xl font-bold text-amber-400">{t("tools.aiDashboard.downtimeHours", { count: production.downtime_hours || 0 })}</p>
              </div>
              <div className="p-3 bg-surface-800/50 rounded-lg text-center">
                <p className="text-xs text-surface-500 mb-1">{t("tools.aiDashboard.productQuality")}</p>
                <p className="text-xl font-bold text-emerald-400">{production.quality_score || 0}%</p>
              </div>
            </div>
          ) : (
            <p className="text-center text-surface-500 py-4 text-sm">{t("tools.aiDashboard.noProductionData")}</p>
          )}
        </Card>
      </div>

      {invAlerts.length > 0 && (
        <Card>
          <h3 className="section-title flex items-center gap-2 mb-4">
            <Warehouse className="w-5 h-5 text-gold-400" />
            {t("tools.aiDashboard.inventoryOptimization")}
          </h3>
          <div className="grid grid-cols-3 gap-4">
            {invAlerts.map((a, i) => (
              <div key={i} className="p-3 bg-surface-800/50 rounded-lg">
                <h4 className="text-sm font-medium text-white mb-1">{a.product}</h4>
                <div className="flex justify-between text-xs text-surface-400 mb-2">
                  <span>{t("tools.aiDashboard.stockLabel", { stock: a.current_stock })}</span>
                  <span>{t("tools.aiDashboard.minStockLabel", { min: a.min_stock })}</span>
                </div>
                <p className="text-xs text-amber-400">{a.suggestion}</p>
              </div>
            ))}
          </div>
        </Card>
      )}
    </div>
  );
}
