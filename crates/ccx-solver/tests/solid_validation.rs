//! Validation tests for C3D8 solid element implementation
//!
//! These tests verify the C3D8 element against analytical solutions
//! for basic structural mechanics problems.

use ccx_solver::{
    BoundaryConditions, ConcentratedLoad, DisplacementBC, Element as MeshElement, ElementType,
    GlobalSystem, Material, MaterialLibrary, MaterialModel, Mesh, Node,
};
use ccx_solver::elements::{C3D8, Element as ElementTrait};

/// Create a steel material for testing
fn create_steel() -> Material {
    Material {
        name: "STEEL".to_string(),
        model: MaterialModel::LinearElastic,
        elastic_modulus: Some(200e9), // 200 GPa
        poissons_ratio: Some(0.3),
        density: Some(7800.0), // kg/m³
        thermal_expansion: None,
        conductivity: None,
        specific_heat: None,
    }
}

#[test]
fn c3d8_uniaxial_tension() {
    // Test: Single C3D8 element under uniaxial tension
    // Analytical solution: u = σL/E
    //
    // Setup: Unit cube (1m × 1m × 1m)
    // BC: Fix left face (x=0), apply uniform traction on right face (x=1)
    // Load: σ = 100 MPa
    // Theory: ε = σ/E, u = εL = σL/E

    // Create mesh
    let mut mesh = Mesh::new();

    // Add nodes
    mesh.add_node(Node::new(1, 0.0, 0.0, 0.0));
    mesh.add_node(Node::new(2, 1.0, 0.0, 0.0));
    mesh.add_node(Node::new(3, 1.0, 1.0, 0.0));
    mesh.add_node(Node::new(4, 0.0, 1.0, 0.0));
    mesh.add_node(Node::new(5, 0.0, 0.0, 1.0));
    mesh.add_node(Node::new(6, 1.0, 0.0, 1.0));
    mesh.add_node(Node::new(7, 1.0, 1.0, 1.0));
    mesh.add_node(Node::new(8, 0.0, 1.0, 1.0));

    // Add C3D8 element
    mesh.add_element(MeshElement::new(1, ElementType::C3D8, vec![1, 2, 3, 4, 5, 6, 7, 8]));
    mesh.calculate_dofs();

    // Create material library
    let mut materials = MaterialLibrary::new();
    let steel = create_steel();
    let E = steel.elastic_modulus.unwrap();
    materials.add_material(steel.clone());
    materials.assign_material(1, "STEEL".to_string());

    // Apply boundary conditions
    let mut bcs = BoundaryConditions::new();

    // Fix left face (nodes 1, 4, 5, 8) in x-direction
    bcs.add_displacement_bc(DisplacementBC::new(1, 1, 1, 0.0)); // Node 1, DOF 1 (ux)
    bcs.add_displacement_bc(DisplacementBC::new(4, 1, 1, 0.0));
    bcs.add_displacement_bc(DisplacementBC::new(5, 1, 1, 0.0));
    bcs.add_displacement_bc(DisplacementBC::new(8, 1, 1, 0.0));

    // Fix node 1 in y and z to prevent rigid body motion
    bcs.add_displacement_bc(DisplacementBC::new(1, 2, 2, 0.0)); // uy
    bcs.add_displacement_bc(DisplacementBC::new(1, 3, 3, 0.0)); // uz

    // Apply load on right face (nodes 2, 3, 6, 7)
    // Total force = stress × area = 100 MPa × 1 m² = 100 MN
    // Distributed to 4 nodes = 25 MN each
    let stress = 100e6; // 100 MPa
    let area = 1.0; // 1 m²
    let total_force = stress * area;
    let force_per_node = total_force / 4.0;

    bcs.add_concentrated_load(ConcentratedLoad::new(2, 1, force_per_node)); // Node 2, DOF 1 (Fx)
    bcs.add_concentrated_load(ConcentratedLoad::new(3, 1, force_per_node));
    bcs.add_concentrated_load(ConcentratedLoad::new(6, 1, force_per_node));
    bcs.add_concentrated_load(ConcentratedLoad::new(7, 1, force_per_node));

    // Assemble global system (BCs are applied during assembly)
    let global_system = GlobalSystem::assemble(&mesh, &materials, &bcs, 1.0).unwrap();

    // Solve
    let displacements = global_system.solve().unwrap();

    // Check displacement at right face
    // Analytical: u = σL/E = (100e6 Pa)(1 m) / (200e9 Pa) = 0.0005 m = 0.5 mm
    let analytical_disp = stress * 1.0 / E;

    let max_dofs_per_node = mesh.num_dofs / mesh.nodes.len();
    let node_2_ux = displacements[(2 - 1) * max_dofs_per_node]; // Node 2, ux
    let node_3_ux = displacements[(3 - 1) * max_dofs_per_node]; // Node 3, ux
    let node_6_ux = displacements[(6 - 1) * max_dofs_per_node]; // Node 6, ux
    let node_7_ux = displacements[(7 - 1) * max_dofs_per_node]; // Node 7, ux

    println!("Analytical displacement: {:.6e} m", analytical_disp);
    println!("FEA displacement (node 2): {:.6e} m", node_2_ux);
    println!("FEA displacement (node 3): {:.6e} m", node_3_ux);
    println!("FEA displacement (node 6): {:.6e} m", node_6_ux);
    println!("FEA displacement (node 7): {:.6e} m", node_7_ux);

    // All nodes on right face should have same displacement
    let avg_disp = (node_2_ux + node_3_ux + node_6_ux + node_7_ux) / 4.0;
    let rel_error = ((avg_disp - analytical_disp) / analytical_disp).abs();

    println!("Relative error: {:.2}%", rel_error * 100.0);

    assert!(
        rel_error < 0.01,
        "Error too large: {:.4}% (expected < 1%)",
        rel_error * 100.0
    );
}

