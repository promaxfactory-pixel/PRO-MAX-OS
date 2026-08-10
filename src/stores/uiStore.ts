import { create } from "zustand";

export type WorkMode = "power" | "stability" | "focus" | "creative" | "night" | "professional";

interface UIState {
  sidebarOpen: boolean;
  notifications: Notification[];
  workMode: WorkMode;
  setWorkMode: (mode: WorkMode) => void;
  toggleSidebar: () => void;
  addNotification: (n: Notification) => void;
  removeNotification: (id: string) => void;
}

export interface Notification {
  id?: string;
  type: "success" | "error" | "warning" | "info";
  title: string;
  message: string;
  duration?: number;
}

const getInitialMode = (): WorkMode => {
  if (typeof window !== "undefined") {
    const saved = localStorage.getItem("promax-work-mode") as WorkMode | null;
    if (saved && ["power", "stability", "focus", "creative", "night", "professional"].includes(saved)) {
      return saved;
    }
  }
  return "professional";
};

export const useUIStore = create<UIState>((set) => ({
  sidebarOpen: true,
  notifications: [],
  workMode: getInitialMode(),
  setWorkMode: (mode) => {
    localStorage.setItem("promax-work-mode", mode);
    document.documentElement.setAttribute("data-mode", mode);
    set({ workMode: mode });
  },
  toggleSidebar: () => set((s) => ({ sidebarOpen: !s.sidebarOpen })),
  addNotification: (n) => set((s) => ({ notifications: [...s.notifications, { ...n, id: n.id || crypto.randomUUID() }] })),
  removeNotification: (id) => set((s) => ({ notifications: s.notifications.filter((n) => n.id !== id) })),
}));

