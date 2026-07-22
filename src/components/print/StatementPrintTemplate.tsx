import { formatOMR, formatDate, htmlEscape } from "@/utils/printUtils";

interface Transaction {
  date: string;
  ref_no: string | null;
  txn_type: string;
  debit_milli: number;
  credit_milli: number;
  balance_milli: number;
  notes: string | null;
}

interface Props {
  title: string;
  entityName: string;
  entityCode?: string | null;
  entityType: "customer" | "supplier";
  openingBalance: number;
  transactions: Transaction[];
  closingBalance: number;
  totalDebit: number;
  totalCredit: number;
  company: any;
  fromDate?: string;
  toDate?: string;
}

const S = {
  c: {
    fontFamily: "'Cairo', 'system-ui', sans-serif",
    direction: 'rtl' as const,
    padding: '30px 35px', color: '#1a1a2e', background: '#ffffff',
    position: 'relative' as const, overflow: 'hidden',
  },
  bar: {
    position: 'absolute' as const, top: 0, left: 0, right: 0, height: '4px',
    background: 'linear-gradient(90deg, #d4af37, #f5d77b, #d4af37)',
  },
  hdr: {
    display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start',
    marginBottom: '20px', paddingBottom: '14px', borderBottom: '2px solid #d4af37',
  },
  cn: { fontSize: '22px', fontWeight: 800, color: '#3b1f8e' },
  dt: { fontSize: '20px', fontWeight: 700, color: '#d4af37', textAlign: 'left' as const },
  dm: { fontSize: '10px', color: '#64748b', marginTop: '4px', textAlign: 'left' as const, lineHeight: '1.8' },
  summary: {
    display: 'flex', justifyContent: 'space-between', alignItems: 'center',
    marginBottom: '16px', padding: '12px 16px', background: '#faf5ff',
    borderRadius: '10px', border: '1px solid #ede9fe',
  },
  tbl: { width: '100%', borderCollapse: 'collapse' as const, marginBottom: '16px' },
  th: {
    background: '#3b1f8e', color: '#fff', fontSize: '9px', padding: '9px 6px',
    textAlign: 'center' as const, fontWeight: 700, letterSpacing: '0.3px',
  },
  td: { fontSize: '10px', padding: '7px 6px', textAlign: 'center' as const, borderBottom: '1px solid #f1f5f9', color: '#334155' },
  tb: {
    width: '280px', marginRight: 'auto' as const,
    padding: '12px 16px', background: '#faf5ff', borderRadius: '10px',
    border: '1px solid #e9d5ff',
  },
  tr: { display: 'flex', justifyContent: 'space-between', padding: '4px 0', fontSize: '11px', color: '#475569' },
  gt: {
    display: 'flex', justifyContent: 'space-between', padding: '8px 0 0',
    fontSize: '14px', fontWeight: 800, color: '#3b1f8e',
    borderTop: '2px solid #d4af37', marginTop: '4px',
  },
  ft: { display: 'flex', justifyContent: 'space-between', marginTop: '30px', paddingTop: '16px', borderTop: '1px solid #e2e8f0' },
  sg: { width: '160px', textAlign: 'center' as const, fontSize: '10px', color: '#94a3b8', padding: '8px 0', borderTop: '1px solid #cbd5e1' },
  pf: { position: 'fixed' as const, bottom: '10px', left: 0, right: 0, textAlign: 'center' as const, fontSize: '7px', color: '#cbd5e1' },
};

const txnLabels: Record<string, string> = {
  invoice: "فاتورة", payment: "دفعة", credit_note: "إشعار دائن", purchase: "مشتريات",
};

