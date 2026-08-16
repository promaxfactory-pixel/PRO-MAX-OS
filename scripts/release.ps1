# ============================================================
# PRO MAX OS - Release Pipeline
# Builds signed installers, verifies signatures, assembles the
# release package, and (optionally) publishes a GitHub release.
#
# Usage:
#   .\scripts\release.ps1 -Publish            # full pipeline + GitHub release
#   .\scripts\release.ps1 -SkipBuild          # reuse existing bundles
#   .\scripts\release.ps1 -SkipUpload         # build + package, no GitHub
#
# Requirements:
#   - gh CLI authenticated against promaxfactory-pixel/PRO-MAX-OS
#   - Updater signing key at ~/.tauri/promax-os.key with password in
#     scripts\.env.release.local (TAURI_SIGNING_PRIVATE_KEY_PASSWORD=...)
#   - Never commit the key or scripts\.env.release.local
# ============================================================
[CmdletBinding()]
param(
  [string]$Version = "",
  [switch]$Publish,
  [switch]$SkipBuild,
  [switch]$SkipUpload
)

$ErrorActionPreference = "Stop"
$root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$tauriDir = Join-Path $root "src-tauri"
$envFile = Join-Path $PSScriptRoot ".env.release.local"
$keyPath = Join-Path $env:USERPROFILE ".tauri\promax-os.key"
$pubKeyPath = "$keyPath.pub"

function Step([string]$msg) { Write-Host "`n=== $msg ===" -ForegroundColor Cyan }
function Die([string]$msg) { Write-Error $msg; exit 1 }

if (-not $Version) {
  $pkg = Get-Content (Join-Path $root "package.json") -Raw | ConvertFrom-Json
  $Version = $pkg.version
}
if ($Version -notmatch '^\d+\.\d+\.\d+$') { Die "Invalid version '$Version'" }
$tag = "v$Version"

# ---- 1. Signing material ---------------------------------------------------
Step "1/9 Signing material"
if (-not (Test-Path $envFile)) { Die "Missing $envFile (TAURI_SIGNING_PRIVATE_KEY_PASSWORD)" }
if (-not (Test-Path $keyPath)) { Die "Missing updater key $keyPath" }
$password = (Get-Content $envFile -Raw).Replace("TAURI_SIGNING_PRIVATE_KEY_PASSWORD=", "").Trim()
if (-not $password) { Die "Empty password in $envFile" }
$env:TAURI_SIGNING_PRIVATE_KEY = (Get-Content $keyPath -Raw).Trim()
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = $password
Write-Host "Signing key loaded (rsign, minisign-compatible pubkey)."

