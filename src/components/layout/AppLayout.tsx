import { useState, useEffect, useMemo } from "react";
import { useLocation } from "react-router-dom";
import { AnimatePresence, motion } from "framer-motion";
import Sidebar from "./Sidebar";
import Topbar from "./Topbar";
import Toast from "@/components/ui/Toast";
import GlobalSearch from "@/components/ui/GlobalSearch";
import CommandPalette from "@/components/ui/CommandPalette";
import { useUIStore } from "@/stores/uiStore";

const pageVariants = {
  initial: { opacity: 0, y: 20, scale: 0.98, filter: "blur(4px)" },
  animate: {
    opacity: 1,
    y: 0,
    scale: 1,
    filter: "blur(0px)",
    transition: { duration: 0.5, ease: [0.16, 1, 0.3, 1] },
  },
  exit: {
    opacity: 0,
    y: -20,
    scale: 1.02,
    filter: "blur(4px)",
    transition: { duration: 0.3, ease: [0.25, 0.1, 0.25, 1] },
  },
};

export default function AppLayout({ children }: { children: React.ReactNode }) {
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const [commandPaletteOpen, setCommandPaletteOpen] = useState(false);
  const location = useLocation();
  const { mode, searchOpen, setSearchOpen } = useUIStore();

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.key === 'b') {
        e.preventDefault();
        setSidebarCollapsed(prev => !prev);
      }
      if (e.ctrlKey && e.key === 'k') {
        e.preventDefault();
        setSearchOpen(true);
      }
      if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key === 'P') {
        e.preventDefault();
        setCommandPaletteOpen(true);
      }
      if (e.key === 'Escape') {
        setSearchOpen(false);
        setCommandPaletteOpen(false);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [setSearchOpen]);

  const animatedBackground = useMemo(
    () => mode === 'night' || mode === 'creative' || mode === 'power',
    [mode]
  );

  return (
    <div className="flex h-screen overflow-hidden relative">
      <div className={animatedBackground ? "bg-mesh animate-mesh" : "bg-mesh"} aria-hidden="true" />
      <div className="bg-particles" aria-hidden="true" />
      <div className="bg-noise" aria-hidden="true" />

      <Toast />
      <GlobalSearch open={searchOpen} onClose={() => setSearchOpen(false)} />
      <CommandPalette open={commandPaletteOpen} onClose={() => setCommandPaletteOpen(false)} />

      <Sidebar collapsed={sidebarCollapsed} onToggle={() => setSidebarCollapsed(!sidebarCollapsed)} currentPath={location.pathname} />

      <div
        className="flex-1 flex flex-col overflow-hidden relative z-10"
        style={{
          marginRight: sidebarCollapsed ? '72px' : 'var(--sidebar-width, 260px)',
          transition: 'margin 0.4s cubic-bezier(0.16, 1, 0.3, 1)',
        }}
      >
        <Topbar />
        <main className="flex-1 overflow-y-auto p-6 pb-20">
          <div className="max-w-[1600px] mx-auto">
            <AnimatePresence mode="wait">
              <motion.div
                key={location.pathname}
                variants={pageVariants}
                initial="initial"
                animate="animate"
                exit="exit"
                custom={location.pathname}
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
