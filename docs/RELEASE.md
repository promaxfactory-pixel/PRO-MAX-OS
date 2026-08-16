# RELEASE.md — Production Release Pipeline

This document describes how a production release of PRO MAX OS is built, signed,
verified, and published. **Release binaries are built locally on the maintainer's
machine** — never by CI (see the guard note in `.github/workflows/ci.yml`).

## 1. Version bump

1. Pick the next version (e.g. `2.6.3`). Semantic increments for breaking
   changes, features, fixes.
2. Bump **all** of these to the same version:
   - `package.json` (frontend)
   - `package-lock.json` (regenerate via `npm install`)
   - `src-tauri/Cargo.toml`
   - `src-tauri/Cargo.lock` (the version line — `cargo check` regenerates)
   - `src-tauri/tauri.conf.json`
   - `src-tauri/src/mcp.rs` (MCP version constant)
   - `README.md` (badges, version table, artifact names, changelog section)
   - `CHANGELOG.md` (new `[x.y.z]` entry at the top)
   - i18n locales: `src/i18n/locales/{ar,en,hi,ur}/*.json`
   - `src/pages/settings/SettingsPage.tsx` and
     `src/pages/license/LicenseActivationPage.tsx` if they print a version
3. Run all gates (below) before tagging.

## 2. Quality gates (must all pass)

```powershell
# backend (from src-tauri/)
cargo test                  # 143+ tests
cargo clippy --lib --bins -- -D warnings
# frontend (repo root)
npx tsc --noEmit
npx vitest run              # 31+ tests
npx eslint src/ --ext .ts,.tsx   # 0 errors (pre-existing warnings OK)
```

## 3. Signing material (updater + Authenticode)

### Updater signing key (Tauri `rsign` format)

- Key pair: `~/.tauri/promax-os.key` (private) and `~/.tauri/promax-os.key.pub` (public).
- The private key **must never be committed**. The password lives in
  `scripts/.env.release.local` (gitignored via `.env.*.local`) as
  `TAURI_SIGNING_PRIVATE_KEY_PASSWORD=...`.
- The **public** key is embedded in `src-tauri/tauri.conf.json` →
  `plugins.updater.pubkey` (base64, `dW50cnVzdGVk...`).

> **Key rotation:** if you rotate the key, `pubkey` must change in the same
> release. Existing installed clients keep verifying against the old embedded
> key, so they will reject updates signed by the new key. Ship the rotation as
> part of a new user-visible release.

### Authenticode code-signing (optional, requires a certificate)

`scripts/sign-artifacts.ps1` wraps `signtool.exe`:

```powershell
# PFX-based cert
.\scripts\sign-artifacts.ps1 -Path "release\PRO MAX OS 2.6.3\*.exe" -Pfx "C:\certs\code.pfx" -PfxPassword $env:PFX_PASS
# or machine store thumbprint
.\scripts\sign-artifacts.ps1 -Path "..." -Thumbprint "SHA1THUMBPRINT"
```

Without a certificate the files are shipped unsigned; updater `.sig` files are
independent of Authenticode and are always produced.

## 4. Build (signed bundles + update artifacts)

```powershell
# from repo root — MUST use npx directly, `npm run tauri build` exits 1 without bundling
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content "C:\Users\USER\.tauri\promax-os.key" -Raw  # inline content
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = (Get-Content "scripts\.env.release.local").Value
npx.cmd tauri build --bundles msi,nsis
```

Preconditions for auto-generated `.sig` files:

- `bundle.createUpdaterArtifacts: true` in `src-tauri/tauri.conf.json`
- both `TAURI_SIGNING_PRIVATE_KEY` (raw file content, trimmed) and the password
  are set in the environment.

Outputs (in `src-tauri/target/release/bundle/`):

- `nsis/PRO MAX OS_<ver>_x64-setup.exe` + `.sig`
- `msi/PRO MAX OS_<ver>_x64_en-US.msi` + `.sig`

## 5. Verify signatures

```powershell
cargo run --release --example verify_updater_sig -- `
  <path-to-setup.exe.sig> <path-to-setup.exe> <path-to-pubkey>
```

Prints `SIG_VERIFY_OK` for each artifact. The example mirrors exactly how the
installed app verifies updates (`base64`-decode pubkey + sig, then
`PublicKey::decode` / `Signature::decode` / `verify(..., true)`).

## 6. Assemble the release folder

`scripts/release.ps1` automates steps 4–9. It assembles
`release/PRO MAX OS <ver>/` containing:

- installers + `.sig` files
- `promax-os.exe`, `promax-api.exe`, `promax-mcp.exe`
- `.promax_os_license`
- `promax.secrets.template.json` (placeholder — the real secrets file is never
  shipped in the repo)
- `SHA256SUMS.txt`
- `README.md` (Arabic, includes default credentials)
- `updates/windows/{nsis,msi}/<ver>.json` — the per-bundle update records

## 7. Publish to GitHub

```powershell
git add -A
git commit -m "release: v<ver>"
git tag v<ver>
git push origin master --tags
gh release create v<ver> --target <commit-sha> --title "PRO MAX OS v<ver>" `
  "release/PRO MAX OS <ver>/PRO MAX OS_<ver>_x64-setup.exe" `
  "release/PRO MAX OS <ver>/PRO MAX OS_<ver>_x64-setup.exe.sig" `
  "release/PRO MAX OS <ver>/PRO MAX OS_<ver>_x64_en-US.msi" `
  "release/PRO MAX OS <ver>/PRO MAX OS_<ver>_x64_en-US.msi.sig" `
  "release/PRO MAX OS <ver>/promax-os.exe" `
  "release/PRO MAX OS <ver>/promax-api.exe" `
  "release/PRO MAX OS <ver>/promax-mcp.exe" `
  "release/PRO MAX OS <ver>/SHA256SUMS.txt"
```

> **Gotcha:** the shell timeout can kill `gh release create` mid-upload leaving
> an untagged release. Push the tag first, then recreate against the commit SHA
> (`gh release create v<ver> --target <sha> ...`); delete the half-uploaded
> release if needed.

## 8. Deploy update feed to the server

Upload the release folder contents to the update host (`releases.promaxos.com`):

```
/update/windows/nsis/<ver>.json
/update/windows/msi/<ver>.json
```

plus the installer files. See `docs/UPDATER.md` for the exact endpoint layout
and JSON format. This step cannot be done from the desktop machine and must be
performed by whoever administers the server.

## 9. Install & smoke-test on a real machine

1. `PRO MAX OS_<ver>_x64-setup.exe /S` (silent NSIS install).
2. Launch `promax-os.exe`; confirm the window title `PRO MAX OS` appears and the
   process stays alive and responsive.
3. Confirm a fresh DB was created in `%APPDATA%\com.promaxos.desktop\promax.db`
   with `schema_version` = `SCHEMA_VERSION` in `src-tauri/src/db.rs` and the
   seeded `admin` user (`must_change_password = 1`).
4. Log in with `admin` / `Admin@2026`, PIN `246810`.
