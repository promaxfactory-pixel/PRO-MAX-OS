import { useState, useEffect, useRef, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { Search, X, User, Package, FileText, Truck, UserCog } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

interface SearchResult {
  id: number;
  title: string;
  subtitle?: string;
  type: "customer" | "product" | "invoice" | "supplier" | "employee";
  route: string;
}

const typeMeta: Record<
  SearchResult["type"],
  { labelKey: string; icon: typeof User; route: (id: number) => string }
> = {
  customer: {
    labelKey: "nav.customers",
    icon: User,
    route: (id) => `/customers/${id}`,
  },
  product: {
    labelKey: "nav.products",
    icon: Package,
    route: (id) => `/products/${id}`,
  },
  invoice: {
    labelKey: "nav.invoices",
    icon: FileText,
    route: (id) => `/invoices/${id}`,
  },
  supplier: {
    labelKey: "nav.suppliers",
    icon: Truck,
    route: (id) => `/suppliers/${id}`,
  },
  employee: {
    labelKey: "nav.employees",
    icon: UserCog,
    route: (id) => `/hr/employees/${id}`,
  },
};

function highlightMatch(text: string, query: string) {
  if (!query) return text;
  const idx = text.toLowerCase().indexOf(query.toLowerCase());
  if (idx === -1) return text;
  return (
    <>
      {text.slice(0, idx)}
      <span className="text-gold-400 font-semibold">{text.slice(idx, idx + query.length)}</span>
      {text.slice(idx + query.length)}
    </>
  );
}

export default function GlobalSearch({ open, onClose }: { open: boolean; onClose: () => void }) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [loading, setLoading] = useState(false);
  const navigate = useNavigate();
  const { t } = useTranslation();
  const { addNotification } = useUIStore();
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (!open) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [open, onClose]);

  useEffect(() => {
    if (open) {
      setQuery("");
      setResults([]);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [open]);

  const performSearch = useCallback(
    async (q: string) => {
      if (!q.trim()) {
        setResults([]);
        return;
      }
      setLoading(true);
      try {
        const [customers, products, invoices, suppliers, employees] =
          await Promise.allSettled([
            invoke<{ id: number; name: string; phone?: string }[]>("list_customers"),
            invoke<{ id: number; name: string; sku?: string }[]>("list_products"),
            invoke<{ id: number; invoice_number: string; customer_name?: string }[]>("list_invoices"),
            invoke<{ id: number; name: string; contact?: string }[]>("list_suppliers"),
            invoke<{ id: number; name: string; role?: string }[]>("list_employees"),
          ]);

        const all: SearchResult[] = [];
        const lowerQ = q.toLowerCase();

        if (customers.status === "fulfilled") {
          for (const c of customers.value) {
            if (c.name.toLowerCase().includes(lowerQ)) {
              all.push({
                id: c.id,
                title: c.name,
                subtitle: c.phone,
                type: "customer",
                route: typeMeta.customer.route(c.id),
              });
            }
          }
        }

        if (products.status === "fulfilled") {
          for (const p of products.value) {
            if (p.name.toLowerCase().includes(lowerQ)) {
              all.push({
                id: p.id,
                title: p.name,
                subtitle: p.sku,
                type: "product",
                route: typeMeta.product.route(p.id),
              });
            }
          }
        }

        if (invoices.status === "fulfilled") {
          for (const i of invoices.value) {
            if (
              i.invoice_number.toLowerCase().includes(lowerQ) ||
              i.customer_name?.toLowerCase().includes(lowerQ)
            ) {
              all.push({
                id: i.id,
                title: i.invoice_number,
                subtitle: i.customer_name,
                type: "invoice",
                route: typeMeta.invoice.route(i.id),
              });
            }
          }
        }

        if (suppliers.status === "fulfilled") {
          for (const s of suppliers.value) {
            if (s.name.toLowerCase().includes(lowerQ)) {
              all.push({
                id: s.id,
                title: s.name,
                subtitle: s.contact,
                type: "supplier",
                route: typeMeta.supplier.route(s.id),
              });
            }
          }
        }

        if (employees.status === "fulfilled") {
          for (const e of employees.value) {
            if (e.name.toLowerCase().includes(lowerQ)) {
              all.push({
                id: e.id,
                title: e.name,
                subtitle: e.role,
                type: "employee",
                route: typeMeta.employee.route(e.id),
              });
            }
          }
        }

        setResults(all);
      } catch (e) {
        setResults([]);
        addNotification({ type: "error", title: t("common.error"), message: t("search.searchFailed") });
      } finally {
        setLoading(false);
      }
    },
    [t],
  );

  useEffect(() => {
    const timer = setTimeout(() => performSearch(query), 300);
    return () => clearTimeout(timer);
  }, [query, performSearch]);

  const groupedResults = results.reduce<Record<string, SearchResult[]>>((acc, r) => {
    (acc[r.type] ??= []).push(r);
    return acc;
  }, {});

  const handleSelect = (route: string) => {
    navigate(route);
    onClose();
  };

  if (!open) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center pt-[15vh] bg-black/50 backdrop-blur-sm"
      onClick={() => onClose()}
    >
      <div
        className="w-full max-w-xl mx-4 bg-surface-800 rounded-2xl border border-surface-700 shadow-2xl overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center gap-3 px-4 py-3 border-b border-surface-700">
          <Search className="h-5 w-5 text-surface-400 shrink-0" />
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder={`${t("common.search")}...`}
            className="flex-1 bg-transparent text-white text-sm placeholder-surface-400 outline-none"
          />
          <kbd className="hidden sm:inline text-[10px] text-surface-400 border border-surface-700 rounded px-1.5 py-0.5">
            ESC
          </kbd>
          {query && (
            <button
              onClick={() => setQuery("")}
              className="text-surface-400 hover:text-white transition-colors"
            >
              <X className="h-4 w-4" />
            </button>
          )}
        </div>

        <div className="max-h-[400px] overflow-y-auto p-2">
          {!query && (
            <div className="flex flex-col items-center justify-center py-12 text-surface-400">
              <Search className="h-10 w-10 mb-3 opacity-30" />
              <p className="text-sm">{t("search.typeToSearch")}</p>
            </div>
          )}

          {query && !loading && results.length === 0 && (
            <div className="py-12 text-center text-surface-400 text-sm">
              {t("search.noResultsFor", { query })}
            </div>
          )}

          {loading && (
            <div className="py-12 text-center text-surface-400 text-sm">{t("search.searching")}</div>
          )}

          {Object.entries(groupedResults).map(([type, items]) => {
            const meta = typeMeta[type as SearchResult["type"]];
            if (!meta) return null;
            const Icon = meta.icon;

            return (
              <div key={type} className="mb-2">
                <div className="flex items-center gap-2 px-3 py-1.5">
                  <Icon className="h-3.5 w-3.5 text-surface-400" />
                  <span className="text-xs font-semibold text-surface-400 uppercase tracking-wider">
                    {t(meta.labelKey)}
                  </span>
                </div>
                {items.map((item) => (
                  <button
                    key={`${item.type}-${item.id}`}
                    onClick={() => handleSelect(item.route)}
                    className="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-right hover:bg-surface-700 transition-colors"
                  >
                    <div className="flex-1 min-w-0">
                      <p className="text-sm text-white truncate">
                        {highlightMatch(item.title, query)}
                      </p>
                      {item.subtitle && (
                        <p className="text-xs text-surface-400 truncate">{item.subtitle}</p>
                      )}
                    </div>
                  </button>
                ))}
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
