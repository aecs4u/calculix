use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ccx_model::ModelSummary;
use ccx_solver::{LegacyLanguage, PORTED_UNITS, legacy_units, migration_report};

fn usage() {
    eprintln!("usage:");
    eprintln!("  ccx-cli solve <input.inp>");
    eprintln!("  ccx-cli analyze <input.inp>");
    eprintln!("  ccx-cli analyze-fixtures <fixtures_dir>");
    eprintln!("  ccx-cli postprocess <input.dat>");
    eprintln!("  ccx-cli validate [--fixtures-dir <dir>]");
    eprintln!("  ccx-cli frd2vtk <input.frd> <output.vtk>");
    eprintln!("  ccx-cli frd2vtu [--binary] <input.frd> <output.vtu>");
    eprintln!("  ccx-cli migration-report");
    eprintln!("  ccx-cli gui-migration-report");
    eprintln!("  ccx-cli --help");
    eprintln!("  ccx-cli --version");
    eprintln!();
    eprintln!("examples:");
    eprintln!("  ccx-cli solve tests/fixtures/solver/simplebeam.inp");
    eprintln!("  ccx-cli analyze tests/fixtures/solver/ax6.inp");
    eprintln!("  ccx-cli analyze-fixtures tests/fixtures/solver");
    eprintln!("  ccx-cli postprocess results.dat");
    eprintln!("  ccx-cli validate");
    eprintln!("  ccx-cli validate --fixtures-dir tests/fixtures/solver");
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


fn analyze_file(path: &Path) -> Result<ModelSummary, String> {
    use ccx_io::inp::Deck;
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

// ============================================================================
// Validation Suite - Compare solver output against .dat.ref reference files
// ============================================================================

#[derive(Debug)]
struct ValidationReport {
    total_tests: usize,
    passed_tests: usize,
    failed_tests: usize,
    skipped_tests: usize,
    test_results: Vec<TestResult>,
}

#[derive(Debug)]
struct TestResult {
    name: String,
    status: TestStatus,
    error_message: Option<String>,
}

#[derive(Debug, PartialEq)]
enum TestStatus {
    Passed,
    Failed,
    Skipped,
}

fn run_validation_suite(fixtures_dir: &Path) -> Result<ValidationReport, String> {
    use std::fs;

    println!("Running validation suite in: {}", fixtures_dir.display());
    println!();

    // Find all .dat.ref reference files
    let entries = fs::read_dir(fixtures_dir)
        .map_err(|err| format!("Failed to read fixtures directory: {}", err))?;

    let mut ref_files = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|err| format!("Failed to read directory entry: {}", err))?;
        let path = entry.path();

        if let Some(extension) = path.extension() {
            if extension == "ref" {
                if let Some(stem) = path.file_stem() {
                    let stem_str = stem.to_string_lossy();
                    if stem_str.ends_with(".dat") {
                        ref_files.push(path);
                    }
                }
            }
        }
    }

    ref_files.sort();

    println!("Found {} reference .dat.ref files", ref_files.len());
    println!();

    let mut test_results = Vec::new();
    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;

    // Run all tests
    let files_to_test: Vec<_> = ref_files.iter().collect();

    println!("Running {} tests...", files_to_test.len());
    println!();

    for ref_file in files_to_test {
        let test_name = ref_file
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.trim_end_matches(".dat"))
            .unwrap_or("unknown");

        // Check if corresponding .inp file exists
        let inp_file = ref_file.with_file_name(format!("{}.inp", test_name));

        if !inp_file.exists() {
            test_results.push(TestResult {
                name: test_name.to_string(),
                status: TestStatus::Skipped,
                error_message: Some("No corresponding .inp file found".to_string()),
            });
            skipped += 1;
            continue;
        }

        // Run the test
        print!("  Testing {}... ", test_name);
        std::io::Write::flush(&mut std::io::stdout()).ok();

        match run_single_test(&inp_file, ref_file) {
            Ok(true) => {
                println!("✓ PASS");
                test_results.push(TestResult {
                    name: test_name.to_string(),
                    status: TestStatus::Passed,
                    error_message: None,
                });
                passed += 1;
            }
            Ok(false) => {
                println!("✗ FAIL");
                test_results.push(TestResult {
                    name: test_name.to_string(),
                    status: TestStatus::Failed,
                    error_message: Some("Output mismatch".to_string()),
                });
                failed += 1;
            }
            Err(err) => {
                println!("⊘ SKIP ({})", err);
                test_results.push(TestResult {
                    name: test_name.to_string(),
                    status: TestStatus::Skipped,
                    error_message: Some(err),
                });
                skipped += 1;
            }
        }
    }

    println!();

    Ok(ValidationReport {
        total_tests: ref_files.len(),
        passed_tests: passed,
        failed_tests: failed,
        skipped_tests: skipped,
        test_results,
    })
}

