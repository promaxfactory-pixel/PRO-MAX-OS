import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { motion, AnimatePresence } from "framer-motion";
import {
  Factory, Package, Plus, Trash2, Clock,
  Sun, Moon, CheckCircle2, TrendingUp, BarChart3,
  Printer
} from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { useAuthStore } from "@/stores/authStore";
import TemperatureMonitor from "@/components/production/TemperatureMonitor";

interface Product {
  id: number;
  name_ar: string | null;
  code: string | null;
}

interface ShiftLine {
  id: number;
  sheet_id: number;
  product_id: number;
  product_name: string | null;
  customer_brand: string | null;
  cartons_produced: number;
  cups_per_carton: number;
  waste_cartons: number;
  ts: string;
  recorded_by: string | null;
}

interface DashboardData {
  today_total_cups: number;
  today_total_cartons: number;
  morning_shift_cartons: number;
  evening_shift_cartons: number;
  products: { product_id: number; product_name: string | null; customer_brand: string | null; total_cartons: number; total_cups: number; waste_cartons: number }[];
  recent_entries: ShiftLine[];
}

function safeNumber(v: string): number {
  const n = Number(v);
  return Number.isFinite(n) && n >= 0 ? n : 0;
}

export default function LiveProductionPage() {
  const [today] = useState(() => new Date().toISOString().split("T")[0]);
  const [shift, setShift] = useState<"صباحي" | "مسائي">("صباحي");
  const [sheetId, setSheetId] = useState<number | null>(null);
  const [products, setProducts] = useState<Product[]>([]);
  const [lines, setLines] = useState<ShiftLine[]>([]);
  const [dashboard, setDashboard] = useState<DashboardData | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [deletingLine, setDeletingLine] = useState<number | null>(null);
  const [closingShift, setClosingShift] = useState(false);
  const [workerId, setWorkerId] = useState<number | null>(null);
  const [workers, setWorkers] = useState<{id: number; name: string; code: string | null; job: string | null}[]>([]);
  const { addNotification } = useUIStore();
  const currentUser = useAuthStore((s) => s.user);

  const [entryForm, setEntryForm] = useState({
    product_id: 0,
    customer_brand: "",
    cartons_produced: 0,
    waste_cartons: 0,
  });

  const loadProducts = useCallback(async () => {
    try {
      const prods = await invoke<Product[]>("list_products", {});
      setProducts(prods.filter((p: any) => p.active !== 0));
    } catch (e) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "فشل تحميل المنتجات: " + String(e) });
    }
  }, [addNotification]);

  const loadWorkers = useCallback(async () => {
    try {
      const w = await invoke<{id: number; name: string; code: string | null; job: string | null}[]>("list_employees_for_production");
      setWorkers(w);
    } catch { /* optional */ }
  }, []);

  const initShift = useCallback(async () => {
    try {
      const id = await invoke<number>("get_shift_sheet", { date: today, shift });
      setSheetId(id);
      const shiftLines = await invoke<ShiftLine[]>("get_shift_lines", { sheetId: id });
      setLines(shiftLines);
    } catch (e) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "فشل تهيئة الوردية: " + String(e) });
    }
  }, [today, shift, addNotification]);

  const refreshDashboard = useCallback(async () => {
    try {
      const data = await invoke<DashboardData>("get_live_dashboard", {});
      setDashboard(data);
    } catch {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "حدث خطأ أثناء تحميل البيانات" });
    }
  }, []);

  useEffect(() => {
    loadProducts();
    loadWorkers();
  }, [loadProducts, loadWorkers]);

  useEffect(() => {
    if (today && shift) {
      setLoading(true);
      initShift().finally(() => setLoading(false));
    }
  }, [today, shift, initShift]);

  useEffect(() => {
    refreshDashboard();
    const interval = setInterval(refreshDashboard, 15000);
    return () => clearInterval(interval);
  }, [refreshDashboard]);

  const handleRecord = async () => {
    if (!sheetId || !entryForm.product_id || entryForm.cartons_produced <= 0) return;
    setSaving(true);
    try {
      await invoke("record_production", {
        sheetId,
        productId: entryForm.product_id,
        customerBrand: entryForm.customer_brand || null,
        cartonsProduced: entryForm.cartons_produced,
        cupsPerCarton: null,
        wasteCartons: entryForm.waste_cartons || null,
        recordedBy: currentUser?.full_name || null,
        workerId: workerId,
      });
      setEntryForm({ product_id: 0, customer_brand: "", cartons_produced: 0, waste_cartons: 0 });
      setWorkerId(null);
      const shiftLines = await invoke<ShiftLine[]>("get_shift_lines", { sheetId });
      setLines(shiftLines);
      await refreshDashboard();
      addNotification({ id: crypto.randomUUID(), type: "success", title: "تم", message: "تم تسجيل الإنتاج بنجاح" });
    } catch (e) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "فشل تسجيل الإنتاج: " + String(e) });
    }
    setSaving(false);
  };

  const handleDeleteLine = async (lineId: number) => {
    setDeletingLine(lineId);
    try {
      await invoke("delete_production_line", { lineId });
      const shiftLines = await invoke<ShiftLine[]>("get_shift_lines", { sheetId });
      setLines(shiftLines);
      await refreshDashboard();
      addNotification({ id: crypto.randomUUID(), type: "info", title: "تم", message: "تم حذف التسجيل" });
    } catch (e) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "فشل حذف التسجيل: " + String(e) });
    }
    setDeletingLine(null);
  };

  const handleCompleteShift = async () => {
    if (!sheetId) return;
    setClosingShift(true);
    try {
      await invoke("complete_shift", { sheetId, completedBy: currentUser?.full_name || "operator" });
      setSheetId(null);
      setLines([]);
      await refreshDashboard();
      addNotification({ id: crypto.randomUUID(), type: "success", title: "تم", message: "تم إقفال الوردية وتحديث المخزون" });
    } catch (e) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: String(e) });
    }
    setClosingShift(false);
  };

  const [editingQty, setEditingQty] = useState<{ id: number; val: number } | null>(null);

  const handleUpdateLine = async (lineId: number, cartons: number) => {
    if (cartons < 0) return;
    try {
      await invoke("update_production_line", { lineId, cartonsProduced: cartons, wasteCartons: null });
      setEditingQty(null);
      const shiftLines = await invoke<ShiftLine[]>("get_shift_lines", { sheetId });
      setLines(shiftLines);
      await refreshDashboard();
    } catch (e) {
      addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ", message: "فشل تحديث الكمية: " + String(e) });
    }
  };

  const todayCups = dashboard?.today_total_cups || 0;
  const morningCartons = dashboard?.morning_shift_cartons || 0;
  const eveningCartons = dashboard?.evening_shift_cartons || 0;
  const totalCupsFormatted = todayCups.toLocaleString();
  const totalCartons = dashboard?.today_total_cartons || 0;

  return (
    <div className="space-y-6" dir="rtl">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="w-12 h-12 rounded-2xl bg-gradient-to-br from-amber-500/20 to-amber-600/10 border border-amber-500/20 flex items-center justify-center">
            <Factory className="w-6 h-6 text-amber-400" />
          </div>
          <div>
            <h1 className="text-2xl font-bold text-white">الإنتاج المباشر</h1>
            <p className="text-sm text-surface-400">{today} — راقب وسجل الإنتاج لحظة بلحظة</p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          {sheetId && lines.length > 0 && (
            <>
              <button onClick={async () => {
                try { await invoke("print_shift_report_thermal", { sheetId, printerName: null }); addNotification({ id: crypto.randomUUID(), type: "success", title: "طباعة", message: "تم إرسال تقرير الوردية إلى الطابعة" }); } catch (e) { addNotification({ id: crypto.randomUUID(), type: "error", title: "خطأ في الطباعة", message: String(e) }); }
              }} className="btn-outline flex items-center gap-2">
                <Printer className="w-4 h-4" />
                طباعة الوردية
              </button>
              <button onClick={handleCompleteShift} disabled={closingShift} className="btn-gold flex items-center gap-2 disabled:opacity-50">
                {closingShift ? <div className="w-4 h-4 border-2 border-surface-900/30 border-t-surface-900 rounded-full animate-spin" /> : <CheckCircle2 className="w-4 h-4" />}
                إقفال الوردية
              </button>
            </>
          )}
          <button onClick={() => { refreshDashboard(); initShift(); }} className="btn-outline flex items-center gap-2">
            <BarChart3 className="w-4 h-4" />
            تحديث
          </button>
        </div>
      </div>

      {/* Live Dashboard Cards */}
      <motion.div
        className="grid grid-cols-4 gap-4"
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
      >
        <motion.div className="card relative overflow-hidden" whileHover={{ scale: 1.02 }}>
          <div className="flex items-center gap-3 mb-2">
            <TrendingUp className="w-5 h-5 text-emerald-400" />
            <span className="text-sm text-surface-400">إجمالي اليوم</span>
          </div>
          <p className="text-3xl font-bold text-white">{totalCartons.toFixed(0)}</p>
          <p className="text-xs text-surface-500">كرتون</p>
          <div className="absolute top-0 left-0 w-32 h-32 bg-emerald-500/5 rounded-full blur-3xl" />
        </motion.div>

        <motion.div className="card" whileHover={{ scale: 1.02 }}>
          <div className="flex items-center gap-3 mb-2">
            <Sun className="w-5 h-5 text-amber-400" />
            <span className="text-sm text-surface-400">صباحي</span>
          </div>
          <p className="text-3xl font-bold text-amber-400">{morningCartons.toFixed(0)}</p>
          <p className="text-xs text-surface-500">كرتون</p>
        </motion.div>

        <motion.div className="card" whileHover={{ scale: 1.02 }}>
          <div className="flex items-center gap-3 mb-2">
            <Moon className="w-5 h-5 text-indigo-400" />
            <span className="text-sm text-surface-400">مسائي</span>
          </div>
          <p className="text-3xl font-bold text-indigo-400">{eveningCartons.toFixed(0)}</p>
          <p className="text-xs text-surface-500">كرتون</p>
        </motion.div>

        <motion.div className="card" whileHover={{ scale: 1.02 }}>
          <div className="flex items-center gap-3 mb-2">
            <Package className="w-5 h-5 text-gold-400" />
            <span className="text-sm text-surface-400">إجمالي الأكواب</span>
          </div>
          <p className="text-3xl font-bold gradient-text">{totalCupsFormatted}</p>
          <p className="text-xs text-surface-500">كوب</p>
        </motion.div>
      </motion.div>

      {/* Machine Temperature Monitor */}
      <TemperatureMonitor />

      {/* Shift Selector + Entry Form */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Left: Shift selector + form */}
        <div className="lg:col-span-1 space-y-4">
          <div className="card">
            <h3 className="section-title mb-4">
              <Clock className="w-4 h-4" />
              الوردية الحالية
            </h3>
            <div className="flex gap-2 mb-4">
              <button
                onClick={() => setShift("صباحي")}
                className={`flex-1 py-3 rounded-xl font-bold text-sm transition-all ${
                  shift === "صباحي"
                    ? "bg-amber-500/20 text-amber-400 border border-amber-500/30 shadow-glow-gold"
                    : "bg-surface-800 text-surface-400 border border-surface-700 hover:border-surface-600"
                }`}
              >
                <Sun className="w-4 h-4 mx-auto mb-1" />
                صباحي
              </button>
              <button
                onClick={() => setShift("مسائي")}
                className={`flex-1 py-3 rounded-xl font-bold text-sm transition-all ${
                  shift === "مسائي"
                    ? "bg-indigo-500/20 text-indigo-400 border border-indigo-500/30 shadow-lg"
                    : "bg-surface-800 text-surface-400 border border-surface-700 hover:border-surface-600"
                }`}
              >
                <Moon className="w-4 h-4 mx-auto mb-1" />
                مسائي
              </button>
            </div>

            {sheetId && (
              <div className="flex items-center gap-2 text-xs text-emerald-400 bg-emerald-500/10 rounded-lg px-3 py-2">
                <CheckCircle2 className="w-3 h-3" />
                الوردية مفتوحة — رقم {sheetId}
              </div>
            )}
          </div>

          {/* Quick Entry Form */}
          <div className="card">
            <h3 className="section-title mb-4">
              <Plus className="w-4 h-4" />
              تسجيل إنتاج
            </h3>
            <div className="space-y-3">
              <div>
                <label className="form-label">المنتج</label>
                <select
                  value={entryForm.product_id}
                  onChange={(e) => setEntryForm({ ...entryForm, product_id: Number(e.target.value) })}
                  className="input-field"
                  aria-label="المنتج"
                >
                  <option value={0}>— اختر المنتج —</option>
                  {products.map((p) => (
                    <option key={p.id} value={p.id}>
                      {p.name_ar || p.code || `منتج #${p.id}`}
                    </option>
                  ))}
                </select>
              </div>
              <div>
                <label className="form-label">العلامة التجارية للعميل</label>
                <input
                  type="text"
                  value={entryForm.customer_brand}
                  onChange={(e) => setEntryForm({ ...entryForm, customer_brand: e.target.value })}
                  placeholder="مثال: ريشوس"
                  className="input-field"
                  dir="rtl"
                  aria-label="العلامة التجارية للعميل"
                />
              </div>
              <div>
                <label className="form-label">العامل *</label>
                <select
                  className="input-field"
                  value={workerId || ''}
                  onChange={(e) => setWorkerId(Number(e.target.value) || null)}
                  aria-label="اختر العامل"
                >
                  <option value="">اختر العامل...</option>
                  {workers.map((w) => (
                    <option key={w.id} value={w.id}>{w.name} ({w.code || w.job})</option>
                  ))}
                </select>
              </div>
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="form-label">كرتون منتج</label>
                  <input
                    type="number"
                    value={entryForm.cartons_produced || ""}
                    onChange={(e) => setEntryForm({ ...entryForm, cartons_produced: safeNumber(e.target.value) })}
                    className="input-field"
                    min="0"
                    placeholder="0"
                    aria-label="كرتون منتج"
                  />
                </div>
                <div>
                  <label className="form-label">تالف (كرتون)</label>
                  <input
                    type="number"
                    value={entryForm.waste_cartons || ""}
                    onChange={(e) => setEntryForm({ ...entryForm, waste_cartons: safeNumber(e.target.value) })}
                    className="input-field"
                    min="0"
                    placeholder="0"
                    aria-label="تالف كرتون"
                  />
                </div>
              </div>
              <motion.button
                onClick={handleRecord}
                disabled={saving || !entryForm.product_id || entryForm.cartons_produced <= 0}
                className="w-full bg-gradient-to-l from-brand-800 to-brand-700 text-white font-bold py-3 rounded-xl hover:from-brand-700 hover:to-brand-600 transition-all disabled:opacity-50 flex items-center justify-center gap-2"
                whileHover={{ scale: 1.01 }}
                whileTap={{ scale: 0.98 }}
              >
                {saving ? (
                  <div className="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                ) : (
                  <>
                    <Plus className="w-4 h-4" />
                    تسجيل الإنتاج
                  </>
                )}
              </motion.button>
            </div>
          </div>
        </div>

        {/* Right: Current shift entries */}
        <div className="lg:col-span-2 space-y-4">
          <div className="card">
            <div className="flex items-center justify-between mb-4">
              <h3 className="section-title">
                سجل إنتاج الوردية {shift}
              </h3>
              <span className="text-sm text-surface-400">
                {loading ? "جارٍ التحميل..." : `${lines.length} تسجيل${lines.length !== 1 ? "ات" : ""}`}
              </span>
            </div>

            {loading ? (
              <div className="flex items-center justify-center py-12">
                <div className="w-8 h-8 border-2 border-brand-500/30 border-t-brand-500 rounded-full animate-spin" />
              </div>
            ) : lines.length === 0 ? (
              <div className="text-center py-12 text-surface-500">
                <Package className="w-12 h-12 mx-auto mb-3 opacity-30" />
                <p>لا يوجد إنتاج مسجل في هذه الوردية</p>
                <p className="text-xs mt-1">سجل أول إنتاج من النموذج على اليسار</p>
              </div>
            ) : (
              <div className="space-y-2">
                <AnimatePresence>
                  {lines.map((line) => (
                    <motion.div
                      key={line.id}
                      initial={{ opacity: 0, x: -20 }}
                      animate={{ opacity: 1, x: 0 }}
                      exit={{ opacity: 0, x: 20, height: 0 }}
                      className="p-3 bg-surface-800/50 rounded-xl border border-surface-700/30 flex items-center justify-between gap-4"
                    >
                      <div className="flex items-center gap-3 min-w-0">
                        <span className="text-xs font-bold text-brand-400 w-6">#</span>
                        <div className="min-w-0">
                          <p className="text-sm font-medium text-white truncate">
                            {line.product_name || `منتج #${line.product_id}`}
                          </p>
                          {line.customer_brand && (
                            <p className="text-xs text-gold-400/80">{line.customer_brand}</p>
                          )}
                        </div>
                      </div>

                      <div className="flex items-center gap-4">
                        <div className="text-center">
                          <p className="text-sm font-bold text-white">
                            {editingQty?.id === line.id ? (
                              <input
                                type="number"
                                value={editingQty.val}
                                onChange={(e) => setEditingQty({ id: line.id, val: safeNumber(e.target.value) })}
                                className="w-20 text-center input-field text-sm py-1"
                                autoFocus
                                aria-label="تعديل الكمية"
                                onBlur={() => handleUpdateLine(line.id, editingQty.val)}
                                onKeyDown={(e) => {
                                  if (e.key === "Enter") handleUpdateLine(line.id, editingQty.val);
                                  if (e.key === "Escape") setEditingQty(null);
                                }}
                              />
                            ) : (
                              <span
                                className="cursor-pointer hover:text-brand-400 transition-colors"
                                onClick={() => setEditingQty({ id: line.id, val: line.cartons_produced })}
                              >
                                {line.cartons_produced.toFixed(0)}
                              </span>
                            )}
                          </p>
                          <p className="text-xs text-surface-500">كرتون</p>
                        </div>
                        <div className="text-center">
                          <p className="text-sm font-bold text-surface-300">{line.cups_per_carton}</p>
                          <p className="text-xs text-surface-500">كوب/كرتون</p>
                        </div>
                        <div className="text-center">
                          <p className="text-sm font-bold text-surface-300">
                            {(line.cartons_produced * line.cups_per_carton).toLocaleString()}
                          </p>
                          <p className="text-xs text-surface-500">كوب</p>
                        </div>
                        <button
                          onClick={() => handleDeleteLine(line.id)}
                          disabled={deletingLine === line.id}
                          className="p-1.5 text-red-400/50 hover:text-red-400 transition-colors disabled:opacity-30"
                        >
                          {deletingLine === line.id ? (
                            <div className="w-4 h-4 border-2 border-red-400/30 border-t-red-400 rounded-full animate-spin" />
                          ) : (
                            <Trash2 className="w-4 h-4" />
                          )}
                        </button>
                      </div>
                    </motion.div>
                  ))}
                </AnimatePresence>
              </div>
            )}
          </div>

          {/* Per-Product Summary */}
          {dashboard && dashboard.products.length > 0 && (
            <div className="card">
              <h3 className="section-title mb-4">
                <BarChart3 className="w-4 h-4" />
                ملخص الإنتاج اليوم — حسب المنتج
              </h3>
              <div className="overflow-hidden rounded-xl border border-surface-700/50">
                <table className="data-table">
                  <thead>
                    <tr>
                      <th>المنتج</th>
                      <th>العلامة التجارية</th>
                      <th>كرتون</th>
                      <th>أكواب</th>
                      <th>تالف</th>
                    </tr>
                  </thead>
                  <tbody>
                    {dashboard.products.map((p) => (
                      <tr key={p.product_id}>
                        <td className="font-medium text-white">{p.product_name || `#${p.product_id}`}</td>
                        <td>{p.customer_brand || <span className="text-surface-500">—</span>}</td>
                        <td>{p.total_cartons.toFixed(0)}</td>
                        <td>{p.total_cups.toLocaleString()}</td>
                        <td>{p.waste_cartons.toFixed(0)}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
