// lexer.rs — rule-based query lexer
//
// Critical design decisions:
//   - NO has_qty flag: all numbers emitted as Num (multi-group queries need multiple Nums)
//   - NO Grade detection: parser handles grade vs qty via R6 grammar rule
//   - after_size_kw: numbers after a SizeKwd → try Size map first, else Sytg (not Num)
//   - Byte-based pos tracking throughout (critical for Vietnamese UTF-8)

use crate::ontology::Ontology;
use crate::token::{ConnOp, Tag, Unit};
use regex::Regex;
use std::sync::LazyLock;

static QTY_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d+(?:[.,]\d+)?)").unwrap());

/// Returns true if the input contains any Vietnamese diacritical character.
/// Vietnamese tonal marks and special letters (ă, đ, ơ, ư and all tone variants)
/// are unambiguous — no overlap with English or common Latin text.
fn is_vietnamese(s: &str) -> bool {
    s.chars().any(|c| matches!(c,
        'ă'|'ắ'|'ằ'|'ẳ'|'ẵ'|'ặ'|
        'â'|'ấ'|'ầ'|'ẩ'|'ẫ'|'ậ'|
        'đ'|
        'ê'|'ế'|'ề'|'ể'|'ễ'|'ệ'|
        'ô'|'ố'|'ồ'|'ổ'|'ỗ'|'ộ'|
        'ơ'|'ớ'|'ờ'|'ở'|'ỡ'|'ợ'|
        'ư'|'ứ'|'ừ'|'ử'|'ữ'|'ự'|
        'à'|'á'|'ả'|'ã'|'ạ'|
        'è'|'é'|'ẻ'|'ẽ'|'ẹ'|
        'ì'|'í'|'ỉ'|'ĩ'|'ị'|
        'ò'|'ó'|'ỏ'|'õ'|'ọ'|
        'ù'|'ú'|'ủ'|'ũ'|'ụ'|
        'ỳ'|'ý'|'ỷ'|'ỹ'|'ỵ'
    ))
}

pub struct Lexer<'a> {
    ontology: &'a Ontology,
}

impl<'a> Lexer<'a> {
    pub fn new(ontology: &'a Ontology) -> Self {
        Self { ontology }
    }

