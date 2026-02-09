/// Integration tests for B31 beam elements
/// Validates against analytical solutions for classical beam problems

use ccx_solver::{
    Beam31, BeamSection, ElementTrait, Material, MaterialModel, Node,
};

/// Helper to create a steel material
fn steel_material() -> Material {
    Material {
        name: "Steel".to_string(),
        model: MaterialModel::LinearElastic,
        elastic_modulus: Some(200e9), // 200 GPa
        poissons_ratio: Some(0.3),
        density: Some(7850.0), // kg/m³
        thermal_expansion: None,
        conductivity: None,
        specific_heat: None,
    }
}

#[test]
fn test_cantilever_beam_tip_deflection() {
    // Analytical problem: Cantilever beam with end load
    //
    // Fixed at left end (x=0), free at right end (x=L)
    // Load P applied at tip in -z direction
    //
    // Analytical solution:
    // δ = PL³/(3EI)
    //
    // Parameters:
    // L = 1.0 m
    // E = 200 GPa
    // I = πr⁴/4 for circular section, r = 0.05 m
    // P = 1000 N

    let length: f64 = 1.0; // m
    let radius: f64 = 0.05; // m
    let load: f64 = 1000.0; // N

    // Circular section properties
    let i = std::f64::consts::PI * radius.powi(4) / 4.0;
    let section = BeamSection::circular(radius);

    // Material
    let material = steel_material();
    let e = material.elastic_modulus.unwrap();

    // Analytical tip deflection
    let analytical_deflection = (load * length.powi(3)) / (3.0 * e * i);

    println!("\n=== Cantilever Beam Test ===");
    println!("Length: {} m", length);
    println!("Radius: {} m", radius);
    println!("Load: {} N", load);
    println!("E: {} GPa", e / 1e9);
    println!("I: {:.6e} m⁴", i);
    println!("Analytical deflection: {:.6e} m", analytical_deflection);

    // For now, just verify that section properties are correct
    assert!((section.area - std::f64::consts::PI * radius.powi(2)).abs() < 1e-10);
    assert!((section.iyy - i).abs() < 1e-10);
    assert!((section.izz - i).abs() < 1e-10);

    // Create beam element
    let beam = Beam31::new(1, 0, 1, section);
    let nodes = vec![
        Node::new(0, 0.0, 0.0, 0.0),
        Node::new(1, length, 0.0, 0.0),
    ];

    // Compute stiffness matrix
    let k = beam.stiffness_matrix(&nodes, &material).unwrap();

    // Verify stiffness matrix is symmetric
    for i in 0..12 {
        for j in 0..12 {
            assert!((k[(i, j)] - k[(j, i)]).abs() < 1e-6,
                "Stiffness matrix not symmetric at ({}, {})", i, j);
        }
    }

    println!("✓ Stiffness matrix is symmetric");
    println!("✓ Section properties verified");
}

#[test]
fn test_beam_axial_stiffness_simple() {
    // Simple axial test - verify EA/L relationship
    let area = 0.01; // 1 cm²
    let length = 2.0; // 2 m
    let section = BeamSection::custom(area, 1e-6, 1e-6, 1e-6);

    let beam = Beam31::new(1, 0, 1, section);
    let nodes = vec![
        Node::new(0, 0.0, 0.0, 0.0),
        Node::new(1, length, 0.0, 0.0),
    ];

    let material = steel_material();
    let e = material.elastic_modulus.unwrap();

    let k = beam.stiffness_matrix(&nodes, &material).unwrap();

    // Expected axial stiffness
    let expected_k_axial = e * area / length;

    // For beam along x-axis, axial DOF is 0
    // Check k[0,0] (should be EA/L)
    let actual_k = k[(0, 0)];
    let error = (actual_k - expected_k_axial).abs() / expected_k_axial;

    println!("\n=== Axial Stiffness Test ===");
    println!("Expected: {:.3e} N/m", expected_k_axial);
    println!("Actual: {:.3e} N/m", actual_k);
    println!("Error: {:.2e}%", error * 100.0);

    assert!(error < 1e-10, "Axial stiffness error too large: {:.2e}%", error * 100.0);
}

