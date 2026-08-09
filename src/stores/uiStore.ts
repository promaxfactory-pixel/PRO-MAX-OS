import { create } from 'zustand';

export type WorkMode = 'default' | 'power' | 'stability' | 'focus' | 'creative' | 'night' | 'professional';
export type Density = 'comfortable' | 'compact';
export type Motion = 'full' | 'reduced';

const MODE_LIST: WorkMode[] = ['default', 'power', 'stability', 'focus', 'creative', 'night', 'professional'];

function getInitialMode(): WorkMode {
  if (typeof window !== 'undefined') {
    const saved = localStorage.getItem('promax-work-mode') as WorkMode | null;
    if (saved && (MODE_LIST as string[]).includes(saved)) return saved;
  }
  return 'professional';
}

function applyDensity(density: Density) {
  document.documentElement.style.setProperty('--user-density', density === 'compact' ? '0.88' : '1');
}

function applyMotion(motion: Motion) {
  if (motion === 'reduced') document.documentElement.setAttribute('data-motion', 'reduced');
  else document.documentElement.removeAttribute('data-motion');
}

export interface Notification {
  id?: string;
  type: 'success' | 'error' | 'warning' | 'info' | 'loading';
  title: string;
  message: string;
  duration?: number;
  action?: { label: string; onClick: () => void };
}

interface UIState {
  sidebarOpen: boolean;
  sidebarCollapsed: boolean;
  currentModule: string;
  searchQuery: string;
  notifications: Notification[];
  workMode: WorkMode;
  mode: WorkMode;
  showOnboarding: boolean;
  density: Density;
  motion: Motion;
  searchOpen: boolean;
  setShowOnboarding: (show: boolean) => void;
  setWorkMode: (mode: WorkMode) => void;
  setMode: (mode: WorkMode) => void;
  setDensity: (density: Density) => void;
  setMotion: (motion: Motion) => void;
  setSearchOpen: (open: boolean) => void;
  toggleSidebar: () => void;
  collapseSidebar: () => void;
  setSidebarCollapsed: (collapsed: boolean) => void;
  setCurrentModule: (module: string) => void;
  setSearchQuery: (query: string) => void;
  addNotification: (n: Notification) => void;
  removeNotification: (id: string) => void;
}

export const useUIStore = create<UIState>((set) => ({
  sidebarOpen: true,
  sidebarCollapsed: false,
  currentModule: 'dashboard',
  searchQuery: '',
  notifications: [],
  workMode: getInitialMode(),
  mode: getInitialMode(),
  showOnboarding: true,
  density: (() => {
    if (typeof window !== 'undefined') {
      const saved = localStorage.getItem('promax-density') as Density | null;
      if (saved === 'comfortable' || saved === 'compact') return saved;
    }
    return 'comfortable';
  })(),
  motion: (() => {
    if (typeof window !== 'undefined') {
      const saved = localStorage.getItem('promax-motion') as Motion | null;
      if (saved === 'full' || saved === 'reduced') return saved;
    }
    return 'full';
  })(),
  searchOpen: false,
  setWorkMode: (mode) => {
    localStorage.setItem('promax-work-mode', mode);
    document.documentElement.setAttribute('data-mode', mode);
    set({ workMode: mode, mode });
  },
  setMode: (mode) => {
    localStorage.setItem('promax-work-mode', mode);
    document.documentElement.setAttribute('data-mode', mode);
    set({ workMode: mode, mode });
  },
  setDensity: (density) => {
    localStorage.setItem('promax-density', density);
    applyDensity(density);
    set({ density });
  },
  setMotion: (motion) => {
    localStorage.setItem('promax-motion', motion);
    applyMotion(motion);
    set({ motion });
  },
  setShowOnboarding: (show) => set({ showOnboarding: show }),
  setSearchOpen: (open) => set({ searchOpen: open }),
  toggleSidebar: () => set((s) => ({ sidebarOpen: !s.sidebarOpen })),
  collapseSidebar: () => set((s) => ({ sidebarCollapsed: !s.sidebarCollapsed })),
  setSidebarCollapsed: (collapsed: boolean) => set({ sidebarCollapsed: collapsed }),
  setCurrentModule: (module) => set({ currentModule: module }),
  setSearchQuery: (query) => set({ searchQuery: query }),
  addNotification: (n) => set((s) => ({ notifications: [...s.notifications, { ...n, id: n.id || crypto.randomUUID() }] })),
  removeNotification: (id) => set((s) => ({ notifications: s.notifications.filter((n) => n.id !== id) })),
}));

if (typeof window !== 'undefined') {
  const state = useUIStore.getState();
  applyDensity(state.density);
  applyMotion(state.motion);
}
