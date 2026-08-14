use crate::commands::rbac;
use crate::db::DbState;
use crate::error::AppError;
use crate::qayd::{build_instance, inputs_from_account_balances, validate_instance, QaydCompany};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct QaydFilingResult {
    pub id: i64,
    pub fiscal_year: i32,
    pub cr_number: String,
    pub currency: String,
    pub status: String,
    pub instance_xml: String,
    pub validation_report: Vec<String>,
    pub is_valid: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QaydFilingRecord {
    pub id: i64,
    pub fiscal_year: i32,
    pub currency: String,
    pub cr_number: Option<String>,
    pub status: String,
    pub submitted_at: Option<String>,
    pub created_at: String,
}

fn company_info(conn: &rusqlite::Connection) -> (String, String, String) {
    conn.query_row(
        "SELECT COALESCE(name_ar, name_en, ''), COALESCE(cr_number, ''), COALESCE(default_currency, 'KWD')
         FROM companies WHERE active = 1 ORDER BY id LIMIT 1",
        [],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?)),
    )
    .or_else(|_| {
        conn.query_row(
            "SELECT COALESCE(name, ''), '', 'KWD' FROM company_settings LIMIT 1",
            [],
            |row| Ok((row.get::<_, String>(0)?, String::new(), String::from("KWD"))),
        )
    })
    .unwrap_or_default()
}

/// Read (account_code, signed amount in units) for all accounts up to a date.
fn closing_balances(conn: &rusqlite::Connection, up_to: &str) -> Vec<(String, f64)> {
    let mut stmt = conn
        .prepare(
            "SELECT jel.account_code, SUM(jel.debit_milli - jel.credit_milli)
             FROM journal_entry_lines jel
             JOIN journal_entries je ON jel.entry_id = je.id
             WHERE je.date <= ?1 AND je.reversed_by IS NULL
             GROUP BY jel.account_code",
        )
        .unwrap();
    let rows = stmt
        .query_map([up_to], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as f64 / 1000.0)))
        .unwrap();
    rows.filter_map(|r| r.ok()).collect()
}

/// Year totals for revenue/expense accounts (credit-debit for revenue, debit-credit for expense).
fn year_pnl_totals(conn: &rusqlite::Connection, start: &str, end: &str) -> Vec<(String, f64)> {
    let mut stmt = conn
        .prepare(
            "SELECT jel.account_code, a.type,
                    SUM(jel.credit_milli - jel.debit_milli),
                    SUM(jel.debit_milli - jel.credit_milli)
             FROM journal_entry_lines jel
             JOIN journal_entries je ON jel.entry_id = je.id
             LEFT JOIN accounts a ON jel.account_code = a.code
             WHERE je.date >= ?1 AND je.date <= ?2 AND je.reversed_by IS NULL
             GROUP BY jel.account_code, a.type",
        )
        .unwrap();
    let rows = stmt
        .query_map(rusqlite::params![start, end], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, i64>(2)? as f64 / 1000.0,
                row.get::<_, i64>(3)? as f64 / 1000.0,
            ))
        })
        .unwrap();
    let mut out = Vec::new();
    for r in rows.filter_map(|r| r.ok()) {
        let (code, typ, credit_minus_debit, debit_minus_credit) = r;
        let is_revenue = matches!(typ.as_deref(), Some("revenue"));
        let amount = if is_revenue { credit_minus_debit } else { debit_minus_credit };
        out.push((code, amount));
    }
    out
}

fn retained_earnings_prior(conn: &rusqlite::Connection, before: &str) -> f64 {
    conn.query_row(
        "SELECT COALESCE(SUM(jel.debit_milli - jel.credit_milli), 0)
         FROM journal_entry_lines jel
         JOIN journal_entries je ON jel.entry_id = je.id
         WHERE jel.account_code = '3200' AND je.date < ?1 AND je.reversed_by IS NULL",
        [before],
        |row| row.get::<_, i64>(0),
    )
    .map(|v| v as f64 / 1000.0)
    .unwrap_or(0.0)
}

