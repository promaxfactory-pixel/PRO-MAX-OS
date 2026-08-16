use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::rbac;
use crate::db::DbState;
use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct ShiftLine {
    pub id: i64,
    pub sheet_id: i64,
    pub product_id: i64,
    pub product_name: Option<String>,
    pub customer_brand: Option<String>,
    pub cartons_produced: f64,
    pub cups_per_carton: i64,
    pub waste_cartons: f64,
    pub unit_cost_milli: i64,
    pub material_cost_milli: i64,
    pub ts: String,
    pub recorded_by: Option<String>,
    pub worker_id: Option<i64>,
    pub worker_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LiveProductionSummary {
    pub today_total_cartons: f64,
    pub today_total_cups: f64,
    pub morning_shift_cartons: f64,
    pub evening_shift_cartons: f64,
    pub products: Vec<ProductProductionSummary>,
    pub recent_entries: Vec<ShiftLine>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductProductionSummary {
    pub product_id: i64,
    pub product_name: Option<String>,
    pub customer_brand: Option<String>,
    pub total_cartons: f64,
    pub total_cups: f64,
    pub waste_cartons: f64,
}

#[tauri::command]
pub fn get_shift_sheet(state: State<'_, DbState>, user_id: i64, date: String, shift: String) -> Result<i64, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "manager", "operator"])?;

    let existing: Option<i64> = conn
        .query_row(
            "SELECT id FROM operations_daily_sheets WHERE date = ?1 AND shift = ?2 AND status = 'Draft'",
            params![date, shift],
            |row| row.get(0),
        )
        .ok();

    if let Some(id) = existing {
        return Ok(id);
    }

    let seq: i64 = conn
        .query_row("SELECT COALESCE(MAX(id), 0) + 1 FROM operations_daily_sheets", [], |row| row.get(0))
        ?;
    let sheet_no = format!("PRD-{:04}", seq);

    conn.execute(
        "INSERT INTO operations_daily_sheets (sheet_no, date, shift, start_time, status, created_at)
         VALUES (?1, ?2, ?3, datetime('now'), 'Draft', datetime('now'))",
        params![sheet_no, date, shift],
    )
    ?;

    let id = conn.last_insert_rowid();
    let _ = rbac::log_audit(&conn, Some(user_id), None, "get_shift_sheet_create", "operations_daily_sheets", Some(id), None, Some(&sheet_no), None);
    Ok(id)
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub fn record_production(
    state: State<'_, DbState>,
    user_id: i64,
    sheet_id: i64,
    product_id: i64,
    customer_brand: Option<String>,
    cartons_produced: f64,
    cups_per_carton: Option<i64>,
    waste_cartons: Option<f64>,
    recorded_by: Option<String>,
    worker_id: Option<i64>,
) -> Result<ShiftLine, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "manager", "operator"])?;

    let cpc = cups_per_carton.unwrap_or(1000);
    let waste = waste_cartons.unwrap_or(0.0);

    conn.execute(
        "INSERT INTO production_shift_lines (sheet_id, product_id, customer_brand, cartons_produced, cups_per_carton, waste_cartons, recorded_by, worker_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![sheet_id, product_id, customer_brand, cartons_produced, cpc, waste, recorded_by, worker_id],
    )
    ?;

    let id = conn.last_insert_rowid();

    let line = conn.query_row(
        "SELECT psl.id, psl.sheet_id, psl.product_id, COALESCE(p.name_ar, p.name_en, '') as product_name,
                psl.customer_brand, psl.cartons_produced, psl.cups_per_carton, psl.waste_cartons, psl.unit_cost_milli, psl.material_cost_milli, psl.ts, psl.recorded_by,
                psl.worker_id, e.name as worker_name
         FROM production_shift_lines psl
         LEFT JOIN products p ON p.id = psl.product_id
         LEFT JOIN employees e ON e.id = psl.worker_id
         WHERE psl.id = ?1",
        params![id],
        row_to_shift_line,
    )?;

    update_sheet_totals(&conn, sheet_id)?;

    let _ = rbac::log_audit(&conn, Some(user_id), None, "record_production", "production_shift_lines", None, None, None, None);

    Ok(line)
}

