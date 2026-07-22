use crate::db::DbState;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct SalesForecast {
    pub period_days: i32,
    pub historical_avg_daily: f64,
    pub trend: String,
    pub trend_pct: f64,
    pub forecast_daily: Vec<DailyForecast>,
    pub confidence: f64,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DailyForecast {
    pub date: String,
    pub predicted_amount: f64,
    pub low_estimate: f64,
    pub high_estimate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerRisk {
    pub customer_id: i64,
    pub customer_name: String,
    pub total_invoices: i64,
    pub total_amount: f64,
    pub overdue_amount: f64,
    pub avg_days_to_pay: f64,
    pub risk_score: f64,
    pub risk_level: String,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductionAnalysis {
    pub total_orders: i64,
    pub avg_waste_pct: f64,
    pub best_machine: Option<String>,
    pub worst_machine: Option<String>,
    pub waste_trend: String,
    pub efficiency_score: f64,
    pub recommendations: Vec<String>,
    pub machine_stats: Vec<MachineStat>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MachineStat {
    pub machine: String,
    pub total_cartons: f64,
    pub waste_cartons: f64,
    pub waste_pct: f64,
    pub efficiency: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CostAnalysis {
    pub total_cogs: f64,
    pub total_revenue: f64,
    pub gross_margin_pct: f64,
    pub cost_trend: String,
    pub avg_cost_per_carton: f64,
    pub cost_anomalies: Vec<CostAnomaly>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CostAnomaly {
    pub date: String,
    pub amount: f64,
    pub expected: f64,
    pub deviation_pct: f64,
    pub explanation: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Insight {
    pub category: String,
    pub severity: String,
    pub title: String,
    pub description: String,
    pub metric_value: Option<f64>,
    pub recommendation: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InventoryOptimization {
    pub total_items: i64,
    pub low_stock_items: Vec<LowStockItem>,
    pub dead_stock_items: Vec<DeadStockItem>,
    pub reorder_suggestions: Vec<ReorderSuggestion>,
    pub total_inventory_value: f64,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LowStockItem {
    pub item_id: i64,
    pub name: String,
    pub current_qty: f64,
    pub reorder_level: f64,
    pub days_until_stockout: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeadStockItem {
    pub item_id: i64,
    pub name: String,
    pub qty: f64,
    pub last_movement: Option<String>,
    pub days_inactive: i64,
    pub value_milli: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReorderSuggestion {
    pub item_id: i64,
    pub name: String,
    pub suggested_qty: f64,
    pub urgency: String,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Anomaly {
    pub category: String,
    pub entity_type: String,
    pub entity_id: i64,
    pub description: String,
    pub expected_value: f64,
    pub actual_value: f64,
    pub deviation_pct: f64,
    pub severity: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiReport {
    pub report_type: String,
    pub generated_at: String,
    pub summary: String,
    pub sections: Vec<AiReportSection>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiReportSection {
    pub title: String,
    pub content: String,
    pub metrics: Vec<ReportMetric>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportMetric {
    pub label: String,
    pub value: f64,
    pub unit: String,
    pub trend: Option<String>,
}
fn linear_regression(data: &[(f64, f64)]) -> (f64, f64) {
    if data.len() < 2 {
        return if data.is_empty() {
            (0.0, 0.0)
        } else {
            (0.0, data[0].1)
        };
    }
    let n = data.len() as f64;
    let sum_x: f64 = data.iter().map(|d| d.0).sum();
    let sum_y: f64 = data.iter().map(|d| d.1).sum();
    let sum_xy: f64 = data.iter().map(|d| d.0 * d.1).sum();
    let sum_x2: f64 = data.iter().map(|d| d.0 * d.0).sum();
    let denom = n * sum_x2 - sum_x * sum_x;
    if denom.abs() < 1e-12 {
        (0.0, sum_y / n)
    } else {
        let slope = (n * sum_xy - sum_x * sum_y) / denom;
        let intercept = (sum_y - slope * sum_x) / n;
        (slope, intercept)
    }
}

fn r_squared(data: &[(f64, f64)], slope: f64, intercept: f64) -> f64 {
    if data.len() < 2 {
        return 1.0;
    }
    let mean_y: f64 = data.iter().map(|d| d.1).sum::<f64>() / data.len() as f64;
    let ss_tot: f64 = data.iter().map(|d| (d.1 - mean_y).powi(2)).sum();
    let ss_res: f64 = data
        .iter()
        .map(|d| {
            let predicted = slope * d.0 + intercept;
            (d.1 - predicted).powi(2)
        })
        .sum();
    if ss_tot.abs() < 1e-12 {
        1.0
    } else {
        1.0 - ss_res / ss_tot
    }
}

fn standard_deviation(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let avg = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values.iter().map(|v| (v - avg).powi(2)).sum::<f64>() / values.len() as f64;
    variance.sqrt()
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

fn days_ago_str(days: i64) -> String {
    let now = chrono::Local::now();
    let date = now - chrono::Duration::days(days);
    date.format("%Y-%m-%d").to_string()
}
#[tauri::command]
pub fn ai_sales_forecast(
    state: State<'_, DbState>,
    days: i32,
) -> Result<SalesForecast, String> {
    crate::commands::licensing::require_feature(crate::commands::licensing::FEAT_AI)?;
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let history_days = 180i64;
    let start_date = days_ago_str(history_days);

    let mut stmt = conn
        .prepare(
            "SELECT date, SUM(total_milli) / 1000.0 as total_rial
             FROM sales_invoices
             WHERE date >= ?1 AND status NOT IN ('Void','Draft')
             GROUP BY date
             ORDER BY date ASC",
        )
        .map_err(|e| e.to_string())?;

    let rows: Vec<(String, f64)> = stmt
        .query_map(params![start_date], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    if rows.is_empty() {
        return Ok(SalesForecast {
            period_days: days,
            historical_avg_daily: 0.0,
            trend: "stable".into(),
            trend_pct: 0.0,
            forecast_daily: Vec::new(),
            confidence: 0.0,
            recommendations: vec!["No historical sales data available for forecasting.".into()],
        });
    }

    let total: f64 = rows.iter().map(|r| r.1).sum();
    let historical_avg = total / rows.len() as f64;

    let regression_data: Vec<(f64, f64)> = rows
        .iter()
        .enumerate()
        .map(|(i, r)| (i as f64, r.1))
        .collect();

    let (slope, intercept) = linear_regression(&regression_data);
    let r2 = r_squared(&regression_data, slope, intercept);

    let daily_values: Vec<f64> = rows.iter().map(|r| r.1).collect();
    let std_dev = standard_deviation(&daily_values);

    let trend_pct = if historical_avg > 0.0 {
        (slope * 30.0 / historical_avg) * 100.0
    } else {
        0.0
    };

    let trend = if trend_pct > 5.0 {
        "increasing"
    } else if trend_pct < -5.0 {
        "decreasing"
    } else {
        "stable"
    };

    let last_index = rows.len() as f64;
    let mut forecast_daily = Vec::new();

    for d in 1..=days {
        let x = last_index + d as f64 - 1.0;
        let predicted = (slope * x + intercept).max(0.0);
        let margin = 1.96 * std_dev * (1.0 + d as f64 / rows.len() as f64).sqrt();
        let low = (predicted - margin).max(0.0);
        let high = predicted + margin;

        let forecast_date = {
            let base = chrono::Local::now().date_naive();
            let fut = base + chrono::Duration::days(d as i64);
            fut.format("%Y-%m-%d").to_string()
        };

        forecast_daily.push(DailyForecast {
            date: forecast_date,
            predicted_amount: (predicted * 1000.0).round() / 1000.0,
            low_estimate: (low * 1000.0).round() / 1000.0,
            high_estimate: (high * 1000.0).round() / 1000.0,
        });
    }

    let confidence = (r2 * 100.0).min(100.0).max(0.0);

    let mut recommendations = Vec::new();
    if trend == "increasing" {
        recommendations.push(format!(
            "Sales trending upward by {:.1}% over {} days. Consider scaling production.",
            trend_pct.abs(), days
        ));
    } else if trend == "decreasing" {
        recommendations.push(format!(
            "Sales declining by {:.1}% over {} days. Review marketing and retention.",
            trend_pct.abs(), days
        ));
    } else {
        recommendations.push("Sales are stable. Good time to optimize operations.".into());
    }

    if confidence < 50.0 {
        recommendations.push(
            "Low forecast confidence due to volatile data. Monitor daily closely.".into(),
        );
    }

    let recent_stmt_result = conn.prepare(
        "SELECT SUM(total_milli)/1000.0 FROM sales_invoices
         WHERE date >= date('now','-7 days') AND status NOT IN ('Void','Draft')",
    );
    if let Ok(mut rstmt) = recent_stmt_result {
        if let Ok(recent_total) = rstmt.query_row([], |row| row.get::<_, f64>(0)) {
            let recent_daily = recent_total / 7.0;
            if recent_daily > historical_avg * 1.2 {
                recommendations.push(
                    "Last 7 days above average. Strong demand - ensure inventory levels.".into(),
                );
            } else if recent_daily < historical_avg * 0.8 {
                recommendations.push(
                    "Last 7 days below average. Consider promotional pricing.".into(),
                );
            }
        }
    }

    Ok(SalesForecast {
        period_days: days,
        historical_avg_daily: (historical_avg * 1000.0).round() / 1000.0,
        trend: trend.into(),
        trend_pct: (trend_pct * 100.0).round() / 100.0,
        forecast_daily,
        confidence,
        recommendations,
    })
}
#[tauri::command]
pub fn ai_customer_risk(state: State<'_, DbState>) -> Result<Vec<CustomerRisk>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT c.id, c.name,
                    COUNT(DISTINCT si.inv_no) as inv_count,
                    COALESCE(SUM(si.total_milli), 0) / 1000.0 as total_rial,
                    COALESCE(SUM(CASE WHEN si.status NOT IN ('Void','Draft') AND si.total_milli > si.paid_milli AND si.date < date('now') THEN si.total_milli - si.paid_milli ELSE 0 END), 0) / 1000.0 as overdue_rial
             FROM customers c
             LEFT JOIN sales_invoices si ON si.customer_id = c.id
             GROUP BY c.id, c.name
             HAVING inv_count > 0
             ORDER BY overdue_rial DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows: Vec<(i64, String, i64, f64, f64)> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, f64>(3)?,
                row.get::<_, f64>(4)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let mut results = Vec::new();

    for (id, name, inv_count, total, overdue) in rows {
        let mut score: f64 = 0.0;

        if total > 0.0 {
            let overdue_ratio = overdue / total;
            score += overdue_ratio * 50.0;
        }

        let balance_stmt_result = conn.prepare("SELECT COALESCE(balance_milli, 0) / 1000.0 FROM customers WHERE id = ?1");
        let balance = if let Ok(mut bstmt) = balance_stmt_result {
            bstmt.query_row(params![id], |row| row.get::<_, f64>(0)).unwrap_or(0.0)
        } else {
            0.0
        };
        if balance > 0.0 && total > 0.0 {
            let balance_ratio = balance / total;
            score += balance_ratio * 20.0;
        }

        let avg_days_stmt_result = conn.prepare(
            "SELECT AVG(date_gap) FROM (
                SELECT julianday(date) - julianday(LAG(date) OVER (ORDER BY date)) as date_gap
                FROM sales_invoices
                WHERE customer_id = ?1 AND status NOT IN ('Void','Draft')
            ) WHERE date_gap IS NOT NULL"
        );
        let avg_days = if let Ok(mut dstmt) = avg_days_stmt_result {
            dstmt.query_row(params![id], |row| row.get::<_, f64>(0)).unwrap_or(30.0)
        } else {
            30.0
        };

        if inv_count <= 2 {
            score += 15.0;
        }

        let risk_level = if score >= 70.0 {
            "critical"
        } else if score >= 50.0 {
            "high"
        } else if score >= 25.0 {
            "medium"
        } else {
            "low"
        };

        let mut recommendations = Vec::new();
        if risk_level == "critical" {
            recommendations.push("Immediate collection action required. Consider credit hold.".into());
        } else if risk_level == "high" {
            recommendations.push("Follow up on overdue payments. Consider reducing credit limit.".into());
        } else if risk_level == "medium" {
            recommendations.push("Monitor payment patterns. Send payment reminders.".into());
        } else {
            recommendations.push("Good payment history. Consider offering extended terms.".into());
        }

        results.push(CustomerRisk {
            customer_id: id,
            customer_name: name,
            total_invoices: inv_count,
            total_amount: (total * 1000.0).round() / 1000.0,
            overdue_amount: (overdue * 1000.0).round() / 1000.0,
            avg_days_to_pay: (avg_days * 10.0).round() / 10.0,
            risk_score: (score * 10.0).round() / 10.0,
            risk_level: risk_level.into(),
            recommendations,
        });
    }

    results.sort_by(|a, b| b.risk_score.partial_cmp(&a.risk_score).unwrap_or(std::cmp::Ordering::Equal));
    Ok(results)
}
#[tauri::command]
pub fn ai_production_analysis(state: State<'_, DbState>) -> Result<ProductionAnalysis, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let order_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM production_orders",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT COALESCE(m.code, 'Unknown'),
                    SUM(pl.cartons_good) as good,
                    SUM(pl.cartons_waste) as waste,
                    CASE WHEN SUM(pl.cartons_good) + SUM(pl.cartons_waste) > 0
                         THEN SUM(pl.cartons_waste) * 100.0 / (SUM(pl.cartons_good) + SUM(pl.cartons_waste))
                         ELSE 0 END as waste_pct
             FROM production_lines pl
             JOIN production_orders po ON po.id = pl.order_id
             LEFT JOIN machines m ON m.id = po.machine_id
             GROUP BY m.code
             ORDER BY waste_pct ASC",
        )
        .map_err(|e| e.to_string())?;

    let machine_rows: Vec<(String, f64, f64, f64)> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, f64>(2)?,
                row.get::<_, f64>(3)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let machine_stats: Vec<MachineStat> = machine_rows
        .iter()
        .map(|(machine, good, waste, pct)| {
            let p: f64 = *pct;
            MachineStat {
            machine: machine.clone(),
            total_cartons: good + waste,
            waste_cartons: *waste,
            waste_pct: (p * 100.0).round() / 100.0,
            efficiency: ((100.0 - p) * 100.0).round() / 100.0,
            }
        })
        .collect();

    let avg_waste = if !machine_rows.is_empty() {
        let total_waste: f64 = machine_rows.iter().map(|r| r.2).sum();
        let total_all: f64 = machine_rows.iter().map(|r| r.1 + r.2).sum();
        if total_all > 0.0 {
            total_waste / total_all * 100.0
        } else {
            0.0
        }
    } else {
        0.0
    };

    let best_machine = machine_rows
        .iter()
        .min_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal))
        .map(|r| r.0.clone());

    let worst_machine = machine_rows
        .iter()
        .max_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal))
        .map(|r| r.0.clone());

    let waste_trend_stmt_result = conn.prepare(
        "SELECT date, SUM(pl.cartons_waste) * 100.0 / (SUM(pl.cartons_good) + SUM(pl.cartons_waste))
         FROM production_lines pl
         JOIN production_orders po ON po.id = pl.order_id
         WHERE po.date >= date('now', '-30 days')
         GROUP BY date ORDER BY date",
    );

    let waste_trend = if let Ok(mut wtstmt) = waste_trend_stmt_result {
        let waste_rates: Vec<f64> = wtstmt
            .query_map([], |row| row.get::<_, f64>(1))
            .into_iter()
            .flat_map(|r| r.into_iter())
            .filter_map(|r| r.ok())
            .collect();
        if waste_rates.len() >= 4 {
            let first_half = mean(&waste_rates[..waste_rates.len() / 2]);
            let second_half = mean(&waste_rates[waste_rates.len() / 2..]);
            if second_half > first_half * 1.1 {
                "increasing"
            } else if second_half < first_half * 0.9 {
                "decreasing"
            } else {
                "stable"
            }
        } else {
            "insufficient_data"
        }
    } else {
        "unknown"
    };

    let efficiency_score = if avg_waste > 0.0 {
        let v: f64 = 100.0 - avg_waste;
        v.max(0.0).min(100.0)
    } else {
        0.0
    };

    let mut recommendations = Vec::new();
    if avg_waste > 10.0 {
        recommendations.push(format!(
            "Average waste rate is {:.1}% which is high. Target below 5%.",
            avg_waste
        ));
    }
    if let (Some(best), Some(worst)) = (&best_machine, &worst_machine) {
        if best != worst {
            if let Some(best_stat) = machine_stats.iter().find(|s| &s.machine == best) {
                if let Some(worst_stat) = machine_stats.iter().find(|s| &s.machine == worst) {
                    recommendations.push(format!(
                        "Machine '{}' wastes {:.1}% vs '{}' at {:.1}%. Investigate '{}'.",
                        worst, worst_stat.waste_pct, best, best_stat.waste_pct, worst
                    ));
                }
            }
        }
    }
    if waste_trend == "increasing" {
        recommendations.push("Waste is trending upward this month. Schedule maintenance check.".into());
    }

    Ok(ProductionAnalysis {
        total_orders: order_count,
        avg_waste_pct: (avg_waste * 100.0).round() / 100.0,
        best_machine,
        worst_machine,
        waste_trend: waste_trend.into(),
        efficiency_score: (efficiency_score * 100.0).round() / 100.0,
        recommendations,
        machine_stats,
    })
}
#[tauri::command]
pub fn ai_cost_analysis(state: State<'_, DbState>) -> Result<CostAnalysis, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let total_cogs: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(pl.cartons_good * p.default_cost_milli), 0) / 1000.0
             FROM production_lines pl
             JOIN products p ON p.id = pl.product_id",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let total_revenue: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(total_milli), 0) / 1000.0
             FROM sales_invoices WHERE status NOT IN ('Void','Draft')",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let total_cartons: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(cartons_good), 0) FROM production_lines",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let gross_margin_pct = if total_revenue > 0.0 {
        ((total_revenue - total_cogs) / total_revenue * 100.0 * 100.0).round() / 100.0
    } else {
        0.0
    };

    let avg_cost_per_carton = if total_cartons > 0.0 {
        (total_cogs / total_cartons * 1000.0).round() / 1000.0
    } else {
        0.0
    };

    let cost_trend_stmt = conn.prepare(
        "SELECT date, SUM(pl.cartons_good * p.default_cost_milli) / 1000.0 as daily_cogs
         FROM production_lines pl
         JOIN products p ON p.id = pl.product_id
         JOIN production_orders po ON po.id = pl.order_id
         WHERE po.date >= date('now', '-60 days')
         GROUP BY date ORDER BY date",
    );

    let (cost_trend, cost_anomalies) = if let Ok(mut ctstmt) = cost_trend_stmt {
        let daily_costs: Vec<(String, f64)> = ctstmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
            })
            .into_iter()
            .flat_map(|r| r.into_iter())
            .filter_map(|r| r.ok())
            .collect();

        if daily_costs.len() >= 6 {
            let values: Vec<f64> = daily_costs.iter().map(|c| c.1).collect();
            let avg = mean(&values);
            let std = standard_deviation(&values);
            let first_half = mean(&values[..values.len() / 2]);
            let second_half = mean(&values[values.len() / 2..]);

            let trend = if second_half > first_half * 1.1 {
                "increasing"
            } else if second_half < first_half * 0.9 {
                "decreasing"
            } else {
                "stable"
            };

            let mut anomalies = Vec::new();
            if std > 0.0 {
                for (date, amount) in &daily_costs {
                    let deviation = (amount - avg).abs();
                    if deviation > 2.0 * std {
                        let deviation_pct = (deviation / avg * 100.0 * 100.0).round() / 100.0;
                        let explanation = if *amount > avg {
                            format!("Cost spike: {:.1}% above average of {:.1} Rial", deviation_pct, avg)
                        } else {
                            format!("Cost dip: {:.1}% below average of {:.1} Rial", deviation_pct, avg)
                        };
                        anomalies.push(CostAnomaly {
                            date: date.clone(),
                            amount: (*amount * 1000.0).round() / 1000.0,
                            expected: (avg * 1000.0).round() / 1000.0,
                            deviation_pct,
                            explanation,
                        });
                    }
                }
            }
            (trend.to_string(), anomalies)
        } else {
            ("insufficient_data".to_string(), Vec::new())
        }
    } else {
        ("unknown".to_string(), Vec::new())
    };

    let mut recommendations = Vec::new();
    if gross_margin_pct < 20.0 {
        recommendations.push(format!(
            "Gross margin is low at {:.1}%. Review pricing strategy or reduce costs.",
            gross_margin_pct
        ));
    } else if gross_margin_pct > 40.0 {
        recommendations.push(format!(
            "Strong gross margin at {:.1}%. Consider reinvesting in growth.",
            gross_margin_pct
        ));
    }
    if cost_trend == "increasing" {
        recommendations.push("COGS trending upward. Analyze raw material costs and supplier pricing.".into());
    }
    if !cost_anomalies.is_empty() {
        recommendations.push(format!(
            "{} cost anomalies detected. Review the flagged entries.",
            cost_anomalies.len()
        ));
    }

    Ok(CostAnalysis {
        total_cogs: (total_cogs * 1000.0).round() / 1000.0,
        total_revenue: (total_revenue * 1000.0).round() / 1000.0,
        gross_margin_pct,
        cost_trend,
        avg_cost_per_carton,
        cost_anomalies,
        recommendations,
    })
}
#[tauri::command]
pub fn ai_dashboard_insights(state: State<'_, DbState>) -> Result<Vec<Insight>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut insights = Vec::new();

    let overdue_result = conn.query_row(
        "SELECT COUNT(*), COALESCE(SUM(total_milli - paid_milli), 0) / 1000.0
         FROM sales_invoices WHERE status = 'overdue'",
        [],
        |row| Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?)),
    );
    if let Ok((overdue_count, overdue_total)) = overdue_result {
        if overdue_count > 0 {
            insights.push(Insight {
                category: "finance".into(),
                severity: if overdue_total > 10000.0 { "critical" } else { "warning" }.into(),
                title: "Overdue Invoices".into(),
                description: format!("{} invoices totaling {:.2} Rial are overdue.", overdue_count, overdue_total),
                metric_value: Some(overdue_total),
                recommendation: "Prioritize collection on overdue accounts to improve cash flow.".into(),
            });
        }
    }

    let low_stock_result = conn.prepare(
        "SELECT COUNT(*) FROM inventory_items WHERE qty_on_hand <= reorder_level AND reorder_level > 0"
    );
    if let Ok(mut lsstmt) = low_stock_result {
        if let Ok(low_count) = lsstmt.query_row([], |row| row.get::<_, i64>(0)) {
            if low_count > 0 {
                insights.push(Insight {
                    category: "inventory".into(),
                    severity: if low_count > 5 { "critical" } else { "warning" }.into(),
                    title: "Low Stock Items".into(),
                    description: format!("{} items are at or below reorder level.", low_count),
                    metric_value: Some(low_count as f64),
                    recommendation: "Review reorder points and place purchase orders to avoid stockouts.".into(),
                });
            }
        }
    }

    let waste_result = conn.prepare(
        "SELECT SUM(pl.cartons_waste) * 100.0 / (SUM(pl.cartons_good) + SUM(pl.cartons_waste))
         FROM production_lines pl
         JOIN production_orders po ON po.id = pl.order_id
         WHERE po.date >= date('now', '-30 days')"
    );
    if let Ok(mut wstmt) = waste_result {
        if let Ok(waste_pct) = wstmt.query_row([], |row| row.get::<_, f64>(0)) {
            if waste_pct > 5.0 {
                insights.push(Insight {
                    category: "production".into(),
                    severity: if waste_pct > 10.0 { "critical" } else { "warning" }.into(),
                    title: "High Waste Rate".into(),
                    description: format!("Monthly waste rate is {:.1}%. Industry target is below 5%.", waste_pct),
                    metric_value: Some(waste_pct),
                    recommendation: "Inspect machine calibration, train operators, review raw material quality.".into(),
                });
            }
        }
    }

    let sales_result = conn.prepare(
        "SELECT SUM(total_milli)/1000.0 FROM sales_invoices
         WHERE date >= date('now', '-7 days') AND status NOT IN ('Void','Draft')"
    );
    if let Ok(mut sstmt) = sales_result {
        if let Ok(recent_sales) = sstmt.query_row([], |row| row.get::<_, f64>(0)) {
            let prev_stmt = conn.prepare(
                "SELECT SUM(total_milli)/1000.0 FROM sales_invoices
                 WHERE date >= date('now', '-14 days') AND date < date('now', '-7 days')
                 AND status NOT IN ('Void','Draft')"
            );
            if let Ok(mut pstmt) = prev_stmt {
                if let Ok(prev_sales) = pstmt.query_row([], |row| row.get::<_, f64>(0)) {
                    if prev_sales > 0.0 {
                        let change_pct = (recent_sales - prev_sales) / prev_sales * 100.0;
                        if change_pct.abs() > 15.0 {
                            let severity = if change_pct < -25.0 { "critical" } else { "warning" };
                            insights.push(Insight {
                                category: "sales".into(),
                                severity: severity.into(),
                                title: "Sales Trend Change".into(),
                                description: format!(
                                    "Weekly sales changed by {:.1}% (from {:.1} to {:.1} Rial).",
                                    change_pct, prev_sales, recent_sales
                                ),
                                metric_value: Some(change_pct),
                                recommendation: if change_pct < 0.0 {
                                    "Sales dipped significantly. Review market conditions and customer activity.".into()
                                } else {
                                    "Strong sales growth this week. Ensure production can sustain the pace.".into()
                                },
                            });
                        }
                    }
                }
            }
        }
    }

    let dead_stock_result = conn.prepare(
        "SELECT COUNT(*) FROM inventory_items ii
         WHERE ii.qty_on_hand > 0
         AND NOT EXISTS (
             SELECT 1 FROM inventory_movements im
             WHERE im.item_id = ii.id AND im.ts >= datetime('now', '-90 days')
         )"
    );
    if let Ok(mut dsstmt) = dead_stock_result {
        if let Ok(dead_count) = dsstmt.query_row([], |row| row.get::<_, i64>(0)) {
            if dead_count > 0 {
                insights.push(Insight {
                    category: "inventory".into(),
                    severity: "warning".into(),
                    title: "Dead Stock Detected".into(),
                    description: format!("{} items have had no movement in 90+ days.", dead_count),
                    metric_value: Some(dead_count as f64),
                    recommendation: "Consider discounting or disposing of slow-moving inventory to free up capital.".into(),
                });
            }
        }
    }

    let margin_result = conn.prepare(
        "SELECT
            (SELECT COALESCE(SUM(total_milli), 0) / 1000.0 FROM sales_invoices WHERE date >= date('now', '-30 days') AND status NOT IN ('Void','Draft')) as revenue,
            (SELECT COALESCE(SUM(pl.cartons_good * p.default_cost_milli), 0) / 1000.0 FROM production_lines pl JOIN products p ON p.id = pl.product_id JOIN production_orders po ON po.id = pl.order_id WHERE po.date >= date('now', '-30 days')) as cogs"
    );
    if let Ok(mut mstmt) = margin_result {
        if let Ok((rev, cogs)) = mstmt.query_row([], |row| {
            Ok((row.get::<_, f64>(0)?, row.get::<_, f64>(1)?))
        }) {
            if rev > 0.0 {
                let margin = (rev - cogs) / rev * 100.0;
                if margin < 15.0 {
                    insights.push(Insight {
                        category: "finance".into(),
                        severity: "critical".into(),
                        title: "Low Profit Margin".into(),
                        description: format!("Current gross margin is {:.1}%, below 15% threshold.", margin),
                        metric_value: Some(margin),
                        recommendation: "Urgently review pricing and cost structure. Negotiate with suppliers.".into(),
                    });
                }
            }
        }
    }

    Ok(insights)
}
#[tauri::command]
pub fn ai_inventory_optimization(state: State<'_, DbState>) -> Result<InventoryOptimization, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let total_items: i64 = conn
        .query_row("SELECT COUNT(*) FROM inventory_items", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let total_value: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(qty_on_hand * avg_cost_milli), 0) / 1000.0 FROM inventory_items",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let mut low_stmt = conn
        .prepare(
            "SELECT ii.id, ii.name_ar, ii.qty_on_hand, ii.reorder_level,
                    CASE WHEN (
                        SELECT COALESCE(SUM(im.qty_out), 0) / 30.0
                        FROM inventory_movements im
                        WHERE im.item_id = ii.id AND im.ts >= datetime('now', '-30 days')
                    ) > 0
                    THEN ii.qty_on_hand / (
                        SELECT COALESCE(SUM(im.qty_out), 0) / 30.0
                        FROM inventory_movements im
                        WHERE im.item_id = ii.id AND im.ts >= datetime('now', '-30 days')
                    )
                    ELSE 999 END as days_until_stockout
             FROM inventory_items ii
             WHERE ii.qty_on_hand <= ii.reorder_level AND ii.reorder_level > 0
             ORDER BY days_until_stockout ASC",
        )
        .map_err(|e| e.to_string())?;

    let low_stock_items: Vec<LowStockItem> = low_stmt
        .query_map([], |row| {
            Ok(LowStockItem {
                item_id: row.get::<_, i64>(0)?,
                name: row.get::<_, String>(1)?,
                current_qty: row.get::<_, f64>(2)?,
                reorder_level: row.get::<_, f64>(3)?,
                days_until_stockout: row.get::<_, f64>(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let mut dead_stmt = conn
        .prepare(
            "SELECT ii.id, ii.name_ar, ii.qty_on_hand,
                    (SELECT MAX(im.ts) FROM inventory_movements im WHERE im.item_id = ii.id) as last_move,
                    ii.qty_on_hand * ii.avg_cost_milli / 1000.0 as value_rial
             FROM inventory_items ii
             WHERE ii.qty_on_hand > 0
             AND NOT EXISTS (
                 SELECT 1 FROM inventory_movements im
                 WHERE im.item_id = ii.id AND im.ts >= datetime('now', '-90 days')
             )
             ORDER BY value_rial DESC",
        )
        .map_err(|e| e.to_string())?;

    let dead_stock_items: Vec<DeadStockItem> = dead_stmt
        .query_map([], |row| {
            let last_move: Option<String> = row.get::<_, Option<String>>(3)?;
            let days_inactive = if let Some(ref ls) = last_move {
                let result = conn.query_row(
                    "SELECT julianday('now') - julianday(?1)",
                    params![ls],
                    |row| row.get::<_, f64>(0),
                );
                result.unwrap_or(90.0) as i64
            } else {
                999
            };
            Ok(DeadStockItem {
                item_id: row.get::<_, i64>(0)?,
                name: row.get::<_, String>(1)?,
                qty: row.get::<_, f64>(2)?,
                last_movement: last_move,
                days_inactive,
                value_milli: row.get::<_, f64>(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let mut reorder_suggestions = Vec::new();
    for item in &low_stock_items {
        let _avg_daily_usage = if item.days_until_stockout > 0.0 && item.days_until_stockout < 999.0 {
            item.current_qty / item.days_until_stockout
        } else {
            0.0
        };
        let suggested_qty = (item.reorder_level * 2.0 - item.current_qty).max(item.reorder_level);
        let urgency = if item.days_until_stockout <= 3.0 {
            "urgent"
        } else if item.days_until_stockout <= 7.0 {
            "high"
        } else {
            "medium"
        };
        reorder_suggestions.push(ReorderSuggestion {
            item_id: item.item_id,
            name: item.name.clone(),
            suggested_qty: (suggested_qty * 100.0).round() / 100.0,
            urgency: urgency.into(),
            reason: format!(
                "Current stock {:.1} below reorder {:.1}. ~{:.0} days until stockout.",
                item.current_qty, item.reorder_level, item.days_until_stockout
            ),
        });
    }

    let mut recommendations = Vec::new();
    if !low_stock_items.is_empty() {
        recommendations.push(format!(
            "{} items need restocking. {} are urgent.",
            low_stock_items.len(),
            reorder_suggestions.iter().filter(|s| s.urgency == "urgent").count()
        ));
    }
    if !dead_stock_items.is_empty() {
        let dead_value: f64 = dead_stock_items.iter().map(|d| d.value_milli).sum();
        recommendations.push(format!(
            "{} dead stock items tying up {:.2} Rial. Consider liquidation.",
            dead_stock_items.len(), dead_value
        ));
    }
    if total_value > 0.0 && dead_stock_items.len() as f64 / total_items as f64 > 0.1 {
        recommendations.push("Dead stock exceeds 10% of inventory. Review procurement strategy.".into());
    }

    Ok(InventoryOptimization {
        total_items,
        low_stock_items,
        dead_stock_items,
        reorder_suggestions,
        total_inventory_value: (total_value * 1000.0).round() / 1000.0,
        recommendations,
    })
}

#[tauri::command]
pub fn ai_anomaly_detection(state: State<'_, DbState>) -> Result<Vec<Anomaly>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut anomalies = Vec::new();

    let invoice_stats: (f64, f64) = conn
        .query_row(
            "SELECT AVG(total_milli)/1000.0, 0 FROM sales_invoices WHERE status NOT IN ('Void','Draft')",
            [],
            |row| Ok((row.get::<_, f64>(0).unwrap_or(0.0), 0.0)),
        )
        .unwrap_or((0.0, 0.0));
    let avg_invoice = invoice_stats.0;

    let mut stmt = conn
        .prepare(
            "SELECT id, inv_no, total_milli/1000.0 as amount, date
             FROM sales_invoices WHERE status NOT IN ('Void','Draft') AND total_milli > 0",
        )
        .map_err(|e| e.to_string())?;
    let invoices: Vec<(i64, String, f64, String)> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, f64>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let amounts: Vec<f64> = invoices.iter().map(|i| i.2).collect();
    let std = standard_deviation(&amounts);

    if avg_invoice > 0.0 && std > 0.0 {
        for (id, inv_no, amount, _date) in &invoices {
            let deviation = (amount - avg_invoice).abs();
            if deviation > 2.0 * std {
                anomalies.push(Anomaly {
                    category: "sales".into(),
                    entity_type: "invoice".into(),
                    entity_id: *id,
                    description: format!("Invoice {} amount {:.2} deviates from average {:.2}", inv_no, amount, avg_invoice),
                    expected_value: (avg_invoice * 1000.0).round() / 1000.0,
                    actual_value: (*amount * 1000.0).round() / 1000.0,
                    deviation_pct: (deviation / avg_invoice * 100.0 * 100.0).round() / 100.0,
                    severity: if deviation > 3.0 * std { "critical" } else { "warning" }.into(),
                });
            }
        }
    }

    let mut prod_stmt = conn
        .prepare(
            "SELECT po.id, COALESCE(m.code, 'Machine'), SUM(pl.cartons_waste),
                    SUM(pl.cartons_good) + SUM(pl.cartons_waste) as total
             FROM production_lines pl
             JOIN production_orders po ON po.id = pl.order_id
             LEFT JOIN machines m ON m.id = po.machine_id
             GROUP BY po.id, m.code
             HAVING total > 0",
        )
        .map_err(|e| e.to_string())?;
    let prod_rows: Vec<(i64, String, f64, f64)> = prod_stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, f64>(2)?,
                row.get::<_, f64>(3)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let waste_rates: Vec<f64> = prod_rows
        .iter()
        .map(|r| if r.3 > 0.0 { r.2 / r.3 * 100.0 } else { 0.0 })
        .collect();
    let avg_waste = mean(&waste_rates);
    let waste_std = standard_deviation(&waste_rates);

    for (id, machine, waste, total) in &prod_rows {
        let waste_pct = if *total > 0.0 { *waste / *total * 100.0 } else { 0.0 };
        if waste_std > 0.0 && (waste_pct - avg_waste).abs() > 2.0 * waste_std {
            anomalies.push(Anomaly {
                category: "production".into(),
                entity_type: "production_order".into(),
                entity_id: *id,
                description: format!("Order on {} waste rate {:.1}% deviates from average {:.1}%", machine, waste_pct, avg_waste),
                expected_value: (avg_waste * 100.0).round() / 100.0,
                actual_value: (waste_pct * 100.0).round() / 100.0,
                deviation_pct: ((waste_pct - avg_waste).abs() / avg_waste.max(1.0) * 100.0 * 100.0).round() / 100.0,
                severity: if waste_pct > avg_waste + 3.0 * waste_std { "critical" } else { "warning" }.into(),
            });
        }
    }

    anomalies.sort_by(|a, b| b.deviation_pct.partial_cmp(&a.deviation_pct).unwrap_or(std::cmp::Ordering::Equal));
    Ok(anomalies)
}

#[tauri::command]
pub fn ai_generate_report(
    state: State<'_, DbState>,
    report_type: String,
) -> Result<AiReport, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let mut sections = Vec::new();
    let mut summary_parts = Vec::new();

    if report_type == "sales" || report_type == "all" {
        let total_revenue: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(total_milli), 0) / 1000.0 FROM sales_invoices WHERE status NOT IN ('Void','Draft')",
                [], |row| row.get(0),
            )
            .unwrap_or(0.0);
        let invoice_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sales_invoices WHERE status NOT IN ('Void','Draft')",
                [], |row| row.get(0),
            )
            .unwrap_or(0);

        let mut top_stmt = conn
            .prepare(
                "SELECT c.name, SUM(si.total_milli)/1000.0 as total
                 FROM sales_invoices si JOIN customers c ON si.customer_id = c.id
                 WHERE si.status NOT IN ('Void','Draft')
                 GROUP BY c.name ORDER BY total DESC LIMIT 5",
            )
            .map_err(|e| e.to_string())?;
        let top_customers: Vec<(String, f64)> = top_stmt
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?)))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        sections.push(AiReportSection {
            title: "Sales Summary".into(),
            content: format!("Total revenue: {:.2} Rial from {} invoices.", total_revenue, invoice_count),
            metrics: vec![
                ReportMetric { label: "Total Revenue".into(), value: total_revenue, unit: "Rial".into(), trend: None },
                ReportMetric { label: "Invoice Count".into(), value: invoice_count as f64, unit: "invoices".into(), trend: None },
            ],
            recommendations: vec![
                format!("Top customer: {} ({:.2} Rial)", top_customers.first().map(|c| c.0.as_str()).unwrap_or("N/A"), top_customers.first().map(|c| c.1).unwrap_or(0.0)),
            ],
        });
        summary_parts.push(format!("Revenue: {:.2} Rial", total_revenue));
    }

    if report_type == "production" || report_type == "all" {
        let total_cartons: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(cartons_good), 0) FROM production_lines",
                [], |row| row.get(0),
            )
            .unwrap_or(0.0);
        let total_waste: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(cartons_waste), 0) FROM production_lines",
                [], |row| row.get(0),
            )
            .unwrap_or(0.0);
        let waste_pct = if total_cartons + total_waste > 0.0 {
            total_waste / (total_cartons + total_waste) * 100.0
        } else {
            0.0
        };

        sections.push(AiReportSection {
            title: "Production Summary".into(),
            content: format!("Total cartons: {:.0}, waste: {:.0} ({:.1}%)", total_cartons, total_waste, waste_pct),
            metrics: vec![
                ReportMetric { label: "Good Cartons".into(), value: total_cartons, unit: "cartons".into(), trend: None },
                ReportMetric { label: "Waste Rate".into(), value: waste_pct, unit: "%".into(), trend: None },
            ],
            recommendations: if waste_pct > 5.0 {
                vec!["Waste rate is above 5% target. Review production processes.".into()]
            } else {
                vec!["Waste rate is within acceptable limits.".into()]
            },
        });
        summary_parts.push(format!("Waste: {:.1}%", waste_pct));
    }

    if report_type == "inventory" || report_type == "all" {
        let total_items: i64 = conn
            .query_row("SELECT COUNT(*) FROM inventory_items", [], |row| row.get(0))
            .unwrap_or(0);
        let low_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM inventory_items WHERE qty_on_hand <= reorder_level AND reorder_level > 0",
                [], |row| row.get(0),
            )
            .unwrap_or(0);
        let total_value: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(qty_on_hand * avg_cost_milli), 0) / 1000.0 FROM inventory_items",
                [], |row| row.get(0),
            )
            .unwrap_or(0.0);

        sections.push(AiReportSection {
            title: "Inventory Summary".into(),
            content: format!("{} items, {} low stock, total value {:.2} Rial", total_items, low_count, total_value),
            metrics: vec![
                ReportMetric { label: "Total Items".into(), value: total_items as f64, unit: "items".into(), trend: None },
                ReportMetric { label: "Low Stock".into(), value: low_count as f64, unit: "items".into(), trend: None },
                ReportMetric { label: "Total Value".into(), value: total_value, unit: "Rial".into(), trend: None },
            ],
            recommendations: if low_count > 0 {
                vec![format!("{} items need restocking.", low_count)]
            } else {
                vec!["All inventory levels are adequate.".into()]
            },
        });
        summary_parts.push(format!("Inventory value: {:.2} Rial", total_value));
    }

    if report_type == "finance" || report_type == "all" {
        let total_revenue: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(total_milli), 0) / 1000.0 FROM sales_invoices WHERE status NOT IN ('Void','Draft')",
                [], |row| row.get(0),
            )
            .unwrap_or(0.0);
        let total_cogs: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(pl.cartons_good * p.default_cost_milli), 0) / 1000.0
                 FROM production_lines pl JOIN products p ON p.id = pl.product_id",
                [], |row| row.get(0),
            )
            .unwrap_or(0.0);
        let margin = if total_revenue > 0.0 {
            (total_revenue - total_cogs) / total_revenue * 100.0
        } else {
            0.0
        };

        sections.push(AiReportSection {
            title: "Financial Summary".into(),
            content: format!("Revenue: {:.2} Rial, COGS: {:.2} Rial, Margin: {:.1}%", total_revenue, total_cogs, margin),
            metrics: vec![
                ReportMetric { label: "Revenue".into(), value: total_revenue, unit: "Rial".into(), trend: None },
                ReportMetric { label: "COGS".into(), value: total_cogs, unit: "Rial".into(), trend: None },
                ReportMetric { label: "Gross Margin".into(), value: margin, unit: "%".into(), trend: None },
            ],
            recommendations: if margin < 20.0 {
                vec!["Margin below 20%. Review pricing and cost structure.".into()]
            } else {
                vec!["Healthy margin. Consider growth investments.".into()]
            },
        });
        summary_parts.push(format!("Margin: {:.1}%", margin));
    }

    Ok(AiReport {
        report_type,
        generated_at: now,
        summary: summary_parts.join(" | "),
        sections,
    })
}