# Changelog

All notable changes to PRO MAX OS are documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.6.1] - 2026-08-15

### Fixed
- **Commercial invoice line totals.** `line_net_milli` was stored as the carton
  unit price instead of `cartons × unit price`, which corrupted the printed line
  totals, the invoice detail view and the product-sales reports
  (`SUM(sales_invoice_lines.line_net_milli)`). Header totals and accounting
  posting were never affected. Regression test added (250 cartons × 15,000 = 3,750,000).
- **Quotation print validity.** "ساري حتى" now shows the actual expiry date
  (`date + validity days`) instead of the literal "+ N يوم" text.
- **Atomic creation.** `create_quotation` and `create_commercial_invoice` are now
  transactional: a failure while inserting lines rolls the whole document back
  instead of leaving an orphan header.
- **Unknown products fail loudly.** Quotation/commercial-invoice lines referencing
  a missing `product_id` now return "المنتج غير موجود" instead of silently
  defaulting to 1,000 cups.
- **Expense summary scope.** `get_expense_summary` accepts an optional
  `approval_status` filter; the factory dashboard gains a "الكل / معتمد فقط"
  toggle and each detail row now shows its approval status badge. Totals keep the
  "all types of expenses" behaviour by default.

### Changed
- Version bumped to 2.6.1 (installers and portable build regenerated).

## [2.6.0] - 2026-08-15

### Added
- **مصنع الأكواب (cup factory).** New dedicated section for the professional paper-cup
  factory, accessible from a new "مصنع الكؤوس" sidebar group:
  - **Factory dashboard (`/factory`).** Live production totals for today (total
    cartons/cups, morning vs. evening shift), per-product and per-worker production
    with waste, and a low-stock inventory alert table. Integrated expense summary
    with a week/month toggle showing the grand total plus breakdowns by source
    (من عهد الموظفين / من أصحاب المصنع / من الحسابات الرئيسية) and by category,
    with the last recorded rows listed.
  - **Quotations (عروض الأسعار / كوتيشن).** Full professional quotation flow with
    sequential numbering (`QUOT-YYYY-####`), client info (linked customer or free
    text), validity period, discount, terms and notes. Line editor supports products
    (cup size, cups per carton, cartons, unit price) or free-form items with a live
    total, Draft/Sent/Accepted/Rejected status lifecycle, and a sky-blue themed
    professional print layout (signature blocks included).
  - **Commercial invoices (فواتير تجارية — غير ضريبية).** Non-VAT commercial
    invoices printed with the factory name only (no company, no VAT column, no
    tax): numbered `CINV-YYYY-####`, customer, payment method, line editor
    (product / cartons / unit price), posting to inventory and accounts, and a
    green-themed print template marked "بيان تجاري غير ضريبي".
  - **E-invoice integration.** Commercial invoices are explicitly excluded from the
    Oman e-invoicing auto-enqueue path (`auto_enqueue_on_post` skips them), keeping
    them fully non-tax and non-Fawtara.

### Changed
- Version bumped to 2.6.0 (installers and portable build regenerated).

### Fixed
- Fresh-install schema: `expenses` now includes `custody_txn_id`,
  `reimbursement_status`, `reimbursement_date`, `reimbursed_by` (previously only
  applied by an interrupted migration), and `sales_invoices` includes
  `is_commercial` — existing databases are migrated automatically on upgrade.
- `get_expense_summary` now returns a `details` list of every expense row in the
  range alongside the source/category aggregates.

### Tests
- Rust: cup-factory suite — quotation create/update/delete/print flow, commercial
  invoice non-VAT + printable, expense summary source/category grouping, and
  commercial invoices skipped by e-invoice auto-enqueue. Full suite 136 green,
  clippy clean.
- Frontend: TypeScript strict, ESLint clean (no errors), production build green.



