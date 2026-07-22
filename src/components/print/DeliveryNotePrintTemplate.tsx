import { formatDate, htmlEscape } from "@/utils/printUtils";

interface Props {
  data: any;
}

export default function DeliveryNotePrintTemplate({ data }: Props) {
  if (!data) return null;
  const { invoice, customer, lines, company } = data;
  return (
    <div id="print-area">
      <div className="print-header">
        <div>
          <div className="company-name">{htmlEscape(company.name) || "المصانع"}</div>
          <div className="factory-name">{htmlEscape(company.factory_name)}</div>
          <div style={{ fontSize: 9, color: "#6b7280", marginTop: 4 }}>
            {htmlEscape(company.address)}<br />
            {htmlEscape(company.phone)}
          </div>
        </div>
        <div>
          <div className="doc-title">إيصال توصيل</div>
          <div className="doc-meta">
            رقم الفاتورة: {htmlEscape(invoice.inv_no)}<br />
            التاريخ: {formatDate(invoice.date)}
          </div>
        </div>
      </div>

      <div className="info-grid">
        <div><span className="label">العميل: </span><span className="value">{htmlEscape(invoice.customer_name)}</span></div>
        <div><span className="label">العنوان: </span><span className="value">{htmlEscape(customer.address)}</span></div>
        <div><span className="label">الهاتف: </span><span className="value">{htmlEscape(customer.phone)}</span></div>
      </div>

      <table>
        <thead>
          <tr>
            <th>#</th>
            <th>المنتج</th>
            <th>الكراتين</th>
            <th>كوب/كرتون</th>
            <th>الإجمالي (كوب)</th>
          </tr>
        </thead>
        <tbody>
          {lines.map((line: any, i: number) => (
            <tr key={line.id || i}>
              <td>{i + 1}</td>
              <td>{htmlEscape(line.product_name)}</td>
              <td style={{ textAlign: "center" }}>{line.cartons}</td>
              <td style={{ textAlign: "center" }}>{line.cups_per_carton}</td>
              <td style={{ textAlign: "center" }}>{line.qty_cups?.toLocaleString()}</td>
            </tr>
          ))}
        </tbody>
      </table>

      <div style={{ marginTop: 16, padding: "8px 12px", background: "#f8f9fc", borderRadius: 6, fontSize: 10 }}>
        <strong>إجمالي الكراتين: </strong>{lines.reduce((s: number, l: any) => s + (l.cartons || 0), 0)}
        &nbsp;&nbsp;|&nbsp;&nbsp;
        <strong>إجمالي الأكواب: </strong>{lines.reduce((s: number, l: any) => s + (l.qty_cups || 0), 0).toLocaleString()}
      </div>

      <div className="footer">
        <div className="sig-box">
          <div className="sig-line"></div>
          <div>توقيع المستلم</div>
        </div>
        <div className="stamp-box">ختم الشركة</div>
        <div className="sig-box">
          <div className="sig-line"></div>
          <div>توقيع السائق</div>
        </div>
      </div>
    </div>
  );
}
