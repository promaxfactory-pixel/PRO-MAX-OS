import { Search, X } from "lucide-react";
import { cn } from "@/lib/utils";
import { useTranslation } from "react-i18next";

interface SearchBarProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  className?: string;
  id?: string;
}

export default function SearchBar({
  value,
  onChange,
  placeholder,
  className,
  id,
}: SearchBarProps) {
  const { t, i18n } = useTranslation();
  const isRtl = i18n.language === "ar" || i18n.language === "ur";
  const defaultPlaceholder = t("common.search");
  const inputId = id || `search-${(placeholder || defaultPlaceholder).replace(/\s+/g, '-').toLowerCase()}`;

  return (
    <div className={cn("relative", className)}>
      <Search className={`absolute ${isRtl ? 'left-3' : 'right-3'} top-1/2 -translate-y-1/2 h-4 w-4 text-surface-400 pointer-events-none`} aria-hidden="true" />
      <input
        id={inputId}
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder || defaultPlaceholder}
        aria-label={placeholder || defaultPlaceholder}
        className={`input-field w-full ${isRtl ? 'pl-10 pr-3' : 'pr-10 pl-3'}`}
      />
      {value && (
        <button
          onClick={() => onChange("")}
          aria-label={t("common.clear")}
          className={`absolute top-1/2 -translate-y-1/2 text-surface-400 hover:text-white transition-colors ${isRtl ? 'right-3' : 'left-3'}`}
        >
          <X className="h-4 w-4" aria-hidden="true" />
        </button>
      )}
    </div>
  );
}