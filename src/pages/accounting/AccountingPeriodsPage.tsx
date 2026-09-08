import { FormEvent, useCallback, useEffect, useMemo, useState } from "react";
import { CalendarRange, LockKeyhole, Plus, RotateCcw, ShieldCheck } from "lucide-react";
import Badge from "@/components/ui/Badge";
import Button from "@/components/ui/Button";
import Card from "@/components/ui/Card";
import DataTable, { Column } from "@/components/ui/DataTable";
import { Input, Textarea } from "@/components/ui/Input";
import Modal from "@/components/ui/Modal";
import { formatDate, formatDateTime } from "@/lib/utils";
import { invoke } from "@/lib/tauri";
import { useAuthStore } from "@/stores/authStore";
import { useUIStore } from "@/stores/uiStore";

interface AccountingPeriod {
  id: number;
  name: string;
  start_date: string;
  end_date: string;
  status: "open" | "closed";
  created_by: number | null;
  created_at: string;
  closed_by: number | null;
  closed_at: string | null;
  close_reason: string | null;
  reopened_by: number | null;
  reopened_at: string | null;
  reopen_reason: string | null;
}

type PeriodAction = "close" | "reopen";

function monthDefaults() {
  const now = new Date();
  const year = now.getFullYear();
  const month = now.getMonth();
  const pad = (value: number) => String(value).padStart(2, "0");
  const startDate = `${year}-${pad(month + 1)}-01`;
  const lastDay = new Date(year, month + 1, 0).getDate();
  return {
    name: `${year}-${pad(month + 1)}`,
    startDate,
    endDate: `${year}-${pad(month + 1)}-${pad(lastDay)}`,
  };
}

