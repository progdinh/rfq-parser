// parser.rs — Grammar rule engine (v3)
//
// Rules applied:
//   R1: [Sytg] → [Conn] → [Num]          = group split
//   R2: [Sytg] → [Conn] → [Sytg]         = same group
//   R3: [Num]  → ...                      = group start
//   R4: [Kwd]  → [Attr]                   = attribute of current group
//   R5: Adjacent Sytg                     = merged in lexer
//   R6: [Sytg/Attr] → [Num] (no Conn)    = spec/grade, NOT a new group
//   R7: [Num] → [Conn] → [Num] → [Sytg]  = quantity range
//   R8: NeedKwd                            = consumed in lexer

use crate::token::{ConnOp, Tag, Unit};

#[derive(Debug, Clone, Default)]
pub struct Chunk {
    pub text: String,
    pub conn: Option<ConnOp>, // connector FOLLOWING this chunk
}

#[derive(Debug, Clone, Default)]
pub struct Group {
    pub qty:       Option<f64>,
    pub qty_max:   Option<f64>,
    pub uom:       Option<String>,
    pub chunks:    Vec<Chunk>,
    pub colors:    Vec<String>,
    pub sizes:     Vec<String>,
    pub materials: Vec<String>,
    pub genders:   Vec<String>,
    pub specs:     Vec<String>,
    pub standards: Vec<String>,
    pub ambiguous: Vec<String>, // semantic unknowns → SLM resolves (ORIGIN/BRAND/GRADE/ATTR)
}

pub fn parse(tags: &[Tag]) -> Vec<Group> {
    if tags.is_empty() { return vec![]; }
    let boundaries = find_boundaries(tags);
    let mut groups = Vec::new();
    for i in 0..boundaries.len() {
        let start = boundaries[i];
        let end   = boundaries.get(i + 1).copied().unwrap_or(tags.len());
        let g = build_group(&tags[start..end]);
        if !g.chunks.is_empty() || g.qty.is_some() {
            groups.push(g);
        }
    }
    groups
}

// ── Group boundary detection ──────────────────────────────────────────────────
// R9: [Attr] → [Conn] → [Attr/Kwd] = connector between attr values is consumed silently

fn find_boundaries(tags: &[Tag]) -> Vec<usize> {
    let mut b = vec![0usize];

    for i in 1..tags.len() {
        let tag = &tags[i];
        if !matches!(tag, Tag::Num { .. }) { continue; }

        // R7: is this Num the qty_max of a range?
        // Pattern: [Num] [Conn] [Num←here] and next is not Num
        let is_range_max = i >= 2
            && matches!(&tags[i - 2], Tag::Num { .. })
            && matches!(&tags[i - 1], Tag::Conn(_))
            && !tags.get(i + 1).map_or(false, |t| matches!(t, Tag::Num { .. }));

        if is_range_max { continue; }

        // R6: is this Num a spec/grade following a Sytg/Attr (no Conn between)?
        // If the immediately preceding token is Sytg, Color, Size, or Material
        // → it's a spec inside the current group, NOT a new group boundary
        let is_spec = matches!(&tags[i - 1],
            Tag::Sytg(_) | Tag::Color(_) | Tag::Size(_) | Tag::Material(_));

        // R-trailing: Num with a unit, after a Sytg-opened group with no prior Num.
        // Pattern: [Sytg+] [Standard|attrs]* [Num{unit}←here]
        // English-style descriptions put quantity at the end ("soybean meal 2500 kg").
        // Keep in the same group rather than splitting into a qty-only orphan group.
        let last_boundary = b.last().copied().unwrap_or(0);
        let slice = &tags[last_boundary..i];
        let is_trailing_qty =
            slice.iter().any(|t| matches!(t, Tag::Sytg(_) | Tag::Provenance(_)))
            && !slice.iter().any(|t| matches!(t, Tag::Num { .. }))
            && matches!(tag, Tag::Num { unit, .. } if !matches!(unit, Unit::None));

        if !is_spec && !is_trailing_qty {
            b.push(i);
        }
    }

    b.sort_unstable();
    b.dedup();
    b
}

// ── Group builder ─────────────────────────────────────────────────────────────