fn update_sheet_totals(conn: &rusqlite::Connection, sheet_id: i64) -> Result<(), AppError> {
    conn.execute(
        "UPDATE operations_daily_sheets SET
            cartons_produced = (SELECT COALESCE(SUM(cartons_produced), 0) FROM production_shift_lines WHERE sheet_id = ?1),
            total_cups = (SELECT COALESCE(SUM(cartons_produced * cups_per_carton), 0) FROM production_shift_lines WHERE sheet_id = ?1)
         WHERE id = ?1",
        params![sheet_id],
    )
    ?;
    Ok(())
}

#[tauri::command]
pub fn get_shift_lines(state: State<'_, DbState>, sheet_id: i64) -> Result<Vec<ShiftLine>, AppError> {
    let conn = state.0.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT psl.id, psl.sheet_id, psl.product_id, COALESCE(p.name_ar, p.name_en, ''),
                    psl.customer_brand, psl.cartons_produced, psl.cups_per_carton, psl.waste_cartons, psl.ts, psl.recorded_by,
                    psl.worker_id, e.name as worker_name
             FROM production_shift_lines psl
             LEFT JOIN products p ON p.id = psl.product_id
             LEFT JOIN employees e ON e.id = psl.worker_id
             WHERE psl.sheet_id = ?1
             ORDER BY psl.ts DESC",
        )
        ?;

    let rows = stmt.query_map(params![sheet_id], row_to_shift_line)?;
    let mut lines = Vec::new();
    for row in rows {
        lines.push(row?);
    }
    Ok(lines)
}

#[tauri::command]
pub fn complete_shift(state: State<'_, DbState>, user_id: i64, sheet_id: i64, completed_by: String) -> Result<String, AppError> {
    let mut conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "manager", "operator"])?;
    let tx = conn.transaction()?;
    let res = complete_shift_inner(&tx, sheet_id, &completed_by)?;
    let _ = rbac::log_audit(&tx, Some(user_id), None, "complete_shift", "operations_daily_sheets", Some(sheet_id), None, None, None);
    tx.commit()?;
    Ok(res)
}