    pub fn tokenize(&self, input: &str) -> Vec<Tag> {
        let mut tags: Vec<Tag> = Vec::new();
        let norm = input.trim().to_lowercase();
        let mut pos = 0usize;
        let mut after_size_kw = false; // suppress Num emission after "size" keyword
        // In Vietnamese, modifiers follow the head noun (post-nominal), so we require
        // a Sytg to have been seen before tagging material or gender tokens.
        // In English, pre-nominal modifiers are valid ("red cotton polo shirts"), so the
        // guard is off. Mixed queries containing any Vietnamese diacritic use vi_mode.
        let vi_mode = is_vietnamese(&norm);
        let mut seen_sytg = false;

        while pos < norm.len() {
            if let Some(c) = norm[pos..].chars().next() {
                if c.is_whitespace() { pos += c.len_utf8(); continue; }
            }
            let rest = &norm[pos..];

            // ── 1. Need keywords — consumed silently ─────────────────────────
            if let Some(n) = self.prefix_bytes(rest, &self.ontology.need_keywords) {
                pos += n; continue;
            }

            // ── 2. Quantity — all numbers, UNLESS after size keyword ──────────
            //    If after_size_kw: numbers go through size map or become Sytg
            if after_size_kw {
                // Try to emit as Size first
                if let Some((canon, n)) = self.map_prefix(rest, &self.ontology.sizes) {
                    tags.push(Tag::Size(canon.to_string()));
                    pos += n; continue;
                }
                // Number not in size map → emit as Sytg (raw), not Num
                if let Some(m) = QTY_RE.find(rest) {
                    if is_boundary(&rest[m.end()..]) {
                        tags.push(Tag::Sytg(m.as_str().to_string()));
                        seen_sytg = true;
                        pos += m.end(); continue;
                    }
                }
            } else {
                if let Some((val, unit, n)) = self.match_quantity(rest) {
                    tags.push(Tag::Num { value: val, unit });
                    pos += n; continue;
                }
            }

            // ── 3. Connectors ────────────────────────────────────────────────
            if rest.starts_with(',') || rest.starts_with('/') {
                tags.push(Tag::Conn(ConnOp::Comma));
                after_size_kw = false;
                pos += 1; continue;
            }
            if let Some(n) = self.prefix_bytes(rest, &self.ontology.coord_range) {
                tags.push(Tag::Conn(ConnOp::RangeTo));
                // intentionally keep after_size_kw: the next token is the range endpoint
                pos += n; continue;
            }
            if let Some(n) = self.prefix_bytes(rest, &self.ontology.coord_and) {
                tags.push(Tag::Conn(ConnOp::And));
                after_size_kw = false;
                seen_sytg = false; // new group starts: head-noun requirement resets
                pos += n; continue;
            }
            if let Some(n) = self.prefix_bytes(rest, &self.ontology.coord_or) {
                tags.push(Tag::Conn(ConnOp::Or));
                after_size_kw = false;
                seen_sytg = false; // new group starts: head-noun requirement resets
                pos += n; continue;
            }

            // ── 4. Keyword markers ───────────────────────────────────────────
            if let Some(n) = self.prefix_bytes(rest, &self.ontology.size_keywords) {
                tags.push(Tag::SizeKwd);
                after_size_kw = true;
                pos += n; continue;
            }
            if let Some(n) = self.prefix_bytes(rest, &self.ontology.color_keywords) {
                tags.push(Tag::ColorKwd);
                after_size_kw = false;
                pos += n; continue;
            }
            if let Some(n) = self.prefix_bytes(rest, &self.ontology.mat_keywords) {
                tags.push(Tag::MaterialKwd);
                after_size_kw = false;
                pos += n; continue;
            }

            // ── 5. Proper compound nouns — before attribute matching ──────────
            // Multi-word proper nouns whose sub-words collide with the attribute
            // ontology (e.g. "việt nam" → "nam" = male gender). Longest-match,
            // emitted as Sytg so the full compound lands in the product chunk.
            if let Some(n) = self.prefix_bytes(rest, &self.ontology.proper_compounds) {
                tags.push(Tag::Sytg(rest[..n].to_string()));
                seen_sytg = true;
                after_size_kw = false;
                pos += n; continue;
            }

            // ── 6. Gender — in VI mode, only after a head noun is established ───
            // In Vietnamese, gender words follow the head noun ("áo nam", "giày nữ").
            // Without a prior Sytg in VI mode, "nam" is more likely a product/geography
            // token than a gender modifier. In English mode always allowed.
            if !vi_mode || seen_sytg {
                if let Some((canon, n)) = self.map_prefix(rest, &self.ontology.genders) {
                    tags.push(Tag::Gender(canon.to_string()));
                    after_size_kw = false;
                    pos += n; continue;
                }
            }

            // ── 7. Standards ─────────────────────────────────────────────────
            if let Some((code, n)) = self.match_standard(rest) {
                tags.push(Tag::Standard(code));
                after_size_kw = false;
                pos += n; continue;
            }

            // ── 8. Attribute values ──────────────────────────────────────────
            if let Some((canon, n)) = self.map_prefix(rest, &self.ontology.colors) {
                tags.push(Tag::Color(canon.to_string()));
                after_size_kw = false;
                pos += n; continue;
            }
            if let Some((canon, n)) = self.map_prefix(rest, &self.ontology.sizes) {
                tags.push(Tag::Size(canon.to_string()));
                // Keep after_size_kw active while consuming sizes
                pos += n; continue;
            }
            // In VI mode, material requires a prior head noun — a lone material word
            // at the start of a group is the product itself ("thép", "nhôm", "cao su").
            // In English mode, pre-nominal materials are valid ("cotton t-shirt").
            if !vi_mode || seen_sytg {
                if let Some((canon, n)) = self.map_prefix(rest, &self.ontology.materials) {
                    tags.push(Tag::Material(canon.to_string()));
                    after_size_kw = false;
                    pos += n; continue;
                }
            }

            // ── 8.5. Geographic origin — EN mode only ────────────────────────
            // Known country/region names emitted as Provenance → Group.ambiguous.
            // The SLM decides whether this is a GI origin, a trade-route qualifier,
            // or noise. Not applied in VI mode (proper_compounds covers VI geography).
            if !vi_mode {
                if let Some((canon, n)) = self.map_prefix(rest, &self.ontology.geo_origins) {
                    tags.push(Tag::Provenance(canon.to_string()));
                    after_size_kw = false;
                    pos += n; continue;
                }
            }

            // ── 9. Residual Sytg ─────────────────────────────────────────────
            let word_bytes: usize = rest.chars()
                .take_while(|&c| !c.is_whitespace()
                    && !matches!(c, ',' | '.' | '!' | '?' | ')'))
                .map(|c| c.len_utf8()).sum();
            let word_bytes = word_bytes.max(1);
            tags.push(Tag::Sytg(rest[..word_bytes].to_string()));
            seen_sytg = true;
            after_size_kw = false;
            pos += word_bytes;
        }

        merge_adjacent_sytg(tags)
    }

