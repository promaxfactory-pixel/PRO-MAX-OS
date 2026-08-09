import { useState, useEffect, useRef, useMemo } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Search, Command, Home, Users, Package, FileText, Settings, BarChart3, ChevronRight, Zap, Moon, Palette, Keyboard } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useUIStore } from "@/stores/uiStore";
import { cn } from "@/lib/utils";

interface CommandItem {
  id: string;
  label: string;
  description?: string;
  icon?: React.ReactNode;
  shortcut?: string;
  section?: string;
  action: () => void;
  keywords?: string[];
}

const sections = [
  { id: "navigation", label: "التنقل", icon: Home },
  { id: "actions", label: "الإجراءات", icon: Zap },
  { id: "settings", label: "الإعدادات", icon: Settings },
];

const commands: CommandItem[] = [
  { id: "dashboard", label: "لوحة التحكم", description: "العودة للرئيسية", icon: <Home className="w-4 h-4" />, shortcut: "⌘H", section: "navigation", action: () => window.location.href = "/", keywords: ["dashboard", "home", "الرئيسية", "لوحة"] },
  { id: "customers", label: "العملاء", description: "إدارة العملاء", icon: <Users className="w-4 h-4" />, shortcut: "⌘C", section: "navigation", action: () => window.location.href = "/customers", keywords: ["customers", "clients", "العملاء"] },
  { id: "products", label: "المنتجات", description: "إدارة المنتجات", icon: <Package className="w-4 h-4" />, shortcut: "⌘P", section: "navigation", action: () => window.location.href = "/products", keywords: ["products", "inventory", "المنتجات", "المخزون"] },
  { id: "invoices", label: "الفواتير", description: "إدارة الفواتير", icon: <FileText className="w-4 h-4" />, shortcut: "⌘I", section: "navigation", action: () => window.location.href = "/invoices", keywords: ["invoices", "billing", "الفواتير"] },
  { id: "reports", label: "التقارير", description: "عرض التقارير", icon: <BarChart3 className="w-4 h-4" />, shortcut: "⌘R", section: "navigation", action: () => window.location.href = "/reports", keywords: ["reports", "analytics", "التقارير"] },
  { id: "settings", label: "الإعدادات", description: "إعدادات النظام", icon: <Settings className="w-4 h-4" />, shortcut: "⌘,", section: "settings", action: () => window.location.href = "/settings", keywords: ["settings", "preferences", "الإعدادات"] },
  { id: "new-customer", label: "عميل جديد", description: "إضافة عميل", icon: <Users className="w-4 h-4" />, shortcut: "⌘N", section: "actions", action: () => window.location.href = "/customers/new", keywords: ["new", "create", "add", "جديد", "إضافة"] },
  { id: "new-invoice", label: "فاتورة جديدة", description: "إنشاء فاتورة", icon: <FileText className="w-4 h-4" />, shortcut: "⌘⇧N", section: "actions", action: () => window.location.href = "/invoices/new", keywords: ["new", "create", "invoice", "فاتورة"] },
  { id: "toggle-theme", label: "تبديل السمة", description: "تبديل الوضع الليلي/النهاري", icon: <Moon className="w-4 h-4" />, shortcut: "⌘⇧T", section: "actions", action: () => { const store = useUIStore.getState(); store.setMode(store.mode === "night" ? "professional" : "night"); }, keywords: ["theme", "dark", "light", "سمة", "ليلي", "نهاري"] },
  { id: "toggle-sidebar", label: "تبديل الشريط الجانبي", description: "طي/توسيع الشريط", icon: <ChevronRight className="w-4 h-4" />, shortcut: "⌘B", section: "actions", action: () => { const store = useUIStore.getState(); store.collapseSidebar(); }, keywords: ["sidebar", "collapse", "شريط", "جانبي"] },
  { id: "command-palette", label: "لوحة الأوامر", description: "فتح لوحة الأوامر", icon: <Command className="w-4 h-4" />, shortcut: "⌘⇧P", section: "actions", action: () => {}, keywords: ["command", "palette", "لوحة", "أوامر"] },
  { id: "shortcuts", label: "اختصارات لوحة المفاتيح", description: "عرض الاختصارات", icon: <Keyboard className="w-4 h-4" />, shortcut: "⌘/", section: "settings", action: () => {}, keywords: ["shortcuts", "keys", "اختصارات"] },
];

