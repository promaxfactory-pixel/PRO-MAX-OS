use crate::db::DbState;
use crate::error::AppError;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct GovEntity {
    pub id: i64,
    pub code: String,
    pub name_ar: String,
    pub name_en: Option<String>,
    pub category: String,
    pub website: Option<String>,
    pub api_endpoint: Option<String>,
    pub active: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct GovIntegration {
    pub id: i64,
    pub entity_id: i64,
    pub config_key: String,
    pub config_value: Option<String>,
    pub encrypted: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GovSubmission {
    pub id: i64,
    pub entity_id: i64,
    pub entity_name: String,
    pub report_template_id: Option<i64>,
    pub status: String,
    pub reference_no: Option<String>,
    pub submitted_at: Option<String>,
    pub submitted_by: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GovDashboard {
    pub entities_count: i64,
    pub active_integrations: i64,
    pub pending_submissions: i64,
    pub successful_submissions: i64,
    pub failed_submissions: i64,
    pub entities: Vec<GovEntity>,
    pub recent_submissions: Vec<GovSubmission>,
}

#[tauri::command]
pub fn gov_get_dashboard(state: State<'_, DbState>) -> Result<GovDashboard, AppError> {
    crate::commands::licensing::require_feature(crate::commands::licensing::FEAT_GOVERNMENT)?;
    let conn = state.0.lock()?;

    let entities_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM gov_entities", [], |row| row.get(0))
        .unwrap_or(0);
    let active_integrations: i64 = conn
        .query_row("SELECT COUNT(*) FROM gov_integrations", [], |row| row.get(0))
        .unwrap_or(0);
    let pending_submissions: i64 = conn
        .query_row("SELECT COUNT(*) FROM gov_submissions WHERE status='pending'", [], |row| row.get(0))
        .unwrap_or(0);
    let successful_submissions: i64 = conn
        .query_row("SELECT COUNT(*) FROM gov_submissions WHERE status='submitted'", [], |row| row.get(0))
        .unwrap_or(0);
    let failed_submissions: i64 = conn
        .query_row("SELECT COUNT(*) FROM gov_submissions WHERE status='failed'", [], |row| row.get(0))
        .unwrap_or(0);

    let mut stmt = conn
        .prepare("SELECT id, code, name_ar, name_en, category, website, api_endpoint, active, notes FROM gov_entities ORDER BY name_ar")
        ?;
    let entities: Vec<GovEntity> = stmt
        .query_map([], |row| {
            Ok(GovEntity {
                id: row.get(0)?,
                code: row.get(1)?,
                name_ar: row.get(2)?,
                name_en: row.get(3)?,
                category: row.get(4)?,
                website: row.get(5)?,
                api_endpoint: row.get(6)?,
                active: row.get::<_, i64>(7)? != 0,
                notes: row.get(8)?,
            })
        })
        ?
        .filter_map(|r| r.ok())
        .collect();

    let mut sub_stmt = conn
        .prepare(
            "SELECT s.id, s.entity_id, e.name_ar, s.report_template_id, s.status,
                    s.reference_no, s.submitted_at, s.submitted_by, s.created_at
             FROM gov_submissions s JOIN gov_entities e ON s.entity_id = e.id
             ORDER BY s.created_at DESC LIMIT 10",
        )
        ?;
    let recent_submissions: Vec<GovSubmission> = sub_stmt
        .query_map([], |row| {
            Ok(GovSubmission {
                id: row.get(0)?,
                entity_id: row.get(1)?,
                entity_name: row.get(2)?,
                report_template_id: row.get(3)?,
                status: row.get(4)?,
                reference_no: row.get(5)?,
                submitted_at: row.get(6)?,
                submitted_by: row.get(7)?,
                created_at: row.get(8)?,
            })
        })
        ?
        .filter_map(|r| r.ok())
        .collect();

    Ok(GovDashboard {
        entities_count,
        active_integrations,
        pending_submissions,
        successful_submissions,
        failed_submissions,
        entities,
        recent_submissions,
    })
}

#[tauri::command]
pub fn gov_list_entities(state: State<'_, DbState>, category: Option<String>) -> Result<Vec<GovEntity>, AppError> {
    let conn = state.0.lock()?;
    match category {
        Some(ref cat) => {
            let mut stmt = conn
                .prepare("SELECT id, code, name_ar, name_en, category, website, api_endpoint, active, notes FROM gov_entities WHERE category=?1 ORDER BY name_ar")
                ?;
            let rows = stmt
                .query_map(params![cat], |row| {
                    Ok(GovEntity {
                        id: row.get(0)?,
                        code: row.get(1)?,
                        name_ar: row.get(2)?,
                        name_en: row.get(3)?,
                        category: row.get(4)?,
                        website: row.get(5)?,
                        api_endpoint: row.get(6)?,
                        active: row.get::<_, i64>(7)? != 0,
                        notes: row.get(8)?,
                    })
                })
                ?
                .filter_map(|r| r.ok())
                .collect();
            Ok(rows)
        }
        None => {
            let mut stmt = conn
                .prepare("SELECT id, code, name_ar, name_en, category, website, api_endpoint, active, notes FROM gov_entities ORDER BY name_ar")
                ?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(GovEntity {
                        id: row.get(0)?,
                        code: row.get(1)?,
                        name_ar: row.get(2)?,
                        name_en: row.get(3)?,
                        category: row.get(4)?,
                        website: row.get(5)?,
                        api_endpoint: row.get(6)?,
                        active: row.get::<_, i64>(7)? != 0,
                        notes: row.get(8)?,
                    })
                })
                ?
                .filter_map(|r| r.ok())
                .collect();
            Ok(rows)
        }
    }
}

#[tauri::command]
pub fn gov_list_submissions(state: State<'_, DbState>, entity_id: Option<i64>) -> Result<Vec<GovSubmission>, AppError> {
    let conn = state.0.lock()?;
    let _ = match entity_id {
        Some(eid) => {
            let mut stmt = conn
                .prepare(
                    "SELECT s.id, s.entity_id, e.name_ar, s.report_template_id, s.status,
                            s.reference_no, s.submitted_at, s.submitted_by, s.created_at
                     FROM gov_submissions s JOIN gov_entities e ON s.entity_id = e.id
                     WHERE s.entity_id=?1 ORDER BY s.created_at DESC LIMIT 50",
                )
                ?;
            let rows = stmt
                .query_map(params![eid], |row| {
                    Ok(GovSubmission {
                        id: row.get(0)?,
                        entity_id: row.get(1)?,
                        entity_name: row.get(2)?,
                        report_template_id: row.get(3)?,
                        status: row.get(4)?,
                        reference_no: row.get(5)?,
                        submitted_at: row.get(6)?,
                        submitted_by: row.get(7)?,
                        created_at: row.get(8)?,
                    })
                })
                ?
                .filter_map(|r| r.ok())
                .collect();
            return Ok(rows);
        }
        None => {
            let mut stmt = conn
                .prepare(
                    "SELECT s.id, s.entity_id, e.name_ar, s.report_template_id, s.status,
                            s.reference_no, s.submitted_at, s.submitted_by, s.created_at
                     FROM gov_submissions s JOIN gov_entities e ON s.entity_id = e.id
                     ORDER BY s.created_at DESC LIMIT 50",
                )
                ?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(GovSubmission {
                        id: row.get(0)?,
                        entity_id: row.get(1)?,
                        entity_name: row.get(2)?,
                        report_template_id: row.get(3)?,
                        status: row.get(4)?,
                        reference_no: row.get(5)?,
                        submitted_at: row.get(6)?,
                        submitted_by: row.get(7)?,
                        created_at: row.get(8)?,
                    })
                })
                ?
                .filter_map(|r| r.ok())
                .collect();
            return Ok(rows);
        }
    };
}

