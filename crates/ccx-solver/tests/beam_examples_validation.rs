/// Validation tests for beam element examples
///
/// Tests parsing and basic validation of beam example INP files
/// from the examples directory.

use ccx_inp::Deck;
use std::collections::HashMap;
use std::path::PathBuf;

/// Statistics for beam example validation
#[derive(Debug, Default)]
struct BeamValidationStats {
    total_files: usize,
    parse_success: usize,
    parse_failed: usize,
    has_b31_element: usize,
    has_b32_element: usize,
    has_b32r_element: usize,
    has_beam_section: usize,
    has_material: usize,
    has_boundary: usize,
    has_loads: usize,
    errors: Vec<(String, String)>,
}

impl BeamValidationStats {
    fn new() -> Self {
        Self::default()
    }

    fn record_parse_success(&mut self) {
        self.parse_success += 1;
    }

    fn record_parse_failure(&mut self, filename: String, error: String) {
        self.parse_failed += 1;
        self.errors.push((filename, error));
    }

    fn analyze_deck(&mut self, deck: &Deck) {
        let mut has_b31 = false;
        let mut has_b32 = false;
        let mut has_b32r = false;
        let mut has_beam_section = false;
        let mut has_material = false;
        let mut has_boundary = false;
        let mut has_loads = false;

        for card in &deck.cards {
            let keyword_upper = card.keyword.to_uppercase();

            // Check for element type definitions
            if keyword_upper == "ELEMENT" {
                for param in &card.parameters {
                    if param.key.to_uppercase() == "TYPE" {
                        if let Some(ref value) = param.value {
                            let type_upper = value.to_uppercase();
                            if type_upper == "B31" {
                                has_b31 = true;
                            } else if type_upper == "B32" {
                                has_b32 = true;
                            } else if type_upper == "B32R" {
                                has_b32r = true;
                            }
                        }
                    }
                }
            }

            // Check for beam sections
            if keyword_upper == "BEAM SECTION" || keyword_upper.starts_with("BEAM SECTION,") {
                has_beam_section = true;
            }

            // Check for materials
            if keyword_upper == "MATERIAL" || keyword_upper.starts_with("MATERIAL,") {
                has_material = true;
            }

            // Check for boundary conditions
            if keyword_upper == "BOUNDARY" {
                has_boundary = true;
            }

            // Check for loads
            if keyword_upper == "CLOAD" || keyword_upper == "DLOAD" {
                has_loads = true;
            }
        }

        if has_b31 {
            self.has_b31_element += 1;
        }
        if has_b32 {
            self.has_b32_element += 1;
        }
        if has_b32r {
            self.has_b32r_element += 1;
        }
        if has_beam_section {
            self.has_beam_section += 1;
        }
        if has_material {
            self.has_material += 1;
        }
        if has_boundary {
            self.has_boundary += 1;
        }
        if has_loads {
            self.has_loads += 1;
        }
    }

    fn print_summary(&self) {
        println!("\n=== Beam Example Validation Summary ===");
        println!("Total files processed: {}", self.total_files);
        println!(
            "Parse success: {} ({:.1}%)",
            self.parse_success,
            (self.parse_success as f64 / self.total_files as f64) * 100.0
        );
        println!("Parse failed: {}", self.parse_failed);

        println!("\n--- Element Type Distribution ---");
        println!("Files with B31 elements: {}", self.has_b31_element);
        println!("Files with B32 elements: {}", self.has_b32_element);
        println!("Files with B32R elements: {}", self.has_b32r_element);

        println!("\n--- Content Analysis ---");
        println!("Files with beam sections: {}", self.has_beam_section);
        println!("Files with materials: {}", self.has_material);
        println!("Files with boundary conditions: {}", self.has_boundary);
        println!("Files with loads: {}", self.has_loads);

        if !self.errors.is_empty() {
            println!("\n--- Parse Failures ({}) ---", self.errors.len());
            for (i, (filename, error)) in self.errors.iter().enumerate().take(10) {
                println!(
                    "  {}. {}: {}",
                    i + 1,
                    filename,
                    error.lines().next().unwrap_or("Unknown error")
                );
            }
            if self.errors.len() > 10 {
                println!("  ... and {} more", self.errors.len() - 10);
            }
        }
    }
}