export default function AccountingPeriodsPage() {
  const addNotification = useUIStore((state) => state.addNotification);
  const role = useAuthStore((state) => state.user?.role);
  const [periods, setPeriods] = useState<AccountingPeriod[]>([]);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [createOpen, setCreateOpen] = useState(false);
  const [form, setForm] = useState(monthDefaults);
  const [pendingAction, setPendingAction] = useState<{
    period: AccountingPeriod;
    action: PeriodAction;
  } | null>(null);
  const [reason, setReason] = useState("");

  const canManage = role === "admin" || role === "accountant";
  const canReopen = role === "admin";

  const load = useCallback(async () => {
    setLoading(true);
    try {
      setPeriods(await invoke<AccountingPeriod[]>("list_accounting_periods"));
    } catch (error) {
      addNotification({ title: "تعذر تحميل الفترات", message: String(error), type: "error" });
    } finally {
      setLoading(false);
    }
  }, [addNotification]);

  useEffect(() => { void load(); }, [load]);

  const openAction = (period: AccountingPeriod, action: PeriodAction) => {
    setReason("");
    setPendingAction({ period, action });
  };

  const createPeriod = async (event: FormEvent) => {
    event.preventDefault();
    setBusy(true);
    try {
      await invoke<number>("create_accounting_period", {
        input: {
          name: form.name,
          start_date: form.startDate,
          end_date: form.endDate,
        },
      });
      addNotification({ title: "تم إنشاء الفترة", message: "الفترة مفتوحة ويمكن الترحيل إليها.", type: "success" });
      setCreateOpen(false);
      setForm(monthDefaults());
      await load();
    } catch (error) {
      addNotification({ title: "تعذر إنشاء الفترة", message: String(error), type: "error" });
    } finally {
      setBusy(false);
    }
  };

  const applyAction = async (event: FormEvent) => {
    event.preventDefault();
    if (!pendingAction) return;
    setBusy(true);
    try {
      const command = pendingAction.action === "close"
        ? "close_accounting_period"
        : "reopen_accounting_period";
      await invoke<void>(command, { periodId: pendingAction.period.id, reason });
      addNotification({
        title: pendingAction.action === "close" ? "تم إغلاق الفترة" : "تمت إعادة فتح الفترة",
        message: pendingAction.action === "close"
          ? "أصبح الترحيل إلى تواريخ هذه الفترة محظورًا."
          : "أصبح الترحيل إلى تواريخ هذه الفترة متاحًا مجددًا.",
        type: "success",
      });
      setPendingAction(null);
      setReason("");
      await load();
    } catch (error) {
      addNotification({ title: "لم تكتمل العملية", message: String(error), type: "error" });
    } finally {
      setBusy(false);
    }
  };

  const columns = useMemo<Column<AccountingPeriod>[]>(() => [
    { key: "name", header: "الفترة", sortable: true, render: (period) => <span className="font-semibold">{period.name}</span> },
    { key: "start_date", header: "من", sortable: true, render: (period) => formatDate(period.start_date) },
    { key: "end_date", header: "إلى", sortable: true, render: (period) => formatDate(period.end_date) },
    {
      key: "status",
      header: "الحالة",
      sortable: true,
      render: (period) => (
        <Badge variant={period.status === "closed" ? "danger" : "success"}>
          {period.status === "closed" ? "مغلقة" : "مفتوحة"}
        </Badge>
      ),
    },
    {
      key: "closed_at",
      header: "آخر إجراء",
      render: (period) => period.status === "closed" && period.closed_at
        ? `أغلقت ${formatDateTime(period.closed_at)}`
        : period.reopened_at
          ? `فتحت ${formatDateTime(period.reopened_at)}`
          : "—",
    },
    {
      key: "actions",
      header: "الإجراء",
      render: (period) => period.status === "open"
        ? canManage && (
          <Button
            size="sm"
            variant="danger"
            icon={<LockKeyhole className="h-4 w-4" />}
            onClick={() => openAction(period, "close")}
          >
            إغلاق
          </Button>
        )
        : canReopen && (
          <Button
            size="sm"
            variant="outline"
            icon={<RotateCcw className="h-4 w-4" />}
            onClick={() => openAction(period, "reopen")}
          >
            إعادة فتح
          </Button>
        ),
    },
  ], [canManage, canReopen]);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title flex items-center gap-2">
            <CalendarRange className="h-6 w-6 text-gold-400" />
            الفترات المالية والإقفال
          </h1>
          <p className="page-subtitle">تحكم زمني يمنع القيود والاستيرادات المؤرخة داخل فترة مغلقة</p>
        </div>
        {canManage && (
          <Button onClick={() => setCreateOpen(true)} icon={<Plus className="h-4 w-4" />}>
            فترة جديدة
          </Button>
        )}
      </div>

      <Card className="flex items-start gap-3 border-emerald-500/20">
        <ShieldCheck className="mt-0.5 h-5 w-5 shrink-0 text-emerald-400" />
        <div>
          <p className="font-semibold text-white">حماية على مستوى قاعدة البيانات</p>
          <p className="mt-1 text-sm text-surface-400">
            الإغلاق يمنع القيد اليدوي والترحيل الآلي والاستيراد المباشر. لن يُغلق النظام فترة تحتوي قيودًا غير متوازنة أو غير صالحة.
          </p>
        </div>
      </Card>

      <DataTable
        columns={columns}
        data={periods}
        loading={loading}
        emptyMessage="لا توجد فترات مالية بعد. أنشئ أول فترة وحدد نطاقها الزمني."
      />

      <Modal
        open={createOpen}
        onClose={() => !busy && setCreateOpen(false)}
        title="إنشاء فترة مالية"
        footer={(
          <>
            <Button variant="outline" onClick={() => setCreateOpen(false)} disabled={busy}>إلغاء</Button>
            <Button type="submit" form="create-accounting-period" loading={busy}>إنشاء الفترة</Button>
          </>
        )}
      >
        <form id="create-accounting-period" className="space-y-4" onSubmit={createPeriod}>
          <Input
            label="اسم الفترة"
            value={form.name}
            maxLength={100}
            required
            onChange={(event) => setForm((current) => ({ ...current, name: event.target.value }))}
          />
          <div className="grid gap-4 sm:grid-cols-2">
            <Input
              label="تاريخ البداية"
              type="date"
              value={form.startDate}
              required
              onChange={(event) => setForm((current) => ({ ...current, startDate: event.target.value }))}
            />
            <Input
              label="تاريخ النهاية"
              type="date"
              value={form.endDate}
              min={form.startDate}
              required
              onChange={(event) => setForm((current) => ({ ...current, endDate: event.target.value }))}
            />
          </div>
          <p className="text-xs text-surface-400">لا يسمح بتداخل نطاق هذه الفترة مع أي فترة موجودة.</p>
        </form>
      </Modal>

      <Modal
        open={pendingAction !== null}
        onClose={() => !busy && setPendingAction(null)}
        title={pendingAction?.action === "close" ? "إغلاق الفترة المالية" : "إعادة فتح الفترة المالية"}
        footer={(
          <>
            <Button variant="outline" onClick={() => setPendingAction(null)} disabled={busy}>إلغاء</Button>
            <Button
              type="submit"
              form="accounting-period-action"
              variant={pendingAction?.action === "close" ? "danger" : "primary"}
              loading={busy}
            >
              تأكيد
            </Button>
          </>
        )}
      >
        <form id="accounting-period-action" className="space-y-4" onSubmit={applyAction}>
          <p className="text-sm text-surface-300">
            {pendingAction?.action === "close"
              ? "بعد الإغلاق لن يقبل النظام أي قيد يحمل تاريخًا داخل هذه الفترة، مهما كان مصدره."
              : "إعادة الفتح عملية استثنائية بصلاحية المدير، وسيُحفظ السبب في سجل التدقيق."}
          </p>
          <Textarea
            label="السبب"
            value={reason}
            minLength={3}
            maxLength={500}
            required
            onChange={(event) => setReason(event.target.value)}
          />
        </form>
      </Modal>
    </div>
  );
}
