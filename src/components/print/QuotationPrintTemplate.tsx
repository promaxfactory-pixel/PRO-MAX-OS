import { formatOMR, formatDate, htmlEscape } from "@/utils/printUtils";

function addDays(base: string, days: number): string {
  const d = new Date(`${base}T00:00:00`);
  if (isNaN(d.getTime())) return base;
  d.setDate(d.getDate() + days);
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${d.getFullYear()}-${m}-${day}`;
}

const S = {
  c: {
    fontFamily: "'Cairo', 'system-ui', sans-serif",
    direction: "rtl" as const,
    padding: "30px 35px",
    color: "#1a1a2e",
    background: "#ffffff",
    position: "relative" as const,
    overflow: "hidden",
  },
  bar: {
    position: "absolute" as const, top: 0, left: 0, right: 0, height: "5px",
    background: "linear-gradient(90deg, #0ea5e9, #38bdf8, #0ea5e9)",
  },
  wm: {
    position: "absolute" as const, bottom: "80px", right: "40px",
    fontSize: "70px", fontWeight: 900, color: "#f8f4ff", opacity: 0.25,
    transform: "rotate(-12deg)", pointerEvents: "none" as const, zIndex: 0,
    fontFamily: "'Plus Jakarta Sans', sans-serif",
    letterSpacing: "8px",
  },
  hdr: {
    display: "flex", justifyContent: "space-between", alignItems: "flex-start",
    marginBottom: "22px", paddingBottom: "16px",
    borderBottom: "2px solid #0ea5e9",
    position: "relative" as const, zIndex: 1,
  },
  cn: {
    fontSize: "24px", fontWeight: 800, color: "#0c4a6e",
    letterSpacing: "0.5px", fontFamily: "'Plus Jakarta Sans', 'Cairo', sans-serif",
  },
  ci: {
    fontSize: "10px", color: "#94a3b8", marginTop: "4px", lineHeight: "1.6",
  },
  dt: {
    fontSize: "22px", fontWeight: 700, color: "#0ea5e9", textAlign: "left" as const,
    letterSpacing: "1px", fontFamily: "'Plus Jakarta Sans', 'Cairo', sans-serif",
  },
  dm: {
    fontSize: "10px", color: "#64748b", marginTop: "6px", textAlign: "left" as const, lineHeight: "1.8",
  },
  cg: {
    display: "grid", gridTemplateColumns: "1fr 1fr", gap: "10px",
    marginBottom: "20px", padding: "14px 16px", background: "#f0f9ff",
    borderRadius: "12px", border: "1px solid #e0f2fe",
    position: "relative" as const, zIndex: 1,
  },
  lb: { color: "#94a3b8", fontSize: "9px", fontWeight: 600, textTransform: "uppercase" as const, letterSpacing: "0.8px", marginBottom: "2px" },
  vl: { color: "#1e293b", fontSize: "12px", fontWeight: 700 },
  tbl: { width: "100%", borderCollapse: "collapse" as const, marginBottom: "20px", position: "relative" as const, zIndex: 1 },
  th: {
    background: "#0c4a6e", color: "#fff", fontSize: "9.5px", padding: "10px 8px",
    textAlign: "center" as const, fontWeight: 700, letterSpacing: "0.5px",
  },
  td: {
    fontSize: "10.5px", padding: "8px", textAlign: "center" as const,
    borderBottom: "1px solid #f1f5f9", color: "#334155",
  },
  tb: {
    width: "290px", marginRight: "auto" as const, marginBottom: "18px",
    padding: "14px 18px", background: "#f0f9ff", borderRadius: "12px",
    border: "1px solid #bae6fd", position: "relative" as const, zIndex: 1,
  },
  tr: {
    display: "flex", justifyContent: "space-between", padding: "5px 0",
    fontSize: "11px", color: "#475569",
  },
  gt: {
    display: "flex", justifyContent: "space-between", padding: "10px 0 0",
    fontSize: "16px", fontWeight: 800, color: "#0c4a6e",
    borderTop: "2px solid #0ea5e9", marginTop: "6px",
  },
  nt: {
    fontSize: "10px", color: "#64748b", marginBottom: "15px",
    padding: "10px 14px", background: "#fffbeb", borderRadius: "10px",
    border: "1px solid #fef3c7", lineHeight: "1.6", position: "relative" as const, zIndex: 1,
  },
  ft: {
    display: "flex", justifyContent: "space-between", marginTop: "35px",
    paddingTop: "20px", borderTop: "1px solid #e2e8f0",
    position: "relative" as const, zIndex: 1,
  },
  sg: {
    width: "160px", textAlign: "center" as const, fontSize: "10px",
    color: "#94a3b8", padding: "8px 0", borderTop: "1px solid #cbd5e1",
  },
  pf: {
    position: "fixed" as const, bottom: "12px", left: 0, right: 0,
    textAlign: "center" as const, fontSize: "7px", color: "#cbd5e1", zIndex: 1,
  },
};

export default function QuotationPrintTemplate({ data }: Props) {
  if (!data) return null;
  const { quotation, lines, company, client_name } = data;
  const currency = quotation.currency || "OMR";
  return (
    <div id="print-area" style={S.c}>
      <div style={S.bar} />
      <div style={S.wm}>PRO MAX OS</div>

      <div style={S.hdr}>
        <div>
          <div style={S.cn}>{htmlEscape(company.factory_name || company.name) || "PRO MAX OS"}</div>
          <div style={S.ci}>
            {htmlEscape(company.address)}<br />
            {htmlEscape(company.phone)}{company.email ? ` | ${htmlEscape(company.email)}` : ""}
          </div>
        </div>
        <div>
          <div style={S.dt}>عرض سعر</div>
          <div style={S.dm}>
            <strong>رقم:</strong> {htmlEscape(quotation.quote_no)}<br />
            <strong>التاريخ:</strong> {formatDate(quotation.date)}<br />
            <strong>ساري حتى:</strong> {formatDate(addDays(quotation.date, quotation.validity_days || 7))}
          </div>
        </div>
      </div>

      <div style={S.cg}>
        <div><div style={S.lb}>إلى / السيد</div><div style={S.vl}>{htmlEscape(client_name)}</div></div>
        <div><div style={S.lb}>المرجع</div><div style={S.vl}>{htmlEscape(quotation.client_contact || "—")}</div></div>
        {quotation.client_phone && <div><div style={S.lb}>الهاتف</div><div style={S.vl} dir="ltr">{htmlEscape(quotation.client_phone)}</div></div>}
        {quotation.client_email && <div><div style={S.lb}>البريد</div><div style={S.vl} dir="ltr">{htmlEscape(quotation.client_email)}</div></div>}
        {quotation.client_address && <div><div style={S.lb}>العنوان</div><div style={S.vl}>{htmlEscape(quotation.client_address)}</div></div>}
      </div>

      {quotation.title && (
        <div style={{ ...S.nt, background: "#f0f9ff", border: "1px solid #e0f2fe" }}>
          <strong>الموضوع: </strong>{htmlEscape(quotation.title)}
        </div>
      )}

      <table style={S.tbl}>
        <thead>
          <tr>
            <th style={{ ...S.th, borderRadius: "0 8px 0 0" }}>#</th>
            <th style={{ ...S.th, textAlign: "right" }}>الصنف</th>
            <th style={S.th}>المقاس</th>
            <th style={S.th}>كوب/كرتون</th>
            <th style={S.th}>الكراتين</th>
            <th style={S.th}>إجمالي الأكواب</th>
            <th style={S.th}>سعر الكرتون</th>
            <th style={{ ...S.th, borderRadius: "8px 0 0 0" }}>الإجمالي</th>
          </tr>
        </thead>
        <tbody>
          {lines?.map((line: any, i: number) => (
            <tr key={line.id || i} style={i % 2 === 1 ? { background: "#faf9ff" } : {}}>
              <td style={S.td}>{i + 1}</td>
              <td style={{ ...S.td, textAlign: "right", fontWeight: 600 }}>{htmlEscape(line.item_name || line.product_name)}</td>
              <td style={S.td}>{htmlEscape(line.cup_size || "—")}</td>
              <td style={S.td}>{(line.cups_per_carton ?? 1000).toLocaleString()}</td>
              <td style={S.td}>{line.cartons?.toLocaleString()}</td>
              <td style={S.td}>{(line.cartons * (line.cups_per_carton ?? 1000)).toLocaleString()}</td>
              <td style={S.td}>{formatOMR(line.unit_price_milli)}</td>
              <td style={{ ...S.td, fontWeight: 700 }}>{formatOMR(line.line_total_milli)}</td>
            </tr>
          ))}
        </tbody>
      </table>

      <div style={S.tb}>
        <div style={S.tr}><span>المجموع الفرعي</span><span>{formatOMR(quotation.net_milli)}</span></div>
        {quotation.discount_milli > 0 && (
          <div style={S.tr}><span>الخصم</span><span style={{ color: "#059669" }}>- {formatOMR(quotation.discount_milli)}</span></div>
        )}
        <div style={S.gt}><span>الإجمالي</span><span>{formatOMR(quotation.total_milli)} {currency}</span></div>
      </div>

      {quotation.terms && (
        <div style={S.nt}><strong>شروط العرض: </strong>{htmlEscape(quotation.terms)}</div>
      )}
      {quotation.notes && (
        <div style={{ ...S.nt, background: "#f0f9ff", border: "1px solid #e0f2fe" }}><strong>ملاحظات: </strong>{htmlEscape(quotation.notes)}</div>
      )}

      <div style={S.ft}>
        <div style={S.sg}>تقديم / المُعد</div>
        <div style={S.sg}>اعتماد العميل</div>
        <div style={S.sg}>ختم المصنع</div>
      </div>

      <div style={S.pf}>
        تمت الطباعة من PRO MAX OS © {new Date().getFullYear()} | عرض سعر رقم {htmlEscape(quotation.quote_no)}
      </div>

      <style>{`
        @import url('https://fonts.googleapis.com/css2?family=Cairo:wght@400;600;700;800&family=Plus+Jakarta+Sans:wght@400;700;800&display=swap');
        @media print {
          body { -webkit-print-color-adjust: exact; print-color-adjust: exact; margin: 0; }
          #print-area { position: relative; }
          @page { margin: 0; }
        }
      `}</style>
    </div>
  );
}

interface Props {
  data: any;
}