fn build_group(tags: &[Tag]) -> Group {
    let mut g = Group::default();
    if tags.is_empty() { return g; }

    let mut i = 0;

    // Leading Num → qty (+ R7 range)
    if let Tag::Num { value, unit } = &tags[0] {
        if tags.len() >= 3
            && matches!(&tags[1], Tag::Conn(_))
            && matches!(&tags[2], Tag::Num { .. })
        {
            // R7: quantity range
            g.qty = Some(*value);
            g.uom = unit.to_str().map(|s| s.to_string());
            if let Tag::Num { value: v2, unit: u2 } = &tags[2] {
                g.qty_max = Some(*v2);
                if !matches!(u2, Unit::None) {
                    g.uom = u2.to_str().map(|s| s.to_string());
                }
            }
            i = 3;
        } else {
            g.qty = Some(*value);
            g.uom = unit.to_str().map(|s| s.to_string());
            i = 1;
        }
    }

    let mut text_buf: Vec<String> = Vec::new();

    while i < tags.len() {
        match &tags[i] {
            Tag::ColorKwd | Tag::SizeKwd | Tag::MaterialKwd => {
                flush(&mut g, &mut text_buf, None);
            }
            Tag::Gender(gen) => {
                add_unique(&mut g.genders, gen.clone());
            }
            Tag::Standard(s) => {
                add_unique(&mut g.standards, s.clone());
            }
            Tag::Provenance(p) => {
                add_unique(&mut g.ambiguous, p.clone());
            }
            Tag::Color(c) => {
                add_unique(&mut g.colors, c.clone());
            }
            Tag::Size(s) => {
                add_unique(&mut g.sizes, s.clone());
            }
            Tag::Material(m) => {
                add_unique(&mut g.materials, m.clone());
            }
            Tag::Sytg(text) => {
                text_buf.push(text.clone());
            }
            Tag::Conn(op) => {
                // Size range expansion: [Size] [RangeTo] [Size] → full slice
                if matches!(op, ConnOp::RangeTo)
                    && matches!(tags.get(i + 1), Some(Tag::Size(_)))
                    && i > 0 && matches!(tags.get(i - 1), Some(Tag::Size(_)))
                {
                    let from = match &tags[i - 1] { Tag::Size(s) => s.clone(), _ => unreachable!() };
                    let to   = match &tags[i + 1] { Tag::Size(s) => s.clone(), _ => unreachable!() };
                    g.sizes.retain(|s| *s != from);
                    for s in expand_size_range(&from, &to) { add_unique(&mut g.sizes, s); }
                    i += 1; // skip Size(to); loop adds one more
                } else {
                    // R9: connector before another attr value is consumed silently
                    let next_is_attr = matches!(
                        tags.get(i + 1),
                        Some(Tag::Color(_))
                            | Some(Tag::Size(_))
                            | Some(Tag::Material(_))
                            | Some(Tag::ColorKwd)
                            | Some(Tag::SizeKwd)
                            | Some(Tag::MaterialKwd)
                    );
                    if !next_is_attr {
                        flush(&mut g, &mut text_buf, Some(op.clone()));
                    }
                }
            }
            // R6: Num inside a group = spec — unless it's a trailing qty.
            // A trailing qty has a non-None unit and no prior qty was set
            // (the group was opened by a Sytg, not a leading Num).
            Tag::Num { value, unit } => {
                if g.qty.is_none() && !matches!(unit, Unit::None) {
                    g.qty = Some(*value);
                    g.uom = unit.to_str().map(String::from);
                } else {
                    let spec = if value.fract() == 0.0 {
                        format!("{}", *value as u64)
                    } else {
                        format!("{:.1}", value)
                    };
                    add_unique(&mut g.specs, spec);
                }
            }
        }
        i += 1;
    }

    flush(&mut g, &mut text_buf, None);
    g
}

// ── Size range expansion ──────────────────────────────────────────────────────

fn expand_size_range(from: &str, to: &str) -> Vec<String> {
    const FAMILIES: &[&[&str]] = &[
        &["XXS","XS","S","M","L","XL","XXL","XXXL","2XL","3XL","4XL"],
        &["small","medium","large"],
        &["27","28","29","30","31","32","33","34","35","36","37","38","39","40","41","42","43","44","45","46"],
        &["DN15","DN20","DN25","DN32","DN40","DN50","DN65","DN80","DN100","DN150","DN200","DN300"],
        &["SCH10","SCH40","SCH80"],
    ];
    for family in FAMILIES {
        let a = family.iter().position(|&s| s == from);
        let b = family.iter().position(|&s| s == to);
        if let (Some(i), Some(j)) = (a, b) {
            let (lo, hi) = if i <= j { (i, j) } else { (j, i) };
            return family[lo..=hi].iter().map(|s| s.to_string()).collect();
        }
    }
    vec![from.to_string(), to.to_string()]
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Finalize text_buf as a Chunk.
/// The `conn` parameter is the connector that FOLLOWS the current chunk.
fn flush(g: &mut Group, text_buf: &mut Vec<String>, conn: Option<ConnOp>) {
    if text_buf.is_empty() {
        // No chunk to create, but update last chunk's conn if provided
        if let (Some(c), Some(last)) = (conn, g.chunks.last_mut()) {
            if last.conn.is_none() { last.conn = Some(c); }
        }
        return;
    }
    let text = text_buf.join(" ");
    text_buf.clear();
    g.chunks.push(Chunk { text, conn });
}

fn add_unique(v: &mut Vec<String>, item: String) {
    if !v.contains(&item) { v.push(item); }
}
