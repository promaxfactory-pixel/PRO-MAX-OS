import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import Button from "@/components/ui/Button";
import Card from "@/components/ui/Card";
import { formatOMR } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, Plus, Save, Trash2 } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";

interface Line {
  item_id: number;
  qty: number;
  unit_cost_milli: number;
}

interface Supplier {
  id: number;
  name: string;
}

interface InventoryItem {
  id: number;
  name: string;
}

export default function PurchaseCreatePage() {
  const navigate = useNavigate();
  const addNotification = useUIStore((s) => s.addNotification);
  const [suppliers, setSuppliers] = useState<Supplier[]>([]);
  const [items, setItems] = useState<InventoryItem[]>([]);
  const [submitting, setSubmitting] = useState(false);

  const [supplierId, setSupplierId] = useState<number>(0);
  const [date, setDate] = useState(new Date().toISOString().split("T")[0]);
  const [supplierInvoiceNo, setSupplierInvoiceNo] = useState("");
  const [vatEnabled, setVatEnabled] = useState(true);
  const [lines, setLines] = useState<Line[]>([
    { item_id: 0, qty: 1, unit_cost_milli: 0 },
  ]);

  useEffect(() => {
    invoke("list_suppliers").then((d) => setSuppliers(d as Supplier[])).catch((e: unknown) => addNotification({ title: "خطأ", message: String(e), type: "error" }));
    invoke("list_inventory_items").then((d) => setItems(d as InventoryItem[])).catch((e: unknown) => addNotification({ title: "خطأ", message: String(e), type: "error" }));
  }, []);

  const updateLine = (index: number, field: keyof Line, value: number) => {
    setLines((prev) =>
      prev.map((l, i) => (i === index ? { ...l, [field]: value } : l))
    );
  };

  const addLine = () => {
    setLines((prev) => [...prev, { item_id: 0, qty: 1, unit_cost_milli: 0 }]);
  };

  const removeLine = (index: number) => {
    if (lines.length <= 1) return;
    setLines((prev) => prev.filter((_, i) => i !== index));
  };

  const lineTotal = (l: Line) => l.qty * l.unit_cost_milli;
  const totalNet = lines.reduce((s, l) => s + lineTotal(l), 0);
  const vatAmount = vatEnabled ? Math.round(totalNet * 0.05) : 0;
  const grandTotal = totalNet + vatAmount;

  const handleSubmit = async () => {
    if (!supplierId || !date || lines.every((l) => !l.item_id)) return;
    setSubmitting(true);
    try {
      await invoke("create_purchase", {
        input: {
          supplier_id: supplierId,
          date,
          supplier_invoice_no: supplierInvoiceNo || null,
          vat_enabled: vatEnabled,
          lines: lines.filter((l) => l.item_id > 0).map((l) => ({
            item_id: l.item_id,
            qty: l.qty,
            unit_cost_milli: l.unit_cost_milli,
          })),
        },
      });
      navigate("/purchases");
    } catch (err) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء الحفظ" });
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div>
          <h1 className="page-title">مشتريات جديدة</h1>
          <p className="page-subtitle">إنشاء أمر شراء جديد</p>
        </div>
        <Button variant="ghost" icon={<ArrowRight className="w-4 h-4" />} onClick={() => navigate("/purchases")}>
          العودة
        </Button>
      </div>

      <Card className="p-6">
        <div className="grid grid-cols-2 gap-6">
          <div className="input-group">
            <label className="input-label">المورد *</label>
            <select
              className="input-field"
              value={supplierId}
              onChange={(e) => setSupplierId(Number(e.target.value))}
              aria-label="المورد"
            >
              <option value={0}>— اختر المورد —</option>
              {suppliers.map((s) => (
                <option key={s.id} value={s.id}>{s.name}</option>
              ))}
            </select>
          </div>

          <div className="input-group">
            <label className="input-label">التاريخ *</label>
            <input
              type="date"
              className="input-field"
              value={date}
              onChange={(e) => setDate(e.target.value)}
              aria-label="التاريخ"
            />
          </div>

          <div className="input-group">
            <label className="input-label">رقم فاتورة المورد</label>
            <input
              type="text"
              className="input-field"
              value={supplierInvoiceNo}
              onChange={(e) => setSupplierInvoiceNo(e.target.value)}
              placeholder="اختياري"
              aria-label="رقم فاتورة المورد"
            />
          </div>

          <div className="input-group">
            <label className="input-label">الضريبة 5%</label>
            <label className="flex items-center gap-2 mt-2 cursor-pointer">
              <input
                type="checkbox"
                className="w-4 h-4 rounded bg-surface-700 border-surface-500 text-brand-500 focus:ring-brand-500"
                checked={vatEnabled}
                onChange={(e) => setVatEnabled(e.target.checked)}
                aria-label="تفعيل الضريبة"
              />
              <span className="text-surface-300 text-sm">تفعيل الضريبة</span>
            </label>
          </div>
        </div>
      </Card>

      <Card className="p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-white">بنود الطلب</h2>
          <Button size="sm" icon={<Plus className="w-4 h-4" />} onClick={addLine}>
            إضافة بند
          </Button>
        </div>

        <div className="space-y-3">
          {lines.map((line, idx) => (
            <div key={idx} className="grid grid-cols-[2fr_1fr_1fr_auto] gap-3 items-end">
              <div className="input-group">
                {idx === 0 && <label className="input-label">الصنف *</label>}
                <select
                  className="input-field"
                  value={line.item_id}
                  onChange={(e) => updateLine(idx, "item_id", Number(e.target.value))}
                  aria-label="الصنف"
                >
                  <option value={0}>— اختر الصنف —</option>
                  {items.map((item) => (
                    <option key={item.id} value={item.id}>{item.name}</option>
                  ))}
                </select>
              </div>

              <div className="input-group">
                {idx === 0 && <label className="input-label">الكمية *</label>}
                <input
                  type="number"
                  className="input-field"
                  min={1}
                  value={line.qty}
                  onChange={(e) => updateLine(idx, "qty", Number(e.target.value))}
                  aria-label="الكمية"
                />
              </div>

              <div className="input-group">
                {idx === 0 && <label className="input-label">سعر الوحدة (ملي) *</label>}
                <input
                  type="number"
                  className="input-field"
                  min={0}
                  value={line.unit_cost_milli}
                  onChange={(e) => updateLine(idx, "unit_cost_milli", Number(e.target.value))}
                  aria-label="سعر الوحدة بالملي"
                />
              </div>

              <div className="flex items-end gap-2 pb-1">
                <span className="text-sm font-mono text-gold-400 whitespace-nowrap min-w-[100px] text-left">
                  {formatOMR(lineTotal(line))}
                </span>
                <button
                  className="p-2 text-surface-400 hover:text-red-400 transition-colors"
                  onClick={() => removeLine(idx)}
                  disabled={lines.length <= 1}
                >
                  <Trash2 className="w-4 h-4" />
                </button>
              </div>
            </div>
          ))}
        </div>
      </Card>

      <Card className="p-6">
        <div className="space-y-2 ml-auto max-w-xs">
          <div className="flex justify-between text-surface-400">
            <span>الإجمالي قبل الضريبة</span>
            <span className="text-white font-mono">{formatOMR(totalNet)}</span>
          </div>
          {vatEnabled && (
            <div className="flex justify-between text-surface-400">
              <span>الضريبة 5%</span>
              <span className="text-white font-mono">{formatOMR(vatAmount)}</span>
            </div>
          )}
          <div className="border-t border-surface-600 pt-2 flex justify-between">
            <span className="text-lg font-semibold text-white">الإجمالي</span>
            <span className="text-lg font-bold text-gold-400 font-mono">{formatOMR(grandTotal)}</span>
          </div>
        </div>
      </Card>

      <div className="flex justify-end gap-3">
        <Button variant="ghost" onClick={() => navigate("/purchases")}>إلغاء</Button>
        <Button icon={<Save className="w-4 h-4" />} onClick={handleSubmit} disabled={submitting}>
          {submitting ? "جاري الحفظ..." : "حفظ أمر الشراء"}
        </Button>
      </div>
    </div>
  );
}
