# DS-QLP Performance Benchmark Report

**Library:** `ds_qlp` v0.1.0  
**Tool:** Criterion 0.5 — 100 samples per benchmark, 3 s warmup  
**Build:** `cargo bench` (`opt-level = 3`, `lto = true`, `codegen-units = 1`)  
**Corpus:** 45 queries across 9 categories (`qlp_corpus.jsonl`)

---

## Throughput — Full Corpus

| Metric | Value |
|---|---|
| Corpus size | 45 queries |
| Batch time (mean) | 270.69 µs |
| **Throughput** | **166,240 queries / second** |
| CI (low–high) | 165,420 – 167,130 q/s |

> DS-QLP processes ~166k queries/second on a single thread.  
> The downstream SLM (4B) handles at most a few hundred queries/second — DS-QLP is not a bottleneck.

---

## Per-Category Latency

All times are `[low  mean  high]` at 95% confidence interval.

### Vietnamese — Simple
Bare product + quantity, no attributes.

| # | Query | Mean (µs) | CI |
|---|---|---|---|
| 1 | `300 áo thun` | 2.675 | [2.653 – 2.703] |
| 2 | `500 kg thép` | 1.841 | [1.830 – 1.853] |
| 3 | `200 cái quần jean` | 2.852 | [2.834 – 2.871] |
| 4 | `50 đôi vớ` | 1.864 | [1.858 – 1.871] |
| 5 | `1000 bộ quần áo` | 2.670 | [2.655 – 2.684] |

**Category range: 1.8 – 2.9 µs**

---

### Vietnamese — Medium
Single product with 1–3 attributes (color, size, gender, material).

| # | Query | Mean (µs) | CI |
|---|---|---|---|
| 1 | `300 áo thun màu đỏ` | 3.738 | [3.713 – 3.762] |
| 2 | `500 áo polo nam size L` | 3.891 | [3.869 – 3.914] |
| 3 | `khoảng 200 quần jean nữ màu xanh` | 4.464 | [4.430 – 4.499] |
| 4 | `tôi cần 150 bộ quần áo cotton` | 4.375 | [4.349 – 4.398] |
| 5 | `300 áo thun màu đỏ xanh kích thước S M L` | 5.893 | [5.784 – 6.064] |

**Category range: 3.7 – 6.1 µs**

---

### Vietnamese — Complex
Multi-group, multi-attribute, size/qty ranges.

| # | Query | Mean (µs) | CI |
|---|---|---|---|
| 1 | `50 áo và quần và slip hoặc 30 đôi vớ` | 7.098 | [7.057 – 7.136] |
| 2 | `tôi muốn mua 300 áo thun và quần jean nam nữ màu đỏ xanh đen kích thước S M L XL` | 10.199 | [10.135 – 10.262] |
| 3 | `tôi cần 150 áo thun hoặc áo polo size S đến XL màu đỏ` | 8.629 | [8.576 – 8.673] |
| 4 | `500 quần hoặc áo màu trắng hoặc đen kích thước M đến XL` | 7.137 | [7.109 – 7.161] |
| 5 | `từ 100 đến 200 áo thun và quần jean nam màu đen size L XL` | 8.552 | [8.508 – 8.590] |

**Category range: 7.1 – 10.2 µs**

---

### English — Simple

| # | Query | Mean (µs) | CI |
|---|---|---|---|
| 1 | `500 polo shirts` | 2.909 | [2.896 – 2.922] |
| 2 | `100 small red cotton t-shirts` | 4.906 | [4.888 – 4.924] |
| 3 | `200 kg stainless steel` | 2.774 | [2.761 – 2.790] |
| 4 | `1000 pcs nitrile gloves` | 2.931 | [2.904 – 2.958] |
| 5 | `50 sets metal shelving` | 3.044 | [3.027 – 3.063] |

**Category range: 2.8 – 4.9 µs**

---

### English — Medium

| # | Query | Mean (µs) | CI |
|---|---|---|---|
| 1 | `500 kg stainless steel pipe 304 DN50` | 5.433 | [5.398 – 5.471] |
| 2 | `1000 blue nitrile gloves size XL industrial grade` | 6.415 | [6.378 – 6.454] |
| 3 | `120 tons zinc ingots silver` | 4.935 | [4.903 – 4.962] |
| 4 | `300 cartons wall paint light grey washable` | 6.493 | [6.452 – 6.532] |
| 5 | `100 small or large red cotton and blue polyester polo shirts and t-shirts` | 9.297 | [9.242 – 9.356] |

**Category range: 4.9 – 9.3 µs**

---

### Mixed EN/VI

