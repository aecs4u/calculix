//! Integration tests using real CalculiX input fixtures.

use ccx_inp::Deck;
use ccx_solver::{AnalysisPipeline, AnalysisType};
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../tests/fixtures/solver");
    path.push(name);
    path
}

#[test]
fn test_beamcr4_fixture() {
    let path = fixture_path("beamcr4.inp");
    let deck = Deck::parse_file_with_includes(&path).expect("Failed to parse beamcr4.inp");

    let pipeline = AnalysisPipeline::detect_from_deck(&deck);
    // beamcr4 uses *VISCO keyword for viscoplastic analysis
    assert_eq!(pipeline.config().analysis_type, AnalysisType::Visco);

    let results = pipeline.run(&deck).expect("Analysis should succeed");
    assert!(results.success);
    assert_eq!(results.num_dofs, 20 * 3); // 20 nodes * 3 DOFs
    assert!(results.message.contains("20 nodes"));
    assert!(results.message.contains("1 elements")); // beamcr4 has 1 C3D20 element
}

#[test]
fn test_membrane2_fixture() {
    let path = fixture_path("membrane2.inp");
    let deck = Deck::parse_file_with_includes(&path).expect("Failed to parse membrane2.inp");

    let pipeline = AnalysisPipeline::detect_from_deck(&deck);
    let results = pipeline.run(&deck).expect("Analysis should succeed");
    assert!(results.success);
    assert!(results.num_dofs > 0);
}

#[test]
fn test_beammix_fixture() {
    let path = fixture_path("beammix.inp");
    let deck = Deck::parse_file_with_includes(&path).expect("Failed to parse beammix.inp");

    let pipeline = AnalysisPipeline::detect_from_deck(&deck);
    let results = pipeline.run(&deck).expect("Analysis should succeed");
    assert!(results.success);
    assert!(results.num_dofs > 0);
}

#[test]
fn test_coupling1_fixture() {
    let path = fixture_path("coupling1.inp");
    let deck = Deck::parse_file_with_includes(&path).expect("Failed to parse coupling1.inp");

    let pipeline = AnalysisPipeline::detect_from_deck(&deck);
    let results = pipeline.run(&deck).expect("Analysis should succeed");
    assert!(results.success);
}

#[test]
fn test_multiple_fixtures_parse_and_initialize() {
    // Test that we can successfully parse and initialize analysis for multiple fixtures
    let fixtures = vec![
        "beamcr4.inp",
        "membrane2.inp",
        "beammix.inp",
        "coupling1.inp",
        "channel6.inp",
    ];

    let mut success_count = 0;
    for fixture_name in fixtures {
        let path = fixture_path(fixture_name);
        if let Ok(deck) = Deck::parse_file_with_includes(&path) {
            let pipeline = AnalysisPipeline::detect_from_deck(&deck);
            if let Ok(results) = pipeline.run(&deck)
                && results.success
            {
                success_count += 1;
            }
        }
    }

    // Require at least 4 out of 5 fixtures to work
    assert!(
        success_count >= 4,
        "Only {} out of 5 fixtures initialized successfully",
        success_count
    );
}
