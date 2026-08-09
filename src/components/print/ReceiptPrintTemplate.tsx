import { useTranslation } from "react-i18next";
import { formatOMR, formatDate, htmlEscape } from "@/utils/printUtils";

interface Props {
  data: any;
}

export default function ReceiptPrintTemplate({ data }: Props) {
  if (!data) return null;
  const { t, i18n } = useTranslation();
  const { payment, customer, company } = data;
  const isRtl = ['ar', 'ur'].includes(i18n.language);
  return (
    <div id="print-area" style={{ direction: isRtl ? 'rtl' : 'ltr' }}>
      <div className="print-header">
        <div>
          <div className="company-name">{htmlEscape(company.name) || t("print.factories")}</div>
          <div className="factory-name">{htmlEscape(company.factory_name)}</div>
          <div style={{ fontSize: 9, color: "#6b7280", marginTop: 4 }}>
            {htmlEscape(company.address)}<br />
            {htmlEscape(company.phone)} | {htmlEscape(company.email)}
          </div>
        </div>
        <div>
          <div className="doc-title">{t("print.receiptTitle")}</div>
          <div className="doc-meta">
            {t("print.invoiceNoLabel")} {htmlEscape(payment.receipt_no) || `#${payment.id}`}<br />
            {t("print.dateLabel")} {formatDate(payment.date)}
          </div>
        </div>
      </div>

      <div className="info-grid">
        <div><span className="label">{t("print.customerLabel")}: </span><span className="value">{htmlEscape(customer.name)}</span></div>
        <div><span className="label">{t("print.amount")}: </span><span className="value" style={{ fontWeight: 700, fontSize: 14, color: "#059669" }}>{formatOMR(payment.amount_milli)}</span></div>
        <div><span className="label">{t("print.paymentMethod")}: </span><span className="value">{htmlEscape(payment.method) || t("print.cashLabel")}</span></div>
        <div><span className="label">{t("print.reference")}: </span><span className="value">{htmlEscape(payment.reference) || "—"}</span></div>
      </div>

      {payment.notes && (
        <div style={{ marginBottom: 16, padding: "8px 12px", background: "#f8f9fc", borderRadius: 6, fontSize: 9 }}>
          <strong>{t("common.notes")}:</strong> {htmlEscape(payment.notes)}
        </div>
      )}

      <div className="footer">
        <div>{company.footer_notes && htmlEscape(company.footer_notes)}</div>
        <div className="stamp-box">{t("print.companyStamp")}</div>
        <div className="sig-box">
          <div className="sig-line"></div>
          <div>{t("print.receivedBy")}</div>
        </div>
      </div>
    </div>
  );
}