/// Closes a shift sheet, posts finished goods to stock at their computed cost,
/// and — when a BOM exists for the product — consumes the required raw
/// materials (qty per carton + waste allowance). The per-line production cost is
/// written back to `production_shift_lines` for accurate factory costing.
///
/// Atomic: if any material is short, the whole shift is rejected and nothing is
/// posted (guarded by the caller's transaction).
pub(crate) fn complete_shift_inner(conn: &rusqlite::Connection, sheet_id: i64, completed_by: &str) -> Result<String, AppError> {
    let status: String = conn.query_row(
        "SELECT status FROM operations_daily_sheets WHERE id = ?1",
        params![sheet_id],
        |row| row.get(0),
    ).map_err(|_| AppError::not_found("الوردية غير موجودة"))?;

    if status != "Draft" {
        return Err(AppError::validation("لا يمكن إقفال وردية تم إقفالها مسبقاً"));
    }

    let lines: Vec<(i64, f64, f64)> = {
        let mut stmt = conn
            .prepare(
                "SELECT psl.product_id, psl.cartons_produced, psl.waste_cartons
                 FROM production_shift_lines psl WHERE psl.sheet_id = ?1",
            )
            ?;
        let rows = stmt.query_map(params![sheet_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?, row.get::<_, f64>(2)?))
        })?;
        let mut v = Vec::new();
        for r in rows {
            v.push(r?);
        }
        v
    };

    // ---- Pass 1: plan and validate every material BEFORE mutating anything,
    // so a shortage anywhere rejects the whole shift atomically even if the
    // caller did not open an outer transaction.
    struct MaterialPlan {
        item_id: i64,
        required: f64,
        avg_cost_milli: i64,
    }
    struct LinePlan {
        product_id: i64,
        cartons: f64,
        materials: Vec<MaterialPlan>,
        has_bom: bool,
        fallback_cost_milli: i64,
    }
    let mut plans: Vec<LinePlan> = Vec::new();

    for (product_id, cartons, waste) in &lines {
        if *cartons <= 0.0 {
            continue;
        }
        let total_produced = *cartons + *waste;
        let mut materials: Vec<MaterialPlan> = Vec::new();
        let mut has_bom = false;

        let mut bom_stmt = conn.prepare(
            "SELECT b.item_id, b.qty_per_carton, b.waste_pct FROM bom b WHERE b.product_id = ?1 AND b.active = 1",
        )?;
        let bom_rows = bom_stmt.query_map(params![product_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?, row.get::<_, f64>(2)?))
        })?;
        for br in bom_rows {
            let (item_id, qty_per_carton, waste_pct) = br?;
            has_bom = true;
            let required = total_produced * qty_per_carton * (1.0 + waste_pct / 100.0);
            if required <= 0.0 {
                continue;
            }
            let (on_hand, item_name, avg_cost): (f64, String, i64) = conn
                .query_row(
                    "SELECT ii.qty_on_hand, COALESCE(ii.name_ar, ii.name_en, ''), ii.avg_cost_milli
                     FROM inventory_items ii WHERE ii.id = ?1",
                    params![item_id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .map_err(|_| AppError::not_found("مادة خام في قائمة BOM غير موجودة بالمخزون"))?;
            if on_hand + 1e-9 < required {
                return Err(AppError::validation(format!(
                    "رصيد غير كافٍ للمادة '{}': المتاح {:.3}، المطلوب {:.3} للورديية",
                    item_name, on_hand, required
                )));
            }
            materials.push(MaterialPlan { item_id, required, avg_cost_milli: avg_cost });
        }

        let fallback_cost_milli: i64 = conn
            .query_row(
                "SELECT COALESCE(default_cost_milli, 0) FROM products WHERE id = ?1",
                params![product_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        plans.push(LinePlan { product_id: *product_id, cartons: *cartons, materials, has_bom, fallback_cost_milli });
    }

    // ---- Pass 2: apply consumption, finished-goods posting and costs.
    for plan in &plans {
        let mut material_cost_milli: i64 = 0;
        for m in &plan.materials {
            conn.execute(
                "UPDATE inventory_items SET qty_on_hand = qty_on_hand - ?1 WHERE id = ?2",
                params![m.required, m.item_id],
            )?;
            conn.execute(
                "INSERT INTO inventory_movements (ts, item_id, mtype, qty_in, qty_out, unit_cost_milli, ref_type, ref_id, notes)
                 VALUES (datetime('now'), ?1, 'production', 0, ?2, ?3, 'production_shift', ?4, 'صرف خامات للورديية')",
                params![m.item_id, m.required, m.avg_cost_milli, sheet_id],
            )?;
            material_cost_milli += (m.required * m.avg_cost_milli as f64).round() as i64;
        }

        // Production cost of the good cartons. With a BOM the cost is the raw
        // material bill; without one we fall back to the product's default cost.
        let total_cost_milli = if plan.has_bom {
            material_cost_milli
        } else {
            (plan.fallback_cost_milli as f64 * plan.cartons).round() as i64
        };
        let unit_cost_milli = (total_cost_milli as f64 / plan.cartons).round() as i64;

        // Finished goods enter stock at the computed cost (weighted-average merge).
        let fin: Option<(f64, i64)> = conn
            .query_row(
                "SELECT ii.qty_on_hand, ii.avg_cost_milli FROM inventory_items ii
                 WHERE ii.product_id = ?1 AND ii.kind = 'finished'",
                params![plan.product_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok();
        if let Some((old_qty, old_avg)) = fin {
            let new_qty = old_qty + plan.cartons;
            let new_avg = if new_qty > 0.0 {
                ((old_qty * old_avg as f64 + plan.cartons * unit_cost_milli as f64) / new_qty).round() as i64
            } else {
                0
            };
            conn.execute(
                "UPDATE inventory_items SET qty_on_hand = qty_on_hand + ?1, avg_cost_milli = ?2
                 WHERE product_id = ?3 AND kind = 'finished'",
                params![plan.cartons, new_avg, plan.product_id],
            )?;
            conn.execute(
                "INSERT INTO inventory_movements (ts, item_id, mtype, qty_in, qty_out, unit_cost_milli, ref_type, ref_id, notes)
                 SELECT datetime('now'), ii.id, 'production', ?1, 0, ?2, 'production_shift', ?3, 'إنتاج من الوردية'
                 FROM inventory_items ii WHERE ii.product_id = ?4 AND ii.kind = 'finished'",
                params![plan.cartons, unit_cost_milli, sheet_id, plan.product_id],
            )?;
        }

        conn.execute(
            "UPDATE production_shift_lines SET unit_cost_milli = ?1, material_cost_milli = ?2
             WHERE sheet_id = ?3 AND product_id = ?4",
            params![unit_cost_milli, material_cost_milli, sheet_id, plan.product_id],
        )?;
    }

    conn.execute(
        "UPDATE operations_daily_sheets SET status = 'Completed', completed_by = ?1, completed_at = datetime('now')
         WHERE id = ?2",
        params![completed_by, sheet_id],
    )
    ?;

    Ok("تم إقفال الوردية وتحديث المخزون".to_string())
}

#[tauri::command]
pub fn update_production_line(
    state: State<'_, DbState>,
    user_id: i64,
    line_id: i64,
    cartons_produced: f64,
    waste_cartons: Option<f64>,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "manager", "operator"])?;

    let sheet_id: i64 = conn.query_row(
        "SELECT psl.sheet_id FROM production_shift_lines psl
         JOIN operations_daily_sheets ods ON ods.id = psl.sheet_id
         WHERE psl.id = ?1 AND ods.status = 'Draft'",
        params![line_id],
        |row| row.get(0),
    ).map_err(|_| AppError::not_found("السطر غير موجود أو الوردية مقفلة"))?;

    conn.execute(
        "UPDATE production_shift_lines SET cartons_produced = ?1, waste_cartons = COALESCE(?2, waste_cartons) WHERE id = ?3",
        params![cartons_produced, waste_cartons, line_id],
    )
    ?;

    update_sheet_totals(&conn, sheet_id)?;

    let _ = rbac::log_audit(&conn, Some(user_id), None, "update_production_line", "production_shift_lines", Some(line_id), None, None, None);

    Ok("تم التحديث".to_string())
}

#[tauri::command]
pub fn delete_production_line(state: State<'_, DbState>, user_id: i64, line_id: i64) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "manager", "operator"])?;

    let sheet_id: i64 = conn.query_row(
        "SELECT psl.sheet_id FROM production_shift_lines psl
         JOIN operations_daily_sheets ods ON ods.id = psl.sheet_id
         WHERE psl.id = ?1 AND ods.status = 'Draft'",
        params![line_id],
        |row| row.get(0),
    ).map_err(|_| AppError::not_found("السطر غير موجود أو الوردية مقفلة"))?;

    conn.execute("DELETE FROM production_shift_lines WHERE id = ?1", params![line_id])
        ?;

    update_sheet_totals(&conn, sheet_id)?;

    let _ = rbac::log_audit(&conn, Some(user_id), None, "delete_production_line", "production_shift_lines", Some(line_id), None, None, None);

    Ok("تم الحذف".to_string())
}

#[tauri::command]
pub fn get_live_dashboard(state: State<'_, DbState>) -> Result<LiveProductionSummary, AppError> {
    let conn = state.0.lock()?;

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    let today_total: f64 = conn.query_row(
        "SELECT COALESCE(SUM(psl.cartons_produced * psl.cups_per_carton), 0) as total
         FROM production_shift_lines psl
         JOIN operations_daily_sheets ods ON ods.id = psl.sheet_id
         WHERE ods.date = ?1",
        params![today],
        |row| row.get(0),
    ).unwrap_or(0.0);

    let today_cartons: f64 = conn.query_row(
        "SELECT COALESCE(SUM(psl.cartons_produced), 0)
         FROM production_shift_lines psl
         JOIN operations_daily_sheets ods ON ods.id = psl.sheet_id
         WHERE ods.date = ?1",
        params![today],
        |row| row.get(0),
    ).unwrap_or(0.0);

    let morning: f64 = conn.query_row(
        "SELECT COALESCE(SUM(psl.cartons_produced), 0)
         FROM production_shift_lines psl
         JOIN operations_daily_sheets ods ON ods.id = psl.sheet_id
         WHERE ods.date = ?1 AND ods.shift = 'صباحي'",
        params![today],
        |row| row.get(0),
    ).unwrap_or(0.0);

    let evening: f64 = conn.query_row(
        "SELECT COALESCE(SUM(psl.cartons_produced), 0)
         FROM production_shift_lines psl
         JOIN operations_daily_sheets ods ON ods.id = psl.sheet_id
         WHERE ods.date = ?1 AND ods.shift = 'مسائي'",
        params![today],
        |row| row.get(0),
    ).unwrap_or(0.0);

    let mut stmt = conn.prepare(
        "SELECT psl.product_id, COALESCE(p.name_ar, p.name_en, '') as pname, psl.customer_brand,
                SUM(psl.cartons_produced) as tot_cartons,
                SUM(psl.cartons_produced * psl.cups_per_carton) as tot_cups,
                SUM(psl.waste_cartons) as tot_waste
         FROM production_shift_lines psl
         JOIN operations_daily_sheets ods ON ods.id = psl.sheet_id
         LEFT JOIN products p ON p.id = psl.product_id
         WHERE ods.date = ?1
         GROUP BY psl.product_id, psl.customer_brand
         ORDER BY tot_cartons DESC",
    )?;

    let products: Vec<ProductProductionSummary> = stmt
        .query_map(params![today], |row| {
            Ok(ProductProductionSummary {
                product_id: row.get(0)?,
                product_name: row.get(1)?,
                customer_brand: row.get(2)?,
                total_cartons: row.get(3)?,
                total_cups: row.get(4)?,
                waste_cartons: row.get(5)?,
            })
        })
        ?
        .filter_map(|r| r.ok())
        .collect();

    let mut recent = conn.prepare(
        "SELECT psl.id, psl.sheet_id, psl.product_id, COALESCE(p.name_ar, p.name_en, ''),
                psl.customer_brand, psl.cartons_produced, psl.cups_per_carton, psl.waste_cartons, psl.unit_cost_milli, psl.material_cost_milli, psl.ts, psl.recorded_by,
                psl.worker_id, e.name as worker_name
         FROM production_shift_lines psl
         JOIN operations_daily_sheets ods ON ods.id = psl.sheet_id
         LEFT JOIN products p ON p.id = psl.product_id
         LEFT JOIN employees e ON e.id = psl.worker_id
         WHERE ods.date = ?1
         ORDER BY psl.ts DESC LIMIT 20",
    )?;

    let recent_entries: Vec<ShiftLine> = recent
        .query_map(params![today], row_to_shift_line)
        ?
        .filter_map(|r| r.ok())
        .collect();

    Ok(LiveProductionSummary {
        today_total_cups: today_total,
        today_total_cartons: today_cartons,
        morning_shift_cartons: morning,
        evening_shift_cartons: evening,
        products,
        recent_entries,
    })
}

#[tauri::command]
pub fn print_shift_report_thermal(
    state: State<'_, DbState>,
    sheet_id: i64,
    printer_name: Option<String>,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;

    let sheet: (String, String, String, String) = conn.query_row(
        "SELECT date, shift, COALESCE(sheet_no, ''), COALESCE(created_by, 'operator') FROM operations_daily_sheets WHERE id = ?1",
        params![sheet_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    ).map_err(|e| format!("Sheet not found: {}", e))?;

    let (date, shift, sheet_no, created_by) = sheet;

    let mut stmt = conn.prepare(
        "SELECT psl.cartons_produced, psl.cups_per_carton, psl.waste_cartons, COALESCE(p.name_ar, p.name_en, ''), psl.customer_brand
         FROM production_shift_lines psl
         LEFT JOIN products p ON p.id = psl.product_id
         WHERE psl.sheet_id = ?1
         ORDER BY psl.ts"
    )?;

    let rows: Vec<(f64, i64, f64, String, Option<String>)> = stmt.query_map(params![sheet_id], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
    })?
    .filter_map(|r| r.ok())
    .collect();

    let mut total_cartons = 0.0;
    let mut total_cups = 0.0;
    let mut total_waste = 0.0;

    let mut lines: Vec<String> = Vec::new();
    lines.push("================================================".to_string());
    lines.push("           ProMax ERP - بروماكس".to_string());
    lines.push("        تقرير إنتاج الوردية".to_string());
    lines.push("================================================".to_string());
    lines.push(format!("  التاريخ: {}", date));
    lines.push(format!("  الوردية: {}", shift));
    lines.push(format!("  رقم الورقة: {}", sheet_no));
    lines.push("------------------------------------------------".to_string());

    for (cartons, cpc, waste, pname, brand) in &rows {
        let cups = cartons * *cpc as f64;
        total_cartons += cartons;
        total_cups += cups;
        total_waste += waste;
        lines.push(format!("  {}:", pname));
        if let Some(b) = brand {
            lines.push(format!("    العلامة: {}", b));
        }
        lines.push(format!("    الكرتون: {}  الأكواب: {}", cartons, cups));
        if *waste > 0.0 {
            lines.push(format!("    التالف: {}", waste));
        }
    }

    lines.push("------------------------------------------------".to_string());
    lines.push(format!("  الإجمالي: {} كرتون", total_cartons));
    lines.push(format!("            {} كوب", total_cups));
    if total_waste > 0.0 {
        lines.push(format!("  التالف:    {} كرتون", total_waste));
    }
    lines.push("------------------------------------------------".to_string());
    lines.push(format!("  مسجل بواسطة: {}", created_by));
    lines.push(format!("  وقت الطباعة: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M")));
    lines.push("================================================".to_string());
    lines.push("".to_string());
    lines.push("".to_string());

    // ESC/POS commands via PowerShell
    let esc = |c: u8| -> String { format!("\\x{:02x}", c) };
    let mut raw = String::new();
    raw.push_str(&esc(0x1b)); raw.push('@'); // Initialize
    raw.push_str(&esc(0x1b)); raw.push('E'); raw.push('\x01'); // Bold on
    raw.push_str("           ProMax ERP\n");
    raw.push_str(&esc(0x1b)); raw.push('E'); raw.push('\x00'); // Bold off
    raw.push_str("----------------------------------------\n");

    for line in &lines {
        raw.push_str(line);
        raw.push('\n');
    }

    raw.push_str(&esc(0x1b)); raw.push('m'); // Cut

    let temp_file = std::env::temp_dir().join("promax_shift_report.bin");
    std::fs::write(&temp_file, raw.as_bytes()).map_err(|e| format!("Failed to write temp file: {}", e))?;

    let printer = printer_name.unwrap_or_default();
    let ps = format!(
        r#"Get-Content -Path {0} -Encoding Byte | Out-Printer -Name {1} -Wait"#,
        crate::commands::device::ps_quote(temp_file.to_str().unwrap_or("")),
        crate::commands::device::ps_quote(&printer)
    );

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps])
        .output()
        .map_err(|e| format!("Failed to print: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::business(format!("Print error: {}", stderr)));
    }

    let _ = std::fs::remove_file(&temp_file);
    Ok("تمت طباعة تقرير الوردية بنجاح".to_string())
}