### Added
- **Fawtara (Oman e-invoicing) foundation.** New `fawtara` module built on the
  existing PINT-OM (CrossIndustryInvoice) engine, explicitly marked as a technical
  foundation for Decision 189/2026 — not yet accredited by the Omani tax authority
  and requiring an ASP/OTA connection to go live.
  - `fawtara_build_payload`: regenerates the PINT-OM XML + SHA-256 hash and builds a
    TLV (tag-length-value) QR payload (seller, tax number, timestamp, total, VAT)
    with a per-tag breakdown for audit.
  - `fawtara_readiness`: company-level compliance checklist (name, VAT number, CR
    number, OMR currency, 5% VAT, ASP/OTA connection) with a readiness score.
  - `fawtara_connector_status` + `fawtara_submit` behind a `FawtaraConnector`
    abstraction: a local `DevConnector` for sandbox (synthetic references) and an
    `HttpsConnector` for a configured production ASP/OTA endpoint; credentials are
    decrypted before transmission.
  - **Auto-enqueue on posting.** `post_invoice` now honors the existing
    "إرسال عند ترحيل الفاتورة" setting: when enabled, posting an invoice
    generates the XML and queues it for submission.
  - E-Invoice page relabeled "الفوترة الإلكترونية — فاوترة" with a live readiness
    card and QR TLV preview; sidebar updated.
- **Custody & petty cash (العهد والصرف النثري).** New `/custody` page over the
  existing `custody` command set: fund creation (opening balance), spend recording,
  fund-to-fund transfers, and date-filtered statements with running balances; three
  stat cards (accounts, total balances, spending limits). Sidebar entry updated.
- **Delete actions on detail pages.** Customer, Supplier, Product, and Employee
  detail pages now offer a soft-delete (حذف) with confirmation dialog, keeping
  accounting history intact.
- **Invoice notes editing.** `InvoiceDetailPage` gains an editable "الملاحظات" card
  backed by `update_invoice` (visible on the printed invoice).

### Changed
- Version bumped to 2.5.0 (installers and portable build regenerated).

### Tests
- Rust: `fawtara` suite — TLV round-trip (incl. extended-length values), truncated
  payload rejection, QR tag layout, bad-base64 rejection, readiness detection for
  missing CR / non-OMR currency, fully-configured readiness, dev connector
  reference, and environment→connector resolution. Full suite 132 green, clippy
  `-D warnings` clean.

## [2.4.1] - 2026-08-15

### Added
- **Credit notes (إشعار دائن) — full flow shipped.**
  - Backend `create_credit_note` (against a posted invoice): per-product return
    quantities validated against the original invoice (price/VAT always taken from
    the original invoice lines, never trusted from the client), cumulative credit
    checks against existing non-voided notes, `CN-YYYY-####` numbering via the
    shared sequence, stock return + `credit_note` inventory movements, and a
    balanced reversal journal (revenue/VAT/AR; inventory/COGS when applicable),
    with AR balance reduction for credit-sale invoices and full audit logging.
  - `list_credit_notes` (with invoice filter), `get_invoice_credit_remaining`
    (per-product remaining returnable quantity), and the existing
    `get_credit_note_for_print` are now all wired into the UI.
  - New `/credit-notes` page (list + print) and a credit-note modal on
    `InvoiceDetailPage` for Posted invoices (editable return quantities defaulting
    to the remaining balance, live totals, reason field).
- **Supplier payment receipt (سند صرف).** `get_supplier_receipt_for_print` +
  `SupplierReceiptPrintTemplate`; `SupplierPaymentPage` now shows a success screen
  with a "طباعة سند صرف" button after recording a payment.

### Changed
- Version bumped to 2.4.1 (installers and portable build regenerated).

### Tests
- Rust: `credit_note_reverses_invoice_and_returns_stock` (journal balance, stock
  return, AR reduction, remaining quantities, over-return rejection, draft-invoice
  rejection) and `supplier_receipt_print_returns_payment_and_supplier`. Full suite
  123 green, clippy `-D warnings` clean.

## [2.4.0] - 2026-08-14

