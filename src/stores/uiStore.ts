import { create } from 'zustand';

export type WorkMode = 'power' | 'stability' | 'focus' | 'creative' | 'night' | 'professional';

interface UIState {
  sidebarOpen: boolean;
  sidebarCollapsed: boolean;
  currentModule: string;
  searchQuery: string;
  notifications: Notification[];
  workMode: WorkMode;
  setWorkMode: (mode: WorkMode) => void;
  toggleSidebar: () => void;
  collapseSidebar: () => void;
  setCurrentModule: (module: string) => void;
  setSearchQuery: (query: string) => void;
  addNotification: (n: Notification) => void;
  removeNotification: (id: string) => void;
}

export interface Notification {
  id: string;
  type: 'success' | 'error' | 'warning' | 'info';
  title: string;
  message: string;
  duration?: number;
}

const getInitialMode = (): WorkMode => {
  if (typeof window !== 'undefined') {
    const saved = localStorage.getItem('promax-work-mode') as WorkMode | null;
    if (saved && ['power', 'stability', 'focus', 'creative', 'night', 'professional'].includes(saved)) {
      return saved;
    }
  }
  return 'professional';
};

export const useUIStore = create<UIState>((set) => ({
  sidebarOpen: true,
  sidebarCollapsed: false,
  currentModule: 'dashboard',
  searchQuery: '',
  notifications: [],
  workMode: getInitialMode(),
  setWorkMode: (mode) => {
    localStorage.setItem('promax-work-mode', mode);
    document.documentElement.setAttribute('data-mode', mode);
    set({ workMode: mode });
  },
  toggleSidebar: () => set((s) => ({ sidebarOpen: !s.sidebarOpen })),
  collapseSidebar: () => set((s) => ({ sidebarCollapsed: !s.sidebarCollapsed })),
  setCurrentModule: (module) => set({ currentModule: module }),
  setSearchQuery: (query) => set({ searchQuery: query }),
  addNotification: (n) => set((s) => ({ notifications: [...s.notifications, n] })),
  removeNotification: (id) => set((s) => ({ notifications: s.notifications.filter((n) => n.id !== id) })),
}));
