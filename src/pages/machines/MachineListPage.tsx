import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import Badge from "@/components/ui/Badge";
import { invoke } from "@tauri-apps/api/core";
import { Plus, Cog, AlertCircle } from "lucide-react";

export default function MachineListPage() {
  const navigate = useNavigate();
  const [machines, setMachines] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);

  useEffect(() => {
    invoke("list_machines")
      .then((d: any) => setMachines(d))
      .catch(() => setError(true))
      .finally(() => setLoading(false));
  }, []);

  const statusMap: Record<string, { label: string; variant: string }> = {
    active: { label: "نشط", variant: "success" },
    maintenance: { label: "صيانة", variant: "warning" },
    inactive: { label: "غير نشط", variant: "danger" },
  };

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">الآلات</h1>
          <p className="page-subtitle">{machines.length} آلة مسجلة</p>
        </div>
        <Button icon={<Plus className="w-4 h-4" />}>إضافة آلة</Button>
      </div>

      {loading ? (
        <div className="flex items-center justify-center h-64"><div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" /></div>
      ) : error ? (
        <div className="flex flex-col items-center justify-center h-64 text-surface-400">
          <AlertCircle className="w-12 h-12 mb-4 text-surface-500" />
          <p className="text-lg font-medium">إدارة الآلات قيد التطوير</p>
          <p className="text-sm text-surface-500 mt-1">سيتم إضافة هذه الميزة قريباً</p>
        </div>
      ) : (
        <div className="grid grid-cols-3 gap-4">
          {machines.map((machine) => (
            <Card key={machine.id} className="cursor-pointer hover:border-brand-500/50 transition-all group" onClick={() => navigate(`/machines/${machine.id}`)}>
              <div className="flex items-start gap-3">
                <div className="w-10 h-10 rounded-xl bg-surface-800/80 flex items-center justify-center text-brand-400 group-hover:scale-110 transition-transform">
                  <Cog className="w-5 h-5" />
                </div>
                <div className="flex-1 min-w-0">
                  <h3 className="font-bold text-white group-hover:text-brand-300 transition-colors truncate">{machine.name}</h3>
                  <p className="text-xs text-surface-400 font-mono">{machine.code || "—"}</p>
                  <p className="text-xs text-surface-500 mt-1">{machine.mtype || "—"}</p>
                </div>
                <div className="text-left">
                  {machine.status && (
                    <Badge variant={(statusMap[machine.status]?.variant as any) || "info"}>
                      {statusMap[machine.status]?.label || machine.status}
                    </Badge>
                  )}
                </div>
              </div>
              <div className="mt-3 flex items-center justify-between text-xs text-surface-400">
                <span>{machine.capacity_cpm ? `${machine.capacity_cpm} كوب/دقيقة` : "—"}</span>
                <span className="truncate max-w-[120px]">{machine.supported_products || "—"}</span>
              </div>
            </Card>
          ))}
          {machines.length === 0 && (
            <div className="col-span-3 text-center py-12 text-surface-500">لا توجد آلات مسجلة</div>
          )}
        </div>
      )}
    </div>
  );
}
