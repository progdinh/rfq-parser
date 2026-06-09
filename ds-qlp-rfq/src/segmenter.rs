use regex::Regex;
use std::sync::LazyLock;

// ── Marker patterns ───────────────────────────────────────────────────────────

/// Numbered list markers: "1. ", "2) "
static RE_NUMBERED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*\d+[.)]\s+").unwrap()
});

/// Bullet markers: "- ", "• ", "* ", "· "
static RE_BULLET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*[-•*·]\s+").unwrap()
});

/// Any RFQ-like signal (used by is_rfq)
static RE_DETECT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?mi)^\s*\d+[.)]\s|^\s*[-•*·]\s|\btarget\s+\d|\b(?:payment\s+terms?|lead[\s-]?time)\b"
    ).unwrap()
});

/// Bare-line heuristic: line that looks like a product entry
static RE_PRODUCT_LINE: LazyLock<Regex> = LazyLock::new(|| {
    // Starts with a digit, or contains a common unit word
    Regex::new(
        r"(?i)^\s*\d|\b(?:pcs|pieces?|sets?|kg|tons?|rolls?|sheets?|drums?|pallets?|cartons?|reels?|boxes?|meters?|litres?|liters?|fcl)\b"
    ).unwrap()
});

/// Bare-line heuristic: line that opens the trade-terms footer
static RE_FOOTER_LINE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)^\s*(?:lead[\s-]?time|payment\s+terms?|incoterms?|delivery\s+(?:time|terms?)|shipping\s+(?:time|terms?)|port\s+of\s+(?:loading|discharge))"
    ).unwrap()
});

// ── Public types ──────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub struct RfqSegments {
    pub preamble: String,
    pub items:    Vec<String>,
    pub footer:   String,
}

// ── Public API ────────────────────────────────────────────────────────────────

pub fn is_rfq(input: &str) -> bool {
    RE_DETECT.is_match(input)
}

/// Segment an RFQ into preamble / items / footer.
///
/// Strategy cascade:
///   1. Numbered markers (`1.`, `2)`)
///   2. Bullet markers (`-`, `•`, `*`, `·`)
///   3. Bare-line fallback (heuristic: product-like lines between preamble and footer)
pub fn segment(input: &str) -> RfqSegments {
    if RE_NUMBERED.is_match(input) {
        return segment_by_markers(input, &RE_NUMBERED);
    }
    if RE_BULLET.is_match(input) {
        return segment_by_markers(input, &RE_BULLET);
    }
    segment_bare_lines(input)
}

// ── Segmentation strategies ───────────────────────────────────────────────────

/// Generic marker-based segmentation (works for both numbered and bullet lists).
/// An "item" is the text between two consecutive markers.
fn segment_by_markers(input: &str, re: &Regex) -> RfqSegments {
    let positions: Vec<(usize, usize)> = re.find_iter(input).map(|m| (m.start(), m.end())).collect();

    if positions.is_empty() {
        return RfqSegments { preamble: input.to_string(), items: vec![], footer: String::new() };
    }

    let preamble = input[..positions[0].0].trim().to_string();

    let mut items = Vec::new();
    for i in 0..positions.len() {
        let content_start = positions[i].1;
        let content_end   = if i + 1 < positions.len() { positions[i + 1].0 } else { input.len() };
        let raw = input[content_start..content_end].trim();
        if !raw.is_empty() { items.push(raw.to_string()); }
    }

    let footer = match items.last() {
        Some(last) => {
            let last_item_end = input.rfind(last.as_str())
                .map(|p| p + last.len())
                .unwrap_or(input.len());
            input[last_item_end..].trim().to_string()
        }
        None => String::new(),
    };

    RfqSegments { preamble, items, footer }
}

/// Bare-line fallback: no markers found.
/// Preamble = lines before the first product-like line.
/// Items     = product-like lines (each line = one item).
/// Footer    = lines starting with a trade-term keyword.
fn segment_bare_lines(input: &str) -> RfqSegments {
    let mut preamble_lines: Vec<&str> = Vec::new();
    let mut item_lines:     Vec<String> = Vec::new();
    let mut footer_lines:   Vec<&str> = Vec::new();
    let mut in_items = false;
    let mut in_footer = false;

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }

        if in_footer {
            footer_lines.push(trimmed);
        } else if RE_FOOTER_LINE.is_match(trimmed) {
            in_footer = true;
            footer_lines.push(trimmed);
        } else if in_items {
            item_lines.push(trimmed.to_string());
        } else if RE_PRODUCT_LINE.is_match(trimmed) {
            in_items = true;
            item_lines.push(trimmed.to_string());
        } else {
            preamble_lines.push(trimmed);
        }
    }

    RfqSegments {
        preamble: preamble_lines.join("\n"),
        items:    item_lines,
        footer:   footer_lines.join("\n"),
    }
}
