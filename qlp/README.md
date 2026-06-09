# qlp

Rule-based query lexer-parser for product search queries (EN / VI / mixed).

Converts informal buyer queries into structured groups with quantity, unit, specs, colors, sizes, materials, and product noun phrases — in <1ms, no LLM required.

```
"300 áo thun và quần jean nam nữ màu đỏ xanh kích thước S M L XL"
        ↓
GROUP_1:
  QTY: 300
  CHUNKS: [áo thun AND quần jean]
  COLOR: [red, blue]
  SIZE: [S, M, L, XL]
  GENDER: [male, female]
```

## Architecture

```
Query (EN / VI / mixed)
    ↓
[Lexer]   → token tags: Num / Conn / ColorKwd / SizeKwd / MaterialKwd / ...
    ↓
[Parser]  → grammar rules R1–R9 → Vec<Group>
    ↓
[DSL]     → compact string for downstream use
```

## Usage (Rust)

```rust
use qlp::parse_query;

let result = parse_query("300 áo thun màu đỏ kích thước S M L");

for group in &result.groups {
    println!("{:?}", group.qty);       // Some(300.0)
    println!("{:?}", group.colors);    // ["red"]
    println!("{:?}", group.sizes);     // ["S", "M", "L"]
}
```

## Grammar rules

| Rule | Pattern | Meaning |
|---|---|---|
| R1 | `[Sytg] → [Conn] → [Num]` | New group on Num after Sytg+Conn |
| R2 | `[Sytg] → [Conn] → [Sytg]` | Chunks within same group |
| R3 | `[Num] → ...` | Leading Num opens a group |
| R4 | `[Kwd] → [Attr]` | Keyword introduces attribute |
| R5 | Adjacent `[Sytg]` tokens | Merged in lexer |
| R6 | `[Sytg\|Attr] → [Num]` (no Conn) | Spec/grade appended, not a new group |
| R7 | `[Num] → [Conn] → [Num]` | Quantity range (`qty` – `qty_max`) |
| R9 | `[Attr] → [Conn] → [Attr\|Kwd]` | Connector between attribute values |

Size range expansion: `S đến XL` → `[S, M, L, XL]`.

## Supported languages

- **Vietnamese** — diacritics, units (cái, bộ, tấm...), keywords
- **English** — full support
- **Mixed EN/VI** — handled naturally

## Project structure

```
src/
├── lib.rs       public API + unit tests
├── token.rs     Tag, Unit, ConnOp enums
├── ontology.rs  attribute dictionaries (colors, sizes, materials, units...)
├── lexer.rs     byte-based tokenizer → Vec<Token>
├── parser.rs    grammar rules → Vec<Group>
└── dsl.rs       Vec<Group> → compact DSL string
```
