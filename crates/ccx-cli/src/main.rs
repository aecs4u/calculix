use std::path::{Path, PathBuf};
use std::process::ExitCode;

use calculix_gui::{LegacyGuiLanguage, PORTED_GUI_UNITS, gui_migration_report, legacy_gui_units};
use ccx_model::ModelSummary;
use ccx_solver::{LegacyLanguage, PORTED_UNITS, legacy_units, migration_report};

fn usage() {
    eprintln!("usage:");
    eprintln!("  ccx-cli analyze <input.inp>");
    eprintln!("  ccx-cli analyze-fixtures <fixtures_dir>");
    eprintln!("  ccx-cli postprocess <input.dat>");
    eprintln!("  ccx-cli frd2vtk <input.frd> <output.vtk>");
    eprintln!("  ccx-cli frd2vtu [--binary] <input.frd> <output.vtu>");
    eprintln!("  ccx-cli migration-report");
    eprintln!("  ccx-cli gui-migration-report");
    eprintln!("  ccx-cli --help");
    eprintln!("  ccx-cli --version");
    eprintln!();
    eprintln!("examples:");
    eprintln!("  ccx-cli analyze tests/fixtures/solver/ax6.inp");
    eprintln!("  ccx-cli analyze-fixtures tests/fixtures/solver");
    eprintln!("  ccx-cli postprocess results.dat");
    eprintln!("  ccx-cli frd2vtk job.frd job.vtk");
    eprintln!("  ccx-cli frd2vtu job.frd job.vtu");
    eprintln!("  ccx-cli frd2vtu --binary job.frd job.vtu");
    eprintln!("  ccx-cli migration-report");
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
    println!("unique_keywords: {}", summary.keyword_counts.len());
}

fn language_label(language: LegacyLanguage) -> &'static str {
    match language {
        LegacyLanguage::C => "C",
        LegacyLanguage::Fortran => "Fortran",
        LegacyLanguage::Header => "Header",
        LegacyLanguage::Other => "Other",
    }
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
    for (language, count) in report.by_language {
        println!("language_{}: {}", language_label(language), count);
    }

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

fn gui_language_label(language: LegacyGuiLanguage) -> &'static str {
    match language {
        LegacyGuiLanguage::C => "C",
        LegacyGuiLanguage::Cpp => "Cpp",
        LegacyGuiLanguage::Header => "Header",
        LegacyGuiLanguage::Other => "Other",
    }
}

fn print_gui_migration_report() {
    let report = gui_migration_report();
    println!("legacy_gui_units_total: {}", report.total_units);
    println!("ported_gui_units: {}", report.ported_units);
    println!("pending_gui_units: {}", report.pending_units);
    for (language, count) in report.by_language {
        println!("gui_language_{}: {}", gui_language_label(language), count);
    }

    if !PORTED_GUI_UNITS.is_empty() {
        println!("ported_gui_list: {}", PORTED_GUI_UNITS.join(", "));
    }

    let pending_preview: Vec<&str> = legacy_gui_units()
        .iter()
        .map(|u| u.legacy_rel_path)
        .filter(|path| !PORTED_GUI_UNITS.iter().any(|ported| ported == path))
        .take(8)
        .collect();
    if !pending_preview.is_empty() {
        println!("pending_gui_preview: {}", pending_preview.join(", "));
    }
}

fn analyze_file(path: &Path) -> Result<ModelSummary, String> {
    let deck = ccx_inp::Deck::parse_file_with_includes(path)
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

fn postprocess_dat_file(path: &Path) -> Result<(), String> {
    use ccx_solver::{read_dat_file, process_integration_points, compute_statistics, write_results};

    // Validate file extension
    if !path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("dat")) {
        return Err("File must have .dat extension".to_string());
    }

    // Read element variable output from .dat file
    println!("Reading element variable output from: {}", path.display());
    let data = read_dat_file(path)?;
    println!("Found {} integration points", data.len());

    // Process data and compute Mises stress, effective strain, PEEQ
    let results = process_integration_points(&data);

    // Compute statistics
    let stats = compute_statistics(&results);

    // Print summary
    println!("\nStatistics:");
    println!("  Mises stress:       min={:.4e}  max={:.4e}  mean={:.4e}",
             stats.mises_min, stats.mises_max, stats.mises_mean);
    println!("  Effective strain:   min={:.4e}  max={:.4e}  mean={:.4e}",
             stats.eeq_min, stats.eeq_max, stats.eeq_mean);
    println!("  Plastic strain:     min={:.4e}  max={:.4e}  mean={:.4e}",
             stats.peeq_min, stats.peeq_max, stats.peeq_mean);

    // Write results to file
    write_results(path, &results, &stats)?;

    Ok(())
}

