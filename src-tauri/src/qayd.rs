//! Kuwait "Qayd" XBRL engine.
//!
//! Qayd is the Ministry of Commerce & Industry (MoCI) portal for filing
//! annual financial statements in XBRL. From 1 January 2027 it is mandatory
//! for all Kuwaiti legal entities (private companies, SPCs, partnerships,
//! joint-stock companies); 2026 is a voluntary filing year. The filing uses
//! the IFRS full taxonomy (ifrs-full) with the Kuwaiti Dinar (KWD) as the
//! reporting currency.
//!
//! This module produces valid XBRL instance documents for the three required
//! financial statements (statement of financial position, statement of profit
//! or loss, statement of cash flows) and maps the application's seeded chart
//! of accounts to IFRS concepts.

use std::collections::BTreeMap;

/// IFRS full taxonomy namespace used by Qayd filings.
pub const IFRS_FULL_NS: &str = "http://xbrl.ifrs.org/taxonomy/2024-01-01/ifrs-full";
/// XBRL instance namespace.
pub const XBRLI_NS: &str = "http://www.xbrl.org/2003/instance";
/// ISO 4217 unit namespace.
pub const ISO4217_NS: &str = "http://www.xbrl.org/2003/iso4217";
/// Default entity identifier scheme for Kuwaiti commercial registry numbers.
pub const KUWAIT_CR_SCHEME: &str = "https://www.qayd.gov.kw/cr";

#[derive(Debug, Clone)]
pub struct QaydCompany {
    pub name_ar: String,
    pub cr_number: String,
    pub currency: String,
    pub fiscal_year: i32,
    pub period_start: String,
    pub period_end: String,
    /// Previous fiscal year end (instant context). Empty => computed from year-1.
    pub prior_period_end: String,
}

impl QaydCompany {
    /// YYYY-MM-DD for the last day of the prior fiscal year.
    pub fn prior_end(&self) -> String {
        if !self.prior_period_end.is_empty() {
            return self.prior_period_end.clone();
        }
        format!("{}-12-31", self.fiscal_year - 1)
    }
}

/// One reported fact: an IFRS concept in a given context with a monetary value.
#[derive(Debug, Clone)]
pub struct QaydFact {
    pub concept: String,
    pub value: f64,
    pub context_ref: &'static str,
    pub decimals: u32,
}

impl QaydFact {
    fn new(concept: &str, value: f64, context_ref: &'static str) -> Self {
        Self { concept: concept.to_string(), value, context_ref, decimals: 2 }
    }
}

/// Aggregated financial statement inputs (in reporting-currency units).
#[derive(Debug, Clone, Default)]
pub struct QaydStatementInputs {
    // Statement of financial position (instant_current).
    pub cash: f64,
    pub receivables: f64,
    pub inventory: f64,
    pub other_current_assets: f64,
    pub noncurrent_assets: f64,
    pub vat_payable: f64,
    pub payables: f64,
    pub other_current_liabilities: f64,
    pub noncurrent_liabilities: f64,
    pub capital: f64,
    pub retained_earnings_prior: f64,
    // Statement of profit or loss (duration_current).
    pub revenue: f64,
    pub cost_of_sales: f64,
    pub other_income: f64,
    pub admin_expenses: f64,
    // Statement of cash flows (duration_current). All optional.
    pub cash_flow_operating: Option<f64>,
    pub cash_flow_investing: Option<f64>,
    pub cash_flow_financing: Option<f64>,
}

/// Derived statement of financial position figures.
#[derive(Debug, Clone, Copy)]
pub struct BalanceSheet {
    pub current_assets: f64,
    pub total_assets: f64,
    pub current_liabilities: f64,
    pub total_liabilities: f64,
    pub retained_earnings: f64,
    pub total_equity: f64,
    pub total_liabilities_and_equity: f64,
}

impl QaydStatementInputs {
    pub fn net_profit(&self) -> f64 {
        self.revenue + self.other_income - self.cost_of_sales - self.admin_expenses
    }

    pub fn gross_profit(&self) -> f64 {
        self.revenue - self.cost_of_sales
    }

    pub fn balance_sheet(&self) -> BalanceSheet {
        let current_assets = self.cash + self.receivables + self.inventory + self.other_current_assets;
        let total_assets = current_assets + self.noncurrent_assets;
        let current_liabilities =
            self.vat_payable + self.payables + self.other_current_liabilities;
        let total_liabilities = current_liabilities + self.noncurrent_liabilities;
        let retained_earnings = self.retained_earnings_prior + self.net_profit();
        let total_equity = self.capital + retained_earnings;
        BalanceSheet {
            current_assets,
            total_assets,
            current_liabilities,
            total_liabilities,
            retained_earnings,
            total_equity,
            total_liabilities_and_equity: total_liabilities + total_equity,
        }
    }

