use crate::commands::rbac;
use crate::db::DbState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct Employee {
    pub id: i64,
    pub code: Option<String>,
    pub name: String,
    pub nationality: Option<String>,
    pub job: Option<String>,
    pub salary_milli: i64,
    pub allowances_milli: i64,
    pub phone: Option<String>,
    pub passport_no: Option<String>,
    pub passport_expiry: Option<String>,
    pub residence_expiry: Option<String>,
    pub visa_expiry: Option<String>,
    pub joining_date: Option<String>,
    pub active: i64,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEmployeeInput {
    pub name: String,
    pub nationality: Option<String>,
    pub job: Option<String>,
    pub salary_milli: Option<i64>,
    pub allowances_milli: Option<i64>,
    pub phone: Option<String>,
    pub passport_no: Option<String>,
    pub passport_expiry: Option<String>,
    pub residence_expiry: Option<String>,
    pub visa_expiry: Option<String>,
    pub joining_date: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEmployeeInput {
    pub name: Option<String>,
    pub nationality: Option<String>,
    pub job: Option<String>,
    pub salary_milli: Option<i64>,
    pub allowances_milli: Option<i64>,
    pub phone: Option<String>,
    pub passport_no: Option<String>,
    pub passport_expiry: Option<String>,
    pub residence_expiry: Option<String>,
    pub visa_expiry: Option<String>,
    pub joining_date: Option<String>,
    pub active: Option<i64>,
    pub notes: Option<String>,
}

#[tauri::command]
pub fn list_employees(state: State<'_, DbState>) -> Result<Vec<Employee>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, code, name, nationality, job, salary_milli, allowances_milli, phone, passport_no, passport_expiry, residence_expiry, visa_expiry, joining_date, active, notes FROM employees WHERE active=1 ORDER BY name",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Employee {
                id: row.get(0)?,
                code: row.get(1)?,
                name: row.get(2)?,
                nationality: row.get(3)?,
                job: row.get(4)?,
                salary_milli: row.get(5)?,
                allowances_milli: row.get(6)?,
                phone: row.get(7)?,
                passport_no: row.get(8)?,
                passport_expiry: row.get(9)?,
                residence_expiry: row.get(10)?,
                visa_expiry: row.get(11)?,
                joining_date: row.get(12)?,
                active: row.get(13)?,
                notes: row.get(14)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_employee(state: State<'_, DbState>, id: i64) -> Result<Employee, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT id, code, name, nationality, job, salary_milli, allowances_milli, phone, passport_no, passport_expiry, residence_expiry, visa_expiry, joining_date, active, notes FROM employees WHERE id=?",
        [id],
        |row| {
            Ok(Employee {
                id: row.get(0)?,
                code: row.get(1)?,
                name: row.get(2)?,
                nationality: row.get(3)?,
                job: row.get(4)?,
                salary_milli: row.get(5)?,
                allowances_milli: row.get(6)?,
                phone: row.get(7)?,
                passport_no: row.get(8)?,
                passport_expiry: row.get(9)?,
                residence_expiry: row.get(10)?,
                visa_expiry: row.get(11)?,
                joining_date: row.get(12)?,
                active: row.get(13)?,
                notes: row.get(14)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_employee(
    state: State<'_, DbState>,
    input: CreateEmployeeInput,
) -> Result<i64, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let year = chrono::Utc::now().format("%Y").to_string();

    let seq: i64 = conn
        .query_row(
            "SELECT COALESCE(last_number,0)+1 FROM doc_sequences WHERE doc_type='EMP' AND year=?",
            [&year],
            |r| r.get(0),
        )
        .unwrap_or(1);
    let _ = conn.execute(
        "INSERT INTO doc_sequences(doc_type, year, last_number) VALUES('EMP',?,?) ON CONFLICT(doc_type, year) DO UPDATE SET last_number=excluded.last_number",
        rusqlite::params![year, seq],
    );
    let emp_code = format!("EMP-{}-{:04}", year, seq);

    conn.execute(
        "INSERT INTO employees(code, name, nationality, job, salary_milli, allowances_milli, phone, passport_no, passport_expiry, residence_expiry, visa_expiry, joining_date, notes) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?)",
        rusqlite::params![
            emp_code,
            input.name,
            input.nationality,
            input.job,
            input.salary_milli.unwrap_or(0),
            input.allowances_milli.unwrap_or(0),
            input.phone,
            input.passport_no,
            input.passport_expiry,
            input.residence_expiry,
            input.visa_expiry,
            input.joining_date,
            input.notes,
        ],
    )
    .map_err(|e| e.to_string())?;
    let emp_id = conn.last_insert_rowid();
    let _ = rbac::log_audit(&conn, None, None, "create_employee", "employees", Some(emp_id), None, Some(&emp_code), None);
    Ok(emp_id)
}

#[tauri::command]
pub fn update_employee(
    state: State<'_, DbState>,
    id: i64,
    input: UpdateEmployeeInput,
) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut sets = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(v) = &input.name {
        sets.push("name=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.nationality {
        sets.push("nationality=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.job {
        sets.push("job=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = input.salary_milli {
        sets.push("salary_milli=?");
        params.push(Box::new(v));
    }
    if let Some(v) = input.allowances_milli {
        sets.push("allowances_milli=?");
        params.push(Box::new(v));
    }
    if let Some(v) = &input.phone {
        sets.push("phone=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.passport_no {
        sets.push("passport_no=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.passport_expiry {
        sets.push("passport_expiry=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.residence_expiry {
        sets.push("residence_expiry=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.visa_expiry {
        sets.push("visa_expiry=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.joining_date {
        sets.push("joining_date=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = input.active {
        sets.push("active=?");
        params.push(Box::new(v));
    }
    if let Some(v) = &input.notes {
        sets.push("notes=?");
        params.push(Box::new(v.clone()));
    }

    if sets.is_empty() {
        return Err("No changes provided".to_string());
    }

    params.push(Box::new(id));
    let sql = format!("UPDATE employees SET {} WHERE id=?", sets.join(", "));
    conn.execute(&sql, rusqlite::params_from_iter(params.iter()))
        .map_err(|e| e.to_string())?;
    let _ = rbac::log_audit(&conn, None, None, "update_employee", "employees", Some(id), None, None, None);
    Ok("Updated successfully".to_string())
}