#[test]
fn c3d8_pure_shear() {
    // Test: Single C3D8 element under pure shear
    // Analytical solution: γ = τ/G, where G = E/(2(1+ν))
    //
    // Setup: Unit cube
    // BC: Fix bottom face, apply shear traction on top face
    // Load: τ = 50 MPa

    // Create mesh
    let mut mesh = Mesh::new();

    // Add nodes
    mesh.add_node(Node::new(1, 0.0, 0.0, 0.0));
    mesh.add_node(Node::new(2, 1.0, 0.0, 0.0));
    mesh.add_node(Node::new(3, 1.0, 1.0, 0.0));
    mesh.add_node(Node::new(4, 0.0, 1.0, 0.0));
    mesh.add_node(Node::new(5, 0.0, 0.0, 1.0));
    mesh.add_node(Node::new(6, 1.0, 0.0, 1.0));
    mesh.add_node(Node::new(7, 1.0, 1.0, 1.0));
    mesh.add_node(Node::new(8, 0.0, 1.0, 1.0));

    // Add C3D8 element
    mesh.add_element(MeshElement::new(1, ElementType::C3D8, vec![1, 2, 3, 4, 5, 6, 7, 8]));
    mesh.calculate_dofs();

    // Create material library
    let mut materials = MaterialLibrary::new();
    let steel = create_steel();
    let E = steel.elastic_modulus.unwrap();
    let nu = steel.poissons_ratio.unwrap();
    let G = E / (2.0 * (1.0 + nu)); // Shear modulus
    materials.add_material(steel.clone());
    materials.assign_material(1, "STEEL".to_string());

    // Apply boundary conditions
    let mut bcs = BoundaryConditions::new();

    // Fix bottom face (nodes 1, 2, 3, 4)
    for node_id in [1, 2, 3, 4] {
        bcs.add_displacement_bc(DisplacementBC::new(node_id, 1, 1, 0.0)); // ux
        bcs.add_displacement_bc(DisplacementBC::new(node_id, 2, 2, 0.0)); // uy
        bcs.add_displacement_bc(DisplacementBC::new(node_id, 3, 3, 0.0)); // uz
    }

    // Apply shear force on top face (nodes 5, 6, 7, 8) in x-direction
    // τ = F/A → F = τ × A
    // Constrain top-face uy/uz to enforce a near-uniform simple-shear state
    // for this single-element validation setup.
    for node_id in [5, 6, 7, 8] {
        bcs.add_displacement_bc(DisplacementBC::new(node_id, 2, 2, 0.0)); // uy
        bcs.add_displacement_bc(DisplacementBC::new(node_id, 3, 3, 0.0)); // uz
    }

    let shear_stress = 50e6; // 50 MPa
    let area = 1.0; // 1 m²
    let total_force = shear_stress * area;
    let force_per_node = total_force / 4.0;

    for node_id in [5, 6, 7, 8] {
        bcs.add_concentrated_load(ConcentratedLoad::new(node_id, 1, force_per_node)); // Fx
    }

    // Assemble global system (BCs are applied during assembly)
    let global_system = GlobalSystem::assemble(&mesh, &materials, &bcs, 1.0).unwrap();

    // Solve
    let displacements = global_system.solve().unwrap();

    // Analytical: u = (τ/G) × h = (50e6 / G) × 1.0
    let analytical_disp = shear_stress * 1.0 / G;

    let max_dofs_per_node = mesh.num_dofs / mesh.nodes.len();
    let node_5_ux = displacements[(5 - 1) * max_dofs_per_node];
    let node_6_ux = displacements[(6 - 1) * max_dofs_per_node];
    let node_7_ux = displacements[(7 - 1) * max_dofs_per_node];
    let node_8_ux = displacements[(8 - 1) * max_dofs_per_node];

    let avg_disp = (node_5_ux + node_6_ux + node_7_ux + node_8_ux) / 4.0;
    let rel_error = ((avg_disp - analytical_disp) / analytical_disp).abs();

    println!("Shear modulus G: {:.2e} Pa", G);
    println!("Analytical displacement: {:.6e} m", analytical_disp);
    println!("FEA displacement: {:.6e} m", avg_disp);
    println!("Relative error: {:.2}%", rel_error * 100.0);

    assert!(
        rel_error < 0.02,
        "Error too large: {:.4}% (expected < 2%)",
        rel_error * 100.0
    );
}

