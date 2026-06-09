use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use qlp::parse_query;

// ── Query corpus by category ──────────────────────────────────────────────────
// Mirrors qlp_corpus.jsonl — edit both together when adding queries.

const VI_SIMPLE: &[&str] = &[
    "300 áo thun",
    "500 kg thép",
    "200 cái quần jean",
    "50 đôi vớ",
    "1000 bộ quần áo",
];

const VI_MEDIUM: &[&str] = &[
    "300 áo thun màu đỏ",
    "500 áo polo nam size L",
    "khoảng 200 quần jean nữ màu xanh",
    "tôi cần 150 bộ quần áo cotton",
    "300 áo thun màu đỏ xanh kích thước S M L",
];

const VI_COMPLEX: &[&str] = &[
    "50 áo và quần và slip hoặc 30 đôi vớ",
    "tôi muốn mua 300 áo thun và quần jean nam nữ màu đỏ xanh đen kích thước S M L XL",
    "tôi cần 150 áo thun hoặc áo polo size S đến XL màu đỏ",
    "500 quần hoặc áo màu trắng hoặc đen kích thước M đến XL",
    "từ 100 đến 200 áo thun và quần jean nam màu đen size L XL",
];

const EN_SIMPLE: &[&str] = &[
    "500 polo shirts",
    "100 small red cotton t-shirts",
    "200 kg stainless steel",
    "1000 pcs nitrile gloves",
    "50 sets metal shelving",
];

const EN_MEDIUM: &[&str] = &[
    "500 kg stainless steel pipe 304 DN50",
    "1000 blue nitrile gloves size XL industrial grade",
    "120 tons zinc ingots silver",
    "300 cartons wall paint light grey washable",
    "100 small or large red cotton and blue polyester polo shirts and t-shirts",
];

const MIXED: &[&str] = &[
    "300 áo thun blue size S M L",
    "500 polo shirts nam nữ màu đỏ và xanh",
    "200 quần jean black cotton size 30 đến 34",
    "tôi cần 100 cotton t-shirts size XL màu navy",
    "500 áo thun + quần jean màu đen size S đến XL",
];

const CHATBOT: &[&str] = &[
    "I need around 500 cartons of nitrile gloves blue size M from Malaysia",
    "looking for 1000 units lithium battery 12V 200Ah solar storage",
    "sourcing 1200 meters denim fabric black cotton from Bangladesh",
    "I want 300 pieces LED panel lights white surface mount aluminum frame",
    "I'm looking for organic cashew nuts W240 whole grade about 2 FCL from Cambodia",
];

const INDUSTRIAL: &[&str] = &[
    "500 kg ống thép 304",
    "200 ống thép không gỉ DN50 SCH40",
    "1000 kg thép carbon ASTM A36",
    "500 nhôm 6061 6 inch",
    "300 ống inox 316L DN100 schedule 40",
];

const RANGE: &[&str] = &[
    "từ 100 đến 200 áo thun",
    "100 đến 500 kg thép",
    "300 áo thun size S đến XL",
    "500 quần jean size 30 đến 34 màu đen",
    "100 đến 200 áo thun hoặc áo polo size XXS đến XL màu đỏ xanh",
];

const ALL_CATEGORIES: &[(&str, &[&str])] = &[
    ("vi_simple",  VI_SIMPLE),
    ("vi_medium",  VI_MEDIUM),
    ("vi_complex", VI_COMPLEX),
    ("en_simple",  EN_SIMPLE),
    ("en_medium",  EN_MEDIUM),
    ("mixed",      MIXED),
    ("chatbot",    CHATBOT),
    ("industrial", INDUSTRIAL),
    ("range",      RANGE),
];

// ── Per-category latency benchmarks ──────────────────────────────────────────
// Measures parse_query latency for each query in each category.
// Use these to identify which query shapes are slowest.

fn bench_categories(c: &mut Criterion) {
    for &(label, queries) in ALL_CATEGORIES {
        let mut group = c.benchmark_group(label);
        for (i, q) in queries.iter().enumerate() {
            group.bench_with_input(
                BenchmarkId::from_parameter(i + 1),
                q,
                |b, q| b.iter(|| parse_query(black_box(q))),
            );
        }
        group.finish();
    }
}

// ── Throughput benchmark ──────────────────────────────────────────────────────
// Runs the full corpus in one batch and reports elements/second.
// This is the headline throughput number for documentation.

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
                for q in &all {
                    black_box(parse_query(black_box(q)));
                }
            })
        });
}

criterion_group!(benches, bench_categories, bench_throughput);
criterion_main!(benches);
