use regex::Regex;
use std::sync::LazyLock;

static RE_PRICE: LazyLock<Regex> = LazyLock::new(|| {
    // Currency optional. Handles English "target"/"@" and Vietnamese "giá mục tiêu".
    // Trailing /unit (e.g. "2.5 SGD/cái") is consumed so it doesn't leak into chunks.
    Regex::new(
        r"(?i)[,(]?\s*(?:target\s*[:=]?\s*|giá\s+mục\s+tiêu\s*|@\s*)(\d+(?:[.,]\d+)?)\s*(USD|EUR|GBP|CNY|JPY|VND|SGD)?\s*(?:/[^\s,).]+)?\s*[.)]?"
    ).unwrap()
});

// Strip markdown bold/italic wrappers: **text** → text
static RE_MARKDOWN_BOLD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\*{1,3}([^*]+)\*{1,3}").unwrap()
});

// Structured EN RFQ field markers — strip prefix, keep content
static RE_PRODUCT_MARKER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^\s*product\s*:\s*").unwrap()
});
static RE_SPEC_MARKER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i),?\s*spec\s*:\s*").unwrap()
});
// Non-numeric qty marker ("Qty: N/A Liters") — strip entirely, no qty to extract
static RE_QTY_NA: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i),?\s*qty\s*:\s*(?:n/a|nil|tbd|na)(?:\s+\w+)?").unwrap()
});
// Orphaned closing parens left after origin or spec extraction
static RE_EMPTY_PAREN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\(\s*[,\s]*\)|\(\s*:\s*\)").unwrap()
});

// English units (word-boundary safe)
static RE_QTY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i),?\s*(\d+(?:[.,]\d+)?)\s*(tons?|kg|kilograms?|pcs|pieces?|sets?|cartons?|drums?|reels?|pallets?|meters?|yards?|liters?|litres?|box|boxes|hundreds?|thousands?|rolls?|sheets?|20ft\s*fcl|fcl)\b\.?"
    ).unwrap()
});

// Vietnamese units — no \b because some units end in diacritics (e.g. bộ, bó).
// The required leading digit is enough to avoid false positives.
static RE_QTY_VI: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r",?\s*(\d+(?:[.,]\d+)?)\s*(cái|bộ|tấn|thùng|bó|hộp|tấm|cuộn|kiện)"
    ).unwrap()
});

// Vietnamese unit → English normalisation
const VI_UNIT_MAP: &[(&str, &str)] = &[
    ("cái",  "pcs"),
    ("bộ",   "sets"),
    ("tấn",  "tonnes"),
    ("thùng","cartons"),
    ("bó",   "bundles"),
    ("hộp",  "boxes"),
    ("tấm",  "sheets"),
    ("cuộn", "rolls"),
    ("kiện", "bales"),
];

// Inline measurement specs: dimensions (180x200cm), thickness (1.0mm),
// electrical (12V, 200Ah, 100kW), volume (330ml), weight (25kg)
static RE_INLINE_SPEC: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i),?\s*(\d+(?:[.,]\d+)?(?:\s*[xX×]\s*\d+(?:[.,]\d+)?)*\s*(?:mm|cm|m\b|oz|kg\b|ml\b|l\b|kw\b|mw\b|kva\b|v\b|ah\b|w\b|khz\b|hz\b|mhz\b|rpm\b|psi\b|mpa\b|bar\b|inch(?:es)?\b))"
    ).unwrap()
});

// English origin: ", country"
const EN_COUNTRIES: &[&str] = &[
    "china", "vietnam", "indonesia", "india", "malaysia", "thailand",
    "south korea", "korea", "japan", "bangladesh", "cambodia", "philippines",
    "brazil", "germany", "turkey", "spain", "peru", "australia",
    "sri lanka", "myanmar", "pakistan", "taiwan",
];

// Vietnamese origin: "xuất xứ <country>" → English name
static RE_VI_ORIGIN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"xuất\s+xứ\s+([^\s,;:.()0-9][^,;:.()0-9]*)").unwrap()
});

