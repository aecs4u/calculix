/// Comprehensive validation test for all example INP files
/// This test parses all example files to ensure compatibility
use ccx_inp::Deck;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

fn find_inp_files(dir: &Path) -> Vec<PathBuf> {
    let mut inp_files = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                inp_files.extend(find_inp_files(&path));
            } else if path.extension().and_then(|s| s.to_str()) == Some("inp") {
                inp_files.push(path);
            }
        }
    }

    inp_files
}

fn categorize_example(path: &Path) -> &'static str {
    let path_str = path.to_string_lossy().to_lowercase();

    if path_str.contains("contact") || path_str.contains("cont/") {
        "Contact"
    } else if path_str.contains("dynamic") {
        "Dynamics"
    } else if path_str.contains("linear") {
        "Linear"
    } else if path_str.contains("nonlinear") {
        "NonLinear"
    } else if path_str.contains("thermal") || path_str.contains("heat") {
        "Thermal"
    } else if path_str.contains("frequency") || path_str.contains("modal") {
        "Modal"
    } else if path_str.contains("buckle") || path_str.contains("buckling") {
        "Buckling"
    } else if path_str.contains("beam") {
        "Beam"
    } else if path_str.contains("shell") {
        "Shell"
    } else if path_str.contains("solid") || path_str.contains("3d") {
        "Solid"
    } else if path_str.contains("plate") {
        "Plate"
    } else if path_str.contains("disk") || path_str.contains("axisym") {
        "Axisymmetric"
    } else if path_str.contains("truss") {
        "Truss"
    } else {
        "Other"
    }
}

#[test]
fn test_parse_all_examples() {
    let examples_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("examples");

    if !examples_dir.exists() {
        eprintln!("Examples directory not found: {:?}", examples_dir);
        return;
    }

    let inp_files = find_inp_files(&examples_dir);
    println!("\nFound {} INP files in examples directory", inp_files.len());

    let mut parse_success = 0;
    let mut parse_fail = 0;
    let mut categories: HashMap<&'static str, (usize, usize)> = HashMap::new();
    let mut failed_files: Vec<(PathBuf, String)> = Vec::new();

    for inp_file in &inp_files {
        let category = categorize_example(inp_file);
        let result = Deck::parse_file(inp_file);

        let (success, fail) = categories.entry(category).or_insert((0, 0));

        match result {
            Ok(_) => {
                parse_success += 1;
                *success += 1;
            }
            Err(e) => {
                parse_fail += 1;
                *fail += 1;
                // Store first 10 failures for reporting
                if failed_files.len() < 10 {
                    failed_files.push((inp_file.clone(), e.to_string()));
                }
            }
        }
    }

    // Print summary
    println!("\n=== Example Files Validation Summary ===");
    println!("Total files: {}", inp_files.len());
    println!("Successfully parsed: {} ({:.1}%)",
             parse_success,
             (parse_success as f64 / inp_files.len() as f64) * 100.0);
    println!("Failed to parse: {} ({:.1}%)",
             parse_fail,
             (parse_fail as f64 / inp_files.len() as f64) * 100.0);

    println!("\n=== Breakdown by Category ===");
    let mut sorted_categories: Vec<_> = categories.iter().collect();
    sorted_categories.sort_by_key(|(name, _)| *name);

    for (category, (success, fail)) in sorted_categories {
        let total = success + fail;
        println!("{:15} {:4} files ({:3} ok, {:3} fail, {:5.1}% success)",
                 category, total, success, fail,
                 (*success as f64 / total as f64) * 100.0);
    }

    if !failed_files.is_empty() {
        println!("\n=== Sample Parse Failures (first 10) ===");
        for (file, error) in &failed_files {
            let rel_path = file.strip_prefix(&examples_dir).unwrap_or(file);
            println!("  - {:?}: {}", rel_path, error.lines().next().unwrap_or("unknown error"));
        }
    }

    // Test passes if we successfully parse at least 90% of files
    let success_rate = (parse_success as f64 / inp_files.len() as f64) * 100.0;
    assert!(
        success_rate >= 90.0,
        "Parse success rate ({:.1}%) is below 90% threshold",
        success_rate
    );
}

#[test]
fn test_truss_examples_in_detail() {
    let examples_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("examples");

    // Test our known truss examples
    let truss_files = vec![
        examples_dir.join("simple_truss.inp"),
        examples_dir.join("three_bar_truss.inp"),
    ];

    for truss_file in truss_files {
        if !truss_file.exists() {
            continue;
        }

        let deck = Deck::parse_file(&truss_file)
            .unwrap_or_else(|e| panic!("Failed to parse {:?}: {}", truss_file, e));

        println!("\nValidated: {:?}", truss_file.file_name().unwrap());
        println!("  Cards: {}", deck.cards.len());

        // Check for expected keywords
        let keywords: Vec<&str> = deck.cards.iter()
            .map(|c| c.keyword.as_str())
            .collect();

        assert!(keywords.contains(&"NODE"), "Should have NODE keyword");
        assert!(keywords.contains(&"ELEMENT"), "Should have ELEMENT keyword");
    }
}

#[test]
fn test_categorization() {
    // Test the categorization logic
    assert_eq!(categorize_example(Path::new("examples/Contact/test.inp")), "Contact");
    assert_eq!(categorize_example(Path::new("examples/Dynamics/modal.inp")), "Dynamics");
    assert_eq!(categorize_example(Path::new("examples/Linear/beam.inp")), "Linear");
    assert_eq!(categorize_example(Path::new("examples/simple_truss.inp")), "Truss");
    assert_eq!(categorize_example(Path::new("examples/thermal/heat.inp")), "Thermal");
}

#[test]
fn test_examples_statistics() {
    let examples_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("examples");

    if !examples_dir.exists() {
        return;
    }

    let inp_files = find_inp_files(&examples_dir);

    // Gather statistics
    let mut total_cards = 0;
    let mut files_with_nodes = 0;
    let mut files_with_elements = 0;

    for inp_file in inp_files.iter().take(50) {  // Sample first 50 for performance
        if let Ok(deck) = Deck::parse_file(inp_file) {
            total_cards += deck.cards.len();

            for card in &deck.cards {
                if card.keyword == "NODE" {
                    files_with_nodes += 1;
                } else if card.keyword == "ELEMENT" {
                    files_with_elements += 1;
                }
            }
        }
    }

    println!("\n=== Example Files Statistics (sample of 50) ===");
    println!("Total cards: {}", total_cards);
    println!("Files with nodes: {}", files_with_nodes);
    println!("Files with elements: {}", files_with_elements);
    println!("Average cards per file: {:.1}", total_cards as f64 / 50.0);
}
