// dsl.rs — Serializes Vec<Group> into the compact DSL string for the SLM
//
// Format:
//   GROUP_1:
//     QTY: <number or null>
//     QTY_MAX: <number>        (only if range)
//     UOM: <unit or null>
//     CHUNKS: [chunk1 [AND/OR] chunk2 ...]
//     COLOR: [val1, val2]      (omitted if empty)
//     SIZE: [val1, val2]       (omitted if empty)
//     MATERIAL: [val1]         (omitted if empty)
//     GENDER: [val1]           (omitted if empty)
//     SPEC: [304, 316L]        (omitted if empty)
//     STANDARD: [ASTM A351]    (omitted if empty)
//
//   GROUP_2:
//     ...

use crate::parser::Group;

pub fn to_dsl(groups: &[Group]) -> String {
    if groups.is_empty() {
        return String::new();
    }

    groups
        .iter()
        .enumerate()
        .map(|(idx, g)| group_to_dsl(idx + 1, g))
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn group_to_dsl(n: usize, g: &Group) -> String {
    let mut lines = Vec::new();

    lines.push(format!("GROUP_{}:", n));

    // QTY
    let qty_str = g.qty.map_or("null".into(), |v| fmt_qty(v));
    lines.push(format!("  QTY: {}", qty_str));

    // QTY_MAX (range only)
    if let Some(max) = g.qty_max {
        lines.push(format!("  QTY_MAX: {}", fmt_qty(max)));
    }

    // UOM
    let uom_str = g.uom.as_deref().unwrap_or("null");
    lines.push(format!("  UOM: {}", uom_str));

    // CHUNKS
    let chunks_str = build_chunks_str(g);
    lines.push(format!("  CHUNKS: {}", chunks_str));

    // Attribute fields — omitted if empty
    if !g.colors.is_empty() {
        lines.push(format!("  COLOR: [{}]", g.colors.join(", ")));
    }
    if !g.sizes.is_empty() {
        lines.push(format!("  SIZE: [{}]", g.sizes.join(", ")));
    }
    if !g.materials.is_empty() {
        lines.push(format!("  MATERIAL: [{}]", g.materials.join(", ")));
    }
    if !g.genders.is_empty() {
        lines.push(format!("  GENDER: [{}]", g.genders.join(", ")));
    }
    if !g.specs.is_empty() {
        lines.push(format!("  SPEC: [{}]", g.specs.join(", ")));
    }
    if !g.standards.is_empty() {
        lines.push(format!("  STANDARD: [{}]", g.standards.join(", ")));
    }
    if !g.ambiguous.is_empty() {
        lines.push(format!("  AMBIGUOUS: [{}]", g.ambiguous.join(", ")));
    }

    lines.join("\n")
}

/// Build the CHUNKS string. The last chunk's connector is always suppressed
/// (it would be a group-boundary connector, not intra-group).
fn build_chunks_str(g: &Group) -> String {
    if g.chunks.is_empty() {
        return "[_]".to_string();
    }
    let mut parts = Vec::new();
    for (i, chunk) in g.chunks.iter().enumerate() {
        parts.push(chunk.text.clone());
        // Only show connector if NOT the last chunk
        if i < g.chunks.len() - 1 {
            if let Some(conn) = &chunk.conn {
                parts.push(format!("[{}]", conn.to_dsl()));
            }
        }
    }
    format!("[{}]", parts.join(" "))
}

fn fmt_qty(v: f64) -> String {
    if v.fract() == 0.0 { format!("{}", v as u64) }
    else { format!("{:.2}", v) }
}
