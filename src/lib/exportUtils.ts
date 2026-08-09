import i18n from "@/i18n";

export interface CsvColumn {
  key: string;
  header: string;
  format?: (value: unknown, row: Record<string, unknown>) => string;
}

function csvCell(value: unknown): string {
  const s = value === null || value === undefined ? "" : String(value);
  return `"${s.replace(/"/g, '""')}"`;
}

export function buildCsv(rows: Record<string, unknown>[], columns: CsvColumn[]): string {
  const header = columns.map((c) => csvCell(c.header)).join(",");
  const lines = rows.map((row) =>
    columns
      .map((c) => {
        const raw = row[c.key];
        const val = c.format ? c.format(raw, row) : raw;
        return csvCell(val);
      })
      .join(",")
  );
  return [header, ...lines].join("\r\n");
}

export function downloadTextFile(filename: string, content: string): void {
  const blob = new Blob(["\uFEFF" + content], { type: "text/csv;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  setTimeout(() => URL.revokeObjectURL(url), 500);
}

export function exportCsv(
  filename: string,
  rows: Record<string, unknown>[],
  columns: CsvColumn[]
): void {
  downloadTextFile(filename, buildCsv(rows, columns));
}

export function toPairs(rows: Record<string, unknown>[]): { label: string; value: string }[] {
  return rows.map((r) => ({
    label: String(r.label ?? ""),
    value: String(r.value ?? ""),
  }));
}

export function printReport(
  title: string,
  sections: { heading: string; rows: { label: string; value: string }[] }[],
  meta?: string
): void {
  const win = window.open("", "_blank", "width=900,height=650");
  if (!win) return;

  const esc = (s: string) =>
    s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");

  const body = sections
    .map(
      (sec) => `
      <h2>${esc(sec.heading)}</h2>
      <table>
        <thead><tr><th>${esc(i18n.t("print.item"))}</th><th>${esc(i18n.t("print.value"))}</th></tr></thead>
        <tbody>
          ${sec.rows
            .map(
              (r) =>
                `<tr><td>${esc(r.label)}</td><td>${esc(r.value)}</td></tr>`
            )
            .join("")}
        </tbody>
      </table>`
    )
    .join("");

  const isRtl = ['ar', 'ur'].includes(i18n.language);
  win.document.write(`<!DOCTYPE html>
<html dir="${isRtl ? "rtl" : "ltr"}" lang="${i18n.language}">
<head>
<meta charset="utf-8">
<title>${esc(title)}</title>
<style>
  * { box-sizing: border-box; }
  body { font-family: "Segoe UI", Tahoma, Arial, sans-serif; margin: 40px; color: #111; background: #fff; }
  h1 { font-size: 20px; margin: 0 0 4px; color: #1a1a2e; }
  .meta { font-size: 12px; color: #666; margin-bottom: 24px; }
  h2 { font-size: 15px; color: #b8860b; margin: 24px 0 8px; border-bottom: 2px solid #eee; padding-bottom: 6px; }
  table { width: 100%; border-collapse: collapse; margin-bottom: 12px; font-size: 12px; }
  th, td { border: 1px solid #ddd; padding: 6px 10px; ${isRtl ? "text-align: right;" : "text-align: left;"} }
  th { background: #f5f5f5; font-weight: 600; }
  tr:nth-child(even) td { background: #fafafa; }
  .total td { font-weight: 700; background: #fffbe6 !important; }
  @media print { body { margin: 20px; } }
</style>
</head>
<body>
  <h1>${esc(title)}</h1>
  <div class="meta">${esc(meta || "")}</div>
  ${body}
</body>
</html>`);
  win.document.close();
  win.focus();
  setTimeout(() => win.print(), 300);
}