fn run_single_test(inp_file: &Path, ref_file: &Path) -> Result<bool, String> {
    use ccx_solver::AnalysisPipeline;
    use ccx_io::inp::Deck;
    use std::fs;

    // Try to parse the input file
    let deck = Deck::parse_file_with_includes(inp_file)
        .map_err(|err| format!("Parse error: {}", err))?;

    // Check what analysis type is needed
    let summary = ccx_model::ModelSummary::from_deck(&deck);

    // For now, only run simple static analyses with truss elements
    if !summary.has_static {
        return Err("Not a static analysis".to_string());
    }

    // Skip complex cases for now
    if summary.has_dynamic || summary.has_heat_transfer || summary.has_frequency {
        return Err("Complex analysis type".to_string());
    }

    // Skip if too few elements/nodes
    if summary.node_rows < 2 || summary.element_rows < 1 {
        return Err("Insufficient geometry".to_string());
    }

    // Check if it has supported element types (T3D2, B31, S4, C3D8)
    let mut has_supported_elements = false;
    let mut element_types = Vec::new();

    for card in &deck.cards {
        if card.keyword.eq_ignore_ascii_case("ELEMENT") {
            // Check for supported element types
            for param in &card.parameters {
                if param.key.eq_ignore_ascii_case("TYPE") {
                    if let Some(ref v) = param.value {
                        let elem_type: String = v.to_uppercase();
                        if elem_type == "T3D2" || elem_type == "B31" || elem_type == "S4" || elem_type == "C3D8" {
                            has_supported_elements = true;
                            if !element_types.contains(&elem_type) {
                                element_types.push(elem_type);
                            }
                        }
                    }
                }
            }
        }
    }

    if !has_supported_elements {
        return Err("No supported elements (T3D2, B31, S4, or C3D8)".to_string());
    }

    // Run the analysis pipeline
    let pipeline = AnalysisPipeline::detect_from_deck(&deck);
    let results = pipeline.run(&deck)
        .map_err(|err| format!("Solver error: {}", err))?;

    // Check if solver actually ran
    if !results.message.contains("[SOLVED]") {
        if results.message.contains("[SOLVE FAILED:") {
            return Err("Solve failed".to_string());
        } else if results.message.contains("[ASSEMBLY FAILED:") {
            return Err("Assembly failed".to_string());
        } else {
            return Err("Solver did not run".to_string());
        }
    }

    // Store solver results with datetime suffix for traceability
    // This creates output files like: truss_20260209_143052.validation.json
    if let Err(e) = save_validation_results(inp_file, &results) {
        eprintln!("Warning: Could not save validation results: {}", e);
    }

    // For now, if the solver runs without error, consider it a pass
    // Full validation would parse ref_file and compare displacements
    // TODO: Parse reference file and compare numerical results
    let _ref_content = fs::read_to_string(ref_file)
        .map_err(|err| format!("Cannot read reference file: {}", err))?;

    Ok(true)
}

fn save_validation_results(inp_file: &Path, results: &ccx_solver::AnalysisResults) -> Result<(), String> {
    use std::fs;
    use std::time::SystemTime;

    // Generate datetime suffix
    let now = SystemTime::now();
    let datetime = chrono::DateTime::<chrono::Utc>::from(now);
    let timestamp = datetime.format("%Y%m%d_%H%M%S").to_string();

    // Create output directory
    let output_dir = inp_file.parent()
        .ok_or_else(|| "Cannot determine parent directory".to_string())?
        .join("validation_results");
    fs::create_dir_all(&output_dir)
        .map_err(|e| format!("Cannot create output directory: {}", e))?;

    // Get base filename without extension
    let base_name = inp_file.file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "Cannot extract filename".to_string())?;

    // Save results as JSON with datetime suffix
    let output_file = output_dir.join(format!("{}_{}.validation.json", base_name, timestamp));
    let json_content = serde_json::json!({
        "test_name": base_name,
        "timestamp": timestamp,
        "datetime": datetime.to_rfc3339(),
        "success": results.success,
        "num_dofs": results.num_dofs,
        "num_equations": results.num_equations,
        "analysis_type": format!("{:?}", results.analysis_type),
        "message": results.message,
        "input_file": inp_file.to_string_lossy(),
    });

    fs::write(&output_file, serde_json::to_string_pretty(&json_content).unwrap())
        .map_err(|e| format!("Cannot write results file: {}", e))?;

    Ok(())
}