/// Find all INP files with beam elements
fn find_beam_examples() -> Vec<PathBuf> {
    use std::process::Command;

    // Find the workspace root by looking for Cargo.toml
    let mut current_dir = std::env::current_dir().expect("Failed to get current directory");

    // Walk up to find workspace root (contains top-level Cargo.toml)
    while !current_dir.join("Cargo.toml").exists() || !current_dir.join("examples").exists() {
        if !current_dir.pop() {
            panic!("Could not find workspace root with examples directory");
        }
    }

    let examples_dir = current_dir.join("examples");

    if !examples_dir.exists() {
        println!("Warning: examples directory not found at {}", examples_dir.display());
        return Vec::new();
    }

    let output = Command::new("find")
        .arg(&examples_dir)
        .arg("-type")
        .arg("f")
        .arg("-name")
        .arg("*.inp")
        .output()
        .expect("Failed to find example files");

    let stdout = String::from_utf8_lossy(&output.stdout);

    stdout
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| PathBuf::from(line.trim()))
        .filter(|path| {
            // Check if file contains beam-related keywords
            if let Ok(content) = std::fs::read_to_string(path) {
                let upper = content.to_uppercase();
                upper.contains("B31")
                    || upper.contains("B32")
                    || upper.contains("BEAM SECTION")
                    || upper.contains("BEAM")
            } else {
                false
            }
        })
        .collect()
}

#[test]
fn test_parse_beam_examples() {
    let beam_files = find_beam_examples();

    println!("\nFound {} beam-related example files", beam_files.len());

    let mut stats = BeamValidationStats::new();
    stats.total_files = beam_files.len();

    for path in &beam_files {
        let filename = path.file_name().unwrap().to_string_lossy().to_string();

        match Deck::parse_file(path) {
            Ok(deck) => {
                stats.record_parse_success();
                stats.analyze_deck(&deck);
            }
            Err(e) => {
                stats.record_parse_failure(filename, format!("{:?}", e));
            }
        }
    }

    stats.print_summary();

    // Require at least 90% parse success
    let success_rate = (stats.parse_success as f64 / stats.total_files as f64) * 100.0;
    assert!(
        success_rate >= 90.0,
        "Beam example parse success rate too low: {:.1}% (expected >= 90%)",
        success_rate
    );
}

#[test]
fn test_b31_element_example() {
    // Find workspace root
    let mut current_dir = std::env::current_dir().expect("Failed to get current directory");
    while !current_dir.join("Cargo.toml").exists() || !current_dir.join("examples").exists() {
        if !current_dir.pop() {
            println!("Skipping test - could not find workspace root");
            return;
        }
    }

    // Test the dedicated B31 example file
    let path = current_dir.join("examples/elements/B31.inp");

    if !path.exists() {
        println!("Skipping test - B31.inp not found");
        return;
    }

    let deck = Deck::parse_file(&path).expect("Failed to parse B31.inp");

    println!("\n=== B31 Element Example ===");
    println!("Total cards: {}", deck.cards.len());

    let mut has_node = false;
    let mut has_element = false;
    let mut has_b31 = false;
    let mut has_material = false;
    let mut has_beam_section = false;
    let mut node_count = 0;
    let mut element_count = 0;

    for card in &deck.cards {
        let keyword_upper = card.keyword.to_uppercase();

        if keyword_upper == "NODE" {
            has_node = true;
            node_count += card.data_lines.len();
        }

        if keyword_upper == "ELEMENT" {
            has_element = true;
            element_count += card.data_lines.len();

            for param in &card.parameters {
                if param.key.to_uppercase() == "TYPE" {
                    if let Some(ref value) = param.value {
                        if value.to_uppercase() == "B31" {
                            has_b31 = true;
                        }
                    }
                }
            }
        }

        if keyword_upper == "MATERIAL" || keyword_upper.starts_with("MATERIAL,") {
            has_material = true;
        }

        if keyword_upper == "BEAM SECTION" || keyword_upper.starts_with("BEAM SECTION,") {
            has_beam_section = true;
        }
    }

    println!("Nodes: {}", node_count);
    println!("Elements: {}", element_count);
    println!("Has B31: {}", has_b31);
    println!("Has material: {}", has_material);
    println!("Has beam section: {}", has_beam_section);

    // Verify structure
    assert!(has_node, "Should have NODE cards");
    assert!(has_element, "Should have ELEMENT cards");
    assert!(has_b31, "Should have B31 element type");
    assert!(has_material, "Should have MATERIAL definition");
    assert!(has_beam_section, "Should have BEAM SECTION definition");

    println!("✓ B31 example successfully parsed and validated");
}

