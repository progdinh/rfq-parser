use regex::Regex;
use std::sync::LazyLock;

static RE_CURRENCY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(USD|EUR|GBP|CNY|JPY|KRW|VND|SGD|AUD|CAD)\b").unwrap()
});
// Match incoterm + following city (Title-cased): "FCA Sydney", "DAP Jakarta"
// Stops at word boundary naturally since city is [A-Z][a-zA-Z]+ (no digits/punct)
static RE_INCOTERM_CITY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(EXW|FCA|FAS|FOB|CFR|CIF|CPT|CIP|DAP|DPU|DDP)\s+([A-Z][a-zA-Z]+(?:\s+[A-Z][a-zA-Z]+)?)").unwrap()
});
static RE_INCOTERM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(EXW|FCA|FAS|FOB|CFR|CIF|CPT|CIP|DAP|DPU|DDP)\b").unwrap()
});
static RE_DESTINATION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(?:to|for|deliver(?:y)? to|destination)\s+([A-Z][a-zA-Z\s]{2,30})(?:[,.]|$)").unwrap()
});
static RE_LEAD_TIME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)lead[\s-]?time[:\s]+([^.\n]+)").unwrap()
});
// English: "payment terms: T/T ..." — Vietnamese: "thanh toán T/T trả trước"
static RE_PAYMENT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:payment(?:\s+terms?)?|thanh\s+toán)[:\s]+([^.\n]+)").unwrap()
});

#[derive(Debug, Clone, Default)]
pub struct TradeTerms {
    pub currency:    Option<String>,
    pub incoterm:    Option<String>,
    pub destination: Option<String>,
    pub lead_time:   Option<String>,
    pub payment:     Option<String>,
}

impl TradeTerms {
    pub fn extract(text: &str) -> Self {
        let mut t = TradeTerms::default();

        if let Some(m) = RE_CURRENCY.find(text) {
            t.currency = Some(m.as_str().to_string());
        }
        // Try "INCOTERM City" first — extracts both incoterm and destination in one pass.
        // Falls back to bare incoterm match if no city follows.
        if let Some(cap) = RE_INCOTERM_CITY.captures(text) {
            t.incoterm     = Some(cap[1].to_string());
            t.destination  = Some(cap[2].trim().to_string());
        } else if let Some(m) = RE_INCOTERM.find(text) {
            t.incoterm = Some(m.as_str().to_string());
        }
        // Explicit destination keyword overrides incoterm-derived destination
        if let Some(cap) = RE_DESTINATION.captures(text) {
            t.destination = Some(cap[1].trim().to_string());
        }
        if let Some(cap) = RE_LEAD_TIME.captures(text) {
            t.lead_time = Some(cap[1].trim().trim_end_matches('.').to_string());
        }
        if let Some(cap) = RE_PAYMENT.captures(text) {
            t.payment = Some(cap[1].trim().trim_end_matches('.').to_string());
        }
        t
    }

    pub fn to_dsl(&self) -> String {
        let mut out = String::from("TRADE_TERMS:\n");
        out.push_str(&fmt_field("  CURRENCY", &self.currency));
        out.push_str(&fmt_field("  INCOTERM", &self.incoterm));
        out.push_str(&fmt_field("  DESTINATION", &self.destination));
        out.push_str(&fmt_field("  LEAD_TIME", &self.lead_time));
        out.push_str(&fmt_field("  PAYMENT", &self.payment));
        out
    }

    pub fn is_empty(&self) -> bool {
        self.currency.is_none()
            && self.incoterm.is_none()
            && self.destination.is_none()
            && self.lead_time.is_none()
            && self.payment.is_none()
    }
}

fn fmt_field(label: &str, val: &Option<String>) -> String {
    match val {
        Some(v) => format!("{}: {}\n", label, v),
        None    => format!("{}: null\n", label),
    }
}