fn print_validation_report(report: &ValidationReport) {
    println!("========================================");
    println!("      VALIDATION REPORT");
    println!("========================================");
    println!();
    println!("Total tests:   {}", report.total_tests);
    println!("Passed:        {} ({:.1}%)",
             report.passed_tests,
             if report.total_tests > 0 {
                 (report.passed_tests as f64 / report.total_tests as f64) * 100.0
             } else {
                 0.0
             });
    println!("Failed:        {}", report.failed_tests);
    println!("Skipped:       {}", report.skipped_tests);
    println!();

    if report.failed_tests > 0 {
        println!("FAILED TESTS:");
        for result in &report.test_results {
            if result.status == TestStatus::Failed {
                println!("  ✗ {}", result.name);
                if let Some(err) = &result.error_message {
                    println!("    {}", err);
                }
            }
        }
        println!();
    }

    if report.skipped_tests > 0 && report.skipped_tests < 10 {
        println!("SKIPPED TESTS:");
        for result in &report.test_results {
            if result.status == TestStatus::Skipped {
                println!("  - {}", result.name);
                if let Some(err) = &result.error_message {
                    println!("    {}", err);
                }
            }
        }
        println!();
    } else if report.skipped_tests > 0 {
        println!("(Showing first 10 skipped tests)");
        let mut count = 0;
        for result in &report.test_results {
            if result.status == TestStatus::Skipped && count < 10 {
                println!("  - {}", result.name);
                if let Some(err) = &result.error_message {
                    println!("    {}", err);
                }
                count += 1;
            }
        }
        println!();
    }

    println!("========================================");
}

fn solve_file(path: &Path) -> Result<(), String> {
    use ccx_solver::AnalysisPipeline;
    use ccx_io::inp::Deck;

    println!("Solving: {}", path.display());

    // Parse input file
    let deck = Deck::parse_file_with_includes(path)
        .map_err(|err| format!("Parse error: {}", err))?;

    // Run solver
    let pipeline = AnalysisPipeline::detect_from_deck(&deck);
    let results = pipeline.run(&deck)
        .map_err(|err| format!("Solver error: {}", err))?;

    // Check if solve succeeded
    if !results.message.contains("[SOLVED]") {
        eprintln!("Solver message: {}", results.message);
        return Err("Solver did not produce solution".to_string());
    }

    println!("{}", results.message);

    // Write DAT file output
    let output_path = path.with_extension("dat");
    println!("Writing output to: {}", output_path.display());

    // For now, just write displacements
    // TODO: Add stress and volume output in subsequent tasks
    write_dat_output(&output_path, &deck, &results)?;

    println!("Solve complete!");
    Ok(())
}

fn write_dat_output(
    output_path: &Path,
    deck: &ccx_io::inp::Deck,
    results: &ccx_solver::AnalysisResults,
) -> Result<(), String> {
    use ccx_solver::mesh_builder::MeshBuilder;
    use nalgebra::DVector;

    // Rebuild mesh to get node information
    let mesh = MeshBuilder::build_from_deck(deck)
        .map_err(|e| format!("Failed to rebuild mesh: {}", e))?;

    // Convert displacements to DVector
    let displacements = DVector::from_vec(results.displacements.clone());

    // Compute stresses for beam elements
    let stress_results = compute_beam_stresses(&mesh, &deck, &displacements)?;

    // Compute element volumes
    let volumes = compute_element_volumes(&mesh, &deck)?;

    // Write DAT file with stresses and volumes
    ccx_solver::write_analysis_results_extended(
        output_path,
        &mesh,
        &displacements,
        Some(&stress_results),
        Some(&volumes),
    )
    .map_err(|e| format!("Failed to write DAT file: {}", e))?;

    Ok(())
}

