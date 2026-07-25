import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import Button from "@/components/ui/Button";
import Card from "@/components/ui/Card";
import Input, { Select, Textarea } from "@/components/ui/Input";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, Save, Plus, Trash2 } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import type { Machine, Product, ProductionLine } from "@/types";

export default function ProductionOrderCreatePage() {
  const navigate = useNavigate();
  const { addNotification } = useUIStore();
  const [machines, setMachines] = useState<Machine[]>([]);
  const [products, setProducts] = useState<Product[]>([]);
  const [machineId, setMachineId] = useState<number>(0);
  const [operator, setOperator] = useState("");
  const [supervisor, setSupervisor] = useState("");
  const [shift, setShift] = useState("morning");
  const [notes, setNotes] = useState("");
  interface OrderLineInput {
    product_id: number;
    cartons_good: number;
    cartons_waste: number;
    worker: string;
  }
  const [lines, setLines] = useState<OrderLineInput[]>([]);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    invoke("list_products").then((d) => setProducts(d as Product[])).catch((e: unknown) => addNotification({ title: 'خطأ', message: String(e), type: 'error' }));
  }, []);

  const addLine = () => setLines([...lines, { product_id: products[0]?.id || 0, cartons_good: 0, cartons_waste: 0, worker: "" }]);
  const removeLine = (i: number) => setLines(lines.filter((_, idx) => idx !== i));
  const updateLine = (i: number, field: keyof OrderLineInput, val: number | string) => { const nl = [...lines]; nl[i] = { ...nl[i], [field]: field.includes("cartons") ? Number(val) : val }; setLines(nl); };

  const handleSave = async () => {
    setSaving(true);
    try {
      const id = await invoke("create_production_order", {
        input: { machine_id: machineId || null, operator, supervisor, shift, notes, lines: lines.map(l => ({ product_id: l.product_id, cartons_good: l.cartons_good, cartons_waste: l.cartons_waste, worker: l.worker })) }
      });
      navigate(`/production/${id}`);
    } catch (err: unknown) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) }); }
    finally { setSaving(false); }
  };

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div className="flex items-center gap-3">
          <button onClick={() => navigate('/production')} className="btn-ghost p-2"><ArrowRight className="w-5 h-5" /></button>
          <div><h1 className="page-title">أمر إنتاج جديد</h1></div>
        </div>
        <Button onClick={handleSave} loading={saving} icon={<Save className="w-4 h-4" />}>حفظ</Button>
      </div>
      <div className="grid grid-cols-3 gap-6">
        <div className="col-span-2 space-y-6">
          <Card>
            <div className="grid grid-cols-3 gap-4">
              <Select label="الوردية" value={shift} onChange={(e) => setShift(e.target.value)} options={[{value:"morning",label:"صباحية"},{value:"evening",label:"مسائية"},{value:"night",label:"ليلية"}]} />
              <Input label="المشغل" value={operator} onChange={(e) => setOperator(e.target.value)} placeholder="اسم المشغل" />
              <Input label="المشرف" value={supervisor} onChange={(e) => setSupervisor(e.target.value)} placeholder="اسم المشرف" />
            </div>
          </Card>
          <Card>
            <div className="flex items-center justify-between mb-4">
              <h3 className="section-title">خطوط الإنتاج</h3>
              <Button size="sm" variant="outline" onClick={addLine} icon={<Plus className="w-3 h-3" />}>إضافة خط</Button>
            </div>
            <div className="space-y-3">
              {lines.map((line, i) => (
                <div key={i} className="flex items-center gap-3 p-3 bg-surface-900/50 rounded-xl border border-surface-700/30">
                  <select value={line.product_id} onChange={(e) => updateLine(i, "product_id", Number(e.target.value))} className="flex-1 bg-surface-800 border border-surface-700 rounded-lg px-3 py-2 text-sm text-white" aria-label="المنتج">
                    {products.map(p => <option key={p.id} value={p.id}>{p.name_ar || p.name_en}</option>)}
                  </select>
                  <input type="number" value={line.cartons_good} onChange={(e) => updateLine(i, "cartons_good", e.target.value)} className="w-24 bg-surface-800 border border-surface-700 rounded-lg px-3 py-2 text-sm text-white text-center" placeholder="صالح" aria-label="كرتون صالح" />
                  <input type="number" value={line.cartons_waste} onChange={(e) => updateLine(i, "cartons_waste", e.target.value)} className="w-24 bg-surface-800 border border-surface-700 rounded-lg px-3 py-2 text-sm text-white text-center" placeholder="هالك" aria-label="كرتون هالك" />
                  <input value={line.worker} onChange={(e) => updateLine(i, "worker", e.target.value)} className="w-32 bg-surface-800 border border-surface-700 rounded-lg px-3 py-2 text-sm text-white" placeholder="العامل" aria-label="العامل" />
                  <button onClick={() => removeLine(i)} className="p-2 text-red-400 hover:bg-red-500/10 rounded-lg"><Trash2 className="w-4 h-4" /></button>
                </div>
              ))}
              {lines.length === 0 && <button onClick={addLine} className="w-full py-8 border-2 border-dashed border-surface-700 rounded-xl text-surface-400 hover:text-white hover:border-brand-500/50 transition-all text-sm">+ أضف خط إنتاج أول</button>}
            </div>
          </Card>
        </div>
        <div className="space-y-6">
          <Card><Textarea label="ملاحظات" value={notes} onChange={(e) => setNotes(e.target.value)} placeholder="ملاحظات..." /></Card>
        </div>
      </div>
    </div>
  );
}
