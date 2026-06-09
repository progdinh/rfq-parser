// lib.rs — Arobid query tagger public API (v3)
//
// Pipeline:
//   parse_query(input) → ParsedQuery
//     .groups   → Vec<Group>  (structured by grammar rules)
//     .slm_dsl  → String      (compact DSL for the SLM — always generated)
//
// ALL queries go through the SLM. The Rust tagger reduces prompt size,
// not SLM calls. The SLM receives ~100-200 tokens instead of ~2000.

pub mod dsl;
pub mod lexer;
pub mod ontology;
pub mod parser;
pub mod token;

pub use parser::{Chunk, Group};
pub use token::{ConnOp, Tag, Unit};

use dsl::to_dsl;
use lexer::Lexer;
use parser::parse;

// ── Public API ────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct ParsedQuery {
    /// Structured groups from the grammar rules
    pub groups:  Vec<Group>,
    /// Compact DSL string ready to inject into the SLM prompt
    pub slm_dsl: String,
    /// Original input preserved for debugging
    pub input:   String,
}

impl ParsedQuery {
    fn fmt_qty(v: f64) -> String {
        if v.fract() == 0.0 { format!("{}", v as u64) } else { format!("{:.2}", v) }
    }

    /// Generate one search string per product chunk, for direct embedding use
    /// (when SLM output is not yet available).
    pub fn to_search_strings(&self) -> Vec<String> {
        let mut strings = Vec::new();
        for g in &self.groups {
            let qty_prefix = g.qty.map(|v| {
                let s = match g.qty_max {
                    Some(max) => format!("{}-{}", Self::fmt_qty(v), Self::fmt_qty(max)),
                    None      => Self::fmt_qty(v),
                };
                match &g.uom {
                    Some(u) => format!("{} {}", s, u),
                    None    => s,
                }
            });

            for chunk in &g.chunks {
                let mut parts = Vec::new();
                if let Some(ref p) = qty_prefix { parts.push(p.clone()); }
                parts.push(chunk.text.clone());
                // Append enriched attributes
                if !g.colors.is_empty()    { parts.push(g.colors.join(" ")); }
                if !g.materials.is_empty() { parts.push(g.materials.join(" ")); }
                if !g.sizes.is_empty()     { parts.push(g.sizes.join(" ")); }
                if !g.genders.is_empty()   { parts.push(g.genders.join(" ")); }
                if !g.specs.is_empty()     { parts.push(g.specs.join(" ")); }
                if !g.ambiguous.is_empty() { parts.push(g.ambiguous.join(" ")); }
                strings.push(parts.join(" ").trim().to_string());
            }

            // If no chunks but we have a qty + attrs: one string with attrs only
            if g.chunks.is_empty() && g.qty.is_some() {
                let mut parts = Vec::new();
                if let Some(ref p) = qty_prefix { parts.push(p.clone()); }
                if !g.colors.is_empty()    { parts.push(g.colors.join(" ")); }
                if !g.materials.is_empty() { parts.push(g.materials.join(" ")); }
                if !g.sizes.is_empty()     { parts.push(g.sizes.join(" ")); }
                if !parts.is_empty() { strings.push(parts.join(" ")); }
            }
        }
        strings
    }
}

