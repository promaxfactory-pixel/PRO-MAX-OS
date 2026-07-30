# PRO MAX OS

[![Version](https://img.shields.io/badge/Version-2.1.0-blue.svg)](https://github.com/promaxfactory-pixel/PRO-MAX-OS)
[![License](https://img.shields.io/badge/License-B2B%20Commercial-red.svg)](LICENSE)
[![Tech Stack](https://img.shields.io/badge/Tech-Tauri%202%20%7C%20React%2019%20%7C%20TypeScript%20%7C%20SQLite-6366F1.svg)](https://github.com/promaxfactory-pixel/PRO-MAX-OS)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)](https://github.com/promaxfactory-pixel/PRO-MAX-OS)
[![Rust](https://img.shields.io/badge/Built%20with-Rust-orange)](https://github.com/promaxfactory-pixel/PRO-MAX-OS)
[![Build](https://img.shields.io/badge/Build-Passing-success)](https://github.com/promaxfactory-pixel/PRO-MAX-OS/actions)
[![Security](https://img.shields.io/badge/Security-Audited-brightgreen)](https://github.com/promaxfactory-pixel/PRO-MAX-OS)

> **PRO MAX OS** — A professional Manufacturing ERP system designed for the paper cup and carton packaging industry in Oman. Built for efficiency, compliance, and scalability.

---

## Overview

PRO MAX OS is a full-featured Enterprise Resource Planning (ERP) solution purpose-built for paper cup and carton manufacturing factories operating in the Sultanate of Oman. It streamlines every aspect of factory operations — from production tracking and inventory management to HR compliance, invoicing, and supplier logistics — all within a single, unified platform.

Built with **Tauri 2** (Rust backend + React 19/TypeScript frontend), PRO MAX OS delivers a lightweight, high-performance desktop application with a premium dual-themed UI (light + dark) featuring gold and purple accents. It features glassmorphism design, 6 visual modes, dynamic CSS variable theming, and runs natively on Windows, macOS, and Linux.

| Attribute | Detail |
|---|---|
| **Version** | 2.1.0 |
| **Target Industry** | Paper Cup & Carton Manufacturing |
| **Region** | Oman (Omani Labor Law Compliant) |
| **License** | B2B Commercial |
| **Tech Stack** | Tauri 2.11, Rust, React 19, TypeScript, Tailwind CSS, SQLite (WAL), Recharts, Framer Motion |
| **Database** | SQLite (WAL mode, 28 migrations, 100 tables, 195+ indexes) |
| **Authentication** | Argon2id + AES-256-GCM + JWT with RBAC |
| **Auto-Updater** | Tauri updater plugin with minisign signing |
| **4 Binaries** | `promax-os` (GUI), `promax-mcp` (MCP stdio), `promax-api` (Actix-Web REST), `promax-mobile` (React Native) |

---

## Features

### Production Management
- Real-time carton and cup production tracking per shift
- Worker productivity monitoring with per-shift output metrics
- Production line status dashboards with live updates
- Defect and waste tracking with root-cause categorization
- Order-to-production pipeline with deadline alerts

### Inventory Management
- Shift-based inventory tracking (start quantity / end quantity)
- Raw material and finished goods stock control
- Low-stock alerts and reorder point notifications
- Batch and shift-level item traceability
- Barcode scanning support for stock-in / stock-out

### HR & Payroll
- Comprehensive Omani labor law compliance (Oman Labour Law)
- Employee records with work permit and residency tracking
- Attendance and shift scheduling with overtime calculations
- Full payroll processing with Omani tax and social insurance deductions
- Leave management compliant with Omani labor regulations (annual, sick, bereavement, Hajj, unpaid)
- End-of-service benefit (EOSB) calculations per Omani law

### Customer Management
- Local and international customer profiles
- Credit limit and balance tracking
- Contact history and order history per customer
- Cash and credit sales classification
- Customer classification (local / international / government)

### Invoice Management
- Full invoice lifecycle: creation, approval, payment, and closure
- Per-item price override for custom quotations
- Returns and credit note generation with original invoice linking
- Multi-currency support (OMR, USD, and more)
- Invoice PDF export and print-ready layouts

### Supplier Management
- Supplier master data with licensing documentation
- Barcode-based supplier identification system
- Purchase order tracking and goods receipt confirmation
- Supplier performance scoring and rating
- Payment history and outstanding balances

### Chinese Import Tracking
- End-to-end shipping status tracking (ocean/air freight)
- Customs clearance workflow with document checklist
- Port clearance status monitoring
- Container and bill of lading tracking
- Import duty and customs fee estimation

### Local Supplier Barter Exchange
- Track barter transactions: bags exchanged for cartons
- Inventory-adjusted barter entries
- Exchange rate and equivalence tracking
- Barter history with full audit trail

### Factory Loan & Installment Tracking
- Loan records with principal, interest, and tenure
- Installment scheduling with due date tracking
- Payment recording and outstanding balance calculation
- Loan-to-asset mapping for factory equipment

### Leave Management
- Omani labor law compliant leave types: Annual, Sick, Maternity, Bereavement, Hajj, Unpaid
- Leave request workflow: Submit → Manager Approval → HR Processing
- Leave balance tracking per employee per year
- Leave accrual rules configurable per employment contract
- Calendar view with team leave coverage

### E-Invoice (ZATCA / FATOORA Integration)
- Saudi ZATCA Phase 2 / FATOORA e-invoice compliance ready
- XML generation per ZATCA specifications
- QR code generation for each e-invoice
- Integration with FATOORA portal for validation and stamping
- Tax invoice numbering with sequential sequence management

### AI-Powered Insights
- Production trend analysis and anomaly detection
- Demand forecasting for cups and cartons
- Predictive inventory replenishment suggestions
- Profitability analysis per product line and customer
- Smart alerts for at-risk orders and delays

### OCR Receipt Scanning
- Upload receipt images for automatic data extraction
- OCR-powered vendor, date, total, and item recognition
- Manual review and correction workflow
- Receipt-to-voucher matching for accounts

### Excel Import with Smart Detection
- Drag-and-drop Excel (.xlsx / .xls) import for bulk data entry
- Auto-detection of column mappings (smart header recognition)
- Preview and validation before commit
- Supports: Products, Customers, Suppliers, Employees, Inventory transactions

### Multi-User RBAC with Audit Logs
- Role-Based Access Control: Super Admin, Manager, Operator, Accountant, Viewer
- Granular permission matrix per module and action
- RBAC enforced on 20+ financial mutation commands via `require_role()`
- Full audit trail: who did what, when, and from where
- Immutable action logs with timestamp and IP address
- Session management with concurrent user controls

### UI / UX - PRO MAX Design System
- Dual theme: Light + Dark with 6 visual modes (Power, Stability, Focus, Creative, Night, Professional)
- 50+ CSS design tokens with glassmorphism and gradients
- Gold/purple brand accent system with dynamic CSS variable theming
- 12 UI faces (6 modes × 2 themes)
- Premium glassmorphism components: Button (7 variants), Modal, Card, DataTable, Toast, Tabs, Badge, ConfirmDialog, GlobalSearch
- Sidebar with section-colored icons, collapse mode with floating tooltips
- Keyboard shortcuts: Ctrl+K (search), Ctrl+B (sidebar toggle)
- Mobile-responsive design for tablets and laptops
- Smooth Framer Motion animations and transitions
- Recharts-powered interactive dashboards and charts
- RTL-ready layout support for Arabic content
- 404 and 403 error pages with glassmorphism styling

---

## Tech Stack

| Layer | Technology | Purpose |
|---|---|---|---|
| **Desktop Framework** | Tauri 2.11 | Native desktop app shell with Rust backend |
| **Backend** | Rust (edition 2021) | High-performance system-level logic |
| **Frontend** | React 19 + TypeScript | Component-based UI with type safety |
| **Styling** | Tailwind CSS | Utility-first responsive design |
| **Database** | SQLite (rusqlite 0.31) | Embedded, serverless relational database (WAL mode) |
| **Visualization** | Recharts | Interactive charts and dashboards |
| **Animations** | Framer Motion | Smooth UI transitions and micro-interactions |
| **Authentication** | Argon2id + JWT (jsonwebtoken 9) | Memory-hard hashing + secure token auth |
| **Encryption** | AES-256-GCM (aes-gcm 0.10) | At-rest encryption for sensitive data |
| **OCR** | Tesseract.js | Client-side receipt text extraction |
| **Excel Parsing** | SheetJS (xlsx) + Calamine (Rust) | Browser + backend spreadsheet processing |
| **API Server** | Actix-Web 4 | REST API server for external integrations |
| **MCP Server** | stdio-based MCP | Model Context Protocol for AI assistant integration |
| **Auto-Updater** | tauri-plugin-updater 2.10 | Minisign-signed automatic updates |
| **PDF** | pdf-extract 0.7 | PDF text extraction for document processing |

---

## Project Structure

```
D:\PRO MAX OS\
├── src-tauri\               Rust backend (Tauri 2.11)
│   ├── src\                 Rust source files
│   │   ├── commands\        50 command modules (287+ commands)
│   │   │   ├── rbac.rs      RBAC require_role() enforcement
│   │   │   ├── invoices.rs  Invoice lifecycle (5 commands with RBAC)
│   │   │   ├── purchases.rs Purchase management
│   │   │   ├── expenses.rs  Expense tracking
│   │   │   ├── inventory.rs Stock & product management
│   │   │   ├── custody.rs   Custody fund/spend (with update + date filters)
│   │   │   ├── payroll.rs   Payroll processing
│   │   │   ├── accounting.rs GL, journal, trial balance
│   │   │   ├── assets.rs    Fixed asset management
│   │   │   ├── budget.rs    Budget planning & actuals
│   │   │   ├── ...          40+ more modules
│   │   ├── db.rs            Database (SCHEMA_VERSION=28, 28 migrations)
│   │   ├── schema.sql       100 tables, 195+ indexes, 71 FK constraints
│   │   ├── lib.rs           All 287+ commands registered
│   │   ├── main.rs          Desktop binary entrypoint
│   │   ├── bin/
│   │   │   ├── mcp_server.rs   MCP stdio server binary
│   │   │   └── api_server.rs   Actix-Web REST API binary
│   │   └── crypto.rs        Argon2id + AES-256-GCM + JWT
│   ├── Cargo.toml           Rust dependencies
│   └── tauri.conf.json      Tauri configuration (updater, CSP, identifier)
├── src\                     React 19 frontend
│   ├── components\          Reusable UI components
│   │   ├── layout\          Sidebar, Topbar, AppLayout
│   │   └── ui\              13 modern components (Button, DataTable, Modal, Toast, etc.)
│   ├── pages\               60+ route-level page components
│   │   ├── errors\          404/403 error pages
│   │   └── ...              Accounting, HR, Inventory, Invoices, Reports, Settings, etc.
│   ├── stores\              State management (auth, UI, license, document)
│   ├── hooks\               Custom React hooks
│   ├── utils\               Utility functions (printUtils, etc.)
│   ├── types\               TypeScript type definitions
│   ├── index.css            CSS Design System (50+ variables, 12 themes)
│   └── App.tsx              Router with lazy-loaded pages
├── docs\                    Documentation (API.md, schema_documentation.md)
├── scripts\                 Build and development scripts
├── .env.example             Template environment variables
├── .gitignore               Git ignore rules
├── package.json             Node.js dependencies and scripts
├── tsconfig.json            TypeScript configuration
├── tailwind.config.js       Tailwind CSS configuration (references CSS variables)
├── vite.config.ts           Vite 6 build configuration
└── README.md                This file
```

---

## Installation

### Prerequisites

| Requirement | Version | Notes |
|---|---|---|
| Node.js | 18.x or higher | LTS recommended |
| npm | 9.x or higher | Bundled with Node.js |
| Rust | 1.70+ | Via `rustup` |
| Tauri CLI | 2.x | `npm install -D @tauri-apps/cli` |
| SQLite | 3.x | Bundled with Tauri |

### Quick Start

```bash
# 1. Clone the repository
git clone https://github.com/promaxfactory-pixel/PRO-MAX-OS.git

# 2. Navigate into the project directory
cd "PRO MAX OS"

# 3. Install all dependencies
npm install

# 4. Start in development mode
npm run tauri dev

# 5. Build for production
npm run tauri build
```

### Development Notes

- The development server hot-reloads both frontend (Vite) and backend (Tauri) on code changes.
- The SQLite database is auto-created on first launch in the user's AppData directory.
- A developer PIN is auto-generated per machine on first run for quick local access.

---

## Configuration

### Application Config

The Tauri application is configured via:

```
src-tauri/tauri.conf.json
```

Key fields include app identity, window settings, build options, and Tauri-specific features (updater, system tray, etc.).

### Company Settings

Company profile and operational settings are stored in the `company_settings` table in the SQLite database. These are accessible through the Admin → Company Settings panel in-app, covering:

- Company name, address, and registration details
- Tax identification (VAT/TIN)
- Default currency and locale
- Printer and receipt template preferences
- Fiscal year start date

### Environment Variables

Create a `.env` file in the project root (see `.env.example` for template):

```env
# JWT signing secret for authentication tokens
PROMAX_JWT_SECRET=your-strong-random-secret-here

# Secret key for license validation and feature flag checks
PROMAX_LICENSING_SECRET=your-licensing-secret-here

# Encryption key (AES-256) for at-rest data protection
PROMAX_ENC_KEY=your-32-byte-encryption-key-here
```

### Developer PIN

On first launch, the system auto-generates a unique developer PIN tied to the machine's hardware identifier. This PIN is used for initial setup and recovery operations. It is displayed once and should be securely recorded.

---

## Database

| Attribute | Detail |
|---|---|
| **Engine** | SQLite (rusqlite 0.31, bundled) |
| **Schema Version** | 28 |
| **Migration Files** | 28 (applied sequentially on startup) |
| **Tables** | 100 |
| **Indexes** | 195+ (including 71 FK indexes + composite reporting indexes) |
| **Foreign Keys** | 71 REFERENCES clauses across 39 tables |
| **Journal Mode** | WAL (Write-Ahead Logging) |
| **Busy Timeout** | 5 seconds |
| **Monetary Unit** | Milli (1/1000 OMR) — stored as `INTEGER` |
| **Location** | User's AppData directory (per-platform) |

### Monetary Precision

All monetary values are stored in **milli** units (1/1000 of an OMR) as `INTEGER` to avoid floating-point precision issues. For example:

- 1.500 OMR → stored as `1500`
- 0.025 OMR → stored as `25`
- 100.000 OMR → stored as `100000`

Conversion to display format divides by 1000 and formats to 3 decimal places.

### Migrations (28 total)

| # | Purpose |
|---|---------|
| 1-23 | Base schema, production, inventory, HR, accounting, reporting, operating advances (from v2.0.0) |
| 24 | PK fix for `login_attempts`, FK indexes on `created_by` columns |
| 25 | 18 missing indexes (accounts, settings, document tables, roles, employees, payroll, stock, docflow, quality, closings); column name fixes |
| 26 | `import_history` table to managed schema; removed runtime `CREATE TABLE` |
| 27 | `reset_token` + `reset_token_expiry` on `users` for password reset |
| 28 | `avg_cost_milli` converted from `REAL` to `INTEGER` (money precision) |

Migrations are embedded in `src-tauri/src/db.rs` and applied automatically on application startup with proper error propagation.

---

## Security

PRO MAX OS v2.1.0 underwent a comprehensive security audit across Rust backend, SQLite database, and React frontend. All 70+ identified issues have been resolved.

### Authentication

| Mechanism | Implementation |
|---|---|
| **Password Hashing** | Argon2id (memory-hard, GPU-resistant, OWASP recommended) |
| **Session Management** | JWT tokens with configurable expiry and blacklist |
| **Token Blacklist** | Revoked tokens are blacklisted until expiry |
| **Multi-User System** | RBAC with role-based permission matrix |
| **Default Credentials** | Random 16-char + `Aa1!` generated per install (no hardcoded defaults) |
| **SQL Injection Prevention** | All dynamic values use parameterized queries; `LIMIT` clamped via `.clamp(1, 500)` |

### RBAC Implementation

- **`require_role()`** function in `commands/rbac.rs` enforces role checks on 20+ financial mutation commands
- Commands protected: invoices (create, post, void, duplicate, update), purchases (create, payment), expenses (create, reimburse, approve), accounting (journal entry), budget (plan, actual), assets (acquisition, depreciation, disposal), payroll (run), cashbank (transfer), cheques (issue), custody (spend, fund, transfer), petty_cash
- 287+ commands all registered in `lib.rs` — complete coverage

### Data Encryption

| Layer | Method |
|---|---|
| **At Rest** | AES-256-GCM for API keys and sensitive configuration |
| **In Transit** | Local IPC via Tauri's secured RPC (no network exposure by default) |

### Access Control

- **RBAC Roles:** Super Admin, Manager, Operator, Accountant, Viewer
- **Granular Permissions:** Per-module, per-action (create, read, update, delete, approve, export)
- **Audit Logging:** All user actions logged with user ID, timestamp, module, action type, and affected record ID

### Error Handling

- **53 unwrap() calls removed** — replaced with proper `AppError` propagation
- **2 panic! calls removed** — safe error handling throughout
- **Frontend**: All data-fetching pages use `useCallback` + `finally` pattern with proper error/loading states (14 files fixed)
- **Infinite spinners** on API failure eliminated

### HTTP Security Headers

The Tauri webview is hardened with Content Security Policy headers:

```
default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; connect-src 'self' http://localhost:*; font-src 'self' data:; object-src 'none'; frame-src 'none'; base-uri 'self'; form-action 'self'; frame-ancestors 'none'
```

### Auto-Updater Security

- Updates signed via **minisign** (public-key cryptography)
- Signing key pair generated per project
- Update integrity verified on each update check
- Update server endpoint: `https://releases.promaxos.com/update/{{target}}/{{current_version}}`

### License Enforcement

- Feature access is controlled via **22 feature flags** tied to the license tier
- License validation is performed against a secure local store
- License upgrades are tied to the configured licensing secret

---

## Build Verification

All checks pass for every commit:

| Check | Status |
|---|---|
| `tsc --noEmit` | ✅ 0 errors |
| `vite build` | ✅ 2,714 modules in ~7s |
| `cargo check` | ✅ 0 errors |
| `cargo clippy` | ✅ 0 warnings |
| `cargo test` | ✅ 44/44 |

## Build Artifacts

Production builds produce the following artifacts in `src-tauri/target/release/bundle/`:

| Format | File | Size |
|---|---|---|
| **MSI** | `PRO MAX OS_2.1.0_x64_en-US.msi` | ~15 MB |
| **NSIS** | `PRO MAX OS_2.1.0_x64-setup.exe` | ~9.2 MB |
| **Portable** | `promax-os.exe` | Optimized release binary |

Build with signing:
```bash
$env:TAURI_SIGNING_PRIVATE_KEY_PATH = "path\to\private.key"
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "your-password"
npx tauri build --bundles "msi,nsis"
```

## 287+ Tauri Commands (50 Modules)

All commands are registered in `src-tauri/src/lib.rs` under the `generate_handler![]` macro. Key modules:

| Module | File | Key Commands |
|---|---|---|
| **RBAC** | `commands/rbac.rs` | `require_role(user_id, &role)` |
| **Invoices** | `commands/invoices.rs` | create, update, post, void, duplicate, get, list, get_payments |
| **Purchases** | `commands/purchases.rs` | create, get, list, create_payment |
| **Expenses** | `commands/expenses.rs` | create, get, list, reimburse, approve |
| **Inventory** | `commands/inventory.rs` | get_stock, adjust, transfer, list_movements |
| **Custody** | `commands/custody.rs` | create_spend, update_spend, create_fund, update_fund, transfer, get_statement |
| **Petty Cash** | `commands/petty_cash.rs` | create, get, list, close |
| **Payroll** | `commands/payroll.rs` | run_payroll, get, list |
| **Accounting** | `commands/accounting.rs` | create_journal, get_entries, trial_balance, financial_statements |
| **Budget** | `commands/budget.rs` | create_plan, create_actual, get_budget_vs_actual |
| **Assets** | `commands/assets.rs` | acquire, depreciate, dispose, get, list |
| **Cash/Bank** | `commands/cashbank.rs` | create_transfer, get, list |
| **Cheques** | `commands/cheques.rs` | issue, get, list |
| **HR** | `commands/hr.rs` | Employees, advances, overtime, payroll |
| **Notifications** | `commands/notifications.rs` | create, list, mark_read, get_alerts |
| **Auth** | `commands/auth.rs` | login, change_password, validate_token, logout |
| **Settings** | `commands/settings.rs` | get_settings, update_settings, update_user |
| **Excel Import** | `commands/excel_import.rs` | import_products, import_customers, import_employees |
| **MCP Server** | `bin/mcp_server.rs` | stdio-based MCP protocol for AI integration |
| **API Server** | `bin/api_server.rs` | Actix-Web REST API for external integrations |

## License

PRO MAX OS is distributed under a **B2B Commercial License**.

| Tier | Price | Includes |
|---|---|---|
| **Free** | $0 | Core production + inventory (limited users) |
| **Basic** | Contact Sales | Full features, up to 3 users |
| **Professional** | Contact Sales | All features, up to 10 users, priority support |
| **Enterprise** | Contact Sales | Unlimited users, custom modules, SLA, on-premise hosting |

### Feature Flags

Access to features is governed by **22 feature flags** that are evaluated based on the active license tier. This allows a single codebase to serve all tiers with controlled feature unlock.

| # | Feature Flag | Free | Basic | Professional | Enterprise |
|---|---|---|---|---|---|
| 1 | `PROD_MANAGEMENT` | ✓ | ✓ | ✓ | ✓ |
| 2 | `INVENTORY_SHIFT` | ✓ | ✓ | ✓ | ✓ |
| 3 | `HR_PAYROLL_OMAN` | ✗ | ✓ | ✓ | ✓ |
| 4 | `LEAVE_MANAGEMENT` | ✗ | ✓ | ✓ | ✓ |
| 5 | `CUSTOMER_CREDIT` | ✗ | ✓ | ✓ | ✓ |
| 6 | `INVOICE_RETURNS` | ✗ | ✓ | ✓ | ✓ |
| 7 | `BARCODE_SUPPLIER` | ✗ | ✓ | ✓ | ✓ |
| 8 | `CHINA_IMPORT_TRACKING` | ✗ | ✗ | ✓ | ✓ |
| 9 | `BARTER_EXCHANGE` | ✗ | ✗ | ✓ | ✓ |
| 10 | `FACTORY_LOAN_TRACKING` | ✗ | ✗ | ✓ | ✓ |
| 11 | `E_INVOICE_ZATCA` | ✗ | ✗ | ✓ | ✓ |
| 12 | `AI_INSIGHTS` | ✗ | ✗ | ✓ | ✓ |
| 13 | `OCR_RECEIPT` | ✗ | ✗ | ✓ | ✓ |
| 14 | `EXCEL_IMPORT` | ✗ | ✗ | ✓ | ✓ |
| 15 | `MULTI_USER_RBAC` | ✗ | ✗ | ✓ | ✓ |
| 16 | `AUDIT_LOGS` | ✗ | ✗ | ✓ | ✓ |
| 17 | `DARK_THEME` | ✓ | ✓ | ✓ | ✓ |
| 18 | `MOBILE_RESPONSIVE` | ✓ | ✓ | ✓ | ✓ |
| 19 | `ANALYTICS_DASHBOARD` | ✗ | ✓ | ✓ | ✓ |
| 20 | `CUSTOM_REPORTS` | ✗ | ✗ | ✓ | ✓ |
| 21 | `API_INTEGRATIONS` | ✗ | ✗ | ✗ | ✓ |
| 22 | `CUSTOM_MODULES` | ✗ | ✗ | ✗ | ✓ |

---

## Author

**Mayadeen Bahla National Company** — صُنع في بهلا، سلطنة عُمان

- **GitHub:** [promaxfactory-pixel](https://github.com/promaxfactory-pixel/PRO-MAX-OS)
- **Releases:** https://github.com/promaxfactory-pixel/PRO-MAX-OS/releases
- **Repository:** https://github.com/promaxfactory-pixel/PRO-MAX-OS
- **Location:** Bahla, Oman

---

## Support

For technical support, feature requests, or licensing inquiries:

- **Email:** support@promaxos.com
- **GitHub Issues:** [https://github.com/promaxfactory-pixel/PRO-MAX-OS/issues](https://github.com/promaxfactory-pixel/PRO-MAX-OS/issues)
- **Update Server:** https://releases.promaxos.com

---

## Changelog

### v2.1.1 (Current) — Patch: RTL/LTR Support, i18n Fixes, Login Overhaul
- Rewrote LoginPage.tsx with proper i18n keys (eliminated mojibake Arabic text corruption)
- Fixed stale hardcoded password hint → now directs users to check console
- Added `data-theme` initialization in `main.tsx` (eliminates theme flash on first load)
- Added RTL/LTR direction support throughout: AppLayout, Sidebar, Topbar, Toast, SearchBar
- Fixed HTML `dir` attribute to be dynamic based on stored language preference
- Fixed Sidebar positioning for both RTL (right-0) and LTR (left-0) modes
- Fixed Topbar search icon, user menu dropdown, notification dot positioning
- Added missing i18n translation keys (lightMode, darkMode, notifications, loginError, etc.)
- Fixed Arabic locale version strings (2.0.0 → 2.1.0)
- Added CSS RTL direction logic in `index.css`

### v2.1.0 — Hardened Security & Production Release
- Comprehensive security audit: 70+ issues resolved
- Comprehensive security audit: 70+ issues resolved
- RBAC on 20+ financial mutation commands via `require_role()`
- Random 16-char admin password (no hardcoded defaults)
- SQL injection fix in notifications.rs
- 53 unwrap() + 2 panic! removed from Rust code
- 5 database migrations (24-28): PK fixes, 18 new indexes, money type conversion
- 71 FOREIGN KEY constraints across 39 tables
- Auto-updater with minisign signing (tauri-plugin-updater v2.10)
- Custody module: update commands, date filtering, description search
- 404/403 error pages with glassmorphism
- Light mode compatibility: text-white CSS variables
- All infinite spinners fixed (14 files)
- Zero warnings: tsc, cargo check, cargo clippy, cargo test

### v2.0.0
- Comprehensive CSS Design System (50+ variables, 12 themes, 6 modes)
- Modernized Sidebar with colored icons, collapse mode, Ctrl+B
- 13 UI components: Button (7 variants), DataTable, Modal, Toast, Tabs, etc.
- Login page locked to dark theme
- Full build verification passing

---

<p align="center">
  <b>PRO MAX OS</b> — نظام إتصنيع متكامل للشركات العمانية
  <br/>
  Empowering Oman's Paper Cup & Carton Manufacturing Industry
  <br/>
  &copy; 2026 Mayadeen Bahla National Company. All rights reserved.
</p>