    fn prefix_bytes(&self, rest: &str, list: &[&str]) -> Option<usize> {
        let mut best: Option<usize> = None;
        for &kw in list {
            if rest.starts_with(kw) && is_boundary(&rest[kw.len()..]) {
                match best {
                    None => best = Some(kw.len()),
                    Some(b) if kw.len() > b => best = Some(kw.len()),
                    _ => {}
                }
            }
        }
        best
    }

    fn map_prefix<'b>(
        &self, rest: &str,
        map: &'b std::collections::HashMap<&'static str, &'static str>,
    ) -> Option<(&'b str, usize)> {
        let mut best: Option<(&'b str, usize)> = None;
        for (&key, &val) in map {
            if rest.starts_with(key) && is_boundary(&rest[key.len()..]) {
                match best {
                    None => best = Some((val, key.len())),
                    Some((_, b)) if key.len() > b => best = Some((val, key.len())),
                    _ => {}
                }
            }
        }
        best
    }

    fn match_quantity(&self, rest: &str) -> Option<(f64, Unit, usize)> {
        let m = QTY_RE.find(rest)?;
        let value: f64 = m.as_str().replace(',', "").parse().ok()?;
        if value == 0.0 { return None; }
        let mut consumed = m.end();
        let after_num = &rest[consumed..];
        let spaces: usize = after_num.chars()
            .take_while(|c| c.is_whitespace())
            .map(|c| c.len_utf8()).sum();
        let after_ws = &after_num[spaces..];
        let mut unit = Unit::None;
        let mut unit_bytes = 0usize;
        for (key, u) in &self.ontology.units {
            if after_ws.starts_with(key) && is_boundary(&after_ws[key.len()..]) {
                if key.len() > unit_bytes {
                    unit = u.clone();
                    unit_bytes = key.len();
                }
            }
        }
        if unit != Unit::None {
            consumed += spaces + unit_bytes;
        } else if !is_boundary(&rest[consumed..]) {
            // Number immediately followed by a non-boundary char (e.g. 9%, 180x200cm,
            // 316L, 12V) — emit as Sytg to keep the token intact, not as a bare Num.
            return None;
        }
        Some((value, unit, consumed))
    }

    fn match_standard(&self, rest: &str) -> Option<(String, usize)> {
        for &prefix in &self.ontology.standards {
            if rest.starts_with(prefix) {
                let after = &rest[prefix.len()..];
                // Skip optional whitespace between prefix and code number
                // so "iso 9001" is consumed as one Standard("ISO 9001") token
                // and the number is not left over to become a spurious Num.
                let spaces: usize = after.chars()
                    .take_while(|c| c.is_whitespace())
                    .map(|c| c.len_utf8()).sum();
                let after_ws = &after[spaces..];
                let code_bytes: usize = after_ws.chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '-')
                    .map(|c| c.len_utf8()).sum();
                let total = prefix.len() + spaces + code_bytes;
                return Some((rest[..total].trim().to_uppercase(), total));
            }
        }
        None
    }
}

fn merge_adjacent_sytg(tags: Vec<Tag>) -> Vec<Tag> {
    let mut result: Vec<Tag> = Vec::new();
    for tag in tags {
        match tag {
            Tag::Sytg(text) => match result.last_mut() {
                Some(Tag::Sytg(prev)) => { prev.push(' '); prev.push_str(&text); }
                _ => result.push(Tag::Sytg(text)),
            },
            other => result.push(other),
        }
    }
    result
}

#[inline]
fn is_boundary(after: &str) -> bool {
    after.chars().next().map_or(true, |c| {
        c.is_whitespace() || matches!(c, ',' | '-' | '.' | '!' | '?' | ')' | '/')
    })
}