const VI_COUNTRIES: &[(&str, &str)] = &[
    ("đức",         "Germany"),
    ("trung quốc",  "China"),
    ("tq",          "China"),       // informal abbreviation
    ("hàn quốc",    "South Korea"),
    ("nhật bản",    "Japan"),
    ("ấn độ",       "India"),
    ("việt nam",    "Vietnam"),
    ("vn",          "Vietnam"),     // informal abbreviation
    ("campuchia",   "Cambodia"),
    ("thái lan",    "Thailand"),
    ("đài loan",    "Taiwan"),
    ("indonesia",   "Indonesia"),
    ("malaysia",    "Malaysia"),
    ("philippines", "Philippines"),
    ("myanmar",     "Myanmar"),
    ("thổ nhĩ kỳ", "Turkey"),
    ("úc",          "Australia"),
    ("mỹ",          "USA"),
];

#[derive(Debug, Clone, Default)]
pub struct NormalizedItem {
    pub description: String,
    pub qty:         Option<f64>,
    pub uom:         Option<String>,
    pub price:       Option<f64>,
    pub currency:    Option<String>,
    pub origin:      Option<String>,
    pub specs:       Vec<String>,
}

pub fn normalize(raw: &str) -> NormalizedItem {
    let mut item = NormalizedItem::default();

    // 0. Strip markdown bold/italic wrappers (**text** → text)
    let raw = RE_MARKDOWN_BOLD.replace_all(raw.trim(), "$1");
    let mut text = raw.trim_end_matches('.').to_string();

    // 0.5. Structured EN RFQ field markers
    // "Product: Car Battery, 3000 Cartons, Spec: Black, 70Ah" →
    //   "Car Battery, 3000 Cartons, Black, 70Ah"
    // The "Spec:" label is stripped but its comma-separated values remain as
    // regular text for qlp to classify (color, material, standard, etc.).
    // "Qty: N/A Liters" is removed — no extractable quantity.
    text = RE_PRODUCT_MARKER.replace(&text, "").to_string();
    text = RE_SPEC_MARKER.replace_all(&text, ", ").to_string();
    text = RE_QTY_NA.replace_all(&text, "").to_string();

    // 1. Extract target price (English or Vietnamese)
    if let Some(cap) = RE_PRICE.captures(&text.clone()) {
        item.price    = cap[1].replace(',', "").parse().ok();
        item.currency = cap.get(2).map(|m| m.as_str().to_uppercase());
        text = text.replacen(cap.get(0).unwrap().as_str(), "", 1);
    }

    // 2. Extract QTY + UOM — English first, Vietnamese fallback
    let text_lower = text.to_lowercase();
    if let Some(cap) = RE_QTY.captures_iter(&text_lower).last() {
        let full = cap.get(0).unwrap().as_str();
        item.qty = cap[1].replace(',', "").parse().ok();
        item.uom = Some(cap[2].trim().to_string());
        let start = text_lower.rfind(full).unwrap_or(text.len());
        if start < text.len() {
            text = format!("{}{}", &text[..start], &text[start + full.len()..]);
        }
    } else if let Some(cap) = RE_QTY_VI.captures_iter(&text_lower).last() {
        let full = cap.get(0).unwrap().as_str();
        item.qty = cap[1].replace(',', "").parse().ok();
        let vi_unit = cap[2].trim();
        item.uom = Some(
            VI_UNIT_MAP.iter()
                .find(|(vi, _)| *vi == vi_unit)
                .map(|(_, en)| en.to_string())
                .unwrap_or_else(|| vi_unit.to_string())
        );
        let start = text_lower.rfind(full).unwrap_or(text.len());
        if start < text.len() {
            text = format!("{}{}", &text[..start], &text[start + full.len()..]);
        }
    }

    // 3. Origin extraction — three strategies in priority order:
    //   a) Vietnamese "xuất xứ X" explicit marker
    //   b) VI_COUNTRIES in parenthetical prefix: "(TQ, ...)" "(Nhật Bản, ...)"
    //   c) EN_COUNTRIES after comma or paren: ", Japan" or "(Japan"
    let mut origin_found = false;

    // 3a. Vietnamese explicit: "xuất xứ Đức" etc.
    {
        let lower = text.to_lowercase();
        if let Some(cap) = RE_VI_ORIGIN.captures(&lower) {
            let vi_name = cap[1].trim().to_string();
            let english = VI_COUNTRIES.iter()
                .find(|(vi, _)| lower.contains(*vi))
                .map(|(_, en)| en.to_string())
                .unwrap_or_else(|| capitalize(&vi_name));
            item.origin = Some(english);
            let full = cap.get(0).unwrap().as_str();
            let start = lower.find(full).unwrap();
            text = format!("{}{}", &text[..start], &text[start + full.len()..]);
            origin_found = true;
        }
    }

    // 3b. VI abbreviations/names in parenthetical prefix.
    // Handles "(TQ, an toàn)", "(Nhật Bản, cần OEM)", "(VN)" etc.
    if !origin_found {
        let lower = text.to_lowercase();
        'vi_paren: for &(vi, en) in VI_COUNTRIES {
            // Try "(vi_name, rest...)" — country is first item in a paren list
            let with_comma = format!("({},", vi);
            // Try "(vi_name)" — country is the sole paren content
            let exact = format!("({})", vi);
            for (pat, keep_paren) in [(&with_comma, true), (&exact, false)] {
                if let Some(pos) = lower.find(pat.as_str()) {
                    item.origin = Some(en.to_string());
                    let match_end = pos + pat.len();
                    if keep_paren {
                        // Find the closing ')' for the remaining paren content
                        let after_country = &text[match_end..];
                        if let Some(close_rel) = after_country.find(')') {
                            let inner = after_country[..close_rel].trim();
                            if inner.is_empty() || inner == "," {
                                // Nothing left in parens — remove entire group including ')'
                                let full_end = match_end + close_rel + 1;
                                text = format!("{}{}", &text[..pos], &text[full_end..]);
                            } else {
                                // Keep remaining content as new paren group
                                let inner_clean = inner.trim_start_matches(',').trim().to_string();
                                let full_end = match_end + close_rel + 1;
                                text = format!("{}({}){}", &text[..pos], inner_clean, &text[full_end..]);
                            }
                        } else {
                            // No closing paren — just strip the prefix
                            text = format!("{}{}", &text[..pos], &text[match_end..]);
                        }
                    } else {
                        // Exact match "(vi_name)" — remove entirely
                        text = format!("{}{}", &text[..pos], &text[match_end..]);
                    }
                    origin_found = true;
                    break 'vi_paren;
                }
            }
        }
    }

    // 3c. EN country names after comma or opening paren
    if !origin_found {
        let lower = text.to_lowercase();
        'en_origin: for &country in EN_COUNTRIES {
            for prefix in [", ", "("] {
                let pattern = format!("{}{}", prefix, country);
                if let Some(pos) = lower.find(&pattern) {
                    let end = pos + pattern.len();
                    let after = &lower[end..];
                    // Verify word boundary after country name
                    if after.is_empty() || after.starts_with(|c: char| !c.is_alphanumeric()) {
                        item.origin = Some(capitalize(country));
                        text = format!("{}{}", &text[..pos], &text[end..]);
                        origin_found = true;
                        break 'en_origin;
                    }
                }
            }
        }
    }

    // Clean up orphaned parens left by origin extraction: "()" "(, spec)"
    text = RE_EMPTY_PAREN.replace_all(&text, "").to_string();
    let _ = origin_found; // suppress unused-variable warning after cleanups

    // 3.5. Extract inline measurement specs
    loop {
        let lower = text.to_lowercase();
        if let Some(cap) = RE_INLINE_SPEC.captures(&lower) {
            let spec = cap[1].trim().to_string();
            let full = cap.get(0).unwrap().as_str();
            let start = lower.find(full).unwrap();
            text = format!("{}{}", &text[..start], &text[start + full.len()..]);
            item.specs.push(spec);
        } else {
            break;
        }
    }

    // 4. Replace commas with spaces, collapse whitespace.
    // Also strip trailing `:` left by the Vietnamese "(specs): qty" separator pattern
    // after qty extraction has already consumed the quantity.
    let cleaned = text
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let cleaned = cleaned.trim_end_matches(':').trim().to_string();

    // 5. Prepend qty so qlp finds it as a leading Num
    item.description = match (&item.qty, &item.uom) {
        (Some(q), Some(u)) => format!("{} {} {}", fmt_qty(*q), u, cleaned),
        (Some(q), None)    => format!("{} {}", fmt_qty(*q), cleaned),
        _                  => cleaned,
    };

    item
}

fn fmt_qty(v: f64) -> String {
    if v.fract() == 0.0 { format!("{}", v as u64) } else { format!("{:.2}", v) }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None    => String::new(),
        Some(f) => f.to_uppercase().to_string() + c.as_str(),
    }
}