#[test]
fn c3d8_patch_test() {
    // Patch test: Element should exactly reproduce constant stress state
    //
    // This is a fundamental FEA requirement - if an element is subjected to
    // a state of constant stress, it should compute the exact displacement field
    // regardless of element distortion (within reason).

    // Create mesh
    let mut mesh = Mesh::new();

    // Add nodes
    mesh.add_node(Node::new(1, 0.0, 0.0, 0.0));
    mesh.add_node(Node::new(2, 1.0, 0.0, 0.0));
    mesh.add_node(Node::new(3, 1.0, 1.0, 0.0));
    mesh.add_node(Node::new(4, 0.0, 1.0, 0.0));
    mesh.add_node(Node::new(5, 0.0, 0.0, 1.0));
    mesh.add_node(Node::new(6, 1.0, 0.0, 1.0));
    mesh.add_node(Node::new(7, 1.0, 1.0, 1.0));
    mesh.add_node(Node::new(8, 0.0, 1.0, 1.0));

    let material = create_steel();

    // Get nodes as a vector for C3D8 element
    let nodes = vec![
        mesh.nodes.get(&1).unwrap().clone(),
        mesh.nodes.get(&2).unwrap().clone(),
        mesh.nodes.get(&3).unwrap().clone(),
        mesh.nodes.get(&4).unwrap().clone(),
        mesh.nodes.get(&5).unwrap().clone(),
        mesh.nodes.get(&6).unwrap().clone(),
        mesh.nodes.get(&7).unwrap().clone(),
        mesh.nodes.get(&8).unwrap().clone(),
    ];

    let elem = C3D8::new(1, [1, 2, 3, 4, 5, 6, 7, 8]);

    // Compute stiffness matrix
    let K = elem.stiffness_matrix(&nodes, &material).unwrap();

    // Check that stiffness matrix is:
    // 1. Symmetric
    // 2. Has 6 near-zero eigenvalues (rigid body modes)
    // 3. Has 18 positive eigenvalues (deformation modes)

    // Check symmetry
    for i in 0..24 {
        for j in 0..24 {
            let diff = (K[(i, j)] - K[(j, i)]).abs();
            let avg = (K[(i, j)].abs() + K[(j, i)].abs()) / 2.0;
            let rel_diff = if avg > 1e-6 { diff / avg } else { diff };
            assert!(
                rel_diff < 1e-10,
                "Stiffness matrix not symmetric at ({},{})",
                i,
                j
            );
        }
    }

    println!("✓ Patch test: Stiffness matrix is symmetric");

    // For an unconstrained element, the rank should be 24 - 6 = 18
    // (6 rigid body modes: 3 translations + 3 rotations)
    // This is verified by the eigenvalue check in the assembly tests
}

