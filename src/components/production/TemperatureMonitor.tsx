import { useState, useEffect, useCallback } from "react";
import { invoke } from "@/lib/tauri";
import { motion, AnimatePresence } from "framer-motion";
import { Thermometer, AlertTriangle, Activity } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

interface LiveMachineTemp {
  machine_id: number;
  machine_name: string;
  temperature: number;
  ts: string;
  status: string;
}

function tempColor(temp: number): string {
  if (temp <= 0) return "text-surface-500";
  if (temp > 180) return "text-red-400";
  if (temp > 150) return "text-amber-400";
  if (temp > 100) return "text-yellow-400";
  return "text-emerald-400";
}

function tempBg(temp: number): string {
  if (temp <= 0) return "bg-surface-800/50";
  if (temp > 180) return "bg-red-500/10 border-red-500/20";
  if (temp > 150) return "bg-amber-500/10 border-amber-500/20";
  if (temp > 100) return "bg-yellow-500/10 border-yellow-500/20";
  return "bg-emerald-500/10 border-emerald-500/20";
}

export default function TemperatureMonitor() {
  const [temps, setTemps] = useState<LiveMachineTemp[]>([]);
  const { addNotification } = useUIStore();

  const fetchTemps = useCallback(async () => {
    try {
      const data = await invoke<LiveMachineTemp[]>("get_live_machine_temps", {});
      setTemps(data);
      for (const m of data) {
        if (m.status === "critical" && m.temperature > 0) {
          addNotification({ id: crypto.randomUUID(), type: "warning", title: "حرارة عالية", message: `${m.machine_name}: ${m.temperature}°C` });
        }
      }
    } catch (e) {
      addNotification({ type: "error", title: "خطأ", message: "فشل جلب حرارة الماكينات" });
    }
  }, [addNotification]);

  useEffect(() => {
    fetchTemps();
    const interval = setInterval(fetchTemps, 10000);
    return () => clearInterval(interval);
  }, [fetchTemps]);

  const activeTemps = temps.filter((t) => t.temperature > 0);
  if (activeTemps.length === 0) return null;

  return (
    <div className="card">
      <div className="flex items-center justify-between mb-4">
        <h3 className="section-title">
          <Thermometer className="w-4 h-4" />
          حرارة الماكينات
        </h3>
        <Activity className="w-4 h-4 text-surface-400" />
      </div>
      <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-3">
        <AnimatePresence>
          {activeTemps.map((m) => (
            <motion.div
              key={m.machine_id}
              initial={{ opacity: 0, scale: 0.9 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0.9 }}
              className={`rounded-xl border p-3 text-center transition-colors ${tempBg(m.temperature)}`}
            >
              <p className="text-xs text-surface-400 truncate mb-1">{m.machine_name}</p>
              <p className={`text-2xl font-bold ${tempColor(m.temperature)}`}>
                {m.temperature > 0 ? `${m.temperature.toFixed(0)}°` : "—"}
              </p>
              {m.status === "critical" && <AlertTriangle className="w-3 h-3 text-red-400 mx-auto mt-1 animate-pulse" />}
            </motion.div>
          ))}
        </AnimatePresence>
      </div>
    </div>
  );
}