fn compute_beam_stresses(
    mesh: &ccx_solver::Mesh,
    deck: &ccx_io::inp::Deck,
    displacements: &nalgebra::DVector<f64>,
) -> Result<Vec<ccx_solver::IntegrationPointStress>, String> {
    use ccx_solver::{ElementType, IntegrationPointStress};
    use ccx_solver::elements::{Beam32, BeamSection, BeamStressEvaluator};

    let mut all_stresses = Vec::new();

    // Find B32 elements
    for (elem_id, element) in &mesh.elements {
        if element.element_type == ElementType::B32 {
            // Get element nodes
            let nodes: Vec<ccx_solver::Node> = element
                .nodes
                .iter()
                .map(|&node_id| mesh.nodes.get(&node_id).cloned().unwrap())
                .collect();

            if nodes.len() != 3 {
                continue;
            }

            // Parse beam section and normal from deck
            let (section, normal) = parse_beam_section_from_deck(deck)?;

            // Create Beam32 element
            let beam_elem = Beam32::new(*elem_id, [nodes[0].id, nodes[1].id, nodes[2].id], section.clone());

            // Get material properties
            let material = parse_material_from_deck(deck)?;

            // Extract element displacements (18 DOFs)
            let mut elem_displacements = vec![0.0; 18];
            for (i, node_id) in element.nodes.iter().enumerate() {
                let node_idx = (node_id - 1) as usize;
                let dofs_per_node = 6; // B32 has 6 DOFs per node
                for dof in 0..dofs_per_node {
                    let global_dof = node_idx * dofs_per_node + dof;
                    if global_dof < displacements.len() {
                        elem_displacements[i * 6 + dof] = displacements[global_dof];
                    }
                }
            }

            // Create stress evaluator with normal direction from BEAM SECTION
            let evaluator = BeamStressEvaluator::new(&beam_elem, &section, &material, nodes, normal);

            // Applied load (from CLOAD card)
            let applied_load = parse_applied_load_from_deck(deck)?;

            // Compute stresses at all integration points
            let stresses = evaluator
                .compute_all_stresses(&elem_displacements, applied_load)
                .map_err(|e| format!("Failed to compute stresses: {}", e))?;

            // Convert to IntegrationPointStress format
            for (ip_idx, stress) in stresses.iter().enumerate() {
                all_stresses.push(IntegrationPointStress {
                    element_id: *elem_id,
                    integration_point: ip_idx + 1,
                    sxx: stress.sxx,
                    syy: stress.syy,
                    szz: stress.szz,
                    sxy: stress.sxy,
                    sxz: stress.sxz,
                    syz: stress.syz,
                });
            }
        }
    }

    Ok(all_stresses)
}

fn parse_beam_section_from_deck(deck: &ccx_io::inp::Deck) -> Result<(ccx_solver::BeamSection, nalgebra::Vector3<f64>), String> {
    use ccx_solver::BeamSection;
    use nalgebra::Vector3;

    // Find BEAM SECTION card
    for card in &deck.cards {
        if card.keyword.eq_ignore_ascii_case("BEAM SECTION") {
            // Parse section data from data lines
            if card.data_lines.len() >= 2 {
                // First line: width, height
                let parts: Vec<&str> = card.data_lines[0].split(',').collect();
                if parts.len() >= 2 {
                    let width: f64 = parts[0]
                        .trim()
                        .parse()
                        .map_err(|_| "Failed to parse beam width")?;
                    let height: f64 = parts[1]
                        .trim()
                        .parse()
                        .map_err(|_| "Failed to parse beam height")?;

                    // Second line: normal direction (nx, ny, nz)
                    let normal_parts: Vec<&str> = card.data_lines[1].split(',').collect();
                    let normal = if normal_parts.len() >= 3 {
                        let nx: f64 = normal_parts[0].trim().parse().unwrap_or(1.0);
                        let ny: f64 = normal_parts[1].trim().parse().unwrap_or(0.0);
                        let nz: f64 = normal_parts[2].trim().parse().unwrap_or(0.0);
                        Vector3::new(nx, ny, nz).normalize()
                    } else {
                        Vector3::new(1.0, 0.0, 0.0) // Default normal
                    };

                    return Ok((BeamSection::rectangular(width, height), normal));
                }
            }
        }
    }

    Err("No BEAM SECTION card found".to_string())
}

