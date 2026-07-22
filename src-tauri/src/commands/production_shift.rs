use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::rbac;
use crate::db::DbState;

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
    pub ts: String,
    pub recorded_by: Option<String>,
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
pub fn get_shift_sheet(state: State<'_, DbState>, date: String, shift: String) -> Result<i64, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

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
        .map_err(|e| e.to_string())?;
    let sheet_no = format!("PRD-{:04}", seq);

    conn.execute(
        "INSERT INTO operations_daily_sheets (sheet_no, date, shift, start_time, status, created_at)
         VALUES (?1, ?2, ?3, datetime('now'), 'Draft', datetime('now'))",
        params![sheet_no, date, shift],
    )
    .map_err(|e| e.to_string())?;

    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn record_production(
    state: State<'_, DbState>,
    sheet_id: i64,
    product_id: i64,
    customer_brand: Option<String>,
    cartons_produced: f64,
    cups_per_carton: Option<i64>,
    waste_cartons: Option<f64>,
    recorded_by: Option<String>,
) -> Result<ShiftLine, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let cpc = cups_per_carton.unwrap_or(1000);
    let waste = waste_cartons.unwrap_or(0.0);

    conn.execute(
        "INSERT INTO production_shift_lines (sheet_id, product_id, customer_brand, cartons_produced, cups_per_carton, waste_cartons, recorded_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![sheet_id, product_id, customer_brand, cartons_produced, cpc, waste, recorded_by],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();

    let line = conn.query_row(
        "SELECT psl.id, psl.sheet_id, psl.product_id, COALESCE(p.name_ar, p.name_en, '') as product_name,
                psl.customer_brand, psl.cartons_produced, psl.cups_per_carton, psl.waste_cartons, psl.ts, psl.recorded_by
         FROM production_shift_lines psl
         LEFT JOIN products p ON p.id = psl.product_id
         WHERE psl.id = ?1",
        params![id],
        row_to_shift_line,
    ).map_err(|e| e.to_string())?;

    update_sheet_totals(&conn, sheet_id)?;

    let _ = rbac::log_audit(&conn, None, None, "record_production", "production_shift_lines", None, None, None, None);

    Ok(line)
}

fn update_sheet_totals(conn: &rusqlite::Connection, sheet_id: i64) -> Result<(), String> {
    conn.execute(
        "UPDATE operations_daily_sheets SET
            cartons_produced = (SELECT COALESCE(SUM(cartons_produced), 0) FROM production_shift_lines WHERE sheet_id = ?1),
            total_cups = (SELECT COALESCE(SUM(cartons_produced * cups_per_carton), 0) FROM production_shift_lines WHERE sheet_id = ?1)
         WHERE id = ?1",
        params![sheet_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_shift_lines(state: State<'_, DbState>, sheet_id: i64) -> Result<Vec<ShiftLine>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT psl.id, psl.sheet_id, psl.product_id, COALESCE(p.name_ar, p.name_en, ''),
                    psl.customer_brand, psl.cartons_produced, psl.cups_per_carton, psl.waste_cartons, psl.ts, psl.recorded_by
             FROM production_shift_lines psl
             LEFT JOIN products p ON p.id = psl.product_id
             WHERE psl.sheet_id = ?1
             ORDER BY psl.ts DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt.query_map(params![sheet_id], row_to_shift_line).map_err(|e| e.to_string())?;
    let mut lines = Vec::new();
    for row in rows {
        lines.push(row.map_err(|e| e.to_string())?);
    }
    Ok(lines)
}

#[tauri::command]
pub fn complete_shift(state: State<'_, DbState>, sheet_id: i64, completed_by: String) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let status: String = conn.query_row(
        "SELECT status FROM operations_daily_sheets WHERE id = ?1",
        params![sheet_id],
        |row| row.get(0),
    ).map_err(|_| "الوريiodية غير موجودة".to_string())?;

    if status != "Draft" {
        return Err("لا يمكن إقفال وردية تم إقفالها مسبقاً".to_string());
    }

    conn.execute(
        "UPDATE operations_daily_sheets SET status = 'Completed', completed_by = ?1, completed_at = datetime('now')
         WHERE id = ?2",
        params![completed_by, sheet_id],
    )
    .map_err(|e| e.to_string())?;

    let lines: Vec<(i64, f64)> = {
        let mut stmt = conn
            .prepare(
                "SELECT psl.product_id, psl.cartons_produced
                 FROM production_shift_lines psl WHERE psl.sheet_id = ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt.query_map(params![sheet_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?))
        }).map_err(|e| e.to_string())?;
        let mut v = Vec::new();
        for r in rows {
            v.push(r.map_err(|e| e.to_string())?);
        }
        v
    };

    for (product_id, cartons) in &lines {
        conn.execute(
            "UPDATE inventory_items SET qty_on_hand = qty_on_hand + ?1 WHERE product_id = ?2 AND kind = 'finished'",
            params![cartons, product_id],
        )
        .map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO inventory_movements (ts, item_id, mtype, qty_in, ref_type, ref_id, notes)
             SELECT datetime('now'), ii.id, 'production', ?1, 'production_shift', ?2, 'إنتاج من الوردية'
             FROM inventory_items ii WHERE ii.product_id = ?3 AND ii.kind = 'finished'",
            params![cartons, sheet_id, product_id],
        )
        .map_err(|e| e.to_string())?;
    }

    let _ = rbac::log_audit(&conn, None, None, "complete_shift", "operations_daily_sheets", Some(sheet_id), None, None, None);

    Ok("تم إقفال الوردية وتحديث المخزون".to_string())
}

#[tauri::command]
pub fn update_production_line(
    state: State<'_, DbState>,
    line_id: i64,
    cartons_produced: f64,
    waste_cartons: Option<f64>,
) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let sheet_id: i64 = conn.query_row(
        "SELECT psl.sheet_id FROM production_shift_lines psl
         JOIN operations_daily_sheets ods ON ods.id = psl.sheet_id
         WHERE psl.id = ?1 AND ods.status = 'Draft'",
        params![line_id],
        |row| row.get(0),
    ).map_err(|_| "السطر غير موجود أو الوردية مقفلة".to_string())?;

    conn.execute(
        "UPDATE production_shift_lines SET cartons_produced = ?1, waste_cartons = COALESCE(?2, waste_cartons) WHERE id = ?3",
        params![cartons_produced, waste_cartons, line_id],
    )
    .map_err(|e| e.to_string())?;

    update_sheet_totals(&conn, sheet_id)?;

    Ok("تم التحديث".to_string())
}

