import { describe, it, expect, vi, beforeEach } from "vitest";
import { useAuthStore } from "./authStore";
import { invoke } from "@tauri-apps/api/core";

const mockUser = {
  id: 1,
  username: "admin",
  full_name: "Administrator",
  role: "admin",
  active: 1,
  must_change_password: 0,
  created_at: "2026-08-10 10:00:00",
};

const localStorageMock = () => {
  const store: Record<string, string> = {};
  return {
    getItem: (k: string) => store[k] ?? null,
    setItem: (k: string, v: string) => {
      store[k] = v;
    },
    removeItem: (k: string) => {
      delete store[k];
    },
    clear: () => {
      Object.keys(store).forEach((k) => delete store[k]);
    },
  };
};

describe("useAuthStore", () => {
  beforeEach(() => {
    const mock = localStorageMock();
    Object.defineProperty(window, "localStorage", {
      value: mock,
      writable: true,
    });
    vi.mocked(invoke).mockReset();
    useAuthStore.setState({
      user: null,
      isAuthenticated: false,
      isLoading: false,
      error: null,
    });
  });

  it("starts unauthenticated with no token", () => {
    const state = useAuthStore.getState();
    expect(state.isAuthenticated).toBe(false);
    expect(state.user).toBeNull();
  });

  it("logs in successfully and stores token + user", async () => {
    vi.mocked(invoke).mockResolvedValue({
      user: mockUser,
      token: "test-jwt-token",
    });

    const result = await useAuthStore.getState().login("admin", "Admin@2026");

    expect(result?.user.username).toBe("admin");    expect(invoke).toHaveBeenCalledWith("login", {
      username: "admin",
      password: "Admin@2026",
    });
    expect(window.localStorage.getItem("auth_token")).toBe("test-jwt-token");
    expect(window.localStorage.getItem("auth_user")).toContain("admin");
    expect(useAuthStore.getState().isAuthenticated).toBe(true);
    expect(useAuthStore.getState().user).toEqual(mockUser);
    expect(useAuthStore.getState().isLoading).toBe(false);
  });

  it("sets error and rethrows on failed login", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("Invalid credentials"));

    await expect(
      useAuthStore.getState().login("admin", "wrong"),
    ).rejects.toThrow("Invalid credentials");

    expect(useAuthStore.getState().error).toBe("Invalid credentials");
    expect(useAuthStore.getState().isAuthenticated).toBe(false);
    expect(useAuthStore.getState().isLoading).toBe(false);
  });

  it("logs out and clears persisted state", () => {
    window.localStorage.setItem("auth_token", "token");
    window.localStorage.setItem("auth_user", JSON.stringify(mockUser));
    useAuthStore.setState({ user: mockUser, isAuthenticated: true });

    useAuthStore.getState().logout();

    expect(useAuthStore.getState().user).toBeNull();
    expect(useAuthStore.getState().isAuthenticated).toBe(false);
    expect(window.localStorage.getItem("auth_token")).toBeNull();
    expect(window.localStorage.getItem("auth_user")).toBeNull();
  });
});