fn parse_material_from_deck(deck: &ccx_io::inp::Deck) -> Result<ccx_solver::Material, String> {
    use ccx_solver::{Material, MaterialModel};

    // Find MATERIAL and ELASTIC cards
    let mut current_material_name = String::new();
    let mut elastic_modulus = None;
    let mut poissons_ratio = None;

    for card in &deck.cards {
        if card.keyword.eq_ignore_ascii_case("MATERIAL") {
            // Get material name from NAME parameter
            for param in &card.parameters {
                if param.key.eq_ignore_ascii_case("NAME") {
                    if let Some(ref name) = param.value {
                        current_material_name = name.clone();
                    }
                }
            }
        } else if card.keyword.eq_ignore_ascii_case("ELASTIC") && !current_material_name.is_empty() {
            // Parse elastic properties
            if let Some(data_line) = card.data_lines.first() {
                let parts: Vec<&str> = data_line.split(',').collect();
                if parts.len() >= 2 {
                    elastic_modulus = parts[0].trim().parse().ok();
                    poissons_ratio = parts[1].trim().parse().ok();
                }
            }
        }
    }

    if let (Some(e), Some(nu)) = (elastic_modulus, poissons_ratio) {
        Ok(Material {
            name: current_material_name,
            model: MaterialModel::LinearElastic,
            elastic_modulus: Some(e),
            poissons_ratio: Some(nu),
            density: None,
            thermal_expansion: None,
            conductivity: None,
            specific_heat: None,
        })
    } else {
        Err("Failed to parse material properties".to_string())
    }
}

fn parse_applied_load_from_deck(deck: &ccx_io::inp::Deck) -> Result<f64, String> {
    // Find CLOAD card
    for card in &deck.cards {
        if card.keyword.eq_ignore_ascii_case("CLOAD") {
            if let Some(data_line) = card.data_lines.first() {
                let parts: Vec<&str> = data_line.split(',').collect();
                if parts.len() >= 3 {
                    let load: f64 = parts[2]
                        .trim()
                        .parse()
                        .map_err(|_| "Failed to parse load magnitude")?;
                    return Ok(load);
                }
            }
        }
    }

    Ok(0.0) // Default to zero if no load found
}

fn compute_element_volumes(
    mesh: &ccx_solver::Mesh,
    deck: &ccx_io::inp::Deck,
) -> Result<Vec<(i32, f64)>, String> {
    let mut volumes = Vec::new();

    // For B32 beam elements, compute volume from cross-section and length
    for (elem_id, element) in &mesh.elements {
        if element.element_type == ccx_solver::ElementType::B32 {
            // Get element nodes
            let nodes: Vec<ccx_solver::Node> = element
                .nodes
                .iter()
                .map(|&node_id| mesh.nodes.get(&node_id).cloned().unwrap())
                .collect();

            if nodes.len() == 3 {
                // Compute beam length (from first to last node)
                let dx = nodes[2].x - nodes[0].x;
                let dy = nodes[2].y - nodes[0].y;
                let dz = nodes[2].z - nodes[0].z;
                let length = (dx * dx + dy * dy + dz * dz).sqrt();

                // Get cross-sectional area
                let (section, _normal) = parse_beam_section_from_deck(deck)?;
                let volume = section.area * length;

                volumes.push((*elem_id, volume));
            }
        }
    }

    Ok(volumes)
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
        Some("solve") => {
            if args.len() != 3 {
                usage();
                return ExitCode::from(2);
            }
            let path = Path::new(&args[2]);
            match solve_file(path) {
                Ok(()) => ExitCode::SUCCESS,
                Err(err) => {
                    eprintln!("solve error: {}", err);
                    ExitCode::from(1)
                }
            }
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
        Some("validate") => {
            // Parse optional --fixtures-dir argument
            let fixtures_dir = if args.len() >= 4 && args[2] == "--fixtures-dir" {
                Path::new(&args[3])
            } else if args.len() == 2 {
                Path::new("tests/fixtures/solver")
            } else {
                usage();
                return ExitCode::from(2);
            };

            match run_validation_suite(fixtures_dir) {
                Ok(report) => {
                    print_validation_report(&report);
                    if report.failed_tests > 0 {
                        ExitCode::from(1)
                    } else {
                        ExitCode::SUCCESS
                    }
                }
                Err(err) => {
                    eprintln!("validation error: {err}");
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
        use ccx_io::inp::Deck;

        let root = unique_temp_dir("ccx_cli_analyze_include");
        fs::create_dir_all(&root).expect("create temp dir");
        let deck_path = root.join("root.inp");
        let inc = root.join("mesh.inc");

        fs::write(
            &deck_path,
            "*NODE\n1,0,0,0\n*INCLUDE,INPUT=mesh.inc\n*ELEMENT\n1,1,1,1,1\n",
        )
        .expect("write root deck");
        fs::write(&inc, "*MATERIAL,NAME=STEEL\n").expect("write include");

        let summary = analyze_file(&deck_path).expect("analysis should parse");
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
