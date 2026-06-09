// ontology.rs — domain lookup tables (colors, sizes, materials, units ...)
//
// KEY CHANGE from v2: `products` list is REMOVED.
// Products are now residual Sytg tokens — no ontology lookup needed.
// This ontology covers only finite, enumerable attribute categories.

use std::collections::HashMap;
use std::sync::LazyLock;

pub static ONTOLOGY: LazyLock<Ontology> = LazyLock::new(Ontology::default);

pub struct Ontology {
    pub colors:           HashMap<&'static str, &'static str>, // surface → canonical EN
    pub sizes:            HashMap<&'static str, &'static str>,
    pub materials:        HashMap<&'static str, &'static str>,
    pub genders:          HashMap<&'static str, &'static str>, // always typed as KWD
    pub standards:        Vec<&'static str>,   // prefix-match (e.g. "astm", "iso")
    pub size_keywords:    Vec<&'static str>,
    pub color_keywords:   Vec<&'static str>,
    pub mat_keywords:     Vec<&'static str>,
    pub need_keywords:    Vec<&'static str>,   // consumed, never emitted
    pub proper_compounds: Vec<&'static str>,   // multi-word proper nouns (geography, brands)
    pub geo_origins:      HashMap<&'static str, &'static str>, // EN commodity origins
    pub coord_and:        Vec<&'static str>,
    pub coord_or:         Vec<&'static str>,
    pub coord_range:      Vec<&'static str>, // emitted as Conn(RangeTo)
    pub units:            HashMap<&'static str, crate::token::Unit>,
}

