import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "@/components/ui/Card";
import { Building2, ScrollText, IdCard, Globe, Clock, CheckCircle2, XCircle, Loader2 } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { useTranslation } from "react-i18next";

interface GovDashboard {
  entities_count: number;
  active_integrations: number;
  pending_submissions: number;
  successful_submissions: number;
  failed_submissions: number;
  entities: GovEntity[];
  recent_submissions: GovSubmission[];
}

interface GovEntity {
  id: number;
  code: string;
  name_ar: string;
  name_en: string | null;
  category: string;
  website: string | null;
  api_endpoint: string | null;
  active: boolean;
  notes: string | null;
}

interface GovSubmission {
  id: number;
  entity_id: number;
  entity_name: string;
  report_template_id: number | null;
  status: string;
  reference_no: string | null;
  submitted_at: string | null;
  submitted_by: string | null;
  created_at: string;
}

interface DocStatus {
  expiring_passports: number;
  expiring_residence: number;
  expiring_visa: number;
  expiring_work_permits: number;
  expiring_renewals: number;
  as_of_date: string;
}

export default function GovernmentDashboardPage() {
  const { t } = useTranslation();
  const { addNotification } = useUIStore();
  const [dashboard, setDashboard] = useState<GovDashboard | null>(null);
  const [docStatus, setDocStatus] = useState<DocStatus | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    Promise.all([
      invoke<GovDashboard>("gov_get_dashboard"),
      invoke<DocStatus>("gov_get_employee_doc_status"),
    ]).then(([d, ds]) => {
      setDashboard(d);
      setDocStatus(ds);
    }).catch((e: unknown) => addNotification({ title: t("common.error"), message: String(e), type: 'error' })).finally(() => setLoading(false));
  }, []);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <Loader2 className="w-8 h-8 animate-spin text-gold-400" />
      </div>
    );
  }

  const stats = [
    { label: t("government.entities"), value: dashboard?.entities_count || 0, icon: Building2, color: "text-blue-400" },
    { label: t("government.activeIntegrations"), value: dashboard?.active_integrations || 0, icon: Globe, color: "text-emerald-400" },
    { label: t("government.pending"), value: dashboard?.pending_submissions || 0, icon: Clock, color: "text-amber-400" },
    { label: t("government.submitted"), value: dashboard?.successful_submissions || 0, icon: CheckCircle2, color: "text-emerald-400" },
    { label: t("government.failed"), value: dashboard?.failed_submissions || 0, icon: XCircle, color: "text-red-400" },
  ];

  const docAlerts = [
    { label: t("government.expiringPassports"), value: docStatus?.expiring_passports || 0, icon: IdCard, color: "text-red-400" },
    { label: t("government.expiringResidence"), value: docStatus?.expiring_residence || 0, icon: ScrollText, color: "text-amber-400" },
    { label: t("government.expiringVisas"), value: docStatus?.expiring_visa || 0, icon: IdCard, color: "text-amber-400" },
    { label: t("government.expiringWorkPermits"), value: docStatus?.expiring_work_permits || 0, icon: ScrollText, color: "text-amber-400" },
  ];

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">{t("government.title")}</h1>
          <p className="page-subtitle">{t("government.subtitle")}</p>
        </div>
      </div>

      <div className="grid grid-cols-5 gap-4">
        {stats.map((s) => (
          <Card key={s.label} className="text-center">
            <s.icon className={`w-6 h-6 mx-auto mb-2 ${s.color}`} />
            <p className="text-2xl font-bold text-white">{s.value}</p>
            <p className="text-xs text-surface-400 mt-1">{s.label}</p>
          </Card>
        ))}
      </div>

      <div className="grid grid-cols-2 gap-6">
        <Card>
          <h3 className="section-title">{t("government.docAlerts")}</h3>
          <div className="space-y-3">
            {docAlerts.map((a) => (
              <div key={a.label} className="flex items-center justify-between p-3 bg-surface-800/50 rounded-xl">
                <div className="flex items-center gap-2">
                  <a.icon className={`w-4 h-4 ${a.color}`} />
                  <span className="text-sm text-surface-300">{a.label}</span>
                </div>
                <span className={`text-sm font-bold ${a.color}`}>{a.value}</span>
              </div>
            ))}
          </div>
        </Card>

        <Card>
          <h3 className="section-title">{t("government.recentSubmissions")}</h3>
          <div className="space-y-2">
            {dashboard?.recent_submissions.length ? (
              dashboard.recent_submissions.map((sub) => (
                <div key={sub.id} className="flex items-center justify-between p-2 bg-surface-800/50 rounded-lg">
                  <div>
                    <p className="text-sm text-white">{sub.entity_name}</p>
                    <p className="text-xs text-surface-500">{sub.created_at}</p>
                  </div>
                  <span className={`badge-${sub.status === 'submitted' ? 'success' : sub.status === 'pending' ? 'warning' : 'danger'}`}>
                    {sub.status === 'submitted' ? t("government.submitted") : sub.status === 'pending' ? t("government.pending") : t("government.failed")}
                  </span>
                </div>
              ))
            ) : (
              <p className="text-sm text-surface-500 text-center py-4">{t("government.noSubmissions")}</p>
            )}
          </div>
        </Card>
      </div>

      <Card>
        <h3 className="section-title">{t("government.registeredEntities")}</h3>
        <div className="grid grid-cols-5 gap-3">
          {dashboard?.entities.map((e) => (
            <div key={e.id} className="p-3 bg-surface-800/50 rounded-xl text-center hover:bg-surface-700/50 transition-colors">
              <p className="text-sm font-medium text-white">{e.name_ar}</p>
              <p className="text-[10px] text-surface-500 mt-1">{e.category}</p>
              {e.active && <span className="inline-block w-1.5 h-1.5 rounded-full bg-emerald-400 mt-2" />}
            </div>
          ))}
        </div>
      </Card>
    </div>
  );
}