# ---- 2. Version consistency ------------------------------------------------
Step "2/9 Version consistency"
$check = @{
  package.json    = ((Get-Content (Join-Path $root "package.json") -Raw | ConvertFrom-Json).version)
  Cargo.toml      = (Select-String (Join-Path $tauriDir "Cargo.toml") "^version\s*=\s*`"([^`"]+)`"").Matches[0].Groups[1].Value
  tauri.conf.json = (Get-Content (Join-Path $tauriDir "tauri.conf.json") -Raw | ConvertFrom-Json).version
}
foreach ($k in $check.Keys) { if ($check[$k] -ne $Version) { Die "Version mismatch: $k is $($check[$k]), expected $Version" } }
Write-Host "Versions consistent ($Version)."

# ---- 3. Verification gates -------------------------------------------------
Step "3/9 Test gates"
Push-Location $tauriDir
try {
  Write-Host "[cargo test --lib]"
  cargo test --lib 2>&1 | Select-Object -Last 3
  if ($LASTEXITCODE -ne 0) { Die "cargo test failed" }
  Write-Host "[cargo clippy]"
  cargo clippy --all-targets --all-features -- -D warnings 2>&1 | Select-Object -Last 3
  if ($LASTEXITCODE -ne 0) { Die "clippy failed" }
} finally { Pop-Location }
Push-Location $root
try {
  Write-Host "[tsc --noEmit]"
  npx.cmd tsc --noEmit; if ($LASTEXITCODE -ne 0) { Die "tsc failed" }
  Write-Host "[vitest run]"
  npx.cmd vitest run; if ($LASTEXITCODE -ne 0) { Die "vitest failed" }
  Write-Host "[eslint]"
  npx.cmd eslint src; if ($LASTEXITCODE -ne 0) { Die "eslint failed" }
} finally { Pop-Location }

# ---- 4. Build signed bundles ------------------------------------------------
if (-not $SkipBuild) {
  Step "4/9 Build signed bundles (msi,nsis)"
  Push-Location $root
  try {
    npx.cmd tauri build --bundles msi,nsis 2>&1 | Select-Object -Last 6
    if ($LASTEXITCODE -ne 0) { Die "tauri build failed" }
  } finally { Pop-Location }
}

$setup = Join-Path $tauriDir "target\release\bundle\nsis\PRO MAX OS_${Version}_x64-setup.exe"
$setupSig = "$setup.sig"
$msi = Join-Path $tauriDir "target\release\bundle\msi\PRO MAX OS_${Version}_x64_en-US.msi"
$msiSig = "$msi.sig"
foreach ($f in @($setup, $setupSig, $msi, $msiSig)) { if (-not (Test-Path $f)) { Die "Missing bundle artifact: $f" } }

# ---- 5. Verify signatures ----------------------------------------------------
Step "5/9 Verify updater signatures"
Push-Location $tauriDir
try {
  foreach ($pair in @(@($setup, $setupSig), @($msi, $msiSig))) {
    $out = cargo run --example verify_updater_sig -- $pair[0] $pair[1] $pubKeyPath 2>&1
    if ($out -notmatch "SIG_VERIFY_OK") { Die "Signature verification failed for $($pair[1])" }
    Write-Host "OK $($pair[1])"
  }
} finally { Pop-Location }

# ---- 6. Assemble release folder ---------------------------------------------
Step "6/9 Assemble release folder"
$relDir = Join-Path $root "release\PRO MAX OS $Version"
New-Item -ItemType Directory -Path $relDir -Force | Out-Null
$items = @(
  @($setup, (Split-Path $setup -Leaf)),
  @($setupSig, (Split-Path $setupSig -Leaf)),
  @($msi, (Split-Path $msi -Leaf)),
  @($msiSig, (Split-Path $msiSig -Leaf)),
  @((Join-Path $tauriDir "target\release\promax-os.exe"), "promax-os.exe"),
  @((Join-Path $tauriDir "target\release\promax-api.exe"), "promax-api.exe"),
  @((Join-Path $tauriDir "target\release\promax-mcp.exe"), "promax-mcp.exe"),
  @((Join-Path $tauriDir ".promax_os_license"), ".promax_os_license")
)
foreach ($i in $items) {
  if (-not (Test-Path $i[0])) { Die "Missing source artifact: $($i[0])" }
  Copy-Item $i[0] (Join-Path $relDir $i[1]) -Force
}
$tmpl = Join-Path $root "release\PRO MAX OS $Version\promax.secrets.template.json"
if (-not (Test-Path $tmpl)) { Die "Missing promax.secrets.template.json - copy it from a previous release folder" }
Write-Host "Assembled: $relDir"

# ---- 7. Checksums + updates.json ---------------------------------------------
Step "7/9 Checksums and updates.json"
$files = Get-ChildItem $relDir -File | Where-Object { $_.Name -ne "SHA256SUMS.txt" }
$lines = foreach ($f in $files) { "$((Get-FileHash $f.FullName -Algorithm SHA256).Hash.ToLower()) *$($f.Name)" }
Set-Content (Join-Path $relDir "SHA256SUMS.txt") $lines -Encoding ascii

$pub = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
$notes = "PRO MAX OS $Version - see CHANGELOG.md for details."
$base = "https://github.com/promaxfactory-pixel/PRO-MAX-OS/releases/download/$tag"
$updDir = Join-Path $relDir "updates\windows"
foreach ($bt in @("nsis", "msi")) {
  $sub = if ($bt -eq "nsis") { "PRO MAX OS_${Version}_x64-setup.exe" } else { "PRO MAX OS_${Version}_x64_en-US.msi" }
  $sig = Get-Content (Join-Path $relDir "$sub.sig") -Raw
  $json = @{
    version  = $Version
    notes    = $notes
    pub_date = $pub
    platforms = @{ "windows-x86_64" = @{ signature = $sig.Trim(); url = "$base/$($sub -replace ' ', '%20')" } }
  } | ConvertTo-Json -Depth 5
  $target = Join-Path $updDir "$bt\$Version.json"
  New-Item -ItemType Directory -Path (Split-Path $target) -Force | Out-Null
  Set-Content $target $json -Encoding utf8
  Write-Host "Wrote $target"
}

# ---- 8. Commit + tag ---------------------------------------------------------
Step "8/9 Commit and tag"
Push-Location $root
try {
  git add -A
  git diff --cached --quiet; if ($LASTEXITCODE -eq 0) { Write-Host "Nothing to commit." }
  else {
    git commit -m "chore: release $Version" | Out-Null
    Write-Host "Committed."
  }
  $existing = git tag -l $tag
  if (-not $existing) {
    git tag $tag
    git push origin master --tags 2>&1 | Out-Null
    Write-Host "Pushed tag $tag"
  } else {
    Write-Host "Tag $tag already exists."
  }
} finally { Pop-Location }

# ---- 9. GitHub release --------------------------------------------------------
if ($Publish -and -not $SkipUpload) {
  Step "9/9 GitHub release"
  Push-Location $root
  try {
    $ghRelease = gh release view $tag --json id 2>$null
    if (-not $ghRelease) {
      gh release create $tag --repo promaxfactory-pixel/PRO-MAX-OS --title "PRO MAX OS $Version" --notes $notes | Out-Null
    }
    gh release upload $tag --repo promaxfactory-pixel/PRO-MAX-OS --clobber `
      (Join-Path $relDir (Split-Path $setup -Leaf)) `
      (Join-Path $relDir (Split-Path $setupSig -Leaf)) `
      (Join-Path $relDir (Split-Path $msi -Leaf)) `
      (Join-Path $relDir (Split-Path $msiSig -Leaf)) `
      (Join-Path $relDir "promax-os.exe") `
      (Join-Path $relDir "promax-api.exe") `
      (Join-Path $relDir "promax-mcp.exe") `
      (Join-Path $relDir "SHA256SUMS.txt")
    Write-Host "Release: https://github.com/promaxfactory-pixel/PRO-MAX-OS/releases/tag/$tag"
  } finally { Pop-Location }
}

Write-Host "`n=== Release $Version complete ===" -ForegroundColor Green
