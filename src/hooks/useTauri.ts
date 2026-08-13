import { useState, useEffect, useCallback } from "react";

export function useTauriCommand<T, A = void>(command: string) {
  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const execute = useCallback(async (args?: A) => {
    setLoading(true);
    setError(null);
    try {
      const { invoke } = await import("@/lib/tauri");
      const result = await invoke<T>(command, args as Record<string, unknown>);
      setData(result);
      setLoading(false);
      return result;
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err));
      setLoading(false);
      throw err;
    }
  }, [command]);

  return { data, loading, error, execute, setData };
}

export function useQuery<T>(fetcher: () => Promise<T>, deps: any[] = []) {
  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);

  const refresh = useCallback(() => setRefreshKey((k) => k + 1), []);

  useEffect(() => {
    setLoading(true);
    fetcher()
      .then(setData)
      .catch((err) => setError(err.toString()))
      .finally(() => setLoading(false));
  }, [...deps, refreshKey]);

  return { data, loading, error, refresh };
}
