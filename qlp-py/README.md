# rfq-parser

Pure **syntagmatic** rule-based parser for **English RFQ documents** and **short product queries**, with a **Vietnamese mode**.

No semantic model, no embeddings, no LLM — purely structural analysis of surface form. Built in Rust with Python bindings via PyO3, it runs in **<1ms** and is designed as a lightweight **pre-processing layer** for LLM-based product name and spec extraction: strip quantities, units, colours, sizes, and materials from the raw text so the model only sees what it actually needs to reason about.

## Install

```bash
pip install rfq-parser
```

## Quick start

```python
import rfq_parser

# Short product query
result = rfq_parser.parse("500 polo shirts red size XL")
item = result.items[0]
print(item.chunks)     # "polo shirts"
print(item.qty)        # 500.0
print(item.uom)        # None (implicit pieces)
print(item.colors)     # ["red"]
print(item.sizes)      # ["XL"]

# RFQ document
rfq = """
Please quote for:
1. Car Battery, 3000 Cartons, Black, 70Ah, Japan. Target: 12 USD/unit.
2. Silk Fabric, 500 Yards, Red, 150cm width, China.
"""
result = rfq_parser.parse(rfq)
for item in result.items:
    print(f"{item.index}. {item.chunks} — qty={item.qty} {item.uom}")
    print(f"   colors={item.colors}, origin={item.origin}")

if result.is_rfq:
    tt = result.trade_terms()  # TradeTerms | None
```

## What it extracts

Each `ParsedItem` contains:

| Field | Description |
|-------|-------------|
| `chunks` | Product noun phrase(s), connectors preserved (`"polo shirt OR t-shirt"`) |
| `qty`, `qty_max`, `uom` | Quantity and unit of measure |
| `colors`, `sizes`, `materials` | Ontology-matched specs |
| `standards`, `specs` | Technical standards and extra specs |
| `origin`, `price`, `currency` | RFQ-specific fields |
| `dims` | Dimensions (e.g. `["150cm", "70Ah"]`) |

`ParseResult.trade_terms()` returns a `TradeTerms` object with `incoterm`, `currency`, `destination`, `lead_time`, `payment` for RFQ documents.

## Why syntagmatic / rule-based?

- **No semantic bindings** — no embeddings, no model weights, no ontology lookups at runtime
- **Deterministic** — same input always gives the same output
- **Fast** — <1ms, safe to call on every keystroke or in a streaming pipeline
- **No dependencies** — no model to download, no API key
- **LLM pre-processing** — strips structured fields (qty, uom, colours, sizes, materials) so downstream LLM calls receive a clean noun phrase, reducing token waste and hallucination surface

## Supported

- English, Vietnamese, French product queries
- Informal RFQ emails and formal procurement documents
- Multi-item RFQs with trade terms (Incoterms, payment, lead time)

## License

MIT
