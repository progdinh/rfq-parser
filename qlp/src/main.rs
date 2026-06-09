use std::io::{self, Write};
// Import the public API from the qlp crate
use qlp::parse_query;

fn main() {
    println!("============================================================");
    println!("        qlp — Query Lexer-Parser Test Bench                ");
    println!("============================================================");
    println!("Enter a purchase query (EN/VI/Mixed). Type 'exit' to quit.\n");

    loop {
        // Render the prompt symbol in the terminal
        print!("🚀 Query > ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        // Skip processing if the input is empty
        if input.is_empty() {
            continue;
        }

        // Clean exit command evaluation
        if input.to_lowercase() == "exit" {
            println!("Goodbye!");
            break;
        }

        println!("\n--- [1. ORIGINAL INPUT] ---");
        println!("\"{}\"", input);

        // ── Execute the query parser pipeline ───────────────────────
        let result = parse_query(input);

        println!("\n--- [2. GENERATED SLM DSL] ---");
        if result.slm_dsl.is_empty() {
            println!("(Empty DSL: No groups or chunks detected)");
        } else {
            // Render the DSL output string using yellow ANSI escape characters (\x1b[33m)
            println!("\x1b[33m{}\x1b[0m", result.slm_dsl);
        }

        println!("\n--- [3. BACKEND EMBEDDING / BM25 FALLBACK STRINGS] ---");
        let search_strings = result.to_search_strings();
        if search_strings.is_empty() {
            println!("(No fallback search strings generated)");
        } else {
            for (i, s) in search_strings.iter().enumerate() {
                // Render search strings using green ANSI escape characters (\x1b[32m)
                println!("  ↳ S{} : \"\x1b[32m{}\x1b[0m\"", i + 1, s);
            }
        }

        println!("\n--- [4. DEBUG INTERNAL STRUCTURE] ---");
        println!("Total semantic groups isolated: {}", result.groups.len());
        for (idx, group) in result.groups.iter().enumerate() {
            println!(
                "  Group {} -> Qty: {:?}, Max Qty: {:?}, Uom: {:?}, Chunks: {}", 
                idx + 1, 
                group.qty, 
                group.qty_max,
                group.uom, 
                group.chunks.len()
            );
        }
        println!("============================================================\n");
    }
}