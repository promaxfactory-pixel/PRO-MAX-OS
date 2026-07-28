import { create } from 'zustand';
import { User } from '@/types';

interface AuthState {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
  login: (username: string, password: string) => Promise<{ user: User; token: string } | null>;
  logout: () => void;
  setUser: (user: User) => void;
  clearError: () => void;
}

export const useAuthStore = create<AuthState>((set) => {
  const storedToken = localStorage.getItem('auth_token');
  const storedUser = localStorage.getItem('auth_user');
  const initialUser = storedUser ? JSON.parse(storedUser) : null;

  return {
    user: initialUser,
    isAuthenticated: !!storedToken,
    isLoading: false,
    error: null,
    login: async (username: string, password: string) => {
      set({ isLoading: true, error: null });
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        const result = await invoke<{ user: User; token: string }>("login", { username, password });
        localStorage.setItem("auth_token", result.token);
        localStorage.setItem("auth_user", JSON.stringify(result.user));
        set({ user: result.user, isAuthenticated: true, isLoading: false });
        return result;
      } catch (err: unknown) {
        set({ error: err instanceof Error ? err.message : String(err), isLoading: false });
        throw err;
      }
    },
    logout: () => {
      localStorage.removeItem('auth_token');
      localStorage.removeItem('auth_user');
      set({ user: null, isAuthenticated: false });
    },
    setUser: (user) => {
      localStorage.setItem('auth_user', JSON.stringify(user));
      set({ user, isAuthenticated: true });
    },
    clearError: () => set({ error: null }),
  };
});
