mod item_normalizer;
mod segmenter;
mod trade_terms;

pub use trade_terms::TradeTerms;

use qlp::{parse_query, Group};
use item_normalizer::{normalize, NormalizedItem};
use segmenter::{is_rfq, segment};

// ── Public output types ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RfqItem {
    pub index:       usize,
    pub qty:         Option<f64>,
    pub uom:         Option<String>,
    pub price:       Option<f64>,
    pub currency:    Option<String>,
    pub origin:      Option<String>,
    pub dims:        Vec<String>,   // extracted inline measurement specs
    pub groups:      Vec<Group>,    // parsed groups from qlp (C1 + S1 for pipeline)
    pub slm_dsl:     String,
    pub search_strs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedRfq {
    pub input:       String,
    pub items:       Vec<RfqItem>,
    pub trade_terms: TradeTerms,
    pub slm_dsl:     String,
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn parse_rfq(input: &str) -> ParsedRfq {
    let input = &input.replace("\\n", "\n").replace("\\t", "\t");
    let segs = segment(input);

    let trade_terms = TradeTerms::extract(input);

    let mut items: Vec<RfqItem> = Vec::new();
    let mut dsl_parts: Vec<String> = Vec::new();

    for (idx, raw) in segs.items.iter().enumerate() {
        let norm: NormalizedItem = normalize(raw);
        let qlp = parse_query(&norm.description);
        let groups = qlp.groups.clone();

        let mut search_strs = qlp.to_search_strings();
        if !norm.specs.is_empty() {
            let spec_suffix = norm.specs.join(" ");
            for s in &mut search_strs {
                s.push(' ');
                s.push_str(&spec_suffix);
            }
        }

        // Resolve currency: item-level first, then RFQ-level trade terms
        let resolved_currency = norm.currency.clone()
            .or_else(|| trade_terms.currency.clone());

        // Strip GROUP_1 wrapper when there is only one group — it adds noise
        // in the RFQ context where each item is already pre-segmented.
        let dsl_body = if qlp.groups.len() == 1 {
            qlp.slm_dsl
                .strip_prefix("GROUP_1:\n")
                .unwrap_or(&qlp.slm_dsl)
                .to_string()
        } else {
            qlp.slm_dsl.clone()
        };

        let item = RfqItem {
            index:       idx + 1,
            qty:         norm.qty,
            uom:         norm.uom.clone(),
            price:       norm.price,
            currency:    resolved_currency.clone(),
            origin:      norm.origin.clone(),
            dims:        norm.specs.clone(),
            groups,
            slm_dsl:     dsl_body.clone(),
            search_strs,
        };

        let mut block = format!("ITEM_{}:\n", idx + 1);
        block.push_str(&dsl_body);
        // Guarantee each appended field starts on its own line
        if !block.ends_with('\n') { block.push('\n'); }
        if !norm.specs.is_empty() {
            block.push_str(&format!("  DIMS: [{}]\n", norm.specs.join(", ")));
        }
        if let Some(p) = norm.price {
            let cur = resolved_currency.as_deref().unwrap_or("?");
            block.push_str(&format!("  PRICE: {} {}\n", p, cur));
        }
        if let Some(o) = &norm.origin {
            block.push_str(&format!("  ORIGIN: {}\n", o));
        }
        dsl_parts.push(block);
        items.push(item);
    }

    if !trade_terms.is_empty() {
        dsl_parts.push(trade_terms.to_dsl());
    }

    ParsedRfq {
        input: input.to_string(),
        items,
        trade_terms,
        slm_dsl: dsl_parts.join("\n"),
    }
}

// ── Unified entry point ───────────────────────────────────────────────────────

pub enum ParseResult {
    SimpleQuery(qlp::ParsedQuery),
    Rfq(ParsedRfq),
}

pub fn parse(input: &str) -> ParseResult {
    let input = &input.replace("\\n", "\n").replace("\\t", "\t");
    if is_rfq(input) {
        ParseResult::Rfq(parse_rfq(input))
    } else {
        ParseResult::SimpleQuery(parse_query(input))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_RFQ: &str = "hi, pls send over a quote for these items in GBP:\n\n1. 120 tons silver zinc ingots @ 2.5 GBP\n2. 12 pcs white led panel lights, surface mount @ 150.0 GBP\n3. 24 kg 0.5mm 201 ss embossed coil, south korea @ 0.05 GBP\n4. 48 tons technical methanol, malaysia @ 300.0 GBP\n5. 500 kg soybean meal, bulk container, brazil @ 0.35 GBP\n\nlead time 6 weeks after artwork approval. payment terms: T/T 30% deposit, bal against BL copy.";

    #[test]
    fn test_is_rfq() {
        assert!(is_rfq(SIMPLE_RFQ));
        assert!(!is_rfq("300 áo thun màu đỏ size S M L"));
    }

    #[test]
    fn test_segment_count() {
        let segs = segmenter::segment(SIMPLE_RFQ);
        assert_eq!(segs.items.len(), 5);
    }

    #[test]
    fn test_trade_terms() {
        let rfq = parse_rfq(SIMPLE_RFQ);
        assert_eq!(rfq.trade_terms.currency.as_deref(), Some("GBP"));
        assert!(rfq.trade_terms.payment.is_some());
    }

    #[test]
    fn test_item_count() {
        let rfq = parse_rfq(SIMPLE_RFQ);
        assert_eq!(rfq.items.len(), 5);
    }

    #[test]
    fn test_item_price() {
        let rfq = parse_rfq(SIMPLE_RFQ);
        assert_eq!(rfq.items[0].price, Some(2.5));
    }

    #[test]
    fn test_item_qty() {
        let rfq = parse_rfq(SIMPLE_RFQ);
        assert_eq!(rfq.items[0].qty, Some(120.0));
    }

    #[test]
    fn test_unified_routing() {
        assert!(matches!(parse("300 áo thun màu đỏ"), ParseResult::SimpleQuery(_)));
        assert!(matches!(parse(SIMPLE_RFQ), ParseResult::Rfq(_)));
    }

    #[test]
    fn test_bullet_segmentation() {
        let rfq = "Please quote:\n- 500 pcs blue LED panels 12V\n- 100 kg stainless wire 1.0mm\n- 50 sets motor drivers\nlead time: 4 weeks";
        assert!(is_rfq(rfq), "bullet RFQ not detected");
        let segs = segmenter::segment(rfq);
        assert_eq!(segs.items.len(), 3, "expected 3 items, got {:?}", segs.items);
        assert!(segs.items[0].contains("LED"), "item 0: {:?}", segs.items[0]);
    }

    // Real-world: formal email with *   **Bold Product:** bullet format
    #[test]
    fn test_email_rfq_markdown_bullets() {
        let rfq = "Subject: RFQ\n\nDear Sales Team,\n\nPlease quote under DAP Jakarta, lead time 14 days:\n\n*   **Kraft Paper Bag:** 1,500 Pieces, Natural Brown, 25x10x30cm (Target: 0.85).\n*   **Corrugated Sheet:** 24 Pallets, 1200x800mm, Thailand (Target: 12.0).\n*   **Nitrile Glove:** 2 Box, Size S (Target: 0.35).";
        assert!(is_rfq(rfq), "email RFQ not detected");
        let rfq_parsed = parse_rfq(rfq);
        assert_eq!(rfq_parsed.items.len(), 3, "items: {:?}", rfq_parsed.items.iter().map(|i| &i.slm_dsl).collect::<Vec<_>>());
        // markdown stripped from description
        assert!(!rfq_parsed.items[0].slm_dsl.contains("**"), "** not stripped: {}", rfq_parsed.items[0].slm_dsl);
        // bare target price captured
        assert_eq!(rfq_parsed.items[0].price, Some(0.85), "price: {:?}", rfq_parsed.items[0].price);
        // dims extracted
        assert!(!rfq_parsed.items[0].dims.is_empty(), "dims: {:?}", rfq_parsed.items[0].dims);
        eprintln!("\n{}", rfq_parsed.slm_dsl);
    }

    #[test]
    fn test_bare_line_segmentation() {
        let rfq = "Dear supplier, pls quote:\n500 pcs blue LED 12V\n100 kg stainless wire\n50 sets motor drivers\nlead time: 4 weeks\npayment terms: T/T 30%";
        assert!(is_rfq(rfq), "bare-line RFQ not detected");
        let segs = segmenter::segment(rfq);
        assert_eq!(segs.items.len(), 3, "expected 3 items, got {:?}", segs.items);
        assert!(segs.items[0].contains("LED"), "item 0: {:?}", segs.items[0]);
    }

    #[test]
    fn test_dims_in_dsl() {
        let rfq = parse_rfq("1. PE stretch film, natural color, 180x200cm, 500 rolls\n2. Stainless steel wire 316L, 1.0mm, 100 kg, south korea\n3. LED driver module, 12V 200Ah, 50 pcs @ 8.5 USD");
        // item 1 — 180x200cm
        assert!(rfq.items[0].dims.iter().any(|d| d.contains("180") && d.contains("200")),
            "item 1 dims: {:?}", rfq.items[0].dims);
        assert!(rfq.slm_dsl.contains("DIMS:"), "DIMS not in DSL:\n{}", rfq.slm_dsl);
        // item 2 — 1.0mm
        assert!(rfq.items[1].dims.iter().any(|d| d.contains("mm")),
            "item 2 dims: {:?}", rfq.items[1].dims);
        // search strings contain the spec suffix
        assert!(rfq.items[0].search_strs[0].contains("180"),
            "search_strs: {:?}", rfq.items[0].search_strs);
        eprintln!("\n--- DSL ---\n{}", rfq.slm_dsl);
        for item in &rfq.items {
            eprintln!("item {} dims: {:?}  search[0]: {:?}", item.index, item.dims, item.search_strs.first());
        }
    }

    #[test]
    fn test_vietnamese_rfq() {
        use item_normalizer::normalize;

        // giá mục tiêu → price; 10L → DIMS; xuất xứ Đức → Germany; 250 FCL → qty
        let n = normalize("Thùng tưới cây kim loại (Metal Watering Can), sắt, 10L, xuất xứ Đức, chất lượng xuất khẩu: 250 FCL (giá mục tiêu 0.12).");
        assert_eq!(n.price, Some(0.12), "price: {:?}", n.price);
        assert_eq!(n.qty,   Some(250.0), "qty: {:?}", n.qty);
        assert!(n.specs.iter().any(|s| s.contains("10")), "dims: {:?}", n.specs);
        assert_eq!(n.origin.as_deref(), Some("Germany"), "origin: {:?}", n.origin);

        // 1200 cái → pcs; 100kW → DIMS; xuất xứ Trung Quốc → China
        let n = normalize("Biến tần năng lượng mặt trời (Solar Inverter), Hybrid, 100kW, PCB, xuất xứ Trung Quốc: 1200 cái.");
        assert_eq!(n.qty, Some(1200.0), "cái qty: {:?}", n.qty);
        assert_eq!(n.uom.as_deref(), Some("pcs"), "cái uom: {:?}", n.uom);
        assert!(n.specs.iter().any(|s| s.contains("kw") || s.contains("100")), "kW dims: {:?}", n.specs);
        assert_eq!(n.origin.as_deref(), Some("China"), "origin: {:?}", n.origin);

        // 300 bộ → sets; 51.2V 200Ah → DIMS; xuất xứ Hàn Quốc → South Korea; giá mục tiêu → price
        let n = normalize("Bộ pin Lithium-Ion, 51.2V 200Ah, xuất xứ Hàn Quốc, ứng dụng nặng: 300 bộ (giá mục tiêu 0.03).");
        assert_eq!(n.qty,   Some(300.0), "bộ qty: {:?}", n.qty);
        assert_eq!(n.uom.as_deref(), Some("sets"), "bộ uom: {:?}", n.uom);
        assert_eq!(n.price, Some(0.03), "price: {:?}", n.price);
        assert_eq!(n.origin.as_deref(), Some("South Korea"), "origin: {:?}", n.origin);
    }

    #[test]
    fn test_en_structured_product_spec_markers() {
        let rfq = parse_rfq(
            "Please quote the following. Lead time 4 weeks, payment via L/C at sight.\n\
             * Product: Car Battery (Heavy-duty), 3000 Cartons, Spec: Black, 70Ah, Start-stop, Japan.\n\
             * Product: Silk Fabric (Luxury grade), 500 Yards, Spec: Red, 150cm width, Silk-wool blend, China, Target: 0.03.\n\
             * Product: Sodium Hypochlorite (Industrial grade), Qty: N/A Liters, Spec: 1000L IBC, Water treatment grade, China, Target: 0.35."
        );
        eprintln!("\n--- EN structured RFQ ---\n{}", rfq.slm_dsl);

        // Product: markers must NOT appear in chunks
        for item in &rfq.items {
            assert!(!item.slm_dsl.contains("product:"), "product: leaked: {}", item.slm_dsl);
            assert!(!item.slm_dsl.contains("spec:"),    "spec: leaked: {}", item.slm_dsl);
        }
        // Car Battery: 3000 cartons, Japan origin
        assert_eq!(rfq.items[0].qty, Some(3000.0));
        assert_eq!(rfq.items[0].origin.as_deref(), Some("Japan"));
        // Silk Fabric: 500 yards, China, price 0.03
        assert_eq!(rfq.items[1].qty, Some(500.0));
        assert_eq!(rfq.items[1].uom.as_deref(), Some("yards"));
        assert_eq!(rfq.items[1].origin.as_deref(), Some("China"));
        assert_eq!(rfq.items[1].price, Some(0.03));
        // Sodium Hypochlorite: N/A qty → null, China, price 0.35
        assert_eq!(rfq.items[2].qty, None);
        assert_eq!(rfq.items[2].origin.as_deref(), Some("China"));
        assert_eq!(rfq.items[2].price, Some(0.35));
    }

    #[test]
    fn test_vi_paren_origin_rfq() {
        let rfq = parse_rfq(
            "FCA Sydney, thanh toán T/T trả trước\n\
             1. Bàn họp (Malaysia, mặt kính đen, hình bầu dục 1.8m): 300 cái, target 2.5 SGD/cái\n\
             2. Kim tiêm 3ml (TQ, an toàn): 1200 thùng, target 0.03 SGD/thùng\n\
             3. Trà xanh Matcha (VN): 2 thùng, target 25 SGD/thùng"
        );
        eprintln!("\n--- VI paren origin RFQ ---\n{}", rfq.slm_dsl);

        assert_eq!(rfq.items[0].origin.as_deref(), Some("Malaysia"));
        assert_eq!(rfq.items[0].qty,   Some(300.0));
        assert_eq!(rfq.items[0].price, Some(2.5));
        assert_eq!(rfq.items[1].origin.as_deref(), Some("China"));
        assert_eq!(rfq.items[1].qty,   Some(1200.0));
        assert_eq!(rfq.items[2].origin.as_deref(), Some("Vietnam"));
        // Trade terms
        assert_eq!(rfq.trade_terms.incoterm.as_deref(),     Some("FCA"));
        assert_eq!(rfq.trade_terms.destination.as_deref(),  Some("Sydney"));
        assert!(rfq.trade_terms.payment.is_some(), "payment should be extracted");
    }

    #[test]
    fn test_inline_spec_extraction() {
        use item_normalizer::normalize;

        // dimension
        let n = normalize("PE stretch film, 180x200cm, 500 rolls");
        assert!(n.specs.iter().any(|s| s.contains("180x200cm") || s.contains("180") && s.contains("200")),
            "expected 180x200cm in specs, got {:?}", n.specs);
        assert!(!n.description.contains("180x200cm"), "spec should be stripped from description");

        // thickness
        let n = normalize("stainless steel wire, 1.0mm, 100 kg");
        assert!(n.specs.iter().any(|s| s.contains("1") && s.contains("mm")),
            "expected mm spec, got {:?}", n.specs);

        // voltage
        let n = normalize("LED driver, 12V, 200Ah, 50 pcs");
        assert!(n.specs.iter().any(|s| s.to_lowercase().contains("v")),
            "expected voltage spec, got {:?}", n.specs);
    }
}