    /// Build the ordered list of facts (balance sheet, P&L, cash flow).
    pub fn to_facts(&self) -> Vec<QaydFact> {
        let bs = self.balance_sheet();
        let mut facts = Vec::new();
        // Statement of financial position.
        for (concept, v) in [
            ("CashAndCashEquivalents", self.cash),
            ("TradeAndOtherReceivablesCurrent", self.receivables),
            ("Inventories", self.inventory),
            ("OtherCurrentAssets", self.other_current_assets),
            ("CurrentAssets", bs.current_assets),
            ("NoncurrentAssets", self.noncurrent_assets),
            ("Assets", bs.total_assets),
            ("CurrentTaxLiabilities", self.vat_payable),
            ("TradeAndOtherPayablesCurrent", self.payables),
            ("OtherCurrentLiabilities", self.other_current_liabilities),
            ("CurrentLiabilities", bs.current_liabilities),
            ("NoncurrentLiabilities", self.noncurrent_liabilities),
            ("Liabilities", bs.total_liabilities),
            ("IssuedCapital", self.capital),
            ("RetainedEarnings", bs.retained_earnings),
            ("Equity", bs.total_equity),
            ("LiabilitiesAndEquity", bs.total_liabilities_and_equity),
        ] {
            facts.push(QaydFact::new(concept, v, "instant_current"));
        }
        // Statement of profit or loss.
        for (concept, v) in [
            ("Revenue", self.revenue),
            ("GrossProfit", self.gross_profit()),
            ("OtherIncome", self.other_income),
            ("AdministrativeExpenses", self.admin_expenses),
            ("ProfitLoss", self.net_profit()),
        ] {
            facts.push(QaydFact::new(concept, v, "duration_current"));
        }
        // Statement of cash flows.
        if let Some(v) = self.cash_flow_operating {
            facts.push(QaydFact::new("NetCashFlowsFromUsedInOperatingActivities", v, "duration_current"));
        }
        if let Some(v) = self.cash_flow_investing {
            facts.push(QaydFact::new("NetCashFlowsFromUsedInInvestingActivities", v, "duration_current"));
        }
        if let Some(v) = self.cash_flow_financing {
            facts.push(QaydFact::new("NetCashFlowsFromUsedInFinancingActivities", v, "duration_current"));
        }
        if self.cash_flow_operating.is_some() {
            let total = self.cash_flow_operating.unwrap_or(0.0)
                + self.cash_flow_investing.unwrap_or(0.0)
                + self.cash_flow_financing.unwrap_or(0.0);
            facts.push(QaydFact::new("IncreaseDecreaseInCashAndCashEquivalents", total, "duration_current"));
        }
        facts
    }
}

/// Map the application's seeded chart-of-account codes to IFRS concepts.
/// The codes come from the migration-30 seed:
///   1100/1101 cash & bank, 1200 receivables, 1320 employee advances,
///   1400 inventory, 2100 VAT payable, 2200 payables, 3100 capital,
///   3200 retained earnings, 4100 sales revenue, 4200 other income,
///   5100 COGS, 5200 admin expenses.
pub fn coa_concept_map() -> BTreeMap<&'static str, &'static str> {
    let mut m = BTreeMap::new();
    m.insert("1100", "CashAndCashEquivalents");
    m.insert("1101", "CashAndCashEquivalents");
    m.insert("1200", "TradeAndOtherReceivablesCurrent");
    m.insert("1320", "TradeAndOtherReceivablesCurrent");
    m.insert("1400", "Inventories");
    m.insert("2100", "CurrentTaxLiabilities");
    m.insert("2200", "TradeAndOtherPayablesCurrent");
    m.insert("3100", "IssuedCapital");
    m.insert("3200", "RetainedEarnings");
    m.insert("4100", "Revenue");
    m.insert("4200", "OtherIncome");
    m.insert("5100", "CostsOfSales");
    m.insert("5200", "AdministrativeExpenses");
    m
}