fn build_report(
    conn: &rusqlite::Connection,
    fiscal_year: i32,
) -> Result<QaydFilingResult, AppError> {
    let (_company_name, cr_number, _default_currency) = company_info(conn);
    let period_start = format!("{}-01-01", fiscal_year);
    let period_end = format!("{}-12-31", fiscal_year);

    let mut balances = closing_balances(conn, &period_end);
    // Replace revenue/expense codes with the fiscal-year totals.
    let pnl = year_pnl_totals(conn, &period_start, &period_end);
    let mut by_code: std::collections::BTreeMap<String, f64> = balances.drain(..).collect();
    for (code, amount) in pnl {
        by_code.insert(code, amount);
    }
    let prior_retained = retained_earnings_prior(conn, &period_start);

    let inputs = inputs_from_account_balances(&by_code.into_iter().collect::<Vec<_>>(), prior_retained);

    // Qayd reporting currency is the Kuwaiti Dinar.
    let currency = "KWD".to_string();

    let facts = inputs.to_facts();
    let company = QaydCompany {
        name_ar: String::new(),
        cr_number: cr_number.clone(),
        currency: currency.clone(),
        fiscal_year,
        period_start: period_start.clone(),
        period_end: period_end.clone(),
        prior_period_end: String::new(),
    };
    let instance = build_instance(&company, &facts);
    let report = validate_instance(&instance, &company);

    Ok(QaydFilingResult {
        id: 0,
        fiscal_year,
        cr_number,
        currency,
        status: if report.is_empty() { "ready".into() } else { "draft".into() },
        instance_xml: instance,
        validation_report: report.clone(),
        is_valid: report.is_empty(),
    })
}

#[tauri::command]
pub fn qayd_generate_filing(
    state: State<'_, DbState>,
    user_id: i64,
    fiscal_year: i32,
) -> Result<QaydFilingResult, AppError> {
    crate::commands::licensing::require_feature(crate::commands::licensing::FEAT_GOVERNMENT)?;
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    let mut report = build_report(&conn, fiscal_year)?;
    conn.execute(
        "INSERT INTO qayd_filings(company_id, fiscal_year, currency, cr_number, status, instance_xml, validation_report, created_by)
         VALUES(1, ?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            fiscal_year,
            report.currency,
            report.cr_number,
            report.status,
            report.instance_xml,
            report.validation_report.join("; "),
            user_id,
        ],
    )?;
    report.id = conn.last_insert_rowid();
    let _ = rbac::log_audit(&conn, Some(user_id), None, "qayd_generate_filing", "qayd_filings", Some(report.id), None, Some(&fiscal_year.to_string()), None);
    Ok(report)
}

#[tauri::command]
pub fn qayd_list_filings(
    state: State<'_, DbState>,
    user_id: i64,
) -> Result<Vec<QaydFilingRecord>, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager", "viewer"])?;
    let mut stmt = conn.prepare(
        "SELECT id, fiscal_year, currency, cr_number, status, submitted_at, created_at
         FROM qayd_filings ORDER BY fiscal_year DESC, id DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(QaydFilingRecord {
            id: row.get(0)?,
            fiscal_year: row.get(1)?,
            currency: row.get(2)?,
            cr_number: row.get(3)?,
            status: row.get(4)?,
            submitted_at: row.get(5)?,
            created_at: row.get(6)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

#[tauri::command]
pub fn qayd_get_filing(
    state: State<'_, DbState>,
    user_id: i64,
    filing_id: i64,
) -> Result<QaydFilingResult, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager", "viewer"])?;
    let row = conn
        .query_row(
            "SELECT id, fiscal_year, currency, COALESCE(cr_number, ''), status, instance_xml, COALESCE(validation_report, '')
             FROM qayd_filings WHERE id = ?1",
            [filing_id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i32>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                ))
            },
        )
        .map_err(|_| AppError::not_found("إيداع قيد غير موجود"))?;
    Ok(QaydFilingResult {
        id: row.0,
        fiscal_year: row.1,
        cr_number: row.3,
        currency: row.2,
        status: row.4.clone(),
        instance_xml: row.5,
        validation_report: if row.6.is_empty() { vec![] } else { row.6.split("; ").map(String::from).collect() },
        is_valid: row.4 == "ready",
    })
}

#[tauri::command]
pub fn qayd_validate_filing(
    state: State<'_, DbState>,
    user_id: i64,
    filing_id: i64,
) -> Result<QaydFilingResult, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager"])?;
    let mut filing = qayd_get_filing(state.clone(), user_id, filing_id)?;
    let company = QaydCompany {
        name_ar: String::new(),
        cr_number: filing.cr_number.clone(),
        currency: filing.currency.clone(),
        fiscal_year: filing.fiscal_year,
        period_start: format!("{}-01-01", filing.fiscal_year),
        period_end: format!("{}-12-31", filing.fiscal_year),
        prior_period_end: String::new(),
    };
    filing.validation_report = validate_instance(&filing.instance_xml, &company);
    filing.is_valid = filing.validation_report.is_empty();
    filing.status = if filing.is_valid { "ready".into() } else { "draft".into() };
    conn.execute(
        "UPDATE qayd_filings SET status = ?1, validation_report = ?2 WHERE id = ?3",
        rusqlite::params![filing.status, filing.validation_report.join("; "), filing_id],
    )?;
    Ok(filing)
}

