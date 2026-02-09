use ccx_solver::{
    compute_effective_strain, compute_mises_stress, compute_statistics, process_integration_points,
    read_dat_file, write_results, StressState, StrainState,
};
use std::fs;

#[test]
fn test_read_sample_dat_file() {
    // Create a temporary .dat file
    let temp_dir = std::env::temp_dir();
    let dat_path = temp_dir.join("test_sample.dat");

    let dat_content = r#"
 stresses (elem, integ.pnt.,sxx,syy,szz,sxy,sxz,syz)

       1       1   1.0000E+02   5.0000E+01   3.0000E+01   1.0000E+01   5.0000E+00   3.0000E+00
       1       2   1.2000E+02   6.0000E+01   4.0000E+01   1.2000E+01   6.0000E+00   4.0000E+00
       2       1   8.0000E+01   4.0000E+01   2.0000E+01   8.0000E+00   4.0000E+00   2.0000E+00

 strains (elem, integ.pnt.,exx,eyy,ezz,exy,exz,eyz)

       1       1   1.0000E-03   5.0000E-04   3.0000E-04   1.0000E-04   5.0000E-05   3.0000E-05
       1       2   1.2000E-03   6.0000E-04   4.0000E-04   1.2000E-04   6.0000E-05   4.0000E-05
       2       1   8.0000E-04   4.0000E-04   2.0000E-04   8.0000E-05   4.0000E-05   2.0000E-05

 equivalent plastic strain

       1       1   0.0000E+00
       1       2   1.0000E-04
       2       1   0.0000E+00
"#;

    fs::write(&dat_path, dat_content).expect("Failed to write test .dat file");

    // Read the file
    let data = read_dat_file(&dat_path).expect("Failed to read .dat file");

    // Verify we got 3 integration points
    assert_eq!(data.len(), 3);

    // Verify first integration point
    assert_eq!(data[0].element_id, 1);
    assert_eq!(data[0].point_id, 1);
    assert!(data[0].stress.is_some());
    assert!(data[0].strain.is_some());
    assert_eq!(data[0].peeq, Some(0.0));

    let stress = data[0].stress.as_ref().unwrap();
    assert_eq!(stress.sxx, 100.0);
    assert_eq!(stress.syy, 50.0);
    assert_eq!(stress.szz, 30.0);

    let strain = data[0].strain.as_ref().unwrap();
    assert_eq!(strain.exx, 0.001);
    assert_eq!(strain.eyy, 0.0005);
    assert_eq!(strain.ezz, 0.0003);

    // Clean up
    fs::remove_file(&dat_path).ok();
}

#[test]
fn test_full_postprocessing_workflow() {
    // Create a temporary .dat file
    let temp_dir = std::env::temp_dir();
    let dat_path = temp_dir.join("test_workflow.dat");

    let dat_content = r#"
 stresses (elem, integ.pnt.,sxx,syy,szz,sxy,sxz,syz)

       1       1   1.0000E+02   0.0000E+00   0.0000E+00   0.0000E+00   0.0000E+00   0.0000E+00
       2       1   2.0000E+02   0.0000E+00   0.0000E+00   0.0000E+00   0.0000E+00   0.0000E+00

 strains (elem, integ.pnt.,exx,eyy,ezz,exy,exz,eyz)

       1       1   1.0000E-03   0.0000E+00   0.0000E+00   0.0000E+00   0.0000E+00   0.0000E+00
       2       1   2.0000E-03   0.0000E+00   0.0000E+00   0.0000E+00   0.0000E+00   0.0000E+00
"#;

    fs::write(&dat_path, dat_content).expect("Failed to write test .dat file");

    // Read the file
    let data = read_dat_file(&dat_path).expect("Failed to read .dat file");
    assert_eq!(data.len(), 2);

    // Process integration points
    let results = process_integration_points(&data);
    assert_eq!(results.len(), 2);

    // For uniaxial stress, Mises = |σ_xx|
    assert!((results[0].mises - 100.0).abs() < 1e-10);
    assert!((results[1].mises - 200.0).abs() < 1e-10);

    // Compute statistics
    let stats = compute_statistics(&results);
    assert_eq!(stats.mises_min, 100.0);
    assert_eq!(stats.mises_max, 200.0);
    assert_eq!(stats.mises_mean, 150.0);

    // Write results
    write_results(&dat_path, &results, &stats).expect("Failed to write results");

    // Verify output file was created
    let output_path = temp_dir.join("test_workflow_IntPtOutput.txt");
    assert!(output_path.exists());

    // Verify output contains expected data
    let output_content = fs::read_to_string(&output_path).expect("Failed to read output file");
    assert!(output_content.contains("MISES"));
    assert!(output_content.contains("EEQ"));
    assert!(output_content.contains("PEEQ"));
    assert!(output_content.contains("Minimum"));
    assert!(output_content.contains("Maximum"));
    assert!(output_content.contains("Mean"));

    // Clean up
    fs::remove_file(&dat_path).ok();
    fs::remove_file(&output_path).ok();
}

