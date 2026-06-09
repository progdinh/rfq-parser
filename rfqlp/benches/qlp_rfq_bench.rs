use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rfqlp::parse_rfq;

// ── RFQ corpus by category ────────────────────────────────────────────────────
// All entries sourced from rfq_benchmark_600.jsonl (English subset).
// Categories reflect real-world buyer behaviour observed in the dataset.

// ── chat_small: informal chat-style, 3–5 items ────────────────────────────────
// Typical WhatsApp / email thread from a buyer. Short preamble, numbered items,
// @ prices, abbreviated field names.

const CHAT_SMALL: &[&str] = &[
    // GEN-0015 — 4 items, SGD, FOB Jakarta
    "hi there, looking for a quote for the following items:\n\n\
     1. 12 pcs medical grade black vinyl gloves (size S, Thailand) @ 1.2 SGD/pc\n\
     2. 1 kg non-clumping bentonite cat litter (20L, retail packaging) @ 4.5 SGD/kg\n\
     3. 1200 cartons black silicone polymer sealant (300ml cartridge, USA) @ 25.0 SGD/ctn\n\
     4. 3 cartons food-grade white kids thermos (1L, China) @ 150.0 SGD/ctn\n\n\
     terms: FOB to Jakarta, lead time 6 weeks after artwork approval, payment Net 30 days.",

    // GEN-0184 — 4 items, SGD, EXW UK
    "Hi, I am looking for a price quotation for the following items based on EXW incoterms \
     with payment via T/T in advance. We require weekly deliveries to the United Kingdom, quoted in SGD.\n\n\
     1. Hydrogen Peroxide (Food grade, H2O2): 3 Tons (Target price: 0.5 SGD)\n\
     2. Bubble Wrap Roll (Anti-static, Clear, LDPE, India): 500 Cartons (Target price: 45.0 SGD)\n\
     3. Mooring Buoy (Inflatable, Black, 2400mm, Polyethylene, Turkey): 10,000 Pieces (Target price: 0.35 SGD)\n\
     4. Dried Mango (Organic, Orange, 500g, India): 1 Ton (Target price: 1.2 SGD)",

    // GEN-2063 — 3 items, D/P at sight
    "Please provide a formal quote for the following items. Payment terms: D/P at sight.\n\n\
     1. Wooden Flooring (Export quality, Laminate, 12mm thick, Click-lock): Quantity TBD, Target Price: 4.50 per Box.\n\
     2. Safety Workwear Set (Waterproof, Orange, Anti-static, India): 12 Pieces.\n\
     3. USB-C Power Adapter (Black, 18W, Metal, Dual port): 50 Pieces, Target Price: 12.00.",
];

// ── numbered_medium: standard numbered list, 6–8 items ────────────────────────
// Most common buyer format. Single preamble, clean numbered items, trailing
// trade terms.

