import { useState, useEffect, useMemo, useCallback } from "react";
import DataTable, { Column } from "@/components/ui/DataTable";
import Badge from "@/components/ui/Badge";
import Button from "@/components/ui/Button";
import Card from "@/components/ui/Card";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { Plus, FileText, AlertTriangle, CheckCircle } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

type ChequeKind = "all" | "issued" | "received";

interface Cheque {
  id: number;
  kind: string;
  cheque_no: string;
  bank: string;
  party: string;
  amount_milli: number;
  due_date: string;
  status: string;
}

export default function ChequesPage() {
  const addNotification = useUIStore((s) => s.addNotification);
  const [cheques, setCheques] = useState<Cheque[]>([]);
  const [loading, setLoading] = useState(true);
  const [kind, setKind] = useState<ChequeKind>("all");
  const [showForm, setShowForm] = useState(false);
  const [saving, setSaving] = useState(false);
  const [form, setForm] = useState({
    kind: "issued",
    cheque_no: "",
    bank: "",
    party: "",
    amount_milli: "",
    due_date: "",
    notes: "",
  });

  const loadCheques = useCallback(async () => {
    setLoading(true);
    try {
      const d = await invoke("list_cheques");
      setCheques(d as Cheque[]);
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء تحميل البيانات" }); }
    finally { setLoading(false); }
  }, [addNotification]);

  useEffect(() => { loadCheques(); }, [loadCheques]);

  const handleCreate = async () => {
    setSaving(true);
    try {
      await invoke("create_cheque", {
        input: {
          kind: form.kind,
          cheque_no: form.cheque_no,
          bank: form.bank,
          party: form.party,
          amount_milli: Number(form.amount_milli),
          due_date: form.due_date,
          notes: form.notes,
        },
      });
      setShowForm(false);
      setForm({ kind: "issued", cheque_no: "", bank: "", party: "", amount_milli: "", due_date: "", notes: "" });
      loadCheques();
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء الحفظ" }); }
    finally { setSaving(false); }
  };

  const filtered = useMemo(() => {
    if (kind === "all") return cheques;
    return cheques.filter((c) => c.kind === kind);
  }, [cheques, kind]);

  const totalIssued = cheques.filter((c) => c.kind === "issued").reduce((s, c) => s + (c.amount_milli || 0), 0);
  const totalReceived = cheques.filter((c) => c.kind === "received").reduce((s, c) => s + (c.amount_milli || 0), 0);
  const outstanding = cheques.filter((c) => c.status === "pending" || c.status === "issued").length;
  const overdue = cheques.filter((c) => {
    if (c.status === "cleared" || c.status === "deposited") return false;
    return c.due_date && c.due_date < new Date().toISOString().split("T")[0];
  }).length;

  const statusMap: Record<string, { label: string; variant: any }> = {
    issued: { label: "صادرة", variant: "warning" },
    pending: { label: "قيد الانتظار", variant: "warning" },
    deposited: { label: "مودعة", variant: "info" },
    cleared: { label: "محصلة", variant: "success" },
    bounced: { label: "مرتجعة", variant: "danger" },
  };

  const kindLabels: Record<string, string> = {
    issued: "صادرة",
    received: "واردة",
  };

  const columns: Column<Cheque>[] = useMemo(() => [
    { key: "kind", header: "النوع", render: (r) => (
      <Badge variant={r.kind === "issued" ? "warning" : "info"}>
        {r.kind === "issued" ? "صادرة" : "واردة"}
      </Badge>
    )},
    { key: "cheque_no", header: "رقم الشيك", sortable: true, render: (r) => <span className="font-mono text-brand-400">{r.cheque_no}</span> },
    { key: "bank", header: "البنك", sortable: true },
    { key: "party", header: "الطرف", sortable: true, render: (r) => r.party || "—" },
    { key: "amount_milli", header: "المبلغ", sortable: true, align: "left", render: (r) => <span className="font-bold text-gold-400">{formatOMR(r.amount_milli)}</span> },
    { key: "due_date", header: "تاريخ الاستحقاق", sortable: true, render: (r) => formatDate(r.due_date) },
    { key: "status", header: "الحالة", render: (r) => {
      const s = statusMap[r.status] || { label: r.status, variant: "default" };
      return <Badge variant={s.variant}>{s.label}</Badge>;
    }},
  ], []);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">إدارة الشيكات</h1>
          <p className="page-subtitle">{cheques.length} شيك</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />} onClick={() => setShowForm(true)}>شيك جديد</Button>
      </div>

      <div className="grid grid-cols-4 gap-4">
        <Card className="text-center">
          <p className="text-2xl font-bold gradient-text">{formatOMR(totalIssued)}</p>
          <p className="text-xs text-surface-400">إجمالي الصادرة</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-brand-400">{formatOMR(totalReceived)}</p>
          <p className="text-xs text-surface-400">إجمالي الواردة</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-gold-400">{outstanding}</p>
          <p className="text-xs text-surface-400">غير محصلة</p>
        </Card>
        <Card className="text-center">
          <p className="text-2xl font-bold text-red-400">{overdue}</p>
          <p className="text-xs text-surface-400">متأخرة</p>
        </Card>
      </div>

      <div className="flex items-center gap-2">
        {(["all", "issued", "received"] as ChequeKind[]).map((k) => (
          <button
            key={k}
            onClick={() => setKind(k)}
            className={`px-4 py-2 rounded-xl text-sm font-medium transition-all ${
              kind === k ? "bg-brand-600 text-white" : "bg-surface-800 text-surface-400 hover:text-white hover:bg-surface-700"
            }`}
          >
            {k === "all" ? "الكل" : k === "issued" ? "صادرة" : "واردة"}
          </button>
        ))}
      </div>

      {showForm && (
        <Card>
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-bold text-white">شيك جديد</h2>
            <button onClick={() => setShowForm(false)} className="text-surface-400 hover:text-white text-xl">&times;</button>
          </div>
          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="input-group">
                <label className="input-label">النوع</label>
                <select value={form.kind} onChange={(e) => setForm({ ...form, kind: e.target.value })} className="input-field" aria-label="النوع">
                  <option value="issued">شيك صادر</option>
                  <option value="received">شيك وارد</option>
                </select>
              </div>
              <div className="input-group">
                <label className="input-label">رقم الشيك</label>
                <input type="text" value={form.cheque_no} onChange={(e) => setForm({ ...form, cheque_no: e.target.value })} className="input-field" dir="ltr" aria-label="رقم الشيك" />
              </div>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="input-group">
                <label className="input-label">البنك</label>
                <input type="text" value={form.bank} onChange={(e) => setForm({ ...form, bank: e.target.value })} className="input-field" aria-label="البنك" />
              </div>
              <div className="input-group">
                <label className="input-label">الطرف</label>
                <input type="text" value={form.party} onChange={(e) => setForm({ ...form, party: e.target.value })} className="input-field" aria-label="الطرف" />
              </div>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="input-group">
                <label className="input-label">المبلغ (ملي)</label>
                <input type="number" value={form.amount_milli} onChange={(e) => setForm({ ...form, amount_milli: e.target.value })} className="input-field" dir="ltr" aria-label="المبلغ بالملي" />
              </div>
              <div className="input-group">
                <label className="input-label">تاريخ الاستحقاق</label>
                <input type="date" value={form.due_date} onChange={(e) => setForm({ ...form, due_date: e.target.value })} className="input-field" aria-label="تاريخ الاستحقاق" />
              </div>
            </div>
            <div className="input-group">
              <label className="input-label">ملاحظات</label>
              <input type="text" value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} className="input-field" aria-label="ملاحظات" />
            </div>
          </div>
          <div className="flex justify-end gap-3 mt-4">
            <Button variant="ghost" onClick={() => setShowForm(false)}>إلغاء</Button>
            <Button icon={<FileText className="w-4 h-4" />} onClick={handleCreate} loading={saving}>إنشاء الشيك</Button>
          </div>
        </Card>
      )}

      <DataTable columns={columns} data={filtered} loading={loading} emptyMessage="لا توجد شيكات" />
    </div>
  );
}