#[test]
fn test_dat_file_stress_only() {
    // Test file with only stress data (no strain or PEEQ)
    let temp_dir = std::env::temp_dir();
    let dat_path = temp_dir.join("test_stress_only.dat");

    let dat_content = r#"
 stresses (elem, integ.pnt.,sxx,syy,szz,sxy,sxz,syz)

       1       1   0.0000E+00   0.0000E+00   0.0000E+00   1.0000E+02   0.0000E+00   0.0000E+00
"#;

    fs::write(&dat_path, dat_content).expect("Failed to write test .dat file");

    // Read the file
    let data = read_dat_file(&dat_path).expect("Failed to read .dat file");
    assert_eq!(data.len(), 1);

    assert!(data[0].stress.is_some());
    assert!(data[0].strain.is_none());
    assert!(data[0].peeq.is_none());

    // Process - should handle missing strain/PEEQ gracefully
    let results = process_integration_points(&data);
    assert_eq!(results.len(), 1);

    // For pure shear τ_xy = 100, Mises = sqrt(3) * 100
    let expected_mises = (3.0_f64).sqrt() * 100.0;
    assert!((results[0].mises - expected_mises).abs() < 1e-10);
    assert_eq!(results[0].eeq, 0.0); // No strain data
    assert_eq!(results[0].peeq, 0.0); // No PEEQ data

    // Clean up
    fs::remove_file(&dat_path).ok();
}

#[test]
fn test_mises_stress_calculation_accuracy() {
    // Test against known Mises stress values for specific stress states

    // Pure tension
    let stress = StressState {
        sxx: 100.0,
        syy: 0.0,
        szz: 0.0,
        sxy: 0.0,
        sxz: 0.0,
        syz: 0.0,
    };
    let mises = compute_mises_stress(&stress);
    assert!((mises - 100.0).abs() < 1e-10);

    // Biaxial stress (σ_xx = 100, σ_yy = 50)
    // Mises = sqrt(σ_xx² - σ_xx*σ_yy + σ_yy²) = sqrt(10000 - 5000 + 2500) = sqrt(7500) ≈ 86.6
    let stress = StressState {
        sxx: 100.0,
        syy: 50.0,
        szz: 0.0,
        sxy: 0.0,
        sxz: 0.0,
        syz: 0.0,
    };
    let mises = compute_mises_stress(&stress);
    let expected = (7500.0_f64).sqrt();
    assert!((mises - expected).abs() < 1e-10);

    // Triaxial equal stress (hydrostatic)
    // Mises = 0 for pure hydrostatic stress
    let stress = StressState {
        sxx: 100.0,
        syy: 100.0,
        szz: 100.0,
        sxy: 0.0,
        sxz: 0.0,
        syz: 0.0,
    };
    let mises = compute_mises_stress(&stress);
    assert!((mises - 0.0).abs() < 1e-10);
}

#[test]
fn test_effective_strain_calculation_accuracy() {
    // Pure uniaxial strain
    let strain = StrainState {
        exx: 0.001,
        eyy: 0.0,
        ezz: 0.0,
        exy: 0.0,
        exz: 0.0,
        eyz: 0.0,
    };
    let eeq = compute_effective_strain(&strain);
    let expected = (2.0 / 3.0) * (1e-6_f64).sqrt();
    assert!((eeq - expected).abs() < 1e-12);

    // Pure shear strain
    let strain = StrainState {
        exx: 0.0,
        eyy: 0.0,
        ezz: 0.0,
        exy: 0.001,
        exz: 0.0,
        eyz: 0.0,
    };
    let eeq = compute_effective_strain(&strain);
    let expected = (2.0 / 3.0) * (3.0 * 1e-6_f64).sqrt();
    assert!((eeq - expected).abs() < 1e-12);
}