#[test]
fn test_beam_example_categories() {
    let beam_files = find_beam_examples();

    let mut categories: HashMap<String, Vec<String>> = HashMap::new();

    for path in &beam_files {
        let filename = path.file_name().unwrap().to_string_lossy().to_string();

        // Categorize based on path
        let category = if path.to_string_lossy().contains("/elements/") {
            "Element Tests"
        } else if path.to_string_lossy().contains("/yahoo/") {
            "Yahoo Forum"
        } else if path.to_string_lossy().contains("/launcher/beams/") {
            "Launcher Beams"
        } else if path.to_string_lossy().contains("/c4w/") {
            "C4W"
        } else {
            "Other"
        };

        categories
            .entry(category.to_string())
            .or_default()
            .push(filename);
    }

    println!("\n=== Beam Example Categories ===");
    let mut sorted_categories: Vec<_> = categories.iter().collect();
    sorted_categories.sort_by_key(|(name, files)| (-(files.len() as i32), name.to_string()));

    for (category, files) in sorted_categories {
        println!("{}: {} files", category, files.len());
        if files.len() <= 5 {
            for file in files {
                println!("  - {}", file);
            }
        }
    }

    assert!(
        !categories.is_empty(),
        "Should have categorized beam examples"
    );
}

#[test]
fn test_solver_ready_b31_examples() {
    // Find B31 examples that should be solvable with our current implementation
    let beam_files = find_beam_examples();

    let mut solver_ready_count = 0;
    let mut solver_ready_files: Vec<String> = Vec::new();

    println!("\n=== Checking Solver-Ready B31 Examples ===");

    for path in &beam_files {
        if let Ok(deck) = Deck::parse_file(path) {
            let mut has_b31 = false;
            let mut has_other_beam = false;
            let mut has_material = false;
            let mut has_beam_section = false;

            for card in &deck.cards {
                let keyword_upper = card.keyword.to_uppercase();

                if keyword_upper == "ELEMENT" {
                    for param in &card.parameters {
                        if param.key.to_uppercase() == "TYPE" {
                            if let Some(ref value) = param.value {
                                let type_upper = value.to_uppercase();
                                if type_upper == "B31" {
                                    has_b31 = true;
                                } else if type_upper == "B32" || type_upper == "B32R" {
                                    has_other_beam = true;
                                }
                            }
                        }
                    }
                }

                if keyword_upper == "MATERIAL" || keyword_upper.starts_with("MATERIAL,") {
                    has_material = true;
                }

                if keyword_upper == "BEAM SECTION" || keyword_upper.starts_with("BEAM SECTION,") {
                    has_beam_section = true;
                }
            }

            // File is solver-ready if it has B31, no other beam types, and necessary definitions
            if has_b31 && !has_other_beam && has_material && has_beam_section {
                solver_ready_count += 1;
                let filename = path.file_name().unwrap().to_string_lossy().to_string();
                solver_ready_files.push(filename);
            }
        }
    }

    println!(
        "Found {} solver-ready B31 examples (pure B31, with materials and sections)",
        solver_ready_count
    );

    if !solver_ready_files.is_empty() {
        println!("\nSolver-ready files:");
        for (i, file) in solver_ready_files.iter().enumerate().take(20) {
            println!("  {}. {}", i + 1, file);
        }
        if solver_ready_files.len() > 20 {
            println!("  ... and {} more", solver_ready_files.len() - 20);
        }
    }

    // We should have at least a few solver-ready examples
    assert!(
        solver_ready_count > 0,
        "Should have at least one solver-ready B31 example"
    );
}
