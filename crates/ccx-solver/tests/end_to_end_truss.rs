//! End-to-end integration test for simple truss problem.
//!
//! This test validates the complete analysis pipeline:
//! 1. Parse input deck
//! 2. Build mesh
//! 3. Build materials
//! 4. Build boundary conditions
//! 5. Assemble global system
//! 6. Solve for displacements
//! 7. Verify results against analytical solution

use ccx_io::inp::Deck;
use ccx_solver::{BCBuilder, GlobalSystem, MaterialLibrary, MeshBuilder};

#[test]
fn simple_truss_end_to_end() {
    // Simple truss problem: 2 nodes, 1 element, axial loading
    let input = r#"
*NODE
1, 0.0, 0.0, 0.0
2, 1.0, 0.0, 0.0
*ELEMENT, TYPE=T3D2
1, 1, 2
*MATERIAL, NAME=STEEL
*ELASTIC
210000, 0.3
*BOUNDARY
1, 1, 3
2, 2, 3
*CLOAD
2, 1, 1000.0
*STEP
*STATIC
*END STEP
"#;

    // Parse input
    let deck = Deck::parse_str(input).expect("Failed to parse input");

    // Build mesh
    let mut mesh = MeshBuilder::build_from_deck(&deck).expect("Failed to build mesh");
    mesh.calculate_dofs();

    // Build materials
    let mut materials = MaterialLibrary::build_from_deck(&deck).expect("Failed to build materials");
    // Assign material to element 1
    materials.assign_material(1, "STEEL".to_string());

    // Build boundary conditions
    let bcs = BCBuilder::build_from_deck(&deck).expect("Failed to build BCs");

    // Assemble system (assume area = 0.001 m² = 1000 mm²)
    let area = 0.001;
    let system =
        GlobalSystem::assemble(&mesh, &materials, &bcs, area).expect("Failed to assemble system");

    // Validate system
    assert!(system.validate().is_ok(), "System validation failed");

    // Solve
    let u = system.solve().expect("Failed to solve system");

    // Analytical solution:
    // u = FL/AE = 1000 * 1.0 / (0.001 * 210000) = 0.004761905 m = 4.76 mm
    let expected_u = 1000.0 * 1.0 / (0.001 * 210000.0);

    // Node 1 should be fixed (penalty method gives small but non-zero values)
    assert!(
        u[0].abs() < 1e-6,
        "Node 1 x-displacement should be ~0, got {}",
        u[0]
    );
    assert!(
        u[1].abs() < 1e-6,
        "Node 1 y-displacement should be ~0, got {}",
        u[1]
    );
    assert!(
        u[2].abs() < 1e-6,
        "Node 1 z-displacement should be ~0, got {}",
        u[2]
    );

    // Node 2 x-displacement should match analytical solution
    let computed_u = u[3];
    let error = (computed_u - expected_u).abs() / expected_u;
    assert!(
        error < 1e-6,
        "Node 2 x-displacement error too large: computed={}, expected={}, error={}",
        computed_u,
        expected_u,
        error
    );

    // Node 2 y and z displacements should be 0 (constrained)
    assert!(u[4].abs() < 1e-6, "Node 2 y-displacement should be ~0");
    assert!(u[5].abs() < 1e-6, "Node 2 z-displacement should be ~0");
}

#[test]
fn three_bar_truss() {
    // Three-bar truss: classic triangular truss problem
    let input = r#"
*NODE
1, 0.0, 0.0, 0.0
2, 1.0, 0.0, 0.0
3, 0.5, 0.866, 0.0
*ELEMENT, TYPE=T3D2
1, 1, 2
2, 2, 3
3, 3, 1
*MATERIAL, NAME=STEEL
*ELASTIC
210000, 0.3
*BOUNDARY
1, 1, 3
2, 1, 3
3, 3, 3
*CLOAD
3, 2, -1000.0
*STEP
*STATIC
*END STEP
"#;

    let deck = Deck::parse_str(input).expect("Failed to parse input");
    let mut mesh = MeshBuilder::build_from_deck(&deck).expect("Failed to build mesh");
    mesh.calculate_dofs();

    let mut materials = MaterialLibrary::build_from_deck(&deck).expect("Failed to build materials");
    // Assign material to all elements
    materials.assign_material(1, "STEEL".to_string());
    materials.assign_material(2, "STEEL".to_string());
    materials.assign_material(3, "STEEL".to_string());

    let bcs = BCBuilder::build_from_deck(&deck).expect("Failed to build BCs");

    // Assemble and solve
    let area = 0.001;
    let system =
        GlobalSystem::assemble(&mesh, &materials, &bcs, area).expect("Failed to assemble system");

    assert!(system.validate().is_ok());

    let u = system.solve().expect("Failed to solve system");

    // Node 1 and 2 are fixed
    assert!(u[0].abs() < 1e-6);
    assert!(u[1].abs() < 1e-6);
    assert!(u[2].abs() < 1e-6);
    assert!(u[3].abs() < 1e-6);
    assert!(u[4].abs() < 1e-6);
    assert!(u[5].abs() < 1e-6);

    // Node 3 should have displacement in negative y direction
    assert!(u[7] < 0.0, "Node 3 should displace downward (negative y)");

    // Node 3 x-displacement should be near zero (symmetry)
    assert!(
        u[6].abs() < 1e-6,
        "Node 3 x-displacement should be near zero due to symmetry"
    );
}

