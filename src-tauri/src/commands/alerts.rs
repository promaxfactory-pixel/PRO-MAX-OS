use crate::db::DbState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct Alert {
    pub alert_type: String,
    pub severity: String,
    pub entity_type: String,
    pub entity_id: i64,
    pub title: String,
    pub message: String,
    pub action_suggestion: String,
    pub value: f64,
    pub threshold: f64,
}

fn check_low_stock(conn: &rusqlite::Connection) -> Vec<Alert> {
    let mut alerts = Vec::new();
    let mut stmt = match conn.prepare(
        "SELECT id, code, name_ar, name_en, qty_on_hand, reorder_level, uom FROM inventory_items WHERE active=1 AND reorder_level > 0 AND qty_on_hand <= reorder_level ORDER BY (qty_on_hand / reorder_level) ASC"
    ) {
        Ok(s) => s,
        Err(_) => return alerts,
    };
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, f64>(4)?,
            row.get::<_, f64>(5)?,
            row.get::<_, Option<String>>(6)?,
        ))
    });
    if let Ok(rows) = rows {
        for r in rows {
            if let Ok((id, code, name_ar, name_en, qty, reorder, uom)) = r {
                let name = name_ar.or(name_en).unwrap_or_else(|| code.unwrap_or_default());
                let severity = if qty <= 0.0 { "critical".to_string() } else { "warning".to_string() };
                let deficit = (reorder * 2.0) - qty;
                let uom_str = uom.unwrap_or_default();
                alerts.push(Alert {
                    alert_type: "low_stock".to_string(),
                    severity,
                    entity_type: "inventory_item".to_string(),
                    entity_id: id,
                    title: format!("مخزون منخفض: {}", name),
                    message: format!("الكمية: {:.0} {}, إعادة الطلب: {:.0} {}", qty, uom_str, reorder, uom_str),
                    action_suggestion: format!("اطلب {:.0} {}", deficit.max(0.0), uom_str),
                    value: qty,
                    threshold: reorder,
                });
            }
        }
    }
    alerts
}

fn check_overdue_invoices(conn: &rusqlite::Connection) -> Vec<Alert> {
    let mut alerts = Vec::new();
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let mut stmt = match conn.prepare(
        "SELECT si.id, si.inv_no, si.date, si.total_milli, si.paid_milli, c.name FROM sales_invoices si JOIN customers c ON c.id=si.customer_id WHERE si.status IN ('Issued','Partially Paid','Posted') AND si.total_milli > si.paid_milli AND si.date <= ? ORDER BY si.date ASC"
    ) {
        Ok(s) => s,
        Err(_) => return alerts,
    };
    let rows = stmt.query_map([&today], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, String>(5)?,
        ))
    });
    if let Ok(rows) = rows {
        for r in rows {
            if let Ok((id, inv_no, date, total, paid, cname)) = r {
                let remaining = total - paid;
                if remaining <= 0 { continue; }
                let inv_date = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d").unwrap_or_default();
                let today_date = chrono::NaiveDate::parse_from_str(&today, "%Y-%m-%d").unwrap_or_default();
                let days = (today_date - inv_date).num_days();
                let severity = if days > 90 { "critical".to_string() } else { "warning".to_string() };
                alerts.push(Alert {
                    alert_type: "overdue_invoice".to_string(),
                    severity,
                    entity_type: "sales_invoice".to_string(),
                    entity_id: id,
                    title: format!("فاتورة متأخرة: {}", inv_no.unwrap_or_default()),
                    message: format!("العميل: {}, المتبقي: {:.3} ر.ع, {} يوم", cname, remaining as f64 / 1000.0, days),
                    action_suggestion: format!("تواصل مع {}", cname),
                    value: remaining as f64,
                    threshold: 0.0,
                });
            }
        }
    }
    alerts
}

