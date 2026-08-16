# ============================================================
# PRO MAX OS - Authenticode code signing (signtool)
# Signs Windows executables/installers with an Authenticode
# certificate to remove SmartScreen warnings.
#
# Usage:
#   .\scripts\sign-artifacts.ps1 -Path ".\release\PRO MAX OS 2.6.3\*.exe" `
#     -CertThumbprint "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX" `
#     -TimestampServer "http://timestamp.digicert.com"
#
#   Or use a PFX:
#   .\scripts\sign-artifacts.ps1 -Path ".\dist\*.exe" -PfxFile ".\codesign.pfx" -PfxPassword "..."
#
# No certificate installed? The script exits 1 with guidance; the
# updater signatures (.sig) are produced separately by release.ps1
# and do NOT require an Authenticode certificate.
# ============================================================
[CmdletBinding()]
param(
  [Parameter(Mandatory)][string]$Path,
  [string]$CertThumbprint = "",
  [string]$PfxFile = "",
  [string]$PfxPassword = "",
  [string]$TimestampServer = "http://timestamp.digicert.com",
  [switch]$Force
)

$ErrorActionPreference = "Stop"

function Find-SignTool {
  $roots = @(
    "${env:ProgramFiles(x86)}\Windows Kits\10\bin",
    "$env:ProgramFiles\Windows Kits\10\bin"
  )
  foreach ($root in $roots) {
    if (-not (Test-Path $root)) { continue }
    $exe = Get-ChildItem $root -Recurse -Filter "signtool.exe" -ErrorAction SilentlyContinue |
      Sort-Object { [version]($_.Directory.Name -replace '\.','.') } -Descending |
      Select-Object -First 1
    if ($exe) { return $exe.FullName }
  }
  return $null
}

$files = Get-ChildItem $Path -ErrorAction Stop
if ($files.Count -eq 0) { Write-Error "No files matched '$Path'"; exit 1 }

$signtool = Find-SignTool
if (-not $signtool) { Write-Error "signtool.exe not found. Install the Windows SDK (Windows Kits\10\bin)"; exit 1 }
Write-Host "signtool: $signtool"

if (-not $CertThumbprint -and -not $PfxFile) {
  Write-Error "Provide -CertThumbprint (installed cert) or -PfxFile. See docs/RELEASE.md for acquiring a code-signing certificate."
  exit 1
}

foreach ($f in $files) {
  Write-Host "Signing $($f.FullName)..."
  if ($CertThumbprint) {
    & $signtool sign /sha1 $CertThumbprint /tr $TimestampServer /td sha256 /fd sha256 /v $f.FullName
  } else {
    & $signtool sign /f $PfxFile /fd sha256 /tr $TimestampServer /td sha256 /p $PfxPassword /v $f.FullName
  }
  if ($LASTEXITCODE -ne 0) { Write-Error "signtool failed for $($f.FullName)"; exit 1 }
  & $signtool verify /pa /v $f.FullName
  if ($LASTEXITCODE -ne 0) { Write-Error "verification failed for $($f.FullName)"; exit 1 }
  Write-Host "OK $($f.FullName)"
}

Write-Host "`nAll artifacts signed and verified." -ForegroundColor Green