#[test]
fn material_properties_affect_displacement() {
    // Same problem with different materials should give different displacements
    let base_input = r#"
*NODE
1, 0.0, 0.0, 0.0
2, 1.0, 0.0, 0.0
*ELEMENT, TYPE=T3D2
1, 1, 2
*MATERIAL, NAME=MAT
*ELASTIC
{E}, 0.3
*BOUNDARY
1, 1, 3
2, 2, 3
*CLOAD
2, 1, 1000.0
*STEP
*STATIC
*END STEP
"#;

    // Solve with E = 210000 MPa (steel)
    let input_steel = base_input.replace("{E}", "210000");
    let deck_steel = Deck::parse_str(&input_steel).unwrap();
    let mut mesh_steel = MeshBuilder::build_from_deck(&deck_steel).unwrap();
    mesh_steel.calculate_dofs();
    let mut materials_steel = MaterialLibrary::build_from_deck(&deck_steel).unwrap();
    materials_steel.assign_material(1, "MAT".to_string());
    let bcs_steel = BCBuilder::build_from_deck(&deck_steel).unwrap();
    let system_steel =
        GlobalSystem::assemble(&mesh_steel, &materials_steel, &bcs_steel, 0.001).unwrap();
    let u_steel = system_steel.solve().unwrap();

    // Solve with E = 70000 MPa (aluminum)
    let input_aluminum = base_input.replace("{E}", "70000");
    let deck_aluminum = Deck::parse_str(&input_aluminum).unwrap();
    let mut mesh_aluminum = MeshBuilder::build_from_deck(&deck_aluminum).unwrap();
    mesh_aluminum.calculate_dofs();
    let mut materials_aluminum = MaterialLibrary::build_from_deck(&deck_aluminum).unwrap();
    materials_aluminum.assign_material(1, "MAT".to_string());
    let bcs_aluminum = BCBuilder::build_from_deck(&deck_aluminum).unwrap();
    let system_aluminum =
        GlobalSystem::assemble(&mesh_aluminum, &materials_aluminum, &bcs_aluminum, 0.001).unwrap();
    let u_aluminum = system_aluminum.solve().unwrap();

    // Aluminum (E=70000) should have 3x larger displacement than steel (E=210000)
    let ratio = u_aluminum[3] / u_steel[3];
    assert!(
        (ratio - 3.0).abs() < 0.01,
        "Displacement ratio should be 3.0, got {}",
        ratio
    );
}

#[test]
fn load_magnitude_scales_linearly() {
    // Linear static: displacement should scale linearly with load
    let base_input = r#"
*NODE
1, 0.0, 0.0, 0.0
2, 1.0, 0.0, 0.0
*ELEMENT, TYPE=T3D2
1, 1, 2
*MATERIAL, NAME=STEEL
*ELASTIC
210000, 0.3
*BOUNDARY
1, 1, 3
2, 2, 3
*CLOAD
2, 1, {LOAD}
*STEP
*STATIC
*END STEP
"#;

    // Solve with load = 100 N
    let input_100 = base_input.replace("{LOAD}", "100.0");
    let deck_100 = Deck::parse_str(&input_100).unwrap();
    let mut mesh_100 = MeshBuilder::build_from_deck(&deck_100).unwrap();
    mesh_100.calculate_dofs();
    let mut materials_100 = MaterialLibrary::build_from_deck(&deck_100).unwrap();
    materials_100.assign_material(1, "STEEL".to_string());
    let bcs_100 = BCBuilder::build_from_deck(&deck_100).unwrap();
    let system_100 = GlobalSystem::assemble(&mesh_100, &materials_100, &bcs_100, 0.001).unwrap();
    let u_100 = system_100.solve().unwrap();

    // Solve with load = 500 N
    let input_500 = base_input.replace("{LOAD}", "500.0");
    let deck_500 = Deck::parse_str(&input_500).unwrap();
    let mut mesh_500 = MeshBuilder::build_from_deck(&deck_500).unwrap();
    mesh_500.calculate_dofs();
    let mut materials_500 = MaterialLibrary::build_from_deck(&deck_500).unwrap();
    materials_500.assign_material(1, "STEEL".to_string());
    let bcs_500 = BCBuilder::build_from_deck(&deck_500).unwrap();
    let system_500 = GlobalSystem::assemble(&mesh_500, &materials_500, &bcs_500, 0.001).unwrap();
    let u_500 = system_500.solve().unwrap();

    // Displacement should scale by factor of 5
    let ratio = u_500[3] / u_100[3];
    assert!(
        (ratio - 5.0).abs() < 0.01,
        "Displacement ratio should be 5.0, got {}",
        ratio
    );
}