#[tauri::command]
pub fn delete_production_line(state: State<'_, DbState>, line_id: i64) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let sheet_id: i64 = conn.query_row(
        "SELECT psl.sheet_id FROM production_shift_lines psl
         JOIN operations_daily_sheets ods ON ods.id = psl.sheet_id
         WHERE psl.id = ?1 AND ods.status = 'Draft'",
        params![line_id],
        |row| row.get(0),
    ).map_err(|_| "السطر غير موجود أو الوردية مقفلة".to_string())?;

    conn.execute("DELETE FROM production_shift_lines WHERE id = ?1", params![line_id])
        .map_err(|e| e.to_string())?;

    update_sheet_totals(&conn, sheet_id)?;

    let _ = rbac::log_audit(&conn, None, None, "delete_production_line", "production_shift_lines", Some(line_id), None, None, None);

    Ok("تم الحذف".to_string())
}

#[tauri::command]
pub fn get_live_dashboard(state: State<'_, DbState>) -> Result<LiveProductionSummary, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

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
    ).map_err(|e| e.to_string())?;

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
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let mut recent = conn.prepare(
        "SELECT psl.id, psl.sheet_id, psl.product_id, COALESCE(p.name_ar, p.name_en, ''),
                psl.customer_brand, psl.cartons_produced, psl.cups_per_carton, psl.waste_cartons, psl.ts, psl.recorded_by
         FROM production_shift_lines psl
         JOIN operations_daily_sheets ods ON ods.id = psl.sheet_id
         LEFT JOIN products p ON p.id = psl.product_id
         WHERE ods.date = ?1
         ORDER BY psl.ts DESC LIMIT 20",
    ).map_err(|e| e.to_string())?;

    let recent_entries: Vec<ShiftLine> = recent
        .query_map(params![today], row_to_shift_line)
        .map_err(|e| e.to_string())?
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
) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

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
    ).map_err(|e| e.to_string())?;

    let rows: Vec<(f64, i64, f64, String, Option<String>)> = stmt.query_map(params![sheet_id], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
    }).map_err(|e| e.to_string())?
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
    raw.push_str(&esc(0x1b)); raw.push_str("@"); // Initialize
    raw.push_str(&esc(0x1b)); raw.push_str("E"); raw.push_str("\x01"); // Bold on
    raw.push_str("           ProMax ERP\n");
    raw.push_str(&esc(0x1b)); raw.push_str("E"); raw.push_str("\x00"); // Bold off
    raw.push_str("----------------------------------------\n");

    for line in &lines {
        raw.push_str(line);
        raw.push_str("\n");
    }

    raw.push_str(&esc(0x1b)); raw.push_str("m"); // Cut

    let temp_file = std::env::temp_dir().join("promax_shift_report.bin");
    std::fs::write(&temp_file, raw.as_bytes()).map_err(|e| format!("Failed to write temp file: {}", e))?;

    let printer = printer_name.unwrap_or_default();
    let ps = format!(
        r#"Get-Content -Path "{0}" -Encoding Byte | Out-Printer -Name "{1}" -Wait"#,
        temp_file.to_str().unwrap_or(""),
        printer
    );

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps])
        .output()
        .map_err(|e| format!("Failed to print: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Print error: {}", stderr));
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
        ts: row.get(8)?,
        recorded_by: row.get(9)?,
    })
}