/// Aggregate a (code, closing balance) list into statement inputs.
/// `prior_retained_earnings` comes from the opening balance of equity.
pub fn inputs_from_account_balances(
    balances: &[(String, f64)],
    prior_retained_earnings: f64,
) -> QaydStatementInputs {
    let mut out = QaydStatementInputs {
        retained_earnings_prior: prior_retained_earnings,
        ..Default::default()
    };
    for (code, amount) in balances {
        match coa_concept_map().get(code.as_str()).copied() {
            Some("CashAndCashEquivalents") => out.cash += amount,
            Some("TradeAndOtherReceivablesCurrent") => out.receivables += amount,
            Some("Inventories") => out.inventory += amount,
            Some("CurrentTaxLiabilities") => out.vat_payable += amount,
            Some("TradeAndOtherPayablesCurrent") => out.payables += amount,
            Some("IssuedCapital") => out.capital += amount,
            Some("RetainedEarnings") => {}
            Some("Revenue") => out.revenue += amount,
            Some("OtherIncome") => out.other_income += amount,
            Some("CostsOfSales") => out.cost_of_sales += amount,
            Some("AdministrativeExpenses") => out.admin_expenses += amount,
            _ => out.other_current_assets += amount,
        }
    }
    out
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn money(v: f64) -> String {
    format!("{:.2}", v)
}

/// Build a complete XBRL instance document for a Qayd filing.
pub fn build_instance(company: &QaydCompany, facts: &[QaydFact]) -> String {
    let currency = if company.currency.is_empty() { "KWD" } else { &company.currency };
    let mut s = String::new();
    s.push_str(&format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<xbrli:xbrl xmlns:xbrli=\"{}\" xmlns:ifrs-full=\"{}\" xmlns:iso4217=\"{}\">\n",
        XBRLI_NS, IFRS_FULL_NS, ISO4217_NS
    ));
    // Entity + contexts.
    s.push_str("<xbrli:context id=\"instant_prior\"><xbrli:entity>");
    s.push_str(&format!("<xbrli:identifier scheme=\"{}\">{}</xbrli:identifier>", KUWAIT_CR_SCHEME, esc(&company.cr_number)));
    s.push_str("</xbrli:entity>");
    s.push_str(&format!("<xbrli:period><xbrli:instant>{}</xbrli:instant></xbrli:period></xbrli:context>\n", company.prior_end()));
    s.push_str("<xbrli:context id=\"instant_current\"><xbrli:entity>");
    s.push_str(&format!("<xbrli:identifier scheme=\"{}\">{}</xbrli:identifier>", KUWAIT_CR_SCHEME, esc(&company.cr_number)));
    s.push_str("</xbrli:entity>");
    s.push_str(&format!("<xbrli:period><xbrli:instant>{}</xbrli:instant></xbrli:period></xbrli:context>\n", company.period_end));
    s.push_str("<xbrli:context id=\"duration_current\"><xbrli:entity>");
    s.push_str(&format!("<xbrli:identifier scheme=\"{}\">{}</xbrli:identifier>", KUWAIT_CR_SCHEME, esc(&company.cr_number)));
    s.push_str("</xbrli:entity>");
    s.push_str(&format!(
        "<xbrli:period><xbrli:startDate>{}</xbrli:startDate><xbrli:endDate>{}</xbrli:endDate></xbrli:period></xbrli:context>\n",
        company.period_start, company.period_end
    ));
    // Unit.
    s.push_str(&format!("<xbrli:unit id=\"iso4217-{}\"><xbrli:measure>iso4217:{}</xbrli:measure></xbrli:unit>\n", currency, currency));
    // Facts.
    for f in facts {
        s.push_str(&format!(
            "<ifrs-full:{} contextRef=\"{}\" decimals=\"{}\" unitRef=\"iso4217-{}\">{}</ifrs-full:{}>\n",
            esc(&f.concept), f.context_ref, f.decimals, currency, money(f.value), esc(&f.concept)
        ));
    }
    s.push_str("</xbrli:xbrl>\n");
    s
}

