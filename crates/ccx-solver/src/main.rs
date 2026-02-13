use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ccx_io::inp::Deck;
use ccx_model::ModelSummary;
use ccx_solver::{AnalysisPipeline, PORTED_UNITS, legacy_units, migration_report};

fn usage() {
    eprintln!("usage:");
    eprintln!("  ccx-solver migration-report");
    eprintln!("  ccx-solver analyze <input.inp>");
    eprintln!("  ccx-solver analyze-fixtures <fixtures_dir>");
    eprintln!("  ccx-solver solve <input.inp>");
}

fn print_migration_report() {
    let report = migration_report();
    println!("legacy_units_total: {}", report.total_units);
    println!("ported_units: {}", report.ported_units);
    println!(
        "superseded_fortran_units: {}",
        report.superseded_fortran_units
    );
    println!("pending_units: {}", report.pending_units);
    if !PORTED_UNITS.is_empty() {
        println!("ported_list: {}", PORTED_UNITS.join(", "));
    }
    let pending_preview: Vec<&str> = legacy_units()
        .iter()
        .map(|u| u.legacy_rel_path)
        .filter(|path| !PORTED_UNITS.iter().any(|ported| ported == path))
        .take(8)
        .collect();
    if !pending_preview.is_empty() {
        println!("pending_preview: {}", pending_preview.join(", "));
    }
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

fn analyze_file(path: &Path) -> Result<ModelSummary, String> {
    let deck = Deck::parse_file_with_includes(path)
        .map_err(|err| format!("{}: {}", path.display(), err))?;
    Ok(ModelSummary::from_deck(&deck))
}

fn collect_inp_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::<PathBuf>::new();
    collect_inp_files_inner(root, &mut out)?;
    out.sort();
    Ok(out)
}

fn collect_inp_files_inner(root: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = std::fs::read_dir(root)
        .map_err(|err| format!("failed to read directory {}: {err}", root.display()))?;
    for entry in entries {
        let entry = entry.map_err(|err| format!("failed to read dir entry: {err}"))?;
        let path = entry.path();
        if path.is_dir() {
            collect_inp_files_inner(&path, out)?;
            continue;
        }
        if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("inp"))
        {
            out.push(path);
        }
    }
    Ok(())
}

fn analyze_fixture_tree(root: &Path) -> Result<usize, String> {
    let files = collect_inp_files(root)?;
    if files.is_empty() {
        println!("no .inp files found in {}", root.display());
        return Ok(0);
    }

    let mut failures = 0usize;
    for path in &files {
        if let Err(err) = analyze_file(path) {
            failures += 1;
            eprintln!("parse_error: {err}");
        }
    }

    println!("fixtures_root: {}", root.display());
    println!("total_inp: {}", files.len());
    println!("parse_ok: {}", files.len().saturating_sub(failures));
    println!("parse_failed: {}", failures);
    Ok(failures)
}

fn solve_file(path: &Path) -> Result<(), String> {
    let deck = Deck::parse_file_with_includes(path)
        .map_err(|err| format!("{}: {}", path.display(), err))?;

    println!("Initializing solver for: {}", path.display());

    // Check if B32R elements need expansion
    if has_b32r_elements(&deck) {
        eprintln!("\n🔧 B32R elements detected - expansion to C3D20R required");
        eprintln!("   This feature is in development (Phase 2.2)");
        eprintln!("   Current implementation uses 1D beam theory\n");
    }

    let pipeline = AnalysisPipeline::detect_from_deck(&deck);
    println!(
        "Detected analysis type: {:?}",
        pipeline.config().analysis_type
    );

    match pipeline.run(&deck) {
        Ok(results) => {
            println!("\nAnalysis Results:");
            println!(
                "  Status: {}",
                if results.success { "SUCCESS" } else { "FAILED" }
            );
            println!("  DOFs: {}", results.num_dofs);
            println!("  Equations: {}", results.num_equations);
            println!("  Message: {}", results.message);
            Ok(())
        }
        Err(err) => Err(format!("Solver error: {}", err)),
    }
}