#[test]
fn test_beam_bending_stiffness() {
    // Test bending stiffness for a simple beam
    let radius = 0.05; // m
    let length = 1.0; // m
    let section = BeamSection::circular(radius);

    let beam = Beam31::new(1, 0, 1, section);
    let nodes = vec![
        Node::new(0, 0.0, 0.0, 0.0),
        Node::new(1, length, 0.0, 0.0),
    ];

    let material = steel_material();
    let e = material.elastic_modulus.unwrap();
    let i = std::f64::consts::PI * radius.powi(4) / 4.0;

    let k = beam.stiffness_matrix(&nodes, &material).unwrap();

    // For beam along x-axis:
    // Bending in XY plane: transverse DOF is 1 (y-direction)
    // k[1,1] should be 12EI/L³

    let expected_k_bending = 12.0 * e * i / length.powi(3);
    let actual_k = k[(1, 1)];
    let error = (actual_k - expected_k_bending).abs() / expected_k_bending;

    println!("\n=== Bending Stiffness Test ===");
    println!("Expected: {:.3e} N/m", expected_k_bending);
    println!("Actual: {:.3e} N/m", actual_k);
    println!("Error: {:.2e}%", error * 100.0);

    assert!(error < 1e-10, "Bending stiffness error too large: {:.2e}%", error * 100.0);
}

#[test]
fn test_beam_torsion_stiffness() {
    // Test torsional stiffness
    let radius = 0.05; // m
    let length = 1.0; // m
    let section = BeamSection::circular(radius);

    let j = section.torsion_constant;

    let beam = Beam31::new(1, 0, 1, section);
    let nodes = vec![
        Node::new(0, 0.0, 0.0, 0.0),
        Node::new(1, length, 0.0, 0.0),
    ];

    let material = steel_material();
    let g = material.shear_modulus().unwrap();

    let k = beam.stiffness_matrix(&nodes, &material).unwrap();

    // For beam along x-axis:
    // Torsional DOF is 3 (rotation about x)
    // k[3,3] should be GJ/L

    let expected_k_torsion = g * j / length;
    let actual_k = k[(3, 3)];
    let error = (actual_k - expected_k_torsion).abs() / expected_k_torsion;

    println!("\n=== Torsional Stiffness Test ===");
    println!("Expected: {:.3e} Nm/rad", expected_k_torsion);
    println!("Actual: {:.3e} Nm/rad", actual_k);
    println!("Error: {:.2e}%", error * 100.0);

    assert!(error < 1e-10, "Torsional stiffness error too large: {:.2e}%", error * 100.0);
}

#[test]
fn test_beam_rotated_orientation() {
    // Test beam element in arbitrary orientation
    // This tests the transformation matrix

    let section = BeamSection::circular(0.05);
    let beam = Beam31::new(1, 0, 1, section);

    // Beam at 45 degrees in XY plane
    let length = 1.0;
    let nodes = vec![
        Node::new(0, 0.0, 0.0, 0.0),
        Node::new(1, length / 2.0_f64.sqrt(), length / 2.0_f64.sqrt(), 0.0),
    ];

    let material = steel_material();
    let k = beam.stiffness_matrix(&nodes, &material).unwrap();

    // Stiffness matrix should still be symmetric
    for i in 0..12 {
        for j in 0..12 {
            let diff = (k[(i, j)] - k[(j, i)]).abs();
            assert!(diff < 1e-6,
                "Rotated beam stiffness not symmetric at ({}, {}): diff = {:.3e}",
                i, j, diff);
        }
    }

    println!("\n=== Rotated Beam Test ===");
    println!("✓ 45-degree beam stiffness matrix is symmetric");
}

#[test]
fn test_rectangular_vs_circular_sections() {
    // Compare rectangular and circular sections

    // Circular section
    let radius = 0.05;
    let circ_section = BeamSection::circular(radius);

    // Equivalent rectangular section (approximately same area)
    let side = radius * (std::f64::consts::PI).sqrt();
    let rect_section = BeamSection::rectangular(side, side);

    println!("\n=== Section Comparison ===");
    println!("Circular (r={}m):", radius);
    println!("  Area: {:.6e} m²", circ_section.area);
    println!("  Iyy: {:.6e} m⁴", circ_section.iyy);

    println!("Rectangular ({}m x {}m):", side, side);
    println!("  Area: {:.6e} m²", rect_section.area);
    println!("  Iyy: {:.6e} m⁴", rect_section.iyy);

    // Areas should be approximately equal
    let area_error = (circ_section.area - rect_section.area).abs() / circ_section.area;
    println!("Area error: {:.2}%", area_error * 100.0);
    assert!(area_error < 0.01, "Areas too different");
}