/// Minimal semantic validation of a generated instance.
pub fn validate_instance(xml: &str, company: &QaydCompany) -> Vec<String> {
    let mut errors = Vec::new();
    if !xml.starts_with("<?xml version=\"1.0\"") {
        errors.push("instance must begin with an XML declaration".into());
    }
    if !xml.contains("xbrli:xbrl") || !xml.contains(XBRLI_NS) {
        errors.push("missing xbrli root".into());
    }
    if !xml.contains(IFRS_FULL_NS) {
        errors.push("missing ifrs-full taxonomy namespace".into());
    }
    if !xml.contains(&company.cr_number) {
        errors.push("entity identifier (CR number) is missing".into());
    }
    for required in [
        "instant_current", "instant_prior", "duration_current",
        "Assets", "Liabilities", "Equity", "ProfitLoss", "Revenue",
    ] {
        if !xml.contains(required) {
            errors.push(format!("missing required element/context: {}", required));
        }
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_company() -> QaydCompany {
        QaydCompany {
            name_ar: "شركة الأمثلة الكويتية".into(),
            cr_number: "12345678".into(),
            currency: "KWD".into(),
            fiscal_year: 2026,
            period_start: "2026-01-01".into(),
            period_end: "2026-12-31".into(),
            prior_period_end: String::new(),
        }
    }

    fn sample_inputs() -> QaydStatementInputs {
        QaydStatementInputs {
            cash: 1000.0,
            receivables: 500.0,
            inventory: 300.0,
            other_current_assets: 0.0,
            noncurrent_assets: 200.0,
            vat_payable: 60.0,
            payables: 250.0,
            other_current_liabilities: 0.0,
            noncurrent_liabilities: 0.0,
            capital: 1000.0,
            retained_earnings_prior: 400.0,
            revenue: 2000.0,
            cost_of_sales: 1560.0,
            other_income: 50.0,
            admin_expenses: 200.0,
            cash_flow_operating: Some(400.0),
            cash_flow_investing: Some(-100.0),
            cash_flow_financing: Some(0.0),
        }
    }

    #[test]
    fn balance_sheet_reconciles() {
        let i = sample_inputs();
        let bs = i.balance_sheet();
        assert_eq!(bs.current_assets, 1800.0);
        assert_eq!(bs.total_assets, 2000.0);
        assert_eq!(bs.current_liabilities, 310.0);
        assert_eq!(bs.retained_earnings, 690.0);
        assert_eq!(bs.total_equity, 1690.0);
        assert!((bs.total_assets - bs.total_liabilities_and_equity).abs() < 1e-9);
    }

    #[test]
    fn instance_is_valid_and_reconciles() {
        let company = sample_company();
        let facts = sample_inputs().to_facts();
        let xml = build_instance(&company, &facts);
        let errs = validate_instance(&xml, &company);
        assert!(errs.is_empty(), "{:?}", errs);
        // Facts carry the right context/unit.
        assert!(xml.contains("<ifrs-full:Assets contextRef=\"instant_current\" decimals=\"2\" unitRef=\"iso4217-KWD\">2000.00</ifrs-full:Assets>"));
        assert!(xml.contains("<ifrs-full:ProfitLoss contextRef=\"duration_current\" decimals=\"2\" unitRef=\"iso4217-KWD\">290.00</ifrs-full:ProfitLoss>"));
        // Prior instant context present.
        assert!(xml.contains("<xbrli:instant>2025-12-31</xbrli:instant>"));
        // Entity identifier.
        assert!(xml.contains("<xbrli:identifier scheme=\"https://www.qayd.gov.kw/cr\">12345678</xbrli:identifier>"));
    }

    #[test]
    fn coa_mapping_covers_seeded_chart() {
        let map = coa_concept_map();
        for code in ["1100", "1101", "1200", "1320", "1400", "2100", "2200", "3100", "4100", "4200", "5100", "5200"] {
            assert!(map.contains_key(code), "missing mapping for {}", code);
        }
    }

    #[test]
    fn coa_balances_aggregate_correctly() {
        let balances = vec![
            ("1100".to_string(), 600.0),
            ("1101".to_string(), 400.0),
            ("1400".to_string(), 300.0),
            ("4100".to_string(), 2000.0),
            ("5100".to_string(), 1200.0),
            ("5200".to_string(), 200.0),
        ];
        let i = inputs_from_account_balances(&balances, 400.0);
        assert_eq!(i.cash, 1000.0);
        assert_eq!(i.inventory, 300.0);
        assert_eq!(i.revenue, 2000.0);
        assert_eq!(i.cost_of_sales, 1200.0);
        assert_eq!(i.admin_expenses, 200.0);
        assert_eq!(i.net_profit(), 600.0);
    }

    #[test]
    fn unknown_accounts_land_in_other_current_assets() {
        let balances = vec![("9999".to_string(), 42.0)];
        let i = inputs_from_account_balances(&balances, 0.0);
        assert_eq!(i.other_current_assets, 42.0);
    }

    #[test]
    fn validate_flags_broken_documents() {
        let company = sample_company();
        let facts = sample_inputs().to_facts();
        let xml = build_instance(&company, &facts);
        let bad = xml.replace(&company.cr_number, "XXXX");
        let errs = validate_instance(&bad, &company);
        assert!(errs.iter().any(|e| e.contains("CR number")));
        let empty = String::new();
        let errs = validate_instance(&empty, &company);
        assert!(!errs.is_empty());
    }
}
