import { useState, useEffect, useMemo, useCallback } from "react";
import DataTable, { Column, type BadgeVariant } from "@/components/ui/DataTable";
import Card from "@/components/ui/Card";
import Badge from "@/components/ui/Badge";
import { formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Search, ChevronDown, ChevronUp } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { useTranslation } from "react-i18next";

interface AuditLog {
  id: number;
  ts: string;
  username: string;
  action: string;
  entity: string;
  entity_id: number | null;
  reason: string;
  old_value: string;
  new_value: string;
}

export default function AuditLogPage() {
  const { t } = useTranslation();
  const addNotification = useUIStore((s) => s.addNotification);
  const [logs, setLogs] = useState<AuditLog[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState("");
  const [entityFilter, setEntityFilter] = useState("");
  const [userFilter, setUserFilter] = useState("");
  const [dateFrom, setDateFrom] = useState("");
  const [dateTo, setDateTo] = useState("");
  const [expandedRow, setExpandedRow] = useState<number | null>(null);

  const loadLogs = useCallback(async () => {
    setLoading(true);
    try {
      const d = await invoke("list_audit_logs");
      setLogs(d as AuditLog[]);
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("tools.loadError") }); }
    finally { setLoading(false); }
  }, [addNotification]);

  useEffect(() => { loadLogs(); }, [loadLogs]);

  const entities = useMemo(() => {
    const set = new Set(logs.map((l) => l.entity).filter(Boolean));
    return Array.from(set).sort();
  }, [logs]);

  const filtered = useMemo(() => {
    return logs.filter((l) => {
      if (search && !l.action?.includes(search) && !l.entity?.includes(search) && !l.username?.includes(search)) return false;
      if (entityFilter && l.entity !== entityFilter) return false;
      if (userFilter && !l.username?.includes(userFilter)) return false;
      if (dateFrom && l.ts?.slice(0, 10) < dateFrom) return false;
      if (dateTo && l.ts?.slice(0, 10) > dateTo) return false;
      return true;
    });
  }, [logs, search, entityFilter, userFilter, dateFrom, dateTo]);

  const today = new Date().toISOString().split("T")[0];
  const todayCount = logs.filter((l) => l.ts?.slice(0, 10) === today).length;
  const activeUsers = new Set(logs.map((l) => l.username).filter(Boolean)).size;

  const columns: Column<AuditLog>[] = useMemo(() => [
    { key: "ts", header: t("auditLog.timestamp"), sortable: true, render: (r) => <span className="text-surface-300 text-xs font-mono">{r.ts ? formatDate(r.ts) : "—"}</span> },
    { key: "username", header: t("auditLog.user"), sortable: true, render: (r) => <span className="font-medium text-brand-400">{r.username || "—"}</span> },
    { key: "action", header: t("auditLog.action"), sortable: true, render: (r) => {
      const v: BadgeVariant = r.action?.includes("delete") || r.action?.includes("remove") ? "danger" : r.action?.includes("create") || r.action?.includes("add") ? "success" : "info";
      return <Badge variant={v}>{r.action || "—"}</Badge>;
    }},
    { key: "entity", header: t("auditLog.entity"), sortable: true, render: (r) => r.entity || "—" },
    { key: "entity_id", header: t("auditLog.entityId"), render: (r) => r.entity_id != null ? <span className="font-mono text-xs">{r.entity_id}</span> : "—" },
    { key: "reason", header: t("print.reason"), render: (r) => <span className="text-surface-400 text-xs truncate max-w-[200px] inline-block">{r.reason || "—"}</span> },
    { key: "expand", header: "", render: (r) => (
      (r.old_value || r.new_value) ? (
        <button onClick={(e) => { e.stopPropagation(); setExpandedRow(expandedRow === r.id ? null : r.id); }} className="p-1.5 text-surface-400 hover:text-white transition-colors rounded-lg hover:bg-surface-700/50">
          {expandedRow === r.id ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
        </button>
      ) : null
    )},
  ], [expandedRow, t]);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">{t("auditLog.title")}</h1>
          <p className="page-subtitle">{t("auditLog.subtitle")}</p>
        </div>
      </div>

      <div className="grid grid-cols-4 gap-4">
        <Card className="text-center">
          <p className="text-2xl font-bold gradient-text">{logs.length}</p>
          <p className="text-xs text-surface-400">{t("auditLog.totalLogs")}</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-gold-400">{todayCount}</p>
          <p className="text-xs text-surface-400">{t("auditLog.todayLogs")}</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-brand-400">{activeUsers}</p>
          <p className="text-xs text-surface-400">{t("auditLog.activeUsers")}</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-emerald-400">{entities.length}</p>
          <p className="text-xs text-surface-400">{t("auditLog.entityTypes")}</p>
        </Card>
      </div>

      <Card>
        <div className="flex flex-wrap items-center gap-3">
          <div className="relative flex-1 min-w-[200px]">
            <Search className="absolute right-3 top-1/2 -translate-y-1/2 w-4 h-4 text-surface-500" />
            <input type="text" placeholder={t("auditLog.searchPlaceholder")} value={search} onChange={(e) => setSearch(e.target.value)} className="w-full pr-10 input-field text-sm" aria-label={t("common.search")} />
          </div>
          <div className="input-group">
            <label className="input-label text-[10px]">{t("auditLog.entity")}</label>
            <select value={entityFilter} onChange={(e) => setEntityFilter(e.target.value)} className="input-field text-sm" aria-label={t("auditLog.entity")}>
              <option value="">{t("common.all")}</option>
              {entities.map((ent) => <option key={ent} value={ent}>{ent}</option>)}
            </select>
          </div>
          <div className="input-group">
            <label className="input-label text-[10px]">{t("auditLog.user")}</label>
            <input type="text" placeholder={t("auditLog.userPlaceholder")} value={userFilter} onChange={(e) => setUserFilter(e.target.value)} className="input-field text-sm" aria-label={t("auditLog.user")} />
          </div>
          <div className="input-group">
            <label className="input-label text-[10px]">{t("common.fromDate")}</label>
            <input type="date" value={dateFrom} onChange={(e) => setDateFrom(e.target.value)} className="input-field text-sm" aria-label={t("common.fromDate")} />
          </div>
          <div className="input-group">
            <label className="input-label text-[10px]">{t("common.toDate")}</label>
            <input type="date" value={dateTo} onChange={(e) => setDateTo(e.target.value)} className="input-field text-sm" aria-label={t("common.toDate")} />
          </div>
        </div>
      </Card>

      <DataTable
        columns={columns}
        data={filtered}
        loading={loading}
        emptyMessage={t("tools.noRecords")}
        onRowClick={(row: AuditLog) => setExpandedRow(expandedRow === row.id ? null : row.id)}
      />

      {expandedRow && filtered.find((r: AuditLog) => r.id === expandedRow) && (
        <Card>
          <div className="grid grid-cols-2 gap-4 text-xs">
            <div>
              <p className="text-surface-400 mb-1 font-medium">{t("auditLog.oldValue")}</p>
              <pre className="text-surface-300 bg-surface-900/80 rounded-lg p-3 overflow-auto max-h-40 whitespace-pre-wrap font-mono">
                {filtered.find((r: AuditLog) => r.id === expandedRow)?.old_value || "—"}
              </pre>
            </div>
            <div>
              <p className="text-surface-400 mb-1 font-medium">{t("auditLog.newValue")}</p>
              <pre className="text-surface-300 bg-surface-900/80 rounded-lg p-3 overflow-auto max-h-40 whitespace-pre-wrap font-mono">
                {filtered.find((r: AuditLog) => r.id === expandedRow)?.new_value || "—"}
              </pre>
            </div>
          </div>
        </Card>
      )}
    </div>
  );
}