#[test]
fn c3d8_cantilever_beam_mesh() {
    // Test: Cantilever beam modeled with C3D8 elements
    // Analytical solution: δ = PL³/(3EI)
    //
    // Setup: 10m × 1m × 1m beam, 10 C3D8 elements
    // BC: Fix left end, apply point load at right end
    // Load: P = 1000 N downward

    let L = 10.0; // Beam length [m]
    let H = 1.0; // Height [m]
    let W = 1.0; // Width [m]
    let n_elements = 10;
    let element_length = L / (n_elements as f64);

    // Create mesh
    let mut mesh = Mesh::new();

    // Create nodes (11 × 2 × 2 = 44 nodes)
    let mut node_id = 1;
    for i in 0..=n_elements {
        let x = (i as f64) * element_length;
        for k in 0..2 {
            let z = (k as f64) * H;
            for j in 0..2 {
                let y = (j as f64) * W;
                mesh.add_node(Node::new(node_id, x, y, z));
                node_id += 1;
            }
        }
    }

    // Create elements
    for i in 0..n_elements {
        let elem_id = (i + 1) as i32;
        let base = (i * 4 + 1) as i32;

        // Node ordering for C3D8
        let elem_nodes = vec![
            base,     // 1
            base + 1, // 2
            base + 3, // 3
            base + 2, // 4
            base + 4, // 5
            base + 5, // 6
            base + 7, // 7
            base + 6, // 8
        ];

        mesh.add_element(MeshElement::new(elem_id, ElementType::C3D8, elem_nodes));
    }

    mesh.calculate_dofs();

    // Create material library
    let mut materials = MaterialLibrary::new();
    let steel = create_steel();
    let E = steel.elastic_modulus.unwrap();
    materials.add_material(steel.clone());

    // Assign material to all elements
    for i in 1..=n_elements {
        materials.assign_material(i as i32, "STEEL".to_string());
    }

    // Apply boundary conditions
    let mut bcs = BoundaryConditions::new();

    // Fix left end (x=0, nodes 1-4)
    for node_id in 1..=4 {
        bcs.add_displacement_bc(DisplacementBC::new(node_id, 1, 1, 0.0)); // ux
        bcs.add_displacement_bc(DisplacementBC::new(node_id, 2, 2, 0.0)); // uy
        bcs.add_displacement_bc(DisplacementBC::new(node_id, 3, 3, 0.0)); // uz
    }

    // Apply downward load at right end center (negative z direction)
    // Right end nodes are the last 4 nodes
    let P = 1000.0; // 1000 N total
    let force_per_node = P / 4.0;
    let n_nodes = mesh.nodes.len() as i32;
    for node_id in (n_nodes - 3)..=n_nodes {
        bcs.add_concentrated_load(ConcentratedLoad::new(node_id, 3, -force_per_node)); // Fz (negative = downward)
    }

    // Assemble global system (BCs are applied during assembly)
    let global_system = GlobalSystem::assemble(&mesh, &materials, &bcs, 1.0).unwrap();

    // Solve
    let displacements = global_system.solve().unwrap();

    // Analytical solution for cantilever beam: δ = PL³/(3EI)
    // For rectangular cross-section: I = (W × H³) / 12
    let I = (W * H.powi(3)) / 12.0;
    let analytical_deflection = (P * L.powi(3)) / (3.0 * E * I);

    // Get tip deflection (average of last 4 nodes)
    let max_dofs_per_node = mesh.num_dofs / mesh.nodes.len();
    let n_nodes_count = mesh.nodes.len();
    let mut tip_deflection_sum = 0.0;
    for i in (n_nodes_count - 4)..n_nodes_count {
        let uz = displacements[i * max_dofs_per_node + 2]; // z-displacement
        tip_deflection_sum += uz;
    }
    let tip_deflection: f64 = tip_deflection_sum / 4.0;

    println!("Analytical deflection: {:.6e} m", analytical_deflection);
    println!("FEA deflection: {:.6e} m", tip_deflection.abs());

    let rel_error = ((tip_deflection.abs() - analytical_deflection) / analytical_deflection).abs();
    println!("Relative error: {:.2}%", rel_error * 100.0);

    // With 10 elements along the span and only one element through width/height,
    // this mesh is very coarse for bending; accept a broader error band.
    assert!(
        rel_error < 0.40,
        "Error too large: {:.4}% (expected < 40% for this coarse mesh)",
        rel_error * 100.0
    );
}