const NUMBERED_MEDIUM: &[&str] = &[
    // GEN-0002 — 7 items, GBP, @ prices
    "hi, pls send over a quote for these items in GBP:\n\n\
     1. 120 tons silver zinc ingots (china) @ 2.5\n\
     2. 12 pcs white led panel lights, surface mount, alum frame, oem req @ 150.0\n\
     3. 24 kg ind. grade 0.5mm 201 ss embossed coil (s. korea) @ 0.05\n\
     4. 48 tons ind. grade technical methanol (malaysia) @ 300.0\n\
     5. 500 kg food-grade soybean meal, bulk container (brazil) @ 0.35\n\
     6. 100 drums organic coconut oil, export quality, 1L (philippines)\n\
     7. 3 sets 18 inch yellow pp floor standing industrial fans (germany) @ 0.05\n\n\
     lead time 6 weeks after artwork approval. payment terms: T/T 30% deposit, bal against BL copy.",

    // GEN-0008 — 8 items, CIF Rotterdam
    "Hello,\n\nI need price for these products CIF Rotterdam.\n\n\
     1. 50 Pieces Cotton Polo T-Shirt, India origin, retail packaging.\n\
     2. 120 Pieces Plastic Jerry Can, red, 5L, LDPE, Vietnam, target price 1.2.\n\
     3. 2 Cartons Glass Jar, 500ml, borosilicate, wide mouth, India, target price 12.0.\n\
     4. 5 Pieces Silicone Sealant, black, 600ml sausage, marine grade, USA, target price 1.2.\n\
     5. 1000 Kilograms Dried Mango, golden, 250g, unsweetened, Vietnam.\n\
     6. 5 Cubic meter Plywood, 4x8ft 1220x2440mm, structural, Russia, target price 0.5.\n\
     7. 1000 Cartons LED Panel Light, round 120mm, surface mount, China, target price 0.12.\n\
     8. 500 Pieces Fire Extinguisher, ABC dry powder, 6kg, CE certified, China, target price 2.5.\n\n\
     Payment T/T 30% deposit, balance against BL copy.",

    // GEN-0018 — 8 items, FOB Sydney, USD
    "Hello, I want buy products for Sydney, FOB trade term. Please send price for list below. \
     Payment T/T 30% deposit, balance against BL copy. I need ready stock.\n\n\
     1. Latex Mattress, indoor, white, King 180x200cm, anti-dust mite, Malaysia, 50 Pieces, target 1.2 USD.\n\
     2. PVC Pipe, UV-resistant, white, electrical conduit, India, 12 Tons, target 0.05 USD.\n\
     3. Tin Ingots, industrial, 25kg ingot, Sn99.9%, LME registered, Indonesia, 48 Tons, target 0.12 USD.\n\
     4. Respirator Mask, white, S/M, activated carbon, N95, 2 Thousands, target 150.0 USD.\n\
     5. Galvanized Steel Sheet, export, silver, 1.0mm, perforated, 120 Sheets, target 7.5 USD.\n\
     6. Glass Bottle, export, blue, 330ml, soda-lime, wine bottle, Turkey, 250 Pieces, target 25.0 USD.\n\
     7. Metal Shelving Unit, grey, chrome wire, wire rack, Turkey, 1000 sets, target 1.2 USD.\n\
     8. Stainless Steel Coil, export, silver, 1.0mm, mirror polished, India, 100 Tons.",
];

// ── numbered_large: numbered list, 12–15 items ────────────────────────────────
// Longer consolidated procurement lists. Cost scales linearly with item count
// since each item runs through the full qlp pipeline.

