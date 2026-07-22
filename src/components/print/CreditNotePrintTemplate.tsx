import { formatOMR, formatDate, htmlEscape } from "@/utils/printUtils";

interface Props {
  data: any;
}

export default function CreditNotePrintTemplate({ data }: Props) {
  if (!data) return null;
  const { credit_note: cn, customer, lines, company } = data;
  return (
    <div id="print-area">
      <div className="print-header">
        <div>
          <div className="company-name">{htmlEscape(company.name) || "المصانع"}</div>
          <div className="factory-name">{htmlEscape(company.factory_name)}</div>
          <div style={{ fontSize: 9, color: "#6b7280", marginTop: 4 }}>
            {htmlEscape(company.address)}<br />
            VAT: {htmlEscape(company.vat_number)}
          </div>
        </div>
        <div>
          <div className="doc-title">إشعار دائن</div>
          <div className="doc-meta">
            رقم: {htmlEscape(cn.cn_no) || `#${cn.id}`}<br />
            التاريخ: {formatDate(cn.date)}<br />
            الفاتورة المرجعية: {htmlEscape(cn.invoice_no) || "—"}
          </div>
        </div>
      </div>

      <div className="info-grid">
        <div><span className="label">العميل: </span><span className="value">{htmlEscape(customer.name)}</span></div>
        <div><span className="label">السبب: </span><span className="value">{htmlEscape(cn.reason) || "—"}</span></div>
      </div>

      {lines.length > 0 && (
        <table>
          <thead>
            <tr>
              <th>#</th>
              <th>المنتج</th>
              <th>الكراتين</th>
              <th>السعر</th>
              <th>الصافي</th>
              <th>الضريبة</th>
            </tr>
          </thead>
          <tbody>
            {lines.map((line: any, i: number) => (
              <tr key={line.id || i}>
                <td>{i + 1}</td>
                <td>{htmlEscape(line.product_name)}</td>
                <td style={{ textAlign: "center" }}>{line.cartons}</td>
                <td style={{ textAlign: "left" }}>{formatOMR(line.unit_price_milli)}</td>
                <td style={{ textAlign: "left" }}>{formatOMR(line.line_net_milli)}</td>
                <td style={{ textAlign: "left" }}>{formatOMR(line.vat_milli)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      <table className="totals">
        <tbody>
          <tr><td>الصافي</td><td style={{ textAlign: "left" }}>{formatOMR(cn.net_milli)}</td></tr>
          <tr><td>الضريبة</td><td style={{ textAlign: "left" }}>{formatOMR(cn.vat_milli)}</td></tr>
          <tr className="total-row"><td>الإجمالي</td><td style={{ textAlign: "left" }}>{formatOMR(cn.total_milli)}</td></tr>
        </tbody>
      </table>

      {cn.notes && (
        <div style={{ marginBottom: 16, padding: "8px 12px", background: "#f8f9fc", borderRadius: 6, fontSize: 9 }}>
          <strong>ملاحظات:</strong> {htmlEscape(cn.notes)}
        </div>
      )}

      <div className="footer">
        <div>{company.footer_notes && htmlEscape(company.footer_notes)}</div>
        <div className="stamp-box">ختم الشركة</div>
        <div className="sig-box">
          <div className="sig-line"></div>
          <div>توقيع المسؤول</div>
        </div>
      </div>
    </div>
  );
}