impl Default for Ontology {
    fn default() -> Self {
        use crate::token::Unit;

        // ── Colors ──────────────────────────────────────────────────────────
        let mut colors = HashMap::new();
        colors.insert("đỏ",      "red");
        colors.insert("xanh",    "blue");
        colors.insert("xanh lam","blue");
        colors.insert("xanh lá", "green");
        colors.insert("đen",     "black");
        colors.insert("trắng",   "white");
        colors.insert("vàng",    "yellow");
        colors.insert("cam",     "orange");
        colors.insert("tím",     "purple");
        colors.insert("hồng",    "pink");
        colors.insert("xám",     "gray");
        colors.insert("nâu",     "brown");
        colors.insert("red",     "red");
        colors.insert("blue",    "blue");
        colors.insert("green",   "green");
        colors.insert("black",   "black");
        colors.insert("white",   "white");
        colors.insert("yellow",  "yellow");
        colors.insert("orange",  "orange");
        colors.insert("purple",  "purple");
        colors.insert("pink",    "pink");
        colors.insert("gray",    "gray");
        colors.insert("grey",    "gray");
        colors.insert("brown",   "brown");
        colors.insert("navy",        "navy");
        colors.insert("beige",       "beige");
        colors.insert("silver",      "silver");
        colors.insert("amber",       "amber");
        colors.insert("clear",       "clear");
        colors.insert("natural",     "natural");
        colors.insert("light grey",  "light_gray");
        colors.insert("light gray",  "light_gray");
        colors.insert("pale yellow", "pale_yellow");

        // ── Sizes ────────────────────────────────────────────────────────────
        let mut sizes = HashMap::new();
        sizes.insert("xxs",  "XXS");
        sizes.insert("xs",   "XS"); sizes.insert("s",    "S");
        sizes.insert("m",    "M");  sizes.insert("l",    "L");
        sizes.insert("xl",   "XL"); sizes.insert("xxl",  "XXL");
        sizes.insert("xxxl", "XXXL"); sizes.insert("2xl", "2XL");
        sizes.insert("3xl",  "3XL"); sizes.insert("4xl",  "4XL");
        sizes.insert("small",  "small");
        sizes.insert("medium", "medium");
        sizes.insert("large",  "large");
        // Numeric clothing — waist/EU range
        for &s in &["27","28","29","30","31","32","33","34","35",
                    "36","37","38","39","40","41","42","43","44","45","46"] {
            sizes.insert(s, s);
        }
        // Industrial — DN pipe sizes
        sizes.insert("dn15",  "DN15"); sizes.insert("dn20",  "DN20");
        sizes.insert("dn25",  "DN25"); sizes.insert("dn32",  "DN32");
        sizes.insert("dn40",  "DN40"); sizes.insert("dn50",  "DN50");
        sizes.insert("dn65",  "DN65"); sizes.insert("dn80",  "DN80");
        sizes.insert("dn100", "DN100"); sizes.insert("dn150","DN150");
        sizes.insert("dn200", "DN200"); sizes.insert("dn300","DN300");
        // Schedule
        sizes.insert("schedule 10", "SCH10");
        sizes.insert("schedule 40", "SCH40");
        sizes.insert("schedule 80", "SCH80");
        sizes.insert("sch 40", "SCH40");
        sizes.insert("sch 80", "SCH80");

        // ── Materials ────────────────────────────────────────────────────────
        let mut materials = HashMap::new();
        materials.insert("cotton",        "cotton");
        materials.insert("polyester",     "polyester");
        // Note: "jean" and "denim" removed from materials —
        // they appear as product name suffixes (quần jean, denim jacket)
        // and cause incorrect splitting. The SLM handles material inference.
        materials.insert("linen",         "linen");
        materials.insert("silk",          "silk");
        materials.insert("wool",          "wool");
        materials.insert("nylon",         "nylon");
        materials.insert("spandex",       "spandex");
        materials.insert("lycra",         "spandex");
        materials.insert("vải bông",      "cotton");
        materials.insert("vải polyester", "polyester");
        materials.insert("lụa",           "silk");
        materials.insert("len",           "wool");
        materials.insert("inox",          "stainless_steel");
        materials.insert("không gỉ",      "stainless_steel");
        materials.insert("thép",          "steel");
        materials.insert("nhôm",          "aluminum");
        materials.insert("đồng",          "copper");
        materials.insert("nhựa",          "plastic");
        materials.insert("cao su",        "rubber");
        materials.insert("activated carbon", "activated_carbon");
        materials.insert("carbon",         "carbon_steel");
        materials.insert("nitrile",        "nitrile");
        materials.insert("steel",          "steel");
        materials.insert("stainless",     "stainless_steel");
        materials.insert("aluminum",      "aluminum");
        materials.insert("aluminium",     "aluminum");
        materials.insert("copper",        "copper");
        materials.insert("plastic",       "plastic");
        materials.insert("rubber",        "rubber");
        materials.insert("carbon steel",  "carbon_steel");
        materials.insert("cast iron",     "cast_iron");

        // ── Genders — always typed as Gender(canonical) ───────────────────────
        // These are detected directly (not via a keyword) — no GenderKwd needed.
        let mut genders = HashMap::new();
        genders.insert("nam",     "male");
        genders.insert("nữ",      "female");
        genders.insert("unisex",  "unisex");
        genders.insert("male",    "male");
        genders.insert("female",  "female");
        genders.insert("men",     "male");
        genders.insert("women",   "female");
        genders.insert("men's",   "male");
        genders.insert("women's", "female");

        // ── Keywords ─────────────────────────────────────────────────────────
        let size_keywords  = vec!["kích thước","kích cỡ","cỡ","size","taille"];
        let color_keywords = vec!["màu","màu sắc","color","colours","colors","colour","couleur"];
        let mat_keywords   = vec!["chất liệu","vật liệu","material","matière"];
        // Need keywords — consumed by lexer, never emitted (longest-match wins)
        let need_keywords  = vec![
            "tôi cần mua","tôi cần","tôi muốn mua","tôi muốn",
            "muốn mua","muốn","cần mua","cần","mua",
            "khoảng",
            "từ",                           // range-start: "từ 100 đến 200"
            "i'm looking for","i am looking for","looking for",
            "i'd like","i need","i want",
            "need","want","sourcing",
        ];
        // Proper compound nouns checked before gender/attribute matching.
        // These contain words that are also in the attribute ontology (e.g. "nam" = male)
        // but here are part of a fixed proper noun and must be emitted as Sytg.
        let proper_compounds = vec![
            // Geographic proper nouns
            "việt nam",          // country name — "nam" would otherwise → Gender(male)
            "hồ chí minh",       // major city / southern hub
            "hà nội",            // capital
            "đà nẵng",           // central hub
            "hải phòng",         // principal northern port
            "cần thơ",           // mekong delta hub
            "quy nhơn",          // central port
            "nha trang",         // south-central port
            "vũng tàu",          // oil & gas port, south
            "cái mép",           // deep-water container port
            "hải dương",         // northern industrial province
            "bình dương",        // southern industrial province
            "đồng nai",          // southern industrial province — "đồng" → copper otherwise
            // GI / compound product names whose sub-words collide with attribute ontology
            "trà xanh",          // green tea — "xanh" → Color("blue") otherwise
            "dừa bến tre",       // Bến Tre coconut (GI product)
            "thanh long",        // dragon fruit — "long" is neutral but compound matters
            "nước mắm",          // fish sauce — standalone product
            "bột mì",            // wheat flour
            "đồng phục",         // uniform/workwear — "đồng" → copper otherwise
        ];

        // Geographic origins — EN mode only, checked before Sytg fallthrough.
        // Emitted as Tag::Provenance → Group.ambiguous for SLM resolution.
        // Multi-word entries win via longest-match in map_prefix.
        let mut geo_origins: HashMap<&'static str, &'static str> = HashMap::new();
        // Asia-Pacific
        geo_origins.insert("china",            "China");
        geo_origins.insert("india",            "India");
        geo_origins.insert("japan",            "Japan");
        geo_origins.insert("south korea",      "South Korea");
        geo_origins.insert("taiwan",           "Taiwan");
        geo_origins.insert("hong kong",        "Hong Kong");
        geo_origins.insert("vietnam",          "Vietnam");
        geo_origins.insert("thailand",         "Thailand");
        geo_origins.insert("malaysia",         "Malaysia");
        geo_origins.insert("indonesia",        "Indonesia");
        geo_origins.insert("philippines",      "Philippines");
        geo_origins.insert("singapore",        "Singapore");
        geo_origins.insert("myanmar",          "Myanmar");
        geo_origins.insert("cambodia",         "Cambodia");
        geo_origins.insert("bangladesh",       "Bangladesh");
        geo_origins.insert("pakistan",         "Pakistan");
        geo_origins.insert("sri lanka",        "Sri Lanka");
        geo_origins.insert("australia",        "Australia");
        geo_origins.insert("new zealand",      "New Zealand");
        // Europe
        geo_origins.insert("germany",          "Germany");
        geo_origins.insert("france",           "France");
        geo_origins.insert("italy",            "Italy");
        geo_origins.insert("spain",            "Spain");
        geo_origins.insert("portugal",         "Portugal");
        geo_origins.insert("netherlands",      "Netherlands");
        geo_origins.insert("belgium",          "Belgium");
        geo_origins.insert("poland",           "Poland");
        geo_origins.insert("czech republic",   "Czech Republic");
        geo_origins.insert("austria",          "Austria");
        geo_origins.insert("switzerland",      "Switzerland");
        geo_origins.insert("sweden",           "Sweden");
        geo_origins.insert("denmark",          "Denmark");
        geo_origins.insert("finland",          "Finland");
        geo_origins.insert("norway",           "Norway");
        geo_origins.insert("uk",               "UK");
        geo_origins.insert("united kingdom",   "UK");
        geo_origins.insert("ukraine",          "Ukraine");
        geo_origins.insert("turkey",           "Turkey");
        geo_origins.insert("russia",           "Russia");
        geo_origins.insert("greece",           "Greece");
        geo_origins.insert("romania",          "Romania");
        geo_origins.insert("hungary",          "Hungary");
        // Americas
        geo_origins.insert("usa",              "USA");
        geo_origins.insert("united states",    "USA");
        geo_origins.insert("canada",           "Canada");
        geo_origins.insert("mexico",           "Mexico");
        geo_origins.insert("brazil",           "Brazil");
        geo_origins.insert("argentina",        "Argentina");
        geo_origins.insert("colombia",         "Colombia");
        geo_origins.insert("chile",            "Chile");
        geo_origins.insert("peru",             "Peru");
        geo_origins.insert("ecuador",          "Ecuador");
        geo_origins.insert("bolivia",          "Bolivia");
        // Middle East & Africa
        geo_origins.insert("saudi arabia",     "Saudi Arabia");
        geo_origins.insert("uae",              "UAE");
        geo_origins.insert("united arab emirates", "UAE");
        geo_origins.insert("israel",           "Israel");
        geo_origins.insert("iran",             "Iran");
        geo_origins.insert("egypt",            "Egypt");
        geo_origins.insert("south africa",     "South Africa");
        geo_origins.insert("nigeria",          "Nigeria");
        geo_origins.insert("ethiopia",         "Ethiopia");
        geo_origins.insert("kenya",            "Kenya");
        geo_origins.insert("ghana",            "Ghana");
        geo_origins.insert("morocco",          "Morocco");
        geo_origins.insert("tanzania",         "Tanzania");
        geo_origins.insert("ivory coast",      "Ivory Coast");
        // Central Asia & Caucasus
        geo_origins.insert("kazakhstan",       "Kazakhstan");
        geo_origins.insert("uzbekistan",       "Uzbekistan");

        let coord_and      = vec!["và","và/hoặc","and","&","+"];
        let coord_or       = vec!["hoặc","or"];
        let coord_range    = vec!["đến"]; // range endpoint: "S đến XL", "100 đến 200"

        // ── Standards ────────────────────────────────────────────────────────
        let standards = vec![
            "astm","iso","din","en ","jis","gb ","bs ","ansi","asme",
            "haccp","ce ","ul ","vde","iec","nf ","api ","aws ",
        ];

        // ── Units ─────────────────────────────────────────────────────────────
        // IMPORTANT: no standalone "l" (ambiguous with 316L) or "m" (ambiguous with M=medium)
        let mut units: HashMap<&'static str, Unit> = HashMap::new();
        units.insert("kg",      Unit::Kg);
        units.insert("kgs",     Unit::Kg);
        units.insert("kilogram",Unit::Kg);
        units.insert("g",       Unit::G);
        units.insert("gram",    Unit::G);
        units.insert("tonne",   Unit::Tonne);
        units.insert("ton",     Unit::Tonne);
        units.insert("tấn",     Unit::Tonne);
        units.insert("đôi",     Unit::Doi);
        units.insert("pcs",     Unit::Pcs);
        units.insert("pc",      Unit::Pcs);
        units.insert("pieces",  Unit::Pcs);
        units.insert("piece",   Unit::Pcs);
        units.insert("cái",     Unit::Cai);
        units.insert("chiếc",   Unit::Cai);
        units.insert("bộ",      Unit::Bo);
        units.insert("sets",    Unit::Sets);
        units.insert("set",     Unit::Sets);
        units.insert("cuộn",    Unit::Cuon);
        units.insert("rolls",   Unit::Rolls);
        units.insert("roll",    Unit::Rolls);
        units.insert("tấm",     Unit::Tam);
        units.insert("sheets",  Unit::Sheets);
        units.insert("sheet",   Unit::Sheets);
        units.insert("m²",      Unit::M2);
        units.insert("m2",      Unit::M2);
        units.insert("cm",      Unit::Cm);
        units.insert("mm",      Unit::Mm);
        units.insert("lít",     Unit::L);
        units.insert("liters",  Unit::L);
        units.insert("litres",  Unit::L);
        units.insert("ml",      Unit::Ml);
        units.insert("dozens",    Unit::Dozens);
        units.insert("dozen",     Unit::Dozens);
        units.insert("tons",      Unit::Tonne);
        units.insert("drums",     Unit::Drums);
        units.insert("drum",      Unit::Drums);
        units.insert("pallets",   Unit::Pallets);
        units.insert("pallet",    Unit::Pallets);
        units.insert("cartons",   Unit::Cartons);
        units.insert("carton",    Unit::Cartons);
        units.insert("boxes",     Unit::Box);
        units.insert("box",       Unit::Box);
        units.insert("reels",     Unit::Reels);
        units.insert("reel",      Unit::Reels);
        units.insert("hundreds",  Unit::Hundreds);
        units.insert("hundred",   Unit::Hundreds);
        units.insert("thousands", Unit::Thousands);
        units.insert("thousand",  Unit::Thousands);
        units.insert("meters",    Unit::Meters);
        units.insert("meter",     Unit::Meters);
        units.insert("metres",    Unit::Meters);
        units.insert("metre",     Unit::Meters);
        units.insert("20ft fcl",  Unit::Fcl);
        units.insert("fcl",       Unit::Fcl);

        Self {
            colors, sizes, materials, genders,
            standards,
            size_keywords, color_keywords, mat_keywords,
            need_keywords, proper_compounds, geo_origins,
            coord_and, coord_or, coord_range,
            units,
        }
    }
}
