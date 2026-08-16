import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke as coreInvoke } from "@tauri-apps/api/core";
import { invoke } from "./tauri";
import { useAuthStore } from "@/stores/authStore";
import type { User } from "@/types";

const mockedCoreInvoke = vi.mocked(coreInvoke);

const sampleUser: User = {
  id: 42,
  username: "admin",
  full_name: "Admin User",
  role: "admin",
  active: 1,
  must_change_password: 0,
  created_at: "2026-01-01",
};

describe("invoke wrapper", () => {
  beforeEach(() => {
    useAuthStore.setState({ user: null });
    mockedCoreInvoke.mockResolvedValue({ ok: true });
  });

  it("injects userId when a user is logged in", async () => {
    useAuthStore.setState({ user: sampleUser });
    await invoke("get_dashboard");
    expect(mockedCoreInvoke).toHaveBeenCalledWith("get_dashboard", { userId: 42 });
  });

  it("merges userId into provided args", async () => {
    useAuthStore.setState({ user: sampleUser });
    await invoke("get_customer", { id: 7 });
    expect(mockedCoreInvoke).toHaveBeenCalledWith("get_customer", { id: 7, userId: 42 });
  });

  it("does not inject userId when logged out", async () => {
    await invoke("get_dashboard", { x: 1 });
    expect(mockedCoreInvoke).toHaveBeenCalledWith("get_dashboard", { x: 1 });
  });

  it("passes undefined args when no args and logged out", async () => {
    await invoke("get_dashboard");
    expect(mockedCoreInvoke).toHaveBeenCalledWith("get_dashboard", undefined);
  });

  it("passes only userId when no args but user is logged in", async () => {
    useAuthStore.setState({ user: sampleUser });
    await invoke("get_dashboard");
    expect(mockedCoreInvoke).toHaveBeenCalledWith("get_dashboard", { userId: 42 });
  });

  it("does not mutate the caller's args object", async () => {
    useAuthStore.setState({ user: sampleUser });
    const args = { id: 5 };
    await invoke("get_customer", args);
    expect(args).toEqual({ id: 5 });
  });

  it("forwards the resolved value and rejects on error", async () => {
    mockedCoreInvoke.mockResolvedValue({ hello: "world" });
    await expect(invoke("ping")).resolves.toEqual({ hello: "world" });

    mockedCoreInvoke.mockRejectedValue(new Error("boom"));
    await expect(invoke("ping")).rejects.toThrow("boom");
  });
});
