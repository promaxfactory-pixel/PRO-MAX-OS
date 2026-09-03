const rolePathPrefixes: Record<string, string[]> = {
  accountant: [
    "/dashboard",
    "/alerts",
    "/invoices",
    "/credit-notes",
    "/customers",
    "/suppliers",
    "/purchases",
    "/accounting",
    "/audit-log",
    "/expenses",
    "/cashbank",
    "/petty-cash",
    "/custody",
    "/cheques",
    "/employee-advances",
    "/factory",
    "/imports",
    "/barter",
    "/installments",
    "/reports",
    "/tools/ocr",
    "/tools/ai-file-import",
    "/tools/historical-import",
    "/tools/excel-import",
    "/tools/einvoice",
    "/tools/qayd",
    "/tools/backup",
  ],
  hr: ["/dashboard", "/alerts", "/hr", "/payroll", "/overtime", "/employee-advances", "/renewals"],
  operator: [
    "/dashboard",
    "/alerts",
    "/products",
    "/inventory",
    "/stock-transfers",
    "/bom",
    "/live-production",
    "/production",
    "/operations",
    "/maintenance",
    "/machines",
    "/quality",
    "/factory",
  ],
  viewer: ["/dashboard", "/alerts", "/reports"],
  user: ["/dashboard", "/alerts"],
};

export function canAccessPath(role: string | undefined, path: string) {
  if (path === "/" || path === "/403" || path === "/settings/change-password") return true;
  if (role === "admin" || role === "manager") return true;
  const prefixes = rolePathPrefixes[role ?? "user"] ?? rolePathPrefixes.user;
  return prefixes.some((prefix) => path === prefix || path.startsWith(`${prefix}/`));
}