export default function StatementPrintTemplate({
  title, entityName, entityCode, entityType, openingBalance, transactions, closingBalance, totalDebit, totalCredit, company, fromDate, toDate,
}: Props) {
  return (
    <div id="print-area" style={S.c}>
      <div style={S.bar} />
      <div style={S.hdr}>
        <div>
          <div style={S.cn}>{htmlEscape(company.name) || "PRO MAX OS"}</div>
          <div style={{ fontSize: 10, color: '#94a3b8', marginTop: 2 }}>{htmlEscape(company.factory_name)}</div>
          <div style={{ fontSize: 9, color: '#94a3b8', marginTop: 4 }}>{htmlEscape(company.address)}</div>
        </div>
        <div>
          <div style={S.dt}>{title}</div>
          <div style={S.dm}>
            {entityType === "customer" ? "العميل" : "المورد"}: {htmlEscape(entityName)}<br />
            {entityCode && <>الكود: {htmlEscape(entityCode)}<br /></>}
            {fromDate && toDate && <>{formatDate(fromDate)} → {formatDate(toDate)}</>}
          </div>
        </div>
      </div>

      <div style={S.summary}>
        <span>الرصيد الافتتاحي: <strong style={{ color: openingBalance >= 0 ? '#059669' : '#dc2626' }}>{formatOMR(openingBalance)}</strong></span>
        <span>إجمالي المدين: <strong>{formatOMR(totalDebit)}</strong></span>
        <span>إجمالي الدائن: <strong>{formatOMR(totalCredit)}</strong></span>
        <span>عدد المعاملات: <strong>{transactions.length}</strong></span>
      </div>

      <table style={S.tbl}>
        <thead>
          <tr>
            <th style={{ ...S.th, borderRadius: '0 8px 0 0' }}>#</th>
            <th style={{ ...S.th, minWidth: 80 }}>التاريخ</th>
            <th style={S.th}>المرجع</th>
            <th style={S.th}>النوع</th>
            <th style={S.th}>مدين</th>
            <th style={S.th}>دائن</th>
            <th style={{ ...S.th, minWidth: 80 }}>الرصيد</th>
            <th style={{ ...S.th, borderRadius: '8px 0 0 0' }}>ملاحظات</th>
          </tr>
        </thead>
        <tbody>
          {transactions.map((txn, i) => (
            <tr key={i} style={i % 2 === 1 ? { background: '#faf9ff' } : {}}>
              <td style={S.td}>{i + 1}</td>
              <td style={S.td}>{formatDate(txn.date)}</td>
              <td style={S.td}>{htmlEscape(txn.ref_no) || "—"}</td>
              <td style={S.td}>{txnLabels[txn.txn_type] || txn.txn_type}</td>
              <td style={{ ...S.td, color: txn.debit_milli > 0 ? '#059669' : '#94a3b8', fontWeight: txn.debit_milli > 0 ? 700 : 400 }}>{txn.debit_milli > 0 ? formatOMR(txn.debit_milli) : "—"}</td>
              <td style={{ ...S.td, color: txn.credit_milli > 0 ? '#dc2626' : '#94a3b8', fontWeight: txn.credit_milli > 0 ? 700 : 400 }}>{txn.credit_milli > 0 ? formatOMR(txn.credit_milli) : "—"}</td>
              <td style={{ ...S.td, fontWeight: 600, color: txn.balance_milli >= 0 ? '#1e293b' : '#dc2626' }}>{formatOMR(txn.balance_milli)}</td>
              <td style={{ ...S.td, fontSize: 9, color: '#94a3b8' }}>{htmlEscape(txn.notes) || "—"}</td>
            </tr>
          ))}
        </tbody>
      </table>

      <div style={S.tb}>
        <div style={S.tr}><span>إجمالي المدين</span><span style={{ color: '#059669', fontWeight: 700 }}>{formatOMR(totalDebit)}</span></div>
        <div style={S.tr}><span>إجمالي الدائن</span><span style={{ color: '#dc2626', fontWeight: 700 }}>{formatOMR(totalCredit)}</span></div>
        <div style={S.gt}><span>الرصيد الختامي</span><span style={{ color: closingBalance >= 0 ? '#059669' : '#dc2626' }}>{formatOMR(closingBalance)}</span></div>
      </div>

      <div style={S.ft}>
        <div style={S.sg}>ختم الشركة</div>
        <div style={S.sg}>التوقيع</div>
      </div>

      <div style={S.pf}>
        تمت الطباعة من PRO MAX OS © {new Date().getFullYear()} | {title} - {htmlEscape(entityName)}
      </div>

      <style>{`
        @import url('https://fonts.googleapis.com/css2?family=Cairo:wght@400;600;700;800&display=swap');
        @media print { body { -webkit-print-color-adjust: exact; print-color-adjust: exact; margin: 0; } @page { margin: 0; } }
      `}</style>
    </div>
  );
}