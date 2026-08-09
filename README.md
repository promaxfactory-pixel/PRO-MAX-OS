# PRO MAX OS

[![Version](https://img.shields.io/badge/Version-2.0.0-blue.svg)](https://github.com/promaxfactory-pixel/PRO-MAX-OS)
[![License](https://img.shields.io/badge/License-B2B%20Commercial-red.svg)](LICENSE)
[![Tech Stack](https://img.shields.io/badge/Tech-Tauri%202%20%7C%20React%2018%20%7C%20TypeScript%20%7C%20SQLite-6366F1.svg)](https://github.com/promaxfactory-pixel/PRO-MAX-OS)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)](https://github.com/promaxfactory-pixel/PRO-MAX-OS)
[![Rust](https://img.shields.io/badge/Built%20with-Rust-orange)](https://github.com/promaxfactory-pixel/PRO-MAX-OS)

> **PRO MAX OS** — A professional Manufacturing ERP system designed for the paper cup and carton packaging industry in Oman. Built for efficiency, compliance, and scalability.

---

## Overview

PRO MAX OS is a full-featured Enterprise Resource Planning (ERP) solution purpose-built for paper cup and carton manufacturing factories operating in the Sultanate of Oman. It streamlines every aspect of factory operations — from production tracking and inventory management to HR compliance, invoicing, and supplier logistics — all within a single, unified platform.

Built with **Tauri 2** (Rust backend + React/TypeScript frontend), PRO MAX OS delivers a lightweight, high-performance desktop application with a premium dark-themed UI featuring gold and indigo accents. It is fully mobile-responsive and runs natively on Windows, macOS, and Linux.

| Attribute | Detail |
|---|---|
| **Version** | 2.0.0 |
| **Target Industry** | Paper Cup & Carton Manufacturing |
| **Region** | Oman (Omani Labor Law Compliant) |
| **License** | B2B Commercial |
| **Tech Stack** | Tauri 2, Rust, React 18, TypeScript, Tailwind CSS, SQLite, Recharts, Framer Motion |
| **Database** | SQLite (WAL mode, 24 migrations) |
| **Authentication** | JWT with Argon2id hashing + RBAC |
| **Mobile** | PWA (`mobile/`) + signed Android APK |
| **Extras** | REST API (`promax-api`) · MCP server (`promax-mcp`) |

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

### AI Any-File Import
- Upload any document/image/PDF — extraction, structure analysis, and mapping to business records
- Commits into customers, suppliers, products, inventory, invoices, purchases, or expenses
- Duplicate detection and full review-before-commit workflow
- Provider failover across 7 AI engines (Ollama, Groq, Gemini, DeepSeek, Mistral, OpenAI, Anthropic)

### Mobile Manager (PWA + Android)
- Self-contained Arabic RTL PWA, served by the built-in API server
- KPI dashboard, invoices/purchases/expenses, approvals, alerts and notifications
- Products, customers, activity, company data, change password, and expense creation
- Offline-ready with a versioned service worker; signed release APK in `mobile/`

### REST API & MCP
- `promax-api`: hardened REST API v3 (JWT, RBAC, rate limiting, audit trail, security headers)
- `promax-mcp`: MCP (JSON-RPC) server exposing database tools to AI clients

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
- Full audit trail: who did what, when, and from where
- Immutable action logs with timestamp and IP address
- Session management with concurrent user controls

### UI / UX
- Premium dark theme with gold and indigo color palette
- Mobile-responsive design for tablets and laptops
- Smooth Framer Motion animations and transitions
- Recharts-powered interactive dashboards and charts
- Keyboard shortcuts for power users
- RTL-ready layout support for Arabic content

---

## Tech Stack

| Layer | Technology | Purpose |
|---|---|---|
| **Desktop Framework** | Tauri 2 | Native desktop app shell with Rust backend |
| **Backend** | Rust | High-performance system-level logic |
| **Frontend** | React 18 + TypeScript | Component-based UI with type safety |
| **Styling** | Tailwind CSS | Utility-first responsive design |
| **Database** | SQLite | Embedded, serverless relational database |
| **Visualization** | Recharts | Interactive charts and dashboards |
| **Animations** | Framer Motion | Smooth UI transitions and micro-interactions |
| **Authentication** | JWT + Argon2id | Secure token-based auth with modern hashing |
| **Encryption** | AES-256-GCM | At-rest encryption for sensitive data |
| **OCR** | Tesseract (Rust backend) | Receipt image text extraction |
| **Excel Parsing** | calamine (Rust) | Spreadsheet parsing in the backend |
| **REST API** | actix-web | Hardened API server (`promax-api`) |
| **AI** | reqwest (rustls) | Multi-provider AI calls with failover |
| **Mobile** | PWA + Tauri Android | `mobile/` app and signed APK |

---

## Project Structure

```
PRO-MAX-OS\
├── src-tauri\            Rust backend (Tauri 2)
│   ├── src\              Rust source (commands, crypto, db, lib)
│   │   └── bin\          Binary entrypoints (api_server, mcp_server)
│   ├── icons\            App icons (desktop + Android/iOS)
│   └── Cargo.toml        Rust dependencies manifest
├── src\                  React frontend
│   ├── components\       Reusable UI components
│   ├── pages\            Route-level page components (per domain)
│   ├── stores\           Zustand stores (auth, ui, license)
│   ├── i18n\             Arabic/English locale files (2143 keys each)
│   ├── hooks\            Custom React hooks
│   ├── lib\              Utilities and helpers
│   ├── types\            TypeScript type definitions
│   └── utils\            Formatting and print helpers
├── mobile\               Self-contained PWA (no build step)
│   ├── app.js            Application logic
│   ├── styles.css        Styling
│   ├── sw.js             Versioned service worker
│   ├── manifest.webmanifest
│   └── PRO-MAX-OS.apk    Signed Android release APK
├── scripts\              Tooling (build, i18n checks, project export)
├── database\             Schema documentation
├── .github\              CI workflows
├── package.json          Node.js dependencies and scripts
├── tsconfig.json         TypeScript configuration
├── tailwind.config.js    Tailwind CSS configuration
├── vite.config.ts        Vite build configuration
└── README.md             This file
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

### REST API & Mobile PWA

```bash
# Run the API server (serves the mobile PWA at http://127.0.0.1:8080)
cd src-tauri
PROMAX_DB_PATH=path/to/app.db PROMAX_JWT_SECRET=change-me cargo run --bin promax-api -- --port 8080 --mobile-dir ../mobile

# Expose to the local network (e.g. to reach it from a phone)
#   add --expose  (binds 0.0.0.0)
```

### Quality Gates

Run these before every release:

| Check | Command |
|---|---|
| Rust build + tests | `cargo check --lib --bins` · `cargo test --lib` (in `src-tauri/`) |
| Lint (Rust) | `cargo clippy --lib --bins -- -D warnings` (in `src-tauri/`) |
| TypeScript | `npx tsc --noEmit` |
| ESLint | `npx eslint src/ --ext .ts,.tsx --max-warnings 0` |
| i18n parity | `node scripts/check-i18n.mjs` (expect `OK: i18n keys consistent`) |
| Production build | `npm run build` |
| Export for AI dev | `node scripts/export-project.mjs` (writes `dist/export/`) |

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
| **Engine** | SQLite |
| **Schema Version** | 24 |
| **Migration Files** | 24 (sequential, numbered) |
| **Journal Mode** | WAL (Write-Ahead Logging) |
| **Busy Timeout** | 5 seconds |
| **Monetary Unit** | Milli (1/1000 OMR) — stored as integers |
| **Location** | User's AppData directory (per-platform) |

### Monetary Precision

All monetary values are stored in **milli** units (1/1000 of an OMR) to avoid floating-point precision issues. For example:

- 1.500 OMR → stored as `1500`
- 0.025 OMR → stored as `25`
- 100.000 OMR → stored as `100000`

Conversion to display format divides by 1000 and formats to 3 decimal places.

### Migrations

Database migrations are located in `database/migrations/` and are applied sequentially on application startup. Each migration file contains both `up` and `down` SQL statements for forward and rollback operations.

---

## Security

### Authentication

| Mechanism | Implementation |
|---|---|
| **Password Hashing** | Argon2id (memory-hard, GPU-resistant) |
| **Session Management** | JWT tokens with configurable expiry |
| **Token Blacklist** | Revoked tokens are blacklisted until expiry |
| **Multi-User System** | RBAC with role-based permission matrix |

### Data Encryption

| Layer | Method |
|---|---|
| **At Rest** | AES-256-GCM for API keys and sensitive configuration |
| **In Transit** | Local IPC via Tauri's secured RPC (no network exposure by default) |

### Access Control

- **RBAC Roles:** Super Admin, Manager, Operator, Accountant, Viewer
- **Granular Permissions:** Per-module, per-action (create, read, update, delete, approve, export)
- **Audit Logging:** All user actions logged with user ID, timestamp, module, action type, and affected record ID

### HTTP Security Headers

The Tauri webview is hardened with Content Security Policy headers:

```
default-src 'self'; script-src 'self'; frame-ancestors 'none'
```

This prevents:
- Inline script execution
- External resource loading
- Frame embedding (clickjacking protection)

### License Enforcement

- Feature access is controlled via **22 feature flags** tied to the license tier
- License validation is performed against a secure local store
- License upgrades are tied to the configured licensing secret

---

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

**Mayadeen Bahla National Company**

- **GitHub:** [promaxfactory-pixel](https://github.com/promaxfactory-pixel/PRO-MAX-OS)
- **Repository:** https://github.com/promaxfactory-pixel/PRO-MAX-OS
- **Location:** Bahla, Oman

---

## Support

For technical support, feature requests, or licensing inquiries:

- **Email:** support@promaxos.com
- **GitHub Issues:** [https://github.com/promaxfactory-pixel/PRO-MAX-OS/issues](https://github.com/promaxfactory-pixel/PRO-MAX-OS/issues)

---

<p align="center">
  <b>PRO MAX OS</b> — Empowering Oman's Paper Cup & Carton Manufacturing Industry
  <br/>
  &copy; 2026 Mayadeen Bahla National Company. All rights reserved.
</p>