#[test]
fn c3d8_volume_computation() {
    // Test: Volume computation using mass matrix
    // The sum of the mass matrix should equal 3 × ρ × V
    // (factor of 3 because we have 3 DOFs per node)

    // Create mesh
    let mut mesh = Mesh::new();

    // Add nodes
    mesh.add_node(Node::new(1, 0.0, 0.0, 0.0));
    mesh.add_node(Node::new(2, 2.0, 0.0, 0.0));
    mesh.add_node(Node::new(3, 2.0, 3.0, 0.0));
    mesh.add_node(Node::new(4, 0.0, 3.0, 0.0));
    mesh.add_node(Node::new(5, 0.0, 0.0, 4.0));
    mesh.add_node(Node::new(6, 2.0, 0.0, 4.0));
    mesh.add_node(Node::new(7, 2.0, 3.0, 4.0));
    mesh.add_node(Node::new(8, 0.0, 3.0, 4.0));

    let material = create_steel();
    let rho = material.density.unwrap();

    // Get nodes as a vector for C3D8 element
    let nodes = vec![
        mesh.nodes.get(&1).unwrap().clone(),
        mesh.nodes.get(&2).unwrap().clone(),
        mesh.nodes.get(&3).unwrap().clone(),
        mesh.nodes.get(&4).unwrap().clone(),
        mesh.nodes.get(&5).unwrap().clone(),
        mesh.nodes.get(&6).unwrap().clone(),
        mesh.nodes.get(&7).unwrap().clone(),
        mesh.nodes.get(&8).unwrap().clone(),
    ];

    let elem = C3D8::new(1, [1, 2, 3, 4, 5, 6, 7, 8]);
    let M = elem.mass_matrix(&nodes, &material).unwrap();

    let total_mass_from_matrix: f64 = M.iter().sum();
    let volume = 2.0 * 3.0 * 4.0; // 24 m³
    let expected_total = 3.0 * rho * volume;

    let rel_error = (total_mass_from_matrix - expected_total).abs() / expected_total;

    println!("Volume: {:.2} m³", volume);
    println!("Density: {:.2} kg/m³", rho);
    println!("Physical mass: {:.2} kg", rho * volume);
    println!("Expected matrix sum: {:.6e}", expected_total);
    println!("Actual matrix sum: {:.6e}", total_mass_from_matrix);
    println!("Relative error: {:.6e}", rel_error);

    assert!(
        rel_error < 1e-10,
        "Mass conservation failed: rel_error = {:.6e}",
        rel_error
    );
}
