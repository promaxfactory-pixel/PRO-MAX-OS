import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import Button from "@/components/ui/Button";
import Card from "@/components/ui/Card";
import Input, { Select, Textarea } from "@/components/ui/Input";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@/lib/tauri";
import { Plus, Trash2, Save, ArrowRight } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { Customer, Product } from "@/types";

interface InvoiceLineInput {
  product_id: number;
  cartons: number;
  unit_price: number;
  customs_price: number;
}

export default function InvoiceCreatePage() {
  const navigate = useNavigate();
  const { addNotification } = useUIStore();
  const [customers, setCustomers] = useState<Customer[]>([]);
  const [products, setProducts] = useState<Product[]>([]);
  const [selectedCustomer, setSelectedCustomer] = useState<number | null>(null);
  const [paymentType, setPaymentType] = useState("credit");
  const [notes, setNotes] = useState("");
  const [lines, setLines] = useState<InvoiceLineInput[]>([]);
  const [saving, setSaving] = useState(false);
  const [useCustoms, setUseCustoms] = useState(false);

  useEffect(() => {
    invoke("list_customers").then((d: unknown) => setCustomers(d as Customer[])).catch((e: unknown) => addNotification({ title: "خطأ", message: String(e), type: "error" }));
    invoke("list_products").then((d: unknown) => setProducts(d as Product[])).catch((e: unknown) => addNotification({ title: "خطأ", message: String(e), type: "error" }));
  }, []);

  const addLine = () => setLines([...lines, { product_id: products[0]?.id || 0, cartons: 1, unit_price: 0, customs_price: 0 }]);
  const removeLine = (i: number) => setLines(lines.filter((_, idx) => idx !== i));
  const updateLine = (i: number, field: keyof InvoiceLineInput, val: number | string) => {
    const newLines = [...lines];
    newLines[i] = { ...newLines[i], [field]: field === "cartons" || field === "unit_price" || field === "customs_price" ? Number(val) : val };
    setLines(newLines);
  };

  const total = lines.reduce((s, l) => s + l.cartons * l.unit_price, 0);
  const vat = total * 0.05;
  const grandTotal = total + vat;

  const handleSave = async () => {
    if (!selectedCustomer || lines.length === 0) return;
    setSaving(true);
    try {
      const id = await invoke("create_invoice", {
        input: {
          customer_id: selectedCustomer,
          payment_type: paymentType,
          notes,
          lines: lines.map(l => ({
            product_id: l.product_id,
            cartons: l.cartons,
            unit_price_milli: l.unit_price * 1000,
            customs_price_milli: useCustoms && l.customs_price > 0 ? l.customs_price * 1000 : null,
          })),
        },
      });
      navigate(`/invoices/${id}`);
    } catch (err: unknown) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(err) }); }
    finally { setSaving(false); }
  };

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div className="flex items-center gap-3">
          <button onClick={() => navigate("/invoices")} className="btn-ghost p-2"><ArrowRight className="w-5 h-5" /></button>
          <div>
            <h1 className="page-title">فاتورة جديدة</h1>
            <p className="page-subtitle">إنشاء فاتورة مبيعات جديدة</p>
          </div>
        </div>
        <div className="flex items-center gap-3">
          <Button variant="outline" onClick={handleSave} loading={saving} icon={<Save className="w-4 h-4" />}>حفظ مسودة</Button>
        </div>
      </div>

      <div className="grid grid-cols-3 gap-6">
        <div className="col-span-2 space-y-6">
          <Card>
            <div className="grid grid-cols-3 gap-4">
              <Select label="العميل" value={selectedCustomer || ""} onChange={(e) => setSelectedCustomer(Number(e.target.value))}
                options={customers.map(c => ({ value: c.id, label: c.name }))} placeholder="اختر العميل" />
              <Select label="نوع الدفع" value={paymentType} onChange={(e) => setPaymentType(e.target.value)}
                options={[{ value: "cash", label: "نقدي" }, { value: "credit", label: "آجل" }, { value: "cheque", label: "شيك" }]} />
              <Input label="التاريخ" type="date" defaultValue={new Date().toISOString().split("T")[0]} />
            </div>
          </Card>

          <Card>
            <div className="flex items-center justify-between mb-4">
              <h3 className="section-title">بنود الفاتورة</h3>
              <div className="flex items-center gap-4">
                <label className="flex items-center gap-2 text-xs text-surface-400 cursor-pointer">
                  <input type="checkbox" checked={useCustoms} onChange={(e) => setUseCustoms(e.target.checked)}
                    className="rounded border-surface-600" aria-label="أسعار الجمارك" />
                  أسعار الجمارك
                </label>
                <Button size="sm" variant="outline" onClick={addLine} icon={<Plus className="w-3 h-3" />}>إضافة بند</Button>
              </div>
            </div>
            <div className="space-y-3">
              {lines.map((line, i) => (
                <div key={i} className="flex items-center gap-3 p-3 bg-surface-900/50 rounded-xl border border-surface-700/30">
                  <select value={line.product_id} onChange={(e) => updateLine(i, "product_id", Number(e.target.value))}
                    className="flex-1 bg-surface-800 border border-surface-700 rounded-lg px-3 py-2 text-sm text-white" aria-label="المنتج">
                    {products.map(p => <option key={p.id} value={p.id}>{p.name_ar || p.name_en} ({p.code})</option>)}
                  </select>
                  <input type="number" value={line.cartons} onChange={(e) => updateLine(i, "cartons", e.target.value)}
                    className="w-20 bg-surface-800 border border-surface-700 rounded-lg px-3 py-2 text-sm text-white text-center" placeholder="كرتون" min="0" aria-label="الكراتين" />
                  <input type="number" value={line.unit_price} onChange={(e) => updateLine(i, "unit_price", e.target.value)}
                    className="w-28 bg-surface-800 border border-surface-700 rounded-lg px-3 py-2 text-sm text-white text-left" placeholder="سعر البيع" min="0" step="0.001" aria-label="سعر البيع" />
                  {useCustoms && (
                    <input type="number" value={line.customs_price} onChange={(e) => updateLine(i, "customs_price", e.target.value)}
                      className="w-28 bg-amber-900/20 border border-amber-700/40 rounded-lg px-3 py-2 text-sm text-amber-300 text-left" placeholder="سعر الجمارك" min="0" step="0.001" aria-label="سعر الجمارك" />
                  )}
                  <span className="text-sm text-surface-400 w-28 text-left shrink-0">{formatOMR(line.cartons * (useCustoms && line.customs_price > 0 ? line.customs_price : line.unit_price) * 1000)}</span>
                  <button onClick={() => removeLine(i)} className="p-2 text-red-400 hover:bg-red-500/10 rounded-lg"><Trash2 className="w-4 h-4" /></button>
                </div>
              ))}
              {lines.length === 0 && (
                <button onClick={addLine} className="w-full py-8 border-2 border-dashed border-surface-700 rounded-xl text-surface-400 hover:text-white hover:border-brand-500/50 transition-all text-sm">
                  + أضف بندًا أولًا
                </button>
              )}
            </div>
          </Card>
        </div>

        <div className="space-y-6">
          <Card className="sticky top-24">
            <h3 className="section-title">الملخص</h3>
            <div className="space-y-3">
              <div className="flex justify-between text-sm"><span className="text-surface-400">الإجمالي قبل الضريبة</span><span>{formatOMR(total * 1000)}</span></div>
              <div className="flex justify-between text-sm"><span className="text-surface-400">ضريبة القيمة المضافة (5%)</span><span>{formatOMR(vat * 1000)}</span></div>
              <hr className="border-surface-700" />
              <div className="flex justify-between text-lg font-bold"><span className="gold-accent">الإجمالي</span><span className="gradient-text">{formatOMR(grandTotal * 1000)}</span></div>
            </div>
          </Card>
          <Card>
            <Textarea label="ملاحظات" value={notes} onChange={(e) => setNotes(e.target.value)} placeholder="ملاحظات إضافية..." />
          </Card>
        </div>
      </div>
    </div>
  );
}
