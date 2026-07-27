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
    pub workpermit_expiry: Option<String>,
    pub insurance_expiry: Option<String>,
    pub contract_end: Option<String>,
    pub id_number: Option<String>,
    pub date_of_birth: Option<String>,
    pub gender: Option<String>,
    pub marital_status: Option<String>,
    pub email: Option<String>,
    pub bank_name: Option<String>,
    pub bank_account_no: Option<String>,
    pub basic_salary_milli: i64,
    pub housing_allowance_milli: i64,
    pub transport_allowance_milli: i64,
    pub food_allowance_milli: i64,
    pub other_allowances_milli: i64,
    pub overtime_rate_milli: f64,
    pub insurance_policy_no: Option<String>,
    pub insurance_premium_milli: i64,
    pub ticket_allowance_milli: i64,
    pub sponsor_name: Option<String>,
    pub sponsor_id: Option<String>,
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
    pub workpermit_expiry: Option<String>,
    pub insurance_expiry: Option<String>,
    pub contract_end: Option<String>,
    pub id_number: Option<String>,
    pub date_of_birth: Option<String>,
    pub gender: Option<String>,
    pub marital_status: Option<String>,
    pub email: Option<String>,
    pub bank_name: Option<String>,
    pub bank_account_no: Option<String>,
    pub basic_salary_milli: Option<i64>,
    pub housing_allowance_milli: Option<i64>,
    pub transport_allowance_milli: Option<i64>,
    pub food_allowance_milli: Option<i64>,
    pub other_allowances_milli: Option<i64>,
    pub overtime_rate_milli: Option<f64>,
    pub insurance_policy_no: Option<String>,
    pub insurance_premium_milli: Option<i64>,
    pub ticket_allowance_milli: Option<i64>,
    pub sponsor_name: Option<String>,
    pub sponsor_id: Option<String>,
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
    pub workpermit_expiry: Option<String>,
    pub insurance_expiry: Option<String>,
    pub contract_end: Option<String>,
    pub id_number: Option<String>,
    pub date_of_birth: Option<String>,
    pub gender: Option<String>,
    pub marital_status: Option<String>,
    pub email: Option<String>,
    pub bank_name: Option<String>,
    pub bank_account_no: Option<String>,
    pub basic_salary_milli: Option<i64>,
    pub housing_allowance_milli: Option<i64>,
    pub transport_allowance_milli: Option<i64>,
    pub food_allowance_milli: Option<i64>,
    pub other_allowances_milli: Option<i64>,
    pub overtime_rate_milli: Option<f64>,
    pub insurance_policy_no: Option<String>,
    pub insurance_premium_milli: Option<i64>,
    pub ticket_allowance_milli: Option<i64>,
    pub sponsor_name: Option<String>,
    pub sponsor_id: Option<String>,
    pub joining_date: Option<String>,
    pub active: Option<i64>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmployeeListItem {
    pub id: i64,
    pub name: String,
    pub code: Option<String>,
    pub job: Option<String>,
}

const EMPLOYEE_COLUMNS: &str = "id, code, name, nationality, job, salary_milli, allowances_milli, phone, passport_no, passport_expiry, residence_expiry, visa_expiry, workpermit_expiry, insurance_expiry, contract_end, id_number, date_of_birth, gender, marital_status, email, bank_name, bank_account_no, basic_salary_milli, housing_allowance_milli, transport_allowance_milli, food_allowance_milli, other_allowances_milli, overtime_rate_milli, insurance_policy_no, insurance_premium_milli, ticket_allowance_milli, sponsor_name, sponsor_id, joining_date, active, notes";