### Fixed
- **Company settings now actually persist.** The settings form sent keys the backend
  could not store (`company_name_ar`, `vat_rate`, bank fields...), so serde silently
  dropped them: company name, VAT rate, currency and bank details were never saved,
  and printed documents always showed the fallback header with a hardcoded 5% VAT.
  Schema migration 35 adds `cr_number`, `currency`, `fiscal_year_start`, `bank_name`,
  `bank_account_no`, `bank_iban`, `bank_swift` to `company_settings`; `CompanySettings`
  / `UpdateSettingsInput` / `CompanyPrintInfo` were aligned; the settings page now
  binds to the real fields and composes `bank_details` for printed invoices.
- **Receipt printing shipped.** `CustomerPaymentPage` now shows a "طباعة الإيصال"
  button after a successful payment, printing the receipt via the existing
  `get_receipt_for_print` command and `ReceiptPrintTemplate` (previously wired
  nowhere in the UI).
- **Payment amounts entered in OMR, not milli.** Customer and supplier payment pages
  label the field "المبلغ (بالريال)", accept `step=0.001`, convert ×1000 before
  saving and show the milli equivalent live. Entering 1.5 now records 1,500 milli.
- **Invoice creation honours the chosen date** (was silently always "today") and the
  VAT preview uses the company default rate instead of a hardcoded 5%.
- **Invoice detail print types** corrected (`InvoicePrintData` / `DeliveryNoteData`
  instead of `SalesInvoice`); stale version strings updated to 2.4.0 across the app.

### Changed
- Schema version 34 → 35 (company profile columns).
- Version bumped to 2.4.0 (installers and portable build regenerated).

## [2.3.0] - 2026-08-14

### Added
- **ZATCA Phase 2 (Saudi e-invoicing)** — `zatca2` engine + command set:
  - UBL 2.1 invoice builder with BR-KSA schema validation (VAT, QR, hashing).
  - ECDSA secp256k1 signing (via `k256`) with encrypted key storage.
  - Phase-2 compliant 9-tag base64 QR (seller name, VAT, timestamp, total,
    VAT amount, hash, signature, certificate, QR encoding schema).
  - Automatic CSID onboarding (compliance → production) against Fatoora,
    plus manual CSR generation; environment split sandbox/production/simplified.
  - Invoice clearance (standard) and reporting (simplified) endpoints with
    UUID capture and rejection tracking (`zatca_settings` table, migration 34).
  - New `zatca2_*` commands and dedicated `/tools/zatca2` UI page.
- **Qayd XBRL filing (Kuwait)** — `qayd` engine + command set:
  - IFRS-Full annual financial statement XBRL instance generation in KWD
    from the general ledger (closing balances, P&L, retained earnings, SOCE).
  - Taxonomy-aligned reporting with reconciliation and validation report.
  - `qayd_*` commands and `/tools/qayd` UI page with XML preview and totals.
- **Multi-branch + offline sync** — `branches` module:
  - Branch CRUD with head-office protection and active toggling.
  - `offline_sync_queue` for mutations captured while disconnected, with
    enqueue/list/mark-synced/retry/stats commands.
  - `/settings/branches` UI page (queue filter, payload inspection, retry).
- Frontend: 3 new pages, sidebar entries and routes for the above.

### Changed
- Schema version 33 → 34 (`branches`, `offline_sync_queue`, `zatca_settings`,
  `qayd_filings`, Phase-2 columns on `e_invoices`).
- Version bumped to 2.3.0 (installers and portable build regenerated).

## [2.2.0] - 2026-08-09

Unified release: the v2.0 AI/mobile line integrated onto the v2.1.1 line
(multi-provider AI, any-file AI import, offline mobile PWA, REST API v3).

### Added
- **Mobile PWA** (`mobile/`): installable offline-first dashboard for phones
  and tablets served directly by the REST API (`--mobile-dir`). Covers KPIs,
  invoices, purchases, expenses, approvals, alerts, notifications, activity,
  customers and products.
- **REST API v3** (`promax-api`): static PWA serving, expense creation,
  KPI/approval/notification/alert/activity/company endpoints, per-IP login and
  API rate limiting, hardened security headers, and tunable server settings.
