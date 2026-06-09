use pyo3::prelude::*;
use qlp::{Chunk, Group};
use rfqlp::ParseResult as RustResult;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn chunks_to_string(chunks: &[Chunk]) -> String {
    let mut parts: Vec<String> = Vec::new();
    for (i, chunk) in chunks.iter().enumerate() {
        parts.push(chunk.text.clone());
        // append connector after each chunk except the last
        if i < chunks.len() - 1 {
            if let Some(conn) = &chunk.conn {
                parts.push(conn.to_dsl().to_string());
            }
        }
    }
    parts.join(" ")
}

fn group_to_item(index: usize, group: &Group) -> ParsedItem {
    ParsedItem {
        index,
        chunks:    chunks_to_string(&group.chunks),
        qty:       group.qty,
        qty_max:   group.qty_max,
        uom:       group.uom.clone(),
        colors:    group.colors.clone(),
        sizes:     group.sizes.clone(),
        materials: group.materials.clone(),
        genders:   group.genders.clone(),
        specs:     group.specs.clone(),
        standards: group.standards.clone(),
        ambiguous: group.ambiguous.clone(),
        price:     None,
        currency:  None,
        origin:    None,
        dims:      vec![],
    }
}

// ── Python types ──────────────────────────────────────────────────────────────

/// One parsed item — either a group from a simple query or a line item from an RFQ.
/// chunks  = C1 : product noun phrases joined with connectors ("polo shirt OR t-shirt")
/// all other fields = S1 : ontology-matched and normalised specs (trusted, no SLM needed)
#[pyclass]
#[derive(Clone)]
pub struct ParsedItem {
    // C1
    #[pyo3(get)] pub chunks:    String,
    #[pyo3(get)] pub index:     usize,
    // S1 — ontology specs
    #[pyo3(get)] pub qty:       Option<f64>,
    #[pyo3(get)] pub qty_max:   Option<f64>,
    #[pyo3(get)] pub uom:       Option<String>,
    #[pyo3(get)] pub colors:    Vec<String>,
    #[pyo3(get)] pub sizes:     Vec<String>,
    #[pyo3(get)] pub materials: Vec<String>,
    #[pyo3(get)] pub genders:   Vec<String>,
    #[pyo3(get)] pub specs:     Vec<String>,
    #[pyo3(get)] pub standards: Vec<String>,
    #[pyo3(get)] pub ambiguous: Vec<String>,
    // RFQ-specific (None for simple queries)
    #[pyo3(get)] pub price:     Option<f64>,
    #[pyo3(get)] pub currency:  Option<String>,
    #[pyo3(get)] pub origin:    Option<String>,
    #[pyo3(get)] pub dims:      Vec<String>,
}

/// Trade terms extracted from an RFQ document (None fields = not found).
#[pyclass]
#[derive(Clone)]
pub struct TradeTerms {
    #[pyo3(get)] pub currency:    Option<String>,
    #[pyo3(get)] pub incoterm:    Option<String>,
    #[pyo3(get)] pub destination: Option<String>,
    #[pyo3(get)] pub lead_time:   Option<String>,
    #[pyo3(get)] pub payment:     Option<String>,
}

/// Top-level result returned by parse().
/// is_rfq = true when the input was routed through the RFQ pipeline.
#[pyclass]
pub struct ParseResult {
    #[pyo3(get)] pub items:   Vec<ParsedItem>,
    #[pyo3(get)] pub is_rfq:  bool,
    // flattened trade terms — None on every field when is_rfq=false or no terms found
    #[pyo3(get)] pub trade_currency:    Option<String>,
    #[pyo3(get)] pub trade_incoterm:    Option<String>,
    #[pyo3(get)] pub trade_destination: Option<String>,
    #[pyo3(get)] pub trade_lead_time:   Option<String>,
    #[pyo3(get)] pub trade_payment:     Option<String>,
}

#[pymethods]
impl ParseResult {
    /// Convenience: returns a TradeTerms object if any trade term was found.
    fn trade_terms(&self, py: Python<'_>) -> PyResult<PyObject> {
        let has_terms = self.trade_currency.is_some()
            || self.trade_incoterm.is_some()
            || self.trade_destination.is_some()
            || self.trade_lead_time.is_some()
            || self.trade_payment.is_some();
        if has_terms {
            let tt = TradeTerms {
                currency:    self.trade_currency.clone(),
                incoterm:    self.trade_incoterm.clone(),
                destination: self.trade_destination.clone(),
                lead_time:   self.trade_lead_time.clone(),
                payment:     self.trade_payment.clone(),
            };
            Ok(Py::new(py, tt)?.into_any())
        } else {
            Ok(py.None())
        }
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

/// Parse a raw query or RFQ document.
/// Routes automatically: simple product queries return one ParsedItem per group,
/// RFQ documents return one ParsedItem per line item.
#[pyfunction]
fn parse(query: &str) -> PyResult<ParseResult> {
    match rfqlp::parse(query) {
        RustResult::SimpleQuery(pq) => {
            let items = pq.groups
                .iter()
                .enumerate()
                .map(|(i, g)| group_to_item(i + 1, g))
                .collect();
            Ok(ParseResult {
                items,
                is_rfq:              false,
                trade_currency:      None,
                trade_incoterm:      None,
                trade_destination:   None,
                trade_lead_time:     None,
                trade_payment:       None,
            })
        }

        RustResult::Rfq(rfq) => {
            let items = rfq.items.iter().map(|rfq_item| {
                let group = rfq_item.groups.first();
                let (chunks, colors, sizes, materials, genders, specs, standards, ambiguous, qty_max) =
                    group.map(|g| (
                        chunks_to_string(&g.chunks),
                        g.colors.clone(),
                        g.sizes.clone(),
                        g.materials.clone(),
                        g.genders.clone(),
                        g.specs.clone(),
                        g.standards.clone(),
                        g.ambiguous.clone(),
                        g.qty_max,
                    )).unwrap_or_default();

                ParsedItem {
                    index:     rfq_item.index,
                    chunks,
                    qty:       rfq_item.qty,
                    qty_max,
                    uom:       rfq_item.uom.clone(),
                    colors,
                    sizes,
                    materials,
                    genders,
                    specs,
                    standards,
                    ambiguous,
                    price:     rfq_item.price,
                    currency:  rfq_item.currency.clone(),
                    origin:    rfq_item.origin.clone(),
                    dims:      rfq_item.dims.clone(),
                }
            }).collect();

            let tt = &rfq.trade_terms;
            Ok(ParseResult {
                items,
                is_rfq:              true,
                trade_currency:      tt.currency.clone(),
                trade_incoterm:      tt.incoterm.clone(),
                trade_destination:   tt.destination.clone(),
                trade_lead_time:     tt.lead_time.clone(),
                trade_payment:       tt.payment.clone(),
            })
        }
    }
}

// ── Module ────────────────────────────────────────────────────────────────────

#[pymodule]
fn rfq_parser(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ParsedItem>()?;
    m.add_class::<TradeTerms>()?;
    m.add_class::<ParseResult>()?;
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    Ok(())
}
