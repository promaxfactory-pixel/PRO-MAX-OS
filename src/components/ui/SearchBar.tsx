import { Search, X } from "lucide-react";
import { cn } from "@/lib/utils";

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
  placeholder = "بحث...",
  className,
  id,
}: SearchBarProps) {
  const inputId = id || `search-${placeholder.replace(/\s+/g, '-').toLowerCase()}`;

  return (
    <div className={cn("relative", className)}>
      <Search className="absolute right-3 top-1/2 -translate-y-1/2 h-4 w-4 text-surface-400 pointer-events-none" aria-hidden="true" />
      <input
        id={inputId}
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        aria-label={placeholder}
        className="input-field w-full pr-10 pl-9"
      />
      {value && (
        <button
          onClick={() => onChange("")}
          aria-label="مسح البحث"
          className="absolute left-3 top-1/2 -translate-y-1/2 text-surface-400 hover:text-white transition-colors"
        >
          <X className="h-4 w-4" aria-hidden="true" />
        </button>
      )}
    </div>
  );
}