#[tauri::command]
pub fn list_employees(state: State<'_, DbState>) -> Result<Vec<Employee>, String> {
    crate::commands::licensing::require_feature(crate::commands::licensing::FEAT_HR)?;
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let sql = format!("SELECT {} FROM employees WHERE active=1 ORDER BY name", EMPLOYEE_COLUMNS);
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
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
                workpermit_expiry: row.get(12)?,
                insurance_expiry: row.get(13)?,
                contract_end: row.get(14)?,
                id_number: row.get(15)?,
                date_of_birth: row.get(16)?,
                gender: row.get(17)?,
                marital_status: row.get(18)?,
                email: row.get(19)?,
                bank_name: row.get(20)?,
                bank_account_no: row.get(21)?,
                basic_salary_milli: row.get(22)?,
                housing_allowance_milli: row.get(23)?,
                transport_allowance_milli: row.get(24)?,
                food_allowance_milli: row.get(25)?,
                other_allowances_milli: row.get(26)?,
                overtime_rate_milli: row.get(27)?,
                insurance_policy_no: row.get(28)?,
                insurance_premium_milli: row.get(29)?,
                ticket_allowance_milli: row.get(30)?,
                sponsor_name: row.get(31)?,
                sponsor_id: row.get(32)?,
                joining_date: row.get(33)?,
                active: row.get(34)?,
                notes: row.get(35)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_employee(state: State<'_, DbState>, id: i64) -> Result<Employee, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let sql = format!("SELECT {} FROM employees WHERE id=?", EMPLOYEE_COLUMNS);
    conn.query_row(&sql, [id], |row| {
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
            workpermit_expiry: row.get(12)?,
            insurance_expiry: row.get(13)?,
            contract_end: row.get(14)?,
            id_number: row.get(15)?,
            date_of_birth: row.get(16)?,
            gender: row.get(17)?,
            marital_status: row.get(18)?,
            email: row.get(19)?,
            bank_name: row.get(20)?,
            bank_account_no: row.get(21)?,
            basic_salary_milli: row.get(22)?,
            housing_allowance_milli: row.get(23)?,
            transport_allowance_milli: row.get(24)?,
            food_allowance_milli: row.get(25)?,
            other_allowances_milli: row.get(26)?,
            overtime_rate_milli: row.get(27)?,
            insurance_policy_no: row.get(28)?,
            insurance_premium_milli: row.get(29)?,
            ticket_allowance_milli: row.get(30)?,
            sponsor_name: row.get(31)?,
            sponsor_id: row.get(32)?,
            joining_date: row.get(33)?,
            active: row.get(34)?,
            notes: row.get(35)?,
        })
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_employees_for_production(
    state: State<'_, DbState>,
) -> Result<Vec<EmployeeListItem>, String> {
    crate::commands::licensing::require_feature(crate::commands::licensing::FEAT_HR)?;
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, name, code, job FROM employees WHERE active=1 ORDER BY name")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(EmployeeListItem {
                id: row.get(0)?,
                name: row.get(1)?,
                code: row.get(2)?,
                job: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
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
    conn.execute(
        "INSERT INTO doc_sequences(doc_type, year, last_number) VALUES('EMP',?,?) ON CONFLICT(doc_type, year) DO UPDATE SET last_number=excluded.last_number",
        rusqlite::params![year, seq],
    ).map_err(|e| format!("Failed to increment employee sequence: {}", e))?;
    let emp_code = format!("EMP-{}-{:04}", year, seq);

    conn.execute(
        "INSERT INTO employees(code, name, nationality, job, salary_milli, allowances_milli, phone, passport_no, passport_expiry, residence_expiry, visa_expiry, workpermit_expiry, insurance_expiry, contract_end, id_number, date_of_birth, gender, marital_status, email, bank_name, bank_account_no, basic_salary_milli, housing_allowance_milli, transport_allowance_milli, food_allowance_milli, other_allowances_milli, overtime_rate_milli, insurance_policy_no, insurance_premium_milli, ticket_allowance_milli, sponsor_name, sponsor_id, joining_date, notes) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
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
            input.workpermit_expiry,
            input.insurance_expiry,
            input.contract_end,
            input.id_number,
            input.date_of_birth,
            input.gender,
            input.marital_status,
            input.email,
            input.bank_name,
            input.bank_account_no,
            input.basic_salary_milli.unwrap_or(0),
            input.housing_allowance_milli.unwrap_or(0),
            input.transport_allowance_milli.unwrap_or(0),
            input.food_allowance_milli.unwrap_or(0),
            input.other_allowances_milli.unwrap_or(0),
            input.overtime_rate_milli.unwrap_or(0.0),
            input.insurance_policy_no,
            input.insurance_premium_milli.unwrap_or(0),
            input.ticket_allowance_milli.unwrap_or(0),
            input.sponsor_name,
            input.sponsor_id,
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
    if let Some(v) = &input.workpermit_expiry {
        sets.push("workpermit_expiry=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.insurance_expiry {
        sets.push("insurance_expiry=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.contract_end {
        sets.push("contract_end=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.id_number {
        sets.push("id_number=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.date_of_birth {
        sets.push("date_of_birth=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.gender {
        sets.push("gender=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.marital_status {
        sets.push("marital_status=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.email {
        sets.push("email=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.bank_name {
        sets.push("bank_name=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.bank_account_no {
        sets.push("bank_account_no=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = input.basic_salary_milli {
        sets.push("basic_salary_milli=?");
        params.push(Box::new(v));
    }
    if let Some(v) = input.housing_allowance_milli {
        sets.push("housing_allowance_milli=?");
        params.push(Box::new(v));
    }
    if let Some(v) = input.transport_allowance_milli {
        sets.push("transport_allowance_milli=?");
        params.push(Box::new(v));
    }
    if let Some(v) = input.food_allowance_milli {
        sets.push("food_allowance_milli=?");
        params.push(Box::new(v));
    }
    if let Some(v) = input.other_allowances_milli {
        sets.push("other_allowances_milli=?");
        params.push(Box::new(v));
    }
    if let Some(v) = input.overtime_rate_milli {
        sets.push("overtime_rate_milli=?");
        params.push(Box::new(v));
    }
    if let Some(v) = &input.insurance_policy_no {
        sets.push("insurance_policy_no=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = input.insurance_premium_milli {
        sets.push("insurance_premium_milli=?");
        params.push(Box::new(v));
    }
    if let Some(v) = input.ticket_allowance_milli {
        sets.push("ticket_allowance_milli=?");
        params.push(Box::new(v));
    }
    if let Some(v) = &input.sponsor_name {
        sets.push("sponsor_name=?");
        params.push(Box::new(v.clone()));
    }
    if let Some(v) = &input.sponsor_id {
        sets.push("sponsor_id=?");
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

#[tauri::command]
pub fn delete_employee(state: State<'_, DbState>, id: i64) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("UPDATE employees SET active=0 WHERE id=?", [id])
        .map_err(|e| e.to_string())?;
    let _ = rbac::log_audit(&conn, None, None, "delete_employee", "employees", Some(id), None, None, None);
    Ok("Deleted successfully".to_string())
}
