import { memo, useState, useRef, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { useAuthStore } from "@/stores/authStore";
import { Search, Bell, LogOut, Settings, Moon, Sun } from "lucide-react";
import ModeSelector from "@/components/ui/ModeSelector";
import { useUIStore } from "@/stores/uiStore";

const Topbar = memo(function Topbar() {
  const [searchOpen, setSearchOpen] = useState(false);
  const [userMenuOpen, setUserMenuOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const searchRef = useRef<HTMLInputElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const { user, logout } = useAuthStore();
  const navigate = useNavigate();

  const toggleTheme = () => {
    const current = document.documentElement.getAttribute('data-theme');
    const next = current === 'light' ? 'dark' : 'light';
    document.documentElement.setAttribute('data-theme', next);
    localStorage.setItem('promax-theme', next);
  };
  const isDark = document.documentElement.getAttribute('data-theme') !== 'light';

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
    <header
      className="h-16 flex items-center justify-between px-6 sticky top-0 z-30 border-b"
      style={{
        background: 'color-mix(in srgb, var(--surface-sidebar) 80%, transparent)',
        backdropFilter: 'blur(16px)',
        borderColor: 'var(--border)',
      }}
      role="banner"
    >
      <div className="flex items-center gap-3">
        {searchOpen ? (
          <div className="relative">
            <Search className="absolute right-3 top-1/2 -translate-y-1/2 w-4 h-4" style={{ color: 'var(--text-muted)' }} aria-hidden="true" />
            <input
              ref={searchRef}
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="ط¨ط­ط«... Ctrl+K"
              aria-label="ط¨ط­ط« ط³ط±ظٹط¹"
              className="w-80 rounded-xl pr-10 pl-4 py-2 text-sm"
              style={{
                background: 'color-mix(in srgb, var(--surface-card) 70%, var(--surface-bg))',
                border: '1px solid var(--border)',
                color: 'var(--text-primary)',
              }}
              onBlur={() => { if (!searchQuery) setSearchOpen(false); }}
            />
          </div>
        ) : (
          <button onClick={() => setSearchOpen(true)} className="btn-ghost flex items-center gap-2 text-sm" aria-label="ظپطھط­ ط§ظ„ط¨ط­ط« ط§ظ„ط³ط±ظٹط¹">
            <Search className="w-4 h-4" aria-hidden="true" />
            <span>ط¨ط­ط«</span>
            <kbd
              className="text-[10px] rounded px-1.5 py-0.5 font-mono"
              style={{
                background: 'color-mix(in srgb, var(--surface-card) 70%, var(--surface-bg))',
                border: '1px solid var(--border)',
                color: 'var(--text-muted)',
              }}
              aria-hidden="true"
            >
              Ctrl+K
            </kbd>
          </button>
        )}
      </div>

      <div className="flex items-center gap-1">
        <ModeSelector />

        <button
          onClick={toggleTheme}
          className="btn-ghost p-2"
          aria-label={isDark ? "ط§ظ„ظˆط¶ط¹ ط§ظ„ظ†ظ‡ط§ط±ظٹ" : "ط§ظ„ظˆط¶ط¹ ط§ظ„ظ„ظٹظ„ظٹ"}
        >
          {isDark ? <Sun className="w-4 h-4" /> : <Moon className="w-4 h-4" />}
        </button>

        <button className="btn-ghost relative p-2" aria-label="ط§ظ„ط¥ط´ط¹ط§ط±ط§طھ">
          <Bell className="w-5 h-5" aria-hidden="true" />
          <span
            className="absolute top-1 right-1 w-2 h-2 rounded-full"
            style={{ background: 'var(--brand-gold)' }}
            aria-hidden="true"
          />
        </button>

        <div className="relative" ref={menuRef}>
          <button
            onClick={() => setUserMenuOpen(!userMenuOpen)}
            aria-haspopup="true"
            aria-expanded={userMenuOpen}
            aria-label="ظ‚ط§ط¦ظ…ط© ط§ظ„ظ…ط³طھط®ط¯ظ…"
            className="flex items-center gap-2 px-3 py-1.5 rounded-xl transition-colors"
            onMouseEnter={e => e.currentTarget.style.background = 'color-mix(in srgb, var(--border) 40%, transparent)'}
            onMouseLeave={e => e.currentTarget.style.background = 'transparent'}
          >
            <div
              className="w-8 h-8 rounded-full flex items-center justify-center"
              style={{ background: 'linear-gradient(135deg, var(--brand-500), var(--brand-700))' }}
            >
              <span className="text-xs font-bold" style={{ color: 'var(--brand-gold)' }}>{user?.full_name?.[0] || 'A'}</span>
            </div>
            <div className="text-right">
              <p className="text-sm font-medium" style={{ color: 'var(--text-primary)' }}>{user?.full_name || 'Admin'}</p>
              <p className="text-[10px]" style={{ color: 'var(--text-muted)' }}>{user?.role || 'admin'}</p>
            </div>
          </button>

          {userMenuOpen && (
            <div
              role="menu"
              aria-label="ط®ظٹط§ط±ط§طھ ط§ظ„ظ…ط³طھط®ط¯ظ…"
              className="absolute right-0 top-full mt-2 w-48 rounded-xl overflow-hidden animate-scale-in"
              style={{
                background: 'var(--surface-card)',
                border: '1px solid var(--border)',
                boxShadow: 'var(--shadow-modal)',
              }}
            >
              <button
                role="menuitem"
                onClick={() => { navigate('/settings'); setUserMenuOpen(false); }}
                className="w-full flex items-center gap-2 px-4 py-2.5 text-sm transition-colors"
                style={{ color: 'var(--text-secondary)' }}
                onMouseEnter={e => e.currentTarget.style.background = 'color-mix(in srgb, var(--border) 40%, transparent)'}
                onMouseLeave={e => e.currentTarget.style.background = 'transparent'}
              >
                <Settings className="w-4 h-4" /> ط§ظ„ط¥ط¹ط¯ط§ط¯ط§طھ
              </button>
              <hr style={{ borderColor: 'var(--border)' }} />
              <button
                role="menuitem"
                onClick={handleLogout}
                className="w-full flex items-center gap-2 px-4 py-2.5 text-sm transition-colors"
                style={{ color: 'var(--danger)' }}
                onMouseEnter={e => e.currentTarget.style.background = 'color-mix(in srgb, var(--danger) 10%, transparent)'}
                onMouseLeave={e => e.currentTarget.style.background = 'transparent'}
              >
                <LogOut className="w-4 h-4" /> طھط³ط¬ظٹظ„ ط§ظ„ط®ط±ظˆط¬
              </button>
            </div>
          )}
        </div>
      </div>
    </header>
  );
});

export default Topbar;

