import { useState, useEffect } from "react";
import { useLocation } from "react-router-dom";
import { AnimatePresence, motion } from "framer-motion";
import { useTranslation } from "react-i18next";
import Sidebar from "./Sidebar";
import Topbar from "./Topbar";
import Toast from "@/components/ui/Toast";
import GlobalSearch from "@/components/ui/GlobalSearch";

export default function AppLayout({ children }: { children: React.ReactNode }) {
  const [sidebarCollapsed, setSidebarCollapsed] = useState(() => {
    const saved = localStorage.getItem("promax-sidebar-collapsed");
    return saved === "true";
  });
  const [searchOpen, setSearchOpen] = useState(false);
  const location = useLocation();
  const { i18n } = useTranslation();
  const isRtl = i18n.language === "ar" || i18n.language === "ur";

  const sidebarWidth = sidebarCollapsed ? "var(--sidebar-collapsed-width)" : "var(--sidebar-width)";
  const marginProp = isRtl ? { marginRight: sidebarWidth } : { marginLeft: sidebarWidth };
  const marginTransition = "margin-right 0.3s cubic-bezier(0.4, 0, 0.2, 1), margin-left 0.3s cubic-bezier(0.4, 0, 0.2, 1)";

  useEffect(() => {
    localStorage.setItem("promax-sidebar-collapsed", String(sidebarCollapsed));
  }, [sidebarCollapsed]);

  useEffect(() => {
    const savedTheme = localStorage.getItem("promax-theme");
    if (savedTheme) {
      document.documentElement.setAttribute("data-theme", savedTheme);
    }
  }, []);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.key === "b") {
        e.preventDefault();
        setSidebarCollapsed(prev => !prev);
      }
      if (e.ctrlKey && e.key === "k") {
        e.preventDefault();
        setSearchOpen(prev => !prev);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  return (
    <div className="flex h-screen overflow-hidden" style={{ background: "var(--surface-bg)" }}>
      <Toast isRtl={isRtl} />
      <GlobalSearch open={searchOpen} onClose={() => setSearchOpen(false)} />
      <Sidebar collapsed={sidebarCollapsed} onToggle={() => setSidebarCollapsed(!sidebarCollapsed)} currentPath={location.pathname} />
      <div className="flex-1 flex flex-col overflow-hidden" style={{ ...marginProp, transition: marginTransition }}>
        <Topbar />
        <main className="flex-1 overflow-y-auto p-6 pb-20">
          <div className="max-w-[1600px] mx-auto">
            <AnimatePresence mode="wait">
              <motion.div
                key={location.pathname}
                initial={{ opacity: 0, y: 8 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -8 }}
                transition={{ duration: 0.25, ease: [0.25, 0.1, 0.25, 1] }}
              >
                {children}
              </motion.div>
            </AnimatePresence>
          </div>
        </main>
      </div>
    </div>
  );
}
