import { invoke as coreInvoke } from "@tauri-apps/api/core";
import { useAuthStore } from "@/stores/authStore";

export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const user = useAuthStore.getState().user;
  if (args === undefined) {
    return coreInvoke<T>(cmd, user?.id != null ? { userId: user.id } : undefined);
  }
  return coreInvoke<T>(cmd, user?.id != null ? { ...args, userId: user.id } : args);
}
