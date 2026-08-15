import { formatOMR, formatDate, htmlEscape } from "@/utils/printUtils";

interface Props {
  data: any;
}

export default function SupplierReceiptPrintTemplate({ data }: Props) {
  if (!data) return null;
  const { payment, supplier, company } = data;
  return (
    <div id="print-area">
      <div className="print-header">
        <div>
          <div className="company-name">{htmlEscape(company.name) || "المصانع"}</div>
          <div className="factory-name">{htmlEscape(company.factory_name)}</div>
          <div style={{ fontSize: 9, color: "#6b7280", marginTop: 4 }}>
            {htmlEscape(company.address)}<br />
            {htmlEscape(company.phone)} | {htmlEscape(company.email)}
          </div>
        </div>
        <div>
          <div className="doc-title">سند صرف</div>
          <div className="doc-meta">
            رقم: {htmlEscape(payment.receipt_no) || `#${payment.id}`}<br />
            التاريخ: {formatDate(payment.date)}
          </div>
        </div>
      </div>

      <div className="info-grid">
        <div><span className="label">المورد: </span><span className="value">{htmlEscape(supplier.name)}</span></div>
        <div><span className="label">المبلغ: </span><span className="value" style={{ fontWeight: 700, fontSize: 14, color: "#b45309" }}>{formatOMR(payment.amount_milli)}</span></div>
        <div><span className="label">طريقة الدفع: </span><span className="value">{htmlEscape(payment.method) || "نقداً"}</span></div>
        <div><span className="label">المرجع: </span><span className="value">{htmlEscape(payment.reference) || "—"}</span></div>
      </div>

      {payment.notes && (
        <div style={{ marginBottom: 16, padding: "8px 12px", background: "#f8f9fc", borderRadius: 6, fontSize: 9 }}>
          <strong>ملاحظات:</strong> {htmlEscape(payment.notes)}
        </div>
      )}

      <div className="footer">
        <div>{company.footer_notes && htmlEscape(company.footer_notes)}</div>
        <div className="stamp-box">ختم الشركة</div>
        <div className="sig-box">
          <div className="sig-line"></div>
          <div>توقيع المستلم</div>
        </div>
      </div>
    </div>
  );
}
