import { useTranslation } from "react-i18next";
import { formatDate, htmlEscape } from "@/utils/printUtils";

interface Props {
  data: any;
}

export default function DeliveryNotePrintTemplate({ data }: Props) {
  if (!data) return null;
  const { t, i18n } = useTranslation();
  const { invoice, customer, lines, company } = data;
  const isRtl = ['ar', 'ur'].includes(i18n.language);
  return (
    <div id="print-area" style={{ direction: isRtl ? 'rtl' : 'ltr' }}>
      <div className="print-header">
        <div>
          <div className="company-name">{htmlEscape(company.name) || t("print.factories")}</div>
          <div className="factory-name">{htmlEscape(company.factory_name)}</div>
          <div style={{ fontSize: 9, color: "#6b7280", marginTop: 4 }}>
            {htmlEscape(company.address)}<br />
            {htmlEscape(company.phone)}
          </div>
        </div>
        <div>
          <div className="doc-title">{t("print.deliveryNoteTitle")}</div>
          <div className="doc-meta">
            {t("invoice.invoiceNo")}: {htmlEscape(invoice.inv_no)}<br />
            {t("print.dateLabel")} {formatDate(invoice.date)}
          </div>
        </div>
      </div>

      <div className="info-grid">
        <div><span className="label">{t("print.customerLabel")}: </span><span className="value">{htmlEscape(invoice.customer_name)}</span></div>
        <div><span className="label">{t("customer.address")}: </span><span className="value">{htmlEscape(customer.address)}</span></div>
        <div><span className="label">{t("customer.phone")}: </span><span className="value">{htmlEscape(customer.phone)}</span></div>
      </div>

      <table>
        <thead>
          <tr>
            <th>#</th>
            <th>{t("print.productLabel")}</th>
            <th>{t("print.cartons")}</th>
            <th>{t("print.cupsPerCarton")}</th>
            <th>{t("print.totalInCups")}</th>
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
        <strong>{t("print.totalCartons")}: </strong>{lines.reduce((s: number, l: any) => s + (l.cartons || 0), 0)}
        &nbsp;&nbsp;|&nbsp;&nbsp;
        <strong>{t("print.totalCups")}: </strong>{lines.reduce((s: number, l: any) => s + (l.qty_cups || 0), 0).toLocaleString()}
      </div>

      <div className="footer">
        <div className="sig-box">
          <div className="sig-line"></div>
          <div>{t("print.receivedBy")}</div>
        </div>
        <div className="stamp-box">{t("print.companyStamp")}</div>
        <div className="sig-box">
          <div className="sig-line"></div>
          <div>{t("print.driverSignature")}</div>
        </div>
      </div>
    </div>
  );
}
