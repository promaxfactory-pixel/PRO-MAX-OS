import { useTranslation } from "react-i18next";
import { formatOMR, formatDate, htmlEscape } from "@/utils/printUtils";

interface Props {
  data: any;
}

export default function CreditNotePrintTemplate({ data }: Props) {
  if (!data) return null;
  const { t, i18n } = useTranslation();
  const { credit_note: cn, customer, lines, company } = data;
  const isRtl = ['ar', 'ur'].includes(i18n.language);
  return (
    <div id="print-area" style={{ direction: isRtl ? 'rtl' : 'ltr' }}>
      <div className="print-header">
        <div>
          <div className="company-name">{htmlEscape(company.name) || t("print.factories")}</div>
          <div className="factory-name">{htmlEscape(company.factory_name)}</div>
          <div style={{ fontSize: 9, color: "#6b7280", marginTop: 4 }}>
            {htmlEscape(company.address)}<br />
            {t("common.vat")}: {htmlEscape(company.vat_number)}
          </div>
        </div>
        <div>
          <div className="doc-title">{t("print.creditNoteTitle")}</div>
          <div className="doc-meta">
            {t("print.invoiceNoLabel")} {htmlEscape(cn.cn_no) || `#${cn.id}`}<br />
            {t("print.dateLabel")} {formatDate(cn.date)}<br />
            {t("print.referenceInvoice")}: {htmlEscape(cn.invoice_no) || "—"}
          </div>
        </div>
      </div>

      <div className="info-grid">
        <div><span className="label">{t("print.customerLabel")}: </span><span className="value">{htmlEscape(customer.name)}</span></div>
        <div><span className="label">{t("print.reason")}: </span><span className="value">{htmlEscape(cn.reason) || "—"}</span></div>
      </div>

      {lines.length > 0 && (
        <table>
          <thead>
            <tr>
              <th>#</th>
              <th>{t("print.productLabel")}</th>
              <th>{t("print.cartons")}</th>
              <th>{t("print.price")}</th>
              <th>{t("print.net")}</th>
              <th>{t("common.vat")}</th>
            </tr>
          </thead>
          <tbody>
            {lines.map((line: any, i: number) => (
              <tr key={line.id || i}>
                <td>{i + 1}</td>
                <td>{htmlEscape(line.product_name)}</td>
                <td style={{ textAlign: "center" }}>{line.cartons}</td>
                <td style={{ textAlign: isRtl ? 'right' : 'left' }}>{formatOMR(line.unit_price_milli)}</td>
                <td style={{ textAlign: isRtl ? 'right' : 'left' }}>{formatOMR(line.line_net_milli)}</td>
                <td style={{ textAlign: isRtl ? 'right' : 'left' }}>{formatOMR(line.vat_milli)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      <table className="totals">
        <tbody>
          <tr><td>{t("print.net")}</td><td style={{ textAlign: isRtl ? 'right' : 'left' }}>{formatOMR(cn.net_milli)}</td></tr>
          <tr><td>{t("common.vat")}</td><td style={{ textAlign: isRtl ? 'right' : 'left' }}>{formatOMR(cn.vat_milli)}</td></tr>
          <tr className="total-row"><td>{t("common.total")}</td><td style={{ textAlign: isRtl ? 'right' : 'left' }}>{formatOMR(cn.total_milli)}</td></tr>
        </tbody>
      </table>

      {cn.notes && (
        <div style={{ marginBottom: 16, padding: "8px 12px", background: "#f8f9fc", borderRadius: 6, fontSize: 9 }}>
          <strong>{t("common.notes")}:</strong> {htmlEscape(cn.notes)}
        </div>
      )}

      <div className="footer">
        <div>{company.footer_notes && htmlEscape(company.footer_notes)}</div>
        <div className="stamp-box">{t("print.companyStamp")}</div>
        <div className="sig-box">
          <div className="sig-line"></div>
          <div>{t("print.issuedBy")}</div>
        </div>
      </div>
    </div>
  );
}