/// Check if deck contains B32R beam elements
fn has_b32r_elements(deck: &Deck) -> bool {
    for card in &deck.cards {
        if card.keyword.to_uppercase() == "ELEMENT" {
            for param in &card.parameters {
                if param.key.to_uppercase() == "TYPE" {
                    if let Some(ref val) = param.value {
                        let typ = val.to_uppercase();
                        if typ == "B32R" || typ == "B32" {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("migration-report") if args.len() == 2 => {
            print_migration_report();
            ExitCode::SUCCESS
        }
        Some("analyze") if args.len() == 3 => {
            let path = Path::new(&args[2]);
            match analyze_file(path) {
                Ok(summary) => {
                    print_summary(&summary);
                    ExitCode::SUCCESS
                }
                Err(err) => {
                    eprintln!("analyze_error: {err}");
                    ExitCode::from(1)
                }
            }
        }
        Some("analyze-fixtures") if args.len() == 3 => {
            let root = Path::new(&args[2]);
            match analyze_fixture_tree(root) {
                Ok(0) => ExitCode::SUCCESS,
                Ok(_) => ExitCode::from(1),
                Err(err) => {
                    eprintln!("analyze_fixtures_error: {err}");
                    ExitCode::from(1)
                }
            }
        }
        Some("solve") if args.len() == 3 => {
            let path = Path::new(&args[2]);
            match solve_file(path) {
                Ok(()) => ExitCode::SUCCESS,
                Err(err) => {
                    eprintln!("solve_error: {err}");
                    ExitCode::from(1)
                }
            }
        }
        _ => {
            usage();
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn collect_inp_files_recurses_and_sorts() {
        let root = unique_temp_dir("ccx_solver_collect_inp");
        let nested = root.join("nested");
        fs::create_dir_all(&nested).expect("create temp tree");

        fs::write(root.join("b.inp"), "*NODE\n1,0,0,0\n").expect("write b");
        fs::write(nested.join("a.inp"), "*NODE\n1,0,0,0\n").expect("write a");
        fs::write(root.join("ignore.dat"), "x").expect("write ignore");

        let files = collect_inp_files(&root).expect("collect should succeed");
        let mut names: Vec<String> = files
            .iter()
            .filter_map(|p| p.file_name().and_then(|s| s.to_str()).map(str::to_string))
            .collect();
        names.sort();

        assert_eq!(names, vec!["a.inp".to_string(), "b.inp".to_string()]);
    }

    #[test]
    fn analyze_file_expands_include_cards() {
        let root = unique_temp_dir("ccx_solver_analyze_include");
        fs::create_dir_all(&root).expect("create temp dir");
        let deck = root.join("root.inp");
        let inc = root.join("mesh.inc");

        fs::write(
            &deck,
            "*NODE\n1,0,0,0\n*INCLUDE,INPUT=mesh.inc\n*ELEMENT\n1,1,1,1,1\n",
        )
        .expect("write deck");
        fs::write(&inc, "*MATERIAL,NAME=STEEL\n").expect("write include");

        let summary = analyze_file(&deck).expect("analysis should succeed");
        assert_eq!(summary.node_rows, 1);
        assert_eq!(summary.element_rows, 1);
        assert_eq!(summary.material_defs, 1);
    }

    #[test]
    fn analyze_fixture_tree_counts_parse_failures() {
        let root = unique_temp_dir("ccx_solver_fixture_tree");
        fs::create_dir_all(&root).expect("create temp dir");

        fs::write(
            root.join("ok.inp"),
            "*NODE\n1,0,0,0\n*ELEMENT\n1,1,1,1,1\n*STEP\n*STATIC\n*END STEP\n",
        )
        .expect("write ok fixture");
        fs::write(root.join("bad.inp"), "1,2,3\n*NODE\n1,0,0,0\n").expect("write bad fixture");

        let failures = analyze_fixture_tree(&root).expect("scan should complete");
        assert_eq!(failures, 1);
    }

    #[test]
    fn solve_file_runs_for_minimal_valid_model() {
        let root = unique_temp_dir("ccx_solver_solve_ok");
        fs::create_dir_all(&root).expect("create temp dir");
        let deck = root.join("solve_ok.inp");

        fs::write(
            &deck,
            "*NODE\n1,0,0,0\n2,1,0,0\n*ELEMENT,TYPE=B31\n1,1,2\n*STEP\n*STATIC\n*END STEP\n",
        )
        .expect("write deck");

        let result = solve_file(&deck);
        assert!(result.is_ok(), "expected solve to initialize successfully");
    }

    #[test]
    fn solve_file_returns_error_when_elements_missing() {
        let root = unique_temp_dir("ccx_solver_solve_missing_elements");
        fs::create_dir_all(&root).expect("create temp dir");
        let deck = root.join("solve_bad.inp");

        fs::write(&deck, "*NODE\n1,0,0,0\n*STEP\n*STATIC\n*END STEP\n").expect("write deck");

        let err = solve_file(&deck).expect_err("solve should fail");
        assert!(err.contains("No elements defined"));
    }

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let pid = std::process::id();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}_{pid}_{nanos}"))
    }
}
