import { memo, useState, useRef, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { useAuthStore } from "@/stores/authStore";
import { Search, Bell, LogOut, Settings } from "lucide-react";
import ModeSelector from "@/components/ui/ModeSelector";

const Topbar = memo(function Topbar() {
  const [searchOpen, setSearchOpen] = useState(false);
  const [userMenuOpen, setUserMenuOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const searchRef = useRef<HTMLInputElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const { user, logout } = useAuthStore();
  const navigate = useNavigate();

  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
        e.preventDefault();
        setSearchOpen(true);
        setTimeout(() => searchRef.current?.focus(), 100);
      }
      if (e.key === 'Escape') {
        setSearchOpen(false);
        setSearchQuery('');
      }
    };
    window.addEventListener('keydown', handleKey);
    return () => window.removeEventListener('keydown', handleKey);
  }, []);

  useEffect(() => {
    const handleClick = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setUserMenuOpen(false);
      }
    };
    document.addEventListener('mousedown', handleClick);
    return () => document.removeEventListener('mousedown', handleClick);
  }, []);

  const handleLogout = () => {
    logout();
    navigate('/login');
  };

  return (
    <header className="h-16 bg-surface-900/60 backdrop-blur-xl border-b border-surface-700/50 flex items-center justify-between px-6 sticky top-0 z-30" role="banner">
      <div className="flex items-center gap-3">
        {searchOpen ? (
          <div className="relative">
            <Search className="absolute right-3 top-1/2 -translate-y-1/2 w-4 h-4 text-surface-400" aria-hidden="true" />
            <input
              ref={searchRef}
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="بحث... Ctrl+K"
              aria-label="بحث سريع"
              className="w-80 bg-surface-800 border border-surface-600 text-white rounded-xl pr-10 pl-4 py-2 text-sm focus:ring-2 focus:ring-brand-500/50 focus:border-brand-500"
              onBlur={() => { if (!searchQuery) setSearchOpen(false); }}
            />
          </div>
        ) : (
          <button onClick={() => setSearchOpen(true)} className="btn-ghost flex items-center gap-2 text-sm text-surface-400" aria-label="فتح البحث السريع">
            <Search className="w-4 h-4" aria-hidden="true" />
            <span>بحث</span>
            <kbd className="text-[10px] bg-surface-800 border border-surface-600 rounded px-1.5 py-0.5 font-mono" aria-hidden="true">Ctrl+K</kbd>
          </button>
        )}
      </div>

      <div className="flex items-center gap-1">
        <ModeSelector />

        <button className="btn-ghost relative p-2" aria-label="الإشعارات">
          <Bell className="w-5 h-5" aria-hidden="true" />
          <span className="absolute top-1 left-1 w-2 h-2 bg-gold-400 rounded-full" aria-hidden="true"></span>
        </button>

        <div className="relative" ref={menuRef}>
          <button
            onClick={() => setUserMenuOpen(!userMenuOpen)}
            aria-haspopup="true"
            aria-expanded={userMenuOpen}
            aria-label="قائمة المستخدم"
            className="flex items-center gap-2 px-3 py-1.5 rounded-xl hover:bg-surface-800 transition-colors"
          >
            <div className="w-8 h-8 rounded-full bg-gradient-to-br from-brand-700 to-brand-900 flex items-center justify-center" aria-hidden="true">
              <span className="text-xs font-bold text-gold-400">{user?.full_name?.[0] || 'A'}</span>
            </div>
            <div className="text-right">
              <p className="text-sm font-medium text-white">{user?.full_name || 'Admin'}</p>
              <p className="text-[10px] text-surface-400">{user?.role || 'admin'}</p>
            </div>
          </button>

          {userMenuOpen && (
            <div role="menu" aria-label="خيارات المستخدم" className="absolute left-0 top-full mt-2 w-48 bg-surface-800 border border-surface-700 rounded-xl shadow-luxury overflow-hidden animate-scale-in">
              <button role="menuitem" onClick={() => { navigate('/settings'); setUserMenuOpen(false); }} className="w-full flex items-center gap-2 px-4 py-2.5 text-sm text-surface-300 hover:bg-surface-700 hover:text-white transition-colors">
                <Settings className="w-4 h-4" aria-hidden="true" /> الإعدادات
              </button>
              <hr className="border-surface-700" aria-hidden="true" />
              <button role="menuitem" onClick={handleLogout} className="w-full flex items-center gap-2 px-4 py-2.5 text-sm text-red-400 hover:bg-red-500/10 transition-colors">
                <LogOut className="w-4 h-4" aria-hidden="true" /> تسجيل الخروج
              </button>
            </div>
          )}
        </div>
      </div>
    </header>
  );
});

export default Topbar;
