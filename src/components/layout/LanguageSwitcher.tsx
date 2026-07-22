import { useTranslation } from "react-i18next";
import { Languages, Check } from "lucide-react";
import { useState, useRef, useEffect } from "react";

const languages = [
  { code: "ar", label: "العربية", dir: "rtl" },
  { code: "en", label: "English", dir: "ltr" },
  { code: "hi", label: "हिन्दी", dir: "ltr" },
  { code: "ur", label: "اردو", dir: "rtl" },
];

export default function LanguageSwitcher() {
  const { i18n } = useTranslation();
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handler = (e: MouseEvent) => { if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false); };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, []);

  const current = languages.find((l) => l.code === i18n.language) || languages[0];

  const switchLang = (code: string) => {
    i18n.changeLanguage(code);
    document.documentElement.dir = languages.find((l) => l.code === code)?.dir || "rtl";
    setOpen(false);
  };

  return (
    <div ref={ref} className="relative">
      <button
        onClick={() => setOpen(!open)}
        className="flex items-center gap-1.5 px-2 py-1.5 rounded-lg text-xs text-gray-400 hover:text-white hover:bg-zinc-800/50 transition-all"
      >
        <Languages className="w-3.5 h-3.5" />
        <span>{current.label}</span>
      </button>
      {open && (
        <div className="absolute bottom-full right-0 mb-1 w-32 bg-zinc-900 border border-zinc-800 rounded-xl shadow-2xl overflow-hidden">
          {languages.map((lang) => (
            <button
              key={lang.code}
              onClick={() => switchLang(lang.code)}
              className={`w-full flex items-center gap-2 px-3 py-2 text-xs transition-colors ${
                i18n.language === lang.code
                  ? "bg-gold-500/10 text-gold-400"
                  : "text-gray-400 hover:text-white hover:bg-zinc-800"
              }`}
            >
              <span className="flex-1 text-right">{lang.label}</span>
              {i18n.language === lang.code && <Check className="w-3 h-3 text-gold-400" />}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
