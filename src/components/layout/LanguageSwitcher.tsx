import { useTranslation } from "react-i18next";
import { Languages, Check } from "lucide-react";
import { useState, useRef, useEffect } from "react";

const languages = [
  { code: "ar", labelKey: "language.ar", dir: "rtl" },
  { code: "en", labelKey: "language.en", dir: "ltr" },
  { code: "hi", labelKey: "language.hi", dir: "ltr" },
  { code: "ur", labelKey: "language.ur", dir: "rtl" },
];

export default function LanguageSwitcher() {
  const { t, i18n } = useTranslation();
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
        <span>{t(current.labelKey)}</span>
      </button>
      {open && (
        <div className="absolute top-full right-0 mt-1 w-40 bg-zinc-900 border border-zinc-800 rounded-xl shadow-2xl overflow-hidden z-50">
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
              <span className="flex-1 text-right">{t(lang.labelKey)}</span>
              {i18n.language === lang.code && <Check className="w-3 h-3 text-gold-400" />}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