const NUMBERED_LARGE: &[&str] = &[
    // GEN-0023 — 15 items, CFR Sydney
    "hi team need a quote for the following items delivered CFR sydney within 7 days. \
     payment terms are T/T 30% deposit and 70% before shipment.\n\n\
     1. 100kg caustic soda flakes (food grade NaOH, 1000kg bag, china) target $0.2\n\
     2. 500L methanol (export quality, 1000L IBC, saudi arabia) target $12.0\n\
     3. 12 cartons USB-C power adapter (30W, metal, multi-port, china)\n\
     4. 10 cartons picture frame (40x50cm, solid wood, multi-opening)\n\
     5. 50 20ft FCL frozen shrimp (black tiger 26/30, indonesia) target $150.0\n\
     6. 50 pcs disposable face mask (white, adult, polypropylene, FFP2, india)\n\
     7. 3000L palm oil (export quality, refined, 1L bottle, colombia) target $7.5\n\
     8. 10 boxes respirator mask (blue, FFP3, activated carbon, china) target $0.12\n\
     9. 120 yards polyester fabric (navy, 180cm wide, 300gsm, china) target $0.5\n\
     10. 50 cartons kraft paper bag (natural brown, 25x10x30cm, recycled, china)\n\
     11. 200 pcs bamboo cutting board (25x15cm, malaysia) target $1.2\n\
     12. 500 sets metal shelving unit (grey, 5-tier, chrome wire, china) target $45\n\
     13. 30 rolls marine rope (6mm, HMPE, 12-strand, turkey)\n\
     14. 100 cartons surgical mask (non-woven SMS, adult, malaysia) target $0.2\n\
     15. 1200 pcs office chair (black, task chair, china) target $99",

    // GEN-0029 — 12 items, T/T 30%
    "hi team, looking for quotes for the following items. all payment terms are T/T 30% deposit, \
     balance vs BL copy.\n\n\
     1. LED Panel Light, Round 120mm, surface mount, origin China, 3 cartons, target $0.12\n\
     2. Picture Frame, Wood tone 60x90cm, aluminum, origin Indonesia, 500 cartons, target $45\n\
     3. Windshield Wiper Blade, 22 inch, hybrid, origin Taiwan, 1500 pieces\n\
     4. Cosmetic Bag, clear PVC w/ handle, 1500 pieces, need OEM service, target $99\n\
     5. Thermal Shipping Label, white 100x150mm, thermal paper, origin India, 24 rolls\n\
     6. Recycled Paper Notebook, A5 80gsm hardcover, origin Vietnam, 50 cartons, target $18\n\
     7. Bamboo Cutting Board, 25x15cm w/ handle, origin Vietnam, 50 sets\n\
     8. Mechanical Keyboard, TKL layout, hot-swap PCB, origin China, 500 pieces, target $45\n\
     9. Stainless Steel Thermos, 500ml, matte black, origin China, 1000 pieces, target $4.5\n\
     10. Safety Helmet, white, HDPE, vented, origin India, 200 pieces, target $2.5\n\
     11. Nitrile Examination Glove, blue, M-XL, sterile, origin Malaysia, 12 cartons, target $18\n\
     12. Office Chair, black, standard task chair, OEM service required, origin China, 100 pieces, target $99",

    // GEN-0029-ext — re-use numbered_medium[2] extended to test 13-item parsing
    "Hello, need quotes for the following consolidated procurement list. CFR Hamburg. Payment D/A 60 days.\n\n\
     1. Cotton T-Shirt, white, unisex, 180gsm, India, 500 pcs, target 2.5 USD\n\
     2. Nitrile Gloves, blue, size M, industrial grade, Malaysia, 1000 cartons, target 18 USD\n\
     3. LED Strip Light, 5m roll, 12V, warm white, IP65, China, 200 rolls, target 4.5 USD\n\
     4. Kraft Paper Bag, natural brown, 25x10x30cm, recycled, China, 1500 pcs, target 0.85 USD\n\
     5. Corrugated Cardboard Sheet, 1200x800mm, kraft brown, Thailand, 24 pallets, target 12 USD\n\
     6. Stainless Steel Wire, 316L, 1.0mm, 100kg coil, South Korea, 50 coils, target 5 USD\n\
     7. PE Stretch Film, natural, 180x200cm, 500 rolls, Vietnam, target 0.35 USD\n\
     8. Bamboo Toothbrush, biodegradable, adult, China, 10000 pcs, target 0.12 USD\n\
     9. Industrial Fan, 24 inch, grey, ATEX certified, Taiwan, 24 pcs, target 1.2 USD\n\
     10. Glass Bottle, 330ml, clear soda-lime, wine, Turkey, 5000 pcs, target 0.35 USD\n\
     11. Latex Mattress, King 180x200cm, white, anti-dust mite, Malaysia, 100 pcs, target 1.2 USD\n\
     12. Surgical Mask, SMS non-woven, adult, Malaysia, 25000 pcs, target 0.2 USD\n\
     13. Cashew Nuts, W450, Cambodia, 3 tonnes, target 1.2 USD",
];

// ── formal_bullets: formal email with markdown bullet items ───────────────────
// Sourced from AI-generated procurement emails in the dataset. Items use the
// `*   **Product Name:** specs (Target: price)` pattern.

