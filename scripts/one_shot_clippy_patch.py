from pathlib import Path

path = Path("src-tauri/src/commands/payroll.rs")
text = path.read_text(encoding="utf-8")
old = '''    let employees: Vec<(i64, i64, i64, i64, i64, i64, i64, i64)> = {
        let mut stmt = tx.prepare(
            "SELECT id, COALESCE(salary_milli,0), COALESCE(basic_salary_milli,0),
                    COALESCE(allowances_milli,0), COALESCE(housing_allowance_milli,0),
                    COALESCE(transport_allowance_milli,0), COALESCE(food_allowance_milli,0),
                    COALESCE(other_allowances_milli,0)
             FROM employees
             WHERE active=1
             ORDER BY id",
        )?;
        let rows = stmt.query_map([], |r| Ok((
            r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?,
            r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?
        )))?;
        rows.collect::<Result<Vec<_>, _>>()?
    };
'''
new = '''    #[derive(Debug)]
    struct EmployeePayConfig {
        employee_id: i64,
        salary: i64,
        basic_configured: i64,
        allowances_fallback: i64,
        housing: i64,
        transport: i64,
        food: i64,
        other: i64,
    }

    let employees: Vec<EmployeePayConfig> = {
        let mut stmt = tx.prepare(
            "SELECT id, COALESCE(salary_milli,0), COALESCE(basic_salary_milli,0),
                    COALESCE(allowances_milli,0), COALESCE(housing_allowance_milli,0),
                    COALESCE(transport_allowance_milli,0), COALESCE(food_allowance_milli,0),
                    COALESCE(other_allowances_milli,0)
             FROM employees
             WHERE active=1
             ORDER BY id",
        )?;
        let rows = stmt.query_map([], |r| Ok(EmployeePayConfig {
            employee_id: r.get(0)?,
            salary: r.get(1)?,
            basic_configured: r.get(2)?,
            allowances_fallback: r.get(3)?,
            housing: r.get(4)?,
            transport: r.get(5)?,
            food: r.get(6)?,
            other: r.get(7)?,
        }))?;
        rows.collect::<Result<Vec<_>, _>>()?
    };
'''
if text.count(old) != 1:
    raise SystemExit(f"employee config block count={text.count(old)}")
text = text.replace(old, new, 1)
old_loop = "    for (employee_id, salary, basic_configured, allowances_fallback, housing, transport, food, other) in employees {"
new_loop = '''    for EmployeePayConfig {
        employee_id,
        salary,
        basic_configured,
        allowances_fallback,
        housing,
        transport,
        food,
        other,
    } in employees {'''
if text.count(old_loop) != 1:
    raise SystemExit(f"employee loop count={text.count(old_loop)}")
text = text.replace(old_loop, new_loop, 1)
path.write_text(text, encoding="utf-8")