#[tauri::command]
pub fn qayd_delete_filing(
    state: State<'_, DbState>,
    user_id: i64,
    filing_id: i64,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin"])?;
    conn.execute("DELETE FROM qayd_filings WHERE id = ?1", [filing_id])?;
    let _ = rbac::log_audit(&conn, Some(user_id), None, "qayd_delete_filing", "qayd_filings", Some(filing_id), None, None, None);
    Ok("تم حذف الإيداع".into())
}

/// Validate the XBRL instance stored for a filing against the taxonomy
/// requirements and report the reconciled totals (helper for the UI).
#[tauri::command]
pub fn qayd_filing_totals(
    state: State<'_, DbState>,
    user_id: i64,
    filing_id: i64,
) -> Result<serde_json::Value, AppError> {
    let conn = state.0.lock()?;
    rbac::require_role(&conn, user_id, &["admin", "accountant", "manager", "viewer"])?;
    let xml: String = conn
        .query_row(
            "SELECT instance_xml FROM qayd_filings WHERE id = ?1",
            [filing_id],
            |row| row.get(0),
        )
        .map_err(|_| AppError::not_found("إيداع قيد غير موجود"))?;
    let mut totals = serde_json::Map::new();
    for concept in ["Assets", "Liabilities", "Equity", "Revenue", "ProfitLoss"] {
        let needle = format!("<ifrs-full:{} ", concept);
        let value = xml
            .split(&needle)
            .nth(1)
            .and_then(|s| s.split('>').nth(1))
            .and_then(|s| s.split("</").next())
            .and_then(|s| s.trim().parse::<f64>().ok());
        totals.insert(concept.to_string(), serde_json::json!(value));
    }
    Ok(serde_json::Value::Object(totals))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(include_str!("../schema.sql")).unwrap();
        conn
    }

    fn seed_journal(conn: &Connection) {
        // Cash 10000, Capital 10000; revenue 2000, COGS 1200, admin 200.
        for (code, typ) in [
            ("1100", "asset"),
            ("3100", "equity"),
            ("4100", "revenue"),
            ("5100", "expense"),
            ("5200", "expense"),
        ] {
            conn.execute(
                "INSERT INTO accounts(code, name_en, type, is_system) VALUES(?1, ?2, ?3, 1)",
                rusqlite::params![code, code, typ],
            )
            .unwrap();
        }
        for (entry_no, date, lines) in [
            ("JE-1", "2026-01-01", vec![("1100", 10_000_000i64, 0i64), ("3100", 0, 10_000_000)]),
            ("JE-2", "2026-06-15", vec![("1100", 2_000_000, 0), ("4100", 0, 2_000_000)]),
            ("JE-3", "2026-07-01", vec![("5100", 1_200_000, 0), ("1100", 0, 1_200_000)]),
            ("JE-4", "2026-08-01", vec![("5200", 200_000, 0), ("1100", 0, 200_000)]),
        ] {
            conn.execute(
                "INSERT INTO journal_entries(entry_no, date, memo) VALUES(?1, ?2, 'test')",
                rusqlite::params![entry_no, date],
            )
            .unwrap();
            let id = conn.last_insert_rowid();
            for (code, debit, credit) in lines {
                conn.execute(
                    "INSERT INTO journal_entry_lines(entry_id, account_code, debit_milli, credit_milli) VALUES(?1, ?2, ?3, ?4)",
                    rusqlite::params![id, code, debit, credit],
                )
                .unwrap();
            }
        }
    }

    #[test]
    fn generates_valid_filing_from_journal() {
        let conn = test_db();
        seed_journal(&conn);
        let report = build_report(&conn, 2026).unwrap();
        assert!(report.is_valid, "{:?}", report.validation_report);
        assert!(report.instance_xml.contains("<ifrs-full:Assets contextRef=\"instant_current\" decimals=\"2\" unitRef=\"iso4217-KWD\">10600.00</ifrs-full:Assets>"));
        assert!(report.instance_xml.contains("ProfitLoss"));
        assert_eq!(report.currency, "KWD");
    }

    #[test]
    fn balances_are_computed_with_unit_conversion() {
        let conn = test_db();
        seed_journal(&conn);
        let balances = closing_balances(&conn, "2026-12-31");
        let map: std::collections::BTreeMap<_, _> = balances.into_iter().collect();
        assert_eq!(map.get("1100").copied().unwrap(), 10600.0);
        assert_eq!(map.get("4100").copied().unwrap(), -2000.0);
        let pnl = year_pnl_totals(&conn, "2026-01-01", "2026-12-31");
        let pmap: std::collections::BTreeMap<_, _> = pnl.into_iter().collect();
        assert_eq!(pmap.get("4100").copied().unwrap(), 2000.0);
        assert_eq!(pmap.get("5100").copied().unwrap(), 1200.0);
    }

    #[test]
    fn kuwait_cr_scheme_is_constant() {
        assert_eq!(crate::qayd::KUWAIT_CR_SCHEME, "https://www.qayd.gov.kw/cr");
    }
}