fn frd2vtk_file(input_path: &Path, output_path: &Path) -> Result<(), String> {
    use ccx_io::{FrdFile, VtkWriter};

    // Validate file extensions
    if !input_path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("frd")) {
        return Err("Input file must have .frd extension".to_string());
    }
    if !output_path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("vtk")) {
        return Err("Output file must have .vtk extension".to_string());
    }

    // Read FRD file
    println!("Reading FRD file: {}", input_path.display());
    let frd = FrdFile::from_file(input_path)
        .map_err(|err| format!("Failed to read FRD file: {}", err))?;

    println!("  Nodes: {}", frd.nodes.len());
    println!("  Elements: {}", frd.elements.len());
    println!("  Result blocks: {}", frd.result_blocks.len());

    // Write VTK file
    println!("Writing VTK file: {}", output_path.display());
    let writer = VtkWriter::new(&frd);
    writer.write_vtk(output_path)
        .map_err(|err| format!("Failed to write VTK file: {}", err))?;

    println!("Conversion complete!");
    Ok(())
}

fn frd2vtu_file(input_path: &Path, output_path: &Path, binary: bool) -> Result<(), String> {
    use ccx_io::{FrdFile, VtkWriter, VtkFormat};

    // Validate file extensions
    if !input_path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("frd")) {
        return Err("Input file must have .frd extension".to_string());
    }
    if !output_path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("vtu")) {
        return Err("Output file must have .vtu extension".to_string());
    }

    // Read FRD file
    println!("Reading FRD file: {}", input_path.display());
    let frd = FrdFile::from_file(input_path)
        .map_err(|err| format!("Failed to read FRD file: {}", err))?;

    println!("  Nodes: {}", frd.nodes.len());
    println!("  Elements: {}", frd.elements.len());
    println!("  Result blocks: {}", frd.result_blocks.len());

    // Write VTU file
    let format = if binary { VtkFormat::Binary } else { VtkFormat::Ascii };
    println!("Writing VTU file ({}): {}",
             if binary { "binary" } else { "ASCII" },
             output_path.display());

    let writer = VtkWriter::new(&frd);
    writer.write_vtu(output_path, format)
        .map_err(|err| format!("Failed to write VTU file: {}", err))?;

    println!("Conversion complete!");
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("help") | Some("-h") | Some("--help") => {
            usage();
            ExitCode::SUCCESS
        }
        Some("--version") | Some("-V") => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some("analyze") => {
            if args.len() != 3 {
                usage();
                return ExitCode::from(2);
            }

            let path = Path::new(&args[2]);
            let summary = match analyze_file(path) {
                Ok(summary) => summary,
                Err(err) => {
                    eprintln!("parse error: {err}");
                    return ExitCode::from(1);
                }
            };
            print_summary(&summary);
            ExitCode::SUCCESS
        }
        Some("analyze-fixtures") => {
            if args.len() != 3 {
                usage();
                return ExitCode::from(2);
            }
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
        Some("postprocess") => {
            if args.len() != 3 {
                usage();
                return ExitCode::from(2);
            }
            let path = Path::new(&args[2]);
            match postprocess_dat_file(path) {
                Ok(()) => ExitCode::SUCCESS,
                Err(err) => {
                    eprintln!("postprocess error: {err}");
                    ExitCode::from(1)
                }
            }
        }
        Some("frd2vtk") => {
            if args.len() != 4 {
                usage();
                return ExitCode::from(2);
            }
            let input_path = Path::new(&args[2]);
            let output_path = Path::new(&args[3]);
            match frd2vtk_file(input_path, output_path) {
                Ok(()) => ExitCode::SUCCESS,
                Err(err) => {
                    eprintln!("frd2vtk error: {err}");
                    ExitCode::from(1)
                }
            }
        }
        Some("frd2vtu") => {
            // Handle optional --binary flag
            let (binary, input_idx, output_idx) = if args.get(2).map(String::as_str) == Some("--binary") {
                if args.len() != 5 {
                    usage();
                    return ExitCode::from(2);
                }
                (true, 3, 4)
            } else {
                if args.len() != 4 {
                    usage();
                    return ExitCode::from(2);
                }
                (false, 2, 3)
            };

            let input_path = Path::new(&args[input_idx]);
            let output_path = Path::new(&args[output_idx]);
            match frd2vtu_file(input_path, output_path, binary) {
                Ok(()) => ExitCode::SUCCESS,
                Err(err) => {
                    eprintln!("frd2vtu error: {err}");
                    ExitCode::from(1)
                }
            }
        }
        Some("migration-report") => {
            if args.len() != 2 {
                usage();
                return ExitCode::from(2);
            }
            print_migration_report();
            ExitCode::SUCCESS
        }
        Some("gui-migration-report") => {
            if args.len() != 2 {
                usage();
                return ExitCode::from(2);
            }
            print_gui_migration_report();
            ExitCode::SUCCESS
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
    fn labels_match_expected_strings() {
        assert_eq!(language_label(LegacyLanguage::C), "C");
        assert_eq!(language_label(LegacyLanguage::Fortran), "Fortran");
        assert_eq!(gui_language_label(LegacyGuiLanguage::Cpp), "Cpp");
        assert_eq!(gui_language_label(LegacyGuiLanguage::Header), "Header");
    }

    #[test]
    fn collect_inp_files_recurses_and_sorts() {
        let root = unique_temp_dir("ccx_cli_collect_inp");
        let nested = root.join("nested");
        fs::create_dir_all(&nested).expect("create temp tree");

        fs::write(root.join("b.INP"), "*NODE\n1,0,0,0\n").expect("write b");
        fs::write(root.join("ignore.txt"), "x").expect("write ignore");
        fs::write(nested.join("a.inp"), "*NODE\n1,0,0,0\n").expect("write a");

        let files = collect_inp_files(&root).expect("collect should succeed");
        let mut names: Vec<String> = files
            .iter()
            .filter_map(|p| p.file_name().and_then(|s| s.to_str()).map(str::to_string))
            .collect();
        names.sort();

        assert_eq!(names, vec!["a.inp".to_string(), "b.INP".to_string()]);
    }

    #[test]
    fn analyze_file_expands_includes() {
        let root = unique_temp_dir("ccx_cli_analyze_include");
        fs::create_dir_all(&root).expect("create temp dir");
        let deck = root.join("root.inp");
        let inc = root.join("mesh.inc");

        fs::write(
            &deck,
            "*NODE\n1,0,0,0\n*INCLUDE,INPUT=mesh.inc\n*ELEMENT\n1,1,1,1,1\n",
        )
        .expect("write root deck");
        fs::write(&inc, "*MATERIAL,NAME=STEEL\n").expect("write include");

        let summary = analyze_file(&deck).expect("analysis should parse");
        assert_eq!(summary.node_rows, 1);
        assert_eq!(summary.element_rows, 1);
        assert_eq!(summary.material_defs, 1);
        assert_eq!(summary.include_files, vec!["mesh.inc".to_string()]);
    }

    #[test]
    fn analyze_fixture_tree_counts_failures() {
        let root = unique_temp_dir("ccx_cli_fixture_tree");
        fs::create_dir_all(&root).expect("create temp dir");

        fs::write(
            root.join("ok.inp"),
            "*NODE\n1,0,0,0\n*ELEMENT\n1,1,1,1,1\n*STEP\n*STATIC\n*END STEP\n",
        )
        .expect("write ok fixture");
        fs::write(root.join("bad.inp"), "1,2,3\n*NODE\n1,0,0,0\n").expect("write bad fixture");

        let failures = analyze_fixture_tree(&root).expect("scan should succeed");
        assert_eq!(failures, 1);
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