- **AI provider layer** (`ai_providers`): OpenAI, Anthropic, Google Gemini,
  Ollama, Groq, DeepSeek, Mistral — encrypted API keys, provider catalog,
  status checks, model listing, and automatic failover chat.
- **AI any-file import** (`ai_file_import`): analyze PDF/images/Excel/CSV/
  DOCX/TXT/JSON/XML via the configured provider, structured field extraction,
  duplicate detection, and one-click commit to invoices, purchases, customers,
  products, suppliers or expenses. New `ai_extractions` table (migration 29).
- **`ai_chat_with_provider`** command with provider selection and failover.
- **Full i18n**: comprehensive en/ar translations (2164 keys/locale), batch
  merge tooling (`scripts/merge-i18n-batches.mjs`, `check-i18n.mjs`).
- **Docs/tooling**: `LICENSE`, `CHANGELOG.md`, project export packager
  (`scripts/export-project.mjs`).

### Fixed
- API server now exposes the full dashboard/KPI surface to the mobile app.
- Missing i18n keys on the 2.1.x line (password change flow, search clear)
  are now translated in both en and ar.

## [2.0.0] - 2026-08-09

Baseline release of the v2 product line.

### Added
- **AI engine layer** (`ai_providers`): 7 providers (Ollama, Groq, Google Gemini,
  DeepSeek, Mistral, OpenAI, Anthropic) with provider status catalog, settings,
  connectivity tests, and automatic failover.
- **AI any-file import** (`ai_file_import`): analyze, list, get, delete, update,
  and commit flows that map documents into customers, suppliers, products,
  inventory, invoices, purchases, and expenses — with duplicate detection.
- **AI assistant**: provider-aware chat with per-provider configuration.
- **AI dashboard** and dedicated import/assistant pages, wired to navigation.
- **REST API server v3** (`promax-api`): JWT (Argon2id) auth, token blacklist,
  RBAC roles, IP rate limiting, audit trail, security headers, and hardened
  static serving of the mobile PWA with SPA fallback and path-traversal guards.
- **Mobile manager**: self-contained Arabic RTL PWA (`mobile/`) — login, KPI
  home, invoices/purchases/expenses, approvals (approve/reject), alerts and
  notifications, products, customers, activity, company, change password, and
  expense creation.
- **Android packaging**: Android toolchain setup, `cargo-ndk` cross-compile,
  Tauri Android project, and signed release APK (`mobile/PRO-MAX-OS.apk`).
- **MCP server** (`promax-mcp`): JSON-RPC MCP interface exposing database tools
  and resources.
- **E-invoice module** (ZATCA/FATOORA-ready), OCR receipt scanning, Excel and
  historical data import, import-shipment tracking, barter exchange,
  operating advances, multi-warehouse stock transfers, production shift
  tracking, government dashboard, and a 24-migration schema
  (`SCHEMA_VERSION = 24`) including the `ai_extractions` table.
- **Tooling**: `scripts/export-project.mjs` (Claude-ready project packager),
  `scripts/check-i18n.mjs` (key parity guard), i18n batch merge tools, and a
  GitHub Actions CI (cargo check/test/clippy, tsc, eslint, Windows build).

### Changed
- `reqwest` uses rustls with webpki roots (no OpenSSL dependency) for
  cross-platform and Android builds.
- PWA render loop refactored to a single-pass shell build (no double render).
- Service worker rewritten with versioned cache and network-first core.
- Static file IO in the API server moved off the actix worker pool (`web::block`).

### Security
- Password hashing via Argon2id; JWT with jti blacklist and logout revocation.
- API rate limiting (10 login attempts / 15 minutes) and hardened HTTP headers.
- Strict path-traversal rejection in the static file server.
- Secrets kept out of the repository (`.env`, `*.secrets.json` ignored).

### Notes
- Database money amounts stored as integer milli (1/1000 OMR).
- The seeded test database login used during development is not a default in
  production builds; set `PROMAX_JWT_SECRET` and strong admin credentials.
