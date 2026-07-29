use crate::error::AppError;
use calamine::{open_workbook, Data, Reader};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileContent {
    pub file_name: String,
    pub file_type: String,
    pub content: String,
    pub metadata: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpreadsheetContent {
    pub file_name: String,
    pub sheets: Vec<SheetData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SheetData {
    pub name: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub row_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub file_name: String,
    pub file_path: String,
    pub extension: String,
    pub size_bytes: u64,
    pub is_readable: bool,
}

fn data_to_string(val: &Data) -> String {
    match val {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => format!("{}", f),
        Data::Int(i) => format!("{}", i),
        Data::Bool(b) => format!("{}", b),
        Data::Error(e) => format!("{:?}", e),
        Data::DateTime(dt) => format!("{}", dt),
        Data::DateTimeIso(s) => s.clone(),
        Data::DurationIso(s) => s.clone(),
    }
}

fn read_text_file(path: &str) -> Result<FileContent, AppError> {
    let content = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
    let file_name = Path::new(path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    Ok(FileContent {
        file_name,
        file_type: "txt".to_string(),
        content,
        metadata: None,
    })
}

fn read_json_file(path: &str) -> Result<FileContent, AppError> {
    let raw = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
    let file_name = Path::new(path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let parsed: serde_json::Value =
        serde_json::from_str(&raw).map_err(|e| format!("Invalid JSON: {}", e))?;
    let formatted =
        serde_json::to_string_pretty(&parsed).map_err(|e| format!("Failed to format JSON: {}", e))?;

    Ok(FileContent {
        file_name,
        file_type: "json".to_string(),
        content: formatted,
        metadata: Some(format!("Valid JSON — {} bytes", raw.len())),
    })
}

fn read_xml_file(path: &str) -> Result<FileContent, AppError> {
    let raw = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
    let file_name = Path::new(path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let _doc = quick_xml::Reader::from_str(&raw);

    Ok(FileContent {
        file_name,
        file_type: "xml".to_string(),
        content: raw,
        metadata: None,
    })
}

fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in line.chars() {
        match ch {
            '"' if in_quotes => in_quotes = false,
            '"' if !in_quotes => in_quotes = true,
            ',' if !in_quotes => {
                fields.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    fields.push(current.trim().to_string());
    fields
}

fn read_csv_file(path: &str) -> Result<FileContent, AppError> {
    let raw = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
    let file_name = Path::new(path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let mut lines: Vec<String> = Vec::new();

    for (i, line) in raw.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let fields = parse_csv_line(line);
        if i == 1 {
            lines.push("-".repeat(60));
        }
        lines.push(fields.join(" | "));
    }

    Ok(FileContent {
        file_name,
        file_type: "csv".to_string(),
        content: lines.join("\n"),
        metadata: Some(format!("{} rows", lines.len().saturating_sub(1))),
    })
}

fn get_extension(path: &str) -> String {
    Path::new(path)
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase()
}

fn extract_text_from_bytes(bytes: &[u8]) -> String {
    let mut result = String::new();
    let mut current = String::new();

    for &b in bytes {
        if (32..127).contains(&b) {
            current.push(b as char);
        } else if b == 10 || b == 13 {
            if !current.is_empty() {
                result.push_str(&current);
                result.push('\n');
                current.clear();
            }
        } else {
            if current.len() > 3 {
                result.push_str(&current);
                result.push(' ');
            }
            current.clear();
        }
    }
    if current.len() > 3 {
        result.push_str(&current);
    }

    result
}

#[tauri::command]
pub fn file_read_text(path: String) -> Result<FileContent, AppError> {
    let ext = get_extension(&path);

    match ext.as_str() {
        "json" => read_json_file(&path),
        "xml" => read_xml_file(&path),
        "csv" => read_csv_file(&path),
        "txt" | "log" | "md" | "toml" | "yaml" | "yml" | "ini" | "cfg" => read_text_file(&path),
        _ => {
            let raw = fs::read(&path).map_err(|e| format!("Failed to read file: {}", e))?;
            let file_name = Path::new(&path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            if let Ok(text) = String::from_utf8(raw.clone()) {
                Ok(FileContent {
                    file_name,
                    file_type: ext,
                    content: text,
                    metadata: None,
                })
            } else {
                Ok(FileContent {
                    file_name,
                    file_type: ext,
                    content: extract_text_from_bytes(&raw),
                    metadata: Some("Binary file — extracted readable strings".to_string()),
                })
            }
        }
    }
}

#[tauri::command]
pub fn file_read_spreadsheet(path: String) -> Result<SpreadsheetContent, AppError> {
    let file_name = Path::new(&path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let mut workbook: calamine::Xlsx<_> = open_workbook(&path)
        .map_err(|e| format!("Failed to open Excel file: {}", e))?;

    let sheet_names = workbook
        .sheet_names()
        .to_owned();

    let mut sheets: Vec<SheetData> = Vec::new();

    for name in &sheet_names {
        let range = workbook
            .worksheet_range(name)
            .map_err(|e| format!("Failed to read sheet '{}': {}", name, e))?;

        let mut rows: Vec<Vec<String>> = Vec::new();
        for row in range.rows() {
            rows.push(row.iter().map(data_to_string).collect());
        }

        let headers = if !rows.is_empty() {
            rows.remove(0)
        } else {
            Vec::new()
        };

        let row_count = rows.len();

        sheets.push(SheetData {
            name: name.clone(),
            headers,
            rows,
            row_count,
        });
    }

    Ok(SpreadsheetContent { file_name, sheets })
}

#[tauri::command]
pub fn file_read_docx(path: String) -> Result<FileContent, AppError> {
    let data = fs::read(&path).map_err(|e| format!("Failed to read file: {}", e))?;
    let file_name = Path::new(&path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let content = extract_text_from_bytes(&data);

    Ok(FileContent {
        file_name,
        file_type: "docx".to_string(),
        content,
        metadata: Some("Extracted readable text from DOCX binary".to_string()),
    })
}

#[tauri::command]
pub fn file_read_any(path: String) -> Result<FileContent, AppError> {
    let ext = get_extension(&path);

    match ext.as_str() {
        "xlsx" | "xls" => {
            let ss = file_read_spreadsheet(path.clone())?;
            let file_name = Path::new(&path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let mut content = String::new();
            for sheet in &ss.sheets {
                content.push_str(&format!("=== {} ===\n", sheet.name));
                content.push_str(&sheet.headers.join(" | "));
                content.push('\n');
                content.push_str(&"-".repeat(60));
                content.push('\n');
                for row in &sheet.rows {
                    content.push_str(&row.join(" | "));
                    content.push('\n');
                }
                content.push('\n');
            }

            Ok(FileContent {
                file_name,
                file_type: ext,
                content,
                metadata: Some(format!("{} sheet(s)", ss.sheets.len())),
            })
        }
        "docx" => file_read_docx(path),
        "json" => read_json_file(&path),
        "xml" => read_xml_file(&path),
        "csv" => read_csv_file(&path),
        _ => read_text_file(&path),
    }
}

#[tauri::command]
pub fn file_get_info(path: String) -> Result<FileInfo, AppError> {
    let metadata = fs::metadata(&path).map_err(|e| format!("Failed to get file info: {}", e))?;

    let file_name = Path::new(&path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let extension = get_extension(&path);

    let _last_modified = metadata
        .modified()
        .ok()
        .and_then(|t| {
            t.duration_since(std::time::UNIX_EPOCH)
                .ok()
                .map(|d| format!("{}", d.as_secs()))
        })
        .unwrap_or_default();

    Ok(FileInfo {
        file_name,
        file_path: path,
        extension,
        size_bytes: metadata.len(),
        is_readable: metadata.is_file(),
    })
}