#[tauri::command]
pub fn gov_get_employee_doc_status(state: State<'_, DbState>) -> Result<Value, AppError> {
    let conn = state.0.lock()?;
    let now = chrono::Local::now().format("%Y-%m-%d").to_string();

    let expiring_passports: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM employees WHERE passport_expiry IS NOT NULL AND passport_expiry <= date('now', '+90 days') AND active=1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let expiring_residence: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM employees WHERE residence_expiry IS NOT NULL AND residence_expiry <= date('now', '+90 days') AND active=1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let expiring_visa: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM employees WHERE visa_expiry IS NOT NULL AND visa_expiry <= date('now', '+90 days') AND active=1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let expiring_work_permits: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM employees WHERE workpermit_expiry IS NOT NULL AND workpermit_expiry <= date('now', '+90 days') AND active=1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let expiring_renewals: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM renewals WHERE status='active' AND expiry_date IS NOT NULL AND expiry_date <= date('now', '+90 days')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(json!({
        "expiring_passports": expiring_passports,
        "expiring_residence": expiring_residence,
        "expiring_visa": expiring_visa,
        "expiring_work_permits": expiring_work_permits,
        "expiring_renewals": expiring_renewals,
        "as_of_date": now,
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GovSubmissionRecord {
    pub entity_code: String,
    pub submission_type: String,
    pub payload_json: String,
}

#[tauri::command]
pub fn gov_submit_report(
    state: State<'_, DbState>,
    input: GovSubmissionRecord,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;

    let entity_id: i64 = conn
        .query_row(
            "SELECT id FROM gov_entities WHERE code=?1",
            params![input.entity_code],
            |row| row.get(0),
        )
        .map_err(|_| AppError::not_found(format!("Government entity '{}' not found", input.entity_code)))?;

    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    conn.execute(
        "INSERT INTO gov_submissions(entity_id, status, payload, submitted_at, submitted_by)
         VALUES(?1, 'submitted', ?2, ?3, 'system')",
        params![entity_id, input.payload_json, now],
    )
    ?;

    Ok(format!("Report submitted to entity #{} successfully at {}", entity_id, now))
}