/// Main entry point — tokenize, parse, and generate SLM DSL.
pub fn parse_query(input: &str) -> ParsedQuery {
    let lexer  = Lexer::new(&ontology::ONTOLOGY);
    let tags   = lexer.tokenize(input);
    let groups   = parse(&tags);
    let slm_dsl  = to_dsl(&groups);

    ParsedQuery {
        groups,
        slm_dsl,
        input: input.to_string(),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn dsl(q: &str) -> String { parse_query(q).slm_dsl }
    fn groups(q: &str) -> Vec<Group> { parse_query(q).groups }

    // ── Single group, simple queries ─────────────────────────────────────────

    #[test]
    fn test_qty_only() {
        let g = groups("500 kg");
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].qty, Some(500.0));
        assert_eq!(g[0].uom.as_deref(), Some("kg"));
    }

    #[test]
    fn test_single_product_vi() {
        let g = groups("300 cái áo thun");
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].qty, Some(300.0));
        assert_eq!(g[0].uom.as_deref(), Some("cái"));
        assert!(g[0].chunks.iter().any(|c| c.text.contains("áo thun")));
    }

    #[test]
    fn test_single_product_en() {
        let g = groups("500 polo shirts");
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].qty, Some(500.0));
        assert!(g[0].chunks.iter().any(|c| c.text.contains("polo shirt")));
    }

    #[test]
    fn test_attrs_via_kwd_vi() {
        let g = groups("300 áo thun màu đỏ xanh kích thước S M L");
        assert_eq!(g.len(), 1);
        assert!(g[0].colors.contains(&"red".to_string()));
        assert!(g[0].colors.contains(&"blue".to_string()));
        assert!(g[0].sizes.contains(&"S".to_string()));
        assert!(g[0].sizes.contains(&"L".to_string()));
    }

    #[test]
    fn test_gender_always_typed() {
        let g = groups("200 áo polo nam");
        assert_eq!(g.len(), 1);
        assert!(g[0].genders.contains(&"male".to_string()));
    }

    // ── Multi-group (R1) ─────────────────────────────────────────────────────

    #[test]
    fn test_two_groups() {
        let g = groups("50 áo và 300 quần");
        assert_eq!(g.len(), 2, "expected 2 groups, got {}: {:?}",
            g.len(), g.iter().map(|g| &g.chunks).collect::<Vec<_>>());
        assert_eq!(g[0].qty, Some(50.0));
        assert_eq!(g[1].qty, Some(300.0));
    }

    #[test]
    fn test_multi_group_complex() {
        let g = groups("50 áo và quần và slip hoặc 30 đôi vớ");
        assert_eq!(g.len(), 2);
        assert_eq!(g[0].qty, Some(50.0));
        // First group should have 3 chunks: áo, quần, slip
        assert_eq!(g[0].chunks.len(), 3, "chunks: {:?}", g[0].chunks);
        assert_eq!(g[1].qty, Some(30.0));
    }

    // ── Quantity range (R7) ───────────────────────────────────────────────────

    #[test]
    fn test_qty_range() {
        let g = groups("100 hoặc 200 áo thun");
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].qty,     Some(100.0));
        assert_eq!(g[0].qty_max, Some(200.0));
    }

    // ── Need keyword (R8) ────────────────────────────────────────────────────

    #[test]
    fn test_need_keyword_stripped() {
        let g = groups("tôi cần 500 áo polo hoặc áo thun size XL");
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].qty, Some(500.0));
        assert!(g[0].sizes.contains(&"XL".to_string()));
        // Both products should be in chunks
        assert_eq!(g[0].chunks.len(), 2);
    }

    // ── Grade (R6) ────────────────────────────────────────────────────────────

    #[test]
    fn test_industrial_grade() {
        let g = groups("500 kg ống thép 304");
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].qty, Some(500.0));
        assert!(g[0].specs.contains(&"304".to_string()), "specs: {:?}", g[0].specs);
    }

    // ── Benchmark queries ─────────────────────────────────────────────────────

    #[test]
    fn test_benchmark_vi() {
        let q = "Cần 300 áo thun và quần jean nam nữ màu đỏ xanh đen kích thước S M L XL";
        let r = parse_query(q);
        println!("VI benchmark DSL:\n{}", r.slm_dsl);
        assert_eq!(r.groups.len(), 1);
        let g = &r.groups[0];
        assert_eq!(g.qty, Some(300.0));
        assert_eq!(g.chunks.len(), 2, "chunks: {:?}", g.chunks);
        assert!(g.colors.contains(&"red".to_string()));
        assert!(g.colors.contains(&"black".to_string()));
        assert!(g.sizes.contains(&"XL".to_string()));
        assert!(g.genders.contains(&"male".to_string()));
        assert!(g.genders.contains(&"female".to_string()));
    }

    #[test]
    fn test_benchmark_en() {
        let q = "100 small or large red cotton and blue polyester polo shirts and t-shirts";
        let r = parse_query(q);
        println!("EN benchmark DSL:\n{}", r.slm_dsl);
        assert_eq!(r.groups.len(), 1);
        let g = &r.groups[0];
        assert_eq!(g.qty, Some(100.0));
        assert!(g.colors.contains(&"red".to_string()));
        assert!(g.colors.contains(&"blue".to_string()));
        assert!(g.materials.contains(&"cotton".to_string()));
        assert!(g.materials.contains(&"polyester".to_string()));
    }

    #[test]
    fn test_benchmark_multi_group() {
        let q = "50 áo và quần và slip hoặc 30 đôi vớ";
        let r = parse_query(q);
        println!("Multi-group DSL:\n{}", r.slm_dsl);
        assert_eq!(r.groups.len(), 2);
        assert_eq!(r.groups[0].chunks.len(), 3);
    }

    // ── Inline color/size — no chunk leak ────────────────────────────────────

    #[test]
    fn test_inline_color_no_chunk_leak() {
        let g = groups("50 quần đỏ");
        assert_eq!(g.len(), 1);
        assert!(g[0].colors.contains(&"red".to_string()));
        assert!(g[0].chunks.iter().all(|c| !c.text.contains("red")),
            "color leaked into chunk: {:?}", g[0].chunks);
    }

    #[test]
    fn test_search_string_no_color_duplicate() {
        let r = parse_query("50 quần đỏ");
        let s = r.to_search_strings();
        assert_eq!(s.len(), 1);
        // "red" must appear exactly once
        assert_eq!(s[0].matches("red").count(), 1, "got: {:?}", s);
    }

    // ── R9: attr-list connector ───────────────────────────────────────────────

    #[test]
    fn test_color_list_conn_no_chunk_leak() {
        let g = groups("500 quần hoặc áo màu trắng hoặc đen");
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].chunks.len(), 2, "chunks: {:?}", g[0].chunks);
        assert!(g[0].colors.contains(&"white".to_string()));
        assert!(g[0].colors.contains(&"black".to_string()));
        assert!(g[0].chunks.iter().all(|c| !c.text.contains("black")),
            "black leaked into chunks: {:?}", g[0].chunks);
    }

    // ── Range with đến ────────────────────────────────────────────────────────

    #[test]
    fn test_range_den() {
        let g = groups("100 đến 200 áo thun");
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].qty,     Some(100.0));
        assert_eq!(g[0].qty_max, Some(200.0));
        assert!(g[0].chunks.iter().any(|c| c.text.contains("áo thun")));
    }

    #[test]
    fn test_range_tu_den() {
        let g = groups("từ 100 đến 200 áo thun");
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].qty,     Some(100.0));
        assert_eq!(g[0].qty_max, Some(200.0));
    }

    #[test]
    fn test_size_range_expansion() {
        let g = groups("tôi cần 150 áo thun hoặc áo polo size S đến XL");
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].qty, Some(150.0));
        assert_eq!(g[0].chunks.len(), 2, "chunks: {:?}", g[0].chunks);
        assert_eq!(g[0].sizes, vec!["S","M","L","XL"], "sizes: {:?}", g[0].sizes);
    }

    #[test]
    fn test_size_range_numeric() {
        let g = groups("500 quần jean size 30 đến 34");
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].sizes, vec!["30","31","32","33","34"], "sizes: {:?}", g[0].sizes);
    }

    // ── Extended need keywords ────────────────────────────────────────────────

    #[test]
    fn test_need_muon_mua() {
        let g = groups("tôi muốn mua 500 áo thun");
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].qty, Some(500.0));
        assert!(g[0].chunks.iter().any(|c| c.text.contains("áo thun")));
    }

    #[test]
    fn test_need_khoang() {
        let g = groups("khoảng 300 cái áo thun");
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].qty, Some(300.0));
        assert_eq!(g[0].uom.as_deref(), Some("cái"));
    }

    // ── "+" connector ─────────────────────────────────────────────────────────

    #[test]
    fn test_plus_connector() {
        let g = groups("50 áo thun + quần jean");
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].chunks.len(), 2, "chunks: {:?}", g[0].chunks);
        assert_eq!(g[0].qty, Some(50.0));
    }

    // ── DSL format ────────────────────────────────────────────────────────────

    #[test]
    fn test_dsl_contains_group_headers() {
        let d = dsl("50 áo và 300 quần");
        assert!(d.contains("GROUP_1:"), "missing GROUP_1 in: {}", d);
        assert!(d.contains("GROUP_2:"), "missing GROUP_2 in: {}", d);
    }

    #[test]
    fn test_dsl_contains_chunks() {
        let d = dsl("300 áo thun màu đỏ kích thước S M L");
        assert!(d.contains("CHUNKS:"), "dsl: {}", d);
        assert!(d.contains("COLOR:"),  "dsl: {}", d);
        assert!(d.contains("SIZE:"),   "dsl: {}", d);
    }

    #[test]
    fn test_dsl_range() {
        let d = dsl("100 hoặc 200 áo thun");
        assert!(d.contains("QTY_MAX: 200"), "dsl: {}", d);
    }

    // ── Search strings ────────────────────────────────────────────────────────

    #[test]
    fn test_search_strings_single() {
        let r = parse_query("300 cái áo thun màu đỏ kích thước S M L");
        let s = r.to_search_strings();
        assert!(!s.is_empty());
        println!("search strings: {:?}", s);
    }

    #[test]
    fn test_search_strings_multi_group() {
        let r = parse_query("50 áo và 300 quần");
        let s = r.to_search_strings();
        assert_eq!(s.len(), 2, "strings: {:?}", s);
    }
}
