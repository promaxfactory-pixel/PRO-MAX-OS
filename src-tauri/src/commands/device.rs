use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct PrinterInfo {
    pub name: String,
    pub is_default: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScannerInfo {
    pub name: String,
}

#[tauri::command]
pub fn list_printers() -> Result<Vec<PrinterInfo>, String> {
    let ps = r#"
Get-CimInstance -ClassName Win32_Printer | Select-Object Name, Default | ConvertTo-Json -Compress
"#;
    let out = Command::new("powershell")
        .args(["-NoProfile", "-Command", ps])
        .output()
        .map_err(|e| format!("Failed to list printers: {}", e))?;

    let stdout = String::from_utf8_lossy(&out.stdout);
    if stdout.trim().is_empty() || stdout.trim() == "null" {
        return Ok(Vec::new());
    }

    serde_json::from_str::<Vec<serde_json::Value>>(&stdout)
        .map(|items| {
            items
                .into_iter()
                .map(|v| PrinterInfo {
                    name: v["Name"].as_str().unwrap_or("").to_string(),
                    is_default: v["Default"].as_bool().unwrap_or(false),
                })
                .collect()
        })
        .or_else(|_| {
            // Single printer result (not array)
            serde_json::from_str::<serde_json::Value>(&stdout).map(|v| {
                vec![PrinterInfo {
                    name: v["Name"].as_str().unwrap_or("").to_string(),
                    is_default: v["Default"].as_bool().unwrap_or(false),
                }]
            })
        })
        .map_err(|e| format!("Failed to parse printer list: {}", e))
}

#[tauri::command]
pub fn print_html(html: String, printer_name: Option<String>) -> Result<String, String> {
    let temp_dir = std::env::temp_dir();
    let html_path = temp_dir.join("promax_print.html");
    std::fs::write(&html_path, &html).map_err(|e| format!("Failed to write temp HTML: {}", e))?;

    let ps = match printer_name {
        Some(name) => format!(
            r#"Start-Process -FilePath "msedge" -ArgumentList "--print-to-printer='{0}','{1}'" -NoNewWindow -Wait"#,
            name.replace("'", "''"),
            html_path.to_str().unwrap_or("")
        ),
        None => format!(
            r#"Start-Process -FilePath "msedge" -ArgumentList "--print-to-printer='','{0}'" -NoNewWindow -Wait"#,
            html_path.to_str().unwrap_or("")
        ),
    };

    Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps])
        .output()
        .map_err(|e| format!("Failed to print: {}", e))?;

    let _ = std::fs::remove_file(&html_path);
    Ok("تم إرسال المستند إلى الطابعة".to_string())
}

#[tauri::command]
pub fn print_thermal(
    lines: Vec<String>,
    printer_name: Option<String>,
    copies: Option<u32>,
) -> Result<String, String> {
    let esc = |c: u8| -> String { format!("\\x{:02x}", c) };
    let mut raw = String::new();

    raw.push_str(&esc(0x1b));
    raw.push_str("@");

    for _ in 0..copies.unwrap_or(1) {
        for line in &lines {
            raw.push_str(line);
            raw.push_str(&esc(0x0a));
        }
    }

    raw.push_str(&esc(0x1b));
    raw.push_str("m");

    let temp_file = std::env::temp_dir().join("promax_thermal.bin");
    std::fs::write(&temp_file, raw.as_bytes()).map_err(|e| format!("Failed to write temp file: {}", e))?;

    let printer = printer_name.unwrap_or_default();
    let ps = format!(
        r#"Get-Content -Path "{0}" -Encoding Byte | Out-Printer -Name "{1}" -Wait"#,
        temp_file.to_str().unwrap_or(""),
        printer
    );

    Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps])
        .output()
        .map_err(|e| format!("Failed to print thermal: {}", e))?;

    let _ = std::fs::remove_file(&temp_file);
    Ok("تمت طباعة الإيصال بنجاح".to_string())
}

#[tauri::command]
pub fn list_scanners() -> Result<Vec<ScannerInfo>, String> {
    let ps = r#"
$scanners = @()
try {
    $wia = New-Object -ComObject WIA.DeviceManager 2>$null
    if ($wia) {
        for ($i = 1; $i -le $wia.DeviceInfos.Count; $i++) {
            $device = $wia.DeviceInfos.Item($i)
            $props = $device.Properties
            $name = $props.Item("Name").Value
            $scanners += @{ name = "$name" }
        }
    }
} catch {}
if ($scanners.Count -eq 0) {
    Write-Output "[]"
} else {
    $scanners | ConvertTo-Json -Compress
}
"#;

    let out = Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps])
        .output()
        .map_err(|e| format!("Failed to list scanners: {}", e))?;

    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if stdout.is_empty() || stdout == "[]" || stdout == "null" {
        return Ok(Vec::new());
    }

    serde_json::from_str::<Vec<serde_json::Value>>(&stdout)
        .map(|items| {
            items
                .into_iter()
                .map(|v| ScannerInfo {
                    name: v["name"].as_str().unwrap_or("").to_string(),
                })
                .collect()
        })
        .or_else(|_| {
            serde_json::from_str::<serde_json::Value>(&stdout).map(|v| {
                vec![ScannerInfo {
                    name: v["name"].as_str().unwrap_or("").to_string(),
                }]
            })
        })
        .map_err(|e| format!("Failed to parse scanner list: {}", e))
}

#[tauri::command]
pub fn scan_document(scanner_name: Option<String>, output_path: Option<String>) -> Result<String, String> {
    let out_path = output_path.unwrap_or_else(|| {
        let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        std::env::temp_dir()
            .join(format!("promax_scan_{}.jpg", ts))
            .to_str()
            .unwrap_or("")
            .to_string()
    });

    let name_filter = match &scanner_name {
        Some(n) => format!("if ($deviceName -eq '{}') {{", n.replace("'", "''")),
        None => "{".to_string(),
    };

    let ps = format!(
        r#"
$outPath = '{0}'
$wia = New-Object -ComObject WIA.DeviceManager 2>$null
if (-not $wia) {{ Write-Error "لا يوجد ماسح ضوئي متصل"; exit 1 }}
$found = $false
for ($i = 1; $i -le $wia.DeviceInfos.Count; $i++) {{
    $deviceInfo = $wia.DeviceInfos.Item($i)
    $deviceName = $deviceInfo.Properties.Item("Name").Value
    $type = $deviceInfo.Type
    if ($type -eq 1) {{
        {1}
            $device = $deviceInfo.Connect()
            $item = $device.Items.Item(1)
            $image = $item.Transfer("{{B96B3CAE-0728-11D3-9D7B-0000F81EF32E}}")
            $image.SaveFile($outPath)
            $found = $true
            break
        }}
    }}
}}
if (-not $found) {{ Write-Error "لم يتم العثور على ماسح ضوئي"; exit 1 }}
Write-Output "OK:$outPath"
"#,
        out_path.replace("'", "''"),
        name_filter,
    );

    let out = Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps])
        .output()
        .map_err(|e| format!("Failed to scan: {}", e))?;

    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(format!("فشل المسح الضوئي: {}", err));
    }

    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if stdout.starts_with("OK:") {
        Ok(stdout[3..].to_string())
    } else {
        Ok(out_path)
    }
}
