from pathlib import Path

# Secure reconciliation command and wire into Tauri handler.
rec = Path("src-tauri/src/commands/reconciliation.rs")
text = rec.read_text(encoding="utf-8")
old_use = "use crate::db::DbState;"
new_use = "use crate::commands::rbac;\nuse crate::db::DbState;"
if text.count(old_use) != 1:
    raise SystemExit(f"reconciliation use anchor count={text.count(old_use)}")
text = text.replace(old_use, new_use, 1)
old_command = '''pub fn get_financial_reconciliation(
    state: State<'_, DbState>,
) -> Result<FinancialReconciliation, AppError> {
    let conn = state.0.lock()?;
    build_financial_reconciliation(&conn)
}'''
new_command = '''pub fn get_financial_reconciliation(
    state: State<'_, DbState>,
    user_id: i64,
) -> Result<FinancialReconciliation, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    build_financial_reconciliation(&conn)
}'''
if text.count(old_command) != 1:
    raise SystemExit(f"reconciliation command anchor count={text.count(old_command)}")
text = text.replace(old_command, new_command, 1)
rec.write_text(text, encoding="utf-8")

lib = Path("src-tauri/src/lib.rs")
ltext = lib.read_text(encoding="utf-8")
anchor = "            commands::dashboard::get_control_balances,\n"
addition = anchor + "            commands::reconciliation::get_financial_reconciliation,\n"
if ltext.count(anchor) != 1:
    raise SystemExit(f"lib handler anchor count={ltext.count(anchor)}")
ltext = ltext.replace(anchor, addition, 1)
lib.write_text(ltext, encoding="utf-8")

page = Path("src/pages/dashboard/DailyOperationsCenterPage.tsx")
ptext = page.read_text(encoding="utf-8")

interface_anchor = '''interface ControlBalance {
  account_code: string; account_name: string; balance_milli: number;
  balance_side: string; interpretation: string;
}
'''
interfaces = interface_anchor + '''interface ReconciliationCheck {
  key: string; account_code: string; label: string;
  gl_balance_milli: number; subledger_balance_milli: number;
  timing_items_milli: number; unposted_items_milli: number;
  adjusted_subledger_milli: number; difference_milli: number;
  status: "ok" | "review" | "mismatch"; detail: string;
}
interface FinancialReconciliation {
  checks: ReconciliationCheck[];
  mismatch_count: number;
  review_count: number;
  all_clear: boolean;
}
'''
if ptext.count(interface_anchor) != 1:
    raise SystemExit(f"interface anchor count={ptext.count(interface_anchor)}")
ptext = ptext.replace(interface_anchor, interfaces, 1)

state_anchor = '  const [controls, setControls] = useState<ControlBalance[]>([]);\n'
state_repl = state_anchor + '  const [reconciliation, setReconciliation] = useState<FinancialReconciliation | null>(null);\n'
if ptext.count(state_anchor) != 1:
    raise SystemExit(f"state anchor count={ptext.count(state_anchor)}")
ptext = ptext.replace(state_anchor, state_repl, 1)

promise_old = '''      const [s, p, k, ctl] = await Promise.all([
        invoke<DashboardStats>("get_dashboard_stats"),
        invoke<LiveProduction>("get_live_dashboard").catch(() => null),
        invoke<OperationalKpis>("get_operational_kpis", { fromDate, toDate }).catch(() => null),
        invoke<ControlBalance[]>("get_control_balances").catch(() => []),
      ]);
      setStats(s);
      setLive(p);
      setKpis(k);
      setControls(ctl);'''
promise_new = '''      const [s, p, k, ctl, rec] = await Promise.all([
        invoke<DashboardStats>("get_dashboard_stats"),
        invoke<LiveProduction>("get_live_dashboard").catch(() => null),
        invoke<OperationalKpis>("get_operational_kpis", { fromDate, toDate }).catch(() => null),
        invoke<ControlBalance[]>("get_control_balances").catch(() => []),
        invoke<FinancialReconciliation>("get_financial_reconciliation").catch(() => null),
      ]);
      setStats(s);
      setLive(p);
      setKpis(k);
      setControls(ctl);
      setReconciliation(rec);'''
if ptext.count(promise_old) != 1:
    raise SystemExit(f"promise anchor count={ptext.count(promise_old)}")
ptext = ptext.replace(promise_old, promise_new, 1)

