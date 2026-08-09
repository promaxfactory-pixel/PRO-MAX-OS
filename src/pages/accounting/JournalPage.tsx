import { useState, useEffect, useMemo } from "react";
import DataTable, { Column } from "@/components/ui/DataTable";
import { formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { useUIStore } from "../../stores/uiStore";
import { JournalEntry } from "@/types";
import { useTranslation } from "react-i18next";

export default function JournalPage() {
  const { t } = useTranslation();
  const { addNotification } = useUIStore();
  const [entries, setEntries] = useState<JournalEntry[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => { invoke("list_journal_entries").then((d) => setEntries(d as JournalEntry[])).catch((e: unknown) => addNotification({ title: t("common.error"), message: String(e), type: 'error' })).finally(() => setLoading(false)); }, []);

  const columns: Column<any>[] = useMemo(() => [
    { key: "entry_no", header: t("journal.number"), sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.entry_no || "—"}</span> },
    { key: "date", header: t("common.date"), sortable: true, render: (r) => formatDate(r.date) },
    { key: "memo", header: t("journal.memo"), sortable: true },
    { key: "ref_type", header: t("print.reference"), render: (r) => r.ref_type ? `${r.ref_type}#${r.ref_id}` : "—" },
    { key: "created_by", header: t("maintenance.sheetDetail.createdBy") },
  ], [t]);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div><h1 className="page-title">{t("tools.excelImport.typeJournal")}</h1><p className="page-subtitle">{t("journal.count", { count: entries.length })}</p></div>
      </div>
      <DataTable columns={columns} data={entries} loading={loading} emptyMessage={t("journal.empty")} />
    </div>
  );
}
