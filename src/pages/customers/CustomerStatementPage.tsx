import { useState, useEffect, useMemo, useCallback } from "react";
import { useParams, useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import DataTable, { Column } from "@/components/ui/DataTable";
import { formatOMR, formatDate } from "@/lib/utils";
import { invoke } from "@/lib/tauri";
import { ArrowRight, Printer, Calendar } from "lucide-react";
import StatementPrintTemplate from "@/components/print/StatementPrintTemplate";
import { printComponent } from "@/utils/printUtils";
import { useUIStore } from "@/stores/uiStore";

export default function CustomerStatementPage() {
  const { id } = useParams();
  const navigate = useNavigate();
  const addNotification = useUIStore((s) => s.addNotification);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [data, setData] = useState<any>(null);
  const [loading, setLoading] = useState(true);
  const [fromDate, setFromDate] = useState("");
  const [toDate, setToDate] = useState("");
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [printData, setPrintData] = useState<any>(null);
  const [showPrint, setShowPrint] = useState(false);

  const loadStatement = useCallback(async () => {
    setLoading(true);
    try {
      const result = await invoke("get_customer_statement", {
        customerId: Number(id),
        fromDate: fromDate || null,
        toDate: toDate || null,
      });
      setData(result);
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء تحميل البيانات" }); }
    finally { setLoading(false); }
  }, [addNotification, id, fromDate, toDate]);

  useEffect(() => { loadStatement(); }, [loadStatement]);

  const handlePrint = async () => {
    try {
      const company = await invoke("get_company_settings");
      setPrintData({ ...data, company });
      setShowPrint(true);
      setTimeout(() => {
        printComponent("print-area");
        setShowPrint(false);
      }, 200);
    } catch (err) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء تحميل البيانات" }); }
  };

  if (loading || !data) {
    return <div className="flex items-center justify-center h-64"><div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" /></div>;
  }

  const txnLabels: Record<string, string> = {
    invoice: "فاتورة",
    payment: "دفعة",
    credit_note: "إشعار دائن",
  };

  const columns: Column<any>[] = useMemo(() => [
    { key: "date", header: "التاريخ", render: (r) => formatDate(r.date) },
    { key: "ref_no", header: "المرجع", render: (r) => r.ref_no || "—" },
    { key: "txn_type", header: "النوع", render: (r) => txnLabels[r.txn_type] || r.txn_type },
    { key: "debit_milli", header: "مدين", align: "left", render: (r) => r.debit_milli > 0 ? <span className="text-emerald-400 font-medium">{formatOMR(r.debit_milli)}</span> : "—" },
    { key: "credit_milli", header: "دائن", align: "left", render: (r) => r.credit_milli > 0 ? <span className="text-red-400 font-medium">{formatOMR(r.credit_milli)}</span> : "—" },
    { key: "balance_milli", header: "الرصيد", align: "left", render: (r) => <span className="font-bold">{formatOMR(r.balance_milli)}</span> },
    { key: "notes", header: "ملاحظات", render: (r) => r.notes || "—" },
  ], []);

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div className="flex items-center gap-3">
          <button onClick={() => navigate(`/customers/${id}`)} className="btn-ghost p-2"><ArrowRight className="w-5 h-5" /></button>
          <div>
            <h1 className="page-title">كشف حساب العميل</h1>
            <p className="page-subtitle">{data.customer.name} — {data.customer.code || ""}</p>
          </div>
        </div>
        <Button variant="outline" icon={<Printer className="w-4 h-4" />} onClick={handlePrint}>طباعة</Button>
      </div>

      <div className="flex items-center gap-4">
        <div className="flex items-center gap-2">
          <Calendar className="w-4 h-4 text-surface-400" />
          <input type="date" value={fromDate} onChange={(e) => setFromDate(e.target.value)} className="input-field text-sm" aria-label="من تاريخ" />
          <span className="text-surface-500">إلى</span>
          <input type="date" value={toDate} onChange={(e) => setToDate(e.target.value)} className="input-field text-sm" aria-label="إلى تاريخ" />
        </div>
      </div>

      <div className="grid grid-cols-4 gap-4">
        <Card><p className="text-sm text-surface-400">الرصيد الافتتاحي</p><p className="text-lg font-bold mt-1">{formatOMR(data.opening_balance_milli)}</p></Card>
        <Card><p className="text-sm text-surface-400">إجمالي المدين</p><p className="text-lg font-bold mt-1 text-emerald-400">{formatOMR(data.total_debit_milli)}</p></Card>
        <Card><p className="text-sm text-surface-400">إجمالي الدائن</p><p className="text-lg font-bold mt-1 text-red-400">{formatOMR(data.total_credit_milli)}</p></Card>
        <Card><p className="text-sm text-surface-400">الرصيد الختامي</p><p className="text-lg font-bold mt-1 gradient-text">{formatOMR(data.closing_balance_milli)}</p></Card>
      </div>

      <Card>
        <DataTable columns={columns} data={data.transactions} compact />
      </Card>

      {showPrint && printData && (
        <div style={{ position: "absolute", left: "-9999px" }}>
          <StatementPrintTemplate
            title="كشف حساب عميل"
            entityName={printData.customer.name}
            entityCode={printData.customer.code}
            entityType="customer"
            openingBalance={printData.opening_balance_milli}
            transactions={printData.transactions}
            closingBalance={printData.closing_balance_milli}
            totalDebit={printData.total_debit_milli}
            totalCredit={printData.total_credit_milli}
            company={printData.company}
            fromDate={fromDate}
            toDate={toDate}
          />
        </div>
      )}
    </div>
  );
}