| # | Query | Mean (µs) | CI |
|---|---|---|---|
| 1 | `300 áo thun blue size S M L` | 4.854 | [4.797 – 4.929] |
| 2 | `500 polo shirts nam nữ màu đỏ và xanh` | 5.543 | [5.507 – 5.577] |
| 3 | `200 quần jean black cotton size 30 đến 34` | 6.609 | [6.566 – 6.657] |
| 4 | `tôi cần 100 cotton t-shirts size XL màu navy` | 5.102 | [5.058 – 5.150] |
| 5 | `500 áo thun + quần jean màu đen size S đến XL` | 7.870 | [7.812 – 7.929] |

**Category range: 4.9 – 7.9 µs**

---

### Chatbot-style
Conversational, longer intent phrases, unstructured trailing specs.

| # | Query | Mean (µs) | CI |
|---|---|---|---|
| 1 | `I need around 500 cartons of nitrile gloves blue size M from Malaysia` | 9.651 | [9.575 – 9.725] |
| 2 | `looking for 1000 units lithium battery 12V 200Ah solar storage` | 8.718 | [8.653 – 8.785] |
| 3 | `sourcing 1200 meters denim fabric black cotton from Bangladesh` | 7.587 | [7.542 – 7.637] |
| 4 | `I want 300 pieces LED panel lights white surface mount aluminum frame` | 8.026 | [7.996 – 8.054] |
| 5 | `I'm looking for organic cashew nuts W240 whole grade about 2 FCL from Cambodia` | 10.284 | [10.195 – 10.369] |

**Category range: 7.6 – 10.3 µs**

---

### Industrial / Technical
Grade specs, standards, DN/schedule sizes.

| # | Query | Mean (µs) | CI |
|---|---|---|---|
| 1 | `500 kg ống thép 304` | 3.407 | [3.385 – 3.429] |
| 2 | `200 ống thép không gỉ DN50 SCH40` | 6.027 | [5.990 – 6.061] |
| 3 | `1000 kg thép carbon ASTM A36` | 4.266 | [4.251 – 4.282] |
| 4 | `500 nhôm 6061 6 inch` | 4.332 | [4.318 – 4.345] |
| 5 | `300 ống inox 316L DN100 schedule 40` | 6.099 | [6.081 – 6.120] |

**Category range: 3.4 – 6.1 µs**

---

### Range Queries
Quantity ranges (`từ...đến`) and size range expansion.

| # | Query | Mean (µs) | CI |
|---|---|---|---|
| 1 | `từ 100 đến 200 áo thun` | 3.440 | [3.422 – 3.458] |
| 2 | `100 đến 500 kg thép` | 2.469 | [2.457 – 2.482] |
| 3 | `300 áo thun size S đến XL` | 4.689 | [4.651 – 4.724] |
| 4 | `500 quần jean size 30 đến 34 màu đen` | 6.099 | [6.068 – 6.132] |
| 5 | `100 đến 200 áo thun hoặc áo polo size XXS đến XL màu đỏ xanh` | 9.149 | [9.098 – 9.204] |

**Category range: 2.5 – 9.2 µs**

---

## Summary

| Category | Min (µs) | Max (µs) | Notes |
|---|---|---|---|
| vi_simple | 1.84 | 2.85 | Fastest category |
| vi_medium | 3.74 | 5.89 | Linear growth with attribute count |
| vi_complex | 7.10 | 10.20 | Multi-group + size range expansion |
| en_simple | 2.77 | 4.91 | Slightly slower than VI simple (ontology scan) |
| en_medium | 4.90 | 9.30 | Long EN queries approach complex VI |
| mixed | 4.80 | 7.87 | Consistent with token count |
| chatbot | 7.59 | 10.28 | Intent phrases add marginal overhead |
| industrial | 3.41 | 6.10 | Standards detection cheap |
| range | 2.47 | 9.15 | Cost scales with size family expansion |
| **full corpus** | — | — | **166,240 q/s · 270 µs / 45 queries** |

### Key observations

- **Latency scales with token count**, not query language. The lexer's per-character scan dominates.
- **Simplest queries (short VI)** complete in under 2 µs.
- **Most complex queries** (long EN, full-range VI complex) peak around 10 µs.
- **Size range expansion** (`S đến XL` → 6 values) adds ~2–3 µs versus a plain size list.
- **Chatbot intent phrases** (`I need`, `I'm looking for`) are consumed in the first pass with negligible overhead.
- **DS-QLP is not a bottleneck** at any realistic request rate. At 166k q/s on one thread, it could serve a 1,000 RPS API with <0.1% of a single core.

---

## Reproducibility

```bash
cargo bench
# HTML reports → target/criterion/
```

To compare against a future change:
```bash
cargo bench -- --save-baseline before
# make changes
cargo bench -- --baseline before
```