const FORMAL_BULLETS: &[&str] = &[
    // GEN-0004 — 8 items, FOB Rotterdam, SGD, markdown bullets
    "Please provide a quotation for the following items, based on FOB destination Rotterdam \
     terms with a 60-day lead time in SGD currency:\n\n\
     *   **Frozen French Fries**: 5000 units (20ft FCL), 9mm, Potato, India origin, target price 2.5.\n\
     *   **Basmati Rice**: 1 Ton, certified organic, 25kg bag, White, Thailand origin, target price 0.2.\n\
     *   **Standing Desk**: 300 Pieces, Walnut, 120x60cm, MDF top, Taiwan origin, target price 300.0.\n\
     *   **Metal Watering Can**: 24 Pieces, Green, 8L, Decorative, Germany origin, target price 4.5.\n\
     *   **Nitrile Examination Gloves**: 12 Cartons, Black, S-XL, Sterile, Thailand origin, target price 18.0.\n\
     *   **Filing Cabinet**: 500 Pieces, Grey, Lateral 2-drawer, Steel, Malaysia origin, target price 45.0.\n\
     *   **Marine Rope**: 500 Reels, 6mm, HMPE, 12-strand, Turkey origin, target price 1.2.\n\
     *   **Bath Towel**: 120 Dozen, Beige, 50x90cm, Hooded, Vietnam origin, target price 0.35.",

    // GEN-0012 — 8 items, DAP Jakarta, formal email header
    "Subject: Request for Quotation - New Supply Partnership Inquiry\n\n\
     Dear Sales Team,\n\n\
     We are formally requesting a quotation for the following items, delivered under DAP terms \
     to Jakarta, with a requested lead time of 14 days:\n\n\
     *   **Kraft Paper Bag:** 1,500 Pieces, Natural Brown, 25x10x30cm, Recycled Kraft Paper (Target: 0.85).\n\
     *   **Corrugated Cardboard Sheet:** 24 Pallets, Kraft brown, 1200x800mm, Thailand (Target: 12.0).\n\
     *   **Fire Extinguisher:** 120 Cartons, ABC dry powder (Target: 0.03).\n\
     *   **Industrial Sewing Machine:** 100 Pieces, Grey, Aluminum body, Zigzag, Germany (Target: 2.5).\n\
     *   **Dried Mango:** 3 Tons, Orange, 250g, Export quality, India (Target: 150.0).\n\
     *   **Nitrile Glove:** 2 Box, Size S, Heavy duty (Target: 0.35).\n\
     *   **Bath Towel:** 120 Dozen, Beige, 50x90cm, Hooded, Vietnam (Target: 0.35).\n\
     *   **Office Chair:** 1,200 Cartons, Black, Standard Task Chair, OEM service required (Target: 99.0).",

    // GEN-0004-ext — mixed dash/asterisk bullets, 7 items
    "Dear Supplier,\n\n\
     Kindly quote for the below items. Incoterm: EXW Shanghai. Payment: T/T 30% deposit. Currency: USD.\n\n\
     - Latex Mattress, King 180x200cm, white, anti-dust mite, 50 pcs, target 1.2 USD\n\
     - Stainless Steel Coil, 316L, 1.0mm, mirror polished, 100 tonnes, target 0.12 USD\n\
     - PE Stretch Film, natural, 500x200cm, 500 rolls, target 0.35 USD\n\
     - LED Driver Module, 12V 200Ah, 50 pcs, target 8.5 USD\n\
     - Nitrile Examination Gloves, black, S-XL, sterile, 12 cartons, target 18 USD\n\
     - Bamboo Cutting Board, 25x15cm, 300 cartons, target 1.2 USD\n\
     - Disposable Syringe, 1ml, Luer lock, rubber piston, Vietnam, 100 boxes, target 0.2 USD",
];

const ALL_CATEGORIES: &[(&str, &[&str])] = &[
    ("chat_small",      CHAT_SMALL),
    ("numbered_medium", NUMBERED_MEDIUM),
    ("numbered_large",  NUMBERED_LARGE),
    ("formal_bullets",  FORMAL_BULLETS),
];

// ── Per-category latency benchmarks ──────────────────────────────────────────

fn bench_categories(c: &mut Criterion) {
    for &(label, rfqs) in ALL_CATEGORIES {
        let mut group = c.benchmark_group(label);
        for (i, rfq) in rfqs.iter().enumerate() {
            group.bench_with_input(
                BenchmarkId::from_parameter(i + 1),
                rfq,
                |b, rfq| b.iter(|| parse_rfq(black_box(rfq))),
            );
        }
        group.finish();
    }
}

// ── Throughput benchmark ──────────────────────────────────────────────────────
// Runs all 12 RFQs in one batch and reports RFQ documents/second.

fn bench_throughput(c: &mut Criterion) {
    let all: Vec<&str> = ALL_CATEGORIES
        .iter()
        .flat_map(|(_, qs)| qs.iter().copied())
        .collect();

    let n = all.len() as u64;

    c.benchmark_group("throughput")
        .throughput(Throughput::Elements(n))
        .bench_function("full_corpus", |b| {
            b.iter(|| {
                for rfq in &all {
                    black_box(parse_rfq(black_box(rfq)));
                }
            })
        });
}

criterion_group!(benches, bench_categories, bench_throughput);
criterion_main!(benches);