card_anchor = '''      </Card>

      <Card>
        <div className="flex items-center justify-between mb-4">
          <div>
            <h2 className="font-bold">متوسطات الشهر حتى اليوم</h2>'''
reconciliation_card = '''      </Card>

      <Card>
        <div className="flex items-center justify-between gap-4 mb-4">
          <div>
            <h2 className="font-bold">مطابقة الدفاتر الفرعية مع الأستاذ العام</h2>
            <p className="text-xs text-surface-500 mt-1">تكشف الفرق الحقيقي وتفصل عنه فروق التوقيت والسجلات التاريخية غير المرحلة</p>
          </div>
          {reconciliation && (
            <span className={`text-xs px-3 py-1.5 rounded-full border ${
              reconciliation.mismatch_count > 0
                ? "text-red-300 border-red-500/30 bg-red-500/10"
                : reconciliation.review_count > 0
                  ? "text-amber-300 border-amber-500/30 bg-amber-500/10"
                  : "text-emerald-300 border-emerald-500/30 bg-emerald-500/10"
            }`}>
              {reconciliation.mismatch_count > 0
                ? `${reconciliation.mismatch_count} فرق غير مفسر`
                : reconciliation.review_count > 0
                  ? `${reconciliation.review_count} بند للمراجعة`
                  : "مطابق"}
            </span>
          )}
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-3">
          {reconciliation?.checks.map((item) => (
            <div key={item.key} className={`p-4 rounded-xl border ${
              item.status === "mismatch"
                ? "border-red-500/30 bg-red-500/5"
                : item.status === "review"
                  ? "border-amber-500/30 bg-amber-500/5"
                  : "border-emerald-500/20 bg-surface-800/30"
            }`}>
              <div className="flex items-start justify-between gap-3">
                <div>
                  <p className="font-bold text-sm">{item.label}</p>
                  <p className="text-[10px] font-mono text-surface-500 mt-1">GL {item.account_code}</p>
                </div>
                <span className={`text-[10px] px-2 py-1 rounded-lg ${
                  item.status === "mismatch"
                    ? "text-red-300 bg-red-500/10"
                    : item.status === "review"
                      ? "text-amber-300 bg-amber-500/10"
                      : "text-emerald-300 bg-emerald-500/10"
                }`}>
                  {item.status === "mismatch" ? "فرق" : item.status === "review" ? "مراجعة" : "مطابق"}
                </span>
              </div>
              <div className="grid grid-cols-2 gap-2 mt-3 text-xs">
                <div className="p-2 rounded-lg bg-surface-900/40">
                  <p className="text-surface-500">الأستاذ العام</p>
                  <p className="font-bold mt-1">{formatOMR(item.gl_balance_milli)}</p>
                </div>
                <div className="p-2 rounded-lg bg-surface-900/40">
                  <p className="text-surface-500">الدفتر المعدل</p>
                  <p className="font-bold mt-1">{formatOMR(item.adjusted_subledger_milli)}</p>
                </div>
              </div>
              {item.timing_items_milli !== 0 && (
                <p className="text-[11px] text-amber-300 mt-2">فروق توقيت بانتظار اعتماد: {formatOMR(item.timing_items_milli)}</p>
              )}
              {item.unposted_items_milli !== 0 && (
                <p className="text-[11px] text-amber-300 mt-1">سجلات غير مرحلة: {formatOMR(item.unposted_items_milli)}</p>
              )}
              {item.difference_milli !== 0 && (
                <p className="text-[11px] text-red-300 mt-2 font-bold">فرق غير مفسر: {formatOMR(Math.abs(item.difference_milli))}</p>
              )}
              <p className="text-[10px] text-surface-500 mt-2 leading-5">{item.detail}</p>
            </div>
          ))}
          {!reconciliation && !loading && (
            <div className="p-4 rounded-xl border border-amber-500/20 text-sm text-amber-300">تعذر تحميل المطابقة المالية. راجع الصلاحيات أو سجل النظام.</div>
          )}
        </div>
      </Card>

      <Card>
        <div className="flex items-center justify-between mb-4">
          <div>
            <h2 className="font-bold">متوسطات الشهر حتى اليوم</h2>'''
if ptext.count(card_anchor) != 1:
    raise SystemExit(f"card anchor count={ptext.count(card_anchor)}")
ptext = ptext.replace(card_anchor, reconciliation_card, 1)
page.write_text(ptext, encoding="utf-8")
