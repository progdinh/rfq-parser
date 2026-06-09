// token.rs — Token types for the Arobid query tagger (v3)
//
// Key change: ProductWord removed. Products are residual Sytg tokens.
// Grade removed from tags — parser handles it via R6 (Num after Sytg = spec).
// All numbers emitted as Num; parser uses grammar rules to distinguish qty vs spec.

#[derive(Debug, Clone, PartialEq)]
pub enum Unit {
    Kg, G, Tonne,
    Pcs, Sets, Dozens, Rolls, Sheets,
    Doi,        // đôi — pair (socks, shoes, etc.)
    M2, Cm, Mm, Meters,
    L, Ml,
    Cai,        // cái
    Bo,         // bộ
    Cuon,       // cuộn
    Tam,        // tấm
    Drums,
    Pallets,
    Cartons,
    Box,
    Reels,
    Hundreds,
    Thousands,
    Fcl,        // 20ft FCL shipping container
    None,
}

impl Unit {
    pub fn to_str(&self) -> Option<&'static str> {
        match self {
            Unit::Kg     => Some("kg"),
            Unit::G      => Some("g"),
            Unit::Tonne  => Some("tonne"),
            Unit::Pcs    => Some("pcs"),
            Unit::Doi    => Some("đôi"),
            Unit::Sets   => Some("sets"),
            Unit::Dozens => Some("dozens"),
            Unit::Rolls  => Some("rolls"),
            Unit::Sheets => Some("sheets"),
            Unit::M2     => Some("m²"),
            Unit::Cm     => Some("cm"),
            Unit::Mm     => Some("mm"),
            Unit::L      => Some("l"),
            Unit::Ml     => Some("ml"),
            Unit::Cai    => Some("cái"),
            Unit::Bo     => Some("bộ"),
            Unit::Cuon   => Some("cuộn"),
            Unit::Tam      => Some("tấm"),
            Unit::Drums    => Some("drums"),
            Unit::Pallets  => Some("pallets"),
            Unit::Cartons  => Some("cartons"),
            Unit::Box      => Some("box"),
            Unit::Reels    => Some("reels"),
            Unit::Hundreds  => Some("hundreds"),
            Unit::Thousands => Some("thousands"),
            Unit::Meters   => Some("meters"),
            Unit::Fcl      => Some("FCL"),
            Unit::None     => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnOp {
    And,
    Or,
    Comma,
    RangeTo, // "đến" — range endpoint connector
}

impl ConnOp {
    pub fn to_dsl(&self) -> &'static str {
        match self {
            ConnOp::And | ConnOp::Comma => "AND",
            ConnOp::Or                  => "OR",
            ConnOp::RangeTo             => "TO",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tag {
    // Structural
    Num { value: f64, unit: Unit },
    Conn(ConnOp),

    // Keyword markers
    ColorKwd,
    SizeKwd,
    MaterialKwd,

    // Typed attribute values
    Color(String),
    Size(String),
    Material(String),
    Gender(String),
    Standard(String),

    // Geographic origin — emitted in EN mode for known country/region names.
    // Routed to Group.ambiguous so the SLM can classify as ORIGIN / BRAND / etc.
    Provenance(String),

    // Residual — noun phrases and product names
    Sytg(String),
}