fn check_production_delays(conn: &rusqlite::Connection) -> Vec<Alert> {
    let mut alerts = Vec::new();
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let stale_date = (chrono::Utc::now() - chrono::Duration::days(7)).format("%Y-%m-%d").to_string();
    
    let mut stmt = match conn.prepare(
        "SELECT id, date, downtime_minutes, run_minutes FROM production_orders WHERE (status='Draft' AND date <= ?) OR (status='Approved' AND downtime_minutes > 120 AND downtime_minutes > run_minutes) ORDER BY date ASC"
    ) {
        Ok(s) => s,
        Err(_) => return alerts,
    };
    let rows = stmt.query_map([&stale_date], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, i64>(3)?,
        ))
    });
    if let Ok(rows) = rows {
        for r in rows {
            if let Ok((id, date, downtime, run)) = r {
                let prod_date = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d").unwrap_or_default();
                let today_date = chrono::NaiveDate::parse_from_str(&today, "%Y-%m-%d").unwrap_or_default();
                let days = (today_date - prod_date).num_days();
                if days > 7 {
                    alerts.push(Alert {
                        alert_type: "production_delay".to_string(),
                        severity: "warning".to_string(),
                        entity_type: "production_order".to_string(),
                        entity_id: id,
                        title: format!("أمر إنتاج متأخر: #{}", id),
                        message: format!("أنشئ قبل {} أيام، لا يزال مسودة", days),
                        action_suggestion: "راجع واعتمد الأمر".to_string(),
                        value: days as f64,
                        threshold: 7.0,
                    });
                } else if downtime > run && downtime > 120 {
                    alerts.push(Alert {
                        alert_type: "production_delay".to_string(),
                        severity: "critical".to_string(),
                        entity_type: "production_order".to_string(),
                        entity_id: id,
                        title: format!("توقف كبير: #{}", id),
                        message: format!("التوقف {} دقيقة > التشغيل {} دقيقة", downtime, run),
                        action_suggestion: "تحقق من سبب التوقف".to_string(),
                        value: downtime as f64,
                        threshold: 120.0,
                    });
                }
            }
        }
    }
    alerts
}

fn check_quality_issues(conn: &rusqlite::Connection) -> Vec<Alert> {
    let mut alerts = Vec::new();
    
    let mut stmt = match conn.prepare(
        "SELECT id, inspector, status, notes FROM quality_inspections WHERE status != 'Passed' ORDER BY date DESC LIMIT 20"
    ) {
        Ok(s) => s,
        Err(_) => return alerts,
    };
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, Option<String>>(3)?,
        ))
    });
    if let Ok(rows) = rows {
        for r in rows {
            if let Ok((id, insp_no, status, notes)) = r {
                let severity = if status == "Failed" { "critical".to_string() } else { "warning".to_string() };
                alerts.push(Alert {
                    alert_type: "quality_issue".to_string(),
                    severity,
                    entity_type: "quality_inspection".to_string(),
                    entity_id: id,
                    title: format!("فحص جودة: {}", insp_no.unwrap_or_else(|| format!("#{}", id))),
                    message: notes.unwrap_or_else(|| format!("الحالة: {}", status)),
                    action_suggestion: "راجع واتخذ إجراء تصحيحي".to_string(),
                    value: 0.0,
                    threshold: 0.0,
                });
            }
        }
    }
    alerts
}

#[tauri::command]
pub fn get_all_alerts(state: State<'_, DbState>) -> Result<Vec<Alert>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut all_alerts: Vec<Alert> = Vec::new();
    all_alerts.extend(check_low_stock(&conn));
    all_alerts.extend(check_overdue_invoices(&conn));
    all_alerts.extend(check_production_delays(&conn));
    all_alerts.extend(check_quality_issues(&conn));
    
    all_alerts.sort_by(|a, b| {
        let ord_a = match a.severity.as_str() { "critical" => 0, "warning" => 1, _ => 2 };
        let ord_b = match b.severity.as_str() { "critical" => 0, "warning" => 1, _ => 2 };
        ord_a.cmp(&ord_b)
    });
    
    Ok(all_alerts)
}
