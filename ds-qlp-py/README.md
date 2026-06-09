# rfq-parser

Fast, rule-based parser for **RFQ (Request for Quotation) documents** and **short product queries**.

Built in Rust with Python bindings via PyO3. Extracts structured product specifications in **<1ms** — no LLM, no network calls.

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

## Why rule-based?

- **Deterministic** — same input always gives the same output
- **Fast** — <1ms, safe to call on every keystroke
- **No dependencies** — no model to download, no API key
- **LLM-friendly** — use the structured output as context to reduce token count in downstream LLM calls

## Supported

- English, Vietnamese, French product queries
- Informal RFQ emails and formal procurement documents
- Multi-item RFQs with trade terms (Incoterms, payment, lead time)

## License

MIT