fn row_to_shift_line(row: &rusqlite::Row) -> rusqlite::Result<ShiftLine> {
    Ok(ShiftLine {
        id: row.get(0)?,
        sheet_id: row.get(1)?,
        product_id: row.get(2)?,
        product_name: row.get(3)?,
        customer_brand: row.get(4)?,
        cartons_produced: row.get(5)?,
        cups_per_carton: row.get(6)?,
        waste_cartons: row.get(7)?,
        unit_cost_milli: row.get(8)?,
        material_cost_milli: row.get(9)?,
        ts: row.get(10)?,
        recorded_by: row.get(11)?,
        worker_id: row.get(12)?,
        worker_name: row.get(13)?,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkerDailySummary {
    pub employee_id: i64,
    pub worker_name: Option<String>,
    pub total_cartons: f64,
    pub total_cups: f64,
    pub total_waste: f64,
    pub products: Vec<ProductProductionSummary>,
}

#[tauri::command]
pub fn get_worker_daily_report(state: State<'_, DbState>, date: String) -> Result<Vec<WorkerDailySummary>, AppError> {
    let conn = state.0.lock()?;

    let mut stmt = conn.prepare(
        "SELECT psl.worker_id, e.name as worker_name,
                SUM(psl.cartons_produced) as total_cartons,
                SUM(psl.cartons_produced * psl.cups_per_carton) as total_cups,
                SUM(psl.waste_cartons) as total_waste
         FROM production_shift_lines psl
         JOIN operations_daily_sheets ods ON ods.id = psl.sheet_id
         LEFT JOIN employees e ON e.id = psl.worker_id
         WHERE ods.date = ?1 AND psl.worker_id IS NOT NULL
         GROUP BY psl.worker_id, e.name
         ORDER BY total_cartons DESC",
    )?;

    let worker_rows: Vec<(i64, Option<String>, f64, f64, f64)> = stmt
        .query_map(params![date], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
        })
        ?
        .filter_map(|r| r.ok())
        .collect();

    let mut results = Vec::new();

    for (worker_id, worker_name, total_cartons, total_cups, total_waste) in worker_rows {
        let mut prod_stmt = conn.prepare(
            "SELECT psl.product_id, COALESCE(p.name_ar, p.name_en, '') as pname, psl.customer_brand,
                    SUM(psl.cartons_produced) as tot_cartons,
                    SUM(psl.cartons_produced * psl.cups_per_carton) as tot_cups,
                    SUM(psl.waste_cartons) as tot_waste
             FROM production_shift_lines psl
             JOIN operations_daily_sheets ods ON ods.id = psl.sheet_id
             LEFT JOIN products p ON p.id = psl.product_id
             WHERE ods.date = ?1 AND psl.worker_id = ?2
             GROUP BY psl.product_id, psl.customer_brand
             ORDER BY tot_cartons DESC",
        )?;

        let products: Vec<ProductProductionSummary> = prod_stmt
            .query_map(params![date, worker_id], |row| {
                Ok(ProductProductionSummary {
                    product_id: row.get(0)?,
                    product_name: row.get(1)?,
                    customer_brand: row.get(2)?,
                    total_cartons: row.get(3)?,
                    total_cups: row.get(4)?,
                    waste_cartons: row.get(5)?,
                })
            })
            ?
            .filter_map(|r| r.ok())
            .collect();

        results.push(WorkerDailySummary {
            employee_id: worker_id,
            worker_name,
            total_cartons,
            total_cups,
            total_waste,
            products,
        });
    }

    Ok(results)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShiftInventorySnapshot {
    pub id: i64,
    pub date: String,
    pub shift: String,
    pub item_id: i64,
    pub opening_qty: f64,
    pub closing_qty: f64,
    pub recorded_by: Option<String>,
    pub ts: String,
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn record_shift_inventory_snapshot(
    state: State<'_, DbState>,
    user_id: i64,
    date: String,
    shift: String,
    item_id: i64,
    opening_qty: f64,
    closing_qty: f64,
    recorded_by: Option<String>,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "manager", "operator"])?;

    conn.execute(
        "INSERT INTO shift_inventory_snapshots (date, shift, item_id, opening_qty, closing_qty, recorded_by, ts)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'))",
        params![date, shift, item_id, opening_qty, closing_qty, recorded_by],
    )
    ?;

    let _ = rbac::log_audit(&conn, Some(user_id), None, "record_shift_inventory_snapshot", "shift_inventory_snapshots", None, None, None, None);

    Ok("تم حفظ جرد الوردية".to_string())
}

#[tauri::command]
pub fn get_shift_inventory_snapshots(state: State<'_, DbState>, date: String) -> Result<Vec<ShiftInventorySnapshot>, AppError> {
    let conn = state.0.lock()?;

    let mut stmt = conn.prepare(
        "SELECT id, date, shift, item_id, opening_qty, closing_qty, recorded_by, ts
         FROM shift_inventory_snapshots
         WHERE date = ?1
         ORDER BY shift, item_id",
    )?;

    let rows = stmt
        .query_map(params![date], |row| {
            Ok(ShiftInventorySnapshot {
                id: row.get(0)?,
                date: row.get(1)?,
                shift: row.get(2)?,
                item_id: row.get(3)?,
                opening_qty: row.get(4)?,
                closing_qty: row.get(5)?,
                recorded_by: row.get(6)?,
                ts: row.get(7)?,
            })
        })
        ?;

    let mut snapshots = Vec::new();
    for row in rows {
        snapshots.push(row?);
    }
    Ok(snapshots)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(include_str!("../schema.sql")).unwrap();
        conn.execute(
            "INSERT INTO users(username, password_hash, salt, role) VALUES('admin', 'x', 'y', 'admin')",
            [],
        )
        .unwrap();
        conn
    }

    fn seed_shift(conn: &Connection, cartons: f64, waste: f64) {
        conn.execute(
            "INSERT INTO products(id, code, name_ar, cups_per_carton, default_cost_milli) VALUES(1, 'P1', 'كوب 9oz', 1000, 5000)",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO inventory_items(id, code, name_ar, kind, qty_on_hand, avg_cost_milli) VALUES(1, 'RM1', 'بكرة ورق', 'raw', 100, 1000)",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO inventory_items(id, code, name_ar, kind, product_id, qty_on_hand, avg_cost_milli) VALUES(2, 'FG1', 'كوب 9oz', 'finished', 1, 0, 0)",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO bom(product_id, item_id, qty_per_carton, waste_pct, active) VALUES(1, 1, 2.0, 10.0, 1)",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO operations_daily_sheets(id, sheet_no, date, shift, status) VALUES(1, 'PRD-0001', '2026-08-16', 'صباحي', 'Draft')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO production_shift_lines(sheet_id, product_id, cartons_produced, cups_per_carton, waste_cartons) VALUES(1, 1, ?1, 1000, ?2)",
            rusqlite::params![cartons, waste],
        ).unwrap();
    }

    #[test]
    fn complete_shift_consumes_bom_and_posts_finished_goods_at_cost() {
        let conn = test_db();
        seed_shift(&conn, 10.0, 0.0);

        let res = complete_shift_inner(&conn, 1, "operator").unwrap();
        assert!(res.contains("إقفال"));

        // raw consumed = 10 * 2.0 * 1.10 = 22
        let raw_qty: f64 = conn
            .query_row("SELECT qty_on_hand FROM inventory_items WHERE id=1", [], |r| r.get(0))
            .unwrap();
        assert!((raw_qty - 78.0).abs() < 1e-9);

        // finished: 10 cartons @ material cost 22000 -> WAC 2200/carton
        let (fg_qty, fg_avg): (f64, i64) = conn
            .query_row("SELECT qty_on_hand, avg_cost_milli FROM inventory_items WHERE id=2", [], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap();
        assert!((fg_qty - 10.0).abs() < 1e-9);
        assert_eq!(fg_avg, 2200);

        // per-line cost persisted
        let (unit_cost, mat_cost): (i64, i64) = conn
            .query_row("SELECT unit_cost_milli, material_cost_milli FROM production_shift_lines WHERE sheet_id=1", [], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap();
        assert_eq!(unit_cost, 2200);
        assert_eq!(mat_cost, 22000);

        // movements recorded
        let raw_out: f64 = conn
            .query_row("SELECT qty_out FROM inventory_movements WHERE item_id=1 AND mtype='production'", [], |r| r.get(0))
            .unwrap();
        assert!((raw_out - 22.0).abs() < 1e-9);
        let fg_in: f64 = conn
            .query_row("SELECT qty_in FROM inventory_movements WHERE item_id=2 AND mtype='production'", [], |r| r.get(0))
            .unwrap();
        assert!((fg_in - 10.0).abs() < 1e-9);

        let status: String = conn
            .query_row("SELECT status FROM operations_daily_sheets WHERE id=1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(status, "Completed");
    }

    #[test]
    fn complete_shift_rejects_shortage_without_mutating() {
        let conn = test_db();
        seed_shift(&conn, 100.0, 0.0); // needs 220 raw, only 100 available

        let err = complete_shift_inner(&conn, 1, "operator").unwrap_err();
        assert!(err.to_string().contains("غير كافٍ"));

        let raw_qty: f64 = conn
            .query_row("SELECT qty_on_hand FROM inventory_items WHERE id=1", [], |r| r.get(0))
            .unwrap();
        assert!((raw_qty - 100.0).abs() < 1e-9);
        let status: String = conn
            .query_row("SELECT status FROM operations_daily_sheets WHERE id=1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(status, "Draft");
    }

    #[test]
    fn complete_shift_falls_back_to_default_cost_without_bom() {
        let conn = test_db();
        conn.execute(
            "INSERT INTO products(id, code, name_ar, cups_per_carton, default_cost_milli) VALUES(1, 'P1', 'كوب 9oz', 1000, 5000)",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO inventory_items(id, code, name_ar, kind, product_id, qty_on_hand, avg_cost_milli) VALUES(2, 'FG1', 'كوب 9oz', 'finished', 1, 0, 0)",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO operations_daily_sheets(id, sheet_no, date, shift, status) VALUES(1, 'PRD-0001', '2026-08-16', 'صباحي', 'Draft')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO production_shift_lines(sheet_id, product_id, cartons_produced, cups_per_carton, waste_cartons) VALUES(1, 1, 10, 1000, 0)",
            [],
        ).unwrap();

        complete_shift_inner(&conn, 1, "operator").unwrap();
        let (fg_qty, fg_avg): (f64, i64) = conn
            .query_row("SELECT qty_on_hand, avg_cost_milli FROM inventory_items WHERE id=2", [], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap();
        assert!((fg_qty - 10.0).abs() < 1e-9);
        assert_eq!(fg_avg, 5000); // product default cost
    }

    #[test]
    fn complete_shift_twice_is_rejected() {
        let conn = test_db();
        seed_shift(&conn, 10.0, 0.0);
        complete_shift_inner(&conn, 1, "operator").unwrap();
        let err = complete_shift_inner(&conn, 1, "operator").unwrap_err();
        assert!(err.to_string().contains("مسبقاً"));
    }
}