export default function CommandPalette({ open, onClose }: { open: boolean; onClose: () => void }) {
  const { t } = useTranslation();
  const [query, setQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);
  const { mode } = useUIStore();

  const filteredCommands = useMemo(() => {
    if (!query.trim()) return commands;
    const q = query.toLowerCase();
    return commands.filter(cmd => 
      cmd.label.toLowerCase().includes(q) ||
      cmd.description?.toLowerCase().includes(q) ||
      cmd.keywords?.some(k => k.toLowerCase().includes(q)) ||
      cmd.shortcut?.toLowerCase().includes(q)
    );
  }, [query]);

  const groupedCommands = useMemo(() => {
    const groups: Record<string, CommandItem[]> = {};
    filteredCommands.forEach(cmd => {
      const section = cmd.section || "actions";
      if (!groups[section]) groups[section] = [];
      groups[section].push(cmd);
    });
    return groups;
  }, [filteredCommands]);

  useEffect(() => {
    if (open) {
      setQuery("");
      setSelectedIndex(0);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [open]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!open) return;
      if (e.key === "ArrowDown") {
        e.preventDefault();
        setSelectedIndex(prev => Math.min(prev + 1, filteredCommands.length - 1));
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        setSelectedIndex(prev => Math.max(prev - 1, 0));
      } else if (e.key === "Enter") {
        e.preventDefault();
        const cmd = filteredCommands[selectedIndex];
        if (cmd) {
          cmd.action();
          onClose();
        }
      } else if (e.key === "Escape") {
        onClose();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [open, filteredCommands, selectedIndex, onClose]);

  useEffect(() => {
    const item = listRef.current?.querySelector(`[data-index="${selectedIndex}"]`);
    item?.scrollIntoView({ block: "nearest" });
  }, [selectedIndex]);

  if (!open) return null;

  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        className="fixed inset-0 z-[100] flex items-start justify-center pt-20"
        onClick={onClose}
      >
        <motion.div
          initial={{ opacity: 0, scale: 0.95, y: -20 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          exit={{ opacity: 0, scale: 0.95, y: -20 }}
          transition={{ type: "spring", damping: 25, stiffness: 300 }}
          className="w-full max-w-2xl mx-4 glass-morphism rounded-2xl overflow-hidden shadow-2xl"
          onClick={e => e.stopPropagation()}
        >
          <div className="relative p-4">
            <div className="flex items-center gap-3">
              <div className="relative flex-1">
                <Search className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-surface-400" aria-hidden="true" />
                <input
                  ref={inputRef}
                  type="text"
                  value={query}
                  onChange={e => { setQuery(e.target.value); setSelectedIndex(0); }}
                  placeholder={t("commandPalette.placeholder") || "ابحث عن أمر أو انتقل إلى..."}
                  className="w-full bg-surface-900/50 border border-surface-700 rounded-xl px-4 py-3 pl-12 text-white placeholder-surface-500 focus:outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-500/20 text-base font-medium"
                  autoComplete="off"
                  spellCheck={false}
                />
                <kbd className="absolute right-4 top-1/2 -translate-y-1/2 px-2 py-1 bg-surface-800 border border-surface-700 rounded-lg text-xs text-surface-400 font-mono">
                  {navigator.platform.includes("Mac") ? "⌘" : "Ctrl"} + K
                </kbd>
              </div>
              <motion.button
                whileHover={{ scale: 1.05 }}
                whileTap={{ scale: 0.95 }}
                onClick={onClose}
                className="p-2 rounded-xl text-surface-400 hover:text-white hover:bg-surface-800 transition-colors"
                aria-label={t("common.close")}
              >
                <Command className="w-5 h-5" />
              </motion.button>
            </div>
          </div>

          <div ref={listRef} className="max-h-[60vh] overflow-y-auto p-4 space-y-4">
            {Object.entries(groupedCommands).map(([sectionId, items]) => {
              const section = sections.find(s => s.id === sectionId);
              return (
                <motion.div
                  key={sectionId}
                  initial={{ opacity: 0, y: 10 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ delay: 0.05 }}
                  className="space-y-2"
                >
                  <div className="flex items-center gap-2 px-2 py-1 text-xs font-semibold text-surface-400 uppercase tracking-wider">
                    {section && <section.icon className="w-3 h-3" />}
                    <span>{t(`commandPalette.sections.${sectionId}`) || section?.label || sectionId}</span>
                    <div className="h-px flex-1 bg-gradient-to-r from-brand-500/30 to-transparent" />
                  </div>
                  <div className="grid gap-1" role="listbox">
                    {items.map((cmd: CommandItem, i: number) => {
                      const isSelected = commands.indexOf(cmd) === selectedIndex;
                      return (
                        <motion.button
                          key={cmd.id}
                          data-index={commands.indexOf(cmd)}
                          initial={{ opacity: 0, x: -10 }}
                          animate={{ opacity: 1, x: 0 }}
                          transition={{ delay: 0.02 * i }}
                          whileHover={{ x: 4 }}
                          whileTap={{ scale: 0.98 }}
                          onClick={() => { cmd.action(); onClose(); }}
                          className={cn(
                            "w-full flex items-center gap-3 p-3 rounded-xl transition-all duration-200",
                            isSelected
                              ? "bg-gradient-to-r from-brand-500/15 to-brand-500/5 border border-brand-500/30 shadow-lg shadow-brand-500/10"
                              : "hover:bg-surface-800/50"
                          )}
                          role="option"
                          aria-selected={isSelected}
                        >
                          <div className={cn(
                            "flex-shrink-0 w-10 h-10 rounded-xl flex items-center justify-center",
                            isSelected ? "bg-brand-500/20 text-brand-400" : "bg-surface-800 text-surface-400"
                          )}>
                            {cmd.icon}
                          </div>
                          <div className="flex-1 min-w-0 text-right">
                            <div className="font-medium text-white truncate">{cmd.label}</div>
                            {cmd.description && <div className="text-xs text-surface-500 truncate">{cmd.description}</div>}
                          </div>
                          {cmd.shortcut && (
                            <kbd className="flex-shrink-0 px-2 py-1 bg-surface-900 border border-surface-700 rounded-lg text-xs text-surface-400 font-mono">
                              {cmd.shortcut}
                            </kbd>
                          )}
                          {isSelected && <ChevronRight className="w-4 h-4 text-brand-400" />}
                        </motion.button>
                      );
                    })}
                  </div>
                </motion.div>
              );
            })}
            {filteredCommands.length === 0 && (
              <motion.div
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                className="py-12 text-center text-surface-500"
              >
                <Search className="w-12 h-12 mx-auto mb-3 text-surface-600" />
                <p className="font-medium text-white">{t("commandPalette.noResults") || "لا توجد نتائج مطابقة"}</p>
                <p className="text-sm mt-1">{t("commandPalette.tryDifferent") || "جرب كلمات مختلفة"}</p>
              </motion.div>
            )}
          </div>

          <div className="border-t border-surface-700/50 px-4 py-3 flex items-center justify-between text-xs text-surface-500">
            <div className="flex items-center gap-2">
              <Keyboard className="w-4 h-4" />
              <span>{navigator.platform.includes("Mac") ? "⌘K" : "Ctrl+K"}</span> فتح البحث
              <span className="mx-2 text-surface-700">|</span>
              <span>⌘⇧P</span> لوحة الأوامر
              <span className="mx-2 text-surface-700">|</span>
              <span>Esc</span> إغلاق
            </div>
            <div className="flex items-center gap-2">
              <Palette className="w-4 h-4" />
              <span>{t("commandPalette.mode") || "السمة"}: {t(`modes.${mode}`) || mode}</span>
            </div>
          </div>
        </motion.div>
      </motion.div>
    </AnimatePresence>
  );
}