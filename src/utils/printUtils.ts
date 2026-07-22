import { formatOMR, formatDate, formatDateTime } from "@/lib/utils";

export { formatOMR, formatDate, formatDateTime };

export function printComponent(elementId: string) {
  const el = document.getElementById(elementId);
  if (!el) return;
  const html = el.innerHTML;
  const win = window.open("", "_blank", "width=900,height=700");
  if (!win) return;
  win.document.write(`<!DOCTYPE html><html lang="ar" dir="rtl"><head><meta charset="utf-8"><title>طباعة</title>
  <style>
    @page{size:A4;margin:12mm 15mm}
    *{margin:0;padding:0;box-sizing:border-box}
    body{font-family:'Segoe UI',Tahoma,sans-serif;color:#1a1a2e;font-size:11pt;line-height:1.6}
    .print-header{display:flex;justify-content:space-between;align-items:flex-start;border-bottom:3px solid #4c1d95;padding-bottom:12px;margin-bottom:16px}
    .print-header .company-name{font-size:18pt;font-weight:800;color:#4c1d95}
    .print-header .factory-name{font-size:11pt;color:#6b7280}
    .print-header .doc-title{font-size:14pt;font-weight:700;color:#312e81;text-align:center}
    .print-header .doc-meta{text-align:center;font-size:9pt;color:#6b7280}
    .info-grid{display:grid;grid-template-columns:1fr 1fr;gap:8px 24px;margin-bottom:16px;font-size:10pt}
    .info-grid .label{color:#6b7280;font-weight:600}
    .info-grid .value{color:#1a1a2e}
    table{width:100%;border-collapse:collapse;margin-bottom:16px}
    th{background:#4c1d95;color:#fff;padding:8px 10px;font-size:10pt;text-align:right;font-weight:600}
    td{padding:7px 10px;border-bottom:1px solid #e5e7eb;font-size:10pt}
    tr:nth-child(even){background:#f8f9fc}
    .totals{width:280px;margin-left:0;margin-right:auto;margin-bottom:16px}
    .totals td{padding:5px 10px;font-size:10pt}
    .totals .total-row td{font-weight:700;font-size:11pt;border-top:2px solid #4c1d95;color:#4c1d95}
    .footer{display:flex;justify-content:space-between;align-items:flex-end;border-top:2px solid #e5e7eb;padding-top:16px;margin-top:24px;font-size:9pt;color:#6b7280}
    .footer .stamp-box{width:140px;height:80px;border:1px dashed #d1d5db;display:flex;align-items:center;justify-content:center;color:#9ca3af;font-size:8pt}
    .footer .sig-box{text-align:center}
    .footer .sig-line{width:160px;border-bottom:1px solid #1a1a2e;margin-bottom:4px;padding-top:48px}
    @media print{body{margin:0;padding:0}}
  </style></head><body>${html}</body></html>`);
  win.document.close();
  win.focus();
  setTimeout(() => { win.print(); win.close(); }, 300);
}

export function htmlEscape(s: string | null | undefined): string {
  if (!s) return "";
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}
