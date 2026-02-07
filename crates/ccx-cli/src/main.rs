use std::process::ExitCode;

use ccx_inp::Deck;
use ccx_model::ModelSummary;

fn usage() {
    eprintln!("usage: ccx-cli analyze <input.inp>");
}

fn print_summary(summary: &ModelSummary) {
    println!("total_cards: {}", summary.total_cards);
    println!("total_data_lines: {}", summary.total_data_lines);
    println!("node_rows: {}", summary.node_rows);
    println!("element_rows: {}", summary.element_rows);
    println!("material_defs: {}", summary.material_defs);
    println!("has_step: {}", summary.has_step);
    println!("has_static: {}", summary.has_static);
    println!("has_dynamic: {}", summary.has_dynamic);
    println!("has_frequency: {}", summary.has_frequency);
    println!("has_heat_transfer: {}", summary.has_heat_transfer);
    if !summary.include_files.is_empty() {
        println!("include_files: {}", summary.include_files.join(", "));
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 || args[1] != "analyze" {
        usage();
        return ExitCode::from(2);
    }

    let deck = match Deck::parse_file(&args[2]) {
        Ok(deck) => deck,
        Err(err) => {
            eprintln!("parse error: {err}");
            return ExitCode::from(1);
        }
    };
    let summary = ModelSummary::from_deck(&deck);
    print_summary(&summary);
    ExitCode::SUCCESS
}

