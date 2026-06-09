use std::io::{self, Write};
use ds_qlp_rfq::{parse, ParseResult};

fn main() {
    println!("============================================================");
    println!("   DS-RFQ — RFQ Document Parser + DS-QLP Test Bench        ");
    println!("============================================================");
    println!("Paste a query or full RFQ document. Type 'exit' to quit.\n");

    loop {
        print!("🚀 Input > ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() { continue; }
        if input.to_lowercase() == "exit" { println!("Goodbye!"); break; }

        println!("\n--- [1. ORIGINAL INPUT] ---");
        println!("\"{}\"", input);

        match parse(input) {
            ParseResult::SimpleQuery(result) => {
                println!("\n[MODE: simple query → DS-QLP]\n");

                println!("--- [2. GENERATED SLM DSL] ---");
                if result.slm_dsl.is_empty() {
                    println!("(Empty DSL: No groups detected)");
                } else {
                    println!("\x1b[33m{}\x1b[0m", result.slm_dsl);
                }

                println!("--- [3. SEARCH STRINGS] ---");
                for (i, s) in result.to_search_strings().iter().enumerate() {
                    println!("  ↳ S{} : \"\x1b[32m{}\x1b[0m\"", i + 1, s);
                }
            }

            ParseResult::Rfq(rfq) => {
                println!("\n[MODE: RFQ document → {} items detected]\n", rfq.items.len());

                println!("--- [2. GENERATED SLM DSL] ---");
                println!("\x1b[33m{}\x1b[0m", rfq.slm_dsl);

                println!("--- [3. SEARCH STRINGS PER ITEM] ---");
                for item in &rfq.items {
                    println!("  Item {} (qty:{:?} uom:{:?} price:{:?} origin:{:?})",
                        item.index, item.qty, item.uom, item.price, item.origin);
                    for (i, s) in item.search_strs.iter().enumerate() {
                        println!("    ↳ S{} : \"\x1b[32m{}\x1b[0m\"", i + 1, s);
                    }
                }
            }
        }

        println!("============================================================\n");
    }
}
