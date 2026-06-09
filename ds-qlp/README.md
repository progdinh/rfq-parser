# DS-QLP

**Deep Search Query Lexer-Parser** — Arobid search preprocessing core.

Converts informal buyer search queries (EN/VI/mixed) into a structured DSL for a downstream SLM, reducing prompt size from ~2000 tokens to ~100–200 tokens.

**Authors:** josephdinh, Arobid

---

## What it does

```
"Cần 300 áo thun và quần jean nam nữ màu đỏ xanh đen kích thước S M L XL"
        ↓
GROUP_1:
  QTY: 300
  UOM: null
  CHUNKS: [áo thun [AND] quần jean]
  COLOR: [red, blue, black]
  SIZE: [S, M, L, XL]
  GENDER: [male, female]
```

The SLM receives this compact DSL and outputs a structured `ProductDetail` (BAML schema).

---

## Position in the Arobid stack

```
Buyer search query
      ↓
  [DS-QLP]          ← this module
      ↓ compact DSL (~100-200 tokens)
  SLM (4B)          ← semantic resolution
      ↓ ProductDetail
  S1 Dify           ← hybrid search (vector + BM25)
      ↓
  S2 BFM-RNK        ← ranking
      ↓
  S3 BFM-DST        ← distribution → backend
```

---

## Architecture

```
Query (EN / VI / mixed)
    ↓
[Lexer]   → tags: Num / Conn / ColorKwd / SizeKwd / MaterialKwd / Color / Size / Material / Gender / Standard / Sytg
    ↓
[Parser]  → grammar rules R1–R9 → Vec<Group>
    ↓
[DSL]     → compact string for SLM (~100–200 tokens)
    ↓
SLM       → ProductDetail { items: ProductSpecs[] }
```

---

## Rust usage

```bash
cargo build
cargo test
```

```rust
use ds_qlp::parse_query;

let result = parse_query("300 áo thun màu đỏ kích thước S M L");
println!("{}", result.slm_dsl);
// GROUP_1:
//   QTY: 300
//   UOM: null
//   CHUNKS: [áo thun]
//   COLOR: [red]
//   SIZE: [S, M, L]

for s in result.to_search_strings() {
    println!("{}", s); // "300 áo thun red S M L"
}
```

---

## Python integration (PyO3 + Maturin)

### Prerequisites

```bash
pip install maturin
# or with uv:
uv add maturin
```

### Development build (fast, no optimisation)

```bash
maturin develop --features python
```

### Release build (optimised wheel)

```bash
maturin build --release --features python
# → produces target/wheels/ds_qlp-*.whl
pip install target/wheels/ds_qlp-*.whl
```

### Google Colab setup

```python
# Cell 1 — install Rust (once per session)
import subprocess, os
subprocess.run(
    "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y",
    shell=True, check=True
)
os.environ["PATH"] += ":/root/.cargo/bin"

# Cell 2 — build and install
!pip install maturin
!maturin develop --features python --manifest-path /path/to/ds_qlp/Cargo.toml
```

### Python API

```python
import ds_qlp

result = ds_qlp.parse_query(
    "300 áo thun và quần jean nam nữ màu đỏ xanh đen kích thước S M L XL"
)

print(result.slm_dsl)   # compact DSL string → feed to SLM
print(result.input)     # original query preserved

for group in result.groups:
    print(group.qty)        # float | None
    print(group.qty_max)    # float | None  (range queries)
    print(group.uom)        # str | None
    print(group.chunks)     # list of Chunk objects
    print(group.colors)     # list[str]
    print(group.sizes)      # list[str]
    print(group.materials)  # list[str]
    print(group.genders)    # list[str]
    print(group.specs)      # list[str]  ← industrial grades: 304, 316L
    print(group.standards)  # list[str]  ← ASTM, ISO...
```

---

## Grammar rules

| Rule | Pattern | Meaning |
|---|---|---|
| R1 | `[Sytg] → [Conn] → [Num]` | New group split on Num after Sytg+Conn |
| R2 | `[Sytg] → [Conn] → [Sytg]` | Chunks within same group |
| R3 | `[Num] → ...` | Leading Num opens a group |
| R4 | `[Kwd] → [Attr]` | Keyword introduces attribute of current group |
| R5 | Adjacent `[Sytg]` tokens | Merged into one token in the lexer |
| R6 | `[Sytg\|Attr] → [Num]` (no Conn) | Spec/grade appended to group, not a new group |
| R7 | `[Num] → [Conn] → [Num]` | Quantity range (`qty` – `qty_max`) |
| R8 | `[NeedKwd]` | Need/intent phrase consumed in lexer |
| R9 | `[Attr] → [Conn] → [Attr\|Kwd]` | Connector between attribute values consumed silently |

Size range expansion is a special case triggered inside R9: when `[Size] → [RangeTo] → [Size]` is detected, the two endpoints are replaced with the full ordered slice from the matching size family (e.g. `S đến XL` → `[S, M, L, XL]`).

---

## Supported languages

- **Vietnamese** — diacritics, units (cái, bộ, tấm, đôi...), color/size/gender keywords
- **English** — full support
- **Mixed EN/VI** — handled naturally (common in Arobid buyer queries)

---

## Project structure

```
ds_qlp/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs       public API + unit tests + PyO3 bindings
    ├── token.rs     Tag, Unit, ConnOp enums
    ├── ontology.rs  attribute dictionaries (colors, sizes, materials, genders, units)
    ├── lexer.rs     byte-based tokenizer → Vec<Tag>
    ├── parser.rs    grammar rules → Vec<Group>
    └── dsl.rs       Vec<Group> → DSL string for SLM
```
