import { create } from "zustand";
import { User } from "@/types";

interface AuthState {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
  login: (username: string, password: string) => Promise<{ user: User; token: string } | null>;
  logout: () => void;
  setUser: (user: User) => void;
  clearError: () => void;
  validateToken: () => Promise<boolean>;
}

function isTokenExpired(): boolean {
  try {
    const token = localStorage.getItem("auth_token");
    if (!token) return true;
    if (token.startsWith("promax_")) {
      const parts = token.split("_");
      const ts = Number(parts[2]);
      if (Number.isNaN(ts)) return true;
      return Date.now() >= (ts + 7 * 86400) * 1000;
    }
    const payload = JSON.parse(atob(token.split(".")[1]));
    return payload.exp ? Date.now() >= payload.exp * 1000 : false;
  } catch {
    localStorage.removeItem("auth_token");
    localStorage.removeItem("auth_user");
    return true;
  }
}

export const useAuthStore = create<AuthState>((set) => {
  const storedToken = localStorage.getItem("auth_token");
  const storedUser = localStorage.getItem("auth_user");
  const initialUser = storedUser ? JSON.parse(storedUser) : null;
  const tokenExpired = storedToken ? isTokenExpired() : true;

  if (tokenExpired && storedToken) {
    localStorage.removeItem("auth_token");
    localStorage.removeItem("auth_user");
  }

  return {
    user: tokenExpired ? null : initialUser,
    isAuthenticated: !!storedToken && !tokenExpired,
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
      localStorage.removeItem("auth_token");
      localStorage.removeItem("auth_user");
      set({ user: null, isAuthenticated: false });
    },
    setUser: (user) => {
      const token = localStorage.getItem("auth_token");
      localStorage.setItem("auth_user", JSON.stringify(user));
      set({ user, isAuthenticated: !!token && !isTokenExpired() });
    },
    clearError: () => set({ error: null }),
    validateToken: async () => {
      try {
        const token = localStorage.getItem("auth_token");
        if (!token) return false;
        if (isTokenExpired()) {
          localStorage.removeItem("auth_token");
          localStorage.removeItem("auth_user");
          set({ user: null, isAuthenticated: false });
          return false;
        }
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("validate_token", { token });
        return true;
      } catch {
        localStorage.removeItem("auth_token");
        localStorage.removeItem("auth_user");
        set({ user: null, isAuthenticated: false });
        return false;
      }
    },
  };
});